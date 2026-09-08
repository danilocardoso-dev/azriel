use super::{market_models::MarketRegimeConfig, market_statistics};

pub const REGIME_ENGINE_VERSION: &str = "REGIME_ENGINE_V1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegimeLabel {
    pub trend: &'static str,
    pub volatility: &'static str,
}

pub fn validate_config(config: &MarketRegimeConfig) -> Result<(), String> {
    if config.trend_window < 2 || config.volatility_window < 3 {
        return Err("janelas de regime devem permitir histórico suficiente".into());
    }
    if config.bear_threshold_pct >= config.bull_threshold_pct {
        return Err("bearThreshold deve ser menor que bullThreshold".into());
    }
    if config.low_volatility_threshold_pct >= config.high_volatility_threshold_pct {
        return Err("lowVolThreshold deve ser menor que highVolThreshold".into());
    }
    if config.minimum_sample_candles == 0 || config.minimum_sample_trades == 0 {
        return Err("amostra mínima deve ser positiva".into());
    }
    Ok(())
}

pub fn classify(closes: &[f64], candle_index: usize, config: &MarketRegimeConfig) -> RegimeLabel {
    let trend = if candle_index + 1 < config.trend_window {
        "sideways"
    } else {
        let start = candle_index + 1 - config.trend_window;
        let change = market_statistics::total_return_pct(closes[start], closes[candle_index]);
        if change >= config.bull_threshold_pct {
            "bull"
        } else if change <= config.bear_threshold_pct {
            "bear"
        } else {
            "sideways"
        }
    };

    let volatility = if candle_index + 1 < config.volatility_window {
        "normal_volatility"
    } else {
        let start = candle_index + 1 - config.volatility_window;
        let returns: Vec<f64> = closes[start..=candle_index]
            .windows(2)
            .map(|pair| pair[1] / pair[0] - 1.0)
            .collect();
        let volatility_pct = market_statistics::population_std(&returns).unwrap_or(0.0) * 100.0;
        if volatility_pct >= config.high_volatility_threshold_pct {
            "high_volatility"
        } else if volatility_pct <= config.low_volatility_threshold_pct {
            "low_volatility"
        } else {
            "normal_volatility"
        }
    };
    RegimeLabel { trend, volatility }
}

pub fn classify_all(closes: &[f64], config: &MarketRegimeConfig) -> Vec<RegimeLabel> {
    (0..closes.len())
        .map(|index| classify(closes, index, config))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> MarketRegimeConfig {
        MarketRegimeConfig {
            trend_window: 4,
            volatility_window: 4,
            bull_threshold_pct: 2.0,
            bear_threshold_pct: -2.0,
            high_volatility_threshold_pct: 2.0,
            low_volatility_threshold_pct: 0.5,
            minimum_sample_candles: 3,
            minimum_sample_trades: 1,
        }
    }

    #[test]
    fn direction_regimes_are_deterministic() {
        assert_eq!(
            classify(&[100.0, 102.0, 104.0, 106.0], 3, &config()).trend,
            "bull"
        );
        assert_eq!(
            classify(&[106.0, 104.0, 102.0, 100.0], 3, &config()).trend,
            "bear"
        );
        assert_eq!(
            classify(&[100.0, 100.2, 99.9, 100.1], 3, &config()).trend,
            "sideways"
        );
    }

    #[test]
    fn volatility_regimes_can_coexist_with_direction() {
        let high = classify(&[100.0, 110.0, 95.0, 108.0], 3, &config());
        assert_eq!(high.volatility, "high_volatility");
        let low = classify(&[100.0, 100.1, 100.2, 100.3], 3, &config());
        assert_eq!(low.volatility, "low_volatility");
        assert_eq!(low.trend, "sideways");
    }

    #[test]
    fn classification_never_reads_future_candles() {
        let first = classify(&[100.0, 101.0, 102.0, 103.0], 2, &config());
        let changed_future = classify(&[100.0, 101.0, 102.0, 500.0], 2, &config());
        assert_eq!(first, changed_future);
    }

    #[test]
    fn invalid_threshold_order_is_rejected() {
        let mut invalid = config();
        invalid.bear_threshold_pct = invalid.bull_threshold_pct;
        assert!(validate_config(&invalid).is_err());
        invalid = config();
        invalid.low_volatility_threshold_pct = invalid.high_volatility_threshold_pct;
        assert!(validate_config(&invalid).is_err());
    }
}
