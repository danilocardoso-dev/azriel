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
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
}

#[derive(Debug, Clone)]
pub struct RiskState {
    pub position: Option<Side>,
    pub equity: f64,
    pub day_start_equity: Option<f64>,
    pub trades_today: i64,
    /// Lado que a tendência de 1h favorece; `None` quando neutra.
    pub trend: Option<Side>,
    /// ATR(14) da vela operada, em % do preço.
    pub atr_pct: Option<f64>,
    /// Distância do preço até a EMA21, em % (positivo = acima).
    pub price_vs_ema21_pct: Option<f64>,
    /// Quanto a posição aberta andou a favor, em %.
    pub position_move_pct: Option<f64>,
    /// EMA9 acima da EMA21 na vela operada (curto prazo virado para cima).
    pub ema9_above_ema21: Option<bool>,
    /// Menor e maior RSI(14) das últimas velas: mostra se o preço veio de sobrevenda/sobrecompra.
    pub rsi_recent_low: Option<f64>,
    pub rsi_recent_high: Option<f64>,
}

/// Tendência de 1h no formato do alinhamento (ALTA/BAIXA/NEUTRO) → lado que ela favorece.
pub fn trend_side(trend_1h: &str) -> Option<Side> {
    match trend_1h {
        "ALTA" => Some(Side::Long),
        "BAIXA" => Some(Side::Short),
        _ => None,
    }
}

/// Contra a tendência de 1h só com convicção alta e sinal de reversão.
pub const COUNTER_TREND_MIN_CONFIDENCE: i64 = 85;
/// Velas olhadas para dizer que o RSI "veio de" sobrevenda ou sobrecompra.
pub const RSI_LOOKBACK: usize = 6;

/// Entrada mais longe que isso (em ATRs) da EMA21 é perseguir preço: o repique costuma vir contra.
pub const MAX_EXTENSION_ATR: f64 = 2.0;
/// O alvo precisa caber em ~6 velas de ATR; abaixo disso o mercado está parado demais para pagar taxa.
pub const TARGET_IN_ATRS: f64 = 6.0;

/// Taxa de ida e volta (taker 0,05% na entrada e na saída), em % do preço.
pub const FEES_PCT: f64 = 0.1;

/// Regras de saída rápida, checadas pelo vigia a cada poucos segundos, sem esperar a IA.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExitRules {
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    /// Depois de o trade chegar a este ganho, ele não pode mais virar perda (0 desliga).
    pub profit_lock_pct: f64,
    pub max_hold_minutes: i64,
}

/// Quanto o preço andou a favor da posição, em %.
pub fn move_pct(side: Side, entry: f64, price: f64) -> f64 {
    if entry <= 0.0 {
        return 0.0;
    }
    match side {
        Side::Long => (price - entry) / entry * 100.0,
        Side::Short => (entry - price) / entry * 100.0,
    }
}

