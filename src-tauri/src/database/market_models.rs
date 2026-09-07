use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDataset {
    pub id: String,
    pub name: String,
    pub asset: String,
    pub timeframe: String,
    pub currency: String,
    pub start_at: String,
    pub end_at: String,
    pub candle_count: usize,
    pub fingerprint: String,
    pub source_path: String,
    pub imported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAgentDefinition {
    pub id: String,
    pub name: String,
    pub strategy_type: String,
    pub strategy_version: String,
    pub default_config_json: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRiskProfile {
    pub id: String,
    pub name: String,
    pub max_position_pct: f64,
    pub max_total_exposure_pct: f64,
    pub max_daily_loss_pct: f64,
    pub max_drawdown_pct: f64,
    pub max_trades_per_day: Option<usize>,
    pub allow_leverage: bool,
    pub allow_short: bool,
    pub allowed_assets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportMarketDatasetInput {
    pub path: String,
    pub name: String,
    pub asset: String,
    pub timeframe: String,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketExperimentInput {
    pub name: String,
    pub dataset_id: String,
    pub risk_profile_id: String,
    pub agent_ids: Vec<String>,
    pub initial_capital: f64,
    pub random_seed: u64,
    pub fee_pct: f64,
    pub slippage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAgentMetric {
    pub agent_id: String,
    pub agent_name: String,
    pub status: String,
    pub final_equity: f64,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub decision_count: usize,
    pub hold_count: usize,
    pub trade_count: usize,
    pub win_rate_pct: f64,
    pub profit_factor: Option<f64>,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub average_exposure_pct: f64,
    pub max_exposure_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketEquityPoint {
    pub timestamp: String,
    pub agent_id: String,
    pub equity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketDecisionLog {
    pub id: i64,
    pub timestamp: String,
    pub agent_id: String,
    pub action: String,
    pub observed_price: f64,
    pub desired_position_pct: Option<f64>,
    pub reasoning: String,
    pub risk_result: String,
    pub risk_reason: String,
    pub approved_position_pct: Option<f64>,
    pub execution_price: Option<f64>,
    pub quantity: Option<f64>,
    pub fees: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketExperimentSummary {
    pub id: String,
    pub name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub asset: String,
    pub timeframe: String,
    pub currency: String,
    pub initial_capital: f64,
    pub risk_profile_id: String,
    pub random_seed: u64,
    pub fee_pct: f64,
    pub slippage_pct: f64,
    pub agent_ids: Vec<String>,
    pub status: String,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketExperimentResult {
    pub experiment: MarketExperimentSummary,
    pub metrics: Vec<MarketAgentMetric>,
    pub equity: Vec<MarketEquityPoint>,
    pub decisions: Vec<MarketDecisionLog>,
}
