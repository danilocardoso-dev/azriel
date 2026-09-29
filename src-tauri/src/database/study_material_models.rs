use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyMaterialRelation {
    pub id: String,
    pub relation_type: String,
    pub relation_id: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyMaterialRelationInput {
    pub relation_type: String,
    pub relation_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialTextContent {
    pub material_id: String,
    pub status: String,
    pub text: Option<String>,
    pub pages: Vec<String>,
    pub extracted_at: Option<String>,
    pub extractor_version: Option<String>,
    pub char_count: i64,
    pub is_stale: bool,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyMaterial {
    pub id: String,
    pub title: String,
    pub material_type: String,
    pub storage_kind: String,
    pub external_url: Option<String>,
    pub original_filename: Option<String>,
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub checksum_sha256: Option<String>,
    pub source_author: Option<String>,
    pub source_title: Option<String>,
    pub source_year: Option<i64>,
    pub description: Option<String>,
    pub page_count: Option<i64>,
    pub status: String,
    pub removed: bool,
    pub managed_file_available: bool,
    pub relations: Vec<StudyMaterialRelation>,
    pub text_content: MaterialTextContent,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyMaterialSummary {
    pub id: String,
    pub title: String,
    pub material_type: String,
    pub storage_kind: String,
    pub original_filename: Option<String>,
    pub source_author: Option<String>,
    pub status: String,
    pub extraction_status: String,
    pub extraction_stale: bool,
    pub relation_labels: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStudyMaterialInput {
    pub id: String,
    pub title: String,
    pub source_path: String,
    pub storage_kind: String,
    pub source_author: Option<String>,
    pub source_title: Option<String>,
    pub source_year: Option<i64>,
    pub description: Option<String>,
    #[serde(default)]
    pub relations: Vec<StudyMaterialRelationInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddLinkStudyMaterialInput {
    pub id: String,
    pub title: String,
    pub external_url: String,
    pub description: Option<String>,
    pub source_author: Option<String>,
    pub source_title: Option<String>,
    pub source_year: Option<i64>,
    #[serde(default)]
    pub relations: Vec<StudyMaterialRelationInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStudyMaterialInput {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub source_author: Option<String>,
    pub source_title: Option<String>,
    pub source_year: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyMaterialListInput {
    pub query: Option<String>,
    pub material_type: Option<String>,
    pub status: Option<String>,
    pub relation_type: Option<String>,
    pub relation_id: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportStudyMaterialResult {
    pub material: StudyMaterial,
    pub duplicate: bool,
}

#[derive(Debug, Clone)]
pub struct MaterialOpenTarget {
    pub storage_kind: String,
    pub local_path: Option<String>,
    pub external_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MaterialPreview {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

fn default_limit() -> i64 {
    50
}
