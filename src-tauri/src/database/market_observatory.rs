use super::market_models::{
    MarketAgentCorrelation, MarketBehaviorMetric, MarketBenchmarkComparison, MarketObservatory,
    MarketPositionEpisode,
};
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub const FORMULA_VERSION: &str = "behavior-v1";
pub const SIMILARITY_THRESHOLD: f64 = 0.85;
const EPSILON: f64 = 0.000_001;

#[derive(Clone, Debug)]
struct Snapshot {
    candle_index: usize,
    timestamp: String,
    quantity: f64,
    equity: f64,
    exposure_pct: f64,
}

#[derive(Clone, Debug)]
struct Decision {
    action: String,
    risk_result: String,
    risk_reason: String,
}

fn rounded(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn percentage(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 { 0.0 } else { rounded(numerator as f64 / denominator as f64 * 100.0) }
}

fn median(values: &mut [usize]) -> f64 {
    if values.is_empty() { return 0.0; }
    values.sort_unstable();
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        rounded((values[middle - 1] + values[middle]) as f64 / 2.0)
    } else {
        values[middle] as f64
    }
}

fn pearson(left: &[f64], right: &[f64]) -> Option<f64> {
    let size = left.len().min(right.len());
    if size < 2 { return None; }
    let left = &left[..size];
    let right = &right[..size];
    let left_mean = left.iter().sum::<f64>() / size as f64;
    let right_mean = right.iter().sum::<f64>() / size as f64;
    let covariance = left.iter().zip(right).map(|(a, b)| (a - left_mean) * (b - right_mean)).sum::<f64>();
    let left_variance = left.iter().map(|value| (value - left_mean).powi(2)).sum::<f64>();
    let right_variance = right.iter().map(|value| (value - right_mean).powi(2)).sum::<f64>();
    if left_variance <= EPSILON || right_variance <= EPSILON { None } else { Some(rounded(covariance / (left_variance * right_variance).sqrt())) }
}

fn equity_returns(snapshots: &[Snapshot]) -> Vec<f64> {
    snapshots.windows(2).map(|pair| {
        if pair[0].equity.abs() <= EPSILON { 0.0 } else { rounded(pair[1].equity / pair[0].equity - 1.0) }
    }).collect()
}

fn build_episodes(agent_id: &str, snapshots: &[Snapshot]) -> Vec<MarketPositionEpisode> {
    let mut episodes = Vec::new();
    let mut active: Option<(usize, usize, String, usize, f64)> = None;
    for snapshot in snapshots {
        if snapshot.quantity > EPSILON {
            if let Some((_, _, _, duration, max_exposure)) = active.as_mut() {
                *duration += 1;
                *max_exposure = max_exposure.max(snapshot.exposure_pct);
            } else {
                active = Some((episodes.len() + 1, snapshot.candle_index, snapshot.timestamp.clone(), 1, snapshot.exposure_pct));
            }
        } else if let Some((episode_index, opened_index, opened_at, duration, max_exposure)) = active.take() {
            episodes.push(MarketPositionEpisode {
                agent_id: agent_id.into(), episode_index, opened_candle_index: opened_index,
                opened_at, closed_candle_index: Some(snapshot.candle_index), closed_at: Some(snapshot.timestamp.clone()),
                duration_candles: duration, max_exposure_pct: rounded(max_exposure), status: "closed".into(),
            });
        }
    }
    if let Some((episode_index, opened_index, opened_at, duration, max_exposure)) = active {
        episodes.push(MarketPositionEpisode {
            agent_id: agent_id.into(), episode_index, opened_candle_index: opened_index, opened_at,
            closed_candle_index: None, closed_at: None, duration_candles: duration,
            max_exposure_pct: rounded(max_exposure), status: "open".into(),
        });
    }
    episodes
}

fn load_agents(connection: &Connection, experiment_id: &str) -> Result<Vec<String>, String> {
    let mut statement = connection.prepare("SELECT agent_id FROM market_experiment_agents WHERE experiment_id=?1 ORDER BY rowid").map_err(|error| error.to_string())?;
    let rows = statement.query_map([experiment_id], |row| row.get(0)).map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
    Ok(rows)
}

