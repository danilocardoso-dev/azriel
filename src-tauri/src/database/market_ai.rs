use crate::ollama::{self, OllamaMessage};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub const AI_AGENT_ID: &str = "ai-technical-v1";
pub const AI_AGENT_V2_ID: &str = "ai-technical-v2";
pub const AI_AGENT_V3_ID: &str = "ai-technical-v3";
pub const AI_AGENT_V4_ID: &str = "ai-technical-v4";
pub const AI_AGENT_V4_1_ID: &str = "ai-technical-v4-1";
pub const AI_AGENT_V4_2_ID: &str = "ai-technical-v4-2";
pub const PROMPT_VERSION: &str = "MARKET_AI_AGENT_V1";
pub const PROMPT_V2_VERSION: &str = "MARKET_AI_AGENT_V2";
pub const PROMPT_V3_VERSION: &str = "MARKET_AI_AGENT_V3";
pub const PROMPT_V4_VERSION: &str = "MARKET_AI_AGENT_V4";
pub const PROMPT_V4_1_VERSION: &str = "MARKET_AI_AGENT_V4_1";
pub const PROMPT_V4_2_VERSION: &str = "MARKET_AI_AGENT_V4_2";
pub const MAX_AI_AGENTS: usize = 1;
pub const SYSTEM_PROMPT: &str = include_str!("../../prompts/market-lab-agent.txt");
pub const SYSTEM_PROMPT_V2: &str = include_str!("../../prompts/market-lab-agent-v2.txt");
pub const SYSTEM_PROMPT_V3: &str = include_str!("../../prompts/market-lab-agent-v3.txt");
pub const SYSTEM_PROMPT_V4: &str = include_str!("../../prompts/market-lab-agent-v4.txt");
pub const SYSTEM_PROMPT_V4_1: &str = include_str!("../../prompts/market-lab-agent-v4-1.txt");
pub const SYSTEM_PROMPT_V4_2: &str = include_str!("../../prompts/market-lab-agent-v4-2.txt");

pub fn is_ai_agent(agent_id: &str) -> bool {
    matches!(
        agent_id,
        AI_AGENT_ID | AI_AGENT_V2_ID | AI_AGENT_V3_ID | AI_AGENT_V4_ID | AI_AGENT_V4_1_ID | AI_AGENT_V4_2_ID
    )
}

