use super::{
    market_hold_diagnostics,
    market_lifecycle_repository::{self, LifecycleHealthTrace, LifecycleIntelligenceReport},
    market_lifecycle_validation::{
        average, classify_overlap, concentration_warning, distribution, percentile,
        rate_consistency, round, sample_status, stability_status, ComponentOutcomeSummary,
        ComponentSnapshot, CrossPeriodMetric, DatasetQualityReport, EvidenceMatrix,
        HealthDistribution, LifecycleLevelOutcome, LifecycleSampleSummary,
        LifecycleValidationBatch, LifecycleValidationBatchInput, LifecycleValidationLifecycle,
        LifecycleValidationPeriod, MultiPeriodLifecycleValidationReport, OutlierSensitivity,
        OverlapStatus, PeriodContextSummary, PeriodDeteriorationDistribution,
        ValidationAuditSummary, VALIDATION_ENGINE_VERSION,
    },
};
use rusqlite::{params, Connection, OptionalExtension};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

const AGENT_ID: &str = "ai-intraday-v1";
const MAX_PERIODS: usize = 12;

#[derive(Debug, Clone)]
struct Candidate {
    experiment_id: String,
    experiment_name: String,
    dataset_id: String,
    dataset_name: String,
    asset: String,
    timeframe: String,
    start_at: String,
    end_at: String,
    candle_count: usize,
    session_count: usize,
    expected_gap_count: usize,
    unexpected_gap_count: usize,
    source_path: String,
    market: String,
    timezone: String,
    session_type: String,
    status: String,
    source_role: String,
    fee_pct: f64,
    slippage_pct: f64,
    risk_profile_id: String,
    execution_model_version: String,
    trigger_engine_version: Option<String>,
    agent_version: String,
    agent_config: Value,
}