fn load_snapshots(connection: &Connection, experiment_id: &str, agent_id: &str) -> Result<Vec<Snapshot>, String> {
    let mut statement = connection.prepare("SELECT candle_index,timestamp,quantity,equity,exposure_pct FROM market_portfolio_snapshots WHERE experiment_id=?1 AND agent_id=?2 ORDER BY candle_index").map_err(|error| error.to_string())?;
    let rows = statement.query_map(params![experiment_id, agent_id], |row| Ok(Snapshot { candle_index: row.get(0)?, timestamp: row.get(1)?, quantity: row.get(2)?, equity: row.get(3)?, exposure_pct: row.get(4)? })).map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
    Ok(rows)
}

fn load_decisions(connection: &Connection, experiment_id: &str, agent_id: &str) -> Result<Vec<Decision>, String> {
    let mut statement = connection.prepare("SELECT d.action,r.result,r.reason FROM market_decisions d JOIN market_risk_evaluations r ON r.decision_id=d.id WHERE d.experiment_id=?1 AND d.agent_id=?2 ORDER BY d.candle_index").map_err(|error| error.to_string())?;
    let rows = statement.query_map(params![experiment_id, agent_id], |row| Ok(Decision { action: row.get(0)?, risk_result: row.get(1)?, risk_reason: row.get(2)? })).map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
    Ok(rows)
}

fn execution_summary(connection: &Connection, experiment_id: &str, agent_id: &str) -> Result<(usize, usize, usize, f64), String> {
    connection.query_row("SELECT COUNT(*),COALESCE(SUM(CASE WHEN o.side='BUY' THEN 1 ELSE 0 END),0),COALESCE(SUM(CASE WHEN o.side='SELL' THEN 1 ELSE 0 END),0),COALESCE(SUM(ABS(x.gross_value)),0) FROM market_executions x JOIN market_orders o ON o.id=x.order_id JOIN market_decisions d ON d.id=o.decision_id WHERE d.experiment_id=?1 AND d.agent_id=?2", params![experiment_id, agent_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))).map_err(|error| error.to_string())
}

