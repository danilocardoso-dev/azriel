//! Autoajuste: o diário de trades vira estatística, o Gemini revisa os erros e propõe lições
//! e parâmetros, e o código só aceita mudanças pequenas e dentro de faixas seguras.

use crate::database::trading_models::{TradingTrade, TuningChange, Tunable};
use serde::Deserialize;
use serde_json::{json, Value};

/// Revisão automática a cada N trades fechados.
pub const REVIEW_EVERY: i64 = 5;
/// Abaixo disso a amostra é pequena demais para mexer em parâmetros; só lições.
pub const MIN_TRADES_FOR_PARAMS: usize = 5;
/// Operação rápida: o autoajuste só alterna entre velas de 5 e 15 minutos.
pub const TUNABLE_INTERVALS: [&str; 2] = ["5m", "15m"];
const STOP_RANGE: (f64, f64) = (0.15, 1.0);
const TARGET_RANGE: (f64, f64) = (0.25, 2.0);
const CONFIDENCE_RANGE: (i64, i64) = (55, 90);
/// Cada revisão move stop/alvo no máximo 30% e a confiança no máximo 10 pontos.
const MAX_RELATIVE_STEP: f64 = 0.3;
const MAX_CONFIDENCE_STEP: i64 = 10;
/// O alvo nunca fica menor que 1,5× o stop: com a taxa de ~0,1%, menos que isso exige acertar demais.
const MIN_REWARD_RISK: f64 = 1.5;
const MAX_LESSONS: usize = 6;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TradeStats {
    pub count: usize,
    pub wins: usize,
    pub pnl: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
}

impl TradeStats {
    pub fn win_rate(&self) -> f64 {
        if self.count == 0 { 0.0 } else { self.wins as f64 / self.count as f64 * 100.0 }
    }
}

pub fn stats(trades: &[TradingTrade]) -> TradeStats {
    let results: Vec<f64> = trades.iter().filter_map(|trade| trade.pnl_usdt).collect();
    let wins: Vec<f64> = results.iter().copied().filter(|pnl| *pnl > 0.0).collect();
    let losses: Vec<f64> = results.iter().copied().filter(|pnl| *pnl <= 0.0).collect();
    let mean = |values: &[f64]| if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 };
    TradeStats { count: results.len(), wins: wins.len(), pnl: results.iter().sum(), avg_win: mean(&wins), avg_loss: mean(&losses) }
}

/// Posição que sumiu da corretora sem o agente fechar: foi stop, alvo ou algo fora do Azriel.
/// Compara o resultado (já com taxas) com o que o stop e o alvo dariam.
pub fn exchange_exit_reason(trade: &TradingTrade, pnl: f64) -> &'static str {
    let notional = trade.quantity * trade.entry_price;
    if notional <= 0.0 {
        return if pnl < 0.0 { "STOP" } else { "TARGET" };
    }
    if pnl <= -0.6 * notional * trade.stop_loss_pct / 100.0 {
        "STOP"
    } else if pnl >= 0.6 * notional * trade.take_profit_pct / 100.0 {
        "TARGET"
    } else {
        "EXCHANGE"
    }
}

