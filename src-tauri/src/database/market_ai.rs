use crate::ollama::{self, OllamaMessage};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub const AI_AGENT_ID: &str = "ai-technical-v1";
pub const PROMPT_VERSION: &str = "MARKET_AI_AGENT_V1";
pub const MAX_AI_AGENTS: usize = 1;
pub const SYSTEM_PROMPT: &str = include_str!("../../prompts/market-lab-agent.txt");

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiConfig {
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub temperature: f64,
    pub decision_interval: usize,
    pub timeout_ms: u64,
    pub max_retries: usize,
}

impl MarketAiConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.provider != "ollama" || self.model.trim().is_empty() {
            return Err("configuração do AI Agent possui provider ou modelo inválido".into());
        }
        if self.prompt_version != PROMPT_VERSION || !(0.0..=0.3).contains(&self.temperature) {
            return Err("prompt version ou temperatura do AI Agent inválido".into());
        }
        if !(1..=100).contains(&self.decision_interval)
            || !(5_000..=180_000).contains(&self.timeout_ms)
            || self.max_retries > 1
        {
            return Err("cadência, timeout ou retry do AI Agent inválido".into());
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

#[derive(Debug, Clone)]
pub struct MarketAiPrompt {
    pub system: String,
    pub snapshot_json: String,
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
    fn complete(
        &self,
        prompt: MarketAiPrompt,
        config: &MarketAiConfig,
    ) -> Result<MarketAiProviderResponse, MarketAiProviderError> {
        let result = tauri::async_runtime::block_on(ollama::chat(
            &self.endpoint,
            &config.model,
            vec![
                OllamaMessage {
                    role: "system".into(),
                    content: prompt.system,
                },
                OllamaMessage {
                    role: "user".into(),
                    content: prompt.snapshot_json,
                },
            ],
            config.timeout_ms.div_ceil(1000),
            "market-agent",
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
        self.average_latency_ms = if self.call_count == 0 {
            0.0
        } else {
            self.total_latency_ms as f64 / self.call_count as f64
        };
    }
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
    pub call_status: &'static str,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub latency_ms: u64,
    pub attempts: usize,
    pub fallback_used: bool,
    pub runtime: MarketAiRuntimeMetrics,
}

pub struct MarketAIAgent;

impl MarketAIAgent {
    pub fn no_call(config: &MarketAiConfig) -> MarketAiDecision {
        MarketAiDecision {
            action: "HOLD",
            desired_position_pct: None,
            confidence: None,
            reason: "Cadência do AI Agent: nenhuma chamada ao LLM neste candle".into(),
            call_status: "NO_LLM_CALL",
            provider: config.provider.clone(),
            model: config.model.clone(),
            prompt_version: config.prompt_version.clone(),
            latency_ms: 0,
            attempts: 0,
            fallback_used: false,
            runtime: MarketAiRuntimeMetrics::default(),
        }
    }

    pub fn decide(
        provider: &dyn MarketAiProvider,
        snapshot: &MarketAiSnapshot,
        config: &MarketAiConfig,
    ) -> MarketAiDecision {
        let prompt = MarketAiPrompt {
            system: SYSTEM_PROMPT.into(),
            snapshot_json: serde_json::to_string(snapshot).unwrap_or_else(|_| "{}".into()),
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
                    match parse_decision(&response.content) {
                        Ok(parsed) => {
                            runtime.successful_call_count += 1;
                            runtime.average_latency_ms =
                                runtime.total_latency_ms as f64 / runtime.call_count as f64;
                            return MarketAiDecision {
                                action: parsed.0,
                                desired_position_pct: Some(parsed.1),
                                confidence: Some(parsed.2),
                                reason: parsed.3,
                                call_status: "VALID",
                                provider: config.provider.clone(),
                                model: final_model,
                                prompt_version: config.prompt_version.clone(),
                                latency_ms: runtime.total_latency_ms,
                                attempts: runtime.call_count,
                                fallback_used: false,
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
            call_status: final_status,
            provider: config.provider.clone(),
            model: final_model,
            prompt_version: config.prompt_version.clone(),
            latency_ms: runtime.total_latency_ms,
            attempts: runtime.call_count,
            fallback_used: true,
            runtime,
        }
    }
}

fn parse_decision(content: &str) -> Result<(&'static str, f64, f64, String), ()> {
    let parsed: StructuredDecision = serde_json::from_str(content.trim()).map_err(|_| ())?;
    let action = match parsed.action.as_str() {
        "BUY" => "BUY",
        "SELL" => "SELL",
        "HOLD" => "HOLD",
        _ => return Err(()),
    };
    if !parsed.desired_position_pct.is_finite()
        || parsed.desired_position_pct < 0.0
        || !parsed.confidence.is_finite()
        || !(0.0..=1.0).contains(&parsed.confidence)
        || parsed.reason.trim().is_empty()
        || parsed.reason.chars().count() > 240
    {
        return Err(());
    }
    Ok((
        action,
        parsed.desired_position_pct,
        parsed.confidence,
        parsed.reason.trim().into(),
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
            provider: "ollama".into(),
            model: "fake:test".into(),
            prompt_version: PROMPT_VERSION.into(),
            temperature: 0.1,
            decision_interval: 5,
            timeout_ms: 5000,
            max_retries: 1,
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
            assert_eq!(parse_decision(&raw).unwrap().0, action);
        }
    }
    #[test]
    fn rejects_invalid_schema_values_and_unknown_fields() {
        assert!(parse_decision("BUY EVERYTHING").is_err());
        assert!(parse_decision(
            r#"{"action":"WAIT","desired_position_pct":0,"confidence":1,"reason":"x"}"#
        )
        .is_err());
        assert!(parse_decision(
            r#"{"action":"BUY","desired_position_pct":-1,"confidence":2,"reason":"x"}"#
        )
        .is_err());
        assert!(parse_decision(
            r#"{"action":"HOLD","desired_position_pct":0,"confidence":1,"reason":"x","extra":1}"#
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
}
