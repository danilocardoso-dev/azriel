use super::{market_ai, market_models::*, market_regimes, market_repository, market_statistics};
use rusqlite::{params, Connection};
use serde_json::json;
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub const VALIDATION_ENGINE_VERSION: &str = "VALIDATION_ENGINE_V1";
const ROBUSTNESS_RULE_VERSION: &str = "ROBUSTNESS_RULES_V1";
const MIN_WALK_FORWARD_WINDOWS: usize = 3;
const ROBUST_POSITIVE_WINDOW_RATIO: f64 = 60.0;
const OVERFITTING_GAP_PCT: f64 = 10.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WindowSpec {
    pub index: usize,
    pub kind: &'static str,
    pub start: usize,
    pub end: usize,
    pub train_start: Option<usize>,
    pub train_end: Option<usize>,
}

fn new_id() -> String {
    format!(
        "validation-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    )
}

fn rounded(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub(crate) fn temporal_windows(
    candle_count: usize,
    config: &ValidationSplitConfig,
) -> Result<Vec<WindowSpec>, String> {
    if config.in_sample_pct + config.validation_pct + config.out_of_sample_pct != 100 {
        return Err("IN_SAMPLE + VALIDATION + OUT_OF_SAMPLE deve ser 100%".into());
    }
    if config.in_sample_pct == 0 || config.validation_pct == 0 || config.out_of_sample_pct == 0 {
        return Err("todos os splits temporais devem ser maiores que zero".into());
    }
    let in_sample = candle_count * config.in_sample_pct / 100;
    let validation = candle_count * config.validation_pct / 100;
    let out_of_sample = candle_count.saturating_sub(in_sample + validation);
    if [in_sample, validation, out_of_sample]
        .iter()
        .any(|size| *size < 2)
    {
        return Err("dataset insuficiente: cada split precisa de ao menos 2 candles".into());
    }
    Ok(vec![
        WindowSpec {
            index: 0,
            kind: "in_sample",
            start: 0,
            end: in_sample - 1,
            train_start: None,
            train_end: None,
        },
        WindowSpec {
            index: 0,
            kind: "validation",
            start: in_sample,
            end: in_sample + validation - 1,
            train_start: None,
            train_end: None,
        },
        WindowSpec {
            index: 0,
            kind: "out_of_sample",
            start: in_sample + validation,
            end: candle_count - 1,
            train_start: None,
            train_end: None,
        },
    ])
}

pub(crate) fn walk_forward_windows(
    candle_count: usize,
    config: &WalkForwardConfig,
) -> Result<Vec<WindowSpec>, String> {
    if config.train_window_size < 2 || config.test_window_size < 2 || config.step_size == 0 {
        return Err("walk-forward exige train/test >= 2 e step > 0".into());
    }
    if config.train_window_size + config.test_window_size > candle_count {
        return Err("dataset insuficiente para a configuração walk-forward".into());
    }
    let mut windows = Vec::new();
    let mut train_start = 0usize;
    while train_start + config.train_window_size + config.test_window_size <= candle_count {
        let train_end = train_start + config.train_window_size - 1;
        let test_start = train_end + 1;
        let test_end = test_start + config.test_window_size - 1;
        windows.push(WindowSpec {
            index: windows.len(),
            kind: "walk_forward",
            start: test_start,
            end: test_end,
            train_start: Some(train_start),
            train_end: Some(train_end),
        });
        train_start += config.step_size;
    }
    Ok(windows)
}

fn validate(
    input: &MarketValidationInput,
    candle_count: usize,
) -> Result<(Vec<WindowSpec>, Vec<WindowSpec>), String> {
    if input.name.trim().is_empty() {
        return Err("nome da validation run é obrigatório".into());
    }
    if input.rolling_window < 2 || input.rolling_window > candle_count {
        return Err("rolling window inválida para o dataset".into());
    }
    if !input.annualization_factor.is_finite() || input.annualization_factor <= 0.0 {
        return Err("annualization factor deve ser positivo".into());
    }
    market_regimes::validate_config(&input.regime_config)?;
    Ok((
        temporal_windows(candle_count, &input.split_config)?,
        walk_forward_windows(candle_count, &input.walk_forward_config)?,
    ))
}

fn experiment_input(input: &MarketValidationInput) -> MarketExperimentInput {
    MarketExperimentInput {
        name: input.name.clone(),
        dataset_id: input.dataset_id.clone(),
        risk_profile_id: input.risk_profile_id.clone(),
        agent_ids: input.agent_ids.clone(),
        initial_capital: input.initial_capital,
        random_seed: input.random_seed,
        fee_pct: input.fee_pct,
        slippage_pct: input.slippage_pct,
    }
}

fn benchmark_buy_hold(candles: &[market_repository::Candle], start: usize, end: usize) -> f64 {
    market_statistics::total_return_pct(candles[start].close, candles[end].close)
}

fn metric_for_run(
    run: &market_repository::AgentRun,
    initial: f64,
    annualization: f64,
    buy_hold_return: f64,
) -> MarketWindowMetric {
    let mut equity = Vec::with_capacity(run.snapshots.len() + 1);
    equity.push(initial);
    equity.extend(run.snapshots.iter().map(|point| point.equity));
    let returns = market_statistics::periodic_returns(&equity);
    let final_equity = equity.last().copied().unwrap_or(initial);
    let total_return = market_statistics::total_return_pct(initial, final_equity);
    let drawdown = market_statistics::max_drawdown_pct(&equity);
    let decisions = run.decisions.len();
    MarketWindowMetric {
        window_id: 0,
        window_index: 0,
        window_type: String::new(),
        agent_id: run.id.clone(),
        agent_name: run.name.clone(),
        total_return_pct: rounded(total_return),
        max_drawdown_pct: rounded(drawdown),
        sharpe: market_statistics::sharpe(&returns, 0.0, annualization).map(rounded),
        sortino: market_statistics::sortino(&returns, 0.0, annualization).map(rounded),
        calmar: market_statistics::calmar(
            initial,
            final_equity,
            returns.len(),
            annualization,
            drawdown,
        )
        .map(rounded),
        trade_count: run
            .decisions
            .iter()
            .filter(|row| row.execution.is_some())
            .count(),
        hold_rate_pct: if decisions == 0 {
            0.0
        } else {
            rounded(run.hold_count as f64 / decisions as f64 * 100.0)
        },
        exposure_pct: if run.snapshots.is_empty() {
            0.0
        } else {
            rounded(
                run.snapshots
                    .iter()
                    .map(|point| point.exposure)
                    .sum::<f64>()
                    / run.snapshots.len() as f64,
            )
        },
        benchmark_cash_excess_pct: rounded(total_return),
        benchmark_buy_hold_excess_pct: rounded(total_return - buy_hold_return),
    }
}

fn persist_window(
    connection: &Connection,
    validation_id: &str,
    spec: &WindowSpec,
    candles: &[market_repository::Candle],
) -> Result<i64, String> {
    connection.execute(
        "INSERT INTO market_validation_windows(validation_run_id,window_index,window_type,start_index,end_index,start_at,end_at,train_start_index,train_end_index,train_start_at,train_end_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
        params![validation_id, spec.index, spec.kind, spec.start, spec.end, candles[spec.start].timestamp, candles[spec.end].timestamp, spec.train_start, spec.train_end, spec.train_start.map(|index| candles[index].timestamp.clone()), spec.train_end.map(|index| candles[index].timestamp.clone())],
    ).map_err(|error| error.to_string())?;
    Ok(connection.last_insert_rowid())
}

fn persist_metric(
    connection: &Connection,
    validation_id: &str,
    window_id: i64,
    metric: &MarketWindowMetric,
) -> Result<(), String> {
    connection.execute(
        "INSERT INTO market_agent_validation_metrics(validation_run_id,window_id,agent_id,total_return_pct,max_drawdown_pct,sharpe,sortino,calmar,trade_count,hold_rate_pct,exposure_pct,benchmark_cash_excess_pct,benchmark_buy_hold_excess_pct,formula_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",
        params![validation_id, window_id, metric.agent_id, metric.total_return_pct, metric.max_drawdown_pct, metric.sharpe, metric.sortino, metric.calmar, metric.trade_count, metric.hold_rate_pct, metric.exposure_pct, metric.benchmark_cash_excess_pct, metric.benchmark_buy_hold_excess_pct, market_statistics::METRIC_FORMULA_VERSION],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

fn persist_regimes(
    connection: &Connection,
    validation_id: &str,
    candles: &[market_repository::Candle],
    labels: &[market_regimes::RegimeLabel],
) -> Result<(), String> {
    for (index, (candle, label)) in candles.iter().zip(labels).enumerate() {
        connection.execute("INSERT INTO market_regimes(validation_run_id,candle_index,timestamp,trend_regime,volatility_regime,formula_version) VALUES(?1,?2,?3,?4,?5,?6)", params![validation_id,index,candle.timestamp,label.trend,label.volatility,market_regimes::REGIME_ENGINE_VERSION]).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn persist_rolling(
    connection: &Connection,
    validation_id: &str,
    runs: &[market_repository::AgentRun],
    window: usize,
    annualization: f64,
) -> Result<(), String> {
    for run in runs {
        let equity: Vec<f64> = run.snapshots.iter().map(|point| point.equity).collect();
        for (index, point) in run.snapshots.iter().enumerate() {
            connection.execute("INSERT INTO market_rolling_metrics(validation_run_id,agent_id,candle_index,timestamp,rolling_return_pct,rolling_volatility_pct,rolling_sharpe,rolling_drawdown_pct,window_size,formula_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)", params![validation_id,run.id,point.candle_index,point.timestamp,market_statistics::rolling_return_pct(&equity,index,window).map(rounded),market_statistics::rolling_volatility_pct(&equity,index,window,annualization).map(rounded),market_statistics::rolling_sharpe(&equity,index,window,annualization).map(rounded),market_statistics::rolling_drawdown_pct(&equity,index,window).map(rounded),window,market_statistics::METRIC_FORMULA_VERSION]).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn regime_metric(
    run: &market_repository::AgentRun,
    labels: &[market_regimes::RegimeLabel],
    regime_type: &str,
    regime: &str,
    config: &MarketRegimeConfig,
) -> MarketRegimeMetric {
    let matches = |index: usize| {
        labels.get(index).is_some_and(|label| {
            if regime_type == "trend" {
                label.trend == regime
            } else {
                label.volatility == regime
            }
        })
    };
    let snapshots: Vec<_> = run
        .snapshots
        .iter()
        .filter(|point| matches(point.candle_index))
        .collect();
    let decisions: Vec<_> = run
        .decisions
        .iter()
        .filter(|point| matches(point.candle_index))
        .collect();
    let mut synthetic = vec![100.0];
    let mut positive = 0.0;
    let mut negative = 0.0;
    let mut wins = 0usize;
    let mut losses = 0usize;
    for pair in run.snapshots.windows(2) {
        if matches(pair[1].candle_index) && pair[0].equity > 0.0 {
            let change = pair[1].equity / pair[0].equity - 1.0;
            synthetic.push(synthetic.last().copied().unwrap_or(100.0) * (1.0 + change));
            let realized_change = pair[1].realized - pair[0].realized;
            if realized_change > 0.0 {
                positive += realized_change;
                wins += 1;
            }
            if realized_change < 0.0 {
                negative += realized_change.abs();
                losses += 1;
            }
        }
    }
    let trades = decisions
        .iter()
        .filter(|row| row.execution.is_some())
        .count();
    MarketRegimeMetric {
        agent_id: run.id.clone(),
        regime_type: regime_type.into(),
        regime: regime.into(),
        candle_count: snapshots.len(),
        total_return_pct: rounded(market_statistics::total_return_pct(
            100.0,
            synthetic.last().copied().unwrap_or(100.0),
        )),
        max_drawdown_pct: rounded(market_statistics::max_drawdown_pct(&synthetic)),
        trade_count: trades,
        hold_rate_pct: if decisions.is_empty() {
            0.0
        } else {
            rounded(
                decisions
                    .iter()
                    .filter(|row| row.decision.action == "HOLD")
                    .count() as f64
                    / decisions.len() as f64
                    * 100.0,
            )
        },
        exposure_pct: if snapshots.is_empty() {
            0.0
        } else {
            rounded(
                snapshots.iter().map(|point| point.exposure).sum::<f64>() / snapshots.len() as f64,
            )
        },
        profit_factor: (negative > 0.0).then(|| rounded(positive / negative)),
        win_rate_pct: ((wins + losses) > 0)
            .then(|| rounded(wins as f64 / (wins + losses) as f64 * 100.0)),
        low_sample_size: snapshots.len() < config.minimum_sample_candles
            || trades < config.minimum_sample_trades,
    }
}

fn persist_regime_metrics(
    connection: &Connection,
    validation_id: &str,
    runs: &[market_repository::AgentRun],
    labels: &[market_regimes::RegimeLabel],
    config: &MarketRegimeConfig,
) -> Result<(), String> {
    for run in runs {
        for (kind, values) in [
            ("trend", &["bull", "bear", "sideways"][..]),
            (
                "volatility",
                &["high_volatility", "normal_volatility", "low_volatility"][..],
            ),
        ] {
            for regime in values {
                let metric = regime_metric(run, labels, kind, regime, config);
                connection.execute("INSERT INTO market_regime_metrics(validation_run_id,agent_id,regime_type,regime,candle_count,total_return_pct,max_drawdown_pct,trade_count,hold_rate_pct,exposure_pct,profit_factor,win_rate_pct,low_sample_size) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)", params![validation_id,metric.agent_id,metric.regime_type,metric.regime,metric.candle_count,metric.total_return_pct,metric.max_drawdown_pct,metric.trade_count,metric.hold_rate_pct,metric.exposure_pct,metric.profit_factor,metric.win_rate_pct,metric.low_sample_size]).map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

fn build_reports(
    metrics: &[MarketWindowMetric],
    agents: &[MarketAgentDefinition],
    risk: &MarketRiskProfile,
) -> Vec<MarketRobustnessReport> {
    agents
        .iter()
        .map(|agent| {
            let split = |kind: &str| {
                metrics
                    .iter()
                    .find(|metric| metric.agent_id == agent.id && metric.window_type == kind)
            };
            let is_metric = split("in_sample").expect("validated split");
            let validation_metric = split("validation").expect("validated split");
            let oos = split("out_of_sample").expect("validated split");
            let walk: Vec<_> = metrics
                .iter()
                .filter(|metric| {
                    metric.agent_id == agent.id && metric.window_type == "walk_forward"
                })
                .collect();
            let returns: Vec<f64> = walk.iter().map(|metric| metric.total_return_pct).collect();
            let drawdowns: Vec<f64> = walk.iter().map(|metric| metric.max_drawdown_pct).collect();
            let positive_ratio = if walk.is_empty() {
                0.0
            } else {
                returns.iter().filter(|value| **value > 0.0).count() as f64 / walk.len() as f64
                    * 100.0
            };
            let average = market_statistics::mean(&returns).unwrap_or(0.0);
            let overfitting_gap = is_metric.total_return_pct - oos.total_return_pct;
            let possible_overfitting = overfitting_gap >= OVERFITTING_GAP_PCT
                && (oos.total_return_pct <= 0.0
                    || oos.total_return_pct < is_metric.total_return_pct * 0.25);
            let status = if walk.len() < MIN_WALK_FORWARD_WINDOWS {
                "insufficient_data"
            } else if oos.total_return_pct > 0.0
                && positive_ratio >= ROBUST_POSITIVE_WINDOW_RATIO
                && oos.max_drawdown_pct <= risk.max_drawdown_pct
                && oos.benchmark_buy_hold_excess_pct >= 0.0
            {
                "robust_candidate"
            } else if positive_ratio < 40.0 || (oos.total_return_pct < 0.0 && possible_overfitting)
            {
                "unstable"
            } else {
                "mixed"
            };
            MarketRobustnessReport {
                agent_id: agent.id.clone(),
                agent_name: agent.name.clone(),
                in_sample_return_pct: is_metric.total_return_pct,
                validation_return_pct: validation_metric.total_return_pct,
                out_of_sample_return_pct: oos.total_return_pct,
                out_of_sample_drawdown_pct: oos.max_drawdown_pct,
                positive_window_ratio_pct: rounded(positive_ratio),
                average_window_return_pct: rounded(average),
                median_window_return_pct: rounded(
                    market_statistics::median(&returns).unwrap_or(0.0),
                ),
                best_window_return_pct: rounded(
                    returns.iter().copied().reduce(f64::max).unwrap_or(0.0),
                ),
                worst_window_return_pct: rounded(
                    returns.iter().copied().reduce(f64::min).unwrap_or(0.0),
                ),
                return_std_across_windows: rounded(
                    market_statistics::population_std(&returns).unwrap_or(0.0),
                ),
                drawdown_std_across_windows: rounded(
                    market_statistics::population_std(&drawdowns).unwrap_or(0.0),
                ),
                benchmark_excess_pct: oos.benchmark_buy_hold_excess_pct,
                overfitting_gap_pct: rounded(overfitting_gap),
                possible_overfitting,
                robustness_status: status.into(),
            }
        })
        .collect()
}

fn persist_reports(
    connection: &Connection,
    validation_id: &str,
    reports: &[MarketRobustnessReport],
) -> Result<(), String> {
    for report in reports {
        connection.execute("INSERT INTO market_robustness_reports(validation_run_id,agent_id,in_sample_return_pct,validation_return_pct,out_of_sample_return_pct,out_of_sample_drawdown_pct,positive_window_ratio_pct,average_window_return_pct,median_window_return_pct,best_window_return_pct,worst_window_return_pct,return_std_across_windows,drawdown_std_across_windows,benchmark_excess_pct,overfitting_gap_pct,possible_overfitting,robustness_status,rule_version) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)", params![validation_id,report.agent_id,report.in_sample_return_pct,report.validation_return_pct,report.out_of_sample_return_pct,report.out_of_sample_drawdown_pct,report.positive_window_ratio_pct,report.average_window_return_pct,report.median_window_return_pct,report.best_window_return_pct,report.worst_window_return_pct,report.return_std_across_windows,report.drawdown_std_across_windows,report.benchmark_excess_pct,report.overfitting_gap_pct,report.possible_overfitting,report.robustness_status,ROBUSTNESS_RULE_VERSION]).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn run(
    connection: &mut Connection,
    input: &MarketValidationInput,
) -> Result<MarketValidationResult, String> {
    let base = experiment_input(input);
    let (ai_config,endpoint)=market_repository::resolve_ai_config(connection,&base)?;
    if input.agent_ids.iter().any(|id|market_ai::is_ai_agent(id)){let health=tauri::async_runtime::block_on(crate::ollama::status(&endpoint,(ai_config.timeout_ms/1000).clamp(5,8)))?;if health.available&&health.models.iter().any(|model|model==&ai_config.model){let provider=market_ai::OllamaMarketAiProvider::new(endpoint);run_with_provider(connection,input,&provider,&ai_config)}else{run_with_provider(connection,input,&market_ai::UnavailableMarketAiProvider,&ai_config)}}else{run_with_provider(connection,input,&market_ai::UnavailableMarketAiProvider,&ai_config)}
}

pub(crate) fn run_with_provider(
    connection: &mut Connection,
    input: &MarketValidationInput,
    provider: &dyn market_ai::MarketAiProvider,
    ai_config: &market_ai::MarketAiConfig,
) -> Result<MarketValidationResult, String> {
    let base = experiment_input(input);
    let (dataset, risk, agents, candles) = market_repository::validate_input(connection, &base)?;
    let (splits, walk_forward) = validate(input, candles.len())?;
    let id = new_id();
    let frozen_agents: Vec<_> = agents.iter().map(|agent| json!({"id":agent.id,"strategyType":agent.strategy_type,"strategyVersion":agent.strategy_version,"parameters":serde_json::from_str::<serde_json::Value>(&agent.default_config_json).unwrap_or_default()})).collect();
    let frozen = json!({"datasetHash":dataset.fingerprint,"agents":frozen_agents,"riskProfile":risk,"initialCapital":input.initial_capital,"seed":input.random_seed,"feePct":input.fee_pct,"slippagePct":input.slippage_pct,"executionTiming":"decision-close-T/execution-open-T+1","riskFreeRate":0.0,"aiConfig":ai_config});
    connection.execute("INSERT INTO market_validation_runs(id,name,dataset_id,dataset_hash,risk_profile_id,agent_ids_json,frozen_config_json,split_config_json,walk_forward_config_json,regime_config_json,rolling_window,annualization_factor,validation_engine_version,metric_formula_version,regime_engine_version,status,started_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,'running',CURRENT_TIMESTAMP)", params![id,input.name.trim(),dataset.id,dataset.fingerprint,risk.id,serde_json::to_string(&input.agent_ids).map_err(|error|error.to_string())?,frozen.to_string(),serde_json::to_string(&input.split_config).map_err(|error|error.to_string())?,serde_json::to_string(&input.walk_forward_config).map_err(|error|error.to_string())?,serde_json::to_string(&input.regime_config).map_err(|error|error.to_string())?,input.rolling_window,input.annualization_factor,VALIDATION_ENGINE_VERSION,market_statistics::METRIC_FORMULA_VERSION,market_regimes::REGIME_ENGINE_VERSION]).map_err(|error|error.to_string())?;

    let execution = (|| -> Result<bool, String> {
        let labels = market_regimes::classify_all(
            &candles
                .iter()
                .map(|candle| candle.close)
                .collect::<Vec<_>>(),
            &input.regime_config,
        );
        persist_regimes(connection, &id, &candles, &labels)?;
        let mut all_metrics = Vec::new();
        let mut ai_runtime = market_ai::MarketAiRuntimeMetrics::default();
        for spec in splits.iter().chain(walk_forward.iter()) {
            if market_repository::kill_switch_active() {
                connection.execute("UPDATE market_validation_runs SET status='aborted',completed_at=CURRENT_TIMESTAMP,error='Kill Switch acionado' WHERE id=?1", [&id]).map_err(|error| error.to_string())?;
                return Ok(false);
            }
            let window_id = persist_window(connection, &id, spec, &candles)?;
            let simulations = market_repository::simulate_range_with_provider(
                &dataset, &risk, &agents, &candles, spec.start, spec.end, &base, provider, ai_config,
            )?;
            if let Some(run)=simulations.iter().find(|run|market_ai::is_ai_agent(&run.id)){ai_runtime.merge(&run.ai_runtime);}
            let buy_hold = benchmark_buy_hold(&candles, spec.start, spec.end);
            for run in &simulations {
                let mut metric = metric_for_run(
                    run,
                    input.initial_capital,
                    input.annualization_factor,
                    buy_hold,
                );
                metric.window_id = window_id;
                metric.window_index = spec.index;
                metric.window_type = spec.kind.into();
                persist_metric(connection, &id, window_id, &metric)?;
                all_metrics.push(metric);
            }
        }
        let full_runs = market_repository::simulate_range_with_provider(
            &dataset,
            &risk,
            &agents,
            &candles,
            0,
            candles.len() - 1,
            &base,
            provider,
            ai_config,
        )?;
        if let Some(run)=full_runs.iter().find(|run|market_ai::is_ai_agent(&run.id)){ai_runtime.merge(&run.ai_runtime);}
        persist_rolling(
            connection,
            &id,
            &full_runs,
            input.rolling_window,
            input.annualization_factor,
        )?;
        persist_regime_metrics(connection, &id, &full_runs, &labels, &input.regime_config)?;
        let reports = build_reports(&all_metrics, &agents, &risk);
        persist_reports(connection, &id, &reports)?;
        if let Some(agent)=agents.iter().find(|agent|market_ai::is_ai_agent(&agent.id)){let distribution=ai_runtime.confidence_distribution();connection.execute("INSERT INTO market_validation_ai_runtime_metrics(validation_run_id,agent_id,call_count,successful_call_count,invalid_response_count,timeout_count,retry_count,fallback_count,average_latency_ms,max_latency_ms,total_latency_ms,input_tokens,output_tokens,prompt_version,buy_count,sell_count,llm_hold_count,no_llm_call_count,average_confidence,average_buy_confidence,average_sell_confidence,average_hold_confidence,min_confidence,max_confidence,median_confidence,confidence_00_20,confidence_20_40,confidence_40_60,confidence_60_80,confidence_80_100) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30)",params![id,agent.id,ai_runtime.call_count,ai_runtime.successful_call_count,ai_runtime.invalid_response_count,ai_runtime.timeout_count,ai_runtime.retry_count,ai_runtime.fallback_count,ai_runtime.average_latency_ms,ai_runtime.max_latency_ms,ai_runtime.total_latency_ms,ai_runtime.input_tokens,ai_runtime.output_tokens,ai_config.prompt_version,ai_runtime.buy_count,ai_runtime.sell_count,ai_runtime.llm_hold_count,ai_runtime.no_llm_call_count,ai_runtime.average_confidence(),ai_runtime.average_buy_confidence(),ai_runtime.average_sell_confidence(),ai_runtime.average_hold_confidence(),ai_runtime.min_confidence(),ai_runtime.max_confidence(),ai_runtime.median_confidence(),distribution[0],distribution[1],distribution[2],distribution[3],distribution[4]]).map_err(|error|error.to_string())?;}
        Ok(true)
    })();

    match execution {
        Ok(false) => get(connection, &id),
        Ok(true) => {
            connection.execute("UPDATE market_validation_runs SET status='completed',completed_at=CURRENT_TIMESTAMP WHERE id=?1", [&id]).map_err(|error| error.to_string())?;
            get(connection, &id)
        }
        Err(error) => {
            let status_result = connection.execute(
                "UPDATE market_validation_runs SET status='failed',completed_at=CURRENT_TIMESTAMP,error=?2 WHERE id=?1",
                params![id, error],
            );
            match status_result {
                Ok(_) => Err(error),
                Err(status_error) => Err(format!(
                    "{error}; falha adicional ao persistir status FAILED: {status_error}"
                )),
            }
        }
    }
}

pub fn list(connection: &Connection) -> Result<Vec<MarketValidationSummary>, String> {
    let mut statement = connection.prepare("SELECT v.id,v.name,v.dataset_id,d.name,v.dataset_hash,v.risk_profile_id,v.agent_ids_json,v.status,v.error,v.started_at,v.completed_at,v.created_at FROM market_validation_runs v JOIN market_datasets d ON d.id=v.dataset_id ORDER BY v.created_at DESC").map_err(|error| error.to_string())?;
    let result = statement
        .query_map([], |row| {
            Ok(MarketValidationSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                dataset_id: row.get(2)?,
                dataset_name: row.get(3)?,
                dataset_hash: row.get(4)?,
                risk_profile_id: row.get(5)?,
                agent_ids: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
                status: row.get(7)?,
                error: row.get(8)?,
                started_at: row.get(9)?,
                completed_at: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string());
    result
}

pub fn get(connection: &Connection, id: &str) -> Result<MarketValidationResult, String> {
    let validation = list(connection)?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or("validation run não encontrada")?;
    let audit = connection.query_row("SELECT frozen_config_json,split_config_json,walk_forward_config_json,regime_config_json,rolling_window,annualization_factor,validation_engine_version,metric_formula_version,regime_engine_version FROM market_validation_runs WHERE id=?1",[id],|row|Ok(MarketValidationAudit{frozen_config_json:row.get(0)?,split_config_json:row.get(1)?,walk_forward_config_json:row.get(2)?,regime_config_json:row.get(3)?,rolling_window:row.get(4)?,annualization_factor:row.get(5)?,validation_engine_version:row.get(6)?,metric_formula_version:row.get(7)?,regime_engine_version:row.get(8)?})).map_err(|error|error.to_string())?;
    let names: HashMap<String, String> = market_repository::list_agents(connection)?
        .into_iter()
        .map(|agent| (agent.id, agent.name))
        .collect();
    let mut statement=connection.prepare("SELECT id,window_index,window_type,start_index,end_index,start_at,end_at,train_start_index,train_end_index,train_start_at,train_end_at FROM market_validation_windows WHERE validation_run_id=?1 ORDER BY CASE window_type WHEN 'in_sample' THEN 0 WHEN 'validation' THEN 1 WHEN 'out_of_sample' THEN 2 ELSE 3 END,window_index").map_err(|error|error.to_string())?;
    let windows = statement
        .query_map([id], |row| {
            Ok(MarketValidationWindow {
                id: row.get(0)?,
                window_index: row.get(1)?,
                window_type: row.get(2)?,
                start_index: row.get(3)?,
                end_index: row.get(4)?,
                start_at: row.get(5)?,
                end_at: row.get(6)?,
                train_start_index: row.get(7)?,
                train_end_index: row.get(8)?,
                train_start_at: row.get(9)?,
                train_end_at: row.get(10)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut statement=connection.prepare("SELECT m.window_id,w.window_index,w.window_type,m.agent_id,m.total_return_pct,m.max_drawdown_pct,m.sharpe,m.sortino,m.calmar,m.trade_count,m.hold_rate_pct,m.exposure_pct,m.benchmark_cash_excess_pct,m.benchmark_buy_hold_excess_pct FROM market_agent_validation_metrics m JOIN market_validation_windows w ON w.id=m.window_id WHERE m.validation_run_id=?1 ORDER BY w.window_type,w.window_index,m.agent_id").map_err(|error|error.to_string())?;
    let metrics = statement
        .query_map([id], |row| {
            let agent_id: String = row.get(3)?;
            Ok(MarketWindowMetric {
                window_id: row.get(0)?,
                window_index: row.get(1)?,
                window_type: row.get(2)?,
                agent_name: names
                    .get(&agent_id)
                    .cloned()
                    .unwrap_or_else(|| agent_id.clone()),
                agent_id,
                total_return_pct: row.get(4)?,
                max_drawdown_pct: row.get(5)?,
                sharpe: row.get(6)?,
                sortino: row.get(7)?,
                calmar: row.get(8)?,
                trade_count: row.get(9)?,
                hold_rate_pct: row.get(10)?,
                exposure_pct: row.get(11)?,
                benchmark_cash_excess_pct: row.get(12)?,
                benchmark_buy_hold_excess_pct: row.get(13)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut statement=connection.prepare("SELECT agent_id,regime_type,regime,candle_count,total_return_pct,max_drawdown_pct,trade_count,hold_rate_pct,exposure_pct,profit_factor,win_rate_pct,low_sample_size FROM market_regime_metrics WHERE validation_run_id=?1 ORDER BY regime_type,regime,agent_id").map_err(|error|error.to_string())?;
    let regime_metrics = statement
        .query_map([id], |row| {
            Ok(MarketRegimeMetric {
                agent_id: row.get(0)?,
                regime_type: row.get(1)?,
                regime: row.get(2)?,
                candle_count: row.get(3)?,
                total_return_pct: row.get(4)?,
                max_drawdown_pct: row.get(5)?,
                trade_count: row.get(6)?,
                hold_rate_pct: row.get(7)?,
                exposure_pct: row.get(8)?,
                profit_factor: row.get(9)?,
                win_rate_pct: row.get(10)?,
                low_sample_size: row.get(11)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut statement=connection.prepare("SELECT agent_id,candle_index,timestamp,rolling_return_pct,rolling_volatility_pct,rolling_sharpe,rolling_drawdown_pct FROM market_rolling_metrics WHERE validation_run_id=?1 ORDER BY candle_index,agent_id").map_err(|error|error.to_string())?;
    let rolling_metrics = statement
        .query_map([id], |row| {
            Ok(MarketRollingMetric {
                agent_id: row.get(0)?,
                candle_index: row.get(1)?,
                timestamp: row.get(2)?,
                rolling_return_pct: row.get(3)?,
                rolling_volatility_pct: row.get(4)?,
                rolling_sharpe: row.get(5)?,
                rolling_drawdown_pct: row.get(6)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut statement=connection.prepare("SELECT agent_id,in_sample_return_pct,validation_return_pct,out_of_sample_return_pct,out_of_sample_drawdown_pct,positive_window_ratio_pct,average_window_return_pct,median_window_return_pct,best_window_return_pct,worst_window_return_pct,return_std_across_windows,drawdown_std_across_windows,benchmark_excess_pct,overfitting_gap_pct,possible_overfitting,robustness_status FROM market_robustness_reports WHERE validation_run_id=?1 ORDER BY agent_id").map_err(|error|error.to_string())?;
    let reports = statement
        .query_map([id], |row| {
            let agent_id: String = row.get(0)?;
            Ok(MarketRobustnessReport {
                agent_name: names
                    .get(&agent_id)
                    .cloned()
                    .unwrap_or_else(|| agent_id.clone()),
                agent_id,
                in_sample_return_pct: row.get(1)?,
                validation_return_pct: row.get(2)?,
                out_of_sample_return_pct: row.get(3)?,
                out_of_sample_drawdown_pct: row.get(4)?,
                positive_window_ratio_pct: row.get(5)?,
                average_window_return_pct: row.get(6)?,
                median_window_return_pct: row.get(7)?,
                best_window_return_pct: row.get(8)?,
                worst_window_return_pct: row.get(9)?,
                return_std_across_windows: row.get(10)?,
                drawdown_std_across_windows: row.get(11)?,
                benchmark_excess_pct: row.get(12)?,
                overfitting_gap_pct: row.get(13)?,
                possible_overfitting: row.get(14)?,
                robustness_status: row.get(15)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(MarketValidationResult {
        validation,
        windows,
        metrics,
        regime_metrics,
        rolling_metrics,
        reports,
        audit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_split_is_ordered_and_has_no_overlap() {
        let windows = temporal_windows(
            100,
            &ValidationSplitConfig {
                in_sample_pct: 60,
                validation_pct: 20,
                out_of_sample_pct: 20,
            },
        )
        .unwrap();
        assert_eq!((windows[0].start, windows[0].end), (0, 59));
        assert_eq!((windows[1].start, windows[1].end), (60, 79));
        assert_eq!((windows[2].start, windows[2].end), (80, 99));
        assert!(windows[0].end < windows[1].start && windows[1].end < windows[2].start);
    }

    #[test]
    fn invalid_split_is_rejected() {
        assert!(temporal_windows(
            100,
            &ValidationSplitConfig {
                in_sample_pct: 60,
                validation_pct: 30,
                out_of_sample_pct: 20
            }
        )
        .is_err());
    }

    #[test]
    fn walk_forward_steps_without_future_leakage() {
        let windows = walk_forward_windows(
            20,
            &WalkForwardConfig {
                train_window_size: 8,
                test_window_size: 4,
                step_size: 4,
            },
        )
        .unwrap();
        assert_eq!(windows.len(), 3);
        assert_eq!(
            (
                windows[0].train_start,
                windows[0].train_end,
                windows[0].start,
                windows[0].end
            ),
            (Some(0), Some(7), 8, 11)
        );
        assert_eq!(
            (
                windows[1].train_start,
                windows[1].train_end,
                windows[1].start,
                windows[1].end
            ),
            (Some(4), Some(11), 12, 15)
        );
        assert!(windows
            .iter()
            .all(|window| window.train_end.unwrap() < window.start));
    }

    #[test]
    fn temporal_split_assigns_rounding_remainder_to_oos_without_gaps() {
        let windows = temporal_windows(
            101,
            &ValidationSplitConfig {
                in_sample_pct: 60,
                validation_pct: 20,
                out_of_sample_pct: 20,
            },
        )
        .unwrap();
        assert_eq!(windows[0].end + 1, windows[1].start);
        assert_eq!(windows[1].end + 1, windows[2].start);
        assert_eq!(windows[2].end, 100);
    }

    fn agent() -> MarketAgentDefinition {
        MarketAgentDefinition {
            id: "agent".into(),
            name: "Agent".into(),
            strategy_type: "test".into(),
            strategy_version: "1".into(),
            default_config_json: "{}".into(),
            enabled: true,
        }
    }
    fn risk() -> MarketRiskProfile {
        MarketRiskProfile {
            id: "risk".into(),
            name: "Risk".into(),
            max_position_pct: 50.0,
            max_total_exposure_pct: 50.0,
            max_daily_loss_pct: 5.0,
            max_drawdown_pct: 20.0,
            max_trades_per_day: None,
            allow_leverage: false,
            allow_short: false,
            allowed_assets: vec![],
        }
    }
    fn metric(
        kind: &str,
        index: usize,
        return_pct: f64,
        drawdown: f64,
        benchmark: f64,
    ) -> MarketWindowMetric {
        MarketWindowMetric {
            window_id: index as i64,
            window_index: index,
            window_type: kind.into(),
            agent_id: "agent".into(),
            agent_name: "Agent".into(),
            total_return_pct: return_pct,
            max_drawdown_pct: drawdown,
            sharpe: Some(1.0),
            sortino: Some(1.0),
            calmar: Some(1.0),
            trade_count: 3,
            hold_rate_pct: 50.0,
            exposure_pct: 25.0,
            benchmark_cash_excess_pct: return_pct,
            benchmark_buy_hold_excess_pct: benchmark,
        }
    }

    #[test]
    fn robustness_candidate_requires_oos_windows_drawdown_and_benchmark() {
        let metrics = vec![
            metric("in_sample", 0, 5.0, 3.0, 1.0),
            metric("validation", 0, 4.0, 3.0, 1.0),
            metric("out_of_sample", 0, 3.0, 4.0, 1.0),
            metric("walk_forward", 0, 1.0, 2.0, 0.5),
            metric("walk_forward", 1, 2.0, 2.0, 0.5),
            metric("walk_forward", 2, -0.5, 2.0, -0.5),
        ];
        let report = build_reports(&metrics, &[agent()], &risk()).remove(0);
        assert_eq!(report.robustness_status, "robust_candidate");
        assert!(!report.possible_overfitting);
    }

    #[test]
    fn strong_is_and_collapsed_oos_raise_overfitting_warning() {
        let metrics = vec![
            metric("in_sample", 0, 25.0, 3.0, 20.0),
            metric("validation", 0, 10.0, 4.0, 5.0),
            metric("out_of_sample", 0, -2.0, 12.0, -5.0),
            metric("walk_forward", 0, 3.0, 3.0, 1.0),
            metric("walk_forward", 1, -4.0, 9.0, -4.0),
            metric("walk_forward", 2, -2.0, 7.0, -2.0),
        ];
        let report = build_reports(&metrics, &[agent()], &risk()).remove(0);
        assert!(report.possible_overfitting);
        assert_eq!(report.robustness_status, "unstable");
        assert_eq!(report.overfitting_gap_pct, 27.0);
    }
}
