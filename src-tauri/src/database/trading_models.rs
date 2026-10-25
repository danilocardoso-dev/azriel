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
