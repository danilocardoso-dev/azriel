use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const ENGINE_VERSION: &str = "HOLD_DIAGNOSTICS_V1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldDiagnosticsConfig {
    pub missed_forward_5_pct: f64,
    pub missed_mfe_5_pct: f64,
    pub adverse_forward_5_pct: f64,
    pub adverse_mae_5_pct: f64,
    pub concentration_warning_pct: f64,
    pub candidate_minimum_count: usize,
    pub candidate_quality_rate_pct: f64,
    pub candidate_effect_pct: f64,
}

impl Default for HoldDiagnosticsConfig {
    fn default() -> Self {
        Self {
            missed_forward_5_pct: 0.30,
            missed_mfe_5_pct: 0.50,
            adverse_forward_5_pct: -0.30,
            adverse_mae_5_pct: -0.50,
            concentration_warning_pct: 90.0,
            candidate_minimum_count: 10,
            candidate_quality_rate_pct: 50.0,
            candidate_effect_pct: 0.30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldPositionState {
    Flat,
    Long,
}

impl HoldPositionState {
    pub fn from_exposure(exposure_pct: f64) -> Self {
        if exposure_pct.abs() <= 0.0001 {
            Self::Flat
        } else {
            Self::Long
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flat => "FLAT",
            Self::Long => "LONG",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HoldQuality {
    GoodHold,
    PotentialMissedOpportunity,
    PotentialLateReduction,
    Inconclusive,
}

impl HoldQuality {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GoodHold => "GOOD_HOLD",
            Self::PotentialMissedOpportunity => "POTENTIAL_MISSED_OPPORTUNITY",
            Self::PotentialLateReduction => "POTENTIAL_LATE_REDUCTION",
            Self::Inconclusive => "INCONCLUSIVE",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OutcomeCandle {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldPostDecisionOutcome {
    pub forward_1: Option<f64>,
    pub forward_5: Option<f64>,
    pub forward_10: Option<f64>,
    pub mfe_1: Option<f64>,
    pub mfe_5: Option<f64>,
    pub mfe_10: Option<f64>,
    pub mae_1: Option<f64>,
    pub mae_5: Option<f64>,
    pub mae_10: Option<f64>,
}

fn round(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn horizon_outcome(
    candles: &[OutcomeCandle],
    candle_index: usize,
    horizon: usize,
    observed_price: f64,
) -> Option<(f64, f64, f64)> {
    if observed_price <= 0.0 || candle_index.checked_add(horizon)? >= candles.len() {
        return None;
    }
    let future = &candles[candle_index + 1..=candle_index + horizon];
    let last = future.last()?;
    let maximum = future
        .iter()
        .map(|item| item.high)
        .fold(f64::NEG_INFINITY, f64::max);
    let minimum = future
        .iter()
        .map(|item| item.low)
        .fold(f64::INFINITY, f64::min);
    Some((
        round((last.close / observed_price - 1.0) * 100.0),
        round(((maximum / observed_price - 1.0) * 100.0).max(0.0)),
        round(((minimum / observed_price - 1.0) * 100.0).min(0.0)),
    ))
}

pub fn post_decision_outcome(
    candles: &[OutcomeCandle],
    candle_index: usize,
    observed_price: f64,
) -> HoldPostDecisionOutcome {
    let one = horizon_outcome(candles, candle_index, 1, observed_price);
    let five = horizon_outcome(candles, candle_index, 5, observed_price);
    let ten = horizon_outcome(candles, candle_index, 10, observed_price);
    HoldPostDecisionOutcome {
        forward_1: one.map(|item| item.0),
        forward_5: five.map(|item| item.0),
        forward_10: ten.map(|item| item.0),
        mfe_1: one.map(|item| item.1),
        mfe_5: five.map(|item| item.1),
        mfe_10: ten.map(|item| item.1),
        mae_1: one.map(|item| item.2),
        mae_5: five.map(|item| item.2),
        mae_10: ten.map(|item| item.2),
    }
}

pub fn classify(
    position: HoldPositionState,
    outcome: &HoldPostDecisionOutcome,
    config: &HoldDiagnosticsConfig,
) -> HoldQuality {
    let Some(forward) = outcome.forward_5 else {
        return HoldQuality::Inconclusive;
    };
    let Some(mfe) = outcome.mfe_5 else {
        return HoldQuality::Inconclusive;
    };
    let Some(mae) = outcome.mae_5 else {
        return HoldQuality::Inconclusive;
    };
    let favorable = forward >= config.missed_forward_5_pct || mfe >= config.missed_mfe_5_pct;
    let adverse = forward <= config.adverse_forward_5_pct || mae <= config.adverse_mae_5_pct;
    if favorable && adverse {
        return HoldQuality::Inconclusive;
    }
    match (position, favorable, adverse) {
        (HoldPositionState::Flat, true, false) => HoldQuality::PotentialMissedOpportunity,
        (HoldPositionState::Flat, false, true) => HoldQuality::GoodHold,
        (HoldPositionState::Long, true, false) => HoldQuality::GoodHold,
        (HoldPositionState::Long, false, true) => HoldQuality::PotentialLateReduction,
        _ => HoldQuality::Inconclusive,
    }
}

pub fn confidence_bucket(confidence: Option<f64>) -> &'static str {
    match confidence {
        Some(value) if value < 0.50 => "LOW",
        Some(value) if value < 0.75 => "MEDIUM",
        Some(_) => "HIGH",
        None => "UNKNOWN",
    }
}

pub fn rsi_bucket(value: Option<f64>) -> &'static str {
    match value {
        Some(value) if value < 30.0 => "<30",
        Some(value) if value < 45.0 => "30-45",
        Some(value) if value < 55.0 => "45-55",
        Some(value) if value <= 70.0 => "55-70",
        Some(_) => ">70",
        None => "UNKNOWN",
    }
}

pub fn relative_volume_bucket(value: Option<f64>) -> &'static str {
    match value {
        Some(value) if value < 0.75 => "<0.75",
        Some(value) if value < 1.0 => "0.75-1.0",
        Some(value) if value <= 1.5 => "1.0-1.5",
        Some(_) => ">1.5",
        None => "UNKNOWN",
    }
}

pub fn vwap_distance_bucket(value: Option<f64>) -> &'static str {
    match value {
        Some(value) if value < -1.0 => "<-1",
        Some(value) if value < 0.0 => "-1-0",
        Some(value) if value <= 1.0 => "0-1",
        Some(_) => ">1",
        None => "UNKNOWN",
    }
}

pub fn ema_spread_bucket(value: Option<f64>) -> &'static str {
    match value {
        Some(value) if value < -0.10 => "NEGATIVE",
        Some(value) if value > 0.10 => "POSITIVE",
        Some(_) => "NEAR_ZERO",
        None => "UNKNOWN",
    }
}

pub fn conflict_signature(
    reason_code: &str,
    trend: &str,
    momentum: &str,
    vwap_position: &str,
    volume: &str,
) -> Option<String> {
    (reason_code == "SIGNAL_CONFLICT").then(|| {
        format!("TREND_{trend} / MOMENTUM_{momentum} / VWAP_{vwap_position} / VOLUME_{volume}")
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldDiagnosticItem {
    pub decision_id: i64,
    pub candle_index: usize,
    pub timestamp: String,
    pub session_id: String,
    pub session_phase: String,
    pub position_state: String,
    pub current_exposure_pct: f64,
    pub confidence: Option<f64>,
    pub confidence_bucket: String,
    pub reason_code: String,
    pub reason: String,
    pub price: f64,
    pub ema_9: Option<f64>,
    pub ema_21: Option<f64>,
    pub vwap: Option<f64>,
    pub rsi_14: Option<f64>,
    pub atr_14: Option<f64>,
    pub relative_volume: Option<f64>,
    pub distance_from_vwap_atr: Option<f64>,
    pub ema_spread_atr: Option<f64>,
    pub trend: String,
    pub momentum: String,
    pub volatility: String,
    pub location: String,
    pub vwap_position: String,
    pub trigger_reason: String,
    pub conflict_signature: Option<String>,
    pub rsi_bucket: String,
    pub relative_volume_bucket: String,
    pub vwap_distance_bucket: String,
    pub ema_spread_bucket: String,
    pub outcome: HoldPostDecisionOutcome,
    pub quality: String,
    pub quality_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldAggregateRow {
    pub key: String,
    pub count: usize,
    pub average_confidence: Option<f64>,
    pub average_forward_1: Option<f64>,
    pub average_forward_5: Option<f64>,
    pub average_forward_10: Option<f64>,
    pub average_mfe_5: Option<f64>,
    pub average_mae_5: Option<f64>,
    pub good_hold_count: usize,
    pub missed_opportunity_count: usize,
    pub late_reduction_count: usize,
    pub inconclusive_count: usize,
    pub good_hold_rate_pct: f64,
    pub missed_opportunity_rate_pct: f64,
    pub late_reduction_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalibrationCandidate {
    pub reason_code: String,
    pub count: usize,
    pub issue: String,
    pub issue_rate_pct: f64,
    pub average_forward_5: Option<f64>,
}

fn average(values: impl Iterator<Item = Option<f64>>) -> Option<f64> {
    let values = values.flatten().collect::<Vec<_>>();
    (!values.is_empty()).then(|| round(values.iter().sum::<f64>() / values.len() as f64))
}

pub fn aggregate_by<F>(items: &[HoldDiagnosticItem], key: F) -> Vec<HoldAggregateRow>
where
    F: Fn(&HoldDiagnosticItem) -> String,
{
    let mut groups: BTreeMap<String, Vec<&HoldDiagnosticItem>> = BTreeMap::new();
    for item in items {
        groups.entry(key(item)).or_default().push(item);
    }
    groups
        .into_iter()
        .map(|(key, group)| {
            let count = group.len();
            let quality_count =
                |quality: &str| group.iter().filter(|item| item.quality == quality).count();
            let good = quality_count("GOOD_HOLD");
            let missed = quality_count("POTENTIAL_MISSED_OPPORTUNITY");
            let late = quality_count("POTENTIAL_LATE_REDUCTION");
            let inconclusive = quality_count("INCONCLUSIVE");
            let rate = |value| round(value as f64 / count.max(1) as f64 * 100.0);
            HoldAggregateRow {
                key,
                count,
                average_confidence: average(group.iter().map(|item| item.confidence)),
                average_forward_1: average(group.iter().map(|item| item.outcome.forward_1)),
                average_forward_5: average(group.iter().map(|item| item.outcome.forward_5)),
                average_forward_10: average(group.iter().map(|item| item.outcome.forward_10)),
                average_mfe_5: average(group.iter().map(|item| item.outcome.mfe_5)),
                average_mae_5: average(group.iter().map(|item| item.outcome.mae_5)),
                good_hold_count: good,
                missed_opportunity_count: missed,
                late_reduction_count: late,
                inconclusive_count: inconclusive,
                good_hold_rate_pct: rate(good),
                missed_opportunity_rate_pct: rate(missed),
                late_reduction_rate_pct: rate(late),
            }
        })
        .collect()
}

pub fn calibration_candidates(
    reasons: &[HoldAggregateRow],
    config: &HoldDiagnosticsConfig,
) -> Vec<CalibrationCandidate> {
    let mut candidates = reasons
        .iter()
        .filter_map(|row| {
            if row.count < config.candidate_minimum_count {
                return None;
            }
            let forward = row.average_forward_5?;
            if row.missed_opportunity_rate_pct >= config.candidate_quality_rate_pct
                && forward >= config.candidate_effect_pct
            {
                Some(CalibrationCandidate {
                    reason_code: row.key.clone(),
                    count: row.count,
                    issue: "MISSED_OPPORTUNITY".into(),
                    issue_rate_pct: row.missed_opportunity_rate_pct,
                    average_forward_5: Some(forward),
                })
            } else if row.late_reduction_rate_pct >= config.candidate_quality_rate_pct
                && forward <= -config.candidate_effect_pct
            {
                Some(CalibrationCandidate {
                    reason_code: row.key.clone(),
                    count: row.count,
                    issue: "LATE_REDUCTION".into(),
                    issue_rate_pct: row.late_reduction_rate_pct,
                    average_forward_5: Some(forward),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .issue_rate_pct
            .total_cmp(&left.issue_rate_pct)
            .then(right.count.cmp(&left.count))
            .then(left.reason_code.cmp(&right.reason_code))
    });
    candidates
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceLevel {
    Insufficient,
    Weak,
    Moderate,
    Strong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldQualitySummary {
    pub total: usize,
    pub good_hold_count: usize,
    pub missed_opportunity_count: usize,
    pub late_reduction_count: usize,
    pub inconclusive_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldDiagnosticsReport {
    pub run_id: String,
    pub experiment_id: String,
    pub experiment_name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub asset: String,
    pub timeframe: String,
    pub agent_id: String,
    pub engine_version: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub total_ai_calls: usize,
    pub total_holds: usize,
    pub hold_rate_pct: f64,
    pub hold_concentration_warning: bool,
    pub flat_count: usize,
    pub long_count: usize,
    pub quality: HoldQualitySummary,
    pub quality_matrix: Vec<HoldAggregateRow>,
    pub reasons: Vec<HoldAggregateRow>,
    pub confidence: Vec<HoldAggregateRow>,
    pub session_phases: Vec<HoldAggregateRow>,
    pub triggers: Vec<HoldAggregateRow>,
    pub trigger_analysis: Vec<HoldTriggerAnalysisRow>,
    pub feature_buckets: Vec<HoldAggregateRow>,
    pub conflicts: Vec<HoldAggregateRow>,
    pub calibration_candidates: Vec<CalibrationCandidate>,
    pub config: HoldDiagnosticsConfig,
    pub items: Vec<HoldDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldTriggerAnalysisRow {
    pub trigger: String,
    pub call_count: usize,
    pub hold_count: usize,
    pub enter_count: usize,
    pub hold_rate_pct: f64,
    pub average_forward_5_after_hold: Option<f64>,
    pub missed_opportunity_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldComparisonMetric {
    pub label: String,
    pub development_value: f64,
    pub out_of_sample_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldCandidateEvidence {
    pub reason_code: String,
    pub issue: String,
    pub development: Option<CalibrationCandidate>,
    pub out_of_sample: Option<CalibrationCandidate>,
    pub evidence: EvidenceLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldDiagnosticsComparison {
    pub development_experiment_id: String,
    pub out_of_sample_experiment_id: String,
    pub asset: String,
    pub timeframe: String,
    pub metrics: Vec<HoldComparisonMetric>,
    pub development_top_reason: Option<String>,
    pub out_of_sample_top_reason: Option<String>,
    pub development_top_candidate: Option<String>,
    pub out_of_sample_top_candidate: Option<String>,
    pub candidate_evidence: Vec<HoldCandidateEvidence>,
}

pub fn summarize_quality(items: &[HoldDiagnosticItem]) -> HoldQualitySummary {
    HoldQualitySummary {
        total: items.len(),
        good_hold_count: items
            .iter()
            .filter(|item| item.quality == "GOOD_HOLD")
            .count(),
        missed_opportunity_count: items
            .iter()
            .filter(|item| item.quality == "POTENTIAL_MISSED_OPPORTUNITY")
            .count(),
        late_reduction_count: items
            .iter()
            .filter(|item| item.quality == "POTENTIAL_LATE_REDUCTION")
            .count(),
        inconclusive_count: items
            .iter()
            .filter(|item| item.quality == "INCONCLUSIVE")
            .count(),
    }
}

pub fn evidence_level(
    development: Option<&CalibrationCandidate>,
    oos: Option<&CalibrationCandidate>,
) -> EvidenceLevel {
    let (Some(dev), Some(oos)) = (development, oos) else {
        return if development.is_some() || oos.is_some() {
            EvidenceLevel::Weak
        } else {
            EvidenceLevel::Insufficient
        };
    };
    if dev.issue != oos.issue {
        return EvidenceLevel::Weak;
    }
    let difference = (dev.issue_rate_pct - oos.issue_rate_pct).abs();
    let minimum_count = dev.count.min(oos.count);
    if minimum_count >= 30 && difference <= 10.0 {
        EvidenceLevel::Strong
    } else if minimum_count >= 20 && difference <= 20.0 {
        EvidenceLevel::Moderate
    } else {
        EvidenceLevel::Weak
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(forward: f64, mfe: f64, mae: f64) -> HoldPostDecisionOutcome {
        HoldPostDecisionOutcome {
            forward_5: Some(forward),
            mfe_5: Some(mfe),
            mae_5: Some(mae),
            ..Default::default()
        }
    }

    #[test]
    fn forward_mfe_and_mae_use_only_future_candles() {
        let candles = vec![
            OutcomeCandle {
                high: 101.0,
                low: 99.0,
                close: 100.0,
            },
            OutcomeCandle {
                high: 102.0,
                low: 98.0,
                close: 101.0,
            },
            OutcomeCandle {
                high: 103.0,
                low: 100.0,
                close: 102.0,
            },
        ];
        let result = post_decision_outcome(&candles, 0, 100.0);
        assert_eq!(result.forward_1, Some(1.0));
        assert_eq!(result.mfe_1, Some(2.0));
        assert_eq!(result.mae_1, Some(-2.0));
        assert_eq!(result.forward_5, None);
    }

    #[test]
    fn hold_quality_distinguishes_flat_and_long() {
        let config = HoldDiagnosticsConfig::default();
        assert_eq!(
            classify(HoldPositionState::Flat, &outcome(0.4, 0.6, -0.1), &config),
            HoldQuality::PotentialMissedOpportunity
        );
        assert_eq!(
            classify(HoldPositionState::Flat, &outcome(-0.4, 0.1, -0.6), &config),
            HoldQuality::GoodHold
        );
        assert_eq!(
            classify(HoldPositionState::Long, &outcome(0.4, 0.6, -0.1), &config),
            HoldQuality::GoodHold
        );
        assert_eq!(
            classify(HoldPositionState::Long, &outcome(-0.4, 0.1, -0.6), &config),
            HoldQuality::PotentialLateReduction
        );
        assert_eq!(
            classify(HoldPositionState::Flat, &outcome(0.4, 0.6, -0.6), &config),
            HoldQuality::Inconclusive
        );
    }

    #[test]
    fn buckets_and_evidence_are_deterministic() {
        assert_eq!(confidence_bucket(Some(0.74)), "MEDIUM");
        assert_eq!(rsi_bucket(Some(45.0)), "45-55");
        assert_eq!(relative_volume_bucket(Some(1.5)), "1.0-1.5");
        assert_eq!(vwap_distance_bucket(Some(-1.0)), "-1-0");
        assert_eq!(ema_spread_bucket(Some(0.0)), "NEAR_ZERO");
        let candidate = CalibrationCandidate {
            reason_code: "SIGNAL_CONFLICT".into(),
            count: 35,
            issue: "MISSED_OPPORTUNITY".into(),
            issue_rate_pct: 55.0,
            average_forward_5: Some(0.4),
        };
        let stable = CalibrationCandidate {
            issue_rate_pct: 51.0,
            ..candidate.clone()
        };
        assert!(matches!(
            evidence_level(Some(&candidate), Some(&stable)),
            EvidenceLevel::Strong
        ));
        assert!(matches!(
            evidence_level(Some(&candidate), None),
            EvidenceLevel::Weak
        ));
    }
}
