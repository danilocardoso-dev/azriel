use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const TRIGGER_ENGINE_VERSION: &str = "MARKET_DECISION_TRIGGER_V1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTriggerConfig {
    pub periodic_candles: usize,
    pub cooldown_candles: usize,
    pub max_calls_per_experiment: usize,
    pub max_calls_per_session: usize,
}

impl Default for DecisionTriggerConfig {
    fn default() -> Self {
        Self {
            periodic_candles: 8,
            cooldown_candles: 2,
            max_calls_per_experiment: 200,
            max_calls_per_session: 32,
        }
    }
}

impl DecisionTriggerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.periodic_candles == 0
            || self.cooldown_candles > self.periodic_candles
            || self.max_calls_per_experiment == 0
            || self.max_calls_per_session == 0
        {
            return Err("configuração do Decision Trigger Engine inválida".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerSnapshot<'a> {
    pub candle_index: usize,
    pub session_id: &'a str,
    pub bias: &'a str,
    pub strength_bucket: &'a str,
    pub regime: &'a str,
    pub conflict: bool,
    pub position_risk_state: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDecision {
    pub should_evaluate: bool,
    pub trigger_reason: Option<&'static str>,
    pub skip_reason: Option<&'static str>,
    pub cooldown_remaining: usize,
    pub call_index: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct MarketDecisionTriggerEngine {
    previous: Option<(String, String, String, bool, String)>,
    last_call_index: Option<usize>,
    calls: usize,
    calls_by_session: HashMap<String, usize>,
}

impl MarketDecisionTriggerEngine {
    pub fn should_evaluate(
        &mut self,
        snapshot: &TriggerSnapshot<'_>,
        config: &DecisionTriggerConfig,
    ) -> TriggerDecision {
        let current = (
            snapshot.bias.into(),
            snapshot.strength_bucket.into(),
            snapshot.regime.into(),
            snapshot.conflict,
            snapshot.position_risk_state.into(),
        );
        let reason = match &self.previous {
            None => Some("INITIAL_STATE"),
            Some(previous) if previous.4 != current.4 => Some("POSITION_RISK_CHANGE"),
            Some(previous) if previous.3 && !current.3 => Some("CONFLICT_RESOLVED"),
            Some(previous) if previous.0 != current.0 => Some("BIAS_CHANGE"),
            Some(previous) if previous.1 != current.1 => Some("STRENGTH_CHANGE"),
            Some(previous) if previous.2 != current.2 => Some("REGIME_CHANGE"),
            _ if snapshot.candle_index % config.periodic_candles == 0 => Some("PERIODIC"),
            _ => None,
        };
        self.previous = Some(current);
        if reason.is_none() {
            return TriggerDecision {
                should_evaluate: false,
                trigger_reason: None,
                skip_reason: Some("NO_TRIGGER"),
                cooldown_remaining: 0,
                call_index: None,
            };
        }
        let session_calls = *self.calls_by_session.get(snapshot.session_id).unwrap_or(&0);
        if self.calls >= config.max_calls_per_experiment
            || session_calls >= config.max_calls_per_session
        {
            return TriggerDecision {
                should_evaluate: false,
                trigger_reason: reason,
                skip_reason: Some("COMPUTE_BUDGET"),
                cooldown_remaining: 0,
                call_index: None,
            };
        }
        if let Some(last) = self.last_call_index {
            let elapsed = snapshot.candle_index.saturating_sub(last);
            if elapsed <= config.cooldown_candles {
                return TriggerDecision {
                    should_evaluate: false,
                    trigger_reason: reason,
                    skip_reason: Some("COOLDOWN"),
                    cooldown_remaining: config.cooldown_candles + 1 - elapsed,
                    call_index: None,
                };
            }
        }
        self.calls += 1;
        *self
            .calls_by_session
            .entry(snapshot.session_id.into())
            .or_insert(0) += 1;
        self.last_call_index = Some(snapshot.candle_index);
        TriggerDecision {
            should_evaluate: true,
            trigger_reason: reason,
            skip_reason: None,
            cooldown_remaining: 0,
            call_index: Some(self.calls),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn snapshot(index: usize, bias: &'static str) -> TriggerSnapshot<'static> {
        TriggerSnapshot {
            candle_index: index,
            session_id: "2026-01-02",
            bias,
            strength_bucket: "MEDIUM",
            regime: "SIDEWAYS",
            conflict: false,
            position_risk_state: "NORMAL",
        }
    }
    #[test]
    fn triggers_initial_change_periodic_and_no_trigger() {
        let mut engine = MarketDecisionTriggerEngine::default();
        let config = DecisionTriggerConfig::default();
        assert!(
            engine
                .should_evaluate(&snapshot(0, "NEUTRAL"), &config)
                .should_evaluate
        );
        assert_eq!(
            engine
                .should_evaluate(&snapshot(1, "NEUTRAL"), &config)
                .skip_reason,
            Some("NO_TRIGGER")
        );
        assert_eq!(
            engine
                .should_evaluate(&snapshot(2, "BULLISH"), &config)
                .skip_reason,
            Some("COOLDOWN")
        );
        assert!(
            engine
                .should_evaluate(&snapshot(8, "BULLISH"), &config)
                .should_evaluate
        );
    }
    #[test]
    fn enforces_compute_budget() {
        let mut engine = MarketDecisionTriggerEngine::default();
        let config = DecisionTriggerConfig {
            periodic_candles: 1,
            cooldown_candles: 0,
            max_calls_per_experiment: 1,
            max_calls_per_session: 1,
        };
        assert!(
            engine
                .should_evaluate(&snapshot(0, "NEUTRAL"), &config)
                .should_evaluate
        );
        assert_eq!(
            engine
                .should_evaluate(&snapshot(1, "NEUTRAL"), &config)
                .skip_reason,
            Some("COMPUTE_BUDGET")
        );
    }
}
