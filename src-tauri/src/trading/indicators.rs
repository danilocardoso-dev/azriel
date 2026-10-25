//! Indicadores técnicos puros sobre candles em ordem cronológica (mais antigo primeiro).

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

/// Média móvel exponencial; semente = média simples dos primeiros `period` valores.
pub fn ema(values: &[f64], period: usize) -> Option<f64> {
    if period == 0 || values.len() < period {
        return None;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let mut current = values[..period].iter().sum::<f64>() / period as f64;
    for value in &values[period..] {
        current = value * k + current * (1.0 - k);
    }
    Some(current)
}

/// RSI de Wilder.
pub fn rsi(closes: &[f64], period: usize) -> Option<f64> {
    if period == 0 || closes.len() <= period {
        return None;
    }
    let changes: Vec<f64> = closes.windows(2).map(|pair| pair[1] - pair[0]).collect();
    let mut gain = changes[..period].iter().filter(|c| **c > 0.0).sum::<f64>() / period as f64;
    let mut loss = changes[..period].iter().filter(|c| **c < 0.0).map(|c| -c).sum::<f64>() / period as f64;
    for change in &changes[period..] {
        gain = (gain * (period as f64 - 1.0) + change.max(0.0)) / period as f64;
        loss = (loss * (period as f64 - 1.0) + (-change).max(0.0)) / period as f64;
    }
    if loss == 0.0 {
        return Some(if gain == 0.0 { 50.0 } else { 100.0 });
    }
    Some(100.0 - 100.0 / (1.0 + gain / loss))
}

/// ATR de Wilder.
pub fn atr(candles: &[Candle], period: usize) -> Option<f64> {
    if period == 0 || candles.len() <= period {
        return None;
    }
    let ranges: Vec<f64> = candles
        .windows(2)
        .map(|pair| {
            let (previous, current) = (pair[0], pair[1]);
            (current.high - current.low)
                .max((current.high - previous.close).abs())
                .max((current.low - previous.close).abs())
        })
        .collect();
    let mut value = ranges[..period].iter().sum::<f64>() / period as f64;
    for range in &ranges[period..] {
        value = (value * (period as f64 - 1.0) + range) / period as f64;
    }
    Some(value)
}

/// Volume do último candle dividido pela média dos `period` anteriores.
pub fn relative_volume(candles: &[Candle], period: usize) -> Option<f64> {
    if candles.len() <= period || period == 0 {
        return None;
    }
    let last = candles.last()?.volume;
    let previous = &candles[candles.len() - 1 - period..candles.len() - 1];
    let average = previous.iter().map(|c| c.volume).sum::<f64>() / period as f64;
    (average > 0.0).then(|| last / average)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(close: f64) -> Candle {
        Candle { open_time: 0, open: close, high: close + 1.0, low: close - 1.0, close, volume: 10.0 }
    }

    #[test]
    fn ema_follows_the_trend_and_needs_enough_data() {
        let rising: Vec<f64> = (1..=30).map(f64::from).collect();
        let fast = ema(&rising, 9).unwrap();
        let slow = ema(&rising, 21).unwrap();
        assert!(fast > slow && fast < 30.0);
        assert!(ema(&rising[..5], 9).is_none());
        assert_eq!(ema(&[2.0, 2.0, 2.0], 3), Some(2.0));
    }

    #[test]
    fn rsi_is_bounded_and_extreme_on_one_way_moves() {
        let rising: Vec<f64> = (1..=30).map(f64::from).collect();
        let falling: Vec<f64> = rising.iter().rev().copied().collect();
        assert_eq!(rsi(&rising, 14), Some(100.0));
        assert_eq!(rsi(&falling, 14), Some(0.0));
        let mixed: Vec<f64> = (0..40).map(|i| if i % 2 == 0 { 100.0 } else { 101.0 }).collect();
        let value = rsi(&mixed, 14).unwrap();
        assert!((40.0..60.0).contains(&value));
    }

    #[test]
    fn atr_and_relative_volume_measure_range_and_activity() {
        let candles: Vec<Candle> = (0..20).map(|i| candle(100.0 + f64::from(i % 2))).collect();
        let value = atr(&candles, 14).unwrap();
        assert!(value >= 2.0 && value <= 3.0);
        let mut busy = candles.clone();
        busy.last_mut().unwrap().volume = 30.0;
        assert_eq!(relative_volume(&busy, 10), Some(3.0));
    }
}
