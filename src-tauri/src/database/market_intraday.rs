use super::{market_positions::{PositionIntent, PositionState}, market_repository::Candle};
use chrono::{DateTime, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::VecDeque, str::FromStr};

pub const FEATURE_ENGINE_VERSION: &str = "INTRADAY_FEATURE_ENGINE_V1";
pub const TREND_AGENT_ID: &str = "intraday-trend-v1";
pub const MOMENTUM_AGENT_ID: &str = "intraday-momentum-v1";
pub const MEAN_REVERSION_AGENT_ID: &str = "intraday-mean-reversion-v1";
pub const TREND_CONFIG_VERSION: &str = "INTRADAY_TREND_CONFIG_V1";
pub const MOMENTUM_CONFIG_VERSION: &str = "INTRADAY_MOMENTUM_CONFIG_V1";
pub const MEAN_REVERSION_CONFIG_VERSION: &str = "INTRADAY_MEAN_REVERSION_CONFIG_V1";

pub fn is_intraday_agent(id: &str) -> bool {
    matches!(id, TREND_AGENT_ID | MOMENTUM_AGENT_ID | MEAN_REVERSION_AGENT_ID)
}

pub fn requires_volume(id: &str) -> bool {
    matches!(id, MOMENTUM_AGENT_ID | MEAN_REVERSION_AGENT_ID)
}

