use serde::{Deserialize, Serialize};

pub const LIFECYCLE_ENGINE_VERSION: &str = "POSITION_LIFECYCLE_CONFIG_V1";
pub const DETERIORATION_ENGINE_VERSION: &str = "POSITION_DETERIORATION_CONFIG_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleState {
    Flat,
    Opening,
    Long,
    Reducing,
    Closing,
    Closed,
}

impl LifecycleState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flat => "FLAT",
            Self::Opening => "OPENING",
            Self::Long => "LONG",
            Self::Reducing => "REDUCING",
            Self::Closing => "CLOSING",
            Self::Closed => "CLOSED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleEventType {
    Open,
    Increase,
    Hold,
    Reduce,
    Exit,
    ForcedExit,
}

impl LifecycleEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "OPEN",
            Self::Increase => "INCREASE",
            Self::Hold => "HOLD",
            Self::Reduce => "REDUCE",
            Self::Exit => "EXIT",
            Self::ForcedExit => "FORCED_EXIT",
        }
    }
}

pub struct PositionLifecycleEngine;

impl PositionLifecycleEngine {
    pub fn request(
        state: LifecycleState,
        event: LifecycleEventType,
    ) -> Result<LifecycleState, String> {
        match (state, event) {
            (LifecycleState::Flat, LifecycleEventType::Open) => Ok(LifecycleState::Opening),
            (LifecycleState::Long, LifecycleEventType::Increase | LifecycleEventType::Hold) => {
                Ok(LifecycleState::Long)
            }
            (LifecycleState::Long, LifecycleEventType::Reduce) => Ok(LifecycleState::Reducing),
            (LifecycleState::Long, LifecycleEventType::Exit | LifecycleEventType::ForcedExit) => {
                Ok(LifecycleState::Closing)
            }
            _ => Err(format!(
                "transição de lifecycle inválida: {} -> {}",
                state.as_str(),
                event.as_str()
            )),
        }
    }