/// Preço de saída aproximado a partir do resultado (inclui as taxas).
pub fn implied_exit_price(trade: &TradingTrade, pnl: f64, fallback: f64) -> f64 {
    if trade.quantity <= 0.0 {
        return fallback;
    }
    let moved = pnl / trade.quantity;
    if trade.side == "LONG" { trade.entry_price + moved } else { trade.entry_price - moved }
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Adjustments {
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub min_confidence: Option<f64>,
    pub candle_interval: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Proposal {
    pub diagnosis: String,
    pub lessons: Vec<String>,
    pub adjustments: Adjustments,
}

fn clean(text: &str, limit: usize) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(limit).collect()
}

/// Lê a resposta do revisor, tolerando cercas ```json e campos ausentes.
pub fn parse_proposal(raw: &str) -> Result<Proposal, String> {
    #[derive(Deserialize)]
    struct Raw {
        #[serde(default)]
        diagnosis: String,
        #[serde(default)]
        lessons: Vec<Value>,
        #[serde(default)]
        adjustments: Option<Value>,
    }
    let start = raw.find('{').ok_or("O revisor não retornou um objeto JSON")?;
    let end = raw.rfind('}').ok_or("O revisor não retornou um objeto JSON")?;
    let parsed: Raw = serde_json::from_str(&raw[start..=end]).map_err(|_| "JSON da revisão inválido".to_string())?;
    let diagnosis = clean(&parsed.diagnosis, 600);
    if diagnosis.is_empty() {
        return Err("A revisão veio sem diagnóstico".into());
    }
    let lessons = parsed
        .lessons
        .iter()
        .filter_map(|lesson| lesson.as_str().map(|text| clean(text, 200)))
        .filter(|lesson| !lesson.is_empty())
        .take(MAX_LESSONS)
        .collect();
    // Números podem vir como texto ("1,2"); valores ilegíveis só viram "sem mudança".
    let number = |value: Option<&Value>| value.and_then(|v| v.as_f64().or_else(|| v.as_str().and_then(|s| s.trim().trim_end_matches('%').replace(',', ".").parse().ok())));
    let adjustments = parsed.adjustments.map(|value| Adjustments {
        stop_loss_pct: number(value.get("stopLossPct")),
        take_profit_pct: number(value.get("takeProfitPct")),
        min_confidence: number(value.get("minConfidence")),
        candle_interval: value.get("candleInterval").and_then(Value::as_str).map(str::to_string),
    });
    Ok(Proposal { diagnosis, lessons, adjustments: adjustments.unwrap_or_default() })
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// Arredonda para cima na segunda casa: a proporção mínima nunca fica abaixo por arredondamento.
fn ceil2(value: f64) -> f64 {
    (value * 100.0 - 1e-9).ceil() / 100.0
}

fn step_limited(current: f64, proposed: f64, range: (f64, f64)) -> f64 {
    let low = current * (1.0 - MAX_RELATIVE_STEP);
    let high = current * (1.0 + MAX_RELATIVE_STEP);
    round2(proposed.clamp(low.min(high), high.max(low)).clamp(range.0, range.1))
}

fn interval_minutes(interval: &str) -> i64 {
    match interval {
        "5m" => 5,
        _ => 15,
    }
}

/// Aplica as travas: faixas absolutas, passo máximo por revisão, alvo ≥ 1,5× stop e tempo
/// gráfico andando no máximo um degrau. Com amostra pequena nada muda.
pub fn apply_guardrails(current: &Tunable, adjustments: &Adjustments, sample: usize) -> (Tunable, Vec<TuningChange>) {
    if sample < MIN_TRADES_FOR_PARAMS {
        return (current.clone(), Vec::new());
    }
    let mut next = current.clone();
    if let Some(stop) = adjustments.stop_loss_pct.filter(|v| v.is_finite() && *v > 0.0) {
        next.stop_loss_pct = step_limited(current.stop_loss_pct, stop, STOP_RANGE);
    }
    if let Some(target) = adjustments.take_profit_pct.filter(|v| v.is_finite() && *v > 0.0) {
        next.take_profit_pct = step_limited(current.take_profit_pct, target, TARGET_RANGE);
    }
    if next.take_profit_pct < next.stop_loss_pct * MIN_REWARD_RISK {
        next.take_profit_pct = ceil2(next.stop_loss_pct * MIN_REWARD_RISK).min(TARGET_RANGE.1);
        // Ainda pequeno demais (stop muito largo): o stop recua para caber na proporção.
        if next.take_profit_pct < next.stop_loss_pct * MIN_REWARD_RISK - 1e-9 {
            next.stop_loss_pct = round2(next.take_profit_pct / MIN_REWARD_RISK);
        }
    }
    if let Some(confidence) = adjustments.min_confidence.filter(|v| v.is_finite()) {
        let confidence = confidence.round() as i64;
        next.min_confidence = confidence
            .clamp(current.min_confidence - MAX_CONFIDENCE_STEP, current.min_confidence + MAX_CONFIDENCE_STEP)
            .clamp(CONFIDENCE_RANGE.0, CONFIDENCE_RANGE.1);
    }
    if let Some(interval) = adjustments.candle_interval.as_deref() {
        let from = TUNABLE_INTERVALS.iter().position(|item| *item == current.candle_interval);
        let to = TUNABLE_INTERVALS.iter().position(|item| *item == interval);
        if let (Some(from), Some(to)) = (from, to) {
            let step = (to as i64 - from as i64).clamp(-1, 1);
            let chosen = TUNABLE_INTERVALS[(from as i64 + step) as usize];
            if chosen != current.candle_interval {
                next.candle_interval = chosen.to_string();
                // O ciclo acompanha o candle: decide logo depois de cada vela fechar.
                next.cycle_minutes = interval_minutes(chosen);
            }
        }
    }
    let pct = |value: f64| format!("{}%", format!("{value:.2}").replace('.', ","));
    let mut changes = Vec::new();
    let mut push = |field: &str, label: &str, from: String, to: String| {
        if from != to {
            changes.push(TuningChange { field: field.into(), label: label.into(), from, to });
        }
    };
    push("stopLossPct", "Stop loss", pct(current.stop_loss_pct), pct(next.stop_loss_pct));
    push("takeProfitPct", "Take profit", pct(current.take_profit_pct), pct(next.take_profit_pct));
    push("minConfidence", "Confiança mínima", format!("{}%", current.min_confidence), format!("{}%", next.min_confidence));
    push("candleInterval", "Tempo gráfico", current.candle_interval.clone(), next.candle_interval.clone());
    if changes.is_empty() {
        return (current.clone(), changes);
    }
    (next, changes)
}

fn trade_json(trade: &TradingTrade) -> Value {
    json!({
        "id": trade.id,
        "side": trade.side,
        "openedAt": trade.opened_at,
        "closedAt": trade.closed_at,
        "entryPrice": trade.entry_price,
        "exitPrice": trade.exit_price.map(|price| (price * 10.0).round() / 10.0),
        "pnlUsdt": trade.pnl_usdt.map(|pnl| (pnl * 1000.0).round() / 1000.0),
        "exitReason": trade.exit_reason,
        "stopLossPct": trade.stop_loss_pct,
        "takeProfitPct": trade.take_profit_pct,
        "interval": trade.candle_interval,
        "confidence": trade.confidence,
        "entryReason": trade.entry_reason,
        "entryContext": serde_json::from_str::<Value>(&trade.entry_context).unwrap_or(Value::Null),
    })
}

fn stats_json(stats: &TradeStats) -> Value {
    json!({
        "trades": stats.count,
        "wins": stats.wins,
        "winRatePct": (stats.win_rate() * 10.0).round() / 10.0,
        "pnlUsdt": (stats.pnl * 1000.0).round() / 1000.0,
        "avgWinUsdt": (stats.avg_win * 1000.0).round() / 1000.0,
        "avgLossUsdt": (stats.avg_loss * 1000.0).round() / 1000.0,
    })
}

pub struct ReviewInput<'a> {
    pub symbol: &'a str,
    pub current: &'a Tunable,
    pub margin_usdt: f64,
    pub leverage: i64,
    pub trades: &'a [TradingTrade],
    pub new_trades: usize,
    pub lessons: &'a [String],
    pub previous: &'a [(String, String, Vec<TuningChange>)],
}

/// Tudo o que o revisor precisa, já resumido: parâmetros atuais, estatística, cada trade
/// com o contexto de entrada e as revisões anteriores.
pub fn build_review_input(input: &ReviewInput) -> Value {
    let recent = &input.trades[input.trades.len().saturating_sub(input.new_trades)..];
    json!({
        "symbol": input.symbol,
        "currentParameters": {
            "stopLossPct": input.current.stop_loss_pct,
            "takeProfitPct": input.current.take_profit_pct,
            "minConfidence": input.current.min_confidence,
            "candleInterval": input.current.candle_interval,
            "marginUsdt": input.margin_usdt,
            "leverage": input.leverage,
        },
        "limits": {
            "stopLossPct": [STOP_RANGE.0, STOP_RANGE.1],
            "takeProfitPct": [TARGET_RANGE.0, TARGET_RANGE.1],
            "minConfidence": [CONFIDENCE_RANGE.0, CONFIDENCE_RANGE.1],
            "candleInterval": TUNABLE_INTERVALS,
            "maxStepPerReview": "stop e alvo ±30%, confiança ±10 pontos, tempo gráfico um degrau",
            "roundTripFeesPct": super::strategy::FEES_PCT,
            "minRewardRisk": MIN_REWARD_RISK,
            "parametersLocked": input.trades.len() < MIN_TRADES_FOR_PARAMS,
        },
        "statsAll": stats_json(&stats(input.trades)),
        "statsSinceLastReview": stats_json(&stats(recent)),
        "trades": input.trades.iter().map(trade_json).collect::<Vec<_>>(),
        "currentLessons": input.lessons,
        "previousReviews": input.previous.iter().map(|(at, diagnosis, changes)| json!({
            "at": at,
            "diagnosis": diagnosis,
            "changes": changes.iter().map(|change| format!("{}: {} → {}", change.label, change.from, change.to)).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trade(side: &str, pnl: Option<f64>) -> TradingTrade {
        TradingTrade {
            id: 1, symbol: "BTC-USDT".into(), side: side.into(), opened_at: "2026-10-29 07:05:22".into(), closed_at: pnl.map(|_| "2026-10-29 14:35:00".into()),
            entry_price: 80_000.0, exit_price: None, quantity: 0.0012, stop_loss_pct: 1.0, take_profit_pct: 2.0, candle_interval: "5m".into(),
            balance_before: 100_000.0, pnl_usdt: pnl, exit_reason: None, confidence: 85, entry_reason: "x".into(), entry_context: "{\"rsi\":40}".into(),
        }
    }

    fn tunable() -> Tunable {
        Tunable { stop_loss_pct: 0.3, take_profit_pct: 0.5, min_confidence: 60, candle_interval: "5m".into(), cycle_minutes: 5 }
    }

    #[test]
    fn stats_summarize_closed_trades() {
        let trades = [trade("SHORT", Some(0.24)), trade("SHORT", Some(-0.1)), trade("LONG", Some(-1.09)), trade("LONG", None)];
        let summary = stats(&trades);
        assert_eq!((summary.count, summary.wins), (3, 1));
        assert!((summary.pnl - -0.95).abs() < 1e-9);
        assert!((summary.win_rate() - 33.333).abs() < 0.01);
        assert!((summary.avg_loss - -0.595).abs() < 1e-9);
        assert_eq!(stats(&[]).win_rate(), 0.0);
    }

    #[test]
    fn exits_on_the_exchange_are_classified_by_the_result() {
        // Exposição de 96 USDT: stop de 1% ≈ −0,96; alvo de 2% ≈ +1,92 (dados reais de 29/10).
        let short = trade("SHORT", None);
        assert_eq!(exchange_exit_reason(&short, -1.0), "STOP");
        assert_eq!(exchange_exit_reason(&short, 1.9), "TARGET");
        assert_eq!(exchange_exit_reason(&short, -0.2), "EXCHANGE");
        assert!((implied_exit_price(&short, -0.96, 0.0) - 80_800.0).abs() < 1e-6);
        assert!((implied_exit_price(&trade("LONG", None), -0.96, 0.0) - 79_200.0).abs() < 1e-6);
    }

    #[test]
    fn proposals_are_parsed_tolerantly() {
        let raw = "```json\n{\"diagnosis\":\" Entradas   tardias \",\"lessons\":[\"Evite vender após queda forte\", 3, \"\"],\"adjustments\":{\"stopLossPct\":\"1,3\",\"minConfidence\":70,\"candleInterval\":\"15m\",\"takeProfitPct\":null}}\n```";
        let proposal = parse_proposal(raw).unwrap();
        assert_eq!(proposal.diagnosis, "Entradas tardias");
        assert_eq!(proposal.lessons, vec!["Evite vender após queda forte".to_string()]);
        assert_eq!(proposal.adjustments, Adjustments { stop_loss_pct: Some(1.3), take_profit_pct: None, min_confidence: Some(70.0), candle_interval: Some("15m".into()) });
        assert!(parse_proposal("{\"lessons\":[]}").is_err());
        assert!(parse_proposal("nada").is_err());
        let many = format!("{{\"diagnosis\":\"d\",\"lessons\":[{}]}}", vec!["\"l\""; 10].join(","));
        assert_eq!(parse_proposal(&many).unwrap().lessons.len(), MAX_LESSONS);
    }

    #[test]
    fn small_samples_never_change_parameters() {
        let wild = Adjustments { stop_loss_pct: Some(3.0), take_profit_pct: Some(8.0), min_confidence: Some(90.0), candle_interval: Some("1h".into()) };
        let (next, changes) = apply_guardrails(&tunable(), &wild, MIN_TRADES_FOR_PARAMS - 1);
        assert_eq!(next, tunable());
        assert!(changes.is_empty());
    }

    #[test]
    fn guardrails_limit_each_step() {
        let wild = Adjustments { stop_loss_pct: Some(3.0), take_profit_pct: Some(8.0), min_confidence: Some(95.0), candle_interval: Some("15m".into()) };
        let (next, changes) = apply_guardrails(&tunable(), &wild, 10);
        assert_eq!(next, Tunable { stop_loss_pct: 0.39, take_profit_pct: 0.65, min_confidence: 70, candle_interval: "15m".into(), cycle_minutes: 15 });
        assert_eq!(changes.len(), 4);
        assert_eq!(changes[0], TuningChange { field: "stopLossPct".into(), label: "Stop loss".into(), from: "0,30%".into(), to: "0,39%".into() });

        let tiny = Adjustments { stop_loss_pct: Some(0.01), min_confidence: Some(10.0), ..Default::default() };
        let (next, _) = apply_guardrails(&tunable(), &tiny, 10);
        assert_eq!((next.stop_loss_pct, next.min_confidence), (0.21, 55));
    }

    #[test]
    fn target_always_pays_at_least_one_and_a_half_times_the_stop() {
        let widen_stop = Adjustments { stop_loss_pct: Some(0.39), take_profit_pct: Some(0.45), ..Default::default() };
        let (next, _) = apply_guardrails(&tunable(), &widen_stop, 10);
        assert_eq!((next.stop_loss_pct, next.take_profit_pct), (0.39, 0.59));
        let edge = Tunable { stop_loss_pct: 1.0, take_profit_pct: 1.6, ..tunable() };
        let (next, _) = apply_guardrails(&edge, &Adjustments { take_profit_pct: Some(1.2), ..Default::default() }, 10);
        assert!(next.take_profit_pct >= next.stop_loss_pct * MIN_REWARD_RISK - 1e-9);
    }

    #[test]
    fn no_op_proposals_change_nothing() {
        let same = Adjustments { stop_loss_pct: Some(0.3), take_profit_pct: Some(0.5), min_confidence: Some(60.0), candle_interval: Some("5m".into()) };
        let (next, changes) = apply_guardrails(&tunable(), &same, 10);
        assert_eq!(next, tunable());
        assert!(changes.is_empty());
        let (_, changes) = apply_guardrails(&tunable(), &Adjustments { candle_interval: Some("3m".into()), ..Default::default() }, 10);
        assert!(changes.is_empty(), "tempo gráfico fora da lista é ignorado");
    }

    #[test]
    fn review_input_carries_stats_trades_and_limits() {
        let trades = [trade("SHORT", Some(0.24)), trade("LONG", Some(-1.0))];
        let current = tunable();
        let input = build_review_input(&ReviewInput {
            symbol: "BTC-USDT", current: &current, margin_usdt: 20.0, leverage: 5, trades: &trades, new_trades: 1,
            lessons: &["l1".into()], previous: &[],
        });
        assert_eq!(input["statsAll"]["trades"], 2);
        assert_eq!(input["statsSinceLastReview"]["trades"], 1);
        assert_eq!(input["statsSinceLastReview"]["wins"], 0);
        assert_eq!(input["trades"][0]["entryContext"]["rsi"], 40);
        assert_eq!(input["limits"]["parametersLocked"], true);
        assert_eq!(input["currentLessons"][0], "l1");
    }
}
