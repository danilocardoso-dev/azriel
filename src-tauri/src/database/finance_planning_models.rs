use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialBudget {
    pub id: String,
    pub year_month: String,
    pub scope: String,
    pub category_id: String,
    pub category_name: String,
    pub category_enabled: bool,
    pub limit_cents: i64,
    pub realized_cents: i64,
    pub committed_cents: i64,
    pub available_cents: i64,
    pub usage_basis_points: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialBudgetInput {
    pub id: String,
    pub year_month: String,
    pub scope: String,
    pub category_id: String,
    pub limit_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BudgetCopyResult {
    pub created: i64,
    pub skipped: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialGoal {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub target_cents: i64,
    pub target_date: Option<String>,
    pub status: String,
    pub contributed_cents: i64,
    pub remaining_cents: i64,
    pub progress_basis_points: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialGoalInput {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub target_cents: i64,
    pub target_date: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalContribution {
    pub id: String,
    pub goal_id: String,
    pub amount_cents: i64,
    pub contribution_date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalContributionInput {
    pub id: String,
    pub goal_id: String,
    pub amount_cents: i64,
    pub contribution_date: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionScenario {
    pub professional_income_reduction_percent: i64,
    pub operating_cost_increase_percent: i64,
    pub new_monthly_commitment_cents: i64,
    pub closed_contract_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionItem {
    pub origin_key: String,
    pub source_type: String,
    pub source_name: String,
    pub direction: String,
    pub scope: String,
    pub category_id: String,
    pub amount_cents: i64,
    pub expected_date: String,
    pub estimated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectionMonth {
    pub month: String,
    pub income_cents: i64,
    pub expense_cents: i64,
    pub result_cents: i64,
    pub accumulated_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialProjection {
    pub view: String,
    pub horizon_months: u32,
    pub base_cents: i64,
    pub items: Vec<ProjectionItem>,
    pub months: Vec<ProjectionMonth>,
    pub base_result_cents: i64,
    pub scenario_result_cents: i64,
    pub scenario_difference_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceAlert {
    pub id: String,
    pub level: String,
    pub alert_type: String,
    pub title: String,
    pub description: String,
    pub reference_id: String,
}
