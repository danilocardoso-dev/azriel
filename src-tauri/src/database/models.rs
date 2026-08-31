use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeArea {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub coverage: i64,
    pub depth: i64,
    pub priority: String,
    pub project_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeHistory {
    pub id: i64,
    pub knowledge_id: String,
    pub coverage: i64,
    pub depth: i64,
    pub recorded_at: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeInput {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub coverage: i64,
    pub depth: i64,
    pub priority: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsInput {
    pub knowledge_id: String,
    pub coverage: i64,
    pub depth: i64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objective: String,
    pub category: String,
    pub status: String,
    pub progress: i64,
    pub next_step: String,
    pub knowledge_area_ids: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub objective: String,
    pub category: String,
    pub status: String,
    pub progress: i64,
    pub next_step: String,
    #[serde(default)]
    pub knowledge_area_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationItem {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub institution: String,
    pub status: String,
    pub start_date: Option<String>,
    pub expected_end_date: Option<String>,
    pub completed_at: Option<String>,
    pub description: String,
    pub period: String,
    pub domains: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationInput {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub institution: String,
    pub status: String,
    pub start_date: Option<String>,
    pub expected_end_date: Option<String>,
    pub completed_at: Option<String>,
    pub description: String,
    pub period: String,
    #[serde(default)]
    pub domains: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub schema_version: i64,
    pub integration_value: f64,
}
