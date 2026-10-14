use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceContract {
    pub id: String,
    pub name: String,
    pub counterparty: String,
    pub description: Option<String>,
    pub scope: String,
    pub amount_cents: i64,
    pub currency: String,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub first_due_date: String,
    pub status: String,
    pub category_id: String,
    pub category_name: String,
    pub notes: Option<String>,
    pub resume_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceContractInput {
    pub id: String,
    pub name: String,
    pub counterparty: String,
    pub description: Option<String>,
    pub scope: String,
    pub amount_cents: i64,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub first_due_date: String,
    pub category_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceRecurrence {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub amount_cents: i64,
    pub estimated: bool,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub first_due_date: String,
    pub status: String,
    pub category_id: String,
    pub category_name: String,
    pub notes: Option<String>,
    pub resume_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceRecurrenceInput {
    pub id: String,
    pub title: String,
    pub scope: String,
    pub amount_cents: i64,
    pub estimated: bool,
    pub frequency: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub first_due_date: String,
    pub category_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceLoan {
    pub id: String,
    pub name: String,
    pub creditor: String,
    pub purpose: Option<String>,
    pub scope: String,
    pub original_principal_cents: i64,
    pub informed_balance_cents: i64,
    pub balance_as_of: String,
    pub installment_count: i64,
    pub interest_rate_basis_points: Option<i64>,
    pub interest_rate_period: Option<String>,
    pub status: String,
    pub category_id: String,
    pub category_name: String,
    pub notes: Option<String>,
    pub resume_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceLoanInput {
    pub id: String,
    pub name: String,
    pub creditor: String,
    pub purpose: Option<String>,
    pub scope: String,
    pub original_principal_cents: i64,
    pub informed_balance_cents: i64,
    pub balance_as_of: String,
    pub installment_count: i64,
    pub interest_rate_basis_points: Option<i64>,
    pub interest_rate_period: Option<String>,
    pub category_id: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceLoanInstallment {
    pub id: String,
    pub loan_id: String,
    pub loan_name: String,
    pub installment_number: i64,
    pub due_date: String,
    pub amount_cents: i64,
    pub principal_cents: Option<i64>,
    pub interest_cents: Option<i64>,
    pub fees_cents: Option<i64>,
    pub notes: Option<String>,
    pub commitment_id: Option<String>,
    pub commitment_status: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceLoanInstallmentInput {
    pub id: String,
    pub loan_id: String,
    pub installment_number: i64,
    pub due_date: String,
    pub amount_cents: i64,
    pub principal_cents: Option<i64>,
    pub interest_cents: Option<i64>,
    pub fees_cents: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitment {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub source_name: String,
    pub occurrence_key: String,
    pub direction: String,
    pub scope: String,
    pub category_id: String,
    pub category_name: String,
    pub expected_amount_cents: i64,
    pub estimated: bool,
    pub competence_date: String,
    pub due_date: String,
    pub status: String,
    pub effective_status: String,
    pub transaction_id: Option<String>,
    pub settled_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub source_type: Option<String>,
    pub scope: Option<String>,
    pub status: Option<String>,
    pub direction: Option<String>,
    pub today: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentAdjustmentInput {
    pub id: String,
    pub expected_amount_cents: i64,
    pub competence_date: String,
    pub due_date: String,
    pub estimated: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentSettlementInput {
    pub commitment_id: String,
    pub transaction_id: String,
    pub actual_amount_cents: i64,
    pub paid_at: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentLinkInput {
    pub commitment_id: String,
    pub transaction_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentSettlementResult {
    pub commitment: FinanceCommitment,
    pub transaction_id: String,
    pub reused: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentGenerationResult {
    pub created: i64,
    pub existing: i64,
    pub window_start: String,
    pub window_end: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FinanceCommitmentSummary {
    pub receivable_next_30_cents: i64,
    pub payable_next_30_cents: i64,
    pub overdue_receivable_cents: i64,
    pub overdue_payable_cents: i64,
    pub overdue_count: i64,
    pub active_contracts: i64,
    pub contracted_monthly_equivalent_cents: i64,
    pub professional_recurring_monthly_equivalent_cents: i64,
    pub future_installments_cents: i64,
    pub informed_debt_balance_cents: i64,
}