pub fn style_for_agent(id: &str) -> Option<&'static str> {
    match id {
        TREND_AGENT_ID => Some("TREND"),
        MOMENTUM_AGENT_ID => Some("MOMENTUM"),
        MEAN_REVERSION_AGENT_ID => Some("MEAN_REVERSION"),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionPhase { Opening, Morning, Midday, Afternoon, Close }

impl SessionPhase {
    pub fn as_str(self) -> &'static str {
        match self { Self::Opening => "OPENING", Self::Morning => "MORNING", Self::Midday => "MIDDAY", Self::Afternoon => "AFTERNOON", Self::Close => "CLOSE" }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntradayFeatures {
    pub candle_index: usize,
    pub timestamp_utc: String,
    pub session_id: String,
    pub close: f64,
    pub ema_9: Option<f64>,
    pub ema_21: Option<f64>,
    pub ema_9_slope: Option<f64>,
    pub ema_21_slope: Option<f64>,
    pub vwap: Option<f64>,
    pub rsi_14: Option<f64>,
    pub atr_14: Option<f64>,
    pub relative_volume: Option<f64>,
    pub rolling_high: Option<f64>,
    pub rolling_low: Option<f64>,
    pub short_term_return: Option<f64>,
    pub distance_from_vwap: Option<f64>,
    pub distance_from_vwap_atr: Option<f64>,
    pub ema_spread_atr: Option<f64>,
    pub price_range_atr: Option<f64>,
    pub session_progress: f64,
    pub session_phase: SessionPhase,
    pub ready: bool,
}

fn average(values: &VecDeque<f64>) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn phase(timestamp_utc: &str, timezone: Tz) -> Result<(SessionPhase, f64), String> {
    let utc = DateTime::parse_from_rfc3339(timestamp_utc).map_err(|error| error.to_string())?;
    let local = utc.with_timezone(&timezone);
    let minute = local.hour() * 60 + local.minute();
    let phase = match minute {
        0..=629 => SessionPhase::Opening,
        630..=719 => SessionPhase::Morning,
        720..=839 => SessionPhase::Midday,
        840..=929 => SessionPhase::Afternoon,
        _ => SessionPhase::Close,
    };
    let progress = ((minute as f64 - 570.0) / 375.0).clamp(0.0, 1.0);
    Ok((phase, progress))
}

pub fn compute_features(candles: &[Candle], timezone_name: &str) -> Result<Vec<IntradayFeatures>, String> {
    let timezone = Tz::from_str(timezone_name).map_err(|_| format!("timezone IANA inválido: {timezone_name}"))?;
    let mut result = Vec::with_capacity(candles.len());
    let mut closes_9 = VecDeque::new();
    let mut closes_21 = VecDeque::new();
    let mut changes = VecDeque::new();
    let mut true_ranges = VecDeque::new();
    let mut prior_volumes = VecDeque::new();
    let mut previous_ema_9 = None;
    let mut previous_ema_21 = None;
    let mut session_id = String::new();
    let mut cumulative_typical_volume = 0.0;
    let mut cumulative_volume = 0.0;

    for (index, candle) in candles.iter().enumerate() {
        if session_id != candle.session_id {
            session_id = candle.session_id.clone();
            cumulative_typical_volume = 0.0;
            cumulative_volume = 0.0;
        }
        closes_9.push_back(candle.close);
        closes_21.push_back(candle.close);
        if closes_9.len() > 9 { closes_9.pop_front(); }
        if closes_21.len() > 21 { closes_21.pop_front(); }
        let ema_9 = if closes_9.len() < 9 { None } else if let Some(previous) = previous_ema_9 { Some(candle.close * (2.0 / 10.0) + previous * (1.0 - 2.0 / 10.0)) } else { average(&closes_9) };
        let ema_21 = if closes_21.len() < 21 { None } else if let Some(previous) = previous_ema_21 { Some(candle.close * (2.0 / 22.0) + previous * (1.0 - 2.0 / 22.0)) } else { average(&closes_21) };
        let ema_9_slope = ema_9.zip(previous_ema_9).map(|(current, previous)| current - previous);
        let ema_21_slope = ema_21.zip(previous_ema_21).map(|(current, previous)| current - previous);

        if let Some(previous) = index.checked_sub(1).map(|value| &candles[value]) {
            changes.push_back(candle.close - previous.close);
            true_ranges.push_back((candle.high - candle.low).max((candle.high - previous.close).abs()).max((candle.low - previous.close).abs()));
            if changes.len() > 14 { changes.pop_front(); }
            if true_ranges.len() > 14 { true_ranges.pop_front(); }
        }
        let rsi_14 = if changes.len() == 14 {
            let gain = changes.iter().map(|value| value.max(0.0)).sum::<f64>() / 14.0;
            let loss = changes.iter().map(|value| (-value).max(0.0)).sum::<f64>() / 14.0;
            Some(if loss == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + gain / loss) })
        } else { None };
        let atr_14 = (true_ranges.len() == 14).then(|| true_ranges.iter().sum::<f64>() / 14.0);
        let relative_volume = candle.volume.and_then(|volume| (prior_volumes.len() == 20).then(|| volume / (prior_volumes.iter().sum::<f64>() / 20.0).max(f64::EPSILON)));
        if let Some(volume) = candle.volume {
            prior_volumes.push_back(volume);
            if prior_volumes.len() > 20 { prior_volumes.pop_front(); }
            let typical = (candle.high + candle.low + candle.close) / 3.0;
            cumulative_typical_volume += typical * volume;
            cumulative_volume += volume;
        }
        let vwap = (cumulative_volume > 0.0).then(|| cumulative_typical_volume / cumulative_volume);
        let previous_start = index.saturating_sub(8);
        let previous = &candles[previous_start..index];
        let rolling_high = (!previous.is_empty()).then(|| previous.iter().map(|value| value.high).fold(f64::NEG_INFINITY, f64::max));
        let rolling_low = (!previous.is_empty()).then(|| previous.iter().map(|value| value.low).fold(f64::INFINITY, f64::min));
        let short_term_return = (index >= 3).then(|| candle.close / candles[index - 3].close - 1.0);
        let distance_from_vwap = vwap.map(|value| candle.close - value);
        let distance_from_vwap_atr = distance_from_vwap.zip(atr_14).and_then(|(distance, atr)| (atr > 0.0).then_some(distance / atr));
        let ema_spread_atr = ema_9.zip(ema_21).zip(atr_14).and_then(|((fast, slow), atr)| (atr > 0.0).then_some((fast - slow) / atr));
        let price_range_atr = atr_14.and_then(|atr| (atr > 0.0).then_some((candle.high - candle.low) / atr));
        let (session_phase, session_progress) = phase(&candle.timestamp_utc, timezone)?;
        let ready = ema_21.is_some() && ema_21_slope.is_some() && rsi_14.is_some() && atr_14.is_some() && rolling_high.is_some();
        result.push(IntradayFeatures { candle_index: index, timestamp_utc: candle.timestamp_utc.clone(), session_id: candle.session_id.clone(), close: candle.close, ema_9, ema_21, ema_9_slope, ema_21_slope, vwap, rsi_14, atr_14, relative_volume, rolling_high, rolling_low, short_term_return, distance_from_vwap, distance_from_vwap_atr, ema_spread_atr, price_range_atr, session_progress, session_phase, ready });
        if ema_9.is_some() { previous_ema_9 = ema_9; }
        if ema_21.is_some() { previous_ema_21 = ema_21; }
    }
    Ok(result)
}

