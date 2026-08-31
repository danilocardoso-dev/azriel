use crate::automation_policy::ActionSource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub confirmation_required: bool,
    pub revision: i64,
    pub steps: Vec<RoutineStep>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineInput {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub enabled: bool,
    pub confirmation_required: bool,
    pub steps: Vec<RoutineStepInput>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineStep {
    pub id: String,
    pub order: i64,
    pub action_id: String,
    pub target_type: String,
    pub target_id: String,
    pub delay_ms: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineStepInput {
    pub id: String,
    pub order: i64,
    pub action_id: String,
    pub target_type: String,
    pub target_id: String,
    #[serde(default)]
    pub delay_ms: i64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineHistory {
    pub id: i64,
    pub routine_id: Option<String>,
    pub routine_name: String,
    pub routine_revision: i64,
    pub source: String,
    pub status: String,
    pub confirmation_required: bool,
    pub confirmed: bool,
    pub total_steps: i64,
    pub completed_steps: i64,
    pub failed_step: Option<i64>,
    pub error: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRoutineRequest {
    pub routine_id: String,
    pub source: ActionSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineActionSummary {
    pub order: i64,
    pub action_id: String,
    pub action_name: String,
    pub target_name: String,
    pub delay_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineConfirmation {
    pub history_id: i64,
    pub routine_id: String,
    pub routine_name: String,
    pub revision: i64,
    pub actions: Vec<RoutineActionSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineExecutionResult {
    pub success: bool,
    pub status: String,
    pub routine_id: String,
    pub routine_name: String,
    pub history_id: i64,
    pub completed_steps: i64,
    pub failed_step: Option<i64>,
    pub error: Option<String>,
    pub confirmation: Option<RoutineConfirmation>,
}
