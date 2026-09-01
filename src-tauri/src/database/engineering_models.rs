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
