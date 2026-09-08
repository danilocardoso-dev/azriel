use super::{market_ai, market_models::*, market_repository, *};
use rusqlite::Connection;
use std::{fs, sync::Mutex, time::{SystemTime, UNIX_EPOCH}};

struct FakeMarketAiProvider { prompts: Mutex<Vec<String>> }
impl market_ai::MarketAiProvider for FakeMarketAiProvider {
    fn complete(&self,prompt:market_ai::MarketAiPrompt,_:&market_ai::MarketAiConfig)->Result<market_ai::MarketAiProviderResponse,market_ai::MarketAiProviderError>{self.prompts.lock().unwrap().push(prompt.snapshot_json);Ok(market_ai::MarketAiProviderResponse{content:r#"{"action":"BUY","desired_position_pct":90,"confidence":0.99,"reason":"strong signal"}"#.into(),model:"fake:market".into(),input_tokens:Some(20),output_tokens:Some(10)})}
}

fn fake_ai_config()->market_ai::MarketAiConfig{market_ai::MarketAiConfig{agent_id:market_ai::AI_AGENT_ID.into(),provider:"ollama".into(),model:"fake:market".into(),prompt_version:market_ai::PROMPT_VERSION.into(),temperature:0.1,decision_interval:5,timeout_ms:5000,max_retries:1}}

struct CalibratedFakeProvider { prompts: Mutex<Vec<market_ai::MarketAiPrompt>> }
impl market_ai::MarketAiProvider for CalibratedFakeProvider {
    fn complete(&self,prompt:market_ai::MarketAiPrompt,_:&market_ai::MarketAiConfig)->Result<market_ai::MarketAiProviderResponse,market_ai::MarketAiProviderError>{
        let snapshot:serde_json::Value=serde_json::from_str(&prompt.snapshot_json).unwrap();
        let positioned=snapshot.pointer("/portfolio/current_position").and_then(|value|value.as_bool()).unwrap_or(false);
        let content=if positioned{r#"{"action":"SELL","desired_position_pct":0,"confidence":0.72,"reason":"trend evidence deteriorated"}"#}else{r#"{"action":"BUY","desired_position_pct":80,"confidence":0.68,"reason":"trend and momentum evidence converge"}"#};
        self.prompts.lock().unwrap().push(prompt);
        Ok(market_ai::MarketAiProviderResponse{content:content.into(),model:"fake:qwen-v2".into(),input_tokens:Some(30),output_tokens:Some(12)})
    }
}

fn fake_ai_v2_config()->market_ai::MarketAiConfig{market_ai::MarketAiConfig{agent_id:market_ai::AI_AGENT_V2_ID.into(),provider:"ollama".into(),model:"fake:qwen-v2".into(),prompt_version:market_ai::PROMPT_V2_VERSION.into(),temperature:0.1,decision_interval:5,timeout_ms:5000,max_retries:1}}

struct SignalAwareFakeProvider { prompts:Mutex<Vec<market_ai::MarketAiPrompt>> }
impl market_ai::MarketAiProvider for SignalAwareFakeProvider { fn complete(&self,prompt:market_ai::MarketAiPrompt,_:&market_ai::MarketAiConfig)->Result<market_ai::MarketAiProviderResponse,market_ai::MarketAiProviderError>{let snapshot:serde_json::Value=serde_json::from_str(&prompt.snapshot_json).unwrap();let bias=snapshot.pointer("/signals/directionalBias").and_then(|value|value.as_str()).unwrap_or("NEUTRAL");let positioned=snapshot.pointer("/portfolio/current_position").and_then(|value|value.as_bool()).unwrap_or(false);let content=match(bias,positioned){("BULLISH",false)=>r#"{"action":"BUY","desired_position_pct":40,"confidence":0.74,"reason_code":"ALIGNED_BULLISH_SIGNAL","reason":"aligned bullish evidence"}"#,("BEARISH",true)=>r#"{"action":"SELL","desired_position_pct":0,"confidence":0.71,"reason_code":"EXIT_SIGNAL","reason":"bearish exit evidence"}"#,_=>r#"{"action":"HOLD","desired_position_pct":0,"confidence":0.46,"reason_code":"INSUFFICIENT_SIGNAL","reason":"signal not actionable"}"#};self.prompts.lock().unwrap().push(prompt);Ok(market_ai::MarketAiProviderResponse{content:content.into(),model:"fake:qwen-v3".into(),input_tokens:Some(40),output_tokens:Some(14)})}}
fn fake_ai_v3_config()->market_ai::MarketAiConfig{market_ai::MarketAiConfig{agent_id:market_ai::AI_AGENT_V3_ID.into(),provider:"ollama".into(),model:"fake:qwen-v3".into(),prompt_version:market_ai::PROMPT_V3_VERSION.into(),temperature:0.1,decision_interval:5,timeout_ms:5000,max_retries:1}}

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

#[test]
fn scientific_validation_is_persistent_frozen_and_reproducible() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("azriel-market-validation-{suffix}.csv"));
    fs::write(&path, fixture_csv()).unwrap();
    let mut connection = database();
    let dataset = market_repository::import_dataset(&mut connection, &ImportMarketDatasetInput { path: path.to_string_lossy().into_owned(), name: "Validation".into(), asset: "TST".into(), timeframe: "1D".into(), currency: Some("BRL".into()) }).unwrap();
    let input = MarketValidationInput {
        name: "VAL-001".into(), dataset_id: dataset.id, risk_profile_id: "balanced-v1".into(),
        agent_ids: vec!["cash".into(), "buy-hold".into(), "simple-trend".into(), "simple-momentum".into(), "random-controlled".into()],
        initial_capital: 50.0, random_seed: 42, fee_pct: 0.1, slippage_pct: 0.05,
        split_config: ValidationSplitConfig { in_sample_pct: 60, validation_pct: 20, out_of_sample_pct: 20 },
        walk_forward_config: WalkForwardConfig { train_window_size: 12, test_window_size: 6, step_size: 6 },
        regime_config: MarketRegimeConfig { trend_window: 5, volatility_window: 5, bull_threshold_pct: 2.0, bear_threshold_pct: -2.0, high_volatility_threshold_pct: 1.5, low_volatility_threshold_pct: 0.25, minimum_sample_candles: 3, minimum_sample_trades: 1 },
        rolling_window: 5, annualization_factor: 252.0,
    };
    let first = market_validation::run(&mut connection, &input).unwrap();
    let second = market_validation::run(&mut connection, &input).unwrap();
    assert_eq!(first.validation.status, "completed");
    assert_eq!(first.windows.len(), 6);
    assert_eq!(first.reports.len(), 5);
    assert_eq!(first.regime_metrics.len(), 30);
    assert_eq!(first.metrics.len(), 30);
    assert_eq!(first.audit.validation_engine_version, "VALIDATION_ENGINE_V1");
    assert!(first.audit.frozen_config_json.contains("strategyVersion"));
    assert!(first.regime_metrics.iter().any(|metric| metric.low_sample_size));
    assert!(first.metrics.iter().flat_map(|metric| [metric.sharpe,metric.sortino,metric.calmar]).flatten().all(|value| value.is_finite()));
    assert!(first.metrics.iter().filter(|metric| metric.agent_id=="cash").all(|metric| metric.total_return_pct==0.0));
    assert_eq!(first.reports.iter().map(|report| (report.agent_id.as_str(), report.out_of_sample_return_pct, report.positive_window_ratio_pct, report.robustness_status.as_str())).collect::<Vec<_>>(), second.reports.iter().map(|report| (report.agent_id.as_str(), report.out_of_sample_return_pct, report.positive_window_ratio_pct, report.robustness_status.as_str())).collect::<Vec<_>>());
    assert_eq!(market_validation::list(&connection).unwrap().len(), 2);
    let _ = fs::remove_file(path);
}

#[test]
fn scientific_validation_persists_failed_status_after_runtime_error() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("azriel-market-validation-failure-{suffix}.csv"));
    fs::write(&path, fixture_csv()).unwrap();
    let mut connection = database();
    let dataset = market_repository::import_dataset(&mut connection, &ImportMarketDatasetInput { path: path.to_string_lossy().into_owned(), name: "Validation failure".into(), asset: "TST".into(), timeframe: "1D".into(), currency: Some("BRL".into()) }).unwrap();
    connection.execute_batch("CREATE TRIGGER force_regime_failure BEFORE INSERT ON market_regimes BEGIN SELECT RAISE(ABORT, 'forced regime failure'); END;").unwrap();
    let input = MarketValidationInput {
        name: "VAL-FAIL".into(), dataset_id: dataset.id, risk_profile_id: "balanced-v1".into(),
        agent_ids: vec!["cash".into(), "buy-hold".into(), "simple-trend".into()],
        initial_capital: 50.0, random_seed: 42, fee_pct: 0.1, slippage_pct: 0.05,
        split_config: ValidationSplitConfig { in_sample_pct: 60, validation_pct: 20, out_of_sample_pct: 20 },
        walk_forward_config: WalkForwardConfig { train_window_size: 12, test_window_size: 6, step_size: 6 },
        regime_config: MarketRegimeConfig { trend_window: 5, volatility_window: 5, bull_threshold_pct: 2.0, bear_threshold_pct: -2.0, high_volatility_threshold_pct: 1.5, low_volatility_threshold_pct: 0.25, minimum_sample_candles: 3, minimum_sample_trades: 1 },
        rolling_window: 5, annualization_factor: 252.0,
    };

    let error = market_validation::run(&mut connection, &input).unwrap_err();
    assert!(error.contains("forced regime failure"));
    let (status, persisted_error): (String, String) = connection.query_row(
        "SELECT status,error FROM market_validation_runs WHERE name='VAL-FAIL'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).unwrap();
    assert_eq!(status, "failed");
    assert!(persisted_error.contains("forced regime failure"));
    let _ = fs::remove_file(path);
}

#[test]
fn scientific_regime_fixture_imports_all_candles_in_temporal_order() {
    let suffix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("azriel-market-regimes-{suffix}.csv"));
    fs::write(&path, include_str!("../../../Docs/market-lab/fixtures/deterministic-regimes-160-candles.csv")).unwrap();
    let mut connection = database();
    let dataset = market_repository::import_dataset(&mut connection, &ImportMarketDatasetInput { path: path.to_string_lossy().into_owned(), name: "Regimes".into(), asset: "TST-REGIME".into(), timeframe: "1D".into(), currency: Some("BRL".into()) }).unwrap();
    assert_eq!(dataset.candle_count, 160);
    assert!(dataset.start_at < dataset.end_at);
    let _ = fs::remove_file(path);
}

#[test]
fn ai_agent_uses_structured_snapshots_risk_and_persistent_runtime() {
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path=std::env::temp_dir().join(format!("azriel-market-ai-{suffix}.csv"));
    fs::write(&path,fixture_csv()).unwrap();
    let mut connection=database();
    let dataset=market_repository::import_dataset(&mut connection,&ImportMarketDatasetInput{path:path.to_string_lossy().into_owned(),name:"AI fixture".into(),asset:"TST".into(),timeframe:"1D".into(),currency:Some("BRL".into())}).unwrap();
    let input=MarketExperimentInput{name:"AI-LAB-001".into(),dataset_id:dataset.id.clone(),risk_profile_id:"balanced-v1".into(),agent_ids:vec!["cash".into(),"buy-hold".into(),"simple-trend".into(),"simple-momentum".into(),market_ai::AI_AGENT_ID.into()],initial_capital:1_000.0,random_seed:42,fee_pct:0.1,slippage_pct:0.05};
    let provider=FakeMarketAiProvider{prompts:Mutex::new(Vec::new())};
    let result=market_repository::run_experiment_with_provider(&mut connection,&input,&provider,&fake_ai_config()).unwrap();
    assert_eq!(result.metrics.len(),5);
    assert!(result.equity.iter().filter(|point|point.timestamp=="2026-01-01T00:00:00Z").all(|point|point.equity==1_000.0));
    let runtime=market_repository::list_ai_runtime(&connection,&result.experiment.id).unwrap();
    assert_eq!(runtime[0].call_count,6);
    assert_eq!(runtime[0].successful_call_count,6);
    assert_eq!(runtime[0].fallback_count,0);
    assert_eq!(runtime[0].buy_count,6);
    assert_eq!(runtime[0].no_llm_call_count,24);
    assert_eq!(runtime[0].prompt_version,market_ai::PROMPT_VERSION);
    let decisions=market_repository::list_ai_decisions(&connection,&result.experiment.id).unwrap();
    assert_eq!(decisions.len(),30);
    assert_eq!(decisions.iter().filter(|decision|decision.call_status=="NO_LLM_CALL").count(),24);
    assert!(decisions.iter().filter(|decision|decision.call_status=="VALID").all(|decision|decision.approved_position_pct.unwrap_or(0.0)<=50.0));
    let prompts=provider.prompts.lock().unwrap();
    assert_eq!(prompts.len(),6);
    let first:serde_json::Value=serde_json::from_str(&prompts[0]).unwrap();
    assert_eq!(first["timestamp"],"2026-01-01T00:00:00Z");
    assert!(first.get("features").is_some());
    assert!(first.get("candles").is_none());
    assert!(decisions.iter().filter(|decision|decision.call_status=="VALID").all(|decision|decision.input_snapshot.is_some()));
    assert!(decisions.iter().filter(|decision|decision.call_status=="NO_LLM_CALL").all(|decision|decision.input_snapshot.is_none()));
    drop(prompts);
    let validation_input=MarketValidationInput{name:"AI-VAL-001".into(),dataset_id:dataset.id,risk_profile_id:"balanced-v1".into(),agent_ids:input.agent_ids.clone(),initial_capital:1_000.0,random_seed:42,fee_pct:0.1,slippage_pct:0.05,split_config:ValidationSplitConfig{in_sample_pct:60,validation_pct:20,out_of_sample_pct:20},walk_forward_config:WalkForwardConfig{train_window_size:12,test_window_size:6,step_size:6},regime_config:MarketRegimeConfig{trend_window:5,volatility_window:5,bull_threshold_pct:2.0,bear_threshold_pct:-2.0,high_volatility_threshold_pct:1.5,low_volatility_threshold_pct:0.25,minimum_sample_candles:3,minimum_sample_trades:1},rolling_window:5,annualization_factor:252.0};
    let validation=market_validation::run_with_provider(&mut connection,&validation_input,&provider,&fake_ai_config()).unwrap();
    assert_eq!(validation.validation.status,"completed");
    assert!(validation.reports.iter().any(|report|report.agent_id==market_ai::AI_AGENT_ID));
    let validation_runtime=market_repository::list_validation_ai_runtime(&connection,&validation.validation.id).unwrap();
    assert_eq!(validation_runtime.len(),1);
    assert!(validation_runtime[0].call_count>0);
    assert!(validation.audit.frozen_config_json.contains(market_ai::PROMPT_VERSION));
    let _=fs::remove_file(path);
}

#[test]
fn ai_v2_persists_rich_context_distribution_forward_returns_and_ab_metadata() {
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path=std::env::temp_dir().join(format!("azriel-market-ai-v2-{suffix}.csv"));
    fs::write(&path,fixture_csv()).unwrap();
    let mut connection=database();
    let dataset=market_repository::import_dataset(&mut connection,&ImportMarketDatasetInput{path:path.to_string_lossy().into_owned(),name:"AI calibration fixture".into(),asset:"AAPL".into(),timeframe:"1D".into(),currency:Some("USD".into())}).unwrap();
    let base_agents=vec!["cash".into(),"buy-hold".into(),"simple-trend".into()];
    let provider=CalibratedFakeProvider{prompts:Mutex::new(Vec::new())};
    let mut v2_agents=base_agents.clone();v2_agents.push(market_ai::AI_AGENT_V2_ID.into());
    let v2_input=MarketExperimentInput{name:"AI-CALIBRATION-001".into(),dataset_id:dataset.id.clone(),risk_profile_id:"balanced-v1".into(),agent_ids:v2_agents,initial_capital:10_000.0,random_seed:42,fee_pct:0.1,slippage_pct:0.05};
    let v2=market_repository::run_experiment_with_provider(&mut connection,&v2_input,&provider,&fake_ai_v2_config()).unwrap();
    let runtime=market_repository::list_ai_runtime(&connection,&v2.experiment.id).unwrap().remove(0);
    assert_eq!(runtime.call_count,6);
    assert_eq!(runtime.successful_call_count,6);
    assert_eq!(runtime.buy_count+runtime.sell_count+runtime.llm_hold_count,6);
    assert!(runtime.buy_count>0&&runtime.sell_count>0);
    assert_eq!(runtime.no_llm_call_count,24);
    assert_eq!(runtime.confidence_distribution.iter().sum::<usize>(),6);
    assert_eq!(runtime.prompt_version,market_ai::PROMPT_V2_VERSION);
    let decisions=market_repository::list_ai_decisions(&connection,&v2.experiment.id).unwrap();
    let called:Vec<_>=decisions.iter().filter(|decision|decision.call_status=="VALID").collect();
    assert_eq!(called.len(),6);
    assert!(called.iter().all(|decision|decision.input_snapshot.as_ref().and_then(|snapshot|snapshot.get("market")).is_some()));
    assert!(called.iter().all(|decision|!decision.input_snapshot.as_ref().unwrap().to_string().contains("forward_return")));
    assert!(called.iter().any(|decision|decision.forward_return_1.is_some()));
    assert!(called.iter().any(|decision|decision.forward_return_10.is_none()));
    assert!(called.iter().filter(|decision|decision.action=="BUY").all(|decision|decision.approved_position_pct.unwrap_or(0.0)<=50.0));
    let prompts=provider.prompts.lock().unwrap();
    assert!(prompts.iter().all(|prompt|prompt.system.contains("Do not require every indicator to agree")));
    assert!(prompts.iter().all(|prompt|!prompt.snapshot_json.contains("forward_return")));
    drop(prompts);

    let v1_provider=FakeMarketAiProvider{prompts:Mutex::new(Vec::new())};
    let mut v1_agents=base_agents;v1_agents.push(market_ai::AI_AGENT_ID.into());
    let v1_input=MarketExperimentInput{name:"AI-CALIBRATION-001 / V1".into(),agent_ids:v1_agents,dataset_id:dataset.id.clone(),..v2_input.clone()};
    market_repository::run_experiment_with_provider(&mut connection,&v1_input,&v1_provider,&fake_ai_config()).unwrap();
    let comparisons=market_repository::list_ai_experiment_comparisons(&connection).unwrap();
    assert_eq!(comparisons.len(),2);
    assert_eq!(comparisons[0].dataset_id,comparisons[1].dataset_id);
    assert_eq!(comparisons[0].decision_interval,comparisons[1].decision_interval);
    assert!(comparisons.iter().any(|entry|entry.prompt_version==market_ai::PROMPT_VERSION));
    assert!(comparisons.iter().any(|entry|entry.prompt_version==market_ai::PROMPT_V2_VERSION));

    let invalid=MarketExperimentInput{name:"invalid dual AI".into(),agent_ids:vec!["cash".into(),market_ai::AI_AGENT_ID.into(),market_ai::AI_AGENT_V2_ID.into()],dataset_id:dataset.id,risk_profile_id:"balanced-v1".into(),initial_capital:10_000.0,random_seed:42,fee_pct:0.1,slippage_pct:0.05};
    assert!(market_repository::validate_input(&connection,&invalid).unwrap_err().contains("no máximo um AI Agent"));
    let _=fs::remove_file(path);
}

#[test]
fn ai_v3_persists_signal_trace_reason_codes_diagnostics_and_frozen_config(){
    let suffix=SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();let path=std::env::temp_dir().join(format!("azriel-market-ai-v3-{suffix}.csv"));fs::write(&path,fixture_csv()).unwrap();let mut connection=database();
    let dataset=market_repository::import_dataset(&mut connection,&ImportMarketDatasetInput{path:path.to_string_lossy().into_owned(),name:"Signal fixture".into(),asset:"AAPL".into(),timeframe:"1D".into(),currency:Some("USD".into())}).unwrap();
    let input=MarketExperimentInput{name:"AI-DECISION-ARCH".into(),dataset_id:dataset.id.clone(),risk_profile_id:"balanced-v1".into(),agent_ids:vec!["cash".into(),"buy-hold".into(),"simple-trend".into(),market_ai::AI_AGENT_V3_ID.into()],initial_capital:10_000.0,random_seed:42,fee_pct:0.1,slippage_pct:0.05};let provider=SignalAwareFakeProvider{prompts:Mutex::new(Vec::new())};
    let result=market_repository::run_experiment_with_provider(&mut connection,&input,&provider,&fake_ai_v3_config()).unwrap();let decisions=market_repository::list_ai_decisions(&connection,&result.experiment.id).unwrap();let valid=decisions.iter().filter(|item|item.call_status=="VALID").collect::<Vec<_>>();assert_eq!(valid.len(),6);assert!(valid.iter().all(|item|item.reason_code.is_some()));assert!(valid.iter().all(|item|item.input_snapshot.as_ref().and_then(|value|value.get("signals")).is_some()));assert!(valid.iter().all(|item|!item.input_snapshot.as_ref().unwrap().to_string().contains("forward_return")));
    assert_eq!(connection.query_row("SELECT COUNT(*) FROM market_ai_signal_traces trace JOIN market_decisions decision ON decision.id=trace.decision_id WHERE decision.experiment_id=?1",[&result.experiment.id],|row|row.get::<_,usize>(0)).unwrap(),6);
    let diagnostics=market_repository::get_signal_diagnostics(&connection,&result.experiment.id).unwrap().unwrap();assert_eq!(diagnostics.signal_engine_version,"SIGNAL_ENGINE_V1");assert_eq!(diagnostics.signal_action_matrix.iter().map(|row|row.buy_count+row.sell_count+row.hold_count).sum::<usize>(),6);assert!(!diagnostics.reason_codes.is_empty());
    let frozen:String=connection.query_row("SELECT config_json FROM market_experiments WHERE id=?1",[&result.experiment.id],|row|row.get(0)).unwrap();assert!(frozen.contains("SIGNAL_ENGINE_V1"));assert!(frozen.contains("SIGNAL_CONFIG_V1"));let prompts=provider.prompts.lock().unwrap();assert!(prompts.iter().all(|prompt|prompt.system.contains("SIGNAL SCORES ARE PRE-COMPUTED")));drop(prompts);
    let invalid=MarketExperimentInput{agent_ids:vec!["cash".into(),market_ai::AI_AGENT_V2_ID.into(),market_ai::AI_AGENT_V3_ID.into()],..input};assert!(market_repository::validate_input(&connection,&invalid).unwrap_err().contains("no máximo um AI Agent"));let _=fs::remove_file(path);
}
