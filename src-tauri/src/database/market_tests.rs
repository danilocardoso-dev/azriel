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
