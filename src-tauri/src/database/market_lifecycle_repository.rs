use super::{
    market_hold_diagnostics::{post_decision_outcome, OutcomeCandle},
    market_hold_repository, market_identity,
    market_intraday::IntradayFeatures,
    market_lifecycle_intelligence::{
        DeteriorationInput, DeteriorationLevel, LifecycleEventType, LifecycleState,
        PositionDeteriorationConfig, PositionDeteriorationEngine, PositionLifecycleEngine,
        DETERIORATION_ENGINE_VERSION, LIFECYCLE_ENGINE_VERSION,
    },
    market_positions,
};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

const AGENT_ID: &str = "ai-intraday-v1";

#[derive(Debug, Clone)]
struct ExperimentIdentity {
    name: String,
    dataset_id: String,
    dataset_name: String,
    asset: String,
    timeframe: String,
    start_at: String,
    end_at: String,
    status: String,
}

#[derive(Debug, Clone)]
struct CandleRow {
    index: usize,
    timestamp: String,
    high: f64,
    low: f64,
    close: f64,
}

#[derive(Debug, Clone)]
struct DecisionMeta {
    id: i64,
    candle_index: usize,
    timestamp: String,
    observed_price: f64,
    confidence: Option<f64>,
    intent: Option<String>,
    reason_code: Option<String>,
    input_snapshot: Option<String>,
    trigger_reason: Option<String>,
    risk_result: Option<String>,
    execution_id: Option<i64>,
    execution_timestamp: Option<String>,
    execution_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleSummary {
    pub lifecycle_id: String,
    pub asset: String,
    pub timeframe: String,
    pub entry_decision_timestamp: Option<String>,
    pub entry_execution_timestamp: String,
    pub entry_price: f64,
    pub initial_exposure_pct: f64,
    pub max_exposure_pct: f64,
    pub exit_decision_timestamp: Option<String>,
    pub exit_execution_timestamp: Option<String>,
    pub exit_price: Option<f64>,
    pub duration_candles: usize,
    pub duration_market_minutes: usize,
    pub overnight: bool,
    pub status: String,
    pub realized_pnl: f64,
    pub realized_pnl_pct: f64,
    pub mfe_pct: f64,
    pub mae_pct: f64,
    pub giveback_pct: f64,
    pub exit_efficiency_pct: Option<f64>,
    pub entry_reason: Option<String>,
    pub exit_reason: Option<String>,
    pub entry_atr: Option<f64>,
    pub entry_vwap: Option<f64>,
    pub entry_ema9: Option<f64>,
    pub entry_ema21: Option<f64>,
    pub entry_rsi: Option<f64>,
    pub entry_relative_volume: Option<f64>,
    pub entry_market_state: Option<Value>,
    pub entry_session_phase: Option<String>,
    pub first_low_health_timestamp: Option<String>,
    pub first_moderate_deterioration_timestamp: Option<String>,
    pub first_high_deterioration_timestamp: Option<String>,
    pub response_delay_candles: Option<usize>,
    pub response_delay_market_minutes: Option<usize>,
    pub response_censored: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleEventView {
    pub id: i64,
    pub lifecycle_id: String,
    pub decision_id: Option<i64>,
    pub execution_id: Option<i64>,
    pub timestamp: String,
    pub candle_index: usize,
    pub event_type: String,
    pub state_before: String,
    pub state_after: String,
    pub position_before: String,
    pub position_after: String,
    pub exposure_before_pct: f64,
    pub exposure_after_pct: f64,
    pub market_price: f64,
    pub unrealized_pnl_pct: f64,
    pub mfe_so_far_pct: f64,
    pub mae_so_far_pct: f64,
    pub ai_intent: Option<String>,
    pub confidence: Option<f64>,
    pub reason_code: Option<String>,
    pub trigger_reason: Option<String>,
    pub risk_result: String,
    pub execution_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleHealthTrace {
    pub lifecycle_id: String,
    pub candle_index: usize,
    pub timestamp: String,
    pub position_age_candles: usize,
    pub position_age_market_minutes: usize,
    pub market_price: f64,
    pub exposure_pct: f64,
    pub return_since_entry_pct: f64,
    pub mfe_since_entry_pct: f64,
    pub mae_since_entry_pct: f64,
    pub distance_from_entry_atr: Option<f64>,
    pub distance_from_mfe_pct: f64,
    pub giveback_from_mfe_pct: f64,
    pub giveback_relative_pct: Option<f64>,
    pub vwap_change_since_entry: Option<f64>,
    pub ema_spread_change_since_entry: Option<f64>,
    pub rsi_change_since_entry: Option<f64>,
    pub trend_changed: bool,
    pub momentum_changed: bool,
    pub trend_component: f64,
    pub momentum_component: f64,
    pub giveback_component: f64,
    pub vwap_component: f64,
    pub volatility_component: f64,
    pub time_component: f64,
    pub deterioration_score: f64,
    pub deterioration_level: String,
    pub position_health: String,
    pub available_at_t: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleOutcome {
    pub lifecycle_id: String,
    pub candle_index: usize,
    pub decision_id: Option<i64>,
    pub hold_quality: Option<String>,
    pub forward_return_1: Option<f64>,
    pub forward_return_5: Option<f64>,
    pub mfe_5: Option<f64>,
    pub mae_5: Option<f64>,
    pub missed_reduction_window: bool,
    pub post_decision_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleOutcomeAggregate {
    pub label: String,
    pub count: usize,
    pub average_forward_1: f64,
    pub average_forward_5: f64,
    pub average_mfe_5: f64,
    pub average_mae_5: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitReasonAggregate {
    pub reason_code: String,
    pub count: usize,
    pub average_pnl_pct: f64,
    pub average_mfe_pct: f64,
    pub average_mae_pct: f64,
    pub average_giveback_pct: f64,
    pub average_duration_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleObservatorySummary {
    pub total_lifecycles: usize,
    pub open_lifecycles: usize,
    pub closed_lifecycles: usize,
    pub average_duration_minutes: f64,
    pub average_pnl_pct: f64,
    pub average_mfe_pct: f64,
    pub average_mae_pct: f64,
    pub average_giveback_pct: f64,
    pub late_reduction_events: usize,
    pub average_response_delay_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LateReductionSummary {
    pub total_hold_while_long: usize,
    pub potential_late_reductions: usize,
    pub rate_pct: f64,
    pub average_forward_5: f64,
    pub average_mae_5: f64,
    pub average_response_delay_minutes: f64,
    pub top_position_health: Option<String>,
    pub top_reason_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleIntelligenceReport {
    pub run_id: String,
    pub experiment_id: String,
    pub experiment_name: String,
    pub dataset_id: String,
    pub dataset_name: String,
    pub asset: String,
    pub timeframe: String,
    pub agent_id: String,
    pub lifecycle_engine_version: String,
    pub deterioration_engine_version: String,
    pub status: String,
    pub created_at: String,
    pub completed_at: Option<String>,
    pub summary: LifecycleObservatorySummary,
    pub late_reduction: LateReductionSummary,
    pub config: PositionDeteriorationConfig,
    pub lifecycles: Vec<LifecycleSummary>,
    pub events: Vec<LifecycleEventView>,
    pub health_trace: Vec<LifecycleHealthTrace>,
    pub outcomes: Vec<LifecycleOutcome>,
    pub health_outcomes: Vec<LifecycleOutcomeAggregate>,
    pub deterioration_outcomes: Vec<LifecycleOutcomeAggregate>,
    pub exit_reasons: Vec<ExitReasonAggregate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleComparisonMetric {
    pub label: String,
    pub development_value: f64,
    pub out_of_sample_value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleDistributionComparison {
    pub level: String,
    pub development_count: usize,
    pub out_of_sample_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LifecycleIntelligenceComparison {
    pub development_experiment_id: String,
    pub out_of_sample_experiment_id: String,
    pub asset: String,
    pub timeframe: String,
    pub metrics: Vec<LifecycleComparisonMetric>,
    pub deterioration_distribution: Vec<LifecycleDistributionComparison>,
}

fn round(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn average(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        round(values.iter().sum::<f64>() / values.len() as f64)
    }
}

fn new_run_id() -> String {
    format!(
        "lifecycle-intelligence-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn identity(connection: &Connection, experiment_id: &str) -> Result<ExperimentIdentity, String> {
    connection
        .query_row(
            "SELECT e.name,e.dataset_id,d.name,e.asset,e.timeframe,d.start_at,d.end_at,e.status
             FROM market_experiments e JOIN market_datasets d ON d.id=e.dataset_id
             WHERE e.id=?1",
            [experiment_id],
            |row| {
                Ok(ExperimentIdentity {
                    name: row.get(0)?,
                    dataset_id: row.get(1)?,
                    dataset_name: row.get(2)?,
                    asset: row.get(3)?,
                    timeframe: row.get(4)?,
                    start_at: row.get(5)?,
                    end_at: row.get(6)?,
                    status: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "experimento não encontrado".to_string())
}

fn load_candles(connection: &Connection, dataset_id: &str) -> Result<Vec<CandleRow>, String> {
    let mut statement = connection.prepare("SELECT candle_index,COALESCE(timestamp_utc,timestamp),high,low,close FROM market_candles WHERE dataset_id=?1 ORDER BY candle_index").map_err(|error|error.to_string())?;
    let rows = statement
        .query_map([dataset_id], |row| {
            Ok(CandleRow {
                index: row.get(0)?,
                timestamp: row.get(1)?,
                high: row.get(2)?,
                low: row.get(3)?,
                close: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

fn load_features(
    connection: &Connection,
    experiment_id: &str,
) -> Result<HashMap<usize, IntradayFeatures>, String> {
    let mut statement = connection
        .prepare("SELECT candle_index,features_json FROM market_intraday_feature_traces WHERE experiment_id=?1 ORDER BY candle_index")
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([experiment_id], |row| {
            Ok((row.get::<_, usize>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;
    let mut features = HashMap::new();
    for row in rows {
        let (index, json) = row.map_err(|error| error.to_string())?;
        features.insert(
            index,
            serde_json::from_str(&json)
                .map_err(|error| format!("feature trace inválida no candle {index}: {error}"))?,
        );
    }
    Ok(features)
}

fn load_decisions(
    connection: &Connection,
    experiment_id: &str,
) -> Result<HashMap<i64, DecisionMeta>, String> {
    let mut statement = connection.prepare("SELECT d.id,d.candle_index,d.timestamp,d.observed_price,d.confidence,a.intent,a.reason_code,c.input_snapshot_json,t.trigger_reason,r.result,x.id,x.timestamp,x.execution_price FROM market_decisions d LEFT JOIN market_ai_decisions a ON a.decision_id=d.id LEFT JOIN market_ai_decision_context c ON c.decision_id=d.id LEFT JOIN market_decision_trigger_audits t ON t.experiment_id=d.experiment_id AND t.agent_id=d.agent_id AND t.candle_index=d.candle_index LEFT JOIN market_risk_evaluations r ON r.decision_id=d.id LEFT JOIN market_orders o ON o.decision_id=d.id LEFT JOIN market_executions x ON x.order_id=o.id WHERE d.experiment_id=?1 AND d.agent_id=?2 ORDER BY d.candle_index").map_err(|error|error.to_string())?;
    let rows = statement
        .query_map(params![experiment_id, AGENT_ID], |row| {
            Ok(DecisionMeta {
                id: row.get(0)?,
                candle_index: row.get(1)?,
                timestamp: row.get(2)?,
                observed_price: row.get(3)?,
                confidence: row.get(4)?,
                intent: row.get(5)?,
                reason_code: row.get(6)?,
                input_snapshot: row.get(7)?,
                trigger_reason: row.get(8)?,
                risk_result: row.get(9)?,
                execution_id: row.get(10)?,
                execution_timestamp: row.get(11)?,
                execution_price: row.get(12)?,
            })
        })
        .map_err(|error| error.to_string())?;
    let mut decisions = HashMap::new();
    for row in rows {
        let decision = row.map_err(|error| error.to_string())?;
        decisions.insert(decision.id, decision);
    }
    Ok(decisions)
}

fn event_type(action: &str) -> Option<LifecycleEventType> {
    match action {
        "ENTER_LONG" => Some(LifecycleEventType::Open),
        "INCREASE_LONG" => Some(LifecycleEventType::Increase),
        "HOLD_POSITION" => Some(LifecycleEventType::Hold),
        "REDUCE_LONG" => Some(LifecycleEventType::Reduce),
        "EXIT_LONG" => Some(LifecycleEventType::Exit),
        "FORCED_EXIT" => Some(LifecycleEventType::ForcedExit),
        _ => None,
    }
}

fn index_for_timestamp(candles: &[CandleRow], timestamp: &str) -> Option<usize> {
    candles
        .iter()
        .find(|candle| candle.timestamp == timestamp)
        .map(|candle| candle.index)
}

fn json_text(value: &Option<String>, pointer: &str) -> Option<String> {
    value
        .as_ref()
        .and_then(|json| serde_json::from_str::<Value>(json).ok())
        .and_then(|snapshot| {
            snapshot
                .pointer(pointer)
                .and_then(Value::as_str)
                .map(str::to_string)
        })
}

fn json_value(value: &Option<String>, pointer: &str) -> Option<Value> {
    value
        .as_ref()
        .and_then(|json| serde_json::from_str::<Value>(json).ok())
        .and_then(|snapshot| snapshot.pointer(pointer).cloned())
}

fn feature_at_or_before(
    features: &HashMap<usize, IntradayFeatures>,
    index: usize,
) -> Option<&IntradayFeatures> {
    (0..=index)
        .rev()
        .find_map(|candidate| features.get(&candidate))
}

fn persist_trace(
    tx: &Transaction<'_>,
    run_id: &str,
    trace: &LifecycleHealthTrace,
) -> Result<(), String> {
    tx.execute("INSERT INTO market_lifecycle_health_trace(run_id,lifecycle_id,candle_index,timestamp,position_age_candles,position_age_market_minutes,market_price,exposure_pct,return_since_entry_pct,mfe_since_entry_pct,mae_since_entry_pct,distance_from_entry_atr,distance_from_mfe_pct,giveback_from_mfe_pct,giveback_relative_pct,vwap_change_since_entry,ema_spread_change_since_entry,rsi_change_since_entry,trend_changed,momentum_changed,trend_component,momentum_component,giveback_component,vwap_component,volatility_component,time_component,deterioration_score,deterioration_level,position_health,available_at_t) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,1)",params![run_id,trace.lifecycle_id,trace.candle_index,trace.timestamp,trace.position_age_candles,trace.position_age_market_minutes,trace.market_price,trace.exposure_pct,trace.return_since_entry_pct,trace.mfe_since_entry_pct,trace.mae_since_entry_pct,trace.distance_from_entry_atr,trace.distance_from_mfe_pct,trace.giveback_from_mfe_pct,trace.giveback_relative_pct,trace.vwap_change_since_entry,trace.ema_spread_change_since_entry,trace.rsi_change_since_entry,trace.trend_changed,trace.momentum_changed,trace.trend_component,trace.momentum_component,trace.giveback_component,trace.vwap_component,trace.volatility_component,trace.time_component,trace.deterioration_score,trace.deterioration_level,trace.position_health]).map_err(|error|error.to_string())?;
    Ok(())
}

fn insert_analysis(
    tx: &Transaction<'_>,
    run_id: &str,
    item: &LifecycleSummary,
) -> Result<(), String> {
    tx.execute("INSERT INTO market_lifecycle_analyses(lifecycle_id,run_id,asset,timeframe,entry_decision_timestamp,entry_execution_timestamp,entry_price,initial_exposure_pct,max_exposure_pct,exit_decision_timestamp,exit_execution_timestamp,exit_price,duration_candles,duration_market_minutes,overnight,status,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,giveback_pct,exit_efficiency_pct,entry_reason,exit_reason,entry_atr,entry_vwap,entry_ema9,entry_ema21,entry_rsi,entry_relative_volume,entry_market_state_json,entry_session_phase,first_low_health_timestamp,first_moderate_deterioration_timestamp,first_high_deterioration_timestamp,response_delay_candles,response_delay_market_minutes,response_censored) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38)",params![item.lifecycle_id,run_id,item.asset,item.timeframe,item.entry_decision_timestamp,item.entry_execution_timestamp,item.entry_price,item.initial_exposure_pct,item.max_exposure_pct,item.exit_decision_timestamp,item.exit_execution_timestamp,item.exit_price,item.duration_candles,item.duration_market_minutes,item.overnight,item.status,item.realized_pnl,item.realized_pnl_pct,item.mfe_pct,item.mae_pct,item.giveback_pct,item.exit_efficiency_pct,item.entry_reason,item.exit_reason,item.entry_atr,item.entry_vwap,item.entry_ema9,item.entry_ema21,item.entry_rsi,item.entry_relative_volume,item.entry_market_state.as_ref().map(Value::to_string),item.entry_session_phase,item.first_low_health_timestamp,item.first_moderate_deterioration_timestamp,item.first_high_deterioration_timestamp,item.response_delay_candles,item.response_delay_market_minutes,item.response_censored]).map_err(|error|error.to_string())?;
    Ok(())
}

pub fn generate(
    connection: &mut Connection,
    experiment_id: &str,
) -> Result<LifecycleIntelligenceReport, String> {
    let experiment = identity(connection, experiment_id)?;
    if experiment.status != "completed" {
        return Err("a análise requer um experimento concluído".into());
    }
    if experiment.timeframe != "15M" {
        return Err("Position Lifecycle Intelligence v0.5.3 requer timeframe 15M".into());
    }
    let agent_present: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM market_experiment_agents WHERE experiment_id=?1 AND agent_id=?2)",
            params![experiment_id, AGENT_ID],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !agent_present {
        return Err("o experimento não contém AI Intraday V1".into());
    }

    let hold_report = market_hold_repository::generate(connection, experiment_id)?;
    let hold_quality = hold_report
        .items
        .iter()
        .map(|item| (item.decision_id, item.quality.clone()))
        .collect::<HashMap<_, _>>();
    let position_report = market_positions::load_report(connection, experiment_id)?;
    let source_lifecycles = position_report
        .lifecycles
        .into_iter()
        .filter(|item| item.agent_id == AGENT_ID)
        .collect::<Vec<_>>();
    let source_events = position_report
        .events
        .into_iter()
        .filter(|item| {
            source_lifecycles
                .iter()
                .any(|lifecycle| lifecycle.id == item.lifecycle_id)
        })
        .collect::<Vec<_>>();
    let candles = load_candles(connection, &experiment.dataset_id)?;
    let outcome_candles = candles
        .iter()
        .map(|candle| OutcomeCandle {
            high: candle.high,
            low: candle.low,
            close: candle.close,
        })
        .collect::<Vec<_>>();
    let features = load_features(connection, experiment_id)?;
    let decisions = load_decisions(connection, experiment_id)?;
    let decisions_by_candle = decisions
        .values()
        .map(|decision| (decision.candle_index, decision.id))
        .collect::<HashMap<_, _>>();
    let engine = PositionDeteriorationEngine::default();
    let run_id = new_run_id();
    let config_json = serde_json::to_string(engine.config()).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction.execute("DELETE FROM market_lifecycle_analysis_runs WHERE experiment_id=?1 AND agent_id=?2 AND lifecycle_engine_version=?3 AND deterioration_engine_version=?4",params![experiment_id,AGENT_ID,LIFECYCLE_ENGINE_VERSION,DETERIORATION_ENGINE_VERSION]).map_err(|error|error.to_string())?;
    transaction.execute("INSERT INTO market_lifecycle_analysis_runs(id,experiment_id,agent_id,lifecycle_engine_version,deterioration_engine_version,deterioration_config_json,status) VALUES(?1,?2,?3,?4,?5,?6,'running')",params![run_id,experiment_id,AGENT_ID,LIFECYCLE_ENGINE_VERSION,DETERIORATION_ENGINE_VERSION,config_json]).map_err(|error|error.to_string())?;

    let mut late_reduction_events = 0usize;
    for lifecycle in &source_lifecycles {
        let lifecycle_events = source_events
            .iter()
            .filter(|event| event.lifecycle_id == lifecycle.id)
            .collect::<Vec<_>>();
        let entry_event = lifecycle_events
            .iter()
            .find(|event| event.action == "ENTER_LONG")
            .copied();
        let exit_event = lifecycle_events
            .iter()
            .rev()
            .find(|event| event.action == "EXIT_LONG" || event.new_exposure_pct <= 0.0001)
            .copied();
        let entry_decision = entry_event
            .and_then(|event| event.decision_id)
            .and_then(|id| decisions.get(&id));
        let exit_decision = exit_event
            .and_then(|event| event.decision_id)
            .and_then(|id| decisions.get(&id));
        let start_index = index_for_timestamp(&candles, &lifecycle.opened_at)
            .or_else(|| entry_decision.map(|decision| decision.candle_index.saturating_add(1)))
            .ok_or_else(|| format!("entrada do lifecycle {} não encontrada", lifecycle.id))?;
        let end_index = lifecycle
            .closed_at
            .as_deref()
            .and_then(|timestamp| index_for_timestamp(&candles, timestamp))
            .unwrap_or_else(|| candles.last().map_or(start_index, |candle| candle.index));
        let baseline_index = entry_decision.map_or(start_index, |decision| decision.candle_index);
        let baseline = feature_at_or_before(&features, baseline_index)
            .ok_or_else(|| format!("entry baseline ausente no lifecycle {}", lifecycle.id))?;
        let entry_atr = baseline.atr_14;
        let entry_vwap = baseline.vwap;
        let entry_spread = baseline.ema_spread_atr;
        let entry_rsi = baseline.rsi_14;
        let entry_short_return = baseline.short_term_return;
        let mut highest = lifecycle.entry_price;
        let mut lowest = lifecycle.entry_price;
        let mut exposure = lifecycle.initial_exposure_pct;
        let mut traces = Vec::new();
        let event_indices = lifecycle_events
            .iter()
            .map(|event| {
                let index = event
                    .execution_id
                    .and_then(|_| index_for_timestamp(&candles, &event.timestamp))
                    .or_else(|| {
                        event
                            .decision_id
                            .and_then(|id| decisions.get(&id))
                            .map(|decision| decision.candle_index)
                    })
                    .unwrap_or(start_index);
                (index, *event)
            })
            .collect::<Vec<_>>();
        for index in start_index..=end_index {
            let candle = candles
                .get(index)
                .ok_or_else(|| format!("candle {index} ausente"))?;
            for (_, event) in event_indices
                .iter()
                .filter(|(event_index, _)| *event_index == index)
            {
                exposure = event.new_exposure_pct;
            }
            highest = highest.max(candle.high);
            lowest = lowest.min(candle.low);
            let feature = feature_at_or_before(&features, index);
            let return_since_entry = (candle.close / lifecycle.entry_price - 1.0) * 100.0;
            let mfe = ((highest / lifecycle.entry_price - 1.0) * 100.0).max(0.0);
            let mae = ((lowest / lifecycle.entry_price - 1.0) * 100.0).min(0.0);
            let giveback = (mfe - return_since_entry).max(0.0);
            let distance_from_entry_atr = entry_atr
                .filter(|atr| *atr > 0.0)
                .map(|atr| (candle.close - lifecycle.entry_price) / atr);
            let vwap_change = feature
                .and_then(|item| item.vwap)
                .zip(entry_vwap)
                .map(|(current, entry)| current - entry);
            let spread_change = feature
                .and_then(|item| item.ema_spread_atr)
                .zip(entry_spread)
                .map(|(current, entry)| current - entry);
            let rsi_change = feature
                .and_then(|item| item.rsi_14)
                .zip(entry_rsi)
                .map(|(current, entry)| current - entry);
            let trend_changed = spread_change.is_some_and(|change| change <= -0.5);
            let momentum_changed = rsi_change.is_some_and(|change| change <= -10.0)
                || feature
                    .and_then(|item| item.short_term_return)
                    .zip(entry_short_return)
                    .is_some_and(|(current, entry)| current < 0.0 && entry >= 0.0);
            let assessment = engine.assess(DeteriorationInput {
                position_age_candles: index.saturating_sub(start_index),
                return_since_entry_pct: return_since_entry,
                mfe_since_entry_pct: mfe,
                mae_since_entry_pct: mae,
                giveback_from_mfe_pct: giveback,
                distance_from_entry_atr,
                vwap_change_atr: vwap_change.zip(entry_atr).map(|(value, atr)| value / atr),
                ema_spread_change_atr: spread_change,
                rsi_change,
                atr_change_ratio: feature
                    .and_then(|item| item.atr_14)
                    .zip(entry_atr)
                    .filter(|(_, entry)| *entry > 0.0)
                    .map(|(current, entry)| current / entry - 1.0),
                session_progress: feature.map(|item| item.session_progress),
                price_below_vwap: feature
                    .and_then(|item| item.vwap)
                    .is_some_and(|vwap| candle.close < vwap),
                trend_deteriorated: trend_changed,
                momentum_deteriorated: momentum_changed,
            });
            let trace = LifecycleHealthTrace {
                lifecycle_id: lifecycle.id.clone(),
                candle_index: index,
                timestamp: candle.timestamp.clone(),
                position_age_candles: index.saturating_sub(start_index),
                position_age_market_minutes: index.saturating_sub(start_index) * 15,
                market_price: round(candle.close),
                exposure_pct: round(exposure),
                return_since_entry_pct: round(return_since_entry),
                mfe_since_entry_pct: round(mfe),
                mae_since_entry_pct: round(mae),
                distance_from_entry_atr: distance_from_entry_atr.map(round),
                distance_from_mfe_pct: round(return_since_entry - mfe),
                giveback_from_mfe_pct: round(giveback),
                giveback_relative_pct: (mfe > 0.0).then(|| round(giveback / mfe * 100.0)),
                vwap_change_since_entry: vwap_change.map(round),
                ema_spread_change_since_entry: spread_change.map(round),
                rsi_change_since_entry: rsi_change.map(round),
                trend_changed,
                momentum_changed,
                trend_component: assessment.trend_component,
                momentum_component: assessment.momentum_component,
                giveback_component: assessment.giveback_component,
                vwap_component: assessment.vwap_component,
                volatility_component: assessment.volatility_component,
                time_component: assessment.time_component,
                deterioration_score: assessment.score,
                deterioration_level: assessment.level.as_str().into(),
                position_health: assessment.health.as_str().into(),
                available_at_t: true,
            };
            persist_trace(&transaction, &run_id, &trace)?;
            let outcome = post_decision_outcome(&outcome_candles, index, candle.close);
            let decision_id = decisions_by_candle.get(&index).copied();
            let has_reduction = event_indices.iter().any(|(event_index, event)| {
                *event_index > index
                    && *event_index <= index.saturating_add(5)
                    && matches!(event.action.as_str(), "REDUCE_LONG" | "EXIT_LONG")
            });
            let missed_reduction = matches!(
                assessment.level,
                DeteriorationLevel::High | DeteriorationLevel::Critical
            ) && !has_reduction
                && (outcome.forward_5.is_some_and(|value| value < 0.0)
                    || outcome.mae_5.is_some_and(|value| value <= -0.5));
            let quality = decision_id.and_then(|id| hold_quality.get(&id).cloned());
            if quality.as_deref() == Some("POTENTIAL_LATE_REDUCTION") {
                late_reduction_events += 1;
            }
            transaction.execute("INSERT INTO market_lifecycle_post_decision_outcomes(run_id,lifecycle_id,candle_index,decision_id,hold_quality,forward_return_1,forward_return_5,mfe_5,mae_5,missed_reduction_window,post_decision_only) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,1)",params![run_id,lifecycle.id,index,decision_id,quality,outcome.forward_1,outcome.forward_5,outcome.mfe_5,outcome.mae_5,missed_reduction]).map_err(|error|error.to_string())?;
            traces.push(trace);
        }
        let first_low = traces
            .iter()
            .find(|trace| {
                matches!(
                    trace.position_health.as_str(),
                    "WEAKENING" | "DETERIORATING" | "CRITICAL"
                )
            })
            .map(|trace| trace.timestamp.clone());
        let first_moderate = traces
            .iter()
            .find(|trace| {
                matches!(
                    trace.deterioration_level.as_str(),
                    "MODERATE" | "HIGH" | "CRITICAL"
                )
            })
            .map(|trace| trace.timestamp.clone());
        let first_high_trace = traces
            .iter()
            .find(|trace| matches!(trace.deterioration_level.as_str(), "HIGH" | "CRITICAL"));
        let response_index = first_high_trace.and_then(|first| {
            event_indices
                .iter()
                .filter(|(index, event)| {
                    *index >= first.candle_index
                        && matches!(event.action.as_str(), "REDUCE_LONG" | "EXIT_LONG")
                })
                .map(|(index, _)| *index)
                .min()
        });
        let response_delay = first_high_trace.map(|first| {
            response_index
                .unwrap_or(end_index)
                .saturating_sub(first.candle_index)
        });
        let analysis = LifecycleSummary {
            lifecycle_id: lifecycle.id.clone(),
            asset: lifecycle.asset.clone(),
            timeframe: experiment.timeframe.clone(),
            entry_decision_timestamp: entry_decision.map(|value| value.timestamp.clone()),
            entry_execution_timestamp: lifecycle.opened_at.clone(),
            entry_price: lifecycle.entry_price,
            initial_exposure_pct: lifecycle.initial_exposure_pct,
            max_exposure_pct: lifecycle.max_exposure_pct,
            exit_decision_timestamp: exit_decision.map(|value| value.timestamp.clone()),
            exit_execution_timestamp: lifecycle.closed_at.clone(),
            exit_price: lifecycle.exit_price,
            duration_candles: lifecycle.holding_candles,
            duration_market_minutes: lifecycle.holding_market_minutes,
            overnight: lifecycle.overnight,
            status: lifecycle.status.clone(),
            realized_pnl: lifecycle.realized_pnl,
            realized_pnl_pct: lifecycle.realized_pnl_pct,
            mfe_pct: lifecycle.mfe_pct,
            mae_pct: lifecycle.mae_pct,
            giveback_pct: lifecycle.profit_giveback_pct,
            exit_efficiency_pct: lifecycle.exit_efficiency_pct,
            entry_reason: lifecycle.entry_reason_code.clone(),
            exit_reason: lifecycle.exit_reason_code.clone(),
            entry_atr,
            entry_vwap,
            entry_ema9: baseline.ema_9,
            entry_ema21: baseline.ema_21,
            entry_rsi,
            entry_relative_volume: baseline.relative_volume,
            entry_market_state: entry_decision
                .and_then(|value| json_value(&value.input_snapshot, "/marketState")),
            entry_session_phase: entry_decision
                .and_then(|value| json_text(&value.input_snapshot, "/market/sessionPhase"))
                .or_else(|| Some(baseline.session_phase.as_str().into())),
            first_low_health_timestamp: first_low,
            first_moderate_deterioration_timestamp: first_moderate,
            first_high_deterioration_timestamp: first_high_trace
                .map(|trace| trace.timestamp.clone()),
            response_delay_candles: response_delay,
            response_delay_market_minutes: response_delay.map(|value| value * 15),
            response_censored: first_high_trace.is_some() && response_index.is_none(),
        };
        insert_analysis(&transaction, &run_id, &analysis)?;

        let trace_by_index = traces
            .iter()
            .map(|trace| (trace.candle_index, trace))
            .collect::<HashMap<_, _>>();
        let mut lifecycle_state = LifecycleState::Flat;
        for (index, event) in event_indices {
            let Some(kind) = event_type(&event.action) else {
                continue;
            };
            let meta = event.decision_id.and_then(|id| decisions.get(&id));
            let trace = trace_by_index
                .get(&index)
                .copied()
                .or_else(|| traces.first())
                .ok_or_else(|| "trace de lifecycle ausente".to_string())?;
            let before = lifecycle_state;
            let requested = PositionLifecycleEngine::request(lifecycle_state, kind)?;
            lifecycle_state = match kind {
                LifecycleEventType::Open
                | LifecycleEventType::Reduce
                | LifecycleEventType::Exit
                | LifecycleEventType::ForcedExit => {
                    PositionLifecycleEngine::settle(requested, event.new_exposure_pct)?
                }
                LifecycleEventType::Increase | LifecycleEventType::Hold => requested,
            };
            let execution_id = event
                .execution_id
                .or_else(|| meta.and_then(|value| value.execution_id));
            let event_timestamp = meta
                .and_then(|value| value.execution_timestamp.clone())
                .unwrap_or_else(|| event.timestamp.clone());
            let confidence = event
                .confidence
                .or_else(|| meta.and_then(|value| value.confidence));
            let reason_code = event
                .reason_code
                .clone()
                .or_else(|| meta.and_then(|value| value.reason_code.clone()));
            let risk_result = if event.risk_result.is_empty() {
                meta.and_then(|value| value.risk_result.clone())
                    .unwrap_or_else(|| "UNKNOWN".into())
            } else {
                event.risk_result.clone()
            };
            transaction.execute("INSERT INTO market_lifecycle_events(run_id,lifecycle_id,decision_id,execution_id,timestamp,candle_index,event_type,state_before,state_after,position_before,position_after,exposure_before_pct,exposure_after_pct,market_price,unrealized_pnl_pct,mfe_so_far_pct,mae_so_far_pct,ai_intent,confidence,reason_code,trigger_reason,risk_result,execution_result) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23)",params![run_id,lifecycle.id,event.decision_id,execution_id,event_timestamp,index,kind.as_str(),before.as_str(),lifecycle_state.as_str(),before.as_str(),lifecycle_state.as_str(),event.previous_exposure_pct,event.new_exposure_pct,meta.and_then(|value|value.execution_price).unwrap_or_else(||meta.map_or(trace.market_price,|value|value.observed_price)),trace.return_since_entry_pct,trace.mfe_since_entry_pct,trace.mae_since_entry_pct,meta.and_then(|value|value.intent.clone()),confidence,reason_code,meta.and_then(|value|value.trigger_reason.clone()),risk_result,if execution_id.is_some(){"FILLED"}else{"NOT_REQUIRED"}]).map_err(|error|error.to_string())?;
        }
    }
    transaction.execute("UPDATE market_lifecycle_analysis_runs SET status='completed',total_lifecycles=?1,late_reduction_events=?2,completed_at=CURRENT_TIMESTAMP WHERE id=?3",params![source_lifecycles.len(),late_reduction_events,run_id]).map_err(|error|error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    get(connection, experiment_id)
}

fn aggregate_outcomes(
    connection: &Connection,
    run_id: &str,
    column: &str,
) -> Result<Vec<LifecycleOutcomeAggregate>, String> {
    let sql = format!("SELECT h.{column},COUNT(*),COALESCE(AVG(o.forward_return_1),0),COALESCE(AVG(o.forward_return_5),0),COALESCE(AVG(o.mfe_5),0),COALESCE(AVG(o.mae_5),0) FROM market_lifecycle_health_trace h JOIN market_lifecycle_post_decision_outcomes o ON o.run_id=h.run_id AND o.lifecycle_id=h.lifecycle_id AND o.candle_index=h.candle_index WHERE h.run_id=?1 GROUP BY h.{column} ORDER BY h.{column}");
    let mut statement = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([run_id], |row| {
            Ok(LifecycleOutcomeAggregate {
                label: row.get(0)?,
                count: row.get(1)?,
                average_forward_1: row.get(2)?,
                average_forward_5: row.get(3)?,
                average_mfe_5: row.get(4)?,
                average_mae_5: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

fn aggregate_exit_reasons(
    connection: &Connection,
    run_id: &str,
) -> Result<Vec<ExitReasonAggregate>, String> {
    let mut statement = connection.prepare("SELECT COALESCE(exit_reason,'UNSPECIFIED'),COUNT(*),COALESCE(AVG(realized_pnl_pct),0),COALESCE(AVG(mfe_pct),0),COALESCE(AVG(mae_pct),0),COALESCE(AVG(giveback_pct),0),COALESCE(AVG(duration_market_minutes),0) FROM market_lifecycle_analyses WHERE run_id=?1 AND status='CLOSED' GROUP BY COALESCE(exit_reason,'UNSPECIFIED') ORDER BY COUNT(*) DESC,COALESCE(exit_reason,'UNSPECIFIED')").map_err(|error|error.to_string())?;
    let rows = statement
        .query_map([run_id], |row| {
            Ok(ExitReasonAggregate {
                reason_code: row.get(0)?,
                count: row.get(1)?,
                average_pnl_pct: row.get(2)?,
                average_mfe_pct: row.get(3)?,
                average_mae_pct: row.get(4)?,
                average_giveback_pct: row.get(5)?,
                average_duration_minutes: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(rows)
}

pub fn get(
    connection: &Connection,
    experiment_id: &str,
) -> Result<LifecycleIntelligenceReport, String> {
    let experiment = identity(connection, experiment_id)?;
    let (run_id, lifecycle_version, deterioration_version, config_json, status, created_at, completed_at) = connection.query_row("SELECT id,lifecycle_engine_version,deterioration_engine_version,deterioration_config_json,status,created_at,completed_at FROM market_lifecycle_analysis_runs WHERE experiment_id=?1 AND agent_id=?2 ORDER BY created_at DESC LIMIT 1",params![experiment_id,AGENT_ID],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?,row.get::<_,String>(5)?,row.get::<_,Option<String>>(6)?))).optional().map_err(|error|error.to_string())?.ok_or_else(||"análise de lifecycle ainda não gerada".to_string())?;
    let config = serde_json::from_str(&config_json).map_err(|error| error.to_string())?;
    let mut lifecycle_statement=connection.prepare("SELECT lifecycle_id,asset,timeframe,entry_decision_timestamp,entry_execution_timestamp,entry_price,initial_exposure_pct,max_exposure_pct,exit_decision_timestamp,exit_execution_timestamp,exit_price,duration_candles,duration_market_minutes,overnight,status,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,giveback_pct,exit_efficiency_pct,entry_reason,exit_reason,entry_atr,entry_vwap,entry_ema9,entry_ema21,entry_rsi,entry_relative_volume,entry_market_state_json,entry_session_phase,first_low_health_timestamp,first_moderate_deterioration_timestamp,first_high_deterioration_timestamp,response_delay_candles,response_delay_market_minutes,response_censored FROM market_lifecycle_analyses WHERE run_id=?1 ORDER BY entry_execution_timestamp").map_err(|error|error.to_string())?;
    let lifecycles = lifecycle_statement
        .query_map([&run_id], |row| {
            Ok(LifecycleSummary {
                lifecycle_id: row.get(0)?,
                asset: row.get(1)?,
                timeframe: row.get(2)?,
                entry_decision_timestamp: row.get(3)?,
                entry_execution_timestamp: row.get(4)?,
                entry_price: row.get(5)?,
                initial_exposure_pct: row.get(6)?,
                max_exposure_pct: row.get(7)?,
                exit_decision_timestamp: row.get(8)?,
                exit_execution_timestamp: row.get(9)?,
                exit_price: row.get(10)?,
                duration_candles: row.get(11)?,
                duration_market_minutes: row.get(12)?,
                overnight: row.get::<_, i64>(13)? != 0,
                status: row.get(14)?,
                realized_pnl: row.get(15)?,
                realized_pnl_pct: row.get(16)?,
                mfe_pct: row.get(17)?,
                mae_pct: row.get(18)?,
                giveback_pct: row.get(19)?,
                exit_efficiency_pct: row.get(20)?,
                entry_reason: row.get(21)?,
                exit_reason: row.get(22)?,
                entry_atr: row.get(23)?,
                entry_vwap: row.get(24)?,
                entry_ema9: row.get(25)?,
                entry_ema21: row.get(26)?,
                entry_rsi: row.get(27)?,
                entry_relative_volume: row.get(28)?,
                entry_market_state: row
                    .get::<_, Option<String>>(29)?
                    .and_then(|value| serde_json::from_str(&value).ok()),
                entry_session_phase: row.get(30)?,
                first_low_health_timestamp: row.get(31)?,
                first_moderate_deterioration_timestamp: row.get(32)?,
                first_high_deterioration_timestamp: row.get(33)?,
                response_delay_candles: row.get(34)?,
                response_delay_market_minutes: row.get(35)?,
                response_censored: row.get::<_, i64>(36)? != 0,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut event_statement=connection.prepare("SELECT id,lifecycle_id,decision_id,execution_id,timestamp,candle_index,event_type,state_before,state_after,position_before,position_after,exposure_before_pct,exposure_after_pct,market_price,unrealized_pnl_pct,mfe_so_far_pct,mae_so_far_pct,ai_intent,confidence,reason_code,trigger_reason,risk_result,execution_result FROM market_lifecycle_events WHERE run_id=?1 ORDER BY candle_index,id").map_err(|error|error.to_string())?;
    let events = event_statement
        .query_map([&run_id], |row| {
            Ok(LifecycleEventView {
                id: row.get(0)?,
                lifecycle_id: row.get(1)?,
                decision_id: row.get(2)?,
                execution_id: row.get(3)?,
                timestamp: row.get(4)?,
                candle_index: row.get(5)?,
                event_type: row.get(6)?,
                state_before: row.get(7)?,
                state_after: row.get(8)?,
                position_before: row.get(9)?,
                position_after: row.get(10)?,
                exposure_before_pct: row.get(11)?,
                exposure_after_pct: row.get(12)?,
                market_price: row.get(13)?,
                unrealized_pnl_pct: row.get(14)?,
                mfe_so_far_pct: row.get(15)?,
                mae_so_far_pct: row.get(16)?,
                ai_intent: row.get(17)?,
                confidence: row.get(18)?,
                reason_code: row.get(19)?,
                trigger_reason: row.get(20)?,
                risk_result: row.get(21)?,
                execution_result: row.get(22)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut trace_statement=connection.prepare("SELECT lifecycle_id,candle_index,timestamp,position_age_candles,position_age_market_minutes,market_price,exposure_pct,return_since_entry_pct,mfe_since_entry_pct,mae_since_entry_pct,distance_from_entry_atr,distance_from_mfe_pct,giveback_from_mfe_pct,giveback_relative_pct,vwap_change_since_entry,ema_spread_change_since_entry,rsi_change_since_entry,trend_changed,momentum_changed,trend_component,momentum_component,giveback_component,vwap_component,volatility_component,time_component,deterioration_score,deterioration_level,position_health,available_at_t FROM market_lifecycle_health_trace WHERE run_id=?1 ORDER BY candle_index").map_err(|error|error.to_string())?;
    let health_trace = trace_statement
        .query_map([&run_id], |row| {
            Ok(LifecycleHealthTrace {
                lifecycle_id: row.get(0)?,
                candle_index: row.get(1)?,
                timestamp: row.get(2)?,
                position_age_candles: row.get(3)?,
                position_age_market_minutes: row.get(4)?,
                market_price: row.get(5)?,
                exposure_pct: row.get(6)?,
                return_since_entry_pct: row.get(7)?,
                mfe_since_entry_pct: row.get(8)?,
                mae_since_entry_pct: row.get(9)?,
                distance_from_entry_atr: row.get(10)?,
                distance_from_mfe_pct: row.get(11)?,
                giveback_from_mfe_pct: row.get(12)?,
                giveback_relative_pct: row.get(13)?,
                vwap_change_since_entry: row.get(14)?,
                ema_spread_change_since_entry: row.get(15)?,
                rsi_change_since_entry: row.get(16)?,
                trend_changed: row.get::<_, i64>(17)? != 0,
                momentum_changed: row.get::<_, i64>(18)? != 0,
                trend_component: row.get(19)?,
                momentum_component: row.get(20)?,
                giveback_component: row.get(21)?,
                vwap_component: row.get(22)?,
                volatility_component: row.get(23)?,
                time_component: row.get(24)?,
                deterioration_score: row.get(25)?,
                deterioration_level: row.get(26)?,
                position_health: row.get(27)?,
                available_at_t: row.get::<_, i64>(28)? != 0,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut outcome_statement=connection.prepare("SELECT lifecycle_id,candle_index,decision_id,hold_quality,forward_return_1,forward_return_5,mfe_5,mae_5,missed_reduction_window,post_decision_only FROM market_lifecycle_post_decision_outcomes WHERE run_id=?1 ORDER BY candle_index").map_err(|error|error.to_string())?;
    let outcomes = outcome_statement
        .query_map([&run_id], |row| {
            Ok(LifecycleOutcome {
                lifecycle_id: row.get(0)?,
                candle_index: row.get(1)?,
                decision_id: row.get(2)?,
                hold_quality: row.get(3)?,
                forward_return_1: row.get(4)?,
                forward_return_5: row.get(5)?,
                mfe_5: row.get(6)?,
                mae_5: row.get(7)?,
                missed_reduction_window: row.get::<_, i64>(8)? != 0,
                post_decision_only: row.get::<_, i64>(9)? != 0,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let closed = lifecycles
        .iter()
        .filter(|item| item.status == "CLOSED")
        .count();
    let response_delays = lifecycles
        .iter()
        .filter_map(|item| item.response_delay_market_minutes.map(|value| value as f64))
        .collect::<Vec<_>>();
    let late_items = outcomes
        .iter()
        .filter(|item| item.hold_quality.as_deref() == Some("POTENTIAL_LATE_REDUCTION"))
        .collect::<Vec<_>>();
    let hold_long = outcomes
        .iter()
        .filter(|item| item.hold_quality.is_some())
        .count();
    let top_health = top_label(
        health_trace
            .iter()
            .filter(|trace| {
                outcomes.iter().any(|outcome| {
                    outcome.lifecycle_id == trace.lifecycle_id
                        && outcome.candle_index == trace.candle_index
                        && outcome.hold_quality.as_deref() == Some("POTENTIAL_LATE_REDUCTION")
                })
            })
            .map(|trace| trace.position_health.clone()),
    );
    let late_decision_ids = late_items
        .iter()
        .filter_map(|item| item.decision_id)
        .collect::<HashSet<_>>();
    let top_reason = top_label(
        events
            .iter()
            .filter(|event| {
                event
                    .decision_id
                    .is_some_and(|id| late_decision_ids.contains(&id))
            })
            .filter_map(|event| event.reason_code.clone()),
    );
    let summary = LifecycleObservatorySummary {
        total_lifecycles: lifecycles.len(),
        open_lifecycles: lifecycles.len() - closed,
        closed_lifecycles: closed,
        average_duration_minutes: average(
            &lifecycles
                .iter()
                .map(|item| item.duration_market_minutes as f64)
                .collect::<Vec<_>>(),
        ),
        average_pnl_pct: average(
            &lifecycles
                .iter()
                .map(|item| item.realized_pnl_pct)
                .collect::<Vec<_>>(),
        ),
        average_mfe_pct: average(
            &lifecycles
                .iter()
                .map(|item| item.mfe_pct)
                .collect::<Vec<_>>(),
        ),
        average_mae_pct: average(
            &lifecycles
                .iter()
                .map(|item| item.mae_pct)
                .collect::<Vec<_>>(),
        ),
        average_giveback_pct: average(
            &lifecycles
                .iter()
                .map(|item| item.giveback_pct)
                .collect::<Vec<_>>(),
        ),
        late_reduction_events: late_items.len(),
        average_response_delay_minutes: average(&response_delays),
    };
    let late_reduction = LateReductionSummary {
        total_hold_while_long: hold_long,
        potential_late_reductions: late_items.len(),
        rate_pct: if hold_long == 0 {
            0.0
        } else {
            round(late_items.len() as f64 / hold_long as f64 * 100.0)
        },
        average_forward_5: average(
            &late_items
                .iter()
                .filter_map(|item| item.forward_return_5)
                .collect::<Vec<_>>(),
        ),
        average_mae_5: average(
            &late_items
                .iter()
                .filter_map(|item| item.mae_5)
                .collect::<Vec<_>>(),
        ),
        average_response_delay_minutes: summary.average_response_delay_minutes,
        top_position_health: top_health,
        top_reason_code: top_reason,
    };
    Ok(LifecycleIntelligenceReport {
        run_id: run_id.clone(),
        experiment_id: experiment_id.into(),
        experiment_name: experiment.name,
        dataset_id: experiment.dataset_id,
        dataset_name: experiment.dataset_name,
        asset: experiment.asset,
        timeframe: experiment.timeframe,
        agent_id: AGENT_ID.into(),
        lifecycle_engine_version: lifecycle_version,
        deterioration_engine_version: deterioration_version,
        status,
        created_at,
        completed_at,
        summary,
        late_reduction,
        config,
        lifecycles,
        events,
        health_trace,
        outcomes,
        health_outcomes: aggregate_outcomes(connection, &run_id, "position_health")?,
        deterioration_outcomes: aggregate_outcomes(connection, &run_id, "deterioration_level")?,
        exit_reasons: aggregate_exit_reasons(connection, &run_id)?,
    })
}

fn top_label(values: impl Iterator<Item = String>) -> Option<String> {
    let mut counts = BTreeMap::<String, usize>::new();
    for value in values {
        *counts.entry(value).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(value, _)| value)
}

pub fn compare(
    connection: &mut Connection,
    development_id: &str,
    oos_id: &str,
) -> Result<LifecycleIntelligenceComparison, String> {
    if development_id == oos_id {
        return Err("DEV e OOS devem ser experimentos diferentes".into());
    }
    let development_identity = identity(connection, development_id)?;
    let oos_identity = identity(connection, oos_id)?;
    market_identity::validate_same_market_identity(
        &development_identity.asset,
        &development_identity.timeframe,
        &oos_identity.asset,
        &oos_identity.timeframe,
    )?;
    if development_identity.end_at >= oos_identity.start_at {
        return Err("O dataset OOS deve começar depois do fim do dataset DEV".into());
    }
    let development =
        get(connection, development_id).or_else(|_| generate(connection, development_id))?;
    let oos = get(connection, oos_id).or_else(|_| generate(connection, oos_id))?;
    let metric = |label: &str, dev: f64, oos_value: f64| LifecycleComparisonMetric {
        label: label.into(),
        development_value: dev,
        out_of_sample_value: oos_value,
    };
    let metrics = vec![
        metric(
            "LIFECYCLES",
            development.summary.total_lifecycles as f64,
            oos.summary.total_lifecycles as f64,
        ),
        metric(
            "AVG DURATION",
            development.summary.average_duration_minutes,
            oos.summary.average_duration_minutes,
        ),
        metric(
            "AVG MFE",
            development.summary.average_mfe_pct,
            oos.summary.average_mfe_pct,
        ),
        metric(
            "AVG MAE",
            development.summary.average_mae_pct,
            oos.summary.average_mae_pct,
        ),
        metric(
            "AVG GIVEBACK",
            development.summary.average_giveback_pct,
            oos.summary.average_giveback_pct,
        ),
        metric(
            "LATE REDUCTION RATE",
            development.late_reduction.rate_pct,
            oos.late_reduction.rate_pct,
        ),
        metric(
            "AVG RESPONSE DELAY",
            development.summary.average_response_delay_minutes,
            oos.summary.average_response_delay_minutes,
        ),
    ];
    let levels = ["NONE", "LOW", "MODERATE", "HIGH", "CRITICAL"];
    let count = |report: &LifecycleIntelligenceReport, level: &str| {
        report
            .health_trace
            .iter()
            .filter(|trace| trace.deterioration_level == level)
            .count()
    };
    let deterioration_distribution = levels
        .into_iter()
        .map(|level| LifecycleDistributionComparison {
            level: level.into(),
            development_count: count(&development, level),
            out_of_sample_count: count(&oos, level),
        })
        .collect();
    Ok(LifecycleIntelligenceComparison {
        development_experiment_id: development_id.into(),
        out_of_sample_experiment_id: oos_id.into(),
        asset: development.asset,
        timeframe: development.timeframe,
        metrics,
        deterioration_distribution,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_label_is_deterministic() {
        assert_eq!(
            top_label(["HEALTHY".into(), "WEAKENING".into(), "HEALTHY".into()].into_iter()),
            Some("HEALTHY".into())
        );
    }

    #[test]
    fn future_outcomes_are_structurally_separate_from_available_at_t_trace() {
        let trace = LifecycleHealthTrace {
            lifecycle_id: "lifecycle-1".into(),
            candle_index: 1,
            timestamp: "2026-01-01T10:00:00Z".into(),
            position_age_candles: 1,
            position_age_market_minutes: 15,
            market_price: 100.0,
            exposure_pct: 25.0,
            return_since_entry_pct: 0.0,
            mfe_since_entry_pct: 0.0,
            mae_since_entry_pct: 0.0,
            distance_from_entry_atr: None,
            distance_from_mfe_pct: 0.0,
            giveback_from_mfe_pct: 0.0,
            giveback_relative_pct: None,
            vwap_change_since_entry: None,
            ema_spread_change_since_entry: None,
            rsi_change_since_entry: None,
            trend_changed: false,
            momentum_changed: false,
            trend_component: 0.0,
            momentum_component: 0.0,
            giveback_component: 0.0,
            vwap_component: 0.0,
            volatility_component: 0.0,
            time_component: 0.0,
            deterioration_score: 0.0,
            deterioration_level: "NONE".into(),
            position_health: "HEALTHY".into(),
            available_at_t: true,
        };
        let outcome = LifecycleOutcome {
            lifecycle_id: trace.lifecycle_id.clone(),
            candle_index: trace.candle_index,
            decision_id: None,
            hold_quality: None,
            forward_return_1: Some(-1.0),
            forward_return_5: Some(-2.0),
            mfe_5: Some(0.1),
            mae_5: Some(-2.5),
            missed_reduction_window: true,
            post_decision_only: true,
        };
        assert!(trace.available_at_t);
        assert!(outcome.post_decision_only);
    }
}
