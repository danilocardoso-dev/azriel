use crate::database::{market_models::*, market_repository, DatabaseState};
use tauri::State;

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn import_market_dataset(state: State<'_, DatabaseState>, input: ImportMarketDatasetInput) -> Result<MarketDataset, String> {
    let mut connection = lock(&state)?;
    market_repository::import_dataset(&mut connection, &input)
}

#[tauri::command]
pub fn list_market_datasets(state: State<'_, DatabaseState>) -> Result<Vec<MarketDataset>, String> {
    let connection = lock(&state)?;
    market_repository::list_datasets(&connection)
}

#[tauri::command]
pub fn list_market_agents(state: State<'_, DatabaseState>) -> Result<Vec<MarketAgentDefinition>, String> {
    let connection = lock(&state)?;
    market_repository::list_agents(&connection)
}

#[tauri::command]
pub fn list_market_risk_profiles(state: State<'_, DatabaseState>) -> Result<Vec<MarketRiskProfile>, String> {
    let connection = lock(&state)?;
    market_repository::list_risk_profiles(&connection)
}

#[tauri::command]
pub fn run_market_experiment(state: State<'_, DatabaseState>, input: MarketExperimentInput) -> Result<MarketExperimentResult, String> {
    market_repository::set_kill_switch(false);
    let mut connection = lock(&state)?;
    market_repository::run_experiment(&mut connection, &input)
}

#[tauri::command]
pub fn list_market_experiments(state: State<'_, DatabaseState>) -> Result<Vec<MarketExperimentSummary>, String> {
    let connection = lock(&state)?;
    market_repository::list_experiments(&connection)
}

#[tauri::command]
pub fn get_market_experiment(state: State<'_, DatabaseState>, id: String) -> Result<MarketExperimentResult, String> {
    let connection = lock(&state)?;
    market_repository::get_experiment(&connection, &id)
}

#[tauri::command]
pub fn rerun_market_experiment(state: State<'_, DatabaseState>, id: String) -> Result<MarketExperimentResult, String> {
    market_repository::set_kill_switch(false);
    let mut connection = lock(&state)?;
    market_repository::rerun(&mut connection, &id)
}

#[tauri::command]
pub fn activate_market_kill_switch() -> bool {
    market_repository::set_kill_switch(true);
    true
}
