use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationInput {
    pub id: String,
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredUrl {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredUrlInput {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistory {
    pub id: i64,
    pub action_id: String,
    pub source: String,
    pub target_type: Option<String>,
    pub target_id: Option<String>,
    pub target_name: Option<String>,
    pub permission: String,
    pub confirmation_required: bool,
    pub confirmed: bool,
    pub success: Option<bool>,
    pub error: Option<String>,
    pub created_at: String,
    pub completed_at: Option<String>,
}
