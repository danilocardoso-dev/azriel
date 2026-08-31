use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_id: Option<String>,
    pub application_id: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceInput {
    pub id: String,
    pub name: String,
    pub path: String,
    pub project_id: Option<String>,
    pub application_id: Option<String>,
    pub enabled: bool,
}
