use super::{market_models::*, market_repository, *};
use rusqlite::Connection;
use std::{fs, time::{SystemTime, UNIX_EPOCH}};

fn fixture_csv() -> String {
    let mut output = String::from("timestamp,open,high,low,close,volume\n");
    for day in 1..=30 {
        let open = 99.0 + day as f64;
        let close = open + if day % 4 == 0 { -0.5 } else { 0.75 };
        output.push_str(&format!("2026-01-{day:02}T00:00:00Z,{open},{},{},{close},1000\n", open + 2.0, open - 2.0));
    }
    output
}

fn database() -> Connection {
    let mut connection = Connection::open_in_memory().unwrap();
    initialize(&mut connection).unwrap();
    connection
}

#[test]
fn complete_market_flow_is_persistent_auditable_and_reproducible() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("azriel-market-{suffix}.csv"));
    fs::write(&path, fixture_csv()).unwrap();
    let mut connection = database();
    let import = || ImportMarketDatasetInput { path: path.to_string_lossy().into_owned(), name: "Fixture".into(), asset: "TST".into(), timeframe: "1D".into(), currency: Some("BRL".into()) };
    let dataset = market_repository::import_dataset(&mut connection, &import()).unwrap();
    let duplicate = market_repository::import_dataset(&mut connection, &import()).unwrap();
    assert_eq!(dataset.id, duplicate.id);
    let input = MarketExperimentInput { name: "Determinism".into(), dataset_id: dataset.id, risk_profile_id: "balanced-v1".into(), agent_ids: vec!["cash".into(), "buy-hold".into(), "simple-trend".into(), "simple-momentum".into(), "random-controlled".into()], initial_capital: 10_000.0, random_seed: 42, fee_pct: 0.1, slippage_pct: 0.05 };
    let first = market_repository::run_experiment(&mut connection, &input).unwrap();
    let second = market_repository::run_experiment(&mut connection, &input).unwrap();
    assert_eq!(first.metrics.len(), 5);
    assert_eq!(first.observatory.behavior.len(), 5);
    assert_eq!(first.observatory.correlations.len(), 15);
    assert_eq!(first.observatory.benchmarks.len(), 5);
    let cash_behavior = first.observatory.behavior.iter().find(|metric| metric.agent_id == "cash").unwrap();
    assert_eq!(cash_behavior.hold_rate_pct, 100.0);
    assert_eq!(cash_behavior.time_in_cash_pct, 100.0);
    assert_eq!(cash_behavior.turnover_pct, 0.0);
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM market_behavior_metrics WHERE experiment_id=?1", [&first.experiment.id], |row| row.get::<_, i64>(0)).unwrap(), 5);
    assert_eq!(first.metrics.iter().map(|m| (m.agent_id.as_str(), m.final_equity, m.trade_count)).collect::<Vec<_>>(), second.metrics.iter().map(|m| (m.agent_id.as_str(), m.final_equity, m.trade_count)).collect::<Vec<_>>());
    let cash = first.metrics.iter().find(|metric| metric.agent_id == "cash").unwrap();
    assert_eq!(cash.final_equity, 10_000.0);
    assert_eq!(cash.trade_count, 0);
    assert_eq!(cash.hold_count, 30);
    assert!(first.decisions.iter().any(|decision| decision.action == "HOLD"));
    assert!(first.decisions.iter().any(|decision| decision.execution_price.is_some()));
    assert_eq!(market_repository::list_experiments(&connection).unwrap().len(), 2);
    let _ = fs::remove_file(path);
}

#[test]
fn six_agents_are_rejected_before_an_experiment_is_created() {
    let mut connection = database();
    let input = MarketExperimentInput { name: "Rejected".into(), dataset_id: "missing".into(), risk_profile_id: "balanced-v1".into(), agent_ids: vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into(), "6".into()], initial_capital: 100.0, random_seed: 1, fee_pct: 0.0, slippage_pct: 0.0 };
    let error = market_repository::run_experiment(&mut connection, &input).unwrap_err();
    assert!(error.contains("runtime limit: 5"));
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM market_experiments", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
}

#[test]
fn cohort_requires_at_least_three_agents() {
    let mut connection = database();
    let input = MarketExperimentInput { name: "Rejected".into(), dataset_id: "missing".into(), risk_profile_id: "balanced-v1".into(), agent_ids: vec!["cash".into(), "buy-hold".into()], initial_capital: 100.0, random_seed: 1, fee_pct: 0.0, slippage_pct: 0.0 };
    let error = market_repository::run_experiment(&mut connection, &input).unwrap_err();
    assert!(error.contains("entre 3 e 5"));
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM market_experiments", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
}

#[test]
fn cohorts_support_three_four_and_five_agents_with_equal_starting_capital() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("azriel-market-cohorts-{suffix}.csv"));
    fs::write(&path, fixture_csv()).unwrap();
    let mut connection = database();
    let dataset = market_repository::import_dataset(&mut connection, &ImportMarketDatasetInput { path: path.to_string_lossy().into_owned(), name: "Cohorts".into(), asset: "TST".into(), timeframe: "1D".into(), currency: Some("BRL".into()) }).unwrap();
    let registry = ["cash", "buy-hold", "simple-trend", "simple-momentum", "random-controlled"];
    for size in 3..=5 {
        let result = market_repository::run_experiment(&mut connection, &MarketExperimentInput { name: format!("Cohort {size}"), dataset_id: dataset.id.clone(), risk_profile_id: "balanced-v1".into(), agent_ids: registry[..size].iter().map(|id| (*id).into()).collect(), initial_capital: 50.0, random_seed: 42, fee_pct: 0.1, slippage_pct: 0.05 }).unwrap();
        assert_eq!(result.metrics.len(), size);
        assert_eq!(result.observatory.behavior.len(), size);
        assert_eq!(result.observatory.correlations.len(), size * (size + 1) / 2);
        assert!(result.equity.iter().filter(|point| point.timestamp == "2026-01-01T00:00:00Z").all(|point| point.equity == 50.0));
        assert!(result.observatory.correlations.iter().filter(|cell| cell.agent_a_id == cell.agent_b_id).all(|cell| cell.equity_return_correlation == Some(1.0) && cell.decision_similarity == 1.0));
    }
    let _ = fs::remove_file(path);
}
