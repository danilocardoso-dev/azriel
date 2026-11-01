use serde::{Deserialize, Serialize};

/// Uma regra de lançamento: despesa ou recebimento, única, parcelada ou permanente.
/// Os meses não são gravados: cada ocorrência é calculada a partir da regra.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MoneyItem {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub amount_cents: i64,
    pub first_date: String,
    pub recurrence: String,
    pub installments: Option<i64>,
    pub project: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyItemInput {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub amount_cents: i64,
    pub first_date: String,
    pub recurrence: String,
    pub installments: Option<i64>,
    pub project: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MoneyEntry {
    pub item: MoneyItem,
    pub date: String,
    /// "2/3" nas parcelas, "permanente" nas recorrentes, vazio nas únicas.
    pub label: String,
    pub paid: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MoneyProjectSummary {
    pub name: String,
    pub month_cents: i64,
    pub received_cents: i64,
    /// Média mensal desde o primeiro recebimento do projeto até o mês consultado.
    pub average_cents: i64,
    pub total_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MoneyTotals {
    pub expenses_cents: i64,
    pub expenses_paid_cents: i64,
    pub income_cents: i64,
    pub income_received_cents: i64,
    pub balance_cents: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyMonth {
    pub month: String,
    pub expenses: Vec<MoneyEntry>,
    pub income: Vec<MoneyEntry>,
    pub totals: MoneyTotals,
    pub projects: Vec<MoneyProjectSummary>,
}
