const EPSILON: f64 = 1e-12;

pub const METRIC_FORMULA_VERSION: &str = "MARKET_METRICS_V1";

pub fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

pub fn population_std(values: &[f64]) -> Option<f64> {
    let average = mean(values)?;
    Some(
        (values
            .iter()
            .map(|value| (value - average).powi(2))
            .sum::<f64>()
            / values.len() as f64)
            .sqrt(),
    )
}

pub fn sample_std(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let average = mean(values)?;
    Some(
        (values
            .iter()
            .map(|value| (value - average).powi(2))
            .sum::<f64>()
            / (values.len() - 1) as f64)
            .sqrt(),
    )
}

pub fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    })
}

pub fn periodic_returns(equity: &[f64]) -> Vec<f64> {
    equity
        .windows(2)
        .filter_map(|pair| (pair[0].abs() > EPSILON).then(|| pair[1] / pair[0] - 1.0))
        .collect()
}

pub fn total_return_pct(initial: f64, final_value: f64) -> f64 {
    if initial.abs() <= EPSILON {
        0.0
    } else {
        (final_value / initial - 1.0) * 100.0
    }
}

pub fn max_drawdown_pct(equity: &[f64]) -> f64 {
    let mut peak = equity.first().copied().unwrap_or(0.0);
    equity.iter().fold(0.0_f64, |worst, value| {
        peak = peak.max(*value);
        if peak <= EPSILON {
            worst
        } else {
            worst.max((peak - value) / peak * 100.0)
        }
    })
}

pub fn sharpe(
    returns: &[f64],
    risk_free_rate_per_period: f64,
    annualization_factor: f64,
) -> Option<f64> {
    if annualization_factor <= 0.0 {
        return None;
    }
    let excess: Vec<f64> = returns
        .iter()
        .map(|value| value - risk_free_rate_per_period)
        .collect();
    let deviation = sample_std(&excess)?;
    if deviation <= EPSILON {
        None
    } else {
        Some(mean(&excess)? / deviation * annualization_factor.sqrt())
    }
}

pub fn sortino(
    returns: &[f64],
    risk_free_rate_per_period: f64,
    annualization_factor: f64,
) -> Option<f64> {
    if returns.is_empty() || annualization_factor <= 0.0 {
        return None;
    }
    let excess: Vec<f64> = returns
        .iter()
        .map(|value| value - risk_free_rate_per_period)
        .collect();
    let downside = (excess
        .iter()
        .map(|value| value.min(0.0).powi(2))
        .sum::<f64>()
        / excess.len() as f64)
        .sqrt();
    if downside <= EPSILON {
        None
    } else {
        Some(mean(&excess)? / downside * annualization_factor.sqrt())
    }
}

pub fn annualized_return_pct(
    initial: f64,
    final_value: f64,
    periods: usize,
    annualization_factor: f64,
) -> Option<f64> {
    if initial <= 0.0 || final_value < 0.0 || periods == 0 || annualization_factor <= 0.0 {
        return None;
    }
    Some(((final_value / initial).powf(annualization_factor / periods as f64) - 1.0) * 100.0)
}

pub fn calmar(
    initial: f64,
    final_value: f64,
    periods: usize,
    annualization_factor: f64,
    max_drawdown_pct: f64,
) -> Option<f64> {
    if max_drawdown_pct <= EPSILON {
        return None;
    }
    Some(
        annualized_return_pct(initial, final_value, periods, annualization_factor)?
            / max_drawdown_pct,
    )
}

pub fn rolling_return_pct(equity: &[f64], end: usize, window: usize) -> Option<f64> {
    if window == 0 || end + 1 < window {
        return None;
    }
    let start = end + 1 - window;
    Some(total_return_pct(equity[start], equity[end]))
}

pub fn rolling_drawdown_pct(equity: &[f64], end: usize, window: usize) -> Option<f64> {
    if window == 0 || end + 1 < window {
        return None;
    }
    Some(max_drawdown_pct(&equity[end + 1 - window..=end]))
}

pub fn rolling_volatility_pct(
    equity: &[f64],
    end: usize,
    window: usize,
    annualization_factor: f64,
) -> Option<f64> {
    if window < 2 || end + 1 < window {
        return None;
    }
    let returns = periodic_returns(&equity[end + 1 - window..=end]);
    Some(sample_std(&returns)? * annualization_factor.sqrt() * 100.0)
}

pub fn rolling_sharpe(
    equity: &[f64],
    end: usize,
    window: usize,
    annualization_factor: f64,
) -> Option<f64> {
    if window < 2 || end + 1 < window {
        return None;
    }
    sharpe(
        &periodic_returns(&equity[end + 1 - window..=end]),
        0.0,
        annualization_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_risk_metrics_are_finite_and_zero_drawdown_is_invalid() {
        let returns = [0.01, -0.005, 0.02, -0.01, 0.015];
        assert!(sharpe(&returns, 0.0, 252.0).unwrap().is_finite());
        assert!(sortino(&returns, 0.0, 252.0).unwrap().is_finite());
        assert!(calmar(100.0, 110.0, 5, 252.0, 4.0).unwrap().is_finite());
        assert_eq!(calmar(100.0, 110.0, 5, 252.0, 0.0), None);
    }

    #[test]
    fn rolling_metrics_wait_for_the_full_window() {
        let equity = [100.0, 101.0, 99.0, 103.0, 104.0];
        assert_eq!(rolling_return_pct(&equity, 2, 4), None);
        assert!(rolling_return_pct(&equity, 3, 4).is_some());
        assert!(rolling_volatility_pct(&equity, 4, 4, 252.0).is_some());
        assert!(rolling_drawdown_pct(&equity, 4, 4).is_some());
    }

    #[test]
    fn dispersion_distinguishes_unstable_windows() {
        let stable = population_std(&[5.0, 4.0, 6.0, 5.0]).unwrap();
        let unstable = population_std(&[30.0, -25.0, 40.0, -20.0]).unwrap();
        assert!(unstable > stable);
        assert_eq!(median(&[1.0, 4.0, 3.0, 2.0]), Some(2.5));
    }

    #[test]
    fn drawdown_and_total_return_match_known_series() {
        assert_eq!(max_drawdown_pct(&[100.0, 120.0, 90.0, 110.0]), 25.0);
        assert!((total_return_pct(100.0, 110.0) - 10.0).abs() < 1e-9);
        assert!((rolling_return_pct(&[100.0, 105.0, 110.0], 2, 3).unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn invalid_metrics_never_produce_nan_or_infinity() {
        assert_eq!(sharpe(&[0.01, 0.01, 0.01], 0.0, 252.0), None);
        assert_eq!(sortino(&[0.01, 0.02], 0.0, 252.0), None);
        assert_eq!(annualized_return_pct(0.0, 10.0, 3, 252.0), None);
        assert_eq!(rolling_volatility_pct(&[100.0], 0, 20, 252.0), None);
    }

    #[test]
    fn annualization_factor_changes_risk_adjusted_metrics() {
        let returns = [0.01, -0.005, 0.02, -0.01];
        let daily = sharpe(&returns, 0.0, 252.0).unwrap();
        let hourly = sharpe(&returns, 0.0, 1638.0).unwrap();
        assert!(hourly > daily);
    }
}