pub fn calculate(connection: &Connection, experiment_id: &str) -> Result<MarketObservatory, String> {
    let agents = load_agents(connection, experiment_id)?;
    let returns_by_agent: HashMap<String, f64> = {
        let mut statement = connection.prepare("SELECT agent_id,total_return_pct FROM market_agent_metrics WHERE experiment_id=?1").map_err(|error| error.to_string())?;
        let rows = statement.query_map([experiment_id], |row| Ok((row.get(0)?, row.get(1)?))).map_err(|error| error.to_string())?
            .collect::<Result<HashMap<_, _>, _>>().map_err(|error| error.to_string())?;
        rows
    };
    let cash_return = returns_by_agent.get("cash").copied();
    let buy_hold_return = returns_by_agent.get("buy-hold").copied();
    let mut behavior = Vec::new();
    let mut episodes = Vec::new();
    let mut snapshots_by_agent = HashMap::new();
    let mut decisions_by_agent = HashMap::new();

    for agent_id in &agents {
        let snapshots = load_snapshots(connection, experiment_id, agent_id)?;
        let decisions = load_decisions(connection, experiment_id, agent_id)?;
        let agent_episodes = build_episodes(agent_id, &snapshots);
        let mut durations = agent_episodes.iter().map(|episode| episode.duration_candles).collect::<Vec<_>>();
        let average_holding = if durations.is_empty() { 0.0 } else { rounded(durations.iter().sum::<usize>() as f64 / durations.len() as f64) };
        let max_holding = durations.iter().copied().max().unwrap_or(0);
        let median_holding = median(&mut durations);
        let buy_count = decisions.iter().filter(|decision| decision.action == "BUY").count();
        let sell_count = decisions.iter().filter(|decision| decision.action == "SELL").count();
        let hold_count = decisions.iter().filter(|decision| decision.action == "HOLD").count();
        let rejected = decisions.iter().filter(|decision| decision.risk_result == "REJECTED").count();
        let modified = decisions.iter().filter(|decision| decision.risk_result == "MODIFIED").count();
        let drawdown_triggers = decisions.iter().filter(|decision| decision.risk_reason.to_lowercase().contains("drawdown")).count();
        let daily_loss_triggers = decisions.iter().filter(|decision| decision.risk_reason.to_lowercase().contains("perda diária")).count();
        let exposed = snapshots.iter().filter(|snapshot| snapshot.quantity > EPSILON).collect::<Vec<_>>();
        let average_exposure = if snapshots.is_empty() { 0.0 } else { rounded(snapshots.iter().map(|snapshot| snapshot.exposure_pct).sum::<f64>() / snapshots.len() as f64) };
        let average_position = if exposed.is_empty() { 0.0 } else { rounded(exposed.iter().map(|snapshot| snapshot.exposure_pct).sum::<f64>() / exposed.len() as f64) };
        let max_exposure = snapshots.iter().map(|snapshot| snapshot.exposure_pct).fold(0.0, f64::max);
        let average_equity = if snapshots.is_empty() { 0.0 } else { snapshots.iter().map(|snapshot| snapshot.equity).sum::<f64>() / snapshots.len() as f64 };
        let (execution_count, entry_count, exit_count, gross_traded) = execution_summary(connection, experiment_id, agent_id)?;
        let turnover = if average_equity.abs() <= EPSILON { 0.0 } else { rounded(gross_traded / average_equity * 100.0) };
        behavior.push(MarketBehaviorMetric {
            agent_id: agent_id.clone(), buy_count, sell_count, hold_count,
            hold_rate_pct: percentage(hold_count, decisions.len()),
            trade_frequency_pct: percentage(execution_count, snapshots.len()),
            average_exposure_pct: average_exposure, max_exposure_pct: rounded(max_exposure),
            average_position_size_pct: average_position, max_position_size_pct: rounded(max_exposure),
            average_holding_candles: average_holding, median_holding_candles: median_holding,
            max_holding_candles: max_holding, turnover_pct: turnover,
            time_in_market_pct: percentage(exposed.len(), snapshots.len()),
            time_in_cash_pct: percentage(snapshots.len().saturating_sub(exposed.len()), snapshots.len()),
            entry_count, exit_count,
            risk_rejection_count: rejected, risk_modification_count: modified,
            risk_rejection_rate_pct: percentage(rejected, decisions.len()),
            risk_modification_rate_pct: percentage(modified, decisions.len()),
            drawdown_trigger_count: drawdown_triggers, daily_loss_trigger_count: daily_loss_triggers,
            formula_version: FORMULA_VERSION.into(),
        });
        episodes.extend(agent_episodes);
        snapshots_by_agent.insert(agent_id.clone(), snapshots);
        decisions_by_agent.insert(agent_id.clone(), decisions);
    }

    let mut correlations = Vec::new();
    for (left_index, left_id) in agents.iter().enumerate() {
        for right_id in agents.iter().skip(left_index) {
            let diagonal = left_id == right_id;
            let left_actions = &decisions_by_agent[left_id];
            let right_actions = &decisions_by_agent[right_id];
            let decision_count = left_actions.len().min(right_actions.len());
            let equal_decisions = left_actions.iter().zip(right_actions).filter(|(left, right)| left.action == right.action).count();
            let decision_similarity = if diagonal { 1.0 } else if decision_count == 0 { 0.0 } else { rounded(equal_decisions as f64 / decision_count as f64) };
            let equity_return_correlation = if diagonal { Some(1.0) } else { pearson(&equity_returns(&snapshots_by_agent[left_id]), &equity_returns(&snapshots_by_agent[right_id])) };
            let high_similarity = !diagonal && (decision_similarity >= SIMILARITY_THRESHOLD || equity_return_correlation.is_some_and(|value| value >= SIMILARITY_THRESHOLD));
            correlations.push(MarketAgentCorrelation { agent_a_id: left_id.clone(), agent_b_id: right_id.clone(), equity_return_correlation, decision_similarity, high_similarity, similarity_threshold: SIMILARITY_THRESHOLD });
        }
    }

    let benchmarks = agents.iter().map(|agent_id| {
        let agent_return = returns_by_agent.get(agent_id).copied().unwrap_or(0.0);
        MarketBenchmarkComparison { agent_id: agent_id.clone(), cash_return_pct: cash_return, buy_hold_return_pct: buy_hold_return, excess_vs_cash_pct: rounded(agent_return - cash_return.unwrap_or(0.0)), excess_vs_buy_hold_pct: buy_hold_return.map(|value| rounded(agent_return - value)) }
    }).collect();
    Ok(MarketObservatory { behavior, episodes, correlations, benchmarks, similarity_threshold: SIMILARITY_THRESHOLD })
}