#[derive(Debug, Clone)]
pub struct StrategyContext<'a> { pub features: &'a IntradayFeatures, pub previous: Option<&'a IntradayFeatures>, pub position_state: PositionState }

#[derive(Debug, Clone)]
pub struct StrategyDecision {
    pub intent: PositionIntent,
    pub confidence: f64,
    pub reason_code: &'static str,
    pub reason: String,
    pub indicators: Value,
    pub thresholds: Value,
}

pub trait IntradayStrategy { fn evaluate(&self, context: &StrategyContext<'_>) -> StrategyDecision; }

#[derive(Default)] pub struct IntradayTrend;
impl IntradayStrategy for IntradayTrend {
    fn evaluate(&self, context: &StrategyContext<'_>) -> StrategyDecision {
        let f = context.features;
        let thresholds = json!({"configVersion":TREND_CONFIG_VERSION,"fastEma":9,"slowEma":21});
        let indicators = json!({"ema9":f.ema_9,"ema21":f.ema_21,"ema9Slope":f.ema_9_slope,"ema21Slope":f.ema_21_slope,"price":f.close});
        let (Some(fast), Some(slow), Some(fast_slope), Some(slow_slope)) = (f.ema_9, f.ema_21, f.ema_9_slope, f.ema_21_slope) else { return decision(PositionIntent::Hold, 0.0, "NOT_READY", "Warmup EMA incompleto", indicators, thresholds); };
        let bullish_cross = context.previous.is_some_and(|p| p.ema_9.zip(p.ema_21).is_some_and(|(a,b)| a <= b)) && fast > slow;
        if context.position_state == PositionState::Flat && f.close > fast && fast > slow && fast_slope > 0.0 && slow_slope > 0.0 {
            return decision(PositionIntent::Enter, 0.75, if bullish_cross {"EMA_CROSS_BULLISH"} else {"TREND_ALIGNMENT"}, "Preço e EMAs alinhados em tendência de alta", indicators, thresholds);
        }
        if context.position_state != PositionState::Flat && fast < slow {
            return decision(PositionIntent::Exit, 0.8, "EMA_CROSS_BEARISH", "EMA9 abaixo da EMA21", indicators, thresholds);
        }
        if context.position_state != PositionState::Flat && (f.close < fast || fast_slope <= 0.0) {
            return decision(PositionIntent::Reduce, 0.6, "TREND_WEAKENING", "Tendência intraday perdeu força", indicators, thresholds);
        }
        decision(PositionIntent::Hold, 0.5, "TREND_NO_SIGNAL", "Sem novo sinal de tendência", indicators, thresholds)
    }
}

#[derive(Default)] pub struct IntradayMomentum;
impl IntradayStrategy for IntradayMomentum {
    fn evaluate(&self, context: &StrategyContext<'_>) -> StrategyDecision {
        let f = context.features;
        let thresholds = json!({"configVersion":MOMENTUM_CONFIG_VERSION,"breakoutLookback":8,"atrPeriod":14,"relativeVolumePeriod":20,"minimumRelativeVolume":1.2,"minimumRangeAtr":0.8});
        let indicators = json!({"price":f.close,"rollingHigh":f.rolling_high,"rollingLow":f.rolling_low,"shortTermReturn":f.short_term_return,"relativeVolume":f.relative_volume,"atr":f.atr_14,"priceRangeAtr":f.price_range_atr});
        let (Some(high), Some(momentum), Some(relative_volume), Some(range_atr)) = (f.rolling_high, f.short_term_return, f.relative_volume, f.price_range_atr) else { return decision(PositionIntent::Hold, 0.0, "NOT_READY", "Warmup de breakout, ATR ou volume incompleto", indicators, thresholds); };
        let breakout = f.close > high && momentum > 0.0;
        if context.position_state == PositionState::Flat && breakout && relative_volume >= 1.2 && range_atr >= 0.8 {
            return decision(PositionIntent::Enter, 0.8, "BREAKOUT_CONFIRMED", "Breakout confirmado por retorno, volume e expansão", indicators, thresholds);
        }
        if context.position_state == PositionState::Flat && breakout {
            return decision(PositionIntent::Hold, 0.4, "BREAKOUT_WEAK_VOLUME", "Breakout sem confirmação suficiente", indicators, thresholds);
        }
        if context.position_state != PositionState::Flat && momentum <= 0.0 {
            return decision(PositionIntent::Exit, 0.7, "MOMENTUM_FADE", "Momentum de curto prazo perdeu direção", indicators, thresholds);
        }
        decision(PositionIntent::Hold, 0.5, "MOMENTUM_NO_SIGNAL", "Sem aceleração ou breakout confirmado", indicators, thresholds)
    }
}

#[derive(Default)] pub struct IntradayMeanReversion;
impl IntradayStrategy for IntradayMeanReversion {
    fn evaluate(&self, context: &StrategyContext<'_>) -> StrategyDecision {
        let f = context.features;
        let thresholds = json!({"configVersion":MEAN_REVERSION_CONFIG_VERSION,"rsiPeriod":14,"atrPeriod":14,"entryDistanceAtr":-1.0,"oversoldRsi":30,"exitRsi":55,"vwapReset":"SESSION"});
        let indicators = json!({"price":f.close,"vwap":f.vwap,"rsi14":f.rsi_14,"atr14":f.atr_14,"distanceFromVwapAtr":f.distance_from_vwap_atr});
        let (Some(vwap), Some(rsi), Some(distance)) = (f.vwap, f.rsi_14, f.distance_from_vwap_atr) else { return decision(PositionIntent::Hold, 0.0, "NOT_READY", "Warmup de VWAP, RSI ou ATR incompleto", indicators, thresholds); };
        if context.position_state == PositionState::Flat && distance <= -1.0 && rsi <= 30.0 {
            return decision(PositionIntent::Enter, 0.8, "MEAN_REVERSION_CONFIRMED", "Desvio negativo de VWAP com RSI sobrevendido", indicators, thresholds);
        }
        if context.position_state != PositionState::Flat && (f.close >= vwap || rsi >= 55.0) {
            return decision(PositionIntent::Exit, 0.75, "MEAN_REVERSION_CONFIRMED", "Preço retornou à média intraday", indicators, thresholds);
        }
        if context.position_state != PositionState::Flat && distance <= -2.0 {
            return decision(PositionIntent::Exit, 0.85, "MEAN_REVERSION_FAILED", "Desvio continuou contra a posição", indicators, thresholds);
        }
        decision(PositionIntent::Hold, 0.5, if distance <= -1.0 {"VWAP_DEVIATION"} else if rsi <= 30.0 {"RSI_OVERSOLD"} else {"MEAN_REVERSION_NO_SIGNAL"}, "Condições de retorno à média ainda incompletas", indicators, thresholds)
    }
}

fn decision(intent: PositionIntent, confidence: f64, reason_code: &'static str, reason: &str, indicators: Value, thresholds: Value) -> StrategyDecision {
    StrategyDecision { intent, confidence, reason_code, reason: reason.into(), indicators, thresholds }
}

pub fn evaluate(agent_id: &str, context: &StrategyContext<'_>) -> Option<StrategyDecision> {
    match agent_id { TREND_AGENT_ID => Some(IntradayTrend.evaluate(context)), MOMENTUM_AGENT_ID => Some(IntradayMomentum.evaluate(context)), MEAN_REVERSION_AGENT_ID => Some(IntradayMeanReversion.evaluate(context)), _ => None }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn candle(index: usize, session: &str, close: f64, volume: f64) -> Candle {
        let minutes = 570 + (index % 26) * 15;
        Candle { timestamp: format!("{session} {:02}:{:02}", minutes/60, minutes%60), timestamp_utc: format!("{session}T{:02}:{:02}:00Z", (minutes/60)+5, minutes%60), epoch_seconds:index as i64*900, session_id:session.into(), session_state:"REGULAR".into(), open:close-0.2, high:close+0.5, low:close-0.5, close, volume:Some(volume) }
    }
    #[test] fn rolling_breakout_excludes_current_candle() { let candles=(0..10).map(|i|candle(i,"2026-01-05",100.0+i as f64,1000.0)).collect::<Vec<_>>(); let f=compute_features(&candles,"America/New_York").unwrap(); assert_eq!(f[9].rolling_high,Some(108.5)); assert!(f[9].close>f[9].rolling_high.unwrap()); }
    #[test] fn vwap_resets_between_sessions() { let candles=vec![candle(0,"2026-01-05",100.0,10.0),candle(1,"2026-01-05",110.0,10.0),candle(0,"2026-01-06",200.0,10.0)]; let f=compute_features(&candles,"America/New_York").unwrap(); assert!(f[1].vwap.unwrap()<110.0); assert!((f[2].vwap.unwrap()-200.0).abs()<0.01); }
    #[test] fn phases_follow_us_regular_session() { let sessions=(0..26).map(|i|candle(i,"2026-01-05",100.0,1000.0)).collect::<Vec<_>>(); let f=compute_features(&sessions,"America/New_York").unwrap(); assert_eq!(f[0].session_phase,SessionPhase::Opening); assert_eq!(f[4].session_phase,SessionPhase::Morning); assert_eq!(f[10].session_phase,SessionPhase::Midday); assert_eq!(f[18].session_phase,SessionPhase::Afternoon); assert_eq!(f[24].session_phase,SessionPhase::Close); assert_eq!(f[0].session_progress,0.0); assert_eq!(f[25].session_progress,1.0); }
    #[test] fn features_have_warmup_and_finite_indicators() { let candles=(0..30).map(|i|candle(i,"2026-01-05",100.0+i as f64*0.2,1000.0+i as f64)).collect::<Vec<_>>(); let f=compute_features(&candles,"America/New_York").unwrap(); assert!(!f[19].ready); assert!(f[21].ready); assert!(f[21].ema_9.unwrap().is_finite()); assert!(f[21].ema_21.unwrap().is_finite()); assert!(f[21].atr_14.unwrap()>0.0); assert_eq!(f[21].rsi_14,Some(100.0)); }
    #[test] fn strategies_are_behaviorally_distinct() { let feature=IntradayFeatures { candle_index:30,timestamp_utc:"2026-01-05T16:00:00Z".into(),session_id:"2026-01-05".into(),close:104.0,ema_9:Some(103.0),ema_21:Some(102.0),ema_9_slope:Some(0.2),ema_21_slope:Some(0.1),vwap:Some(100.0),rsi_14:Some(65.0),atr_14:Some(1.0),relative_volume:Some(1.5),rolling_high:Some(103.0),rolling_low:Some(98.0),short_term_return:Some(0.02),distance_from_vwap:Some(4.0),distance_from_vwap_atr:Some(4.0),ema_spread_atr:Some(1.0),price_range_atr:Some(1.1),session_progress:0.5,session_phase:SessionPhase::Midday,ready:true }; let ctx=StrategyContext{features:&feature,previous:None,position_state:PositionState::Flat}; assert_eq!(evaluate(TREND_AGENT_ID,&ctx).unwrap().intent,PositionIntent::Enter); assert_eq!(evaluate(MOMENTUM_AGENT_ID,&ctx).unwrap().intent,PositionIntent::Enter); assert_eq!(evaluate(MEAN_REVERSION_AGENT_ID,&ctx).unwrap().intent,PositionIntent::Hold); }
    #[test] fn mean_reversion_detects_oversold_vwap_deviation() { let mut feature=IntradayFeatures { candle_index:30,timestamp_utc:String::new(),session_id:String::new(),close:97.0,ema_9:Some(98.0),ema_21:Some(99.0),ema_9_slope:Some(-0.1),ema_21_slope:Some(-0.1),vwap:Some(100.0),rsi_14:Some(25.0),atr_14:Some(1.0),relative_volume:Some(1.0),rolling_high:Some(101.0),rolling_low:Some(96.0),short_term_return:Some(-0.01),distance_from_vwap:Some(-3.0),distance_from_vwap_atr:Some(-3.0),ema_spread_atr:Some(-1.0),price_range_atr:Some(1.0),session_progress:0.5,session_phase:SessionPhase::Midday,ready:true }; let result=evaluate(MEAN_REVERSION_AGENT_ID,&StrategyContext{features:&feature,previous:None,position_state:PositionState::Flat}).unwrap(); assert_eq!(result.intent,PositionIntent::Enter); assert_eq!(result.reason_code,"MEAN_REVERSION_CONFIRMED"); feature.rsi_14=Some(50.0); assert_eq!(evaluate(MEAN_REVERSION_AGENT_ID,&StrategyContext{features:&feature,previous:None,position_state:PositionState::Flat}).unwrap().intent,PositionIntent::Hold); }
    #[test] fn ema_rsi_atr_volume_low_and_normalization_are_deterministic() { let candles=(0..30).map(|i|candle(i,"2026-01-05",100.0+i as f64,if i==20{2000.0}else{1000.0})).collect::<Vec<_>>(); let f=compute_features(&candles,"America/New_York").unwrap(); assert!((f[8].ema_9.unwrap()-104.0).abs()<1e-9); assert!((f[20].ema_21.unwrap()-110.0).abs()<1e-9); assert!(f[21].ema_9_slope.unwrap()>0.0&&f[21].ema_21_slope.unwrap()>0.0); assert_eq!(f[21].rsi_14,Some(100.0)); assert!((f[21].atr_14.unwrap()-1.5).abs()<1e-9); assert!(f[20].relative_volume.unwrap()>1.9); assert_eq!(f[9].rolling_low,Some(100.5)); assert!(f[21].distance_from_vwap_atr.unwrap().is_finite()); }
    #[test] fn future_mutation_never_changes_past_features() { let candles=(0..30).map(|i|candle(i,"2026-01-05",100.0+i as f64*0.1,1000.0)).collect::<Vec<_>>(); let before=compute_features(&candles,"America/New_York").unwrap(); let mut changed=candles.clone(); changed[29].close=10000.0; changed[29].high=10001.0; let after=compute_features(&changed,"America/New_York").unwrap(); assert_eq!(before[28].ema_9,after[28].ema_9); assert_eq!(before[28].ema_21,after[28].ema_21); assert_eq!(before[28].vwap,after[28].vwap); assert_eq!(before[28].rolling_high,after[28].rolling_high); }
    #[test] fn strategy_exit_and_weak_signal_paths_are_explicit() { let base=IntradayFeatures { candle_index:30,timestamp_utc:String::new(),session_id:String::new(),close:100.0,ema_9:Some(99.0),ema_21:Some(100.0),ema_9_slope:Some(-0.1),ema_21_slope:Some(0.0),vwap:Some(100.0),rsi_14:Some(60.0),atr_14:Some(1.0),relative_volume:Some(0.8),rolling_high:Some(99.0),rolling_low:Some(95.0),short_term_return:Some(0.01),distance_from_vwap:Some(0.0),distance_from_vwap_atr:Some(0.0),ema_spread_atr:Some(-1.0),price_range_atr:Some(0.5),session_progress:0.5,session_phase:SessionPhase::Midday,ready:true }; let trend=evaluate(TREND_AGENT_ID,&StrategyContext{features:&base,previous:None,position_state:PositionState::LongOpen}).unwrap(); assert_eq!(trend.intent,PositionIntent::Exit); assert_eq!(trend.reason_code,"EMA_CROSS_BEARISH"); let momentum=evaluate(MOMENTUM_AGENT_ID,&StrategyContext{features:&base,previous:None,position_state:PositionState::Flat}).unwrap(); assert_eq!(momentum.intent,PositionIntent::Hold); assert_eq!(momentum.reason_code,"BREAKOUT_WEAK_VOLUME"); let mut fade=base.clone(); fade.short_term_return=Some(-0.01); let momentum=evaluate(MOMENTUM_AGENT_ID,&StrategyContext{features:&fade,previous:None,position_state:PositionState::LongOpen}).unwrap(); assert_eq!(momentum.intent,PositionIntent::Exit); assert_eq!(momentum.reason_code,"MOMENTUM_FADE"); }
}