/// Decide se o trade sai agora. `peak` é o melhor movimento já visto desde a entrada.
/// Devolve o motivo (para o diário) e a explicação (para o histórico).
pub fn quick_exit(moved: f64, peak: f64, minutes_open: i64, rules: &ExitRules) -> Option<(&'static str, String)> {
    let fmt = |value: f64| format!("{value:+.2}%").replace('.', ",");
    if moved >= rules.take_profit_pct {
        return Some(("TARGET", format!("Alvo atingido: {} desde a entrada.", fmt(moved))));
    }
    if moved <= -rules.stop_loss_pct {
        return Some(("STOP", format!("Stop atingido: {} desde a entrada.", fmt(moved))));
    }
    // A trava fecha ainda no positivo depois da taxa: o lucro que apareceu não vira perda.
    let floor = FEES_PCT + 0.02;
    if rules.profit_lock_pct > 0.0 && peak >= rules.profit_lock_pct && moved <= floor {
        return Some(("LOCK", format!("Trava de lucro: chegou a {} e voltou para {}; fecha antes de virar perda.", fmt(peak), fmt(moved))));
    }
    if minutes_open >= rules.max_hold_minutes {
        return Some(("TIMEOUT", format!("Tempo máximo de {} min: sai com {}.", rules.max_hold_minutes, fmt(moved))));
    }
    None
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
    // Fechar no meio do ruído só paga taxa (dados de 01–02/11: seis saídas por "estagnação",
    // todas com −0,1 a −0,2 USDT). Nessa faixa quem decide é o vigia: stop, trava e tempo máximo.
    let small_wiggle = |moved: f64| moved > -config.stop_loss_pct / 2.0 && moved < FEES_PCT + 0.05;
    let fmt = |value: f64| format!("{value:+.2}%").replace('.', ",");
    match (decision.action, state.position) {
        (Action::Hold, _) => outcome(Action::Hold, ""),
        (Action::Close, None) => outcome(Action::Hold, "Sem posição aberta para fechar."),
        (Action::Close, Some(side)) if state.position_move_pct.is_some_and(small_wiggle) && state.trend != Some(opposite(side)) => outcome(
            Action::Hold,
            &format!("Saída bloqueada: o trade está em {} (oscilação pequena). Stop, trava de lucro e tempo máximo cuidam da saída.", fmt(state.position_move_pct.unwrap_or(0.0))),
        ),
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
        (action @ (Action::OpenLong | Action::OpenShort), None) => {
            let side = if action == Action::OpenLong { Side::Long } else { Side::Short };
            match entry_block(side, decision.confidence, state, config) {
                Some(note) => outcome(Action::Hold, &note),
                None => outcome(action, ""),
            }
        }
    }
}

fn opposite(side: Side) -> Side {
    if side == Side::Long { Side::Short } else { Side::Long }
}

/// Travas de entrada que não dependem da IA. Contra a tendência de 1h só passa uma reversão
/// clara: confiança alta, RSI vindo do extremo e curto prazo já virado a favor. Em qualquer
/// lado: sem perseguir preço esticado e só com volatilidade suficiente para o alvo pagar a taxa.
pub fn entry_block(side: Side, confidence: i64, state: &RiskState, config: &RiskConfig) -> Option<String> {
    let name = if side == Side::Long { "compra" } else { "venda" };
    if state.trend == Some(opposite(side)) {
        let (rsi_came_from_extreme, short_term_turned) = match side {
            Side::Long => (state.rsi_recent_low.is_some_and(|low| low < 30.0), state.ema9_above_ema21 == Some(true)),
            Side::Short => (state.rsi_recent_high.is_some_and(|high| high > 70.0), state.ema9_above_ema21 == Some(false)),
        };
        let extreme = if side == Side::Long { "sobrevenda (RSI abaixo de 30)" } else { "sobrecompra (RSI acima de 70)" };
        if confidence < COUNTER_TREND_MIN_CONFIDENCE {
            return Some(format!("Entrada bloqueada: {name} contra a tendência de 1h pede confiança de pelo menos {COUNTER_TREND_MIN_CONFIDENCE}% (veio {confidence}%)."));
        }
        if !rsi_came_from_extreme {
            return Some(format!("Entrada bloqueada: {name} contra a tendência de 1h sem reversão; o RSI não veio de {extreme} nas últimas {RSI_LOOKBACK} velas."));
        }
        if !short_term_turned {
            return Some(format!("Entrada bloqueada: {name} contra a tendência de 1h com o curto prazo ainda contra (EMA9 x EMA21)."));
        }
    }
    let atr = state.atr_pct?;
    if atr * TARGET_IN_ATRS < config.take_profit_pct {
        return Some(format!("Entrada bloqueada: mercado parado (ATR {:.3}%); o alvo de {:.2}% não cabe no movimento atual.", atr, config.take_profit_pct));
    }
    let extension = state.price_vs_ema21_pct.map(|distance| if side == Side::Long { distance } else { -distance });
    if extension.is_some_and(|distance| distance > MAX_EXTENSION_ATR * atr) {
        return Some(format!("Entrada bloqueada: preço já esticado ({:.2}% da EMA21, mais de {MAX_EXTENSION_ATR} ATRs). Espere o respiro.", extension.unwrap_or(0.0)));
    }
    None
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
    /// Lições da última revisão dos trades (autoajuste).
    pub lessons: &'a [String],
    pub exit_rules: ExitRules,
    /// Minutos desde a entrada da posição aberta, pelo diário.
    pub minutes_open: Option<i64>,
}

