use super::{market_ai, market_intraday::IntradayFeatures};
use serde::{Deserialize, Serialize};

pub const CONTEXT_VERSION: &str = market_ai::CONTEXT_INTRADAY_V1_VERSION;

pub const REASON_CODES: &[&str] = &[
    "INTRADAY_TREND_ALIGNMENT",
    "INTRADAY_MOMENTUM_ALIGNMENT",
    "INTRADAY_BREAKOUT",
    "VWAP_SUPPORT",
    "VWAP_REVERSION_RISK",
    "EXTENDED_FROM_VWAP",
    "LOW_RELATIVE_VOLUME",
    "HIGH_VOLATILITY",
    "SIGNAL_CONFLICT",
    "MOMENTUM_DETERIORATION",
    "TREND_DETERIORATION",
    "SESSION_CONTEXT",
    "POSITION_ALREADY_OPTIMAL",
    "INSUFFICIENT_INTRADAY_SIGNAL",
    "RISK_REDUCTION",
    "EXIT_SIGNAL",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntradayTrendState {
    Bullish,
    Bearish,
    Neutral,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntradayLevelState {
    Above,
    Below,
    Near,
    Unknown,
}
impl IntradayLevelState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Above => "ABOVE",
            Self::Below => "BELOW",
            Self::Near => "NEAR",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayMarketState {
    pub trend: IntradayTrendState,
    pub vwap_position: IntradayLevelState,
    pub momentum: IntradayTrendState,
    pub volume: String,
    pub volatility: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayBreakoutFlags {
    pub near_rolling_high: bool,
    pub near_rolling_low: bool,
    pub breakout_above_recent_high: bool,
    pub breakdown_below_recent_low: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayAiMarket {
    pub asset: String,
    pub timeframe: String,
    pub price: f64,
    pub timestamp: String,
    pub session_id: String,
    pub session_phase: String,
    pub session_progress: f64,
    pub candles_since_open: usize,
    pub candles_until_close: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayAiIndicators {
    pub ema_9: Option<f64>,
    pub ema_21: Option<f64>,
    pub ema_spread_atr: Option<f64>,
    pub vwap: Option<f64>,
    pub distance_from_vwap_atr: Option<f64>,
    pub rsi_14: Option<f64>,
    pub atr_14: Option<f64>,
    pub relative_volume: Option<f64>,
    pub rolling_high: Option<f64>,
    pub rolling_low: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayAiSignals {
    pub trend: String,
    pub momentum: String,
    pub bias: String,
    pub strength: String,
    pub conflict: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayAiPosition {
    pub state: String,
    pub current_exposure_pct: f64,
    pub average_entry_price: Option<f64>,
    pub unrealized_pnl_pct: Option<f64>,
    pub holding_candles: usize,
    pub holding_market_minutes: usize,
    pub overnight: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketAiIntradaySnapshotV1 {
    pub context_version: &'static str,
    pub market: IntradayAiMarket,
    pub indicators: IntradayAiIndicators,
    pub signals: IntradayAiSignals,
    pub market_state: IntradayMarketState,
    pub breakout_flags: IntradayBreakoutFlags,
    pub position: IntradayAiPosition,
    pub risk_context: market_ai::MarketAiRiskContext,
}

fn trend(features: &IntradayFeatures) -> IntradayTrendState {
    match (
        features.ema_9,
        features.ema_21,
        features.ema_9_slope,
        features.ema_21_slope,
    ) {
        (Some(fast), Some(slow), Some(fast_slope), Some(slow_slope))
            if fast > slow && fast_slope > 0.0 && slow_slope >= 0.0 =>
        {
            IntradayTrendState::Bullish
        }
        (Some(fast), Some(slow), Some(fast_slope), Some(slow_slope))
            if fast < slow && fast_slope < 0.0 && slow_slope <= 0.0 =>
        {
            IntradayTrendState::Bearish
        }
        _ => IntradayTrendState::Neutral,
    }
}

pub fn derive_state(
    features: &IntradayFeatures,
) -> (
    IntradayMarketState,
    IntradayBreakoutFlags,
    IntradayAiSignals,
) {
    let atr = features.atr_14.filter(|value| *value > 0.0);
    let near_high = features
        .rolling_high
        .zip(atr)
        .is_some_and(|(high, atr)| (features.close - high).abs() <= atr * 0.25);
    let near_low = features
        .rolling_low
        .zip(atr)
        .is_some_and(|(low, atr)| (features.close - low).abs() <= atr * 0.25);
    let breakout = features
        .rolling_high
        .is_some_and(|high| features.close > high);
    let breakdown = features.rolling_low.is_some_and(|low| features.close < low);
    let trend_state = trend(features);
    let momentum = match features.short_term_return {
        Some(value) if value > 0.002 => IntradayTrendState::Bullish,
        Some(value) if value < -0.002 => IntradayTrendState::Bearish,
        _ => IntradayTrendState::Neutral,
    };
    let vwap_position = match features.distance_from_vwap_atr {
        Some(value) if value > 0.15 => IntradayLevelState::Above,
        Some(value) if value < -0.15 => IntradayLevelState::Below,
        Some(_) => IntradayLevelState::Near,
        None => IntradayLevelState::Unknown,
    };
    let volume = match features.relative_volume {
        Some(value) if value >= 1.5 => "HIGH",
        Some(value) if value < 0.8 => "LOW",
        Some(_) => "NORMAL",
        None => "UNKNOWN",
    };
    let volatility = match features.price_range_atr {
        Some(value) if value >= 1.5 => "HIGH",
        Some(value) if value < 0.6 => "LOW",
        Some(_) => "NORMAL",
        None => "UNKNOWN",
    };
    let location = if breakout {
        "ABOVE_RECENT_HIGH"
    } else if breakdown {
        "BELOW_RECENT_LOW"
    } else if near_high {
        "NEAR_RECENT_HIGH"
    } else if near_low {
        "NEAR_RECENT_LOW"
    } else {
        "MID_RANGE"
    };
    let bias = match trend_state {
        IntradayTrendState::Bullish => "BULLISH",
        IntradayTrendState::Bearish => "BEARISH",
        IntradayTrendState::Neutral => "NEUTRAL",
    };
    let momentum_name = match momentum {
        IntradayTrendState::Bullish => "BULLISH",
        IntradayTrendState::Bearish => "BEARISH",
        IntradayTrendState::Neutral => "NEUTRAL",
    };
    let conflict = (bias == "BULLISH" && momentum_name == "BEARISH")
        || (bias == "BEARISH" && momentum_name == "BULLISH");
    let strength = if breakout
        || breakdown
        || features
            .ema_spread_atr
            .is_some_and(|value| value.abs() >= 0.8)
    {
        "HIGH"
    } else if bias != "NEUTRAL" {
        "MEDIUM"
    } else {
        "LOW"
    };
    (
        IntradayMarketState {
            trend: trend_state,
            vwap_position,
            momentum,
            volume: volume.into(),
            volatility: volatility.into(),
            location: location.into(),
        },
        IntradayBreakoutFlags {
            near_rolling_high: near_high,
            near_rolling_low: near_low,
            breakout_above_recent_high: breakout,
            breakdown_below_recent_low: breakdown,
        },
        IntradayAiSignals {
            trend: bias.into(),
            momentum: momentum_name.into(),
            bias: bias.into(),
            strength: strength.into(),
            conflict,
        },
    )
}

#[allow(clippy::too_many_arguments)]
pub fn build_snapshot(
    asset: &str,
    timeframe: &str,
    features: &IntradayFeatures,
    position: &market_ai::MarketAiPositionContextV42,
    average_entry_price: Option<f64>,
    overnight: bool,
    risk_context: market_ai::MarketAiRiskContext,
) -> MarketAiIntradaySnapshotV1 {
    let (market_state, breakout_flags, signals) = derive_state(features);
    let candles_since_open = (features.session_progress * 25.0).round() as usize;
    MarketAiIntradaySnapshotV1 {
        context_version: CONTEXT_VERSION,
        market: IntradayAiMarket {
            asset: asset.into(),
            timeframe: timeframe.into(),
            price: features.close,
            timestamp: features.timestamp_utc.clone(),
            session_id: features.session_id.clone(),
            session_phase: features.session_phase.as_str().into(),
            session_progress: features.session_progress,
            candles_since_open,
            candles_until_close: 25usize.saturating_sub(candles_since_open),
        },
        indicators: IntradayAiIndicators {
            ema_9: features.ema_9,
            ema_21: features.ema_21,
            ema_spread_atr: features.ema_spread_atr,
            vwap: features.vwap,
            distance_from_vwap_atr: features.distance_from_vwap_atr,
            rsi_14: features.rsi_14,
            atr_14: features.atr_14,
            relative_volume: features.relative_volume,
            rolling_high: features.rolling_high,
            rolling_low: features.rolling_low,
        },
        signals,
        market_state,
        breakout_flags,
        position: IntradayAiPosition {
            state: position.state.clone(),
            current_exposure_pct: position.current_exposure_pct,
            average_entry_price,
            unrealized_pnl_pct: position.unrealized_pnl_pct,
            holding_candles: position.holding_candles,
            holding_market_minutes: position.holding_candles * 15,
            overnight,
        },
        risk_context,
    }
}

pub fn validate_dataset(timeframe: &str, has_complete_volume: bool) -> Result<(), String> {
    if timeframe != "15M" {
        return Err("AI Intraday V1 aceita somente datasets 15M".into());
    }
    if !has_complete_volume {
        return Err("AI Intraday V1 exige volume completo em todos os candles".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::market_intraday::SessionPhase;

    fn features() -> IntradayFeatures {
        IntradayFeatures {
            candle_index: 30,
            timestamp_utc: "2026-03-12T15:00:00Z".into(),
            session_id: "2026-03-12".into(),
            close: 105.0,
            ema_9: Some(104.0),
            ema_21: Some(102.0),
            ema_9_slope: Some(0.2),
            ema_21_slope: Some(0.1),
            vwap: Some(102.0),
            rsi_14: Some(65.0),
            atr_14: Some(1.0),
            relative_volume: Some(1.6),
            rolling_high: Some(104.0),
            rolling_low: Some(99.0),
            short_term_return: Some(0.01),
            distance_from_vwap: Some(3.0),
            distance_from_vwap_atr: Some(3.0),
            ema_spread_atr: Some(2.0),
            price_range_atr: Some(1.6),
            session_progress: 0.4,
            session_phase: SessionPhase::Morning,
            ready: true,
        }
    }

    #[test]
    fn derives_intraday_state_without_a_decision() {
        let (state, flags, signals) = derive_state(&features());
        assert!(matches!(state.trend, IntradayTrendState::Bullish));
        assert!(flags.breakout_above_recent_high);
        assert_eq!(signals.bias, "BULLISH");
        assert!(!signals.conflict);
    }

    #[test]
    fn rejects_daily_and_missing_volume() {
        assert!(validate_dataset("1D", true).is_err());
        assert!(validate_dataset("15M", false).is_err());
        assert!(validate_dataset("15M", true).is_ok());
    }
}
