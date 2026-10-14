use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialCategory {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialCategoryInput {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialTransaction {
    pub id: String,
    pub transaction_type: String,
    pub scope: String,
    pub description: String,
    pub amount_cents: i64,
    pub category_id: String,
    pub category_name: String,
    pub competence_date: String,
    pub due_date: Option<String>,
    pub paid_at: Option<String>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialTransactionInput {
    pub id: String,
    pub transaction_type: String,
    pub scope: String,
    pub description: String,
    pub amount_cents: i64,
    pub category_id: String,
    pub competence_date: String,
    pub due_date: Option<String>,
    pub paid_at: Option<String>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinancialTransactionFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub category_id: Option<String>,
    pub transaction_type: Option<String>,
    pub scope: Option<String>,
    pub status: Option<String>,
    pub view: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FinancialTotals {
    pub income_cents: i64,
    pub expense_cents: i64,
    pub result_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialDashboard {
    pub month: String,
    pub view: String,
    pub realized: FinancialTotals,
    pub planned: FinancialTotals,
    pub personal_realized: FinancialTotals,
    pub personal_planned: FinancialTotals,
    pub professional_realized: FinancialTotals,
    pub professional_planned: FinancialTotals,
    pub upcoming_due: Vec<FinancialTransaction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialCategoryTotal {
    pub category_id: String,
    pub category_name: String,
    pub scope: String,
    pub amount_cents: i64,
    pub share_basis_points: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialMonthlyReport {
    pub month: String,
    pub view: String,
    pub realized: FinancialTotals,
    pub planned: FinancialTotals,
    pub personal: FinancialTotals,
    pub professional: FinancialTotals,
    pub category_expenses: Vec<FinancialCategoryTotal>,
    pub previous_month: String,
    pub previous_realized: FinancialTotals,
    pub previous_has_data: bool,
    pub result_change_cents: i64,
    pub result_change_basis_points: Option<i64>,
}
