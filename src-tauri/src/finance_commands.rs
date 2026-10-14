use crate::database::{finance_models::*, finance_repository, DatabaseState};
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
pub fn list_financial_categories(
    state: State<'_, DatabaseState>,
    include_disabled: bool,
) -> Result<Vec<FinancialCategory>, String> {
    let connection = lock(&state)?;
    finance_repository::list_categories(&connection, include_disabled)
}

#[tauri::command]
pub fn save_financial_category(
    state: State<'_, DatabaseState>,
    input: FinancialCategoryInput,
) -> Result<FinancialCategory, String> {
    let connection = lock(&state)?;
    finance_repository::save_category(&connection, &input)
}

#[tauri::command]
pub fn set_financial_category_enabled(
    state: State<'_, DatabaseState>,
    id: String,
    enabled: bool,
) -> Result<FinancialCategory, String> {
    let connection = lock(&state)?;
    finance_repository::set_category_enabled(&connection, &id, enabled)
}

#[tauri::command]
pub fn list_financial_transactions(
    state: State<'_, DatabaseState>,
    filter: FinancialTransactionFilter,
) -> Result<Vec<FinancialTransaction>, String> {
    let connection = lock(&state)?;
    finance_repository::list_transactions(&connection, &filter)
}

#[tauri::command]
pub fn save_financial_transaction(
    state: State<'_, DatabaseState>,
    input: FinancialTransactionInput,
) -> Result<FinancialTransaction, String> {
    let connection = lock(&state)?;
    finance_repository::save_transaction(&connection, &input)
}

#[tauri::command]
pub fn mark_financial_transaction_paid(
    state: State<'_, DatabaseState>,
    id: String,
    paid_at: String,
) -> Result<FinancialTransaction, String> {
    let connection = lock(&state)?;
    finance_repository::mark_transaction_paid(&connection, &id, &paid_at)
}

#[tauri::command]
pub fn cancel_financial_transaction(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<FinancialTransaction, String> {
    let connection = lock(&state)?;
    finance_repository::cancel_transaction(&connection, &id)
}

#[tauri::command]
pub fn get_financial_dashboard(
    state: State<'_, DatabaseState>,
    month: String,
    view: String,
    today: String,
) -> Result<FinancialDashboard, String> {
    let connection = lock(&state)?;
    finance_repository::dashboard(&connection, &month, &view, &today)
}

#[tauri::command]
pub fn get_financial_monthly_report(
    state: State<'_, DatabaseState>,
    month: String,
    view: String,
) -> Result<FinancialMonthlyReport, String> {
    let connection = lock(&state)?;
    finance_repository::monthly_report(&connection, &month, &view)
}
