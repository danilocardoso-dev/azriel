use crate::{database::{self, market_models::*, market_repository, market_validation, DatabaseState}, ollama};
use tauri::State;

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn import_market_dataset(
    state: State<'_, DatabaseState>,
    input: ImportMarketDatasetInput,
) -> Result<MarketDataset, String> {
    let mut connection = lock(&state)?;
    market_repository::import_dataset(&mut connection, &input)
}

#[tauri::command]
pub fn list_market_datasets(state: State<'_, DatabaseState>) -> Result<Vec<MarketDataset>, String> {
    let connection = lock(&state)?;
    market_repository::list_datasets(&connection)
}

#[tauri::command]
pub fn list_market_agents(
    state: State<'_, DatabaseState>,
) -> Result<Vec<MarketAgentDefinition>, String> {
    let connection = lock(&state)?;
    market_repository::list_agents(&connection)
}

#[tauri::command]
pub fn list_market_risk_profiles(
    state: State<'_, DatabaseState>,
) -> Result<Vec<MarketRiskProfile>, String> {
    let connection = lock(&state)?;
    market_repository::list_risk_profiles(&connection)
}

fn worker_connection(path:&std::path::Path)->Result<rusqlite::Connection,String>{let mut connection=rusqlite::Connection::open(path).map_err(|error|error.to_string())?;database::initialize(&mut connection)?;Ok(connection)}

#[tauri::command]
pub async fn run_market_experiment(
    state: State<'_, DatabaseState>,
    input: MarketExperimentInput,
) -> Result<MarketExperimentResult, String> {
    market_repository::set_kill_switch(false);
    let path=state.path.clone();
    tauri::async_runtime::spawn_blocking(move||{let mut connection=worker_connection(&path)?;market_repository::run_experiment(&mut connection,&input)}).await.map_err(|error|error.to_string())?
}

#[tauri::command]
pub fn list_market_experiments(
    state: State<'_, DatabaseState>,
) -> Result<Vec<MarketExperimentSummary>, String> {
    let connection = lock(&state)?;
    market_repository::list_experiments(&connection)
}

#[tauri::command]
pub fn get_market_experiment(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<MarketExperimentResult, String> {
    let connection = lock(&state)?;
    market_repository::get_experiment(&connection, &id)
}

#[tauri::command]
pub async fn rerun_market_experiment(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<MarketExperimentResult, String> {
    market_repository::set_kill_switch(false);
    let path=state.path.clone();
    tauri::async_runtime::spawn_blocking(move||{let mut connection=worker_connection(&path)?;market_repository::rerun(&mut connection,&id)}).await.map_err(|error|error.to_string())?
}

#[tauri::command]
pub fn activate_market_kill_switch() -> bool {
    market_repository::set_kill_switch(true);
    true
}

#[tauri::command]
pub async fn run_market_validation(
    state: State<'_, DatabaseState>,
    input: MarketValidationInput,
) -> Result<MarketValidationResult, String> {
    market_repository::set_kill_switch(false);
    let path=state.path.clone();
    tauri::async_runtime::spawn_blocking(move||{let mut connection=worker_connection(&path)?;market_validation::run(&mut connection,&input)}).await.map_err(|error|error.to_string())?
}

fn config_probe_input(agent_id:String)->MarketExperimentInput{MarketExperimentInput{name:"probe".into(),dataset_id:String::new(),risk_profile_id:String::new(),agent_ids:vec![agent_id],initial_capital:1.0,random_seed:42,fee_pct:0.0,slippage_pct:0.0}}

#[tauri::command]
pub async fn get_market_ai_status(state:State<'_,DatabaseState>,agent_id:Option<String>)->Result<MarketAiStatus,String>{let requested=agent_id.unwrap_or_else(||crate::database::market_ai::AI_AGENT_V3_ID.into());let(config,endpoint)={let connection=lock(&state)?;market_repository::resolve_ai_config(&connection,&config_probe_input(requested))?};let health=ollama::status(&endpoint,(config.timeout_ms/1000).clamp(5,8)).await?;let model_available=health.available&&health.models.iter().any(|model|model==&config.model);let error=if health.available&&!model_available{Some(format!("O modelo '{}' não está instalado no Ollama",config.model))}else{health.error};Ok(MarketAiStatus{agent_id:config.agent_id,configured:true,available:model_available,provider:config.provider,model:config.model,prompt_version:config.prompt_version,decision_interval:config.decision_interval,timeout_ms:config.timeout_ms,max_retries:config.max_retries,error})}

#[tauri::command]
pub fn update_market_ai_config(state:State<'_,DatabaseState>,input:UpdateMarketAiConfigInput)->Result<MarketAiStatus,String>{let connection=lock(&state)?;market_repository::update_ai_config(&connection,&input)?;let(config,_)=market_repository::resolve_ai_config(&connection,&config_probe_input(input.agent_id))?;Ok(MarketAiStatus{agent_id:config.agent_id,configured:true,available:false,provider:config.provider,model:config.model,prompt_version:config.prompt_version,decision_interval:config.decision_interval,timeout_ms:config.timeout_ms,max_retries:config.max_retries,error:None})}

#[tauri::command]
pub fn list_market_ai_runtime(state:State<'_,DatabaseState>,experiment_id:String)->Result<Vec<MarketAiRuntimeMetric>,String>{let connection=lock(&state)?;market_repository::list_ai_runtime(&connection,&experiment_id)}

#[tauri::command]
pub fn list_market_validation_ai_runtime(state:State<'_,DatabaseState>,validation_id:String)->Result<Vec<MarketAiRuntimeMetric>,String>{let connection=lock(&state)?;market_repository::list_validation_ai_runtime(&connection,&validation_id)}

#[tauri::command]
pub fn list_market_ai_decisions(state:State<'_,DatabaseState>,experiment_id:String)->Result<Vec<MarketAiDecisionLog>,String>{let connection=lock(&state)?;market_repository::list_ai_decisions(&connection,&experiment_id)}

#[tauri::command]
pub fn list_market_ai_experiment_comparisons(state:State<'_,DatabaseState>)->Result<Vec<MarketAiExperimentComparison>,String>{let connection=lock(&state)?;market_repository::list_ai_experiment_comparisons(&connection)}

#[tauri::command]
pub fn get_market_signal_diagnostics(state:State<'_,DatabaseState>,experiment_id:String)->Result<Option<MarketSignalDiagnostics>,String>{let connection=lock(&state)?;market_repository::get_signal_diagnostics(&connection,&experiment_id)}

#[tauri::command]
pub fn list_market_validations(
    state: State<'_, DatabaseState>,
) -> Result<Vec<MarketValidationSummary>, String> {
    let connection = lock(&state)?;
    market_validation::list(&connection)
}

#[tauri::command]
pub fn get_market_validation(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<MarketValidationResult, String> {
    let connection = lock(&state)?;
    market_validation::get(&connection, &id)
}
