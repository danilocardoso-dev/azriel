use super::{
    market_ai,
    market_hold_diagnostics::{
        aggregate_by, calibration_candidates, classify, confidence_bucket, conflict_signature,
        ema_spread_bucket, evidence_level, post_decision_outcome, relative_volume_bucket,
        rsi_bucket, summarize_quality, vwap_distance_bucket, HoldAggregateRow,
        HoldCandidateEvidence, HoldComparisonMetric, HoldDiagnosticItem, HoldDiagnosticsComparison,
        HoldDiagnosticsConfig, HoldDiagnosticsReport, HoldPositionState, HoldPostDecisionOutcome,
        HoldTriggerAnalysisRow, OutcomeCandle, ENGINE_VERSION,
    },
    market_identity,
};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    time::{SystemTime, UNIX_EPOCH},
};

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

fn new_run_id() -> String {
    format!(
        "hold-diagnostic-{}",
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
        .ok_or_else(|| "Experimento não encontrado".to_string())
}

fn json_number(snapshot: &Value, pointer: &str) -> Option<f64> {
    snapshot.pointer(pointer).and_then(Value::as_f64)
}

fn json_text(snapshot: &Value, pointer: &str) -> String {
    snapshot
        .pointer(pointer)
        .and_then(Value::as_str)
        .unwrap_or("UNKNOWN")
        .to_string()
}

fn quality_context(position: HoldPositionState, quality: &str) -> String {
    match (position, quality) {
        (HoldPositionState::Flat, "GOOD_HOLD") => "GOOD AVOIDANCE — queda posterior enquanto FLAT",
        (HoldPositionState::Flat, "POTENTIAL_MISSED_OPPORTUNITY") => {
            "Oportunidade potencial perdida após HOLD enquanto FLAT"
        }
        (HoldPositionState::Long, "GOOD_HOLD") => "Continuidade favorável após HOLD enquanto LONG",
        (HoldPositionState::Long, "POTENTIAL_LATE_REDUCTION") => {
            "Deterioração potencial sem redução após HOLD enquanto LONG"
        }
        _ => "Movimento posterior sem evidência direcional suficiente",
    }
    .to_string()
}

fn item_from_row(
    row: &Row<'_>,
    candles: &[OutcomeCandle],
    config: &HoldDiagnosticsConfig,
) -> Result<HoldDiagnosticItem, String> {
    let decision_id: i64 = row.get(0).map_err(|error| error.to_string())?;
    let candle_index: usize = row.get(1).map_err(|error| error.to_string())?;
    let timestamp: String = row.get(2).map_err(|error| error.to_string())?;
    let observed_price: f64 = row.get(3).map_err(|error| error.to_string())?;
    let exposure_before: f64 = row.get(4).map_err(|error| error.to_string())?;
    let confidence: Option<f64> = row.get(5).map_err(|error| error.to_string())?;
    let reason: String = row.get(6).map_err(|error| error.to_string())?;
    let reason_code: Option<String> = row.get(7).map_err(|error| error.to_string())?;
    let raw_snapshot: String = row.get(8).map_err(|error| error.to_string())?;
    let trigger_reason: Option<String> = row.get(9).map_err(|error| error.to_string())?;
    let snapshot: Value = serde_json::from_str(&raw_snapshot)
        .map_err(|error| format!("Contexto intraday inválido na decisão {decision_id}: {error}"))?;
    if json_text(&snapshot, "/contextVersion") != market_ai::CONTEXT_INTRADAY_V1_VERSION {
        return Err(format!(
            "Decisão {decision_id} não possui contexto {}",
            market_ai::CONTEXT_INTRADAY_V1_VERSION
        ));
    }
    let exposure =
        json_number(&snapshot, "/position/currentExposurePct").unwrap_or(exposure_before);
    let position = HoldPositionState::from_exposure(exposure);
    let outcome = post_decision_outcome(candles, candle_index, observed_price);
    let quality = classify(position, &outcome, config).as_str().to_string();
    let code = reason_code.unwrap_or_else(|| "UNSPECIFIED".into());
    let trend = json_text(&snapshot, "/marketState/trend");
    let momentum = json_text(&snapshot, "/marketState/momentum");
    let volume = json_text(&snapshot, "/marketState/volume");
    let vwap_position = json_text(&snapshot, "/marketState/vwapPosition");
    let rsi = json_number(&snapshot, "/indicators/rsi14");
    let relative_volume = json_number(&snapshot, "/indicators/relativeVolume");
    let vwap_distance = json_number(&snapshot, "/indicators/distanceFromVwapAtr");
    let ema_spread = json_number(&snapshot, "/indicators/emaSpreadAtr");
    Ok(HoldDiagnosticItem {
        decision_id,
        candle_index,
        timestamp,
        session_id: json_text(&snapshot, "/market/sessionId"),
        session_phase: json_text(&snapshot, "/market/sessionPhase"),
        position_state: position.as_str().into(),
        current_exposure_pct: exposure,
        confidence,
        confidence_bucket: confidence_bucket(confidence).into(),
        reason_code: code.clone(),
        reason,
        price: observed_price,
        ema_9: json_number(&snapshot, "/indicators/ema9"),
        ema_21: json_number(&snapshot, "/indicators/ema21"),
        vwap: json_number(&snapshot, "/indicators/vwap"),
        rsi_14: rsi,
        atr_14: json_number(&snapshot, "/indicators/atr14"),
        relative_volume,
        distance_from_vwap_atr: vwap_distance,
        ema_spread_atr: ema_spread,
        trend: trend.clone(),
        momentum: momentum.clone(),
        volatility: json_text(&snapshot, "/marketState/volatility"),
        location: json_text(&snapshot, "/marketState/location"),
        vwap_position: vwap_position.clone(),
        trigger_reason: trigger_reason.unwrap_or_else(|| "UNKNOWN".into()),
        conflict_signature: conflict_signature(&code, &trend, &momentum, &vwap_position, &volume),
        rsi_bucket: rsi_bucket(rsi).into(),
        relative_volume_bucket: relative_volume_bucket(relative_volume).into(),
        vwap_distance_bucket: vwap_distance_bucket(vwap_distance).into(),
        ema_spread_bucket: ema_spread_bucket(ema_spread).into(),
        outcome,
        quality_context: quality_context(position, &quality),
        quality,
    })
}

fn insert_item(
    connection: &Connection,
    run_id: &str,
    item: &HoldDiagnosticItem,
) -> Result<(), String> {
    let outcome = &item.outcome;
    connection
        .execute(
            "INSERT INTO market_hold_diagnostics(
          run_id,decision_id,candle_index,timestamp,session_id,session_phase,position_state,
          current_exposure_pct,confidence,confidence_bucket,reason_code,reason,price,ema_9,ema_21,
          vwap,rsi_14,atr_14,relative_volume,distance_from_vwap_atr,ema_spread_atr,trend,momentum,
          volatility,location,vwap_position,trigger_reason,conflict_signature,rsi_bucket,
          relative_volume_bucket,vwap_distance_bucket,ema_spread_bucket,forward_1,forward_5,
          forward_10,mfe_1,mfe_5,mfe_10,mae_1,mae_5,mae_10,quality,quality_context)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,
                ?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38,
                ?39,?40,?41,?42,?43)",
            params![
                run_id,
                item.decision_id,
                item.candle_index,
                item.timestamp,
                item.session_id,
                item.session_phase,
                item.position_state,
                item.current_exposure_pct,
                item.confidence,
                item.confidence_bucket,
                item.reason_code,
                item.reason,
                item.price,
                item.ema_9,
                item.ema_21,
                item.vwap,
                item.rsi_14,
                item.atr_14,
                item.relative_volume,
                item.distance_from_vwap_atr,
                item.ema_spread_atr,
                item.trend,
                item.momentum,
                item.volatility,
                item.location,
                item.vwap_position,
                item.trigger_reason,
                item.conflict_signature,
                item.rsi_bucket,
                item.relative_volume_bucket,
                item.vwap_distance_bucket,
                item.ema_spread_bucket,
                outcome.forward_1,
                outcome.forward_5,
                outcome.forward_10,
                outcome.mfe_1,
                outcome.mfe_5,
                outcome.mfe_10,
                outcome.mae_1,
                outcome.mae_5,
                outcome.mae_10,
                item.quality,
                item.quality_context
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub fn generate(
    connection: &mut Connection,
    experiment_id: &str,
) -> Result<HoldDiagnosticsReport, String> {
    let experiment = identity(connection, experiment_id)?;
    if experiment.status != "completed" {
        return Err("O diagnóstico exige um experimento concluído".into());
    }
    if experiment.timeframe != "15M" {
        return Err("HOLD Diagnostics v1 aceita somente experimentos 15M".into());
    }
    let prompt: Option<String> = connection.query_row(
        "SELECT a.prompt_version FROM market_decisions d JOIN market_ai_decisions a ON a.decision_id=d.id
         WHERE d.experiment_id=?1 AND d.agent_id=?2 LIMIT 1",
        params![experiment_id, market_ai::AI_INTRADAY_V1_ID], |row| row.get(0),
    ).optional().map_err(|error| error.to_string())?;
    if prompt.as_deref() != Some(market_ai::PROMPT_INTRADAY_V1_VERSION) {
        return Err("O experimento não contém decisões congeladas do AI Intraday V1".into());
    }
    let candles = {
        let mut statement = connection.prepare(
            "SELECT high,low,close FROM market_candles WHERE dataset_id=?1 ORDER BY candle_index"
        ).map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([&experiment.dataset_id], |row| {
                Ok(OutcomeCandle {
                    high: row.get(0)?,
                    low: row.get(1)?,
                    close: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?
    };
    let total_ai_calls: usize = connection.query_row(
        "SELECT COUNT(*) FROM market_decisions d JOIN market_ai_decisions a ON a.decision_id=d.id
         WHERE d.experiment_id=?1 AND d.agent_id=?2 AND a.call_status='VALID'",
        params![experiment_id, market_ai::AI_INTRADAY_V1_ID], |row| row.get(0),
    ).map_err(|error| error.to_string())?;
    let config = HoldDiagnosticsConfig::default();
    let items = {
        let mut statement = connection.prepare(
            "SELECT d.id,d.candle_index,d.timestamp,d.observed_price,d.exposure_before,d.confidence,
                    d.reasoning,a.reason_code,c.input_snapshot_json,t.trigger_reason
             FROM market_decisions d
             JOIN market_ai_decisions a ON a.decision_id=d.id
             JOIN market_ai_decision_context c ON c.decision_id=d.id
             LEFT JOIN market_decision_trigger_audits t ON t.experiment_id=d.experiment_id
                  AND t.agent_id=d.agent_id AND t.candle_index=d.candle_index
             WHERE d.experiment_id=?1 AND d.agent_id=?2 AND a.call_status='VALID' AND a.intent='HOLD'
             ORDER BY d.candle_index"
        ).map_err(|error| error.to_string())?;
        let mut rows = statement
            .query(params![experiment_id, market_ai::AI_INTRADAY_V1_ID])
            .map_err(|error| error.to_string())?;
        let mut result = Vec::new();
        while let Some(row) = rows.next().map_err(|error| error.to_string())? {
            result.push(item_from_row(row, &candles, &config)?);
        }
        result
    };
    if total_ai_calls > 0 && items.is_empty() {
        return Err("As chamadas persistidas não possuem HOLDs válidos com contexto intraday; nenhuma nova chamada ao Ollama foi feita".into());
    }
    let run_id = connection.query_row(
        "SELECT id FROM market_hold_diagnostic_runs WHERE experiment_id=?1 AND agent_id=?2 AND engine_version=?3",
        params![experiment_id, market_ai::AI_INTRADAY_V1_ID, ENGINE_VERSION], |row| row.get::<_,String>(0),
    ).optional().map_err(|error| error.to_string())?.unwrap_or_else(new_run_id);
    let config_json = serde_json::to_string(&config).map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    transaction.execute(
        "INSERT INTO market_hold_diagnostic_runs(id,experiment_id,agent_id,engine_version,config_json,status,total_ai_calls,total_holds,error,completed_at)
         VALUES(?1,?2,?3,?4,?5,'running',?6,?7,NULL,NULL)
         ON CONFLICT(experiment_id,agent_id,engine_version) DO UPDATE SET config_json=excluded.config_json,status='running',total_ai_calls=excluded.total_ai_calls,total_holds=excluded.total_holds,error=NULL,completed_at=NULL",
        params![run_id,experiment_id,market_ai::AI_INTRADAY_V1_ID,ENGINE_VERSION,config_json,total_ai_calls,items.len()],
    ).map_err(|error| error.to_string())?;
    transaction
        .execute(
            "DELETE FROM market_hold_diagnostics WHERE run_id=?1",
            [&run_id],
        )
        .map_err(|error| error.to_string())?;
    for item in &items {
        insert_item(&transaction, &run_id, item)?;
    }
    transaction.execute("UPDATE market_hold_diagnostic_runs SET status='completed',completed_at=CURRENT_TIMESTAMP WHERE id=?1", [&run_id]).map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    get(connection, experiment_id)
}

fn map_persisted_item(row: &Row<'_>) -> rusqlite::Result<HoldDiagnosticItem> {
    Ok(HoldDiagnosticItem {
        decision_id: row.get(0)?,
        candle_index: row.get(1)?,
        timestamp: row.get(2)?,
        session_id: row.get(3)?,
        session_phase: row.get(4)?,
        position_state: row.get(5)?,
        current_exposure_pct: row.get(6)?,
        confidence: row.get(7)?,
        confidence_bucket: row.get(8)?,
        reason_code: row.get(9)?,
        reason: row.get(10)?,
        price: row.get(11)?,
        ema_9: row.get(12)?,
        ema_21: row.get(13)?,
        vwap: row.get(14)?,
        rsi_14: row.get(15)?,
        atr_14: row.get(16)?,
        relative_volume: row.get(17)?,
        distance_from_vwap_atr: row.get(18)?,
        ema_spread_atr: row.get(19)?,
        trend: row.get(20)?,
        momentum: row.get(21)?,
        volatility: row.get(22)?,
        location: row.get(23)?,
        vwap_position: row.get(24)?,
        trigger_reason: row.get(25)?,
        conflict_signature: row.get(26)?,
        rsi_bucket: row.get(27)?,
        relative_volume_bucket: row.get(28)?,
        vwap_distance_bucket: row.get(29)?,
        ema_spread_bucket: row.get(30)?,
        outcome: HoldPostDecisionOutcome {
            forward_1: row.get(31)?,
            forward_5: row.get(32)?,
            forward_10: row.get(33)?,
            mfe_1: row.get(34)?,
            mfe_5: row.get(35)?,
            mfe_10: row.get(36)?,
            mae_1: row.get(37)?,
            mae_5: row.get(38)?,
            mae_10: row.get(39)?,
        },
        quality: row.get(40)?,
        quality_context: row.get(41)?,
    })
}

fn feature_aggregates(items: &[HoldDiagnosticItem]) -> Vec<HoldAggregateRow> {
    let selectors: [(&str, fn(&HoldDiagnosticItem) -> String); 4] = [
        ("RSI", |item| item.rsi_bucket.clone()),
        ("RELATIVE_VOLUME", |item| {
            item.relative_volume_bucket.clone()
        }),
        ("VWAP_DISTANCE_ATR", |item| {
            item.vwap_distance_bucket.clone()
        }),
        ("EMA_SPREAD_ATR", |item| item.ema_spread_bucket.clone()),
    ];
    selectors
        .into_iter()
        .flat_map(|(label, selector)| {
            aggregate_by(items, selector)
                .into_iter()
                .map(move |mut row| {
                    row.key = format!("{label} / {}", row.key);
                    row
                })
        })
        .collect()
}

fn trigger_analysis(
    connection: &Connection,
    experiment_id: &str,
    hold_rows: &[HoldAggregateRow],
) -> Result<Vec<HoldTriggerAnalysisRow>, String> {
    let mut statement = connection
        .prepare(
            "SELECT COALESCE(t.trigger_reason,'UNKNOWN'),COUNT(*),
                SUM(CASE WHEN a.intent='HOLD' THEN 1 ELSE 0 END),
                SUM(CASE WHEN a.intent='ENTER' THEN 1 ELSE 0 END)
         FROM market_decisions d JOIN market_ai_decisions a ON a.decision_id=d.id
         LEFT JOIN market_decision_trigger_audits t ON t.experiment_id=d.experiment_id
              AND t.agent_id=d.agent_id AND t.candle_index=d.candle_index
         WHERE d.experiment_id=?1 AND d.agent_id=?2 AND a.call_status='VALID'
         GROUP BY COALESCE(t.trigger_reason,'UNKNOWN') ORDER BY COUNT(*) DESC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![experiment_id, market_ai::AI_INTRADAY_V1_ID],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, usize>(1)?,
                    row.get::<_, usize>(2)?,
                    row.get::<_, usize>(3)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?;
    rows.map(|row| {
        let (trigger, call_count, hold_count, enter_count) =
            row.map_err(|error| error.to_string())?;
        let hold = hold_rows.iter().find(|item| item.key == trigger);
        Ok(HoldTriggerAnalysisRow {
            trigger,
            call_count,
            hold_count,
            enter_count,
            hold_rate_pct: if call_count == 0 {
                0.0
            } else {
                hold_count as f64 / call_count as f64 * 100.0
            },
            average_forward_5_after_hold: hold.and_then(|item| item.average_forward_5),
            missed_opportunity_rate_pct: hold
                .map(|item| item.missed_opportunity_rate_pct)
                .unwrap_or(0.0),
        })
    })
    .collect()
}

pub fn get(connection: &Connection, experiment_id: &str) -> Result<HoldDiagnosticsReport, String> {
    let experiment = identity(connection, experiment_id)?;
    let metadata = connection.query_row(
        "SELECT id,agent_id,engine_version,status,created_at,completed_at,total_ai_calls,total_holds,config_json
         FROM market_hold_diagnostic_runs WHERE experiment_id=?1 AND agent_id=?2 AND engine_version=?3",
        params![experiment_id,market_ai::AI_INTRADAY_V1_ID,ENGINE_VERSION],
        |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?,row.get::<_,String>(4)?,row.get::<_,Option<String>>(5)?,row.get::<_,usize>(6)?,row.get::<_,usize>(7)?,row.get::<_,String>(8)?)),
    ).optional().map_err(|error| error.to_string())?.ok_or_else(|| "Diagnóstico HOLD ainda não foi gerado para este experimento".to_string())?;
    let mut statement = connection.prepare(
        "SELECT decision_id,candle_index,timestamp,session_id,session_phase,position_state,current_exposure_pct,
                confidence,confidence_bucket,reason_code,reason,price,ema_9,ema_21,vwap,rsi_14,atr_14,relative_volume,
                distance_from_vwap_atr,ema_spread_atr,trend,momentum,volatility,location,vwap_position,trigger_reason,
                conflict_signature,rsi_bucket,relative_volume_bucket,vwap_distance_bucket,ema_spread_bucket,
                forward_1,forward_5,forward_10,mfe_1,mfe_5,mfe_10,mae_1,mae_5,mae_10,quality,quality_context
         FROM market_hold_diagnostics WHERE run_id=?1 ORDER BY candle_index"
    ).map_err(|error| error.to_string())?;
    let items = statement
        .query_map([&metadata.0], map_persisted_item)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let config: HoldDiagnosticsConfig =
        serde_json::from_str(&metadata.8).map_err(|error| error.to_string())?;
    let reasons = aggregate_by(&items, |item| item.reason_code.clone());
    let triggers = aggregate_by(&items, |item| item.trigger_reason.clone());
    let trigger_analysis = trigger_analysis(connection, experiment_id, &triggers)?;
    let hold_rate_pct = if metadata.6 == 0 {
        0.0
    } else {
        metadata.7 as f64 / metadata.6 as f64 * 100.0
    };
    Ok(HoldDiagnosticsReport {
        run_id: metadata.0,
        experiment_id: experiment_id.into(),
        experiment_name: experiment.name,
        dataset_id: experiment.dataset_id,
        dataset_name: experiment.dataset_name,
        asset: experiment.asset,
        timeframe: experiment.timeframe,
        agent_id: metadata.1,
        engine_version: metadata.2,
        status: metadata.3,
        created_at: metadata.4,
        completed_at: metadata.5,
        total_ai_calls: metadata.6,
        total_holds: metadata.7,
        hold_rate_pct,
        hold_concentration_warning: hold_rate_pct > config.concentration_warning_pct,
        flat_count: items
            .iter()
            .filter(|item| item.position_state == "FLAT")
            .count(),
        long_count: items
            .iter()
            .filter(|item| item.position_state == "LONG")
            .count(),
        quality: summarize_quality(&items),
        quality_matrix: aggregate_by(&items, |item| item.quality.clone()),
        confidence: aggregate_by(&items, |item| item.confidence_bucket.clone()),
        session_phases: aggregate_by(&items, |item| item.session_phase.clone()),
        triggers,
        trigger_analysis,
        feature_buckets: feature_aggregates(&items),
        conflicts: aggregate_by(
            &items
                .iter()
                .filter(|item| item.conflict_signature.is_some())
                .cloned()
                .collect::<Vec<_>>(),
            |item| item.conflict_signature.clone().unwrap_or_default(),
        ),
        calibration_candidates: calibration_candidates(&reasons, &config),
        reasons,
        config,
        items,
    })
}

fn metric(label: &str, dev: f64, oos: f64) -> HoldComparisonMetric {
    HoldComparisonMetric {
        label: label.into(),
        development_value: dev,
        out_of_sample_value: oos,
    }
}

fn average_metric(
    items: &[HoldDiagnosticItem],
    select: fn(&HoldDiagnosticItem) -> Option<f64>,
) -> f64 {
    let values = items.iter().filter_map(select).collect::<Vec<_>>();
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f64>() / values.len() as f64
    }
}

pub fn compare(
    connection: &Connection,
    development_id: &str,
    oos_id: &str,
) -> Result<HoldDiagnosticsComparison, String> {
    if development_id == oos_id {
        return Err("DEV e OOS devem ser experimentos diferentes".into());
    }
    let dev_identity = identity(connection, development_id)?;
    let oos_identity = identity(connection, oos_id)?;
    market_identity::validate_same_market_identity(
        &dev_identity.asset,
        &dev_identity.timeframe,
        &oos_identity.asset,
        &oos_identity.timeframe,
    )?;
    if dev_identity.end_at >= oos_identity.start_at {
        return Err("O dataset OOS deve começar depois do fim do dataset DEV".into());
    }
    let dev = get(connection, development_id)?;
    let oos = get(connection, oos_id)?;
    let rate = |count: usize, total: usize| {
        if total == 0 {
            0.0
        } else {
            count as f64 / total as f64 * 100.0
        }
    };
    let metrics = vec![
        metric("HOLD RATE", dev.hold_rate_pct, oos.hold_rate_pct),
        metric(
            "FLAT HOLD",
            rate(dev.flat_count, dev.total_holds),
            rate(oos.flat_count, oos.total_holds),
        ),
        metric(
            "LONG HOLD",
            rate(dev.long_count, dev.total_holds),
            rate(oos.long_count, oos.total_holds),
        ),
        metric(
            "GOOD HOLD",
            rate(dev.quality.good_hold_count, dev.total_holds),
            rate(oos.quality.good_hold_count, oos.total_holds),
        ),
        metric(
            "MISSED OPPORTUNITY",
            rate(dev.quality.missed_opportunity_count, dev.total_holds),
            rate(oos.quality.missed_opportunity_count, oos.total_holds),
        ),
        metric(
            "LATE REDUCTION",
            rate(dev.quality.late_reduction_count, dev.total_holds),
            rate(oos.quality.late_reduction_count, oos.total_holds),
        ),
        metric(
            "INCONCLUSIVE",
            rate(dev.quality.inconclusive_count, dev.total_holds),
            rate(oos.quality.inconclusive_count, oos.total_holds),
        ),
        metric(
            "AVG FWD1",
            average_metric(&dev.items, |item| item.outcome.forward_1),
            average_metric(&oos.items, |item| item.outcome.forward_1),
        ),
        metric(
            "AVG FWD5",
            average_metric(&dev.items, |item| item.outcome.forward_5),
            average_metric(&oos.items, |item| item.outcome.forward_5),
        ),
        metric(
            "AVG FWD10",
            average_metric(&dev.items, |item| item.outcome.forward_10),
            average_metric(&oos.items, |item| item.outcome.forward_10),
        ),
        metric(
            "AVG MFE5",
            average_metric(&dev.items, |item| item.outcome.mfe_5),
            average_metric(&oos.items, |item| item.outcome.mfe_5),
        ),
        metric(
            "AVG MAE5",
            average_metric(&dev.items, |item| item.outcome.mae_5),
            average_metric(&oos.items, |item| item.outcome.mae_5),
        ),
    ];
    let mut keys = BTreeSet::new();
    for item in dev
        .calibration_candidates
        .iter()
        .chain(&oos.calibration_candidates)
    {
        keys.insert((item.reason_code.clone(), item.issue.clone()));
    }
    let candidate_evidence = keys
        .into_iter()
        .map(|(reason_code, issue)| {
            let development = dev
                .calibration_candidates
                .iter()
                .find(|item| item.reason_code == reason_code && item.issue == issue)
                .cloned();
            let out_of_sample = oos
                .calibration_candidates
                .iter()
                .find(|item| item.reason_code == reason_code && item.issue == issue)
                .cloned();
            HoldCandidateEvidence {
                reason_code,
                issue,
                evidence: evidence_level(development.as_ref(), out_of_sample.as_ref()),
                development,
                out_of_sample,
            }
        })
        .collect();
    let top_reason = |report: &HoldDiagnosticsReport| {
        report
            .reasons
            .iter()
            .max_by_key(|item| item.count)
            .map(|item| item.key.clone())
    };
    let top_candidate = |report: &HoldDiagnosticsReport| {
        report
            .calibration_candidates
            .first()
            .map(|item| format!("{} / {}", item.reason_code, item.issue))
    };
    Ok(HoldDiagnosticsComparison {
        development_experiment_id: development_id.into(),
        out_of_sample_experiment_id: oos_id.into(),
        asset: dev_identity.asset,
        timeframe: dev_identity.timeframe,
        metrics,
        development_top_reason: top_reason(&dev),
        out_of_sample_top_reason: top_reason(&oos),
        development_top_candidate: top_candidate(&dev),
        out_of_sample_top_candidate: top_candidate(&oos),
        candidate_evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    #[test]
    fn snapshot_helpers_read_frozen_camel_case_contract() {
        let snapshot = serde_json::json!({"contextVersion":"MARKET_AI_INTRADAY_CONTEXT_V1","position":{"currentExposurePct":25.0},"indicators":{"rsi14":51.0}});
        assert_eq!(
            json_number(&snapshot, "/position/currentExposurePct"),
            Some(25.0)
        );
        assert_eq!(json_number(&snapshot, "/indicators/rsi14"), Some(51.0));
        assert_eq!(
            json_text(&snapshot, "/contextVersion"),
            market_ai::CONTEXT_INTRADAY_V1_VERSION
        );
    }

    #[test]
    fn generates_and_reuses_persisted_hold_diagnostics_without_new_ai_calls() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,currency,start_at,end_at,candle_count,fingerprint,source_path,market,timezone,session_type) VALUES('dataset-dev','DEV','AAPL','15M','USD','2026-01-01T14:30:00Z','2026-01-01T17:00:00Z',11,'diagnostic-fixture','fixture.csv','US_EQUITIES','America/New_York','REGULAR')", []).unwrap();
        for candle_index in 0..11 {
            let close = 100.0 + candle_index as f64;
            connection.execute("INSERT INTO market_candles(dataset_id,candle_index,timestamp,open,high,low,close,volume,timestamp_utc,session_id,session_state) VALUES('dataset-dev',?1,?2,?3,?4,?5,?6,1000,?2,'2026-01-01','REGULAR')", params![candle_index,format!("2026-01-01T{:02}:00:00Z",candle_index),close,close+0.5,close-0.5,close]).unwrap();
        }
        connection.execute("INSERT INTO market_experiments(id,name,dataset_id,asset,timeframe,currency,initial_capital,risk_profile_id,random_seed,fee_pct,slippage_pct,config_json,status,started_at,completed_at,market,timezone,session_type) VALUES('exp-dev','DEV HOLD','dataset-dev','AAPL','15M','USD',10000,'balanced-v1',42,0.1,0.05,'{}','completed',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP,'US_EQUITIES','America/New_York','REGULAR')", []).unwrap();
        connection.execute("INSERT INTO market_decisions(experiment_id,agent_id,candle_index,timestamp,observed_price,action,desired_position_pct,confidence,reasoning,cash_before,equity_before,exposure_before) VALUES('exp-dev','ai-intraday-v1',0,'2026-01-01T00:00:00Z',100,'HOLD',0,0.61,'conflito',10000,10000,0)", []).unwrap();
        let decision_id = connection.last_insert_rowid();
        connection.execute("INSERT INTO market_ai_decisions(decision_id,call_status,provider,model,prompt_version,latency_ms,attempts,fallback_used,reason_code,intent) VALUES(?1,'VALID','ollama','qwen2.5:3b','MARKET_AI_INTRADAY_V1',10,1,0,'SIGNAL_CONFLICT','HOLD')", [decision_id]).unwrap();
        let snapshot = serde_json::json!({
            "contextVersion":"MARKET_AI_INTRADAY_CONTEXT_V1",
            "market":{"sessionId":"2026-01-01","sessionPhase":"MORNING"},
            "position":{"currentExposurePct":0.0},
            "indicators":{"ema9":101.0,"ema21":100.0,"vwap":100.0,"rsi14":55.0,"atr14":1.0,"relativeVolume":1.2,"distanceFromVwapAtr":0.0,"emaSpreadAtr":1.0},
            "marketState":{"trend":"BULLISH","momentum":"BEARISH","volume":"NORMAL","volatility":"NORMAL","location":"MID_RANGE","vwapPosition":"NEAR"}
        }).to_string();
        connection.execute("INSERT INTO market_ai_decision_context(decision_id,input_snapshot_json) VALUES(?1,?2)", params![decision_id,snapshot]).unwrap();
        connection.execute("INSERT INTO market_decision_trigger_audits(experiment_id,agent_id,candle_index,timestamp_utc,session_id,should_evaluate,trigger_reason,cooldown_remaining,call_index) VALUES('exp-dev','ai-intraday-v1',0,'2026-01-01T00:00:00Z','2026-01-01',1,'SESSION_OPEN',0,1)", []).unwrap();
        let ai_rows_before: usize = connection
            .query_row("SELECT COUNT(*) FROM market_ai_decisions", [], |row| {
                row.get(0)
            })
            .unwrap();

        let first = generate(&mut connection, "exp-dev").unwrap();
        let second = generate(&mut connection, "exp-dev").unwrap();

        assert_eq!(first.total_ai_calls, 1);
        assert_eq!(first.total_holds, 1);
        assert_eq!(first.items[0].quality, "POTENTIAL_MISSED_OPPORTUNITY");
        assert_eq!(first.items[0].trigger_reason, "SESSION_OPEN");
        assert_eq!(first.items[0].outcome.forward_5, Some(5.0));
        assert_eq!(first.run_id, second.run_id);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_hold_diagnostics", [], |row| {
                    row.get::<_, usize>(0)
                })
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_ai_decisions", [], |row| row
                    .get::<_, usize>(
                    0
                ))
                .unwrap(),
            ai_rows_before
        );
    }
}