fn new_batch_id() -> String {
    format!(
        "lifecycle-validation-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn now(connection: &Connection) -> Result<String, String> {
    connection
        .query_row("SELECT CURRENT_TIMESTAMP", [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn valid_role(role: &str) -> bool {
    matches!(
        role,
        "DEVELOPMENT" | "OOS" | "HOLDOUT" | "ADDITIONAL_VALIDATION"
    )
}

fn load_candidate(
    connection: &Connection,
    experiment_id: &str,
    source_role: &str,
) -> Result<Candidate, String> {
    if !valid_role(source_role) {
        return Err(format!("source_role inválido para {experiment_id}"));
    }
    connection
        .query_row(
            "SELECT e.id,e.name,e.dataset_id,d.name,e.asset,e.timeframe,d.start_at,d.end_at,
                    d.candle_count,d.session_count,d.expected_gap_count,d.unexpected_gap_count,
                    d.source_path,d.market,d.timezone,d.session_type,
                    e.status,e.fee_pct,e.slippage_pct,e.risk_profile_id,
                    e.execution_model_version,e.trigger_engine_version,a.strategy_version,ea.config_json
             FROM market_experiments e
             JOIN market_datasets d ON d.id=e.dataset_id
             JOIN market_experiment_agents ea ON ea.experiment_id=e.id AND ea.agent_id=?2
             JOIN market_agents a ON a.id=ea.agent_id
             WHERE e.id=?1",
            params![experiment_id, AGENT_ID],
            |row| {
                let config: String = row.get(23)?;
                Ok(Candidate {
                    experiment_id: row.get(0)?,
                    experiment_name: row.get(1)?,
                    dataset_id: row.get(2)?,
                    dataset_name: row.get(3)?,
                    asset: row.get(4)?,
                    timeframe: row.get(5)?,
                    start_at: row.get(6)?,
                    end_at: row.get(7)?,
                    candle_count: row.get::<_, i64>(8)?.max(0) as usize,
                    session_count: row.get::<_, i64>(9)?.max(0) as usize,
                    expected_gap_count: row.get::<_, i64>(10)?.max(0) as usize,
                    unexpected_gap_count: row.get::<_, i64>(11)?.max(0) as usize,
                    source_path: row.get(12)?,
                    market: row.get(13)?,
                    timezone: row.get(14)?,
                    session_type: row.get(15)?,
                    status: row.get(16)?,
                    fee_pct: row.get(17)?,
                    slippage_pct: row.get(18)?,
                    risk_profile_id: row.get(19)?,
                    execution_model_version: row.get(20)?,
                    trigger_engine_version: row.get(21)?,
                    agent_version: row.get(22)?,
                    agent_config: serde_json::from_str(&config).unwrap_or(Value::Null),
                    source_role: source_role.into(),
                })
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| {
            format!(
                "experimento {experiment_id} não encontrado ou não contém o agente {AGENT_ID}"
            )
        })
}

fn dataset_quality(
    connection: &Connection,
    candidate: &Candidate,
) -> Result<DatasetQualityReport, String> {
    let aggregate = connection
        .query_row(
            "SELECT COUNT(*),COALESCE(MIN(timestamp),''),COALESCE(MAX(timestamp),''),
                    COALESCE(SUM(CASE WHEN timestamp IS NULL OR open IS NULL OR high IS NULL OR low IS NULL OR close IS NULL OR volume IS NULL THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN open<=0 OR high<=0 OR low<=0 OR close<=0 THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN volume<0 THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN high<open OR high<close OR high<low THEN 1 ELSE 0 END),0),
                    COALESCE(SUM(CASE WHEN low>open OR low>close OR low>high THEN 1 ELSE 0 END),0)
             FROM market_candles WHERE dataset_id=?1",
            [&candidate.dataset_id],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?.max(0) as usize,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?.max(0) as usize,
                    row.get::<_, i64>(4)?.max(0) as usize,
                    row.get::<_, i64>(5)?.max(0) as usize,
                    row.get::<_, i64>(6)?.max(0) as usize,
                    row.get::<_, i64>(7)?.max(0) as usize,
                ))
            },
        )
        .map_err(|error| error.to_string())?;
    let duplicate_timestamp_count = connection
        .query_row(
            "SELECT COUNT(*) FROM (SELECT timestamp FROM market_candles WHERE dataset_id=?1 GROUP BY timestamp HAVING COUNT(*)>1)",
            [&candidate.dataset_id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(|error| error.to_string())?
        .max(0) as usize;
    let mut statement = connection
        .prepare("SELECT timestamp FROM market_candles WHERE dataset_id=?1 ORDER BY candle_index")
        .map_err(|error| error.to_string())?;
    let timestamps = statement
        .query_map([&candidate.dataset_id], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let out_of_order_count = timestamps
        .windows(2)
        .filter(|pair| pair[1] < pair[0])
        .count();

    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    if aggregate.0 == 0 {
        errors.push("dataset sem candles persistidos".into());
    }
    if aggregate.0 != candidate.candle_count {
        errors.push(format!(
            "candle_count divergente: metadata={} persistido={}",
            candidate.candle_count, aggregate.0
        ));
    }
    if aggregate.1 != candidate.start_at || aggregate.2 != candidate.end_at {
        errors.push("range persistido diverge da metadata do dataset".into());
    }
    for (count, label) in [
        (aggregate.3, "valores nulos"),
        (duplicate_timestamp_count, "timestamps duplicados"),
        (out_of_order_count, "timestamps fora de ordem"),
        (aggregate.4, "preços não positivos"),
        (aggregate.5, "volumes negativos"),
        (aggregate.6, "high inconsistente"),
        (aggregate.7, "low inconsistente"),
    ] {
        if count > 0 {
            errors.push(format!("{label}: {count}"));
        }
    }
    if candidate.market != "US_EQUITIES"
        || candidate.timezone != "America/New_York"
        || candidate.session_type != "REGULAR"
    {
        errors.push(format!(
            "sessão incompatível: market={} timezone={} session_type={}",
            candidate.market, candidate.timezone, candidate.session_type
        ));
    }
    if candidate.unexpected_gap_count > 0 {
        warnings.push(format!(
            "unexpected gaps persistidos: {}",
            candidate.unexpected_gap_count
        ));
    }
    let source_file_available = Path::new(&candidate.source_path).is_file();
    if !source_file_available {
        warnings.push(
            "SOURCE_FILE_UNAVAILABLE: quality gate executado sobre candles canônicos persistidos"
                .into(),
        );
    }
    let status = if !errors.is_empty() {
        "FAIL"
    } else if warnings.is_empty() {
        "PASS"
    } else {
        "PASS_WITH_WARNINGS"
    };
    Ok(DatasetQualityReport {
        dataset_id: candidate.dataset_id.clone(),
        dataset_name: candidate.dataset_name.clone(),
        status: status.into(),
        candle_count: aggregate.0,
        session_count: candidate.session_count,
        first_at: aggregate.1,
        last_at: aggregate.2,
        null_value_count: aggregate.3,
        duplicate_timestamp_count,
        out_of_order_count,
        nonpositive_price_count: aggregate.4,
        negative_volume_count: aggregate.5,
        invalid_high_count: aggregate.6,
        invalid_low_count: aggregate.7,
        expected_gap_count: candidate.expected_gap_count,
        unexpected_gap_count: candidate.unexpected_gap_count,
        market: candidate.market.clone(),
        timezone: candidate.timezone.clone(),
        session_type: candidate.session_type.clone(),
        source_file_available,
        warnings,
        errors,
    })
}

fn validate_candidates(candidates: &[Candidate]) -> Result<Vec<String>, String> {
    if candidates.len() < 2 {
        return Err("selecione ao menos dois experimentos independentes".into());
    }
    if candidates.len() > MAX_PERIODS {
        return Err(format!("o batch aceita no máximo {MAX_PERIODS} períodos"));
    }
    let first = &candidates[0];
    let mut seen = HashSet::new();
    for candidate in candidates {
        if !seen.insert(candidate.experiment_id.as_str()) {
            return Err("um experimento não pode aparecer duas vezes no mesmo batch".into());
        }
        if candidate.status != "completed" {
            return Err(format!(
                "o experimento {} ainda não está concluído",
                candidate.experiment_name
            ));
        }
        if candidate.asset != first.asset {
            return Err("os períodos devem usar o mesmo ativo".into());
        }
        if candidate.timeframe != first.timeframe {
            return Err("os períodos devem usar o mesmo timeframe".into());
        }
        if candidate.agent_version != first.agent_version
            || candidate.agent_config != first.agent_config
        {
            return Err("configuração ou versão do agente divergente entre períodos".into());
        }
        if candidate.risk_profile_id != first.risk_profile_id {
            return Err("perfil de risco divergente entre períodos".into());
        }
        if candidate.execution_model_version != first.execution_model_version
            || candidate.trigger_engine_version != first.trigger_engine_version
        {
            return Err("semântica de execução ou trigger divergente entre períodos".into());
        }
        if (candidate.fee_pct - first.fee_pct).abs() > f64::EPSILON
            || (candidate.slippage_pct - first.slippage_pct).abs() > f64::EPSILON
        {
            return Err("fees ou slippage divergentes entre períodos".into());
        }
    }

    let mut statuses = vec![OverlapStatus::NonOverlapping.as_str().to_string(); candidates.len()];
    for left in 0..candidates.len() {
        for right in (left + 1)..candidates.len() {
            let overlap = classify_overlap(
                &candidates[left].start_at,
                &candidates[left].end_at,
                &candidates[right].start_at,
                &candidates[right].end_at,
            );
            if overlap != OverlapStatus::NonOverlapping {
                statuses[left] = overlap.as_str().into();
                statuses[right] = overlap.as_str().into();
                return Err(format!(
                    "períodos {} e {} são incompatíveis: {}",
                    candidates[left].dataset_name,
                    candidates[right].dataset_name,
                    overlap.as_str()
                ));
            }
        }
    }
    Ok(statuses)
}

fn context_summary(
    connection: &Connection,
    candidate: &Candidate,
) -> Result<PeriodContextSummary, String> {
    let mut statement = connection
        .prepare("SELECT close FROM market_candles WHERE dataset_id=?1 ORDER BY candle_index")
        .map_err(|error| error.to_string())?;
    let closes = statement
        .query_map([&candidate.dataset_id], |row| row.get::<_, f64>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let returns = closes
        .windows(2)
        .filter_map(|pair| (pair[0] > 0.0).then_some((pair[1] / pair[0] - 1.0) * 100.0))
        .collect::<Vec<_>>();
    let period_return_pct = closes
        .first()
        .zip(closes.last())
        .filter(|(first, _)| **first > 0.0)
        .map(|(first, last)| (last / first - 1.0) * 100.0)
        .unwrap_or_default();
    let mean = average(&returns);
    let realized_volatility_pct = if returns.len() > 1 {
        round(
            (returns
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / (returns.len() - 1) as f64)
                .sqrt()
                * (returns.len() as f64).sqrt(),
        )
    } else {
        0.0
    };

    let mut feature_statement = connection
        .prepare(
            "SELECT features_json FROM market_intraday_feature_traces
             WHERE experiment_id=?1 ORDER BY candle_index",
        )
        .map_err(|error| error.to_string())?;
    let feature_json = feature_statement
        .query_map([&candidate.experiment_id], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut atr_percentages = Vec::new();
    let mut relative_volumes = Vec::new();
    for raw in feature_json {
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let (Some(atr), Some(close)) = (
            value.get("atr14").and_then(Value::as_f64),
            value.get("close").and_then(Value::as_f64),
        ) {
            if close > 0.0 {
                atr_percentages.push(atr / close * 100.0);
            }
        }
        if let Some(relative_volume) = value.get("relativeVolume").and_then(Value::as_f64) {
            relative_volumes.push(relative_volume);
        }
    }
    Ok(PeriodContextSummary {
        period_return_pct: round(period_return_pct),
        realized_volatility_pct,
        average_atr_pct: (!atr_percentages.is_empty()).then(|| average(&atr_percentages)),
        trend_proxy: if period_return_pct > 1.0 {
            "UP"
        } else if period_return_pct < -1.0 {
            "DOWN"
        } else {
            "SIDEWAYS"
        }
        .into(),
        average_relative_volume: (!relative_volumes.is_empty()).then(|| average(&relative_volumes)),
    })
}

fn component(trace: &LifecycleHealthTrace) -> ComponentSnapshot {
    ComponentSnapshot {
        trend: trace.trend_component,
        momentum: trace.momentum_component,
        giveback: trace.giveback_component,
        vwap: trace.vwap_component,
        volatility: trace.volatility_component,
        time: trace.time_component,
    }
}

fn component_average(traces: &[&LifecycleHealthTrace]) -> ComponentSnapshot {
    ComponentSnapshot {
        trend: average(
            &traces
                .iter()
                .map(|item| item.trend_component)
                .collect::<Vec<_>>(),
        ),
        momentum: average(
            &traces
                .iter()
                .map(|item| item.momentum_component)
                .collect::<Vec<_>>(),
        ),
        giveback: average(
            &traces
                .iter()
                .map(|item| item.giveback_component)
                .collect::<Vec<_>>(),
        ),
        vwap: average(
            &traces
                .iter()
                .map(|item| item.vwap_component)
                .collect::<Vec<_>>(),
        ),
        volatility: average(
            &traces
                .iter()
                .map(|item| item.volatility_component)
                .collect::<Vec<_>>(),
        ),
        time: average(
            &traces
                .iter()
                .map(|item| item.time_component)
                .collect::<Vec<_>>(),
        ),
    }
}

fn component_maximum(traces: &[&LifecycleHealthTrace]) -> ComponentSnapshot {
    let maximum = |extract: fn(&LifecycleHealthTrace) -> f64| {
        traces.iter().map(|item| extract(item)).fold(0.0, f64::max)
    };
    ComponentSnapshot {
        trend: round(maximum(|item| item.trend_component)),
        momentum: round(maximum(|item| item.momentum_component)),
        giveback: round(maximum(|item| item.giveback_component)),
        vwap: round(maximum(|item| item.vwap_component)),
        volatility: round(maximum(|item| item.volatility_component)),
        time: round(maximum(|item| item.time_component)),
    }
}

fn dominant_component(snapshot: &ComponentSnapshot) -> String {
    [
        ("TREND", snapshot.trend),
        ("MOMENTUM", snapshot.momentum),
        ("GIVEBACK", snapshot.giveback),
        ("VWAP", snapshot.vwap),
        ("VOLATILITY", snapshot.volatility),
        ("TIME", snapshot.time),
    ]
    .into_iter()
    .max_by(|left, right| left.1.total_cmp(&right.1).then(left.0.cmp(right.0)))
    .map(|item| item.0.to_string())
    .unwrap_or_else(|| "NONE".into())
}

fn health_rank(value: &str) -> usize {
    match value {
        "STRONG" => 0,
        "HEALTHY" => 1,
        "WEAKENING" => 2,
        "DETERIORATING" => 3,
        "CRITICAL" => 4,
        _ => 0,
    }
}

fn level_rank(value: &str) -> usize {
    match value {
        "NONE" => 0,
        "LOW" => 1,
        "MODERATE" => 2,
        "HIGH" => 3,
        "CRITICAL" => 4,
        _ => 0,
    }
}

fn response_delay(
    traces: &[&LifecycleHealthTrace],
    events: &[&market_lifecycle_repository::LifecycleEventView],
    minimum_level: usize,
) -> (bool, Option<usize>) {
    let Some(first) = traces
        .iter()
        .find(|trace| level_rank(&trace.deterioration_level) >= minimum_level)
    else {
        return (false, None);
    };
    let response = events.iter().find(|event| {
        event.candle_index >= first.candle_index
            && matches!(event.event_type.as_str(), "REDUCE" | "EXIT" | "FORCED_EXIT")
    });
    let delay = response.and_then(|event| {
        traces
            .iter()
            .find(|trace| trace.candle_index == event.candle_index)
            .map(|trace| {
                trace
                    .position_age_market_minutes
                    .saturating_sub(first.position_age_market_minutes)
            })
    });
    (true, delay)
}

fn health_distribution(traces: &[&LifecycleHealthTrace]) -> HealthDistribution {
    let denominator = traces.len().max(1) as f64;
    let pct = |label: &str| {
        round(
            traces
                .iter()
                .filter(|trace| trace.position_health == label)
                .count() as f64
                * 100.0
                / denominator,
        )
    };
    HealthDistribution {
        strong_pct: pct("STRONG"),
        healthy_pct: pct("HEALTHY"),
        weakening_pct: pct("WEAKENING"),
        deteriorating_pct: pct("DETERIORATING"),
        critical_pct: pct("CRITICAL"),
    }
}

fn lifecycle_rows(
    period_id: &str,
    source_role: &str,
    report: &LifecycleIntelligenceReport,
) -> Vec<LifecycleValidationLifecycle> {
    report
        .lifecycles
        .iter()
        .map(|lifecycle| {
            let traces = report
                .health_trace
                .iter()
                .filter(|trace| trace.lifecycle_id == lifecycle.lifecycle_id)
                .collect::<Vec<_>>();
            let events = report
                .events
                .iter()
                .filter(|event| event.lifecycle_id == lifecycle.lifecycle_id)
                .collect::<Vec<_>>();
            let late = report
                .outcomes
                .iter()
                .filter(|item| {
                    item.lifecycle_id == lifecycle.lifecycle_id && item.missed_reduction_window
                })
                .collect::<Vec<_>>();
            let maximum = traces.iter().copied().max_by(|left, right| {
                left.deterioration_score
                    .total_cmp(&right.deterioration_score)
                    .then(left.candle_index.cmp(&right.candle_index).reverse())
            });
            let dominant_health = {
                let mut counts = BTreeMap::<String, usize>::new();
                for trace in &traces {
                    *counts.entry(trace.position_health.clone()).or_default() += 1;
                }
                counts
                    .into_iter()
                    .max_by(|left, right| left.1.cmp(&right.1).then(left.0.cmp(&right.0).reverse()))
                    .map(|item| item.0)
                    .unwrap_or_else(|| "HEALTHY".into())
            };
            let worst_health = traces
                .iter()
                .max_by_key(|trace| health_rank(&trace.position_health))
                .map(|trace| trace.position_health.clone())
                .unwrap_or_else(|| "HEALTHY".into());
            let average_components = component_average(&traces);
            let maximum_components = component_maximum(&traces);
            let first_high = traces
                .iter()
                .find(|trace| level_rank(&trace.deterioration_level) >= 3)
                .map(|trace| component(trace));
            let first_critical = traces
                .iter()
                .find(|trace| level_rank(&trace.deterioration_level) >= 4)
                .map(|trace| component(trace));
            let (has_deterioration, response_from_first_deterioration_minutes) =
                response_delay(&traces, &events, 1);
            let (_, response_from_high_minutes) = response_delay(&traces, &events, 3);
            let (_, response_from_critical_minutes) = response_delay(&traces, &events, 4);
            let response_status = if !has_deterioration {
                "NOT_APPLICABLE"
            } else if response_from_first_deterioration_minutes.is_some() {
                "RESOLVED"
            } else {
                "UNRESOLVED"
            };
            LifecycleValidationLifecycle {
                period_id: period_id.into(),
                source_role: source_role.into(),
                experiment_id: report.experiment_id.clone(),
                lifecycle_id: lifecycle.lifecycle_id.clone(),
                entry_at: lifecycle.entry_execution_timestamp.clone(),
                exit_at: lifecycle.exit_execution_timestamp.clone(),
                status: lifecycle.status.clone(),
                duration_minutes: lifecycle.duration_market_minutes,
                pnl_pct: (lifecycle.status == "CLOSED").then_some(lifecycle.realized_pnl_pct),
                mfe_pct: lifecycle.mfe_pct,
                mae_pct: lifecycle.mae_pct,
                giveback_pct: lifecycle.giveback_pct,
                worst_health,
                dominant_health,
                health_distribution: health_distribution(&traces),
                max_deterioration_score: maximum
                    .map(|trace| trace.deterioration_score)
                    .unwrap_or_default(),
                max_deterioration_level: maximum
                    .map(|trace| trace.deterioration_level.clone())
                    .unwrap_or_else(|| "NONE".into()),
                first_timestamp_at_max_level: maximum.map(|trace| trace.timestamp.clone()),
                late_reduction_events: late.len(),
                has_late_reduction: !late.is_empty(),
                response_delay_minutes: response_from_first_deterioration_minutes,
                response_status: response_status.into(),
                response_from_first_deterioration_minutes,
                response_from_high_minutes,
                response_from_critical_minutes,
                overnight: lifecycle.overnight,
                censored_at_dataset_end: lifecycle.status == "OPEN",
                dominant_deterioration_component: dominant_component(&average_components),
                average_components,
                maximum_components,
                component_at_first_high: first_high,
                component_at_first_critical: first_critical,
            }
        })
        .collect()
}

fn period_summary(
    connection: &Connection,
    period_id: &str,
    candidate: &Candidate,
    overlap_status: &str,
    report: &LifecycleIntelligenceReport,
    rows: &[LifecycleValidationLifecycle],
) -> Result<LifecycleValidationPeriod, String> {
    let closed_pnl = rows
        .iter()
        .filter_map(|row| row.pnl_pct)
        .collect::<Vec<_>>();
    let response = rows
        .iter()
        .filter_map(|row| row.response_delay_minutes.map(|value| value as f64))
        .collect::<Vec<_>>();
    let eligible = report
        .outcomes
        .iter()
        .map(|item| item.lifecycle_id.as_str())
        .collect::<HashSet<_>>()
        .len();
    let late = report
        .outcomes
        .iter()
        .filter(|item| item.missed_reduction_window)
        .collect::<Vec<_>>();
    let late_lifecycles = rows.iter().filter(|row| row.has_late_reduction).count();
    let late_forward = late
        .iter()
        .filter_map(|item| item.forward_return_5)
        .collect::<Vec<_>>();
    let late_mae = late
        .iter()
        .filter_map(|item| item.mae_5)
        .collect::<Vec<_>>();
    Ok(LifecycleValidationPeriod {
        period_id: period_id.into(),
        experiment_id: candidate.experiment_id.clone(),
        experiment_name: candidate.experiment_name.clone(),
        dataset_id: candidate.dataset_id.clone(),
        dataset_name: candidate.dataset_name.clone(),
        source_role: candidate.source_role.clone(),
        start_at: candidate.start_at.clone(),
        end_at: candidate.end_at.clone(),
        candle_count: candidate.candle_count,
        session_count: candidate.session_count,
        lifecycle_count: rows.len(),
        lifecycle_run_id: report.run_id.clone(),
        closed_count: rows.iter().filter(|row| row.status == "CLOSED").count(),
        open_count: rows.iter().filter(|row| row.status == "OPEN").count(),
        duration: distribution(
            &rows
                .iter()
                .map(|row| row.duration_minutes as f64)
                .collect::<Vec<_>>(),
        ),
        pnl: distribution(&closed_pnl),
        mfe: distribution(&rows.iter().map(|row| row.mfe_pct).collect::<Vec<_>>()),
        mae: distribution(&rows.iter().map(|row| row.mae_pct).collect::<Vec<_>>()),
        giveback: distribution(&rows.iter().map(|row| row.giveback_pct).collect::<Vec<_>>()),
        late_reduction_events: late.len(),
        late_reduction_lifecycles: late_lifecycles,
        eligible_lifecycles: eligible,
        event_late_reduction_rate_pct: if report.late_reduction.total_hold_while_long == 0 {
            0.0
        } else {
            round(late.len() as f64 * 100.0 / report.late_reduction.total_hold_while_long as f64)
        },
        lifecycle_late_reduction_rate_pct: if eligible == 0 {
            0.0
        } else {
            round(late_lifecycles as f64 * 100.0 / eligible as f64)
        },
        average_forward_5: (!late_forward.is_empty()).then(|| average(&late_forward)),
        average_mae_5: (!late_mae.is_empty()).then(|| average(&late_mae)),
        response_delay: distribution(&response),
        unresolved_response_count: rows
            .iter()
            .filter(|row| row.response_status == "UNRESOLVED")
            .count(),
        overlap_status: overlap_status.into(),
        context: context_summary(connection, candidate)?,
    })
}

fn deterioration_distributions(
    period_id: &str,
    report: &LifecycleIntelligenceReport,
    rows: &[LifecycleValidationLifecycle],
) -> Vec<PeriodDeteriorationDistribution> {
    let total_events = report.health_trace.len().max(1);
    ["NONE", "LOW", "MODERATE", "HIGH", "CRITICAL"]
        .into_iter()
        .map(|level| {
            let event_count = report
                .health_trace
                .iter()
                .filter(|trace| trace.deterioration_level == level)
                .count();
            let threshold = level_rank(level);
            let lifecycle_count = rows
                .iter()
                .filter(|row| level_rank(&row.max_deterioration_level) >= threshold)
                .count();
            PeriodDeteriorationDistribution {
                period_id: period_id.into(),
                level: level.into(),
                event_count,
                event_pct: round(event_count as f64 * 100.0 / total_events as f64),
                lifecycle_count,
                lifecycle_reach_pct: if rows.is_empty() {
                    0.0
                } else {
                    round(lifecycle_count as f64 * 100.0 / rows.len() as f64)
                },
            }
        })
        .collect()
}

fn lifecycle_outcomes(
    rows: &[LifecycleValidationLifecycle],
    by_health: bool,
) -> Vec<LifecycleLevelOutcome> {
    let labels: &[&str] = if by_health {
        &[
            "STRONG",
            "HEALTHY",
            "WEAKENING",
            "DETERIORATING",
            "CRITICAL",
        ]
    } else {
        &["NONE", "LOW", "MODERATE", "HIGH", "CRITICAL"]
    };
    labels
        .iter()
        .filter_map(|label| {
            let matching = rows
                .iter()
                .filter(|row| {
                    if by_health {
                        row.worst_health == *label
                    } else {
                        row.max_deterioration_level == *label
                    }
                })
                .collect::<Vec<_>>();
            if matching.is_empty() {
                return None;
            }
            let pnl = matching
                .iter()
                .filter_map(|row| row.pnl_pct)
                .collect::<Vec<_>>();
            Some(LifecycleLevelOutcome {
                label: (*label).into(),
                lifecycle_count: matching.len(),
                average_pnl_pct: (!pnl.is_empty()).then(|| average(&pnl)),
                median_pnl_pct: (!pnl.is_empty()).then(|| percentile(&pnl, 0.5)),
                average_mfe_pct: average(
                    &matching.iter().map(|row| row.mfe_pct).collect::<Vec<_>>(),
                ),
                average_mae_pct: average(
                    &matching.iter().map(|row| row.mae_pct).collect::<Vec<_>>(),
                ),
                average_giveback_pct: average(
                    &matching
                        .iter()
                        .map(|row| row.giveback_pct)
                        .collect::<Vec<_>>(),
                ),
                late_reduction_rate_pct: round(
                    matching.iter().filter(|row| row.has_late_reduction).count() as f64 * 100.0
                        / matching.len() as f64,
                ),
            })
        })
        .collect()
}

fn component_value(snapshot: &ComponentSnapshot, label: &str) -> f64 {
    match label {
        "TREND" => snapshot.trend,
        "MOMENTUM" => snapshot.momentum,
        "GIVEBACK" => snapshot.giveback,
        "VWAP" => snapshot.vwap,
        "VOLATILITY" => snapshot.volatility,
        "TIME" => snapshot.time,
        _ => 0.0,
    }
}

fn component_outcomes(rows: &[LifecycleValidationLifecycle]) -> Vec<ComponentOutcomeSummary> {
    let period_ids = rows
        .iter()
        .map(|row| row.period_id.clone())
        .collect::<BTreeSet<_>>();
    let scopes = std::iter::once(None)
        .chain(period_ids.into_iter().map(Some))
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    for period_id in scopes {
        let selected = rows
            .iter()
            .filter(|row| period_id.as_ref().is_none_or(|id| &row.period_id == id))
            .collect::<Vec<_>>();
        for label in [
            "TREND",
            "MOMENTUM",
            "GIVEBACK",
            "VWAP",
            "VOLATILITY",
            "TIME",
        ] {
            let values = selected
                .iter()
                .map(|row| component_value(&row.average_components, label))
                .collect::<Vec<_>>();
            let maximum = selected
                .iter()
                .map(|row| component_value(&row.maximum_components, label))
                .fold(0.0, f64::max);
            let pnl = selected
                .iter()
                .filter_map(|row| row.pnl_pct)
                .collect::<Vec<_>>();
            result.push(ComponentOutcomeSummary {
                period_id: period_id.clone(),
                component: label.into(),
                lifecycle_count: selected.len(),
                average_value: average(&values),
                maximum_value: round(maximum),
                average_pnl_pct: (!pnl.is_empty()).then(|| average(&pnl)),
                average_mae_pct: average(
                    &selected.iter().map(|row| row.mae_pct).collect::<Vec<_>>(),
                ),
                average_giveback_pct: average(
                    &selected
                        .iter()
                        .map(|row| row.giveback_pct)
                        .collect::<Vec<_>>(),
                ),
                late_reduction_rate_pct: if selected.is_empty() {
                    0.0
                } else {
                    round(
                        selected.iter().filter(|row| row.has_late_reduction).count() as f64 * 100.0
                            / selected.len() as f64,
                    )
                },
            });
        }
    }
    result
}

fn cross_metric(label: &str, values: Vec<f64>) -> CrossPeriodMetric {
    let median = percentile(&values, 0.5);
    let minimum = values.iter().copied().fold(f64::INFINITY, f64::min);
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    CrossPeriodMetric {
        label: label.into(),
        values,
        median,
        minimum: if minimum.is_finite() {
            round(minimum)
        } else {
            0.0
        },
        maximum: if maximum.is_finite() {
            round(maximum)
        } else {
            0.0
        },
        dispersion: if minimum.is_finite() && maximum.is_finite() {
            round(maximum - minimum)
        } else {
            0.0
        },
    }
}

fn consistency(
    periods: &[LifecycleValidationPeriod],
    rows: &[LifecycleValidationLifecycle],
) -> Vec<CrossPeriodMetric> {
    let max_score_by_period = periods
        .iter()
        .map(|period| {
            percentile(
                &rows
                    .iter()
                    .filter(|row| row.period_id == period.period_id)
                    .map(|row| row.max_deterioration_score)
                    .collect::<Vec<_>>(),
                0.5,
            )
        })
        .collect::<Vec<_>>();
    let reach = |minimum: usize| {
        periods
            .iter()
            .map(|period| {
                let selected = rows
                    .iter()
                    .filter(|row| row.period_id == period.period_id)
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    0.0
                } else {
                    round(
                        selected
                            .iter()
                            .filter(|row| level_rank(&row.max_deterioration_level) >= minimum)
                            .count() as f64
                            * 100.0
                            / selected.len() as f64,
                    )
                }
            })
            .collect::<Vec<_>>()
    };
    vec![
        cross_metric(
            "LIFECYCLE LATE REDUCTION RATE",
            periods
                .iter()
                .map(|period| period.lifecycle_late_reduction_rate_pct)
                .collect(),
        ),
        cross_metric(
            "EVENT LATE REDUCTION RATE",
            periods
                .iter()
                .map(|period| period.event_late_reduction_rate_pct)
                .collect(),
        ),
        cross_metric(
            "MEDIAN RESPONSE DELAY",
            periods
                .iter()
                .map(|period| period.response_delay.median)
                .collect(),
        ),
        cross_metric(
            "MFE",
            periods.iter().map(|period| period.mfe.median).collect(),
        ),
        cross_metric(
            "MAE",
            periods.iter().map(|period| period.mae.median).collect(),
        ),
        cross_metric(
            "GIVEBACK",
            periods
                .iter()
                .map(|period| period.giveback.median)
                .collect(),
        ),
        cross_metric("MAX DETERIORATION SCORE", max_score_by_period),
        cross_metric("HIGH LIFECYCLE REACH RATE", reach(3)),
        cross_metric("CRITICAL LIFECYCLE REACH RATE", reach(4)),
    ]
}

fn outlier(metric: &str, values: Vec<(String, f64)>) -> OutlierSensitivity {
    if values.is_empty() {
        return OutlierSensitivity {
            metric: metric.into(),
            lifecycle_id: None,
            full_average: 0.0,
            without_largest_outlier_average: 0.0,
            delta: 0.0,
        };
    }
    let full = average(&values.iter().map(|item| item.1).collect::<Vec<_>>());
    let largest_index = values
        .iter()
        .enumerate()
        .max_by(|left, right| left.1 .1.abs().total_cmp(&right.1 .1.abs()))
        .map(|item| item.0)
        .unwrap_or_default();
    let without = values
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (index != largest_index).then_some(item.1))
        .collect::<Vec<_>>();
    let sensitivity = if without.is_empty() {
        full
    } else {
        average(&without)
    };
    OutlierSensitivity {
        metric: metric.into(),
        lifecycle_id: Some(values[largest_index].0.clone()),
        full_average: full,
        without_largest_outlier_average: sensitivity,
        delta: round(full - sensitivity),
    }
}

fn outlier_sensitivity(rows: &[LifecycleValidationLifecycle]) -> Vec<OutlierSensitivity> {
    let pairs = |extract: fn(&LifecycleValidationLifecycle) -> Option<f64>| {
        rows.iter()
            .filter_map(|row| extract(row).map(|value| (row.lifecycle_id.clone(), value)))
            .collect::<Vec<_>>()
    };
    vec![
        outlier("DURATION", pairs(|row| Some(row.duration_minutes as f64))),
        outlier("PNL", pairs(|row| row.pnl_pct)),
        outlier("MFE", pairs(|row| Some(row.mfe_pct))),
        outlier("MAE", pairs(|row| Some(row.mae_pct))),
        outlier("GIVEBACK", pairs(|row| Some(row.giveback_pct))),
    ]
}

fn evidence(
    periods: &[LifecycleValidationPeriod],
    rows: &[LifecycleValidationLifecycle],
    score_stability: &str,
) -> EvidenceMatrix {
    let health = lifecycle_outcomes(rows, true);
    let pnl_values = health
        .iter()
        .filter_map(|item| item.average_pnl_pct)
        .collect::<Vec<_>>();
    let health_relation = if rows.iter().filter(|row| row.pnl_pct.is_some()).count() < 4 {
        "NO_SIGNAL"
    } else if pnl_values.len() >= 2
        && pnl_values.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - pnl_values.iter().copied().fold(f64::INFINITY, f64::min)
            >= 1.0
    {
        "OBSERVABLE"
    } else {
        "MIXED"
    };
    let period_dominants = periods
        .iter()
        .map(|period| {
            let mut counts = BTreeMap::<String, usize>::new();
            for row in rows.iter().filter(|row| row.period_id == period.period_id) {
                *counts
                    .entry(row.dominant_deterioration_component.clone())
                    .or_default() += 1;
            }
            counts
                .into_iter()
                .max_by_key(|item| item.1)
                .map(|item| item.0)
        })
        .collect::<Vec<_>>();
    let component_consistency = if period_dominants.len() < 2 || period_dominants.contains(&None) {
        "INSUFFICIENT"
    } else if period_dominants.windows(2).all(|pair| pair[0] == pair[1]) {
        "CONSISTENT"
    } else {
        "MIXED"
    };
    let giveback_medians = periods
        .iter()
        .map(|period| period.giveback.median)
        .collect::<Vec<_>>();
    let giveback_consistency = if giveback_medians.len() < 2 {
        "INSUFFICIENT"
    } else {
        let spread = giveback_medians
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            - giveback_medians
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
        if spread <= 1.0 {
            "CONSISTENT"
        } else if spread <= 2.0 {
            "MIXED"
        } else {
            "INCONSISTENT"
        }
    };
    EvidenceMatrix {
        sample_size: sample_status(rows.len()),
        period_coverage: match periods.len() {
            0..=1 => "INSUFFICIENT",
            2..=5 => "PARTIAL",
            _ => "BROADER",
        }
        .into(),
        late_reduction: rate_consistency(
            &periods
                .iter()
                .map(|period| period.lifecycle_late_reduction_rate_pct)
                .collect::<Vec<_>>(),
        ),
        deterioration_stability: score_stability.into(),
        health_outcome_relation: health_relation.into(),
        component_consistency: component_consistency.into(),
        giveback_consistency: giveback_consistency.into(),
    }
}

fn persist_report(
    connection: &mut Connection,
    report: &MultiPeriodLifecycleValidationReport,
    input: &LifecycleValidationBatchInput,
) -> Result<(), String> {
    let tx = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "DELETE FROM market_lifecycle_validation_lifecycles WHERE batch_id=?1",
        [&report.batch.batch_id],
    )
    .map_err(|error| error.to_string())?;
    for period in &report.periods {
        tx.execute(
            "INSERT INTO market_lifecycle_validation_periods(id,batch_id,experiment_id,dataset_id,source_role,asset,timeframe,start_at,end_at,candle_count,session_count,lifecycle_count,lifecycle_run_id,overlap_status,summary_json)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
             ON CONFLICT(id) DO UPDATE SET
               lifecycle_count=excluded.lifecycle_count,
               lifecycle_run_id=excluded.lifecycle_run_id,
               overlap_status=excluded.overlap_status,
               summary_json=excluded.summary_json,
               period_status='COMPLETED',
               completed_at=COALESCE(completed_at,CURRENT_TIMESTAMP),
               error=NULL",
            params![period.period_id,report.batch.batch_id,period.experiment_id,period.dataset_id,period.source_role,report.batch.asset,report.batch.timeframe,period.start_at,period.end_at,period.candle_count,period.session_count,period.lifecycle_count,period.lifecycle_run_id,period.overlap_status,serde_json::to_string(period).map_err(|error|error.to_string())?],
        ).map_err(|error|error.to_string())?;
        for row in report
            .lifecycles
            .iter()
            .filter(|row| row.period_id == period.period_id)
        {
            tx.execute(
                "INSERT INTO market_lifecycle_validation_lifecycles(batch_id,period_id,lifecycle_id,status,entry_at,exit_at,duration_minutes,realized_pnl_pct,mfe_pct,mae_pct,giveback_pct,worst_health,dominant_health,max_deterioration_score,max_deterioration_level,first_timestamp_at_max_level,late_reduction_events,has_late_reduction,response_delay_minutes,response_status,response_from_first_deterioration_minutes,response_from_high_minutes,response_from_critical_minutes,overnight,censored_at_dataset_end,health_distribution_json,components_json)
                 VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27)",
                params![report.batch.batch_id,row.period_id,row.lifecycle_id,row.status,row.entry_at,row.exit_at,row.duration_minutes,row.pnl_pct,row.mfe_pct,row.mae_pct,row.giveback_pct,row.worst_health,row.dominant_health,row.max_deterioration_score,row.max_deterioration_level,row.first_timestamp_at_max_level,row.late_reduction_events,row.has_late_reduction,row.response_delay_minutes,row.response_status,row.response_from_first_deterioration_minutes,row.response_from_high_minutes,row.response_from_critical_minutes,row.overnight,row.censored_at_dataset_end,serde_json::to_string(&row.health_distribution).map_err(|error|error.to_string())?,serde_json::to_string(&json!({"average":row.average_components,"maximum":row.maximum_components,"firstHigh":row.component_at_first_high,"firstCritical":row.component_at_first_critical,"dominant":row.dominant_deterioration_component})).map_err(|error|error.to_string())?],
            ).map_err(|error|error.to_string())?;
        }
    }
    let result_json = serde_json::to_string(report).map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE market_lifecycle_validation_batches
         SET dataset_count=?2,experiment_count=?3,lifecycle_count=?4,status=?5,
             lifecycle_config_version=?6,deterioration_config_version=?7,
             config_json=?8,result_json=?9,warnings_json=?10,evidence_matrix_json=?11,
             reused_experiments=?12,rebuilt_artifacts=?13,new_llm_runs=?14,
             failed_periods=?15,skipped_periods=?16,duration_ms=?17,audit_json=?18,
             error=NULL,completed_at=?19
         WHERE id=?1",
        params![
            report.batch.batch_id,
            report.batch.dataset_count,
            report.batch.experiment_count,
            report.batch.lifecycle_count,
            report.batch.status,
            report.batch.lifecycle_config_version,
            report.batch.deterioration_config_version,
            serde_json::to_string(input).map_err(|error| error.to_string())?,
            result_json,
            serde_json::to_string(&report.warnings).map_err(|error| error.to_string())?,
            serde_json::to_string(&report.evidence).map_err(|error| error.to_string())?,
            report.audit.reused_experiments,
            report.audit.rebuilt_artifacts,
            report.audit.new_llm_runs,
            report.audit.failed_periods,
            report.audit.skipped_periods,
            report.audit.duration_ms,
            serde_json::to_string(&report.audit).map_err(|error| error.to_string())?,
            report.batch.completed_at
        ],
    )
    .map_err(|error| error.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

fn build_report(
    connection: &mut Connection,
    batch_id: &str,
    name: &str,
    created_at: &str,
    candidates: &[Candidate],
    period_ids: &[String],
    overlap_statuses: &[String],
    quality: Vec<DatasetQualityReport>,
    audit: ValidationAuditSummary,
    warnings: &mut Vec<String>,
) -> Result<MultiPeriodLifecycleValidationReport, String> {
    let mut reports = Vec::new();
    for candidate in candidates {
        let report = match market_lifecycle_repository::get(connection, &candidate.experiment_id) {
            Ok(report) => report,
            Err(_) => {
                warnings.push(format!(
                    "REBUILD MISSING ARTIFACTS: {} reconstruído localmente sem Ollama",
                    candidate.experiment_name
                ));
                market_lifecycle_repository::generate(connection, &candidate.experiment_id)?
            }
        };
        reports.push(report);
    }
    let first_report = reports
        .first()
        .ok_or_else(|| "nenhum relatório lifecycle disponível".to_string())?;
    for report in &reports {
        let report_config =
            serde_json::to_value(&report.config).map_err(|error| error.to_string())?;
        let first_config =
            serde_json::to_value(&first_report.config).map_err(|error| error.to_string())?;
        if report.lifecycle_engine_version != first_report.lifecycle_engine_version
            || report.deterioration_engine_version != first_report.deterioration_engine_version
            || report_config != first_config
        {
            return Err("configuração lifecycle/deterioration divergente entre períodos".into());
        }
    }

    let mut all_rows = Vec::new();
    let mut periods = Vec::new();
    let mut deterioration_by_period = Vec::new();
    for (((candidate, report), overlap_status), period_id) in candidates
        .iter()
        .zip(reports.iter())
        .zip(overlap_statuses.iter())
        .zip(period_ids.iter())
    {
        let rows = lifecycle_rows(period_id, &candidate.source_role, report);
        deterioration_by_period.extend(deterioration_distributions(period_id, report, &rows));
        periods.push(period_summary(
            connection,
            period_id,
            candidate,
            overlap_status,
            report,
            &rows,
        )?);
        all_rows.extend(rows);
    }

    let score_medians = periods
        .iter()
        .map(|period| {
            percentile(
                &all_rows
                    .iter()
                    .filter(|row| row.period_id == period.period_id)
                    .map(|row| row.max_deterioration_score)
                    .collect::<Vec<_>>(),
                0.5,
            )
        })
        .collect::<Vec<_>>();
    let score_stability = stability_status(&score_medians, all_rows.len());
    if let Some(warning) = concentration_warning(
        &periods
            .iter()
            .map(|period| period.lifecycle_count)
            .collect::<Vec<_>>(),
    ) {
        warnings.push(warning);
    }
    if all_rows.iter().any(|row| row.censored_at_dataset_end) {
        warnings.push(
            "OPEN_AT_END / CENSORED_AT_DATASET_END presente; PnL realizado usa somente CLOSED"
                .into(),
        );
    }
    if all_rows.len() < 20 {
        warnings.push("INSUFFICIENT SAMPLE: menos de 20 lifecycles independentes".into());
    }
    if periods.len() < 6 {
        warnings.push(format!(
            "INSUFFICIENT_PERIOD_COVERAGE: {} de 6 períodos desejados",
            periods.len()
        ));
    }

    let completed_at = now(connection)?;
    let lifecycle_late_count = all_rows.iter().filter(|row| row.has_late_reduction).count();
    let eligible = periods
        .iter()
        .map(|period| period.eligible_lifecycles)
        .sum::<usize>();
    let late_events = periods
        .iter()
        .map(|period| period.late_reduction_events)
        .sum::<usize>();
    let hold_events = reports
        .iter()
        .map(|report| report.late_reduction.total_hold_while_long)
        .sum::<usize>();
    let delays = all_rows
        .iter()
        .filter_map(|row| row.response_delay_minutes.map(|value| value as f64))
        .collect::<Vec<_>>();
    let evidence = evidence(&periods, &all_rows, &score_stability);
    let batch = LifecycleValidationBatch {
        batch_id: batch_id.into(),
        name: name.into(),
        status: "COMPLETED".into(),
        asset: candidates[0].asset.clone(),
        timeframe: candidates[0].timeframe.clone(),
        agent_id: AGENT_ID.into(),
        agent_version: candidates[0].agent_version.clone(),
        lifecycle_config_version: first_report.lifecycle_engine_version.clone(),
        deterioration_config_version: first_report.deterioration_engine_version.clone(),
        hold_diagnostics_version: market_hold_diagnostics::ENGINE_VERSION.into(),
        execution_model_version: candidates[0].execution_model_version.clone(),
        validation_engine_version: VALIDATION_ENGINE_VERSION.into(),
        dataset_count: candidates
            .iter()
            .map(|candidate| candidate.dataset_id.as_str())
            .collect::<HashSet<_>>()
            .len(),
        experiment_count: candidates.len(),
        lifecycle_count: all_rows.len(),
        created_at: created_at.into(),
        started_at: Some(created_at.into()),
        completed_at: Some(completed_at),
    };
    let sample = LifecycleSampleSummary {
        datasets: batch.dataset_count,
        periods: periods.len(),
        experiments: candidates.len(),
        lifecycles: all_rows.len(),
        closed: all_rows.iter().filter(|row| row.status == "CLOSED").count(),
        open: all_rows.iter().filter(|row| row.status == "OPEN").count(),
        censored: all_rows
            .iter()
            .filter(|row| row.censored_at_dataset_end)
            .count(),
        date_coverage_start: candidates
            .iter()
            .map(|candidate| candidate.start_at.as_str())
            .min()
            .unwrap_or_default()
            .into(),
        date_coverage_end: candidates
            .iter()
            .map(|candidate| candidate.end_at.as_str())
            .max()
            .unwrap_or_default()
            .into(),
        lifecycle_late_reduction_rate_pct: if eligible == 0 {
            0.0
        } else {
            round(lifecycle_late_count as f64 * 100.0 / eligible as f64)
        },
        event_late_reduction_rate_pct: if hold_events == 0 {
            0.0
        } else {
            round(late_events as f64 * 100.0 / hold_events as f64)
        },
        median_response_delay_minutes: (!delays.is_empty()).then(|| percentile(&delays, 0.5)),
        score_stability: score_stability.clone(),
        sample_status: sample_status(all_rows.len()),
    };
    let cross_period_consistency = consistency(&periods, &all_rows);
    Ok(MultiPeriodLifecycleValidationReport {
        batch,
        sample,
        periods,
        deterioration_by_period,
        deterioration_outcomes: lifecycle_outcomes(&all_rows, false),
        health_outcomes: lifecycle_outcomes(&all_rows, true),
        components: component_outcomes(&all_rows),
        consistency: cross_period_consistency,
        evidence,
        outlier_sensitivity: outlier_sensitivity(&all_rows),
        quality,
        audit,
        warnings: warnings.clone(),
        lifecycles: all_rows,
        post_decision_only: true,
    })
}

fn insert_period_checkpoints(
    connection: &Connection,
    batch_id: &str,
    candidates: &[Candidate],
    overlap_statuses: &[String],
) -> Result<Vec<String>, String> {
    let mut period_ids = Vec::new();
    for (index, (candidate, overlap_status)) in
        candidates.iter().zip(overlap_statuses.iter()).enumerate()
    {
        let period_id = format!("{batch_id}-period-{:02}", index + 1);
        connection.execute(
            "INSERT OR IGNORE INTO market_lifecycle_validation_periods(id,batch_id,experiment_id,dataset_id,source_role,asset,timeframe,start_at,end_at,candle_count,session_count,lifecycle_count,lifecycle_run_id,overlap_status,summary_json,period_status,quality_status,quality_json,artifact_mode)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,0,NULL,?12,'{}','PENDING','PENDING','{}','PENDING')",
            params![period_id,batch_id,candidate.experiment_id,candidate.dataset_id,candidate.source_role,candidate.asset,candidate.timeframe,candidate.start_at,candidate.end_at,candidate.candle_count,candidate.session_count,overlap_status],
        ).map_err(|error| error.to_string())?;
        period_ids.push(period_id);
    }
    Ok(period_ids)
}

fn checkpoint_period(
    connection: &Connection,
    period_id: &str,
    period_status: &str,
    quality: &DatasetQualityReport,
    artifact_mode: &str,
    error: Option<&str>,
) -> Result<(), String> {
    connection.execute(
        "UPDATE market_lifecycle_validation_periods
         SET period_status=?2,quality_status=?3,quality_json=?4,artifact_mode=?5,error=?6,
             started_at=COALESCE(started_at,CURRENT_TIMESTAMP),
             completed_at=CASE WHEN ?2 IN ('COMPLETED','FAILED','SKIPPED') THEN CURRENT_TIMESTAMP ELSE completed_at END
         WHERE id=?1",
        params![period_id,period_status,quality.status,serde_json::to_string(quality).map_err(|error|error.to_string())?,artifact_mode,error],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

fn audit_from_candidate(
    candidate: &Candidate,
    reused_experiments: usize,
    rebuilt_artifacts: usize,
    failed_periods: usize,
    duration_ms: usize,
) -> ValidationAuditSummary {
    let value = |key: &str| {
        candidate
            .agent_config
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN")
            .to_string()
    };
    ValidationAuditSummary {
        reused_experiments,
        rebuilt_artifacts,
        new_llm_runs: 0,
        failed_periods,
        skipped_periods: 0,
        duration_ms,
        model: value("model"),
        prompt_version: value("promptVersion"),
        context_version: value("contextVersion"),
        trigger_version: value("triggerVersion"),
        position_sizing_version: value("positionSizingVersion"),
        risk_policy_version: value("riskPolicyVersion"),
        execution_model_version: value("executionModelVersion"),
        fee_pct: candidate.fee_pct,
        slippage_pct: candidate.slippage_pct,
    }
}

fn execute_batch(
    connection: &mut Connection,
    batch_id: &str,
    name: &str,
    created_at: &str,
    input: &LifecycleValidationBatchInput,
    candidates: &[Candidate],
    overlap_statuses: &[String],
) -> Result<MultiPeriodLifecycleValidationReport, String> {
    let timer = Instant::now();
    let period_ids = insert_period_checkpoints(connection, batch_id, candidates, overlap_statuses)?;
    let mut warnings = Vec::new();
    let mut quality = Vec::new();
    let mut successful_candidates = Vec::new();
    let mut successful_period_ids = Vec::new();
    let mut successful_overlap_statuses = Vec::new();
    let mut reused_experiments = 0usize;
    let mut rebuilt_artifacts = 0usize;
    let mut failed_periods = 0usize;
    for ((candidate, period_id), overlap_status) in candidates
        .iter()
        .zip(period_ids.iter())
        .zip(overlap_statuses.iter())
    {
        let existing = connection
            .query_row(
                "SELECT period_status,quality_json FROM market_lifecycle_validation_periods WHERE id=?1",
                [period_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if let Some((status, raw_quality)) = existing {
            if status == "COMPLETED" {
                let quality_report = serde_json::from_str::<DatasetQualityReport>(&raw_quality)
                    .unwrap_or(dataset_quality(connection, candidate)?);
                for warning in &quality_report.warnings {
                    warnings.push(format!("{}: {warning}", candidate.dataset_name));
                }
                quality.push(quality_report);
                successful_candidates.push(candidate.clone());
                successful_period_ids.push(period_id.clone());
                successful_overlap_statuses.push(overlap_status.clone());
                reused_experiments += 1;
                continue;
            }
        }
        connection.execute(
            "UPDATE market_lifecycle_validation_periods SET period_status='RUNNING',started_at=COALESCE(started_at,CURRENT_TIMESTAMP) WHERE id=?1",
            [period_id],
        ).map_err(|error|error.to_string())?;
        let quality_report = dataset_quality(connection, candidate)?;
        for warning in &quality_report.warnings {
            warnings.push(format!("{}: {warning}", candidate.dataset_name));
        }
        if quality_report.status == "FAIL" {
            let error = quality_report.errors.join("; ");
            checkpoint_period(
                connection,
                period_id,
                "FAILED",
                &quality_report,
                "PENDING",
                Some(&error),
            )?;
            warnings.push(format!("FAILED PERIOD {}: {error}", candidate.dataset_name));
            failed_periods += 1;
            quality.push(quality_report);
            continue;
        }
        let artifact_mode =
            match market_lifecycle_repository::get(connection, &candidate.experiment_id) {
                Ok(_) => {
                    reused_experiments += 1;
                    "REUSED"
                }
                Err(_) => match market_lifecycle_repository::generate(
                    connection,
                    &candidate.experiment_id,
                ) {
                    Ok(_) => {
                        rebuilt_artifacts += 1;
                        warnings.push(format!(
                            "REBUILD MISSING ARTIFACTS: {} reconstruído localmente sem Ollama",
                            candidate.experiment_name
                        ));
                        "REBUILT"
                    }
                    Err(error) => {
                        checkpoint_period(
                            connection,
                            period_id,
                            "FAILED",
                            &quality_report,
                            "PENDING",
                            Some(&error),
                        )?;
                        warnings.push(format!("FAILED PERIOD {}: {error}", candidate.dataset_name));
                        failed_periods += 1;
                        quality.push(quality_report);
                        continue;
                    }
                },
            };
        checkpoint_period(
            connection,
            period_id,
            "COMPLETED",
            &quality_report,
            artifact_mode,
            None,
        )?;
        quality.push(quality_report);
        successful_candidates.push(candidate.clone());
        successful_period_ids.push(period_id.clone());
        successful_overlap_statuses.push(overlap_status.clone());
    }
    if successful_candidates.is_empty() {
        let error = "nenhum período passou pelo quality gate e pela preparação de artefatos";
        connection.execute(
            "UPDATE market_lifecycle_validation_batches SET status='FAILED',failed_periods=?2,error=?3,completed_at=CURRENT_TIMESTAMP WHERE id=?1",
            params![batch_id,failed_periods,error],
        ).map_err(|database_error|database_error.to_string())?;
        return Err(error.into());
    }
    let audit = audit_from_candidate(
        &candidates[0],
        reused_experiments,
        rebuilt_artifacts,
        failed_periods,
        timer.elapsed().as_millis() as usize,
    );
    match build_report(
        connection,
        batch_id,
        name,
        &created_at,
        &successful_candidates,
        &successful_period_ids,
        &successful_overlap_statuses,
        quality,
        audit,
        &mut warnings,
    ) {
        Ok(mut report) => {
            report.audit.duration_ms = timer.elapsed().as_millis() as usize;
            if failed_periods > 0 {
                report.batch.status = "PARTIAL".into();
            }
            persist_report(connection, &report, input)?;
            Ok(report)
        }
        Err(error) => {
            let _ = connection.execute(
                "UPDATE market_lifecycle_validation_batches SET status='FAILED',error=?2,completed_at=CURRENT_TIMESTAMP WHERE id=?1",
                params![batch_id,error],
            );
            Err(error)
        }
    }
}

pub fn create(
    connection: &mut Connection,
    input: &LifecycleValidationBatchInput,
) -> Result<MultiPeriodLifecycleValidationReport, String> {
    let name = input.name.trim();
    if name.len() < 3 || name.len() > 120 {
        return Err("o nome do batch deve ter entre 3 e 120 caracteres".into());
    }
    let candidates = input
        .periods
        .iter()
        .map(|period| load_candidate(connection, &period.experiment_id, &period.source_role))
        .collect::<Result<Vec<_>, _>>()?;
    let overlap_statuses = validate_candidates(&candidates)?;
    let batch_id = new_batch_id();
    let created_at = now(connection)?;
    connection.execute(
        "INSERT INTO market_lifecycle_validation_batches(id,name,asset,timeframe,agent_id,agent_version,lifecycle_config_version,deterioration_config_version,hold_diagnostics_version,execution_model_version,status,config_json,created_at,started_at)
         VALUES(?1,?2,?3,?4,?5,?6,'PENDING','PENDING',?7,?8,'VALIDATING',?9,?10,?10)",
        params![batch_id,name,candidates[0].asset,candidates[0].timeframe,AGENT_ID,candidates[0].agent_version,market_hold_diagnostics::ENGINE_VERSION,candidates[0].execution_model_version,serde_json::to_string(input).map_err(|error|error.to_string())?,created_at],
    ).map_err(|error|error.to_string())?;
    execute_batch(
        connection,
        &batch_id,
        name,
        &created_at,
        input,
        &candidates,
        &overlap_statuses,
    )
}

pub fn resume(
    connection: &mut Connection,
    batch_id: &str,
) -> Result<MultiPeriodLifecycleValidationReport, String> {
    let row = connection
        .query_row(
            "SELECT name,status,config_json,created_at FROM market_lifecycle_validation_batches WHERE id=?1",
            [batch_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "batch não encontrado".to_string())?;
    if row.1 == "COMPLETED" {
        return get(connection, batch_id);
    }
    let input = serde_json::from_str::<LifecycleValidationBatchInput>(&row.2)
        .map_err(|error| format!("configuração persistida inválida: {error}"))?;
    let candidates = input
        .periods
        .iter()
        .map(|period| load_candidate(connection, &period.experiment_id, &period.source_role))
        .collect::<Result<Vec<_>, _>>()?;
    let overlap_statuses = validate_candidates(&candidates)?;
    connection.execute(
        "UPDATE market_lifecycle_validation_batches SET status='VALIDATING',error=NULL,completed_at=NULL WHERE id=?1",
        [batch_id],
    ).map_err(|error|error.to_string())?;
    execute_batch(
        connection,
        batch_id,
        &row.0,
        &row.3,
        &input,
        &candidates,
        &overlap_statuses,
    )
}

pub fn get(
    connection: &Connection,
    batch_id: &str,
) -> Result<MultiPeriodLifecycleValidationReport, String> {
    let raw = connection
        .query_row(
            "SELECT result_json FROM market_lifecycle_validation_batches WHERE id=?1 AND status IN ('COMPLETED','PARTIAL')",
            [batch_id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .flatten()
        .ok_or_else(|| "batch concluído ou parcial não encontrado".to_string())?;
    serde_json::from_str(&raw).map_err(|error| error.to_string())
}

pub fn list(connection: &Connection) -> Result<Vec<LifecycleValidationBatch>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id,name,status,asset,timeframe,agent_id,agent_version,lifecycle_config_version,
                    deterioration_config_version,hold_diagnostics_version,execution_model_version,
                    dataset_count,experiment_count,lifecycle_count,created_at,started_at,completed_at
             FROM market_lifecycle_validation_batches ORDER BY created_at DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(LifecycleValidationBatch {
                batch_id: row.get(0)?,
                name: row.get(1)?,
                status: row.get(2)?,
                asset: row.get(3)?,
                timeframe: row.get(4)?,
                agent_id: row.get(5)?,
                agent_version: row.get(6)?,
                lifecycle_config_version: row.get(7)?,
                deterioration_config_version: row.get(8)?,
                hold_diagnostics_version: row.get(9)?,
                execution_model_version: row.get(10)?,
                validation_engine_version: VALIDATION_ENGINE_VERSION.into(),
                dataset_count: row.get::<_, i64>(11)?.max(0) as usize,
                experiment_count: row.get::<_, i64>(12)?.max(0) as usize,
                lifecycle_count: row.get::<_, i64>(13)?.max(0) as usize,
                created_at: row.get(14)?,
                started_at: row.get(15)?,
                completed_at: row.get(16)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn export_json(connection: &Connection, batch_id: &str) -> Result<String, String> {
    serde_json::to_string_pretty(&get(connection, batch_id)?).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn candidate(id: &str, asset: &str, timeframe: &str, start: &str, end: &str) -> Candidate {
        Candidate {
            experiment_id: id.into(),
            experiment_name: format!("Experiment {id}"),
            dataset_id: format!("dataset-{id}"),
            dataset_name: format!("Dataset with a different name {id}"),
            asset: asset.into(),
            timeframe: timeframe.into(),
            start_at: start.into(),
            end_at: end.into(),
            candle_count: 520,
            session_count: 20,
            expected_gap_count: 19,
            unexpected_gap_count: 0,
            source_path: format!("{id}.csv"),
            market: "US_EQUITIES".into(),
            timezone: "America/New_York".into(),
            session_type: "REGULAR".into(),
            status: "completed".into(),
            source_role: if id == "dev" { "DEVELOPMENT" } else { "OOS" }.into(),
            fee_pct: 0.1,
            slippage_pct: 0.05,
            risk_profile_id: "balanced-v1".into(),
            execution_model_version: "EXECUTION_MODEL_V1".into(),
            trigger_engine_version: Some("MARKET_DECISION_TRIGGER_V2".into()),
            agent_version: "MARKET_AI_INTRADAY_V1".into(),
            agent_config: json!({"seed":42,"temperature":0.1}),
        }
    }

    fn insert_persisted_lifecycle_fixture(
        connection: &Connection,
        suffix: &str,
        start: &str,
        end: &str,
    ) {
        let dataset_id = format!("dataset-{suffix}");
        let experiment_id = format!("experiment-{suffix}");
        let lifecycle_id = format!("lifecycle-{suffix}");
        let run_id = format!("run-{suffix}");
        connection.execute(
            "INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path,session_count,market,timezone,session_type,expected_gap_count,unexpected_gap_count) VALUES(?1,?2,'AAPL','15M',?3,?4,2,?5,?6,1,'US_EQUITIES','America/New_York','REGULAR',0,0)",
            params![dataset_id,format!("Dataset {suffix}"),start,end,format!("fingerprint-{suffix}"),format!("{suffix}.csv")],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_candles(dataset_id,candle_index,timestamp,open,high,low,close,volume) VALUES(?1,0,?2,100,101,99,100,1000),(?1,1,?3,100,103,99,102,1100)",
            params![dataset_id,start,end],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_experiments(id,name,dataset_id,asset,timeframe,currency,initial_capital,risk_profile_id,random_seed,fee_pct,slippage_pct,config_json,status,started_at,completed_at) VALUES(?1,?2,?3,'AAPL','15M','USD',10000,'balanced-v1',42,0.1,0.05,'{}','completed',?4,?5)",
            params![experiment_id,format!("Experiment {suffix}"),dataset_id,start,end],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_experiment_agents(experiment_id,agent_id,config_json,status) VALUES(?1,?2,'{\"frozen\":true}','completed')",
            params![experiment_id,AGENT_ID],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_trade_lifecycles(id,experiment_id,agent_id,asset,lifecycle_index,opened_at,closed_at,entry_price,average_entry_price,exit_price,initial_exposure_pct,max_exposure_pct,holding_candles,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,exit_efficiency_pct,profit_giveback_pct,entry_reason_code,exit_reason_code,status,reentry,engine_version,holding_market_minutes,overnight) VALUES(?1,?2,?3,'AAPL',1,?4,?5,100,100,102,25,25,2,50,2,3,-1,66.67,1,'ENTRY','EXIT','CLOSED',0,'POSITION_ENGINE_V1',15,0)",
            params![lifecycle_id,experiment_id,AGENT_ID,start,end],
        ).unwrap();
        let config = serde_json::to_string(
            &crate::database::market_lifecycle_intelligence::PositionDeteriorationConfig::default(),
        )
        .unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_analysis_runs(id,experiment_id,agent_id,lifecycle_engine_version,deterioration_engine_version,deterioration_config_json,status,total_lifecycles,late_reduction_events,completed_at) VALUES(?1,?2,?3,'POSITION_LIFECYCLE_CONFIG_V1','POSITION_DETERIORATION_CONFIG_V1',?4,'completed',1,1,CURRENT_TIMESTAMP)",
            params![run_id,experiment_id,AGENT_ID,config],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_analyses(lifecycle_id,run_id,asset,timeframe,entry_execution_timestamp,entry_price,initial_exposure_pct,max_exposure_pct,exit_execution_timestamp,exit_price,duration_candles,duration_market_minutes,overnight,status,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,giveback_pct,response_censored) VALUES(?1,?2,'AAPL','15M',?3,100,25,25,?4,102,2,15,0,'CLOSED',50,2,3,-1,1,0)",
            params![lifecycle_id,run_id,start,end],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_health_trace(run_id,lifecycle_id,candle_index,timestamp,position_age_candles,position_age_market_minutes,market_price,exposure_pct,return_since_entry_pct,mfe_since_entry_pct,mae_since_entry_pct,distance_from_mfe_pct,giveback_from_mfe_pct,trend_changed,momentum_changed,trend_component,momentum_component,giveback_component,vwap_component,volatility_component,time_component,deterioration_score,deterioration_level,position_health,available_at_t) VALUES(?1,?2,0,?3,1,15,101,25,1,2,-1,1,1,0,0,0.1,0.2,0.4,0.1,0.1,0.1,0.6,'HIGH','DETERIORATING',1)",
            params![run_id,lifecycle_id,start],
        ).unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_post_decision_outcomes(run_id,lifecycle_id,candle_index,hold_quality,forward_return_1,forward_return_5,mfe_5,mae_5,missed_reduction_window,post_decision_only) VALUES(?1,?2,0,'POTENTIAL_LATE_REDUCTION',0.1,-0.5,0.2,-1.0,1,1)",
            params![run_id,lifecycle_id],
        ).unwrap();
    }

    #[test]
    fn evidence_never_creates_a_single_score() {
        let matrix = evidence(&[], &[], "INSUFFICIENT_SAMPLE");
        assert_eq!(matrix.sample_size, "INSUFFICIENT");
        assert_eq!(matrix.deterioration_stability, "INSUFFICIENT_SAMPLE");
    }

    #[test]
    fn different_dataset_names_with_same_market_identity_are_accepted() {
        let candidates = vec![
            candidate("dev", "AAPL", "15M", "2026-01-01", "2026-01-31"),
            candidate("oos", "AAPL", "15M", "2026-02-01", "2026-02-28"),
        ];
        assert_eq!(
            validate_candidates(&candidates).unwrap(),
            vec!["NON_OVERLAPPING", "NON_OVERLAPPING"]
        );
    }

    #[test]
    fn different_asset_timeframe_and_config_are_rejected() {
        let dev = candidate("dev", "AAPL", "15M", "2026-01-01", "2026-01-31");
        let mut different_asset = candidate("oos", "MSFT", "15M", "2026-02-01", "2026-02-28");
        assert!(validate_candidates(&[dev.clone(), different_asset.clone()])
            .unwrap_err()
            .contains("mesmo ativo"));
        different_asset.asset = "AAPL".into();
        different_asset.timeframe = "1D".into();
        assert!(validate_candidates(&[dev.clone(), different_asset.clone()])
            .unwrap_err()
            .contains("mesmo timeframe"));
        different_asset.timeframe = "15M".into();
        different_asset.agent_config = json!({"seed":7,"temperature":0.1});
        assert!(validate_candidates(&[dev, different_asset])
            .unwrap_err()
            .contains("configuração ou versão"));
    }

    #[test]
    fn overlap_touching_and_duplicate_ranges_are_rejected_by_default() {
        let dev = candidate("dev", "AAPL", "15M", "2026-01-01", "2026-01-31");
        for (start, end, expected) in [
            ("2026-01-20", "2026-02-20", "OVERLAPPING"),
            ("2026-01-31", "2026-02-20", "TOUCHING_BOUNDARY"),
            ("2026-01-01", "2026-01-31", "DUPLICATE_RANGE"),
        ] {
            let error =
                validate_candidates(&[dev.clone(), candidate("oos", "AAPL", "15M", start, end)])
                    .unwrap_err();
            assert!(error.contains(expected), "{error}");
        }
    }

    #[test]
    fn unresolved_response_is_not_replaced_by_duration() {
        let trace = LifecycleHealthTrace {
            lifecycle_id: "l1".into(),
            candle_index: 1,
            timestamp: "2026-01-01T10:00:00Z".into(),
            position_age_candles: 1,
            position_age_market_minutes: 15,
            market_price: 100.0,
            exposure_pct: 50.0,
            return_since_entry_pct: -1.0,
            mfe_since_entry_pct: 0.0,
            mae_since_entry_pct: -1.0,
            distance_from_entry_atr: None,
            distance_from_mfe_pct: 1.0,
            giveback_from_mfe_pct: 1.0,
            giveback_relative_pct: None,
            vwap_change_since_entry: None,
            ema_spread_change_since_entry: None,
            rsi_change_since_entry: None,
            trend_changed: true,
            momentum_changed: false,
            trend_component: 0.5,
            momentum_component: 0.0,
            giveback_component: 0.2,
            vwap_component: 0.1,
            volatility_component: 0.0,
            time_component: 0.0,
            deterioration_score: 0.5,
            deterioration_level: "HIGH".into(),
            position_health: "DETERIORATING".into(),
            available_at_t: true,
        };
        let traces = vec![&trace];
        let (applicable, delay) = response_delay(&traces, &[], 3);
        assert!(applicable);
        assert_eq!(delay, None);
    }

    #[test]
    fn completed_batch_is_persistent_reloadable_and_exportable() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        let batch = LifecycleValidationBatch {
            batch_id: "batch-persisted".into(),
            name: "Persisted validation".into(),
            status: "COMPLETED".into(),
            asset: "AAPL".into(),
            timeframe: "15M".into(),
            agent_id: AGENT_ID.into(),
            agent_version: "MARKET_AI_INTRADAY_V1".into(),
            lifecycle_config_version: "POSITION_LIFECYCLE_CONFIG_V1".into(),
            deterioration_config_version: "POSITION_DETERIORATION_CONFIG_V1".into(),
            hold_diagnostics_version: market_hold_diagnostics::ENGINE_VERSION.into(),
            execution_model_version: "EXECUTION_MODEL_V1".into(),
            validation_engine_version: VALIDATION_ENGINE_VERSION.into(),
            dataset_count: 2,
            experiment_count: 2,
            lifecycle_count: 4,
            created_at: "2026-01-01 00:00:00".into(),
            started_at: Some("2026-01-01 00:00:00".into()),
            completed_at: Some("2026-01-01 00:00:01".into()),
        };
        let report = MultiPeriodLifecycleValidationReport {
            batch: batch.clone(),
            sample: LifecycleSampleSummary {
                datasets: 2,
                periods: 2,
                experiments: 2,
                lifecycles: 4,
                sample_status: "INSUFFICIENT".into(),
                ..LifecycleSampleSummary::default()
            },
            periods: vec![],
            lifecycles: vec![],
            deterioration_by_period: vec![],
            deterioration_outcomes: vec![],
            health_outcomes: vec![],
            components: vec![],
            consistency: vec![],
            evidence: EvidenceMatrix {
                sample_size: "INSUFFICIENT".into(),
                period_coverage: "PARTIAL".into(),
                late_reduction: "MIXED".into(),
                deterioration_stability: "INSUFFICIENT_SAMPLE".into(),
                health_outcome_relation: "NO_SIGNAL".into(),
                component_consistency: "INSUFFICIENT".into(),
                giveback_consistency: "INSUFFICIENT".into(),
            },
            outlier_sensitivity: vec![],
            quality: vec![],
            audit: ValidationAuditSummary::default(),
            warnings: vec!["test warning".into()],
            post_decision_only: true,
        };
        let raw = serde_json::to_string(&report).unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_validation_batches(id,name,asset,timeframe,agent_id,agent_version,lifecycle_config_version,deterioration_config_version,hold_diagnostics_version,execution_model_version,dataset_count,experiment_count,lifecycle_count,status,config_json,result_json,warnings_json,evidence_matrix_json,created_at,started_at,completed_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,2,2,4,'COMPLETED','{}',?11,'[]','{}',?12,?12,?13)",
            params![batch.batch_id,batch.name,batch.asset,batch.timeframe,batch.agent_id,batch.agent_version,batch.lifecycle_config_version,batch.deterioration_config_version,batch.hold_diagnostics_version,batch.execution_model_version,raw,batch.created_at,batch.completed_at],
        ).unwrap();

        assert_eq!(list(&connection).unwrap().len(), 1);
        let loaded = get(&connection, "batch-persisted").unwrap();
        assert_eq!(loaded.sample.lifecycles, 4);
        assert!(loaded.post_decision_only);
        let exported = export_json(&connection, "batch-persisted").unwrap();
        assert!(exported.contains("MULTI_PERIOD_LIFECYCLE_VALIDATION_V1"));
    }

    #[test]
    fn batch_reuses_persisted_artifacts_and_aggregates_by_lifecycle() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        insert_persisted_lifecycle_fixture(
            &connection,
            "dev",
            "2026-01-01T14:30:00Z",
            "2026-01-01T14:45:00Z",
        );
        insert_persisted_lifecycle_fixture(
            &connection,
            "oos",
            "2026-02-01T14:30:00Z",
            "2026-02-01T14:45:00Z",
        );
        let report = create(
            &mut connection,
            &LifecycleValidationBatchInput {
                name: "DEV plus OOS".into(),
                periods: vec![
                    super::super::market_lifecycle_validation::LifecycleValidationPeriodInput {
                        experiment_id: "experiment-dev".into(),
                        source_role: "DEVELOPMENT".into(),
                    },
                    super::super::market_lifecycle_validation::LifecycleValidationPeriodInput {
                        experiment_id: "experiment-oos".into(),
                        source_role: "OOS".into(),
                    },
                ],
            },
        )
        .unwrap();

        assert_eq!(report.batch.status, "COMPLETED");
        assert_eq!(report.sample.lifecycles, 2);
        assert_eq!(report.sample.lifecycle_late_reduction_rate_pct, 100.0);
        assert_eq!(report.periods.len(), 2);
        assert!(report.warnings.iter().all(|item| !item.contains("REBUILD")));
        assert!(report.post_decision_only);
        assert_eq!(
            get(&connection, &report.batch.batch_id)
                .unwrap()
                .lifecycles
                .len(),
            2
        );
    }

    #[test]
    fn quality_gate_rejects_invalid_persisted_ohlcv() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        insert_persisted_lifecycle_fixture(
            &connection,
            "quality",
            "2026-01-01T14:30:00Z",
            "2026-01-01T14:45:00Z",
        );
        connection
            .execute(
                "UPDATE market_candles SET high=98 WHERE dataset_id='dataset-quality' AND candle_index=0",
                [],
            )
            .unwrap();
        let candidate = load_candidate(&connection, "experiment-quality", "DEVELOPMENT").unwrap();
        let quality = dataset_quality(&connection, &candidate).unwrap();
        assert_eq!(quality.status, "FAIL");
        assert_eq!(quality.invalid_high_count, 1);
    }

    #[test]
    fn resume_preserves_completed_checkpoint_and_does_not_duplicate_rows() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        insert_persisted_lifecycle_fixture(
            &connection,
            "dev-resume",
            "2026-01-01T14:30:00Z",
            "2026-01-01T14:45:00Z",
        );
        insert_persisted_lifecycle_fixture(
            &connection,
            "oos-resume",
            "2026-02-01T14:30:00Z",
            "2026-02-01T14:45:00Z",
        );
        let created = create(
            &mut connection,
            &LifecycleValidationBatchInput {
                name: "Resume validation".into(),
                periods: vec![
                    super::super::market_lifecycle_validation::LifecycleValidationPeriodInput {
                        experiment_id: "experiment-dev-resume".into(),
                        source_role: "DEVELOPMENT".into(),
                    },
                    super::super::market_lifecycle_validation::LifecycleValidationPeriodInput {
                        experiment_id: "experiment-oos-resume".into(),
                        source_role: "OOS".into(),
                    },
                ],
            },
        )
        .unwrap();
        connection
            .execute(
                "UPDATE market_lifecycle_validation_batches SET status='PARTIAL' WHERE id=?1",
                [&created.batch.batch_id],
            )
            .unwrap();
        connection.execute(
            "UPDATE market_lifecycle_validation_periods SET period_status='FAILED' WHERE batch_id=?1 AND source_role='OOS'",
            [&created.batch.batch_id],
        ).unwrap();

        let resumed = resume(&mut connection, &created.batch.batch_id).unwrap();
        assert_eq!(resumed.batch.batch_id, created.batch.batch_id);
        assert_eq!(resumed.batch.status, "COMPLETED");
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_lifecycle_validation_periods WHERE batch_id=?1",
                    [&created.batch.batch_id],
                    |row| row.get::<_, usize>(0),
                )
                .unwrap(),
            2
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_lifecycle_validation_lifecycles WHERE batch_id=?1",
                    [&created.batch.batch_id],
                    |row| row.get::<_, usize>(0),
                )
                .unwrap(),
            2
        );
    }
}
