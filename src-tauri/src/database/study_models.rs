use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySession {
    pub id: String,
    pub roadmap_id: Option<String>,
    pub stage_id: Option<String>,
    pub topic_id: Option<String>,
    pub activity_id: Option<String>,
    pub roadmap_name: String,
    pub stage_name: String,
    pub topic_name: String,
    pub activity_title: String,
    pub planned_focus_minutes: i64,
    pub actual_focus_seconds: i64,
    pub current_focus_seconds: i64,
    pub linked_note_count: i64,
    pub observed_at: String,
    pub break_seconds: i64,
    pub status: String,
    pub started_at: String,
    pub running_since: Option<String>,
    pub paused_at: Option<String>,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartStudySessionInput {
    pub id: String,
    pub roadmap_id: Option<String>,
    pub activity_id: Option<String>,
    pub planned_focus_minutes: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySessionListInput {
    pub roadmap_id: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyRoadmapFocus {
    pub roadmap_id: Option<String>,
    pub roadmap_name: String,
    pub focus_seconds: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyTodaySummary {
    pub focus_seconds: i64,
    pub completed_sessions: i64,
    pub completed_activities: i64,
    pub most_studied_roadmap: Option<StudyRoadmapFocus>,
    pub active_session: Option<StudySession>,
    pub last_session: Option<StudySession>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySettings {
    pub focus_minutes: i64,
    pub short_break_minutes: i64,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudySettingsInput {
    pub focus_minutes: i64,
    pub short_break_minutes: i64,
}
