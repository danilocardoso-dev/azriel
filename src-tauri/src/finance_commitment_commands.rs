use crate::database::{
    finance_commitment_models::*, finance_commitment_repository as repository, DatabaseState,
};
use tauri::State;

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}
macro_rules! with_connection {
    ($state:expr,$call:expr) => {{
        let connection = lock(&$state)?;
        $call(&connection)
    }};
}

#[tauri::command]
pub fn list_finance_contracts(
    state: State<'_, DatabaseState>,
) -> Result<Vec<FinanceContract>, String> {
    with_connection!(state, repository::list_contracts)
}
#[tauri::command]
pub fn save_finance_contract(
    state: State<'_, DatabaseState>,
    input: FinanceContractInput,
) -> Result<FinanceContract, String> {
    with_connection!(state, |c| repository::save_contract(c, &input))
}
#[tauri::command]
pub fn list_finance_recurrences(
    state: State<'_, DatabaseState>,
) -> Result<Vec<FinanceRecurrence>, String> {
    with_connection!(state, repository::list_recurrences)
}
#[tauri::command]
pub fn save_finance_recurrence(
    state: State<'_, DatabaseState>,
    input: FinanceRecurrenceInput,
) -> Result<FinanceRecurrence, String> {
    with_connection!(state, |c| repository::save_recurrence(c, &input))
}
#[tauri::command]
pub fn list_finance_loans(state: State<'_, DatabaseState>) -> Result<Vec<FinanceLoan>, String> {
    with_connection!(state, repository::list_loans)
}
#[tauri::command]
pub fn save_finance_loan(
    state: State<'_, DatabaseState>,
    input: FinanceLoanInput,
) -> Result<FinanceLoan, String> {
    with_connection!(state, |c| repository::save_loan(c, &input))
}
#[tauri::command]
pub fn list_finance_loan_installments(
    state: State<'_, DatabaseState>,
    loan_id: String,
) -> Result<Vec<FinanceLoanInstallment>, String> {
    with_connection!(state, |c| repository::list_installments(c, &loan_id))
}
#[tauri::command]
pub fn save_finance_loan_installment(
    state: State<'_, DatabaseState>,
    input: FinanceLoanInstallmentInput,
) -> Result<FinanceLoanInstallment, String> {
    with_connection!(state, |c| repository::save_installment(c, &input))
}
#[tauri::command]
pub fn set_finance_origin_status(
    state: State<'_, DatabaseState>,
    source_type: String,
    id: String,
    status: String,
    effective_date: String,
) -> Result<(), String> {
    with_connection!(state, |c| repository::set_origin_status(
        c,
        &source_type,
        &id,
        &status,
        &effective_date
    ))
}
#[tauri::command]
pub fn generate_finance_commitments(
    state: State<'_, DatabaseState>,
    today: String,
    horizon_months: u32,
) -> Result<FinanceCommitmentGenerationResult, String> {
    with_connection!(state, |c| repository::generate_commitments(
        c,
        &today,
        horizon_months
    ))
}
#[tauri::command]
pub fn list_finance_commitments(
    state: State<'_, DatabaseState>,
    filter: FinanceCommitmentFilter,
) -> Result<Vec<FinanceCommitment>, String> {
    with_connection!(state, |c| repository::list_commitments(c, &filter))
}
#[tauri::command]
pub fn adjust_finance_commitment(
    state: State<'_, DatabaseState>,
    input: FinanceCommitmentAdjustmentInput,
    today: String,
) -> Result<FinanceCommitment, String> {
    with_connection!(state, |c| repository::adjust_commitment(c, &input, &today))
}
#[tauri::command]
pub fn cancel_finance_commitment(
    state: State<'_, DatabaseState>,
    id: String,
    today: String,
) -> Result<FinanceCommitment, String> {
    with_connection!(state, |c| repository::cancel_commitment(c, &id, &today))
}
#[tauri::command]
pub fn settle_finance_commitment(
    state: State<'_, DatabaseState>,
    input: FinanceCommitmentSettlementInput,
    today: String,
) -> Result<FinanceCommitmentSettlementResult, String> {
    with_connection!(state, |c| repository::settle_commitment(c, &input, &today))
}
#[tauri::command]
pub fn link_finance_commitment_transaction(
    state: State<'_, DatabaseState>,
    input: FinanceCommitmentLinkInput,
    today: String,
) -> Result<FinanceCommitmentSettlementResult, String> {
    with_connection!(state, |c| repository::link_transaction(c, &input, &today))
}
#[tauri::command]
pub fn get_finance_commitment_summary(
    state: State<'_, DatabaseState>,
    today: String,
) -> Result<FinanceCommitmentSummary, String> {
    with_connection!(state, |c| repository::commitment_summary(c, &today))
}
