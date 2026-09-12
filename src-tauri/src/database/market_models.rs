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
pub struct MarketAiStatus {
    pub agent_id: String,
    pub configured: bool,
    pub available: bool,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub decision_interval: usize,
    pub timeout_ms: u64,
    pub max_retries: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMarketAiConfigInput {
    pub agent_id: String,
    pub decision_interval: usize,
    pub timeout_ms: u64,
    pub max_retries: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiRuntimeMetric {
    pub agent_id: String,
    pub prompt_version: String,
    pub call_count: usize,
    pub successful_call_count: usize,
    pub invalid_response_count: usize,
    pub timeout_count: usize,
    pub retry_count: usize,
    pub fallback_count: usize,
    pub average_latency_ms: f64,
    pub max_latency_ms: u64,
    pub total_latency_ms: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub buy_count: usize,
    pub sell_count: usize,
    pub llm_hold_count: usize,
    pub no_llm_call_count: usize,
    pub average_confidence: Option<f64>,
    pub average_buy_confidence: Option<f64>,
    pub average_sell_confidence: Option<f64>,
    pub average_hold_confidence: Option<f64>,
    pub min_confidence: Option<f64>,
    pub max_confidence: Option<f64>,
    pub median_confidence: Option<f64>,
    pub confidence_distribution: Vec<usize>,
    pub first_pass_valid_count: usize,
    pub retry_recovered_count: usize,
    pub final_valid_count: usize,
    pub final_invalid_count: usize,
    pub system_fallback_count: usize,
    pub first_attempt_average_latency_ms: f64,
    pub retry_average_latency_ms: f64,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub slow_call_count: usize,
    pub first_pass_valid_rate_pct: f64,
    pub retry_recovery_rate_pct: f64,
    pub final_valid_rate_pct: f64,
    pub fallback_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiValidationStageLog {
    pub stage: String,
    pub success: bool,
    pub error_code: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiAttemptLog {
    pub attempt_number: usize,
    pub raw_response: Option<String>,
    pub status: String,
    pub error_type: Option<String>,
    pub error_field: Option<String>,
    pub error_value: Option<String>,
    pub error_message: Option<String>,
    pub normalized_from_wrapped_json: bool,
    pub latency_ms: u64,
    pub stages: Vec<MarketAiValidationStageLog>,
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
    pub exposure_pct: f64,
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
pub struct MarketBehaviorMetric {
    pub agent_id: String,
    pub buy_count: usize,
    pub sell_count: usize,
    pub hold_count: usize,
    pub hold_rate_pct: f64,
    pub trade_frequency_pct: f64,
    pub average_exposure_pct: f64,
    pub max_exposure_pct: f64,
    pub average_position_size_pct: f64,
    pub max_position_size_pct: f64,
    pub average_holding_candles: f64,
    pub median_holding_candles: f64,
    pub max_holding_candles: usize,
    pub turnover_pct: f64,
    pub time_in_market_pct: f64,
    pub time_in_cash_pct: f64,
    pub entry_count: usize,
    pub exit_count: usize,
    pub risk_rejection_count: usize,
    pub risk_modification_count: usize,
    pub risk_rejection_rate_pct: f64,
    pub risk_modification_rate_pct: f64,
    pub drawdown_trigger_count: usize,
    pub daily_loss_trigger_count: usize,
    pub formula_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketPositionEpisode {
    pub agent_id: String,
    pub episode_index: usize,
    pub opened_candle_index: usize,
    pub opened_at: String,
    pub closed_candle_index: Option<usize>,
    pub closed_at: Option<String>,
    pub duration_candles: usize,
    pub max_exposure_pct: f64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAgentCorrelation {
    pub agent_a_id: String,
    pub agent_b_id: String,
    pub equity_return_correlation: Option<f64>,
    pub decision_similarity: f64,
    pub high_similarity: bool,
    pub similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketBenchmarkComparison {
    pub agent_id: String,
    pub cash_return_pct: Option<f64>,
    pub buy_hold_return_pct: Option<f64>,
    pub excess_vs_cash_pct: f64,
    pub excess_vs_buy_hold_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketObservatory {
    pub behavior: Vec<MarketBehaviorMetric>,
    pub episodes: Vec<MarketPositionEpisode>,
    pub correlations: Vec<MarketAgentCorrelation>,
    pub benchmarks: Vec<MarketBenchmarkComparison>,
    pub similarity_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTradeLifecycle {
    pub id: String,
    pub agent_id: String,
    pub asset: String,
    pub lifecycle_index: usize,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub entry_price: f64,
    pub average_entry_price: f64,
    pub exit_price: Option<f64>,
    pub initial_exposure_pct: f64,
    pub max_exposure_pct: f64,
    pub holding_candles: usize,
    pub realized_pnl: f64,
    pub realized_pnl_pct: f64,
    pub mfe_pct: f64,
    pub mae_pct: f64,
    pub exit_efficiency_pct: Option<f64>,
    pub profit_giveback_pct: f64,
    pub entry_reason_code: Option<String>,
    pub exit_reason_code: Option<String>,
    pub status: String,
    pub reentry: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketPositionEvent {
    pub id: i64,
    pub lifecycle_id: String,
    pub decision_id: Option<i64>,
    pub execution_id: Option<i64>,
    pub timestamp: String,
    pub action: String,
    pub previous_exposure_pct: f64,
    pub target_exposure_pct: f64,
    pub new_exposure_pct: f64,
    pub signal_bias: Option<String>,
    pub confidence: Option<f64>,
    pub reason_code: Option<String>,
    pub risk_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketPositionMetric {
    pub agent_id: String,
    pub closed_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
    pub average_holding_candles: f64,
    pub median_holding_candles: f64,
    pub average_mfe_pct: f64,
    pub average_mae_pct: f64,
    pub average_exit_efficiency_pct: Option<f64>,
    pub average_profit_giveback_pct: f64,
    pub rapid_reentry_count: usize,
    pub rapid_exit_count: usize,
    pub reentry_count: usize,
    pub average_entry_exposure_pct: f64,
    pub average_max_exposure_pct: f64,
    pub invalid_position_action_count: usize,
    pub flat_sell_attempt_count: usize,
    pub redundant_exit_count: usize,
    pub engine_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MarketPositionLifecycleReport {
    pub lifecycles: Vec<MarketTradeLifecycle>,
    pub events: Vec<MarketPositionEvent>,
    pub metrics: Vec<MarketPositionMetric>,
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
    pub observatory: MarketObservatory,
    pub position_lifecycle: MarketPositionLifecycleReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationSplitConfig {
    pub in_sample_pct: usize,
    pub validation_pct: usize,
    pub out_of_sample_pct: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalkForwardConfig {
    pub train_window_size: usize,
    pub test_window_size: usize,
    pub step_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRegimeConfig {
    pub trend_window: usize,
    pub volatility_window: usize,
    pub bull_threshold_pct: f64,
    pub bear_threshold_pct: f64,
    pub high_volatility_threshold_pct: f64,
    pub low_volatility_threshold_pct: f64,
    pub minimum_sample_candles: usize,
    pub minimum_sample_trades: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketValidationInput {
    pub name: String,
    pub dataset_id: String,
    pub risk_profile_id: String,
    pub agent_ids: Vec<String>,
    pub initial_capital: f64,
    pub random_seed: u64,
    pub fee_pct: f64,
    pub slippage_pct: f64,
    pub split_config: ValidationSplitConfig,
    pub walk_forward_config: WalkForwardConfig,
    pub regime_config: MarketRegimeConfig,
    pub rolling_window: usize,
    pub annualization_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketValidationSummary {
    pub id: String,
    pub name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub dataset_hash: String,
    pub risk_profile_id: String,
    pub agent_ids: Vec<String>,
    pub status: String,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketValidationWindow {
    pub id: i64,
    pub window_index: usize,
    pub window_type: String,
    pub start_index: usize,
    pub end_index: usize,
    pub start_at: String,
    pub end_at: String,
    pub train_start_index: Option<usize>,
    pub train_end_index: Option<usize>,
    pub train_start_at: Option<String>,
    pub train_end_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketWindowMetric {
    pub window_id: i64,
    pub window_index: usize,
    pub window_type: String,
    pub agent_id: String,
    pub agent_name: String,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub sharpe: Option<f64>,
    pub sortino: Option<f64>,
    pub calmar: Option<f64>,
    pub trade_count: usize,
    pub hold_rate_pct: f64,
    pub exposure_pct: f64,
    pub benchmark_cash_excess_pct: f64,
    pub benchmark_buy_hold_excess_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRegimeMetric {
    pub agent_id: String,
    pub regime_type: String,
    pub regime: String,
    pub candle_count: usize,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub trade_count: usize,
    pub hold_rate_pct: f64,
    pub exposure_pct: f64,
    pub profit_factor: Option<f64>,
    pub win_rate_pct: Option<f64>,
    pub low_sample_size: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRollingMetric {
    pub agent_id: String,
    pub candle_index: usize,
    pub timestamp: String,
    pub rolling_return_pct: Option<f64>,
    pub rolling_volatility_pct: Option<f64>,
    pub rolling_sharpe: Option<f64>,
    pub rolling_drawdown_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketRobustnessReport {
    pub agent_id: String,
    pub agent_name: String,
    pub in_sample_return_pct: f64,
    pub validation_return_pct: f64,
    pub out_of_sample_return_pct: f64,
    pub out_of_sample_drawdown_pct: f64,
    pub positive_window_ratio_pct: f64,
    pub average_window_return_pct: f64,
    pub median_window_return_pct: f64,
    pub best_window_return_pct: f64,
    pub worst_window_return_pct: f64,
    pub return_std_across_windows: f64,
    pub drawdown_std_across_windows: f64,
    pub benchmark_excess_pct: f64,
    pub overfitting_gap_pct: f64,
    pub possible_overfitting: bool,
    pub robustness_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketValidationAudit {
    pub frozen_config_json: String,
    pub split_config_json: String,
    pub walk_forward_config_json: String,
    pub regime_config_json: String,
    pub rolling_window: usize,
    pub annualization_factor: f64,
    pub validation_engine_version: String,
    pub metric_formula_version: String,
    pub regime_engine_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketValidationResult {
    pub validation: MarketValidationSummary,
    pub windows: Vec<MarketValidationWindow>,
    pub metrics: Vec<MarketWindowMetric>,
    pub regime_metrics: Vec<MarketRegimeMetric>,
    pub rolling_metrics: Vec<MarketRollingMetric>,
    pub reports: Vec<MarketRobustnessReport>,
    pub audit: MarketValidationAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiDecisionLog {
    pub decision_id: i64,
    pub timestamp: String,
    pub action: String,
    pub desired_position_pct: Option<f64>,
    pub confidence: Option<f64>,
    pub reason: String,
    pub reason_code: Option<String>,
    pub validation_code: Option<String>,
    pub risk_result: String,
    pub risk_reason: String,
    pub approved_position_pct: Option<f64>,
    pub execution_price: Option<f64>,
    pub call_status: String,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub latency_ms: u64,
    pub attempts: usize,
    pub fallback_used: bool,
    pub input_snapshot: Option<serde_json::Value>,
    pub forward_return_1: Option<f64>,
    pub forward_return_5: Option<f64>,
    pub forward_return_10: Option<f64>,
    pub signal_disagreement: bool,
    pub lifecycle_action: Option<String>,
    pub first_failure_type: Option<String>,
    pub fallback_reason: Option<String>,
    pub output_attempts: Vec<MarketAiAttemptLog>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiExperimentComparison {
    pub experiment_id: String,
    pub experiment_name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub risk_profile_id: String,
    pub initial_capital: f64,
    pub random_seed: u64,
    pub fee_pct: f64,
    pub slippage_pct: f64,
    pub decision_interval: usize,
    pub agent_id: String,
    pub prompt_version: String,
    pub call_count: usize,
    pub successful_call_count: usize,
    pub invalid_response_count: usize,
    pub timeout_count: usize,
    pub buy_count: usize,
    pub sell_count: usize,
    pub llm_hold_count: usize,
    pub trade_count: usize,
    pub hold_rate_pct: f64,
    pub average_confidence: Option<f64>,
    pub total_return_pct: f64,
    pub max_drawdown_pct: f64,
    pub average_exposure_pct: f64,
    pub average_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSignalMatrixRow {
    pub label: String,
    pub buy_count: usize,
    pub sell_count: usize,
    pub hold_count: usize,
    pub average_forward_5: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketReasonCodeCount {
    pub reason_code: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSignalDiagnostics {
    pub signal_engine_version: String,
    pub signal_config_version: String,
    pub bullish_count: usize,
    pub bearish_count: usize,
    pub neutral_count: usize,
    pub strong_count: usize,
    pub weak_count: usize,
    pub low_conflict_count: usize,
    pub medium_conflict_count: usize,
    pub high_conflict_count: usize,
    pub action_collapse: bool,
    pub collapsed_action: Option<String>,
    pub confidence_collapse: bool,
    pub collapsed_confidence: Option<f64>,
    pub disagreement_count: usize,
    pub disagreement_rate_pct: f64,
    pub strong_bullish_buy_rate_pct: Option<f64>,
    pub strong_bullish_hold_rate_pct: Option<f64>,
    pub strong_bearish_sell_rate_pct: Option<f64>,
    pub strong_bearish_hold_rate_pct: Option<f64>,
    pub signal_action_matrix: Vec<MarketSignalMatrixRow>,
    pub conflict_action_matrix: Vec<MarketSignalMatrixRow>,
    pub reason_codes: Vec<MarketReasonCodeCount>,
}
