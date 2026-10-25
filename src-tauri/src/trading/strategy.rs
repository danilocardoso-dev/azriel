//! Contexto enviado à IA, leitura da decisão e limites de risco. Tudo puro e testável:
//! a IA propõe, o código decide se a proposta pode ser executada.

use super::indicators::{atr, ema, relative_volume, rsi, Candle};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    OpenLong,
    OpenShort,
    Close,
    Hold,
}

impl Action {
    pub fn as_str(self) -> &'static str {
        match self {
            Action::OpenLong => "OPEN_LONG",
            Action::OpenShort => "OPEN_SHORT",
            Action::Close => "CLOSE",
            Action::Hold => "HOLD",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Decision {
    pub action: Action,
    pub confidence: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenPosition {
    pub side: Side,
    pub quantity: f64,
    pub entry_price: f64,
    pub unrealized: f64,
    pub leverage: i64,
}

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub min_confidence: i64,
    pub max_daily_loss_pct: f64,
    pub max_trades_per_day: i64,
}

#[derive(Debug, Clone)]
pub struct RiskState {
    pub position: Option<Side>,
    pub equity: f64,
    pub day_start_equity: Option<f64>,
    pub trades_today: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RiskOutcome {
    pub action: Action,
    pub note: String,
    /// Perda diária atingida: o motor deve fechar o que estiver aberto e pausar.
    pub halt: bool,
}

pub fn daily_loss_pct(state: &RiskState) -> f64 {
    match state.day_start_equity {
        Some(start) if start > 0.0 => ((start - state.equity) / start * 100.0).max(0.0),
        _ => 0.0,
    }
}

pub fn apply_risk(decision: &Decision, state: &RiskState, config: &RiskConfig) -> RiskOutcome {
    let outcome = |action: Action, note: &str| RiskOutcome { action, note: note.into(), halt: false };
    let loss = daily_loss_pct(state);
    if loss >= config.max_daily_loss_pct {
        return RiskOutcome {
            action: if state.position.is_some() { Action::Close } else { Action::Hold },
            note: format!("Perda diária de {loss:.2}% atingiu o limite de {:.2}%. Motor pausado.", config.max_daily_loss_pct),
            halt: true,
        };
    }
    match (decision.action, state.position) {
        (Action::Hold, _) => outcome(Action::Hold, ""),
        (Action::Close, None) => outcome(Action::Hold, "Sem posição aberta para fechar."),
        (Action::Close, Some(_)) => outcome(Action::Close, ""),
        (Action::OpenLong, Some(Side::Short)) | (Action::OpenShort, Some(Side::Long)) => {
            outcome(Action::Close, "Sinal contrário à posição: apenas fecha, sem inverter no mesmo ciclo.")
        }
        (Action::OpenLong | Action::OpenShort, Some(_)) => outcome(Action::Hold, "Já existe posição nessa direção."),
        (Action::OpenLong | Action::OpenShort, None) if decision.confidence < config.min_confidence => {
            outcome(Action::Hold, &format!("Confiança {}% abaixo do mínimo de {}%.", decision.confidence, config.min_confidence))
        }
        (Action::OpenLong | Action::OpenShort, None) if state.trades_today >= config.max_trades_per_day => {
            outcome(Action::Hold, &format!("Limite de {} entradas por dia atingido.", config.max_trades_per_day))
        }
        (action, None) => outcome(action, ""),
    }
}

/// Lê a resposta da IA, tolerando cercas ```json e texto em volta do objeto.
pub fn parse_decision(raw: &str) -> Result<Decision, String> {
    #[derive(Deserialize)]
    struct RawDecision {
        action: String,
        #[serde(default)]
        confidence: Value,
        #[serde(default)]
        reason: String,
    }
    let start = raw.find('{').ok_or("A IA não retornou um objeto JSON")?;
    let end = raw.rfind('}').ok_or("A IA não retornou um objeto JSON")?;
    let parsed: RawDecision = serde_json::from_str(&raw[start..=end]).map_err(|_| "JSON de decisão inválido".to_string())?;
    let action = match parsed.action.trim().to_ascii_uppercase().replace([' ', '-'], "_").as_str() {
        "OPEN_LONG" | "LONG" | "BUY" => Action::OpenLong,
        "OPEN_SHORT" | "SHORT" | "SELL" => Action::OpenShort,
        "CLOSE" | "EXIT" => Action::Close,
        "HOLD" | "WAIT" => Action::Hold,
        other => return Err(format!("Ação desconhecida da IA: {}", other.chars().take(30).collect::<String>())),
    };
    let confidence = parsed
        .confidence
        .as_f64()
        .or_else(|| parsed.confidence.as_str().and_then(|text| text.trim_end_matches('%').trim().parse().ok()))
        .unwrap_or(0.0)
        .clamp(0.0, 100.0)
        .round() as i64;
    let reason = parsed.reason.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(240).collect();
    Ok(Decision { action, confidence, reason })
}

/// Quantidade para `margin * leverage` de exposição, arredondada para baixo na precisão do contrato.
pub fn order_quantity(margin_usdt: f64, leverage: i64, price: f64, precision: u32, min_quantity: f64, min_notional: f64) -> Result<f64, String> {
    if price <= 0.0 || margin_usdt <= 0.0 || leverage <= 0 {
        return Err("Parâmetros de tamanho de ordem inválidos".into());
    }
    let factor = 10f64.powi(precision as i32);
    // O epsilon evita que 0.002 vire 0.0019 por erro de ponto flutuante.
    let quantity = (margin_usdt * leverage as f64 / price * factor + 1e-9).floor() / factor;
    if quantity <= 0.0 || quantity < min_quantity {
        return Err(format!("Quantidade {quantity} abaixo do mínimo do contrato ({min_quantity}). Aumente a margem."));
    }
    if quantity * price < min_notional {
        return Err(format!("Valor da ordem abaixo do mínimo do contrato ({min_notional} USDT). Aumente a margem."));
    }
    Ok(quantity)
}

/// Preços de stop loss e take profit a partir da entrada, arredondados na precisão do contrato.
pub fn protection_prices(side: Side, price: f64, stop_loss_pct: f64, take_profit_pct: f64, precision: u32) -> (f64, f64) {
    let round = |value: f64| {
        let factor = 10f64.powi(precision as i32);
        (value * factor).round() / factor
    };
    match side {
        Side::Long => (round(price * (1.0 - stop_loss_pct / 100.0)), round(price * (1.0 + take_profit_pct / 100.0))),
        Side::Short => (round(price * (1.0 + stop_loss_pct / 100.0)), round(price * (1.0 - take_profit_pct / 100.0))),
    }
}

pub struct ContextInput<'a> {
    pub symbol: &'a str,
    pub interval: &'a str,
    pub candles: &'a [Candle],
    pub hourly: &'a [Candle],
    pub position: Option<&'a OpenPosition>,
    pub equity: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub leverage: i64,
    pub margin_usdt: f64,
    pub recent_decisions: &'a [(String, String, String)],
}

fn round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    (value * factor).round() / factor
}

/// Snapshot compacto do mercado e da conta: indicadores já calculados para o modelo não fazer conta.
pub fn build_context(input: &ContextInput) -> Result<Value, String> {
    let closes: Vec<f64> = input.candles.iter().map(|c| c.close).collect();
    let price = *closes.last().ok_or("Sem candles para analisar")?;
    if closes.len() < 60 {
        return Err("Histórico de candles insuficiente para os indicadores".into());
    }
    let pct = |value: Option<f64>| value.map(|v| round((price - v) / v * 100.0, 3));
    let hourly_closes: Vec<f64> = input.hourly.iter().map(|c| c.close).collect();
    let digits = if price >= 1000.0 { 1 } else if price >= 1.0 { 4 } else { 6 };
    let recent: Vec<Value> = input.candles[input.candles.len().saturating_sub(12)..]
        .iter()
        .map(|c| json!([round(c.open, digits), round(c.high, digits), round(c.low, digits), round(c.close, digits), round(c.volume, 2)]))
        .collect();
    let position = match input.position {
        None => json!({"state": "FLAT"}),
        Some(position) => json!({
            "state": if position.side == Side::Long { "LONG" } else { "SHORT" },
            "entryPrice": round(position.entry_price, digits),
            "unrealizedUsdt": round(position.unrealized, 2),
            "movePct": round(match position.side {
                Side::Long => (price - position.entry_price) / position.entry_price * 100.0,
                Side::Short => (position.entry_price - price) / position.entry_price * 100.0,
            }, 3),
            "leverage": position.leverage,
        }),
    };
    Ok(json!({
        "symbol": input.symbol,
        "interval": input.interval,
        "price": round(price, digits),
        "indicators": {
            "priceVsEma9Pct": pct(ema(&closes, 9)),
            "priceVsEma21Pct": pct(ema(&closes, 21)),
            "priceVsEma50Pct": pct(ema(&closes, 50)),
            "ema9AboveEma21": ema(&closes, 9).zip(ema(&closes, 21)).map(|(fast, slow)| fast > slow),
            "rsi14": rsi(&closes, 14).map(|v| round(v, 1)),
            "atr14Pct": atr(input.candles, 14).map(|v| round(v / price * 100.0, 3)),
            "relativeVolume": relative_volume(input.candles, 20).map(|v| round(v, 2)),
            "change1hPct": closes.len().checked_sub(5).map(|i| round((price - closes[i]) / closes[i] * 100.0, 3)),
        },
        "trend1h": {
            "ema21AboveEma50": ema(&hourly_closes, 21).zip(ema(&hourly_closes, 50)).map(|(fast, slow)| fast > slow),
            "rsi14": rsi(&hourly_closes, 14).map(|v| round(v, 1)),
        },
        "lastCandlesOHLCV": recent,
        "position": position,
        "account": {"equityUsdt": round(input.equity, 2)},
        "riskRules": {
            "marginPerTradeUsdt": input.margin_usdt,
            "leverage": input.leverage,
            "stopLossPct": input.stop_loss_pct,
            "takeProfitPct": input.take_profit_pct,
        },
        "recentDecisions": input.recent_decisions.iter().map(|(at, action, reason)| json!({"at": at, "action": action, "reason": reason})).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RISK: RiskConfig = RiskConfig { min_confidence: 60, max_daily_loss_pct: 5.0, max_trades_per_day: 10 };

    fn state(position: Option<Side>) -> RiskState {
        RiskState { position, equity: 1000.0, day_start_equity: Some(1000.0), trades_today: 0 }
    }
    fn decision(action: Action, confidence: i64) -> Decision {
        Decision { action, confidence, reason: "x".into() }
    }

    #[test]
    fn risk_rules_filter_the_ai_proposal() {
        assert_eq!(apply_risk(&decision(Action::OpenLong, 80), &state(None), &RISK).action, Action::OpenLong);
        assert_eq!(apply_risk(&decision(Action::OpenLong, 40), &state(None), &RISK).action, Action::Hold);
        assert_eq!(apply_risk(&decision(Action::OpenLong, 90), &state(Some(Side::Long)), &RISK).action, Action::Hold);
        assert_eq!(apply_risk(&decision(Action::OpenShort, 90), &state(Some(Side::Long)), &RISK).action, Action::Close);
        assert_eq!(apply_risk(&decision(Action::Close, 10), &state(None), &RISK).action, Action::Hold);
        assert_eq!(apply_risk(&decision(Action::Close, 10), &state(Some(Side::Short)), &RISK).action, Action::Close);
        let mut busy = state(None);
        busy.trades_today = 10;
        assert_eq!(apply_risk(&decision(Action::OpenShort, 95), &busy, &RISK).action, Action::Hold);
    }

    #[test]
    fn daily_loss_limit_closes_and_halts() {
        let mut losing = state(Some(Side::Long));
        losing.equity = 940.0;
        let outcome = apply_risk(&decision(Action::OpenLong, 99), &losing, &RISK);
        assert_eq!(outcome.action, Action::Close);
        assert!(outcome.halt);
        losing.position = None;
        assert_eq!(apply_risk(&decision(Action::OpenLong, 99), &losing, &RISK).action, Action::Hold);
        losing.equity = 990.0;
        assert!(!apply_risk(&decision(Action::OpenLong, 99), &losing, &RISK).halt);
    }

    #[test]
    fn decisions_are_parsed_tolerantly() {
        let parsed = parse_decision("```json\n{\"action\":\"open_long\",\"confidence\":\"72%\",\"reason\":\"EMA9 acima da EMA21\"}\n```").unwrap();
        assert_eq!(parsed, Decision { action: Action::OpenLong, confidence: 72, reason: "EMA9 acima da EMA21".into() });
        assert_eq!(parse_decision(r#"{"action":"HOLD","confidence":150}"#).unwrap().confidence, 100);
        assert!(parse_decision("sem json").is_err());
        assert!(parse_decision(r#"{"action":"YOLO"}"#).is_err());
    }

    #[test]
    fn order_size_and_protection_respect_contract_rules() {
        assert_eq!(order_quantity(20.0, 5, 50_000.0, 4, 0.0001, 2.0).unwrap(), 0.002);
        assert!(order_quantity(1.0, 1, 50_000.0, 4, 0.0001, 2.0).is_err());
        assert!(order_quantity(1.0, 1, 50_000.0, 6, 0.000001, 2.0).is_err());
        assert_eq!(protection_prices(Side::Long, 100.0, 1.0, 2.0, 2), (99.0, 102.0));
        assert_eq!(protection_prices(Side::Short, 100.0, 1.0, 2.0, 2), (101.0, 98.0));
    }

    #[test]
    fn context_has_precomputed_indicators_and_position() {
        let candles: Vec<Candle> = (0..80).map(|i| Candle { open_time: i, open: 100.0 + i as f64, high: 101.0 + i as f64, low: 99.0 + i as f64, close: 100.5 + i as f64, volume: 10.0 }).collect();
        let position = OpenPosition { side: Side::Long, quantity: 1.0, entry_price: 170.0, unrealized: 9.5, leverage: 5 };
        let context = build_context(&ContextInput {
            symbol: "BTC-USDT", interval: "15m", candles: &candles, hourly: &candles, position: Some(&position),
            equity: 1000.0, stop_loss_pct: 1.0, take_profit_pct: 2.0, leverage: 5, margin_usdt: 20.0, recent_decisions: &[],
        }).unwrap();
        assert_eq!(context["indicators"]["ema9AboveEma21"], true);
        assert_eq!(context["position"]["state"], "LONG");
        assert_eq!(context["lastCandlesOHLCV"].as_array().unwrap().len(), 12);
        assert!(build_context(&ContextInput { candles: &candles[..10], ..ContextInput {
            symbol: "BTC-USDT", interval: "15m", candles: &candles, hourly: &candles, position: None,
            equity: 1000.0, stop_loss_pct: 1.0, take_profit_pct: 2.0, leverage: 5, margin_usdt: 20.0, recent_decisions: &[],
        } }).is_err());
    }
}
