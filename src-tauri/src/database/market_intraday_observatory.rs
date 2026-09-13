use super::{market_intraday, market_models::*};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use std::collections::BTreeMap;

fn parse_json(value: String) -> rusqlite::Result<serde_json::Value> {
    serde_json::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            value.len(),
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

pub fn persist_features(
    tx: &Transaction<'_>,
    experiment_id: &str,
    features: &[market_intraday::IntradayFeatures],
) -> Result<(), String> {
    for feature in features {
        tx.execute(
            "INSERT INTO market_intraday_feature_traces(experiment_id,candle_index,timestamp_utc,session_id,session_phase,session_progress,ready,features_json,engine_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![experiment_id,feature.candle_index,feature.timestamp_utc,feature.session_id,feature.session_phase.as_str(),feature.session_progress,feature.ready,serde_json::to_string(feature).map_err(|error|error.to_string())?,market_intraday::FEATURE_ENGINE_VERSION],
        ).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[derive(Default)]
struct StyleMetric {
    style: String,
    trend_entries: usize,
    ema_cross_entries: usize,
    breakout_attempts: usize,
    breakout_entries: usize,
    failed_breakouts: usize,
    vwap_deviation_events: usize,
    mean_reversion_entries: usize,
    successful_reversions: usize,
}

pub fn persist_aggregates(
    tx: &Transaction<'_>,
    experiment_id: &str,
    initial_capital: f64,
) -> Result<(), String> {
    let mut metrics: BTreeMap<String, StyleMetric> = BTreeMap::new();
    let mut statement = tx.prepare("SELECT d.agent_id,s.style,s.intent,s.reason_code FROM market_intraday_strategy_decisions s JOIN market_decisions d ON d.id=s.decision_id WHERE d.experiment_id=?1 ORDER BY d.candle_index").map_err(|error|error.to_string())?;
    let rows = statement.query_map([experiment_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?,row.get::<_,String>(3)?))).map_err(|error|error.to_string())?;
    for row in rows {
        let (agent, style, intent, reason) = row.map_err(|error| error.to_string())?;
        let metric = metrics.entry(agent).or_default();
        metric.style = style;
        if intent == "ENTER" {
            if metric.style == "TREND" { metric.trend_entries += 1; }
            if metric.style == "MOMENTUM" { metric.breakout_entries += 1; }
            if metric.style == "MEAN_REVERSION" { metric.mean_reversion_entries += 1; }
        }
        if reason == "EMA_CROSS_BULLISH" { metric.ema_cross_entries += 1; }
        if reason.starts_with("BREAKOUT_") { metric.breakout_attempts += 1; }
        if reason == "BREAKOUT_WEAK_VOLUME" || reason == "MOMENTUM_FADE" { metric.failed_breakouts += 1; }
        if reason == "VWAP_DEVIATION" { metric.vwap_deviation_events += 1; }
        if reason == "MEAN_REVERSION_CONFIRMED" && intent == "EXIT" { metric.successful_reversions += 1; }
    }
    drop(statement);
    for (agent, metric) in metrics {
        let average_duration: f64 = tx.query_row("SELECT COALESCE(AVG(holding_market_minutes),0) FROM market_trade_lifecycles WHERE experiment_id=?1 AND agent_id=?2",params![experiment_id,agent],|row|row.get(0)).map_err(|error|error.to_string())?;
        tx.execute("INSERT INTO market_intraday_strategy_metrics(experiment_id,agent_id,style,trend_entries,ema_cross_entries,avg_trend_duration_minutes,breakout_attempts,breakout_entries,failed_breakouts,vwap_deviation_events,mean_reversion_entries,successful_reversions) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![experiment_id,agent,metric.style,metric.trend_entries,metric.ema_cross_entries,average_duration,metric.breakout_attempts,metric.breakout_entries,metric.failed_breakouts,metric.vwap_deviation_events,metric.mean_reversion_entries,metric.successful_reversions]).map_err(|error|error.to_string())?;
    }

    let mut holding_statement = tx.prepare("SELECT agent_id,holding_market_minutes FROM market_trade_lifecycles WHERE experiment_id=?1").map_err(|error|error.to_string())?;
    let holding_rows = holding_statement.query_map([experiment_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,usize>(1)?))).map_err(|error|error.to_string())?;
    let mut buckets: BTreeMap<(String, String), usize> = BTreeMap::new();
    for row in holding_rows {
        let (agent, minutes) = row.map_err(|error| error.to_string())?;
        let bucket = match minutes { 0..=15 => "0-15", 16..=60 => "16-60", 61..=180 => "61-180", _ => "180+" };
        *buckets.entry((agent, bucket.into())).or_default() += 1;
    }
    drop(holding_statement);
    for ((agent, bucket), count) in buckets {
        tx.execute("INSERT INTO market_holding_time_distribution(experiment_id,agent_id,bucket,trade_count) VALUES(?1,?2,?3,?4)",params![experiment_id,agent,bucket,count]).map_err(|error|error.to_string())?;
    }

    let mut phase_statement = tx.prepare("SELECT l.agent_id,f.session_phase,l.realized_pnl FROM market_trade_lifecycles l JOIN market_executions x ON x.timestamp=l.opened_at JOIN market_orders o ON o.id=x.order_id JOIN market_decisions d ON d.id=o.decision_id AND d.agent_id=l.agent_id JOIN market_intraday_feature_traces f ON f.experiment_id=d.experiment_id AND f.candle_index=d.candle_index WHERE l.experiment_id=?1 AND l.status='CLOSED'").map_err(|error|error.to_string())?;
    let phase_rows = phase_statement.query_map([experiment_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,f64>(2)?))).map_err(|error|error.to_string())?;
    let mut phases: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for row in phase_rows { let (agent, phase, pnl)=row.map_err(|error|error.to_string())?; phases.entry((agent,phase)).or_default().push(pnl); }
    drop(phase_statement);
    for ((agent, phase), pnl) in phases {
        let trades=pnl.len(); let wins=pnl.iter().filter(|value|**value>0.0).count(); let sum=pnl.iter().sum::<f64>();
        tx.execute("INSERT INTO market_session_phase_performance(experiment_id,agent_id,session_phase,trades,return_pct,win_rate_pct,average_pnl) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![experiment_id,agent,phase,trades,if initial_capital>0.0{sum/initial_capital*100.0}else{0.0},if trades>0{wins as f64/trades as f64*100.0}else{0.0},if trades>0{sum/trades as f64}else{0.0}]).map_err(|error|error.to_string())?;
    }

    let agents = tx.prepare("SELECT agent_id FROM market_experiment_agents WHERE experiment_id=?1 AND agent_id IN (?2,?3,?4) ORDER BY agent_id").and_then(|mut statement| statement.query_map(params![experiment_id,market_intraday::TREND_AGENT_ID,market_intraday::MOMENTUM_AGENT_ID,market_intraday::MEAN_REVERSION_AGENT_ID],|row|row.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()).map_err(|error|error.to_string())?;
    for left in 0..agents.len() { for right in left+1..agents.len() {
        let mut pair_statement=tx.prepare("SELECT a.action,b.action FROM market_decisions a JOIN market_decisions b ON b.experiment_id=a.experiment_id AND b.candle_index=a.candle_index WHERE a.experiment_id=?1 AND a.agent_id=?2 AND b.agent_id=?3").map_err(|error|error.to_string())?;
        let pairs=pair_statement.query_map(params![experiment_id,agents[left],agents[right]],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).map_err(|error|error.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())?;
        let comparable=pairs.len(); let same=pairs.iter().filter(|(a,b)|a==b).count(); let entries=pairs.iter().filter(|(a,b)|a=="BUY"&&b=="BUY").count(); let exits=pairs.iter().filter(|(a,b)|a=="SELL"&&b=="SELL").count();
        tx.execute("INSERT INTO market_intraday_agent_overlap(experiment_id,agent_a_id,agent_b_id,same_direction_decision_rate,same_entry_window_count,same_exit_window_count) VALUES(?1,?2,?3,?4,?5,?6)",params![experiment_id,agents[left],agents[right],if comparable>0{same as f64/comparable as f64*100.0}else{0.0},entries,exits]).map_err(|error|error.to_string())?;
    }}
    Ok(())
}

pub fn load(connection: &Connection, experiment_id: &str) -> Result<MarketIntradayStrategyReport, String> {
    let feature_engine_version = connection.query_row("SELECT engine_version FROM market_intraday_feature_traces WHERE experiment_id=?1 LIMIT 1",[experiment_id],|row|row.get(0)).optional().map_err(|error|error.to_string())?.unwrap_or_default();
    let mut statement=connection.prepare("SELECT candle_index,timestamp_utc,session_id,session_phase,session_progress,ready,features_json,engine_version FROM market_intraday_feature_traces WHERE experiment_id=?1 ORDER BY candle_index DESC LIMIT 2000").map_err(|error|error.to_string())?;
    let feature_traces=statement.query_map([experiment_id],|row|Ok(MarketIntradayFeatureTrace{candle_index:row.get(0)?,timestamp_utc:row.get(1)?,session_id:row.get(2)?,session_phase:row.get(3)?,session_progress:row.get(4)?,ready:row.get(5)?,features:parse_json(row.get(6)?)?,engine_version:row.get(7)?})).map_err(|error|error.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())?;
    let mut statement=connection.prepare("SELECT d.id,d.candle_index,d.timestamp,d.agent_id,s.style,s.intent,s.reason_code,s.reason,s.indicators_json,s.thresholds_json,s.position_before,s.generated_target_exposure_pct,r.result,x.execution_price FROM market_intraday_strategy_decisions s JOIN market_decisions d ON d.id=s.decision_id JOIN market_risk_evaluations r ON r.decision_id=d.id LEFT JOIN market_orders o ON o.decision_id=d.id LEFT JOIN market_executions x ON x.order_id=o.id WHERE d.experiment_id=?1 ORDER BY d.candle_index DESC,d.agent_id LIMIT 2500").map_err(|error|error.to_string())?;
    let decisions=statement.query_map([experiment_id],|row|Ok(MarketIntradayStrategyDecision{decision_id:row.get(0)?,candle_index:row.get(1)?,timestamp:row.get(2)?,agent_id:row.get(3)?,style:row.get(4)?,intent:row.get(5)?,reason_code:row.get(6)?,reason:row.get(7)?,indicators:parse_json(row.get(8)?)?,thresholds:parse_json(row.get(9)?)?,position_before:row.get(10)?,generated_target_exposure_pct:row.get(11)?,risk_result:row.get(12)?,execution_price:row.get(13)?})).map_err(|error|error.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())?;
    let metrics=load_rows(connection,"SELECT agent_id,style,trend_entries,ema_cross_entries,avg_trend_duration_minutes,breakout_attempts,breakout_entries,failed_breakouts,vwap_deviation_events,mean_reversion_entries,successful_reversions FROM market_intraday_strategy_metrics WHERE experiment_id=?1",experiment_id,|row|Ok(MarketIntradayStrategyMetric{agent_id:row.get(0)?,style:row.get(1)?,trend_entries:row.get(2)?,ema_cross_entries:row.get(3)?,average_trend_duration_minutes:row.get(4)?,breakout_attempts:row.get(5)?,breakout_entries:row.get(6)?,failed_breakouts:row.get(7)?,vwap_deviation_events:row.get(8)?,mean_reversion_entries:row.get(9)?,successful_reversions:row.get(10)?}))?;
    let phase_performance=load_rows(connection,"SELECT agent_id,session_phase,trades,return_pct,win_rate_pct,average_pnl FROM market_session_phase_performance WHERE experiment_id=?1 ORDER BY agent_id,session_phase",experiment_id,|row|Ok(MarketSessionPhasePerformance{agent_id:row.get(0)?,session_phase:row.get(1)?,trades:row.get(2)?,return_pct:row.get(3)?,win_rate_pct:row.get(4)?,average_pnl:row.get(5)?}))?;
    let holding_distribution=load_rows(connection,"SELECT agent_id,bucket,trade_count FROM market_holding_time_distribution WHERE experiment_id=?1 ORDER BY agent_id,bucket",experiment_id,|row|Ok(MarketHoldingTimeBucket{agent_id:row.get(0)?,bucket:row.get(1)?,trade_count:row.get(2)?}))?;
    let overlaps=load_rows(connection,"SELECT agent_a_id,agent_b_id,same_direction_decision_rate,same_entry_window_count,same_exit_window_count FROM market_intraday_agent_overlap WHERE experiment_id=?1 ORDER BY agent_a_id,agent_b_id",experiment_id,|row|Ok(MarketIntradayAgentOverlap{agent_a_id:row.get(0)?,agent_b_id:row.get(1)?,same_direction_decision_rate:row.get(2)?,same_entry_window_count:row.get(3)?,same_exit_window_count:row.get(4)?}))?;
    Ok(MarketIntradayStrategyReport{feature_engine_version,feature_traces,decisions,metrics,phase_performance,holding_distribution,overlaps})
}

fn load_rows<T, F>(
    connection: &Connection,
    sql: &str,
    id: &str,
    mapper: F,
) -> Result<Vec<T>, String>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    let mut statement = connection.prepare(sql).map_err(|error| error.to_string())?;
    let values = statement
        .query_map([id], mapper)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(values)
}
