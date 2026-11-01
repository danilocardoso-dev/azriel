use crate::database::{money_models::*, money_repository, DatabaseState};
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
pub fn money_month(state: State<'_, DatabaseState>, month: String) -> Result<MoneyMonth, String> {
    let connection = lock(&state)?;
    money_repository::month_view(&connection, &month)
}
#[tauri::command]
pub fn money_save_item(state: State<'_, DatabaseState>, input: MoneyItemInput) -> Result<MoneyItem, String> {
    let connection = lock(&state)?;
    money_repository::save_item(&connection, &input)
}
#[tauri::command]
pub fn money_delete_item(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    money_repository::delete_item(&connection, &id)
}
#[tauri::command]
pub fn money_set_paid(state: State<'_, DatabaseState>, id: String, month: String, paid: bool) -> Result<(), String> {
    let connection = lock(&state)?;
    money_repository::set_paid(&connection, &id, &month, paid)
}
#[tauri::command]
pub fn money_projects(state: State<'_, DatabaseState>) -> Result<Vec<String>, String> {
    let connection = lock(&state)?;
    money_repository::project_names(&connection)
}
