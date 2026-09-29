use serde::{Deserialize, Serialize};

pub const VALIDATION_ENGINE_VERSION: &str = "MULTI_PERIOD_LIFECYCLE_VALIDATION_V1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleValidationPeriodInput {
    pub experiment_id: String,
    pub source_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleValidationBatchInput {
    pub name: String,
    pub periods: Vec<LifecycleValidationPeriodInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverlapStatus {
    NonOverlapping,
    TouchingBoundary,
    Overlapping,
    DuplicateRange,
}

impl OverlapStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NonOverlapping => "NON_OVERLAPPING",
            Self::TouchingBoundary => "TOUCHING_BOUNDARY",
            Self::Overlapping => "OVERLAPPING",
            Self::DuplicateRange => "DUPLICATE_RANGE",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DistributionSummary {
    pub average: f64,
    pub median: f64,
    pub p25: Option<f64>,
    pub p75: Option<f64>,
    pub p90: Option<f64>,
    pub minimum: f64,
    pub maximum: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleValidationBatch {
    pub batch_id: String,
    pub name: String,
    pub status: String,
    pub asset: String,
    pub timeframe: String,
    pub agent_id: String,
    pub agent_version: String,
    pub lifecycle_config_version: String,
    pub deterioration_config_version: String,
    pub hold_diagnostics_version: String,
    pub execution_model_version: String,
    pub validation_engine_version: String,
    pub dataset_count: usize,
    pub experiment_count: usize,
    pub lifecycle_count: usize,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSampleSummary {
    pub datasets: usize,
    pub periods: usize,
    pub experiments: usize,
    pub lifecycles: usize,
    pub closed: usize,
    pub open: usize,
    #[serde(default)]
    pub censored: usize,
    pub date_coverage_start: String,
    pub date_coverage_end: String,
    pub lifecycle_late_reduction_rate_pct: f64,
    pub event_late_reduction_rate_pct: f64,
    pub median_response_delay_minutes: Option<f64>,
    pub score_stability: String,
    pub sample_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetQualityReport {
    pub dataset_id: String,
    pub dataset_name: String,
    pub status: String,
    pub candle_count: usize,
    pub session_count: usize,
    pub first_at: String,
    pub last_at: String,
    pub null_value_count: usize,
    pub duplicate_timestamp_count: usize,
    pub out_of_order_count: usize,
    pub nonpositive_price_count: usize,
    pub negative_volume_count: usize,
    pub invalid_high_count: usize,
    pub invalid_low_count: usize,
    pub expected_gap_count: usize,
    pub unexpected_gap_count: usize,
    pub market: String,
    pub timezone: String,
    pub session_type: String,
    pub source_file_available: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ValidationAuditSummary {
    pub reused_experiments: usize,
    pub rebuilt_artifacts: usize,
    pub new_llm_runs: usize,
    pub failed_periods: usize,
    pub skipped_periods: usize,
    pub duration_ms: usize,
    pub model: String,
    pub prompt_version: String,
    pub context_version: String,
    pub trigger_version: String,
    pub position_sizing_version: String,
    pub risk_policy_version: String,
    pub execution_model_version: String,
    pub fee_pct: f64,
    pub slippage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PeriodContextSummary {
    pub period_return_pct: f64,
    pub realized_volatility_pct: f64,
    pub average_atr_pct: Option<f64>,
    pub trend_proxy: String,
    pub average_relative_volume: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleValidationPeriod {
    pub period_id: String,
    pub experiment_id: String,
    pub experiment_name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub source_role: String,
    pub start_at: String,
    pub end_at: String,
    pub candle_count: usize,
    pub session_count: usize,
    pub lifecycle_count: usize,
    pub lifecycle_run_id: String,
    pub closed_count: usize,
    pub open_count: usize,
    pub duration: DistributionSummary,
    pub pnl: DistributionSummary,
    pub mfe: DistributionSummary,
    pub mae: DistributionSummary,
    pub giveback: DistributionSummary,
    pub late_reduction_events: usize,
    pub late_reduction_lifecycles: usize,
    pub eligible_lifecycles: usize,
    pub event_late_reduction_rate_pct: f64,
    pub lifecycle_late_reduction_rate_pct: f64,
    pub average_forward_5: Option<f64>,
    pub average_mae_5: Option<f64>,
    pub response_delay: DistributionSummary,
    pub unresolved_response_count: usize,
    pub overlap_status: String,
    pub context: PeriodContextSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct HealthDistribution {
    pub strong_pct: f64,
    pub healthy_pct: f64,
    pub weakening_pct: f64,
    pub deteriorating_pct: f64,
    pub critical_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSnapshot {
    pub trend: f64,
    pub momentum: f64,
    pub giveback: f64,
    pub vwap: f64,
    pub volatility: f64,
    pub time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleValidationLifecycle {
    pub period_id: String,
    pub source_role: String,
    pub experiment_id: String,
    pub lifecycle_id: String,
    pub entry_at: String,
    pub exit_at: Option<String>,
    pub status: String,
    pub duration_minutes: usize,
    pub pnl_pct: Option<f64>,
    pub mfe_pct: f64,
    pub mae_pct: f64,
    pub giveback_pct: f64,
    pub worst_health: String,
    pub dominant_health: String,
    pub health_distribution: HealthDistribution,
    pub max_deterioration_score: f64,
    pub max_deterioration_level: String,
    pub first_timestamp_at_max_level: Option<String>,
    pub late_reduction_events: usize,
    pub has_late_reduction: bool,
    pub response_delay_minutes: Option<usize>,
    pub response_status: String,
    pub response_from_first_deterioration_minutes: Option<usize>,
    pub response_from_high_minutes: Option<usize>,
    pub response_from_critical_minutes: Option<usize>,
    pub overnight: bool,
    pub censored_at_dataset_end: bool,
    pub average_components: ComponentSnapshot,
    pub maximum_components: ComponentSnapshot,
    pub component_at_first_high: Option<ComponentSnapshot>,
    pub component_at_first_critical: Option<ComponentSnapshot>,
    pub dominant_deterioration_component: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeriodDeteriorationDistribution {
    pub period_id: String,
    pub level: String,
    pub event_count: usize,
    pub event_pct: f64,
    pub lifecycle_count: usize,
    pub lifecycle_reach_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleLevelOutcome {
    pub label: String,
    pub lifecycle_count: usize,
    pub average_pnl_pct: Option<f64>,
    pub median_pnl_pct: Option<f64>,
    pub average_mfe_pct: f64,
    pub average_mae_pct: f64,
    pub average_giveback_pct: f64,
    pub late_reduction_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOutcomeSummary {
    pub period_id: Option<String>,
    pub component: String,
    pub lifecycle_count: usize,
    pub average_value: f64,
    pub maximum_value: f64,
    pub average_pnl_pct: Option<f64>,
    pub average_mae_pct: f64,
    pub average_giveback_pct: f64,
    pub late_reduction_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CrossPeriodMetric {
    pub label: String,
    pub values: Vec<f64>,
    pub median: f64,
    pub minimum: f64,
    pub maximum: f64,
    pub dispersion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceMatrix {
    pub sample_size: String,
    #[serde(default)]
    pub period_coverage: String,
    pub late_reduction: String,
    pub deterioration_stability: String,
    pub health_outcome_relation: String,
    pub component_consistency: String,
    #[serde(default)]
    pub giveback_consistency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutlierSensitivity {
    pub metric: String,
    pub lifecycle_id: Option<String>,
    pub full_average: f64,
    pub without_largest_outlier_average: f64,
    pub delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiPeriodLifecycleValidationReport {
    pub batch: LifecycleValidationBatch,
    pub sample: LifecycleSampleSummary,
    pub periods: Vec<LifecycleValidationPeriod>,
    pub lifecycles: Vec<LifecycleValidationLifecycle>,
    pub deterioration_by_period: Vec<PeriodDeteriorationDistribution>,
    pub deterioration_outcomes: Vec<LifecycleLevelOutcome>,
    pub health_outcomes: Vec<LifecycleLevelOutcome>,
    pub components: Vec<ComponentOutcomeSummary>,
    pub consistency: Vec<CrossPeriodMetric>,
    pub evidence: EvidenceMatrix,
    pub outlier_sensitivity: Vec<OutlierSensitivity>,
    #[serde(default)]
    pub quality: Vec<DatasetQualityReport>,
    #[serde(default)]
    pub audit: ValidationAuditSummary,
    pub warnings: Vec<String>,
    pub post_decision_only: bool,
}

pub fn round(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        round(values.iter().sum::<f64>() / values.len() as f64)
    }
}

pub fn percentile(values: &[f64], percentile: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = percentile.clamp(0.0, 1.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        round(sorted[lower])
    } else {
        round(sorted[lower] + (sorted[upper] - sorted[lower]) * (rank - lower as f64))
    }
}

pub fn distribution(values: &[f64]) -> DistributionSummary {
    if values.is_empty() {
        return DistributionSummary::default();
    }
    DistributionSummary {
        average: average(values),
        median: percentile(values, 0.5),
        p25: (values.len() >= 4).then(|| percentile(values, 0.25)),
        p75: (values.len() >= 4).then(|| percentile(values, 0.75)),
        p90: (values.len() >= 10).then(|| percentile(values, 0.9)),
        minimum: round(values.iter().copied().fold(f64::INFINITY, f64::min)),
        maximum: round(values.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
    }
}

pub fn classify_overlap(
    left_start: &str,
    left_end: &str,
    right_start: &str,
    right_end: &str,
) -> OverlapStatus {
    if left_start == right_start && left_end == right_end {
        OverlapStatus::DuplicateRange
    } else if left_end == right_start || right_end == left_start {
        OverlapStatus::TouchingBoundary
    } else if left_start < right_end && right_start < left_end {
        OverlapStatus::Overlapping
    } else {
        OverlapStatus::NonOverlapping
    }
}

pub fn sample_status(lifecycles: usize) -> String {
    match lifecycles {
        0..=19 => "INSUFFICIENT",
        20..=29 => "GROWING",
        _ => "BROADER",
    }
    .into()
}

pub fn stability_status(values: &[f64], sample_size: usize) -> String {
    if values.len() < 2 || sample_size < 4 {
        return "INSUFFICIENT_SAMPLE".into();
    }
    let spread = values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - values.iter().copied().fold(f64::INFINITY, f64::min);
    if spread <= 0.1 {
        "STABLE"
    } else if spread <= 0.25 {
        "SHIFTED"
    } else {
        "HIGHLY_SHIFTED"
    }
    .into()
}

pub fn rate_consistency(values: &[f64]) -> String {
    if values.len() < 2 {
        return "INCONSISTENT".into();
    }
    let spread = values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        - values.iter().copied().fold(f64::INFINITY, f64::min);
    if spread <= 15.0 {
        "CONSISTENT"
    } else if spread <= 30.0 {
        "MIXED"
    } else {
        "INCONSISTENT"
    }
    .into()
}

pub fn concentration_warning(period_counts: &[usize]) -> Option<String> {
    let total: usize = period_counts.iter().sum();
    let largest = period_counts.iter().copied().max().unwrap_or_default();
    if total > 0 && largest as f64 / total as f64 > 0.6 {
        Some(format!(
            "CONCENTRATION WARNING: um período concentra {:.2}% dos lifecycles",
            largest as f64 * 100.0 / total as f64
        ))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlap_detector_distinguishes_all_ranges() {
        assert_eq!(
            classify_overlap("2026-01-01", "2026-01-10", "2026-01-11", "2026-01-20"),
            OverlapStatus::NonOverlapping
        );
        assert_eq!(
            classify_overlap("2026-01-01", "2026-01-10", "2026-01-10", "2026-01-20"),
            OverlapStatus::TouchingBoundary
        );
        assert_eq!(
            classify_overlap("2026-01-01", "2026-01-12", "2026-01-10", "2026-01-20"),
            OverlapStatus::Overlapping
        );
        assert_eq!(
            classify_overlap("2026-01-01", "2026-01-10", "2026-01-01", "2026-01-10"),
            OverlapStatus::DuplicateRange
        );
    }

    #[test]
    fn distribution_and_sample_status_are_lifecycle_based() {
        let values = vec![1.0, 2.0, 3.0, 100.0];
        let summary = distribution(&values);
        assert_eq!(summary.median, 2.5);
        assert_eq!(summary.p25, Some(1.75));
        assert_eq!(sample_status(19), "INSUFFICIENT");
        assert_eq!(sample_status(20), "GROWING");
        assert_eq!(sample_status(30), "BROADER");
    }

    #[test]
    fn stability_and_concentration_are_descriptive() {
        assert_eq!(stability_status(&[0.2, 0.25], 8), "STABLE");
        assert_eq!(stability_status(&[0.1, 0.5], 8), "HIGHLY_SHIFTED");
        assert!(concentration_warning(&[7, 3]).is_some());
        assert!(concentration_warning(&[5, 5]).is_none());
    }
}
