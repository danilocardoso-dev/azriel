use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeBaseline {
    pub knowledge_area_id: String,
    pub coverage: i64,
    pub depth: i64,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KnowledgeEvent {
    pub id: String,
    pub knowledge_node_id: String,
    pub source_type: String,
    pub source_id: Option<String>,
    pub event_type: String,
    pub coverage_delta: i64,
    pub depth_delta: i64,
    pub integration_delta: i64,
    pub description: String,
    pub created_at: String,
    pub activity_type: Option<String>,
    pub roadmap_id: Option<String>,
    pub topic_id: Option<String>,
    pub evidence_cycle: i64,
    pub formula_version: String,
    pub coverage_impact: f64,
    pub depth_impact: f64,
    pub integration_impact: f64,
    pub metadata_json: String,
    pub reversal_of_event_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapActivity {
    pub id: String,
    pub title: String,
    pub description: String,
    pub activity_type: String,
    pub status: String,
    pub completed_at: Option<String>,
    pub order: i64,
    #[serde(default)]
    pub primary_knowledge_node_id: Option<String>,
    #[serde(default)]
    pub secondary_knowledge_node_ids: Vec<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub research_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningMutation {
    pub created_events: Vec<KnowledgeEvent>,
    pub affected_knowledge_ids: Vec<String>,
    pub integration: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapSaveResult {
    pub roadmaps: Vec<StudyRoadmap>,
    pub learning: LearningMutation,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningEngineStatus {
    pub formula_version: String,
    pub integration_baseline: f64,
    pub current_integration: f64,
    pub event_count: i64,
    pub last_recalculated_at: Option<String>,
    pub status: String,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapTopic {
    pub id: String,
    pub name: String,
    pub description: String,
    pub knowledge_node_id: Option<String>,
    pub state: String,
    pub order: i64,
    #[serde(default)]
    pub activities: Vec<RoadmapActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapStage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub order: i64,
    #[serde(default)]
    pub topics: Vec<RoadmapTopic>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyRoadmap {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub completed_activities: i64,
    pub total_activities: i64,
    pub progress: i64,
    pub stages: Vec<RoadmapStage>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyRoadmapInput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    #[serde(default)]
    pub stages: Vec<RoadmapStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchItem {
    pub id: String,
    pub title: String,
    pub domain: String,
    pub objective: String,
    pub description: String,
    pub kind: String,
    pub status: String,
    pub impact: String,
    pub knowledge_node_id: Option<String>,
    pub roadmap_id: Option<String>,
    pub roadmap_topic_id: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StarkSummary {
    pub roadmap_count: i64,
    pub active_roadmap_count: i64,
    pub research_count: i64,
    pub active_research_count: i64,
    pub baseline_count: i64,
    pub event_count: i64,
}
