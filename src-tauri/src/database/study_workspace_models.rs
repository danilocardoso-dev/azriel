use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNotebook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub note_count: i64,
    pub last_note_updated_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNotebookInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNote {
    pub id: String,
    pub notebook_id: String,
    pub notebook_title: String,
    pub title: String,
    pub content: String,
    pub content_format: String,
    pub roadmap_id: Option<String>,
    pub roadmap_name: Option<String>,
    pub stage_id: Option<String>,
    pub stage_name: Option<String>,
    pub topic_id: Option<String>,
    pub topic_name: Option<String>,
    pub activity_id: Option<String>,
    pub activity_title: Option<String>,
    pub study_session_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNoteSummary {
    pub id: String,
    pub notebook_id: String,
    pub notebook_title: String,
    pub title: String,
    pub preview: String,
    pub roadmap_id: Option<String>,
    pub roadmap_name: Option<String>,
    pub activity_id: Option<String>,
    pub activity_title: Option<String>,
    pub study_session_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNoteInput {
    pub id: String,
    pub notebook_id: String,
    pub title: String,
    pub content: String,
    pub roadmap_id: Option<String>,
    pub stage_id: Option<String>,
    pub topic_id: Option<String>,
    pub activity_id: Option<String>,
    pub study_session_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNoteListInput {
    pub notebook_id: Option<String>,
    pub activity_id: Option<String>,
    pub study_session_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyNoteSearchInput {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}
