use serde::{Deserialize, Serialize};

pub const SIGNAL_ENGINE_VERSION: &str = "SIGNAL_ENGINE_V1";
pub const SIGNAL_CONFIG_VERSION: &str = "SIGNAL_CONFIG_V1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SignalEngineConfig {
    pub version: String,
    pub trend_weight: f64,
    pub momentum_weight: f64,
    pub volatility_weight: f64,
    pub strong_signal_threshold: f64,
    pub weak_signal_threshold: f64,
    pub conflict_threshold: f64,
    pub trend_normalization_pct: f64,
    pub momentum_normalization_pct: f64,
    pub high_volatility: f64,
}

impl Default for SignalEngineConfig {
    fn default() -> Self {
        Self {
            version: SIGNAL_CONFIG_VERSION.into(),
            trend_weight: 0.45,
            momentum_weight: 0.45,
            volatility_weight: 0.10,
            strong_signal_threshold: 0.65,
            weak_signal_threshold: 0.25,
            conflict_threshold: 0.50,
            trend_normalization_pct: 3.0,
            momentum_normalization_pct: 5.0,
            high_volatility: 0.03,
        }
    }
}

impl SignalEngineConfig {
    pub fn validate(&self) -> Result<(), String> {
        let weight_sum = self.trend_weight + self.momentum_weight + self.volatility_weight;
        if (weight_sum - 1.0).abs() > 0.000_001
            || self.trend_weight <= 0.0
            || self.momentum_weight <= 0.0
            || self.volatility_weight < 0.0
            || !(0.0..=1.0).contains(&self.weak_signal_threshold)
            || !(self.weak_signal_threshold..=1.0).contains(&self.strong_signal_threshold)
            || !(0.0..=1.0).contains(&self.conflict_threshold)
            || self.trend_normalization_pct <= 0.0
            || self.momentum_normalization_pct <= 0.0
            || self.high_volatility <= 0.0
            || self.version != SIGNAL_CONFIG_VERSION
        {
            return Err("configuração do Signal Engine inválida".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarketSignalInput {
    pub price_vs_sma_short_pct: Option<f64>,
    pub price_vs_sma_long_pct: Option<f64>,
    pub sma_spread_pct: Option<f64>,
    pub sma_short_slope_pct: Option<f64>,
    pub sma_long_slope_pct: Option<f64>,
    pub return_1_pct: Option<f64>,
    pub return_5_pct: Option<f64>,
    pub return_10_pct: Option<f64>,
    pub return_20_pct: Option<f64>,
    pub rolling_volatility: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectionalBias { Bullish, Bearish, Neutral }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictLevel { Low, Medium, High }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSignalSummary {
    pub engine_version: String,
    pub config_version: String,
    pub trend_score: f64,
    pub momentum_score: f64,
    pub volatility_score: f64,
    pub mean_reversion_score: Option<f64>,
    pub bullish_evidence: f64,
    pub bearish_evidence: f64,
    pub signal_strength: f64,
    pub directional_bias: DirectionalBias,
    pub conflict_level: ConflictLevel,
}

fn normalized(value: Option<f64>, scale: f64) -> Option<f64> {
    value.filter(|v| v.is_finite()).map(|v| (v / scale).clamp(-1.0, 1.0))
}

fn average(values: impl Iterator<Item = Option<f64>>) -> f64 {
    let values: Vec<_> = values.flatten().collect();
    if values.is_empty() { 0.0 } else { values.iter().sum::<f64>() / values.len() as f64 }
}

pub fn evaluate(input: &MarketSignalInput, config: &SignalEngineConfig) -> Result<MarketSignalSummary, String> {
    config.validate()?;
    let trend = average([
        normalized(input.price_vs_sma_short_pct, config.trend_normalization_pct),
        normalized(input.price_vs_sma_long_pct, config.trend_normalization_pct * 1.5),
        normalized(input.sma_spread_pct, config.trend_normalization_pct),
        normalized(input.sma_short_slope_pct, config.trend_normalization_pct),
        normalized(input.sma_long_slope_pct, config.trend_normalization_pct),
    ].into_iter());
    let momentum = average([
        normalized(input.return_1_pct, config.momentum_normalization_pct * 0.4),
        normalized(input.return_5_pct, config.momentum_normalization_pct),
        normalized(input.return_10_pct, config.momentum_normalization_pct * 1.6),
        normalized(input.return_20_pct, config.momentum_normalization_pct * 2.4),
    ].into_iter());
    let volatility = input.rolling_volatility.filter(|v| v.is_finite()).map(|v| (v / config.high_volatility).clamp(0.0, 1.0)).unwrap_or(0.0);
    let directional_weight = config.trend_weight + config.momentum_weight;
    let bullish = (config.trend_weight * trend.max(0.0) + config.momentum_weight * momentum.max(0.0)) / directional_weight;
    let bearish = (config.trend_weight * (-trend).max(0.0) + config.momentum_weight * (-momentum).max(0.0)) / directional_weight;
    let conflict = if trend.signum() != momentum.signum() { trend.abs().min(momentum.abs()) } else { 0.0 };
    let conflict_level = if conflict >= config.conflict_threshold { ConflictLevel::High } else if conflict >= config.weak_signal_threshold { ConflictLevel::Medium } else { ConflictLevel::Low };
    let strength = ((bullish - bearish).abs() * (1.0 - config.volatility_weight * volatility)).clamp(0.0, 1.0);
    let bias = if strength < config.weak_signal_threshold { DirectionalBias::Neutral } else if bullish > bearish { DirectionalBias::Bullish } else { DirectionalBias::Bearish };
    Ok(MarketSignalSummary {
        engine_version: SIGNAL_ENGINE_VERSION.into(), config_version: config.version.clone(),
        trend_score: trend, momentum_score: momentum, volatility_score: volatility,
        mean_reversion_score: None, bullish_evidence: bullish, bearish_evidence: bearish,
        signal_strength: strength, directional_bias: bias, conflict_level,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn aligned(sign: f64) -> MarketSignalInput { MarketSignalInput { price_vs_sma_short_pct: Some(3.0*sign), price_vs_sma_long_pct: Some(5.0*sign), sma_spread_pct: Some(3.0*sign), sma_short_slope_pct: Some(2.0*sign), sma_long_slope_pct: Some(1.5*sign), return_1_pct: Some(2.0*sign), return_5_pct: Some(5.0*sign), return_10_pct: Some(8.0*sign), return_20_pct: Some(12.0*sign), rolling_volatility: Some(0.01), ..Default::default() } }
    #[test] fn aligned_trend_and_momentum_are_directional_and_normalized() { for (sign, expected) in [(1.0, DirectionalBias::Bullish),(-1.0,DirectionalBias::Bearish)] { let s=evaluate(&aligned(sign),&Default::default()).unwrap(); assert_eq!(s.directional_bias,expected); assert!(s.signal_strength>=0.65); assert!((-1.0..=1.0).contains(&s.trend_score)); assert!((-1.0..=1.0).contains(&s.momentum_score)); }}
    #[test] fn opposing_components_create_high_conflict() { let mut input=aligned(1.0); input.return_1_pct=Some(-2.0);input.return_5_pct=Some(-5.0);input.return_10_pct=Some(-8.0);input.return_20_pct=Some(-12.0);let s=evaluate(&input,&Default::default()).unwrap();assert_eq!(s.conflict_level,ConflictLevel::High);assert_eq!(s.directional_bias,DirectionalBias::Neutral); }
    #[test] fn volatility_is_context_not_direction() { let mut low=aligned(1.0);low.rolling_volatility=Some(0.0);let mut high=low.clone();high.rolling_volatility=Some(0.03);let a=evaluate(&low,&Default::default()).unwrap();let b=evaluate(&high,&Default::default()).unwrap();assert_eq!(a.directional_bias,b.directional_bias);assert!(b.signal_strength<a.signal_strength); }
    #[test] fn configuration_is_versioned_and_validated() { let mut config=SignalEngineConfig::default();assert_eq!(config.version,SIGNAL_CONFIG_VERSION);config.trend_weight=0.8;assert!(config.validate().is_err()); }
}