    pub fn settle(
        state: LifecycleState,
        exposure_after_pct: f64,
    ) -> Result<LifecycleState, String> {
        match state {
            LifecycleState::Opening if exposure_after_pct > 0.0 => Ok(LifecycleState::Long),
            LifecycleState::Reducing if exposure_after_pct > 0.0 => Ok(LifecycleState::Long),
            LifecycleState::Reducing | LifecycleState::Closing
                if exposure_after_pct.abs() <= 0.0001 =>
            {
                Ok(LifecycleState::Closed)
            }
            _ => Err(format!(
                "liquidação de estado inválida: {} com exposição {:.4}",
                state.as_str(),
                exposure_after_pct
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeteriorationLevel {
    None,
    Low,
    Moderate,
    High,
    Critical,
}

impl DeteriorationLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Low => "LOW",
            Self::Moderate => "MODERATE",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionHealth {
    Strong,
    Healthy,
    Weakening,
    Deteriorating,
    Critical,
}

impl PositionHealth {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strong => "STRONG",
            Self::Healthy => "HEALTHY",
            Self::Weakening => "WEAKENING",
            Self::Deteriorating => "DETERIORATING",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDeteriorationConfig {
    pub trend_weight: f64,
    pub momentum_weight: f64,
    pub giveback_weight: f64,
    pub vwap_weight: f64,
    pub volatility_weight: f64,
    pub time_weight: f64,
    pub low_threshold: f64,
    pub moderate_threshold: f64,
    pub high_threshold: f64,
    pub critical_threshold: f64,
}

impl Default for PositionDeteriorationConfig {
    fn default() -> Self {
        Self {
            trend_weight: 0.22,
            momentum_weight: 0.18,
            giveback_weight: 0.25,
            vwap_weight: 0.15,
            volatility_weight: 0.12,
            time_weight: 0.08,
            low_threshold: 0.20,
            moderate_threshold: 0.40,
            high_threshold: 0.60,
            critical_threshold: 0.80,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeteriorationInput {
    pub position_age_candles: usize,
    pub return_since_entry_pct: f64,
    pub mfe_since_entry_pct: f64,
    pub mae_since_entry_pct: f64,
    pub giveback_from_mfe_pct: f64,
    pub distance_from_entry_atr: Option<f64>,
    pub vwap_change_atr: Option<f64>,
    pub ema_spread_change_atr: Option<f64>,
    pub rsi_change: Option<f64>,
    pub atr_change_ratio: Option<f64>,
    pub session_progress: Option<f64>,
    pub price_below_vwap: bool,
    pub trend_deteriorated: bool,
    pub momentum_deteriorated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeteriorationAssessment {
    pub trend_component: f64,
    pub momentum_component: f64,
    pub giveback_component: f64,
    pub vwap_component: f64,
    pub volatility_component: f64,
    pub time_component: f64,
    pub score: f64,
    pub level: DeteriorationLevel,
    pub health: PositionHealth,
}

pub struct PositionDeteriorationEngine {
    config: PositionDeteriorationConfig,
}

fn clamp(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn round(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

impl Default for PositionDeteriorationEngine {
    fn default() -> Self {
        Self {
            config: PositionDeteriorationConfig::default(),
        }
    }
}

impl PositionDeteriorationEngine {
    pub fn config(&self) -> &PositionDeteriorationConfig {
        &self.config
    }

    pub fn assess(&self, input: DeteriorationInput) -> DeteriorationAssessment {
        let trend_component = clamp(
            input
                .ema_spread_change_atr
                .map_or(0.0, |change| -change / 1.5)
                + if input.trend_deteriorated { 0.35 } else { 0.0 },
        );
        let momentum_component = clamp(
            input.rsi_change.map_or(0.0, |change| -change / 30.0)
                + if input.momentum_deteriorated {
                    0.35
                } else {
                    0.0
                },
        );
        let giveback_component = if input.mfe_since_entry_pct <= 0.0 {
            0.0
        } else {
            clamp(input.giveback_from_mfe_pct / input.mfe_since_entry_pct.max(0.25))
        };
        let vwap_component = clamp(
            input.vwap_change_atr.map_or(0.0, |change| -change / 1.5)
                + if input.price_below_vwap { 0.35 } else { 0.0 },
        );
        let volatility_component = clamp(
            input.atr_change_ratio.unwrap_or(0.0).max(0.0)
                + (-input.mae_since_entry_pct / 2.0).max(0.0)
                + input
                    .distance_from_entry_atr
                    .map_or(0.0, |distance| (-distance / 2.0).max(0.0)),
        );
        let age_component = input.position_age_candles.saturating_sub(16) as f64 / 32.0;
        let late_session_component = input
            .session_progress
            .map_or(0.0, |progress| ((progress - 0.75) / 0.25).max(0.0) * 0.25);
        let time_component = clamp(age_component.max(late_session_component));
        let score = round(
            trend_component * self.config.trend_weight
                + momentum_component * self.config.momentum_weight
                + giveback_component * self.config.giveback_weight
                + vwap_component * self.config.vwap_weight
                + volatility_component * self.config.volatility_weight
                + time_component * self.config.time_weight,
        );
        let level = if score >= self.config.critical_threshold {
            DeteriorationLevel::Critical
        } else if score >= self.config.high_threshold {
            DeteriorationLevel::High
        } else if score >= self.config.moderate_threshold {
            DeteriorationLevel::Moderate
        } else if score >= self.config.low_threshold {
            DeteriorationLevel::Low
        } else {
            DeteriorationLevel::None
        };
        let health = match level {
            DeteriorationLevel::None if input.return_since_entry_pct > 0.25 => {
                PositionHealth::Strong
            }
            DeteriorationLevel::None | DeteriorationLevel::Low => PositionHealth::Healthy,
            DeteriorationLevel::Moderate => PositionHealth::Weakening,
            DeteriorationLevel::High => PositionHealth::Deteriorating,
            DeteriorationLevel::Critical => PositionHealth::Critical,
        };
        DeteriorationAssessment {
            trend_component: round(trend_component),
            momentum_component: round(momentum_component),
            giveback_component: round(giveback_component),
            vwap_component: round(vwap_component),
            volatility_component: round(volatility_component),
            time_component: round(time_component),
            score,
            level,
            health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_state_machine_covers_open_increase_hold_reduce_and_exit() {
        let opening =
            PositionLifecycleEngine::request(LifecycleState::Flat, LifecycleEventType::Open)
                .unwrap();
        assert_eq!(opening, LifecycleState::Opening);
        let long = PositionLifecycleEngine::settle(opening, 25.0).unwrap();
        assert_eq!(
            PositionLifecycleEngine::request(long, LifecycleEventType::Increase).unwrap(),
            LifecycleState::Long
        );
        assert_eq!(
            PositionLifecycleEngine::request(long, LifecycleEventType::Hold).unwrap(),
            LifecycleState::Long
        );
        let reducing = PositionLifecycleEngine::request(long, LifecycleEventType::Reduce).unwrap();
        assert_eq!(
            PositionLifecycleEngine::settle(reducing, 10.0).unwrap(),
            LifecycleState::Long
        );
        let closing = PositionLifecycleEngine::request(long, LifecycleEventType::Exit).unwrap();
        assert_eq!(
            PositionLifecycleEngine::settle(closing, 0.0).unwrap(),
            LifecycleState::Closed
        );
    }

    #[test]
    fn lifecycle_rejects_impossible_transitions() {
        assert!(
            PositionLifecycleEngine::request(LifecycleState::Flat, LifecycleEventType::Reduce)
                .is_err()
        );
        assert!(
            PositionLifecycleEngine::request(LifecycleState::Closed, LifecycleEventType::Hold)
                .is_err()
        );
    }

    #[test]
    fn giveback_and_deterioration_are_deterministic_and_bounded() {
        let engine = PositionDeteriorationEngine::default();
        let assessment = engine.assess(DeteriorationInput {
            position_age_candles: 24,
            return_since_entry_pct: 0.5,
            mfe_since_entry_pct: 2.0,
            mae_since_entry_pct: -0.8,
            giveback_from_mfe_pct: 1.5,
            distance_from_entry_atr: Some(0.4),
            vwap_change_atr: Some(-0.8),
            ema_spread_change_atr: Some(-1.0),
            rsi_change: Some(-20.0),
            atr_change_ratio: Some(0.4),
            session_progress: Some(0.5),
            price_below_vwap: true,
            trend_deteriorated: true,
            momentum_deteriorated: true,
        });
        assert!((0.0..=1.0).contains(&assessment.score));
        assert!(matches!(
            assessment.level,
            DeteriorationLevel::High | DeteriorationLevel::Critical
        ));
        assert!(matches!(
            assessment.health,
            PositionHealth::Deteriorating | PositionHealth::Critical
        ));
        assert_eq!(
            assessment.score,
            engine
                .assess(DeteriorationInput {
                    position_age_candles: 24,
                    return_since_entry_pct: 0.5,
                    mfe_since_entry_pct: 2.0,
                    mae_since_entry_pct: -0.8,
                    giveback_from_mfe_pct: 1.5,
                    distance_from_entry_atr: Some(0.4),
                    vwap_change_atr: Some(-0.8),
                    ema_spread_change_atr: Some(-1.0),
                    rsi_change: Some(-20.0),
                    atr_change_ratio: Some(0.4),
                    session_progress: Some(0.5),
                    price_below_vwap: true,
                    trend_deteriorated: true,
                    momentum_deteriorated: true,
                })
                .score
        );
    }

    #[test]
    fn healthy_trace_does_not_require_future_outcomes() {
        let assessment = PositionDeteriorationEngine::default().assess(DeteriorationInput {
            position_age_candles: 3,
            return_since_entry_pct: 0.7,
            mfe_since_entry_pct: 0.8,
            giveback_from_mfe_pct: 0.1,
            ..DeteriorationInput::default()
        });
        assert_eq!(assessment.level, DeteriorationLevel::None);
        assert_eq!(assessment.health, PositionHealth::Strong);
    }
}
