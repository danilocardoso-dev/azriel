use serde::{Deserialize, Serialize};

/// Configuração do módulo de trading. As chaves da BingX nunca saem do backend.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingSettings {
    pub api_key_configured: bool,
    pub symbol: String,
    pub candle_interval: String,
    pub cycle_minutes: i64,
    pub margin_usdt: f64,
    pub leverage: i64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_daily_loss_pct: f64,
    pub min_confidence: i64,
    pub engine: String,
    pub profit_lock_pct: f64,
    pub max_hold_minutes: i64,
    pub running: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingSettingsInput {
    pub symbol: String,
    pub candle_interval: String,
    pub cycle_minutes: i64,
    pub margin_usdt: f64,
    pub leverage: i64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub max_daily_loss_pct: f64,
    pub min_confidence: i64,
    pub engine: String,
    pub profit_lock_pct: f64,
    pub max_hold_minutes: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingKeysInput {
    pub api_key: String,
    pub secret_key: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingDecisionRecord {
    pub id: i64,
    pub created_at: String,
    pub symbol: String,
    pub price: f64,
    pub position_before: String,
    pub proposed_action: String,
    pub final_action: String,
    pub confidence: i64,
    pub reason: String,
    pub risk_note: String,
    pub engine: String,
    pub model: String,
    pub order_status: String,
    pub order_detail: String,
    pub equity: Option<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct NewTradingDecision {
    pub symbol: String,
    pub price: f64,
    pub position_before: String,
    pub proposed_action: String,
    pub final_action: String,
    pub confidence: i64,
    pub reason: String,
    pub risk_note: String,
    pub engine: String,
    pub model: String,
    pub order_status: String,
    pub order_detail: String,
    pub equity: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EquityPoint {
    pub created_at: String,
    pub equity: f64,
    pub balance: f64,
    pub unrealized: f64,
}

/// Um trade do diário: aberto pelo agente (ou adotado) e fechado com o resultado realizado.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TradingTrade {
    pub id: i64,
    pub symbol: String,
    pub side: String,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub quantity: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub candle_interval: String,
    pub balance_before: f64,
    pub pnl_usdt: Option<f64>,
    pub exit_reason: Option<String>,
    pub confidence: i64,
    pub entry_reason: String,
    pub entry_context: String,
}

#[derive(Debug, Clone, Default)]
pub struct NewTradingTrade {
    pub symbol: String,
    pub side: String,
    pub entry_price: f64,
    pub quantity: f64,
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub candle_interval: String,
    pub balance_before: f64,
    pub confidence: i64,
    pub entry_reason: String,
    pub entry_context: String,
}

/// Um parâmetro alterado por uma revisão, já formatado para a tela.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TuningChange {
    pub field: String,
    pub label: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingReview {
    pub id: i64,
    pub created_at: String,
    pub last_trade_id: i64,
    pub trades_count: i64,
    pub pnl_usdt: f64,
    pub win_rate: f64,
    pub diagnosis: String,
    pub lessons: Vec<String>,
    pub changes: Vec<TuningChange>,
    pub model: String,
    pub reverted: bool,
}

#[derive(Debug, Clone)]
pub struct NewTradingReview {
    pub last_trade_id: i64,
    pub trades_count: i64,
    pub pnl_usdt: f64,
    pub win_rate: f64,
    pub diagnosis: String,
    pub lessons: Vec<String>,
    pub changes: Vec<TuningChange>,
    pub settings_before: Tunable,
    pub model: String,
}

/// Os únicos parâmetros que o autoajuste pode mexer. Margem, alavancagem, perda diária,
/// par e motor ficam sempre com você.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tunable {
    pub stop_loss_pct: f64,
    pub take_profit_pct: f64,
    pub min_confidence: i64,
    pub candle_interval: String,
    pub cycle_minutes: i64,
}

impl Tunable {
    pub fn of(settings: &TradingSettings) -> Self {
        Self {
            stop_loss_pct: settings.stop_loss_pct,
            take_profit_pct: settings.take_profit_pct,
            min_confidence: settings.min_confidence,
            candle_interval: settings.candle_interval.clone(),
            cycle_minutes: settings.cycle_minutes,
        }
    }
}