fn round(value: f64, digits: i32) -> f64 {
    let factor = 10f64.powi(digits);
    (value * factor).round() / factor
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Alignment {
    pub short_term: &'static str,
    pub trend_1h: &'static str,
    pub signal: &'static str,
}

/// Direção do mercado calculada no código, igual para os dois lados:
/// curto prazo pelas EMAs 9/21 e pelo preço contra a EMA21; tendência pelas EMAs 9/21 do 1h.
pub fn alignment(closes: &[f64], hourly_closes: &[f64]) -> Alignment {
    let price = closes.last().copied().unwrap_or_default();
    let short_term = match (ema(closes, 9), ema(closes, 21)) {
        (Some(fast), Some(slow)) if fast > slow && price > slow => "ALTA",
        (Some(fast), Some(slow)) if fast < slow && price < slow => "BAIXA",
        _ => "NEUTRO",
    };
    // EMAs 9/21 no 1h: as 21/50 demoravam dias para virar e deixavam o agente comprando na queda.
    let trend_1h = match (ema(hourly_closes, 9), ema(hourly_closes, 21)) {
        (Some(fast), Some(slow)) if fast > slow => "ALTA",
        (Some(fast), Some(slow)) if fast < slow => "BAIXA",
        _ => "NEUTRO",
    };
    let signal = if short_term == trend_1h && short_term != "NEUTRO" { short_term } else { "MISTO" };
    Alignment { short_term, trend_1h, signal }
}

/// Menor e maior RSI(14) nas últimas RSI_LOOKBACK velas fechadas.
pub fn recent_rsi(closes: &[f64]) -> Option<(f64, f64)> {
    let values: Vec<f64> = (0..RSI_LOOKBACK).filter_map(|back| closes.len().checked_sub(back).and_then(|end| rsi(&closes[..end], 14))).collect();
    let low = values.iter().copied().reduce(f64::min)?;
    let high = values.iter().copied().reduce(f64::max)?;
    Some((low, high))
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
            "minutesOpen": input.minutes_open,
        }),
    };
    let direction = alignment(&closes, &hourly_closes);
    Ok(json!({
        "symbol": input.symbol,
        "interval": input.interval,
        "price": round(price, digits),
        "alignment": {
            "signal": direction.signal,
            "shortTerm": direction.short_term,
            "trend1h": direction.trend_1h,
        },
        "indicators": {
            "priceVsEma9Pct": pct(ema(&closes, 9)),
            "priceVsEma21Pct": pct(ema(&closes, 21)),
            "priceVsEma50Pct": pct(ema(&closes, 50)),
            "ema9AboveEma21": ema(&closes, 9).zip(ema(&closes, 21)).map(|(fast, slow)| fast > slow),
            "rsi14": rsi(&closes, 14).map(|v| round(v, 1)),
            "rsiRecentLow": recent_rsi(&closes).map(|(low, _)| round(low, 1)),
            "rsiRecentHigh": recent_rsi(&closes).map(|(_, high)| round(high, 1)),
            "atr14Pct": atr(input.candles, 14).map(|v| round(v / price * 100.0, 3)),
            "relativeVolume": relative_volume(input.candles, 20).map(|v| round(v, 2)),
            "change1hPct": closes.len().checked_sub(5).map(|i| round((price - closes[i]) / closes[i] * 100.0, 3)),
        },
        "trend1h": {
            "ema9AboveEma21": ema(&hourly_closes, 9).zip(ema(&hourly_closes, 21)).map(|(fast, slow)| fast > slow),
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
            "profitLockPct": input.exit_rules.profit_lock_pct,
            "maxHoldMinutes": input.exit_rules.max_hold_minutes,
            "roundTripFeesPct": FEES_PCT,
        },
        "lessons": input.lessons,
        "recentDecisions": input.recent_decisions.iter().map(|(at, action, reason)| json!({"at": at, "action": action, "reason": reason})).collect::<Vec<_>>(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RISK: RiskConfig = RiskConfig { min_confidence: 60, max_daily_loss_pct: 5.0, max_trades_per_day: 10, stop_loss_pct: 0.3, take_profit_pct: 0.5 };

    fn state(position: Option<Side>) -> RiskState {
        RiskState {
            position, equity: 1000.0, day_start_equity: Some(1000.0), trades_today: 0, trend: None, atr_pct: None, price_vs_ema21_pct: None,
            position_move_pct: None, ema9_above_ema21: None, rsi_recent_low: None, rsi_recent_high: None,
        }
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

    /// Snapshot de entrada como os do diário: tendência, ATR e distância da EMA21.
    fn flat(trend: &str, atr: f64, vs_ema21: f64) -> RiskState {
        RiskState { trend: trend_side(trend), atr_pct: Some(atr), price_vs_ema21_pct: Some(vs_ema21), ..state(None) }
    }

    #[test]
    fn entries_follow_the_trend_and_skip_chasing_and_dead_markets() {
        let long = decision(Action::OpenLong, 80);
        let short = decision(Action::OpenShort, 80);
        // Trade 14 (01/11 22:25): compra com 1h em BAIXA, sem reversão, perdeu.
        let blocked = apply_risk(&long, &flat("BAIXA", 0.164, 0.1), &RISK);
        assert_eq!(blocked.action, Action::Hold);
        assert!(blocked.note.contains("pelo menos 85%"));
        // A queda de 02/11 19h: venda a favor do 1h, com movimento e sem esticar.
        assert_eq!(apply_risk(&short, &flat("BAIXA", 0.19, -0.116), &RISK).action, Action::OpenShort);
        // Trade 12 (ATR 0,062%): mercado parado demais para o alvo de 0,5%.
        assert!(apply_risk(&long, &flat("ALTA", 0.062, 0.112), &RISK).note.contains("mercado parado"));
        // Trade 18: compra 0,427% acima da EMA21 com ATR 0,168% (2,5 ATRs) — perseguindo preço.
        assert!(apply_risk(&long, &flat("ALTA", 0.168, 0.427), &RISK).note.contains("esticado"));
        // O mesmo esticamento a favor de uma venda (abaixo da EMA21) também bloqueia; acima não.
        assert!(apply_risk(&short, &flat("BAIXA", 0.168, -0.427), &RISK).note.contains("esticado"));
        assert_eq!(apply_risk(&short, &flat("BAIXA", 0.168, 0.2), &RISK).action, Action::OpenShort);
        // Tendência neutra libera os dois lados; sem ATR não há como medir, então não trava.
        assert_eq!(apply_risk(&long, &flat("NEUTRO", 0.15, 0.1), &RISK).action, Action::OpenLong);
        assert_eq!(apply_risk(&long, &state(None), &RISK).action, Action::OpenLong);
    }

    #[test]
    fn counter_trend_entries_need_a_clear_reversal() {
        // Compra na queda: RSI veio de 24, EMA9 voltou para cima da EMA21 e confiança 88 — passa.
        let reversal = RiskState { ema9_above_ema21: Some(true), rsi_recent_low: Some(24.0), rsi_recent_high: Some(48.0), ..flat("BAIXA", 0.15, 0.05) };
        assert_eq!(apply_risk(&decision(Action::OpenLong, 88), &reversal, &RISK).action, Action::OpenLong);
        // Faltando qualquer peça, segura.
        assert!(apply_risk(&decision(Action::OpenLong, 80), &reversal, &RISK).note.contains("85%"));
        let no_oversold = RiskState { rsi_recent_low: Some(38.0), ..reversal.clone() };
        assert!(apply_risk(&decision(Action::OpenLong, 90), &no_oversold, &RISK).note.contains("sobrevenda"));
        let still_down = RiskState { ema9_above_ema21: Some(false), ..reversal.clone() };
        assert!(apply_risk(&decision(Action::OpenLong, 90), &still_down, &RISK).note.contains("curto prazo"));
        // O espelho para vender na alta.
        let top = RiskState { ema9_above_ema21: Some(false), rsi_recent_low: Some(55.0), rsi_recent_high: Some(76.0), ..flat("ALTA", 0.15, -0.05) };
        assert_eq!(apply_risk(&decision(Action::OpenShort, 86), &top, &RISK).action, Action::OpenShort);
        // A favor da tendência, nada disso é exigido.
        assert_eq!(apply_risk(&decision(Action::OpenShort, 65), &flat("BAIXA", 0.15, -0.05), &RISK).action, Action::OpenShort);
    }

    #[test]
    fn recent_rsi_spans_the_last_candles() {
        let mut closes: Vec<f64> = (0..60).map(|i| 100.0 - i as f64 * 0.5).collect();
        closes.extend([71.0, 72.0, 73.0]);
        let (low, high) = recent_rsi(&closes).unwrap();
        assert!(low < 30.0 && high > low);
        assert_eq!(recent_rsi(&closes[..10]), None);
    }

    #[test]
    fn the_agent_cannot_close_on_small_wiggles() {
        let close = decision(Action::Close, 70);
        let at = |moved: f64, trend: &str| RiskState { position_move_pct: Some(moved), trend: trend_side(trend), ..state(Some(Side::Long)) };
        // Trade 14 fechado por "estagnação" em −0,005%: agora segura, o vigia decide.
        let held = apply_risk(&close, &at(-0.005, "ALTA"), &RISK);
        assert_eq!(held.action, Action::Hold);
        assert!(held.note.contains("oscilação pequena"));
        assert_eq!(apply_risk(&close, &at(0.2, "ALTA"), &RISK).action, Action::Close, "lucro acima da taxa pode ser realizado");
        assert_eq!(apply_risk(&close, &at(-0.2, "ALTA"), &RISK).action, Action::Close, "perda passando de meio stop pode ser cortada");
        assert_eq!(apply_risk(&close, &at(-0.05, "BAIXA"), &RISK).action, Action::Close, "a tendência virou contra a compra");
    }

    #[test]
    fn closing_is_free_without_measured_movement_and_reversals_only_close() {
        // Sem o movimento medido (posição adotada), a IA pode fechar; sinal contrário só fecha, não inverte.
        assert_eq!(apply_risk(&decision(Action::Close, 60), &state(Some(Side::Short)), &RISK).action, Action::Close);
        let reversal = apply_risk(&decision(Action::OpenLong, 90), &state(Some(Side::Short)), &RISK);
        assert_eq!(reversal.action, Action::Close);
        assert!(reversal.note.contains("sem inverter"));
    }

    const RULES: ExitRules = ExitRules { stop_loss_pct: 0.3, take_profit_pct: 0.5, profit_lock_pct: 0.25, max_hold_minutes: 30 };

    #[test]
    fn quick_exits_take_profit_cut_losses_and_lock_gains() {
        assert!((move_pct(Side::Long, 100.0, 100.5) - 0.5).abs() < 1e-9);
        assert!((move_pct(Side::Short, 100.0, 100.5) - -0.5).abs() < 1e-9);
        assert_eq!(quick_exit(0.1, 0.1, 5, &RULES), None);
        assert_eq!(quick_exit(0.5, 0.5, 5, &RULES).unwrap().0, "TARGET");
        assert_eq!(quick_exit(-0.31, 0.0, 5, &RULES).unwrap().0, "STOP");
        // Chegou a +0,3% e voltou para +0,1%: fecha ainda positivo depois da taxa.
        let (reason, note) = quick_exit(0.1, 0.3, 8, &RULES).unwrap();
        assert_eq!(reason, "LOCK");
        assert!(note.contains("+0,30%"));
        assert_eq!(quick_exit(0.2, 0.3, 8, &RULES), None, "ainda acima do piso da trava");
        assert_eq!(quick_exit(0.05, 0.2, 8, &RULES), None, "nunca chegou ao gatilho da trava");
        assert_eq!(quick_exit(0.05, 0.2, 30, &RULES).unwrap().0, "TIMEOUT");
        let no_lock = ExitRules { profit_lock_pct: 0.0, ..RULES };
        assert_eq!(quick_exit(0.0, 0.4, 8, &no_lock), None);
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
    fn alignment_is_symmetric_for_up_and_down_markets() {
        let rising: Vec<f64> = (0..80).map(|i| 100.0 + i as f64).collect();
        let falling: Vec<f64> = rising.iter().rev().copied().collect();
        assert_eq!(alignment(&rising, &rising), Alignment { short_term: "ALTA", trend_1h: "ALTA", signal: "ALTA" });
        assert_eq!(alignment(&falling, &falling), Alignment { short_term: "BAIXA", trend_1h: "BAIXA", signal: "BAIXA" });
        assert_eq!(alignment(&rising, &falling).signal, "MISTO");
        assert_eq!(alignment(&rising[..10], &rising).short_term, "NEUTRO");
    }

    #[test]
    fn context_has_precomputed_indicators_and_position() {
        let candles: Vec<Candle> = (0..80).map(|i| Candle { open_time: i, open: 100.0 + i as f64, high: 101.0 + i as f64, low: 99.0 + i as f64, close: 100.5 + i as f64, volume: 10.0 }).collect();
        let position = OpenPosition { side: Side::Long, quantity: 1.0, entry_price: 170.0, unrealized: 9.5, leverage: 5 };
        let context = build_context(&ContextInput {
            symbol: "BTC-USDT", interval: "15m", candles: &candles, hourly: &candles, position: Some(&position),
            equity: 1000.0, stop_loss_pct: 1.0, take_profit_pct: 2.0, leverage: 5, margin_usdt: 20.0, recent_decisions: &[], lessons: &[],
            exit_rules: ExitRules { stop_loss_pct: 1.0, take_profit_pct: 2.0, profit_lock_pct: 0.25, max_hold_minutes: 30 }, minutes_open: Some(12),
        }).unwrap();
        assert_eq!(context["indicators"]["ema9AboveEma21"], true);
        assert_eq!(context["alignment"]["signal"], "ALTA");
        assert_eq!(context["position"]["state"], "LONG");
        assert_eq!(context["lastCandlesOHLCV"].as_array().unwrap().len(), 12);
        assert!(build_context(&ContextInput { candles: &candles[..10], ..ContextInput {
            symbol: "BTC-USDT", interval: "15m", candles: &candles, hourly: &candles, position: None,
            equity: 1000.0, stop_loss_pct: 1.0, take_profit_pct: 2.0, leverage: 5, margin_usdt: 20.0, recent_decisions: &[], lessons: &[],
            exit_rules: ExitRules { stop_loss_pct: 1.0, take_profit_pct: 2.0, profit_lock_pct: 0.25, max_hold_minutes: 30 }, minutes_open: Some(12),
        } }).is_err());
    }
}