pub fn prompt_for_version(version: &str) -> Option<&'static str> {
    match version {
        PROMPT_VERSION => Some(SYSTEM_PROMPT),
        PROMPT_V2_VERSION => Some(SYSTEM_PROMPT_V2),
        PROMPT_V3_VERSION => Some(SYSTEM_PROMPT_V3),
        PROMPT_V4_VERSION => Some(SYSTEM_PROMPT_V4),
        PROMPT_V4_1_VERSION => Some(SYSTEM_PROMPT_V4_1),
        PROMPT_V4_2_VERSION => Some(SYSTEM_PROMPT_V4_2),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiConfig {
    pub agent_id: String,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub temperature: f64,
    pub decision_interval: usize,
    pub timeout_ms: u64,
    pub max_retries: usize,
    pub position_sizing: crate::database::market_positions::PositionSizingConfig,
}

impl MarketAiConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !is_ai_agent(&self.agent_id) || self.provider != "ollama" || self.model.trim().is_empty()
        {
            return Err("configuração do AI Agent possui provider ou modelo inválido".into());
        }
        let expected_prompt = match self.agent_id.as_str() {
            AI_AGENT_V2_ID => PROMPT_V2_VERSION,
            AI_AGENT_V3_ID => PROMPT_V3_VERSION,
            AI_AGENT_V4_ID => PROMPT_V4_VERSION,
            AI_AGENT_V4_1_ID => PROMPT_V4_1_VERSION,
            AI_AGENT_V4_2_ID => PROMPT_V4_2_VERSION,
            _ => PROMPT_VERSION,
        };
        if self.prompt_version != expected_prompt
            || prompt_for_version(&self.prompt_version).is_none()
            || !(0.0..=0.3).contains(&self.temperature)
        {
            return Err("prompt version ou temperatura do AI Agent inválido".into());
        }
        if !(1..=100).contains(&self.decision_interval)
            || !(5_000..=180_000).contains(&self.timeout_ms)
            || self.max_retries > 1
        {
            return Err("cadência, timeout ou retry do AI Agent inválido".into());
        }
        if self.prompt_version == PROMPT_V4_2_VERSION {
            self.position_sizing.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiSnapshot {
    pub timestamp: String,
    pub asset: String,
    pub timeframe: String,
    pub price: f64,
    pub features: MarketAiFeatures,
    pub portfolio: MarketAiPortfolio,
    pub risk_context: MarketAiRiskContext,
    pub memory: MarketAiMemory,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiFeatures {
    pub return_1: Option<f64>,
    pub return_5: Option<f64>,
    pub sma_short: Option<f64>,
    pub sma_long: Option<f64>,
    pub volatility: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiPortfolio {
    pub cash: f64,
    pub equity: f64,
    pub exposure_pct: f64,
    pub current_position: bool,
    pub entry_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiRiskContext {
    pub max_position_pct: f64,
    pub max_total_exposure_pct: f64,
    pub allow_short: bool,
    pub allow_leverage: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiMemory {
    pub last_action: Option<String>,
    pub recent_decision_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiSnapshotV2 {
    pub market: MarketAiMarket,
    pub returns: MarketAiReturns,
    pub trend: MarketAiTrend,
    pub momentum: MarketAiMomentum,
    pub volatility: MarketAiVolatility,
    pub regime: MarketAiRegime,
    pub portfolio: MarketAiPortfolioV2,
    pub previous_decision: Option<MarketAiPreviousDecision>,
    pub risk_context: MarketAiRiskContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiMarket {
    pub timestamp: String,
    pub asset: String,
    pub timeframe: String,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiReturns {
    pub return_1: Option<f64>,
    pub return_5: Option<f64>,
    pub return_10: Option<f64>,
    pub return_20: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiTrend {
    pub sma_short: Option<f64>,
    pub sma_long: Option<f64>,
    pub price_vs_sma_short_pct: Option<f64>,
    pub price_vs_sma_long_pct: Option<f64>,
    pub sma_spread_pct: Option<f64>,
    pub short_sma_slope: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiMomentum {
    pub momentum_5: Option<f64>,
    pub momentum_10: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiVolatility {
    pub rolling_volatility: Option<f64>,
    pub volatility_regime: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiRegime {
    pub trend: String,
    pub volatility: String,
    pub engine_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiPortfolioV2 {
    pub cash: f64,
    pub equity: f64,
    pub current_position: bool,
    pub position_pct: f64,
    pub entry_price: Option<f64>,
    pub unrealized_pnl_pct: Option<f64>,
    pub holding_period: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiPreviousDecision {
    pub action: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiSnapshotV3 {
    pub market: MarketAiMarket,
    pub signals: crate::database::market_signals::MarketSignalSummary,
    pub regime: MarketAiRegime,
    pub portfolio: MarketAiPortfolioV2,
    pub key_features: MarketAiKeyFeatures,
    pub risk_context: MarketAiRiskContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiSnapshotV4 {
    pub market: MarketAiMarket,
    pub signals: crate::database::market_signals::MarketSignalSummary,
    pub regime: MarketAiRegime,
    pub position: crate::database::market_positions::PositionContext,
    pub risk_context: MarketAiRiskContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiSnapshotV42 {
    pub market: MarketAiMarket,
    pub signals: crate::database::market_signals::MarketSignalSummary,
    pub regime: MarketAiRegime,
    pub position: MarketAiPositionContextV42,
    pub risk_context: MarketAiRiskContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiPositionContextV42 {
    pub state: String,
    pub current_exposure_pct: f64,
    pub average_entry_price: Option<f64>,
    pub unrealized_pnl_pct: Option<f64>,
    pub holding_candles: usize,
}

impl MarketAiPositionContextV42 {
    pub fn from_position(
        position: &crate::database::market_positions::PositionContext,
    ) -> Result<Self, String> {
        use crate::database::market_positions::PositionState;
        if !position.exposure_pct.is_finite() || !(0.0..=100.0).contains(&position.exposure_pct) {
            return Err("POSITION_CONTEXT_ERROR: exposição inválida".into());
        }
        let state = match position.state {
            PositionState::Flat if position.exposure_pct.abs() <= 0.01 => "FLAT",
            PositionState::LongOpen | PositionState::LongReduced if position.exposure_pct > 0.01 => "LONG",
            _ => return Err("POSITION_CONTEXT_ERROR: estado e exposição inconsistentes".into()),
        };
        Ok(Self {
            state: state.into(),
            current_exposure_pct: position.exposure_pct,
            average_entry_price: position.average_entry_price,
            unrealized_pnl_pct: position.unrealized_pnl_pct,
            holding_candles: position.holding_candles,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketAiKeyFeatures {
    pub return_5: Option<f64>,
    pub return_20: Option<f64>,
    pub price_vs_sma_short_pct: Option<f64>,
    pub sma_spread_pct: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct MarketAiPrompt {
    pub system: String,
    pub snapshot_json: String,
    pub structured_output_schema: Option<serde_json::Value>,
    pub retry_instruction: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarketAiProviderResponse {
    pub content: String,
    pub model: String,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketAiProviderErrorKind {
    Timeout,
    Offline,
}

#[derive(Debug, Clone)]
pub struct MarketAiProviderError {
    pub kind: MarketAiProviderErrorKind,
}

pub trait MarketAiProvider: Send + Sync {
    fn supports_structured_output(&self) -> bool {
        false
    }

    fn complete(
        &self,
        prompt: MarketAiPrompt,
        config: &MarketAiConfig,
    ) -> Result<MarketAiProviderResponse, MarketAiProviderError>;
}

pub struct OllamaMarketAiProvider {
    endpoint: String,
}

pub struct UnavailableMarketAiProvider;

impl MarketAiProvider for UnavailableMarketAiProvider {
    fn complete(
        &self,
        _: MarketAiPrompt,
        _: &MarketAiConfig,
    ) -> Result<MarketAiProviderResponse, MarketAiProviderError> {
        Err(MarketAiProviderError {
            kind: MarketAiProviderErrorKind::Offline,
        })
    }
}

impl OllamaMarketAiProvider {
    pub fn new(endpoint: String) -> Self {
        Self { endpoint }
    }
}

impl MarketAiProvider for OllamaMarketAiProvider {
    fn supports_structured_output(&self) -> bool {
        true
    }

    fn complete(
        &self,
        prompt: MarketAiPrompt,
        config: &MarketAiConfig,
    ) -> Result<MarketAiProviderResponse, MarketAiProviderError> {
        let mut messages = vec![
            OllamaMessage {
                role: "system".into(),
                content: prompt.system,
            },
            OllamaMessage {
                role: "user".into(),
                content: prompt.snapshot_json,
            },
        ];
        if let Some(instruction) = prompt.retry_instruction {
            messages.push(OllamaMessage {
                role: "user".into(),
                content: instruction,
            });
        }
        let result = tauri::async_runtime::block_on(ollama::chat_with_format(
            &self.endpoint,
            &config.model,
            messages,
            config.timeout_ms.div_ceil(1000),
            "market-agent",
            prompt.structured_output_schema,
        ))
        .map_err(|error| MarketAiProviderError {
            kind: if error.to_ascii_lowercase().contains("tempo limite") {
                MarketAiProviderErrorKind::Timeout
            } else {
                MarketAiProviderErrorKind::Offline
            },
        })?;
        Ok(MarketAiProviderResponse {
            content: result.content,
            model: result.model,
            input_tokens: None,
            output_tokens: None,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuredDecision {
    action: String,
    desired_position_pct: f64,
    confidence: f64,
    reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuredDecisionV3 {
    action: String,
    desired_position_pct: f64,
    confidence: f64,
    reason_code: String,
    reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StructuredDecisionV4 {
    action: String,
    target_exposure_pct: f64,
    confidence: f64,
    reason_code: String,
    reason: String,
}

const REASON_CODES: &[&str] = &[
    "ALIGNED_BULLISH_SIGNAL",
    "ALIGNED_BEARISH_SIGNAL",
    "CONFLICTING_SIGNALS",
    "INSUFFICIENT_SIGNAL",
    "HIGH_VOLATILITY",
    "WAITING_CONFIRMATION",
    "POSITION_ALREADY_OPTIMAL",
    "TREND_DETERIORATION",
    "MOMENTUM_DETERIORATION",
    "EXIT_SIGNAL",
];
const REASON_CODES_V4: &[&str] = &[
    "ALIGNED_BULLISH_SIGNAL",
    "ALIGNED_BEARISH_SIGNAL",
    "TREND_DETERIORATION",
    "MOMENTUM_DETERIORATION",
    "REGIME_REVERSAL",
    "POSITION_ALREADY_OPTIMAL",
    "INSUFFICIENT_SIGNAL",
    "CONFLICTING_SIGNALS",
    "REDUCE_RISK",
    "EXIT_SIGNAL",
    "REENTRY_SIGNAL",
];

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DegenerationDiagnostics {
    pub sample_size: usize,
    pub action_collapse: bool,
    pub collapsed_action: Option<String>,
    pub confidence_collapse: bool,
    pub collapsed_confidence: Option<f64>,
}

pub fn detect_degeneration(values: &[(String, f64)]) -> DegenerationDiagnostics {
    use std::collections::HashMap;
    let mut actions: HashMap<&str, usize> = HashMap::new();
    let mut confidences: HashMap<i64, usize> = HashMap::new();
    for (action, confidence) in values {
        *actions.entry(action.as_str()).or_default() += 1;
        *confidences
            .entry((confidence * 10_000.0).round() as i64)
            .or_default() += 1;
    }
    let minimum = 20;
    let threshold = (values.len() as f64 * 0.95).ceil() as usize;
    let action = actions.into_iter().max_by_key(|(_, count)| *count);
    let confidence = confidences.into_iter().max_by_key(|(_, count)| *count);
    DegenerationDiagnostics {
        sample_size: values.len(),
        action_collapse: values.len() >= minimum
            && action.is_some_and(|(_, count)| count >= threshold),
        collapsed_action: action
            .filter(|(_, count)| values.len() >= minimum && *count >= threshold)
            .map(|(value, _)| value.into()),
        confidence_collapse: values.len() >= minimum
            && confidence.is_some_and(|(_, count)| count >= threshold),
        collapsed_confidence: confidence
            .filter(|(_, count)| values.len() >= minimum && *count >= threshold)
            .map(|(value, _)| value as f64 / 10_000.0),
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiRuntimeMetrics {
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
    pub first_pass_valid_count: usize,
    pub retry_recovered_count: usize,
    pub final_valid_count: usize,
    pub final_invalid_count: usize,
    pub system_fallback_count: usize,
    pub first_attempt_total_latency_ms: u64,
    pub retry_total_latency_ms: u64,
    pub slow_call_count: usize,
    #[serde(skip)]
    decision_latency_values: Vec<u64>,
    #[serde(skip)]
    confidence_values: Vec<f64>,
    #[serde(skip)]
    buy_confidences: Vec<f64>,
    #[serde(skip)]
    sell_confidences: Vec<f64>,
    #[serde(skip)]
    hold_confidences: Vec<f64>,
}

impl MarketAiRuntimeMetrics {
    pub fn merge(&mut self, other: &Self) {
        self.call_count += other.call_count;
        self.successful_call_count += other.successful_call_count;
        self.invalid_response_count += other.invalid_response_count;
        self.timeout_count += other.timeout_count;
        self.retry_count += other.retry_count;
        self.fallback_count += other.fallback_count;
        self.total_latency_ms += other.total_latency_ms;
        self.max_latency_ms = self.max_latency_ms.max(other.max_latency_ms);
        self.input_tokens = sum_optional(self.input_tokens, other.input_tokens);
        self.output_tokens = sum_optional(self.output_tokens, other.output_tokens);
        self.buy_count += other.buy_count;
        self.sell_count += other.sell_count;
        self.llm_hold_count += other.llm_hold_count;
        self.no_llm_call_count += other.no_llm_call_count;
        self.first_pass_valid_count += other.first_pass_valid_count;
        self.retry_recovered_count += other.retry_recovered_count;
        self.final_valid_count += other.final_valid_count;
        self.final_invalid_count += other.final_invalid_count;
        self.system_fallback_count += other.system_fallback_count;
        self.first_attempt_total_latency_ms += other.first_attempt_total_latency_ms;
        self.retry_total_latency_ms += other.retry_total_latency_ms;
        self.slow_call_count += other.slow_call_count;
        self.decision_latency_values
            .extend(other.decision_latency_values.iter().copied());
        self.confidence_values
            .extend(other.confidence_values.iter().copied());
        self.buy_confidences
            .extend(other.buy_confidences.iter().copied());
        self.sell_confidences
            .extend(other.sell_confidences.iter().copied());
        self.hold_confidences
            .extend(other.hold_confidences.iter().copied());
        self.average_latency_ms = if self.call_count == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.call_count as f64
        };
    }

    pub fn register_valid_decision(&mut self, action: &str, confidence: f64) {
        self.confidence_values.push(confidence);
        match action {
            "BUY" => {
                self.buy_count += 1;
                self.buy_confidences.push(confidence);
            }
            "SELL" => {
                self.sell_count += 1;
                self.sell_confidences.push(confidence);
            }
            "HOLD" => {
                self.llm_hold_count += 1;
                self.hold_confidences.push(confidence);
            }
            _ => {}
        }
    }

    pub fn average_confidence(&self) -> Option<f64> {
        average(&self.confidence_values)
    }

    pub fn average_buy_confidence(&self) -> Option<f64> {
        average(&self.buy_confidences)
    }

    pub fn average_sell_confidence(&self) -> Option<f64> {
        average(&self.sell_confidences)
    }

    pub fn average_hold_confidence(&self) -> Option<f64> {
        average(&self.hold_confidences)
    }

    pub fn min_confidence(&self) -> Option<f64> {
        self.confidence_values.iter().copied().reduce(f64::min)
    }

    pub fn max_confidence(&self) -> Option<f64> {
        self.confidence_values.iter().copied().reduce(f64::max)
    }

    pub fn median_confidence(&self) -> Option<f64> {
        let mut values = self.confidence_values.clone();
        values.sort_by(f64::total_cmp);
        let middle = values.len() / 2;
        match values.len() {
            0 => None,
            length if length % 2 == 0 => Some((values[middle - 1] + values[middle]) / 2.0),
            _ => Some(values[middle]),
        }
    }

    pub fn confidence_distribution(&self) -> [usize; 5] {
        let mut bins = [0; 5];
        for confidence in &self.confidence_values {
            let index = if *confidence >= 0.8 {
                4
            } else if *confidence >= 0.6 {
                3
            } else if *confidence >= 0.4 {
                2
            } else if *confidence >= 0.2 {
                1
            } else {
                0
            };
            bins[index] += 1;
        }
        bins
    }

    pub fn register_decision_latency(&mut self, latency_ms: u64) {
        self.decision_latency_values.push(latency_ms);
    }

    pub fn p50_latency_ms(&self) -> u64 {
        percentile(&self.decision_latency_values, 0.50)
    }

    pub fn p95_latency_ms(&self) -> u64 {
        percentile(&self.decision_latency_values, 0.95)
    }
}

fn percentile(values: &[u64], ratio: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() as f64 * ratio).ceil() as usize).saturating_sub(1);
    sorted[rank.min(sorted.len() - 1)]
}

fn average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn sum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a + b),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct MarketAiDecision {
    pub action: &'static str,
    pub desired_position_pct: Option<f64>,
    pub confidence: Option<f64>,
    pub reason: String,
    pub reason_code: Option<String>,
    pub lifecycle_action: Option<String>,
    pub intent: Option<String>,
    pub generated_target_exposure_pct: Option<f64>,
    pub position_sizing_version: Option<String>,
    pub call_status: &'static str,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub latency_ms: u64,
    pub attempts: usize,
    pub fallback_used: bool,
    pub input_snapshot_json: Option<String>,
    pub first_failure_type: Option<String>,
    pub fallback_reason: Option<String>,
    pub attempt_audits: Vec<crate::database::market_ai_reliability::DecisionAttemptAudit>,
    pub runtime: MarketAiRuntimeMetrics,
}

pub struct MarketAIAgent;

impl MarketAIAgent {
    pub fn no_call(config: &MarketAiConfig) -> MarketAiDecision {
        let mut runtime = MarketAiRuntimeMetrics::default();
        runtime.no_llm_call_count = 1;
        MarketAiDecision {
            action: "HOLD",
            desired_position_pct: None,
            confidence: None,
            reason: "Cadência do AI Agent: nenhuma chamada ao LLM neste candle".into(),
            reason_code: None,
            lifecycle_action: None,
            intent: None,
            generated_target_exposure_pct: None,
            position_sizing_version: None,
            call_status: "NO_LLM_CALL",
            provider: config.provider.clone(),
            model: config.model.clone(),
            prompt_version: config.prompt_version.clone(),
            latency_ms: 0,
            attempts: 0,
            fallback_used: false,
            input_snapshot_json: None,
            first_failure_type: None,
            fallback_reason: None,
            attempt_audits: Vec::new(),
            runtime,
        }
    }

    pub fn decide<T: Serialize>(
        provider: &dyn MarketAiProvider,
        snapshot: &T,
        config: &MarketAiConfig,
    ) -> MarketAiDecision {
        let snapshot_json = serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".into());
        let prompt = MarketAiPrompt {
            system: prompt_for_version(&config.prompt_version)
                .unwrap_or(SYSTEM_PROMPT)
                .into(),
            snapshot_json: snapshot_json.clone(),
            structured_output_schema: None,
            retry_instruction: None,
        };
        let mut runtime = MarketAiRuntimeMetrics::default();
        let mut final_status = "INVALID";
        let mut final_model = config.model.clone();
        for attempt in 0..=config.max_retries {
            if attempt > 0 {
                runtime.retry_count += 1;
            }
            runtime.call_count += 1;
            let started = Instant::now();
            let response = provider.complete(prompt.clone(), config);
            let latency = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            runtime.total_latency_ms += latency;
            runtime.max_latency_ms = runtime.max_latency_ms.max(latency);
            match response {
                Ok(response) => {
                    final_model = response.model.clone();
                    runtime.input_tokens =
                        sum_optional(runtime.input_tokens, response.input_tokens);
                    runtime.output_tokens =
                        sum_optional(runtime.output_tokens, response.output_tokens);
                    match parse_decision(&response.content, &config.prompt_version) {
                        Ok(parsed) => {
                            runtime.successful_call_count += 1;
                            runtime.register_valid_decision(parsed.0, parsed.2);
                            runtime.average_latency_ms =
                                runtime.total_latency_ms as f64 / runtime.call_count as f64;
                            return MarketAiDecision {
                                action: parsed.0,
                                desired_position_pct: Some(parsed.1),
                                confidence: Some(parsed.2),
                                reason: parsed.3,
                                reason_code: parsed.4,
                                lifecycle_action: parsed.5,
                                intent: None,
                                generated_target_exposure_pct: None,
                                position_sizing_version: None,
                                call_status: "VALID",
                                provider: config.provider.clone(),
                                model: final_model,
                                prompt_version: config.prompt_version.clone(),
                                latency_ms: runtime.total_latency_ms,
                                attempts: runtime.call_count,
                                fallback_used: false,
                                input_snapshot_json: Some(snapshot_json),
                                first_failure_type: None,
                                fallback_reason: None,
                                attempt_audits: Vec::new(),
                                runtime,
                            };
                        }
                        Err(()) => {
                            runtime.invalid_response_count += 1;
                            final_status = "INVALID";
                        }
                    }
                }
                Err(error) if error.kind == MarketAiProviderErrorKind::Timeout => {
                    runtime.timeout_count += 1;
                    final_status = "TIMEOUT";
                }
                Err(_) => {
                    final_status = "OFFLINE";
                }
            }
        }
        runtime.fallback_count = 1;
        runtime.average_latency_ms = if runtime.call_count == 0 {
            0.0
        } else {
            runtime.total_latency_ms as f64 / runtime.call_count as f64
        };
        MarketAiDecision {
            action: "HOLD",
            desired_position_pct: None,
            confidence: None,
            reason: format!("Fallback seguro HOLD: {final_status}"),
            reason_code: None,
            lifecycle_action: None,
            intent: None,
            generated_target_exposure_pct: None,
            position_sizing_version: None,
            call_status: final_status,
            provider: config.provider.clone(),
            model: final_model,
            prompt_version: config.prompt_version.clone(),
            latency_ms: runtime.total_latency_ms,
            attempts: runtime.call_count,
            fallback_used: true,
            input_snapshot_json: Some(snapshot_json),
            first_failure_type: None,
            fallback_reason: Some("AI_OUTPUT_FALLBACK".into()),
            attempt_audits: Vec::new(),
            runtime,
        }
    }

    pub fn decide_v4_1(
        provider: &dyn MarketAiProvider,
        snapshot: &MarketAiSnapshotV4,
        config: &MarketAiConfig,
    ) -> MarketAiDecision {
        use crate::database::market_ai_reliability::{
            self, DecisionAttemptAudit, DecisionValidationStage, InvalidOutputType,
            SLOW_CALL_THRESHOLD_MS,
        };

        let snapshot_json = serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".into());
        let mut runtime = MarketAiRuntimeMetrics::default();
        let mut audits = Vec::new();
        let mut first_failure = None::<InvalidOutputType>;
        let mut final_status = "INVALID";
        let mut final_model = config.model.clone();

        for attempt in 0..=config.max_retries {
            if attempt > 0 {
                runtime.retry_count += 1;
            }
            runtime.call_count += 1;
            let prompt = MarketAiPrompt {
                system: SYSTEM_PROMPT_V4_1.into(),
                snapshot_json: snapshot_json.clone(),
                structured_output_schema: provider
                    .supports_structured_output()
                    .then(market_ai_reliability::output_schema),
                retry_instruction: (attempt > 0).then(|| {
                    market_ai_reliability::retry_instruction(
                        first_failure.unwrap_or(InvalidOutputType::UnknownInvalid),
                    )
                }),
            };
            let started = Instant::now();
            let response = provider.complete(prompt, config);
            let latency = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            runtime.total_latency_ms += latency;
            runtime.max_latency_ms = runtime.max_latency_ms.max(latency);
            if attempt == 0 {
                runtime.first_attempt_total_latency_ms += latency;
            } else {
                runtime.retry_total_latency_ms += latency;
            }
            if latency > SLOW_CALL_THRESHOLD_MS {
                runtime.slow_call_count += 1;
            }

            match response {
                Ok(response) => {
                    final_model = response.model.clone();
                    runtime.input_tokens = sum_optional(runtime.input_tokens, response.input_tokens);
                    runtime.output_tokens = sum_optional(runtime.output_tokens, response.output_tokens);
                    match market_ai_reliability::validate(
                        &response.content,
                        snapshot.position.state,
                        snapshot.position.exposure_pct,
                    ) {
                        Ok(parsed) => {
                            audits.push(DecisionAttemptAudit {
                                attempt_number: attempt + 1,
                                raw_response: Some(response.content),
                                status: "VALID".into(),
                                error_type: None,
                                error_field: None,
                                error_value: None,
                                error_message: None,
                                normalized_from_wrapped_json: parsed.normalized_from_wrapped_json,
                                latency_ms: latency,
                                stages: parsed.stages,
                            });
                            runtime.successful_call_count += 1;
                            runtime.final_valid_count = 1;
                            if attempt == 0 {
                                runtime.first_pass_valid_count = 1;
                            } else {
                                runtime.retry_recovered_count = 1;
                            }
                            runtime.register_valid_decision(
                                parsed.action.internal_action(),
                                parsed.confidence,
                            );
                            runtime.register_decision_latency(runtime.total_latency_ms);
                            runtime.average_latency_ms =
                                runtime.total_latency_ms as f64 / runtime.call_count as f64;
                            return MarketAiDecision {
                                action: parsed.action.internal_action(),
                                desired_position_pct: Some(parsed.target_exposure_pct),
                                confidence: Some(parsed.confidence),
                                reason: parsed.reason,
                                reason_code: Some(parsed.reason_code),
                                lifecycle_action: Some(parsed.action.as_str().into()),
                                intent: None,
                                generated_target_exposure_pct: None,
                                position_sizing_version: None,
                                call_status: "VALID",
                                provider: config.provider.clone(),
                                model: final_model,
                                prompt_version: config.prompt_version.clone(),
                                latency_ms: runtime.total_latency_ms,
                                attempts: runtime.call_count,
                                fallback_used: false,
                                input_snapshot_json: Some(snapshot_json),
                                first_failure_type: first_failure.map(|value| value.as_str().into()),
                                fallback_reason: None,
                                attempt_audits: audits,
                                runtime,
                            };
                        }
                        Err(error) => {
                            runtime.invalid_response_count += 1;
                            first_failure.get_or_insert(error.error_type);
                            final_status = "INVALID";
                            let error_value = market_ai_reliability::audit_error_value(
                                &response.content,
                                error.error_field.as_deref(),
                            );
                            audits.push(DecisionAttemptAudit {
                                attempt_number: attempt + 1,
                                raw_response: Some(response.content),
                                status: "INVALID".into(),
                                error_type: Some(error.error_type.as_str().into()),
                                error_field: error.error_field,
                                error_value,
                                error_message: Some(error.message),
                                normalized_from_wrapped_json: error.normalized_from_wrapped_json,
                                latency_ms: latency,
                                stages: error.stages,
                            });
                        }
                    }
                }
                Err(error) => {
                    let message = if error.kind == MarketAiProviderErrorKind::Timeout {
                        runtime.timeout_count += 1;
                        final_status = "TIMEOUT";
                        "O provider excedeu o tempo limite"
                    } else {
                        final_status = "OFFLINE";
                        "O provider local está indisponível"
                    };
                    first_failure.get_or_insert(InvalidOutputType::ProviderError);
                    audits.push(DecisionAttemptAudit {
                        attempt_number: attempt + 1,
                        raw_response: None,
                        status: "PROVIDER_ERROR".into(),
                        error_type: Some(InvalidOutputType::ProviderError.as_str().into()),
                        error_field: None,
                        error_value: None,
                        error_message: Some(message.into()),
                        normalized_from_wrapped_json: false,
                        latency_ms: latency,
                        stages: vec![DecisionValidationStage {
                            stage: "PROVIDER".into(),
                            success: false,
                            error_code: Some(InvalidOutputType::ProviderError.as_str().into()),
                            message: Some(message.into()),
                        }],
                    });
                }
            }
        }

        let fallback = if snapshot.position.state == crate::database::market_positions::PositionState::Flat {
            crate::database::market_positions::LifecycleAction::StayFlat
        } else {
            crate::database::market_positions::LifecycleAction::HoldPosition
        };
        let fallback_target = if fallback == crate::database::market_positions::LifecycleAction::StayFlat {
            0.0
        } else {
            snapshot.position.exposure_pct
        };
        runtime.fallback_count = 1;
        runtime.system_fallback_count = 1;
        runtime.final_invalid_count = 1;
        runtime.register_decision_latency(runtime.total_latency_ms);
        runtime.average_latency_ms = if runtime.call_count == 0 {
            0.0
        } else {
            runtime.total_latency_ms as f64 / runtime.call_count as f64
        };
        let failure_name = first_failure
            .map(InvalidOutputType::as_str)
            .unwrap_or("UNKNOWN_INVALID");
        MarketAiDecision {
            action: fallback.internal_action(),
            desired_position_pct: Some(fallback_target),
            confidence: None,
            reason: format!("Fallback semântico do sistema após {failure_name}"),
            reason_code: None,
            lifecycle_action: Some(fallback.as_str().into()),
            intent: None,
            generated_target_exposure_pct: None,
            position_sizing_version: None,
            call_status: final_status,
            provider: config.provider.clone(),
            model: final_model,
            prompt_version: config.prompt_version.clone(),
            latency_ms: runtime.total_latency_ms,
            attempts: runtime.call_count,
            fallback_used: true,
            input_snapshot_json: Some(snapshot_json),
            first_failure_type: Some(failure_name.into()),
            fallback_reason: Some("AI_OUTPUT_FALLBACK".into()),
            attempt_audits: audits,
            runtime,
        }
    }

    pub fn position_context_error(
        config: &MarketAiConfig,
        current_exposure_pct: f64,
        is_flat: bool,
    ) -> MarketAiDecision {
        let generated = crate::database::market_positions::generate_from_intent(
            if is_flat {
                crate::database::market_positions::PositionState::Flat
            } else {
                crate::database::market_positions::PositionState::LongOpen
            },
            current_exposure_pct,
            crate::database::market_positions::PositionIntent::Hold,
            &config.position_sizing,
        )
        .ok();
        let mut runtime = MarketAiRuntimeMetrics::default();
        runtime.final_invalid_count = 1;
        runtime.fallback_count = 1;
        runtime.system_fallback_count = 1;
        MarketAiDecision {
            action: "HOLD",
            desired_position_pct: Some(current_exposure_pct),
            confidence: None,
            reason: "Fallback seguro: POSITION_CONTEXT_ERROR".into(),
            reason_code: None,
            lifecycle_action: generated
                .as_ref()
                .map(|value| value.lifecycle_action.as_str().into()),
            intent: Some("HOLD".into()),
            generated_target_exposure_pct: Some(current_exposure_pct),
            position_sizing_version: Some(config.position_sizing.version.clone()),
            call_status: "INVALID",
            provider: config.provider.clone(),
            model: config.model.clone(),
            prompt_version: config.prompt_version.clone(),
            latency_ms: 0,
            attempts: 0,
            fallback_used: true,
            input_snapshot_json: None,
            first_failure_type: Some("POSITION_CONTEXT_ERROR".into()),
            fallback_reason: Some("POSITION_CONTEXT_ERROR".into()),
            attempt_audits: Vec::new(),
            runtime,
        }
    }

    pub fn decide_v4_2(
        provider: &dyn MarketAiProvider,
        snapshot: &MarketAiSnapshotV42,
        position_state: crate::database::market_positions::PositionState,
        config: &MarketAiConfig,
    ) -> MarketAiDecision {
        use crate::database::market_ai_reliability::{
            self, DecisionAttemptAudit, DecisionValidationStage, InvalidOutputType,
            SLOW_CALL_THRESHOLD_MS,
        };

        let snapshot_json = serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".into());
        let current_exposure = snapshot.position.current_exposure_pct;
        let mut runtime = MarketAiRuntimeMetrics::default();
        let mut audits = Vec::new();
        let mut first_failure = None::<InvalidOutputType>;
        let mut final_status = "INVALID";
        let mut final_model = config.model.clone();

        for attempt in 0..=config.max_retries {
            if attempt > 0 {
                runtime.retry_count += 1;
            }
            runtime.call_count += 1;
            let prompt = MarketAiPrompt {
                system: SYSTEM_PROMPT_V4_2.into(),
                snapshot_json: snapshot_json.clone(),
                structured_output_schema: provider
                    .supports_structured_output()
                    .then(market_ai_reliability::output_schema_v4_2),
                retry_instruction: (attempt > 0).then(|| {
                    market_ai_reliability::retry_instruction(
                        first_failure.unwrap_or(InvalidOutputType::UnknownInvalid),
                    )
                }),
            };
            let started = Instant::now();
            let response = provider.complete(prompt, config);
            let latency = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
            runtime.total_latency_ms += latency;
            runtime.max_latency_ms = runtime.max_latency_ms.max(latency);
            if attempt == 0 {
                runtime.first_attempt_total_latency_ms += latency;
            } else {
                runtime.retry_total_latency_ms += latency;
            }
            if latency > SLOW_CALL_THRESHOLD_MS {
                runtime.slow_call_count += 1;
            }
            match response {
                Ok(response) => {
                    final_model = response.model.clone();
                    runtime.input_tokens = sum_optional(runtime.input_tokens, response.input_tokens);
                    runtime.output_tokens = sum_optional(runtime.output_tokens, response.output_tokens);
                    match market_ai_reliability::validate_v4_2(&response.content, position_state) {
                        Ok(parsed) => {
                            let generated = match crate::database::market_positions::generate_from_intent(
                                position_state,
                                current_exposure,
                                parsed.intent,
                                &config.position_sizing,
                            ) {
                                Ok(value) => value,
                                Err(_) => {
                                    first_failure.get_or_insert(InvalidOutputType::PositionContextError);
                                    break;
                                }
                            };
                            audits.push(DecisionAttemptAudit {
                                attempt_number: attempt + 1,
                                raw_response: Some(response.content),
                                status: "VALID".into(),
                                error_type: None,
                                error_field: None,
                                error_value: None,
                                error_message: None,
                                normalized_from_wrapped_json: parsed.normalized_from_wrapped_json,
                                latency_ms: latency,
                                stages: parsed.stages,
                            });
                            runtime.successful_call_count += 1;
                            runtime.final_valid_count = 1;
                            if attempt == 0 {
                                runtime.first_pass_valid_count = 1;
                            } else {
                                runtime.retry_recovered_count = 1;
                            }
                            runtime.register_valid_decision(
                                generated.lifecycle_action.internal_action(),
                                parsed.confidence,
                            );
                            runtime.register_decision_latency(runtime.total_latency_ms);
                            runtime.average_latency_ms =
                                runtime.total_latency_ms as f64 / runtime.call_count as f64;
                            return MarketAiDecision {
                                action: generated.lifecycle_action.internal_action(),
                                desired_position_pct: Some(generated.generated_target_exposure_pct),
                                confidence: Some(parsed.confidence),
                                reason: parsed.reason,
                                reason_code: Some(parsed.reason_code),
                                lifecycle_action: Some(generated.lifecycle_action.as_str().into()),
                                intent: Some(parsed.intent.as_str().into()),
                                generated_target_exposure_pct: Some(generated.generated_target_exposure_pct),
                                position_sizing_version: Some(generated.sizing_config_version),
                                call_status: "VALID",
                                provider: config.provider.clone(),
                                model: final_model,
                                prompt_version: config.prompt_version.clone(),
                                latency_ms: runtime.total_latency_ms,
                                attempts: runtime.call_count,
                                fallback_used: false,
                                input_snapshot_json: Some(snapshot_json),
                                first_failure_type: first_failure.map(|value| value.as_str().into()),
                                fallback_reason: None,
                                attempt_audits: audits,
                                runtime,
                            };
                        }
                        Err(error) => {
                            runtime.invalid_response_count += 1;
                            first_failure.get_or_insert(error.error_type);
                            let error_value = market_ai_reliability::audit_error_value(
                                &response.content,
                                error.error_field.as_deref(),
                            );
                            audits.push(DecisionAttemptAudit {
                                attempt_number: attempt + 1,
                                raw_response: Some(response.content),
                                status: "INVALID".into(),
                                error_type: Some(error.error_type.as_str().into()),
                                error_field: error.error_field,
                                error_value,
                                error_message: Some(error.message),
                                normalized_from_wrapped_json: error.normalized_from_wrapped_json,
                                latency_ms: latency,
                                stages: error.stages,
                            });
                        }
                    }
                }
                Err(error) => {
                    let message = if error.kind == MarketAiProviderErrorKind::Timeout {
                        runtime.timeout_count += 1;
                        final_status = "TIMEOUT";
                        "O provider excedeu o tempo limite"
                    } else {
                        final_status = "OFFLINE";
                        "O provider local está indisponível"
                    };
                    first_failure.get_or_insert(InvalidOutputType::ProviderError);
                    audits.push(DecisionAttemptAudit {
                        attempt_number: attempt + 1,
                        raw_response: None,
                        status: "PROVIDER_ERROR".into(),
                        error_type: Some(InvalidOutputType::ProviderError.as_str().into()),
                        error_field: None,
                        error_value: None,
                        error_message: Some(message.into()),
                        normalized_from_wrapped_json: false,
                        latency_ms: latency,
                        stages: vec![DecisionValidationStage {
                            stage: "PROVIDER".into(),
                            success: false,
                            error_code: Some(InvalidOutputType::ProviderError.as_str().into()),
                            message: Some(message.into()),
                        }],
                    });
                }
            }
        }

        let generated = crate::database::market_positions::generate_from_intent(
            position_state,
            current_exposure,
            crate::database::market_positions::PositionIntent::Hold,
            &config.position_sizing,
        )
        .unwrap_or(crate::database::market_positions::GeneratedPositionDecision {
            intent: crate::database::market_positions::PositionIntent::Hold,
            lifecycle_action: if position_state == crate::database::market_positions::PositionState::Flat {
                crate::database::market_positions::LifecycleAction::StayFlat
            } else {
                crate::database::market_positions::LifecycleAction::HoldPosition
            },
            previous_exposure_pct: current_exposure,
            generated_target_exposure_pct: current_exposure,
            sizing_config_version: config.position_sizing.version.clone(),
        });
        runtime.fallback_count = 1;
        runtime.system_fallback_count = 1;
        runtime.final_invalid_count = 1;
        runtime.register_decision_latency(runtime.total_latency_ms);
        runtime.average_latency_ms = if runtime.call_count == 0 { 0.0 } else { runtime.total_latency_ms as f64 / runtime.call_count as f64 };
        let failure_name = first_failure.map(InvalidOutputType::as_str).unwrap_or("UNKNOWN_INVALID");
        MarketAiDecision {
            action: generated.lifecycle_action.internal_action(),
            desired_position_pct: Some(generated.generated_target_exposure_pct),
            confidence: None,
            reason: format!("Fallback semântico do sistema após {failure_name}"),
            reason_code: None,
            lifecycle_action: Some(generated.lifecycle_action.as_str().into()),
            intent: Some("HOLD".into()),
            generated_target_exposure_pct: Some(generated.generated_target_exposure_pct),
            position_sizing_version: Some(generated.sizing_config_version),
            call_status: final_status,
            provider: config.provider.clone(),
            model: final_model,
            prompt_version: config.prompt_version.clone(),
            latency_ms: runtime.total_latency_ms,
            attempts: runtime.call_count,
            fallback_used: true,
            input_snapshot_json: Some(snapshot_json),
            first_failure_type: Some(failure_name.into()),
            fallback_reason: Some("AI_OUTPUT_FALLBACK".into()),
            attempt_audits: audits,
            runtime,
        }
    }
}

fn validate_fields(
    action: &str,
    desired: f64,
    confidence: f64,
    reason: &str,
) -> Result<&'static str, ()> {
    let action = match action {
        "BUY" => "BUY",
        "SELL" => "SELL",
        "HOLD" => "HOLD",
        _ => return Err(()),
    };
    if !desired.is_finite()
        || !(0.0..=100.0).contains(&desired)
        || !confidence.is_finite()
        || !(0.0..=1.0).contains(&confidence)
        || reason.trim().is_empty()
        || reason.chars().count() > 240
    {
        return Err(());
    }
    Ok(action)
}

fn parse_decision(
    content: &str,
    prompt_version: &str,
) -> Result<
    (
        &'static str,
        f64,
        f64,
        String,
        Option<String>,
        Option<String>,
    ),
    (),
> {
    if prompt_version == PROMPT_V4_VERSION {
        use std::str::FromStr;
        let parsed: StructuredDecisionV4 = serde_json::from_str(content.trim()).map_err(|_| ())?;
        let lifecycle =
            crate::database::market_positions::LifecycleAction::from_str(&parsed.action)
                .map_err(|_| ())?;
        validate_fields(
            lifecycle.internal_action(),
            parsed.target_exposure_pct,
            parsed.confidence,
            &parsed.reason,
        )?;
        if !REASON_CODES_V4.contains(&parsed.reason_code.as_str()) {
            return Err(());
        }
        return Ok((
            lifecycle.internal_action(),
            parsed.target_exposure_pct,
            parsed.confidence,
            parsed.reason.trim().into(),
            Some(parsed.reason_code),
            Some(lifecycle.as_str().into()),
        ));
    }
    if prompt_version == PROMPT_V3_VERSION {
        let parsed: StructuredDecisionV3 = serde_json::from_str(content.trim()).map_err(|_| ())?;
        let action = validate_fields(
            &parsed.action,
            parsed.desired_position_pct,
            parsed.confidence,
            &parsed.reason,
        )?;
        if !REASON_CODES.contains(&parsed.reason_code.as_str()) {
            return Err(());
        }
        return Ok((
            action,
            parsed.desired_position_pct,
            parsed.confidence,
            parsed.reason.trim().into(),
            Some(parsed.reason_code),
            None,
        ));
    }
    let parsed: StructuredDecision = serde_json::from_str(content.trim()).map_err(|_| ())?;
    let action = validate_fields(
        &parsed.action,
        parsed.desired_position_pct,
        parsed.confidence,
        &parsed.reason,
    )?;
    Ok((
        action,
        parsed.desired_position_pct,
        parsed.confidence,
        parsed.reason.trim().into(),
        None,
        None,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeProvider {
        responses: Mutex<Vec<Result<MarketAiProviderResponse, MarketAiProviderError>>>,
    }
    impl MarketAiProvider for FakeProvider {
        fn complete(
            &self,
            _: MarketAiPrompt,
            _: &MarketAiConfig,
        ) -> Result<MarketAiProviderResponse, MarketAiProviderError> {
            self.responses.lock().unwrap().remove(0)
        }
    }
    fn config() -> MarketAiConfig {
        MarketAiConfig {
            agent_id: AI_AGENT_ID.into(),
            provider: "ollama".into(),
            model: "fake:test".into(),
            prompt_version: PROMPT_VERSION.into(),
            temperature: 0.1,
            decision_interval: 5,
            timeout_ms: 5000,
            max_retries: 1,
            position_sizing: Default::default(),
        }
    }
    fn snapshot() -> MarketAiSnapshot {
        MarketAiSnapshot {
            timestamp: "T".into(),
            asset: "TST".into(),
            timeframe: "1D".into(),
            price: 10.0,
            features: MarketAiFeatures {
                return_1: None,
                return_5: None,
                sma_short: None,
                sma_long: None,
                volatility: None,
            },
            portfolio: MarketAiPortfolio {
                cash: 100.0,
                equity: 100.0,
                exposure_pct: 0.0,
                current_position: false,
                entry_price: None,
            },
            risk_context: MarketAiRiskContext {
                max_position_pct: 50.0,
                max_total_exposure_pct: 50.0,
                allow_short: false,
                allow_leverage: false,
            },
            memory: MarketAiMemory {
                last_action: None,
                recent_decision_count: 0,
            },
        }
    }
    fn ok(content: &str) -> Result<MarketAiProviderResponse, MarketAiProviderError> {
        Ok(MarketAiProviderResponse {
            content: content.into(),
            model: "fake:test".into(),
            input_tokens: None,
            output_tokens: None,
        })
    }

    #[test]
    fn accepts_buy_sell_and_hold_schema() {
        for action in ["BUY", "SELL", "HOLD"] {
            let raw = format!(
                r#"{{"action":"{action}","desired_position_pct":20,"confidence":0.8,"reason":"test"}}"#
            );
            assert_eq!(parse_decision(&raw, PROMPT_VERSION).unwrap().0, action);
        }
    }
    #[test]
    fn rejects_invalid_schema_values_and_unknown_fields() {
        assert!(parse_decision("BUY EVERYTHING", PROMPT_VERSION).is_err());
        assert!(parse_decision(
            r#"{"action":"WAIT","desired_position_pct":0,"confidence":1,"reason":"x"}"#,
            PROMPT_VERSION
        )
        .is_err());
        assert!(parse_decision(
            r#"{"action":"BUY","desired_position_pct":-1,"confidence":2,"reason":"x"}"#,
            PROMPT_VERSION
        )
        .is_err());
        assert!(parse_decision(
            r#"{"action":"HOLD","desired_position_pct":0,"confidence":1,"reason":"x","extra":1}"#,
            PROMPT_VERSION
        )
        .is_err());
    }
    #[test]
    fn retries_once_then_uses_safe_fallback() {
        let provider = FakeProvider {
            responses: Mutex::new(vec![ok("invalid"), ok("invalid")]),
        };
        let result = MarketAIAgent::decide(&provider, &snapshot(), &config());
        assert_eq!(result.action, "HOLD");
        assert!(result.fallback_used);
        assert_eq!(result.runtime.call_count, 2);
        assert_eq!(result.runtime.retry_count, 1);
        assert_eq!(result.runtime.invalid_response_count, 2);
    }
    #[test]
    fn timeout_and_offline_are_audited() {
        for kind in [
            MarketAiProviderErrorKind::Timeout,
            MarketAiProviderErrorKind::Offline,
        ] {
            let mut cfg = config();
            cfg.max_retries = 0;
            let provider = FakeProvider {
                responses: Mutex::new(vec![Err(MarketAiProviderError { kind })]),
            };
            let result = MarketAIAgent::decide(&provider, &snapshot(), &cfg);
            assert_eq!(result.action, "HOLD");
            assert!(result.fallback_used);
        }
    }

    #[test]
    fn v1_and_v2_prompts_remain_distinct_and_available() {
        assert!(SYSTEM_PROMPT.contains("HOLD is valid"));
        assert!(SYSTEM_PROMPT_V2.contains("Do not require every indicator to agree"));
        assert_ne!(SYSTEM_PROMPT, SYSTEM_PROMPT_V2);
        let mut v2 = config();
        v2.agent_id = AI_AGENT_V2_ID.into();
        v2.prompt_version = PROMPT_V2_VERSION.into();
        assert!(v2.validate().is_ok());
    }

    #[test]
    fn decision_distribution_keeps_no_call_separate_from_llm_hold() {
        let provider = FakeProvider {
            responses: Mutex::new(vec![ok(
                r#"{"action":"HOLD","desired_position_pct":0,"confidence":0.6,"reason":"balanced"}"#,
            )]),
        };
        let decision = MarketAIAgent::decide(&provider, &snapshot(), &config());
        let no_call = MarketAIAgent::no_call(&config());
        assert_eq!(decision.runtime.llm_hold_count, 1);
        assert_eq!(decision.runtime.no_llm_call_count, 0);
        assert_eq!(no_call.runtime.llm_hold_count, 0);
        assert_eq!(no_call.runtime.no_llm_call_count, 1);
    }

    #[test]
    fn confidence_statistics_and_bins_are_mathematically_stable() {
        let mut metrics = MarketAiRuntimeMetrics::default();
        metrics.register_valid_decision("BUY", 0.1);
        metrics.register_valid_decision("SELL", 0.3);
        metrics.register_valid_decision("HOLD", 0.7);
        metrics.register_valid_decision("HOLD", 0.9);
        assert_eq!(metrics.average_confidence(), Some(0.5));
        assert_eq!(metrics.median_confidence(), Some(0.5));
        assert_eq!(metrics.min_confidence(), Some(0.1));
        assert_eq!(metrics.max_confidence(), Some(0.9));
        assert_eq!(metrics.confidence_distribution(), [1, 1, 0, 1, 1]);
    }

    #[test]
    fn v3_requires_a_controlled_reason_code() {
        let valid = r#"{"action":"BUY","desired_position_pct":25,"confidence":0.73,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"aligned"}"#;
        assert_eq!(
            parse_decision(valid, PROMPT_V3_VERSION)
                .unwrap()
                .4
                .as_deref(),
            Some("ALIGNED_BULLISH_SIGNAL")
        );
        assert!(parse_decision(
            &valid.replace("ALIGNED_BULLISH_SIGNAL", "INVENTED"),
            PROMPT_V3_VERSION
        )
        .is_err());
        assert!(parse_decision(r#"{"action":"BUY","desired_position_pct":101,"confidence":0.7,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"x"}"#,PROMPT_V3_VERSION).is_err());
    }

    #[test]
    fn v4_requires_lifecycle_action_and_total_target_exposure() {
        let valid = r#"{"action":"ENTER_LONG","target_exposure_pct":25,"confidence":0.73,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"aligned"}"#;
        let parsed = parse_decision(valid, PROMPT_V4_VERSION).unwrap();
        assert_eq!(parsed.0, "BUY");
        assert_eq!(parsed.1, 25.0);
        assert_eq!(parsed.5.as_deref(), Some("ENTER_LONG"));
        assert!(parse_decision(&valid.replace("ENTER_LONG", "BUY"), PROMPT_V4_VERSION).is_err());
        assert!(parse_decision(
            r#"{"action":"EXIT_LONG","target_exposure_pct":101,"confidence":0.7,"reason_code":"EXIT_SIGNAL","reason":"x"}"#,
            PROMPT_V4_VERSION
        )
        .is_err());
    }

    fn config_v4_1() -> MarketAiConfig {
        MarketAiConfig {
            agent_id: AI_AGENT_V4_1_ID.into(),
            provider: "ollama".into(),
            model: "fake:test".into(),
            prompt_version: PROMPT_V4_1_VERSION.into(),
            temperature: 0.1,
            decision_interval: 5,
            timeout_ms: 5_000,
            max_retries: 1,
            position_sizing: Default::default(),
        }
    }

    fn config_v4_2() -> MarketAiConfig {
        MarketAiConfig {
            agent_id: AI_AGENT_V4_2_ID.into(),
            provider: "ollama".into(),
            model: "fake:test".into(),
            prompt_version: PROMPT_V4_2_VERSION.into(),
            temperature: 0.1,
            decision_interval: 5,
            timeout_ms: 5_000,
            max_retries: 1,
            position_sizing: Default::default(),
        }
    }

    fn snapshot_v4(
        state: crate::database::market_positions::PositionState,
        exposure_pct: f64,
    ) -> MarketAiSnapshotV4 {
        let signals = crate::database::market_signals::evaluate(
            &crate::database::market_signals::MarketSignalInput::default(),
            &crate::database::market_signals::SignalEngineConfig::default(),
        )
        .unwrap();
        MarketAiSnapshotV4 {
            market: MarketAiMarket {
                timestamp: "T".into(),
                asset: "TST".into(),
                timeframe: "1D".into(),
                price: 100.0,
            },
            signals,
            regime: MarketAiRegime {
                trend: "SIDEWAYS".into(),
                volatility: "NORMAL".into(),
                engine_version: "REGIME_ENGINE_V1".into(),
            },
            position: crate::database::market_positions::PositionContext {
                state,
                quantity: if exposure_pct > 0.0 { 1.0 } else { 0.0 },
                exposure_pct,
                entry_price: (exposure_pct > 0.0).then_some(95.0),
                average_entry_price: (exposure_pct > 0.0).then_some(95.0),
                current_price: 100.0,
                unrealized_pnl_pct: (exposure_pct > 0.0).then_some(5.0),
                holding_candles: if exposure_pct > 0.0 { 10 } else { 0 },
                max_favorable_excursion_pct: None,
                max_adverse_excursion_pct: None,
                last_entry_timestamp: None,
                last_exit_timestamp: None,
                time_since_last_exit: None,
                reentry_count: 0,
            },
            risk_context: MarketAiRiskContext {
                max_position_pct: 50.0,
                max_total_exposure_pct: 50.0,
                allow_short: false,
                allow_leverage: false,
            },
        }
    }

    fn snapshot_v4_2(
        state: crate::database::market_positions::PositionState,
        exposure_pct: f64,
    ) -> MarketAiSnapshotV42 {
        let legacy = snapshot_v4(state, exposure_pct);
        MarketAiSnapshotV42 {
            market: legacy.market,
            signals: legacy.signals,
            regime: legacy.regime,
            position: MarketAiPositionContextV42::from_position(&legacy.position).unwrap(),
            risk_context: legacy.risk_context,
        }
    }

    #[test]
    fn v4_1_structured_retry_recovers_and_separates_validity_metrics() {
        let provider = FakeProvider {
            responses: Mutex::new(vec![
                ok("{ action: ENTER_LONG }"),
                ok(r#"{"action":"ENTER_LONG","target_exposure_pct":25,"confidence":0.7,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"aligned"}"#),
            ]),
        };
        let result = MarketAIAgent::decide_v4_1(
            &provider,
            &snapshot_v4(crate::database::market_positions::PositionState::Flat, 0.0),
            &config_v4_1(),
        );
        assert_eq!(result.lifecycle_action.as_deref(), Some("ENTER_LONG"));
        assert!(!result.fallback_used);
        assert_eq!(result.runtime.first_pass_valid_count, 0);
        assert_eq!(result.runtime.retry_recovered_count, 1);
        assert_eq!(result.runtime.final_valid_count, 1);
        assert_eq!(result.runtime.final_invalid_count, 0);
        assert_eq!(result.attempt_audits.len(), 2);
        assert_eq!(result.first_failure_type.as_deref(), Some("INVALID_JSON"));
    }

    #[test]
    fn v4_1_first_pass_valid_is_not_counted_as_retry_or_fallback() {
        let provider = FakeProvider { responses: Mutex::new(vec![ok(r#"{"action":"STAY_FLAT","target_exposure_pct":0,"confidence":0.55,"reason_code":"INSUFFICIENT_SIGNAL","reason":"stay flat"}"#)]) };
        let result = MarketAIAgent::decide_v4_1(&provider, &snapshot_v4(crate::database::market_positions::PositionState::Flat, 0.0), &config_v4_1());
        assert_eq!(result.runtime.call_count, 1);
        assert_eq!(result.runtime.first_pass_valid_count, 1);
        assert_eq!(result.runtime.retry_recovered_count, 0);
        assert_eq!(result.runtime.final_valid_count, 1);
        assert_eq!(result.runtime.system_fallback_count, 0);
    }

    #[test]
    fn v4_1_retry_failure_uses_flat_and_long_stateful_fallbacks() {
        for (state, exposure, expected_action, expected_target) in [
            (crate::database::market_positions::PositionState::Flat, 0.0, "STAY_FLAT", 0.0),
            (crate::database::market_positions::PositionState::LongOpen, 37.0, "HOLD_POSITION", 37.0),
        ] {
            let provider = FakeProvider { responses: Mutex::new(vec![ok("invalid"), ok("invalid")]) };
            let result = MarketAIAgent::decide_v4_1(&provider, &snapshot_v4(state, exposure), &config_v4_1());
            assert_eq!(result.lifecycle_action.as_deref(), Some(expected_action));
            assert_eq!(result.desired_position_pct, Some(expected_target));
            assert_eq!(result.fallback_reason.as_deref(), Some("AI_OUTPUT_FALLBACK"));
            assert_eq!(result.runtime.final_invalid_count, 1);
            assert_eq!(result.runtime.system_fallback_count, 1);
            assert_eq!(result.runtime.retry_count, 1);
        }
    }

    #[test]
    fn v4_1_latency_percentiles_are_deterministic() {
        let mut metrics = MarketAiRuntimeMetrics::default();
        for latency in [100, 200, 300, 400, 5_001] {
            metrics.register_decision_latency(latency);
        }
        assert_eq!(metrics.p50_latency_ms(), 300);
        assert_eq!(metrics.p95_latency_ms(), 5_001);
    }

    #[test]
    fn v4_2_converts_intent_to_target_and_normalizes_confidence() {
        let provider = FakeProvider { responses: Mutex::new(vec![ok(r#"{"intent":"ENTER","confidence_pct":83,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"aligned"}"#)]) };
        let result = MarketAIAgent::decide_v4_2(
            &provider,
            &snapshot_v4_2(crate::database::market_positions::PositionState::Flat, 0.0),
            crate::database::market_positions::PositionState::Flat,
            &config_v4_2(),
        );
        assert_eq!(result.intent.as_deref(), Some("ENTER"));
        assert_eq!(result.lifecycle_action.as_deref(), Some("ENTER_LONG"));
        assert_eq!(result.generated_target_exposure_pct, Some(25.0));
        assert_eq!(result.confidence, Some(0.83));
        assert_eq!(result.position_sizing_version.as_deref(), Some("POSITION_SIZING_V1"));
        assert_eq!(result.runtime.final_valid_count, 1);
    }

    #[test]
    fn v4_2_uses_position_aware_hold_fallback() {
        for (state, exposure, expected_action) in [
            (crate::database::market_positions::PositionState::Flat, 0.0, "STAY_FLAT"),
            (crate::database::market_positions::PositionState::LongOpen, 40.0, "HOLD_POSITION"),
        ] {
            let provider = FakeProvider { responses: Mutex::new(vec![ok("invalid"), ok("invalid")]) };
            let result = MarketAIAgent::decide_v4_2(
                &provider,
                &snapshot_v4_2(state, exposure),
                state,
                &config_v4_2(),
            );
            assert_eq!(result.intent.as_deref(), Some("HOLD"));
            assert_eq!(result.lifecycle_action.as_deref(), Some(expected_action));
            assert_eq!(result.generated_target_exposure_pct, Some(exposure));
            assert_eq!(result.runtime.system_fallback_count, 1);
        }
    }

    #[test]
    fn v4_2_position_context_is_explicit_and_rejects_inconsistency() {
        let flat = snapshot_v4(crate::database::market_positions::PositionState::Flat, 0.0);
        let context = MarketAiPositionContextV42::from_position(&flat.position).unwrap();
        assert_eq!(context.state, "FLAT");
        assert_eq!(context.current_exposure_pct, 0.0);
        let mut corrupt = flat.position;
        corrupt.exposure_pct = 25.0;
        assert!(MarketAiPositionContextV42::from_position(&corrupt)
            .unwrap_err()
            .contains("POSITION_CONTEXT_ERROR"));
    }

    #[test]
    fn collapse_detectors_ignore_small_samples_and_detect_repetition() {
        assert!(!detect_degeneration(&vec![("HOLD".into(), 0.6); 19]).action_collapse);
        let collapsed = detect_degeneration(&vec![("HOLD".into(), 0.6); 20]);
        assert!(collapsed.action_collapse);
        assert!(collapsed.confidence_collapse);
        let diverse = (0..20)
            .map(|index| {
                (
                    if index % 2 == 0 { "BUY" } else { "HOLD" }.into(),
                    index as f64 / 20.0,
                )
            })
            .collect::<Vec<_>>();
        let diagnostics = detect_degeneration(&diverse);
        assert!(!diagnostics.action_collapse);
        assert!(!diagnostics.confidence_collapse);
    }
}
