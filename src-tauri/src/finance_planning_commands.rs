use crate::database::{
    finance_planning_models::*, finance_planning_repository as repository, DatabaseState,
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
macro_rules! call {
    ($state:expr,$body:expr) => {{
        let connection = lock(&$state)?;
        $body(&connection)
    }};
}

#[tauri::command]
pub fn list_finance_budgets(
    state: State<'_, DatabaseState>,
    year_month: String,
    view: String,
) -> Result<Vec<FinancialBudget>, String> {
    call!(state, |c| repository::list_budgets(c, &year_month, &view))
}
#[tauri::command]
pub fn save_finance_budget(
    state: State<'_, DatabaseState>,
    input: FinancialBudgetInput,
) -> Result<FinancialBudget, String> {
    call!(state, |c| repository::save_budget(c, &input))
}
#[tauri::command]
pub fn delete_finance_budget(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    call!(state, |c| repository::delete_budget(c, &id))
}
#[tauri::command]
pub fn copy_finance_budgets(
    state: State<'_, DatabaseState>,
    from_month: String,
    to_month: String,
) -> Result<BudgetCopyResult, String> {
    call!(state, |c| repository::copy_budgets(
        c,
        &from_month,
        &to_month
    ))
}
#[tauri::command]
pub fn list_finance_goals(
    state: State<'_, DatabaseState>,
    include_archived: bool,
) -> Result<Vec<FinancialGoal>, String> {
    call!(state, |c| repository::list_goals(c, include_archived))
}
#[tauri::command]
pub fn save_finance_goal(
    state: State<'_, DatabaseState>,
    input: FinancialGoalInput,
) -> Result<FinancialGoal, String> {
    call!(state, |c| repository::save_goal(c, &input))
}
#[tauri::command]
pub fn list_finance_goal_contributions(
    state: State<'_, DatabaseState>,
    goal_id: String,
) -> Result<Vec<GoalContribution>, String> {
    call!(state, |c| repository::list_contributions(c, &goal_id))
}
#[tauri::command]
pub fn save_finance_goal_contribution(
    state: State<'_, DatabaseState>,
    input: GoalContributionInput,
) -> Result<GoalContribution, String> {
    call!(state, |c| repository::save_contribution(c, &input))
}
#[tauri::command]
pub fn delete_finance_goal_contribution(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<(), String> {
    call!(state, |c| repository::delete_contribution(c, &id))
}
#[tauri::command]
pub fn get_finance_projection(
    state: State<'_, DatabaseState>,
    today: String,
    view: String,
    horizon_months: u32,
    base_cents: i64,
    scenario: ProjectionScenario,
) -> Result<FinancialProjection, String> {
    call!(state, |c| repository::projection(
        c,
        &today,
        &view,
        horizon_months,
        base_cents,
        &scenario
    ))
}
#[tauri::command]
pub fn get_finance_alerts(
    state: State<'_, DatabaseState>,
    today: String,
    year_month: String,
    view: String,
    upcoming_days: u32,
) -> Result<Vec<FinanceAlert>, String> {
    call!(state, |c| repository::alerts(
        c,
        &today,
        &year_month,
        &view,
        upcoming_days
    ))
}