pub fn load(connection: &Connection, experiment_id: &str) -> Result<MarketObservatory, String> {
    let count: usize = connection.query_row("SELECT COUNT(*) FROM market_behavior_metrics WHERE experiment_id=?1", [experiment_id], |row| row.get(0)).map_err(|error| error.to_string())?;
    if count == 0 {
        return calculate(connection, experiment_id);
    }
    let behavior = {
        let mut statement = connection.prepare("SELECT agent_id,buy_count,sell_count,hold_count,hold_rate_pct,trade_frequency_pct,average_exposure_pct,max_exposure_pct,average_position_size_pct,max_position_size_pct,average_holding_candles,median_holding_candles,max_holding_candles,turnover_pct,time_in_market_pct,time_in_cash_pct,entry_count,exit_count,risk_rejection_count,risk_modification_count,risk_rejection_rate_pct,risk_modification_rate_pct,drawdown_trigger_count,daily_loss_trigger_count,formula_version FROM market_behavior_metrics WHERE experiment_id=?1 ORDER BY agent_id").map_err(|error| error.to_string())?;
        let rows = statement.query_map([experiment_id], |row| Ok(MarketBehaviorMetric { agent_id: row.get(0)?, buy_count: row.get(1)?, sell_count: row.get(2)?, hold_count: row.get(3)?, hold_rate_pct: row.get(4)?, trade_frequency_pct: row.get(5)?, average_exposure_pct: row.get(6)?, max_exposure_pct: row.get(7)?, average_position_size_pct: row.get(8)?, max_position_size_pct: row.get(9)?, average_holding_candles: row.get(10)?, median_holding_candles: row.get(11)?, max_holding_candles: row.get(12)?, turnover_pct: row.get(13)?, time_in_market_pct: row.get(14)?, time_in_cash_pct: row.get(15)?, entry_count: row.get(16)?, exit_count: row.get(17)?, risk_rejection_count: row.get(18)?, risk_modification_count: row.get(19)?, risk_rejection_rate_pct: row.get(20)?, risk_modification_rate_pct: row.get(21)?, drawdown_trigger_count: row.get(22)?, daily_loss_trigger_count: row.get(23)?, formula_version: row.get(24)? })).map_err(|error| error.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
        rows
    };
    let episodes = {
        let mut statement = connection.prepare("SELECT agent_id,episode_index,opened_candle_index,opened_at,closed_candle_index,closed_at,duration_candles,max_exposure_pct,status FROM market_position_episodes WHERE experiment_id=?1 ORDER BY agent_id,episode_index").map_err(|error| error.to_string())?;
        let rows = statement.query_map([experiment_id], |row| Ok(MarketPositionEpisode { agent_id: row.get(0)?, episode_index: row.get(1)?, opened_candle_index: row.get(2)?, opened_at: row.get(3)?, closed_candle_index: row.get(4)?, closed_at: row.get(5)?, duration_candles: row.get(6)?, max_exposure_pct: row.get(7)?, status: row.get(8)? })).map_err(|error| error.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
        rows
    };
    let correlations = {
        let mut statement = connection.prepare("SELECT agent_a_id,agent_b_id,equity_return_correlation,decision_similarity,high_similarity,similarity_threshold FROM market_agent_correlations WHERE experiment_id=?1 ORDER BY agent_a_id,agent_b_id").map_err(|error| error.to_string())?;
        let rows = statement.query_map([experiment_id], |row| Ok(MarketAgentCorrelation { agent_a_id: row.get(0)?, agent_b_id: row.get(1)?, equity_return_correlation: row.get(2)?, decision_similarity: row.get(3)?, high_similarity: row.get(4)?, similarity_threshold: row.get(5)? })).map_err(|error| error.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
        rows
    };
    let benchmarks = {
        let mut statement = connection.prepare("SELECT agent_id,cash_return_pct,buy_hold_return_pct,excess_vs_cash_pct,excess_vs_buy_hold_pct FROM market_comparison_snapshots WHERE experiment_id=?1 ORDER BY agent_id").map_err(|error| error.to_string())?;
        let rows = statement.query_map([experiment_id], |row| Ok(MarketBenchmarkComparison { agent_id: row.get(0)?, cash_return_pct: row.get(1)?, buy_hold_return_pct: row.get(2)?, excess_vs_cash_pct: row.get(3)?, excess_vs_buy_hold_pct: row.get(4)? })).map_err(|error| error.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
        rows
    };
    Ok(MarketObservatory { behavior, episodes, correlations, benchmarks, similarity_threshold: SIMILARITY_THRESHOLD })
}

pub fn persist(connection: &Connection, experiment_id: &str) -> Result<MarketObservatory, String> {
    let observatory = calculate(connection, experiment_id)?;
    connection.execute("DELETE FROM market_behavior_metrics WHERE experiment_id=?1", [experiment_id]).map_err(|error| error.to_string())?;
    connection.execute("DELETE FROM market_position_episodes WHERE experiment_id=?1", [experiment_id]).map_err(|error| error.to_string())?;
    connection.execute("DELETE FROM market_agent_correlations WHERE experiment_id=?1", [experiment_id]).map_err(|error| error.to_string())?;
    connection.execute("DELETE FROM market_comparison_snapshots WHERE experiment_id=?1", [experiment_id]).map_err(|error| error.to_string())?;
    for metric in &observatory.behavior {
        connection.execute("INSERT INTO market_behavior_metrics VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,CURRENT_TIMESTAMP)", params![experiment_id,metric.agent_id,metric.buy_count,metric.sell_count,metric.hold_count,metric.hold_rate_pct,metric.trade_frequency_pct,metric.average_exposure_pct,metric.max_exposure_pct,metric.average_position_size_pct,metric.max_position_size_pct,metric.average_holding_candles,metric.median_holding_candles,metric.max_holding_candles,metric.turnover_pct,metric.time_in_market_pct,metric.time_in_cash_pct,metric.entry_count,metric.exit_count,metric.risk_rejection_count,metric.risk_modification_count,metric.risk_rejection_rate_pct,metric.risk_modification_rate_pct,metric.drawdown_trigger_count,metric.daily_loss_trigger_count,metric.formula_version]).map_err(|error| error.to_string())?;
    }
    for episode in &observatory.episodes {
        connection.execute("INSERT INTO market_position_episodes(experiment_id,agent_id,episode_index,opened_candle_index,opened_at,closed_candle_index,closed_at,duration_candles,max_exposure_pct,status) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![experiment_id,episode.agent_id,episode.episode_index,episode.opened_candle_index,episode.opened_at,episode.closed_candle_index,episode.closed_at,episode.duration_candles,episode.max_exposure_pct,episode.status]).map_err(|error| error.to_string())?;
    }
    for correlation in &observatory.correlations {
        connection.execute("INSERT INTO market_agent_correlations VALUES(?1,?2,?3,?4,?5,?6,?7,?8)", params![experiment_id,correlation.agent_a_id,correlation.agent_b_id,correlation.equity_return_correlation,correlation.decision_similarity,correlation.high_similarity,correlation.similarity_threshold,FORMULA_VERSION]).map_err(|error| error.to_string())?;
    }
    for benchmark in &observatory.benchmarks {
        connection.execute("INSERT INTO market_comparison_snapshots(experiment_id,agent_id,cash_return_pct,buy_hold_return_pct,excess_vs_cash_pct,excess_vs_buy_hold_pct) VALUES(?1,?2,?3,?4,?5,?6)", params![experiment_id,benchmark.agent_id,benchmark.cash_return_pct,benchmark.buy_hold_return_pct,benchmark.excess_vs_cash_pct,benchmark.excess_vs_buy_hold_pct]).map_err(|error| error.to_string())?;
    }
    Ok(observatory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentages_and_median_are_deterministic() {
        assert_eq!(percentage(7, 10), 70.0);
        assert_eq!(percentage(1, 0), 0.0);
        assert_eq!(median(&mut [1, 3, 9]), 3.0);
        assert_eq!(median(&mut [1, 3, 5, 9]), 4.0);
    }

    #[test]
    fn pearson_uses_returns_and_rejects_zero_variance() {
        assert_eq!(pearson(&[1.0, 2.0, 3.0], &[2.0, 4.0, 6.0]), Some(1.0));
        assert_eq!(pearson(&[1.0, 1.0, 1.0], &[2.0, 4.0, 6.0]), None);
    }

    #[test]
    fn position_episode_counts_exposed_candles() {
        let snapshots = vec![
            Snapshot { candle_index: 0, timestamp: "0".into(), quantity: 0.0, equity: 1.0, exposure_pct: 0.0 },
            Snapshot { candle_index: 1, timestamp: "1".into(), quantity: 1.0, equity: 1.0, exposure_pct: 20.0 },
            Snapshot { candle_index: 2, timestamp: "2".into(), quantity: 1.0, equity: 1.0, exposure_pct: 30.0 },
            Snapshot { candle_index: 3, timestamp: "3".into(), quantity: 0.0, equity: 1.0, exposure_pct: 0.0 },
        ];
        let episodes = build_episodes("agent", &snapshots);
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].duration_candles, 2);
        assert_eq!(episodes[0].closed_candle_index, Some(3));
        assert_eq!(episodes[0].max_exposure_pct, 30.0);
    }
}
