use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub knowledge_area_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskInput {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub project_id: Option<String>,
    pub knowledge_area_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
    pub status: String,
    pub project_id: Option<String>,
    pub knowledge_area_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteInput {
    pub id: String,
    pub title: Option<String>,
    pub content: String,
    pub status: String,
    pub project_id: Option<String>,
    pub knowledge_area_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCounters {
    pub pending: i64,
    pub today: i64,
    pub overdue: i64,
    pub priority: i64,
    pub notes: i64,
}
