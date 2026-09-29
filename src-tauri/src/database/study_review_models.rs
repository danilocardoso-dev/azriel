use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyCard {
    pub id: String,
    pub front: String,
    pub back: String,
    pub status: String,
    pub roadmap_id: Option<String>,
    pub roadmap_name: Option<String>,
    pub stage_id: Option<String>,
    pub stage_name: Option<String>,
    pub topic_id: Option<String>,
    pub topic_name: Option<String>,
    pub activity_id: Option<String>,
    pub activity_title: Option<String>,
    pub notebook_id: Option<String>,
    pub notebook_title: Option<String>,
    pub note_id: Option<String>,
    pub note_title: Option<String>,
    pub due_at: String,
    pub last_reviewed_at: Option<String>,
    pub review_count: i64,
    pub correct_count: i64,
    pub incorrect_count: i64,
    pub current_interval_seconds: i64,
    pub scheduler_version: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyCardInput {
    pub id: String,
    pub front: String,
    pub back: String,
    pub roadmap_id: Option<String>,
    pub stage_id: Option<String>,
    pub topic_id: Option<String>,
    pub activity_id: Option<String>,
    pub notebook_id: Option<String>,
    pub note_id: Option<String>,
    #[serde(default)]
    pub source_material_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyCardListInput {
    pub query: Option<String>,
    pub status: Option<String>,
    pub filter: Option<String>,
    pub roadmap_id: Option<String>,
    pub activity_id: Option<String>,
    pub note_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewDashboardSummary {
    pub overdue: i64,
    pub due_today: i64,
    pub new_cards: i64,
    pub reviewed_today: i64,
    pub last_reviewed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewQueueItem {
    pub item_id: Option<String>,
    pub order: i64,
    pub category: String,
    pub card: StudyCard,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartStudyReviewSessionInput {
    pub id: String,
    pub mode: String,
    pub study_session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewSession {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: String,
    pub planned_cards: i64,
    pub reviewed_cards: i64,
    pub study_session_id: Option<String>,
    pub context_label: String,
    pub current_item: Option<StudyReviewQueueItem>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitStudyReviewResultInput {
    pub id: String,
    pub review_session_id: String,
    pub session_item_id: String,
    pub result: String,
    pub response_time_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewResult {
    pub session: StudyReviewSession,
    pub next_due_at: String,
    pub next_interval_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewSummary {
    pub session_id: String,
    pub status: String,
    pub reviewed_cards: i64,
    pub again: i64,
    pub hard: i64,
    pub good: i64,
    pub easy: i64,
    pub duration_seconds: i64,
    pub next_review_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyReviewSessionListInput {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}
