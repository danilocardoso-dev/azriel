use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringCalibration {
    pub pinch_start_threshold: f64,
    pub pinch_release_threshold: f64,
    pub smoothing_alpha: f64,
    pub rotation_sensitivity: f64,
    pub min_scale: f64,
    pub max_scale: f64,
    pub comfortable_hand_distance: f64,
    pub calibrated: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringCalibrationInput {
    pub pinch_start_threshold: f64,
    pub pinch_release_threshold: f64,
    pub smoothing_alpha: f64,
    pub rotation_sensitivity: f64,
    pub min_scale: f64,
    pub max_scale: f64,
    pub comfortable_hand_distance: f64,
    pub calibrated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringModelComponentInput {
    pub component_identity: String,
    pub original_name: String,
    pub structural_path: String,
    pub component_type: String,
    pub selectable: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterEngineeringModelInput {
    pub model_identity: String,
    pub file_name: String,
    pub format: String,
    pub byte_size: i64,
    pub components: Vec<EngineeringModelComponentInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSemantic {
    pub model_identity: String,
    pub component_identity: String,
    pub original_name: String,
    pub structural_path: String,
    pub component_type: String,
    pub semantic_label: String,
    pub subsystem_id: Option<String>,
    pub role: String,
    pub description: String,
    pub notes: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSemanticInput {
    pub model_identity: String,
    pub component_identity: String,
    pub semantic_label: String,
    pub subsystem_id: Option<String>,
    pub role: String,
    pub description: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringSubsystem {
    pub id: String,
    pub model_identity: String,
    pub name: String,
    pub description: String,
    pub parent_subsystem_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineeringSubsystemInput {
    pub id: String,
    pub model_identity: String,
    pub name: String,
    pub description: String,
    pub parent_subsystem_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationship {
    pub id: String,
    pub model_identity: String,
    pub source_component_identity: String,
    pub target_component_identity: String,
    pub relationship_type: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRelationshipInput {
    pub id: String,
    pub model_identity: String,
    pub source_component_identity: String,
    pub target_component_identity: String,
    pub relationship_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssemblyIntelligenceSnapshot {
    pub semantics: Vec<ComponentSemantic>,
    pub subsystems: Vec<EngineeringSubsystem>,
    pub relationships: Vec<ComponentRelationship>,
}
