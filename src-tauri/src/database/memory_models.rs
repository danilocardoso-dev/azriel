use serde::{Deserialize, Serialize};

/// Núcleo de identidade: o texto que diz ao Azriel quem ele é. Editado só pelo operador.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub content: String,
    pub is_default: bool,
    pub updated_at: String,
}

/// Uma lembrança de longo prazo do Azriel.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub category: String,
    pub source: String,
    pub conversation_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInput {
    pub id: String,
    pub content: String,
    pub category: String,
    pub source: String,
    pub conversation_id: Option<String>,
}
