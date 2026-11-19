//! DeepSeek pela API oficial, com a chave do Senhor. Mesma API compatível com OpenAI do Gemini.

use crate::database::{deepseek_repository::{self, DeepSeekSettings, DeepSeekSettingsInput}, DatabaseState};
use crate::gemini::{self, Compat};
use crate::ollama::{OllamaAgentMessage, OllamaAgentResult};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tauri::State;

const BASE_URL: &str = "https://api.deepseek.com";
pub const DEEPSEEK: Compat = Compat { name: "DeepSeek", base_url: BASE_URL, strip_empty_parameters: false };

/// Ids de modelos de conversa devolvidos por GET /models, em ordem.
pub fn parse_models(payload: &Value) -> Vec<String> {
    let mut models: Vec<String> = payload["data"]
        .as_array()
        .map(|list| list.as_slice())
        .unwrap_or_default()
        .iter()
        .filter_map(|model| model["id"].as_str())
        .filter(|id| deepseek_repository::valid_model(id))
        .map(String::from)
        .collect();
    models.sort();
    models.dedup();
    models
}

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn get_deepseek_settings(state: State<'_, DatabaseState>) -> Result<DeepSeekSettings, String> {
    deepseek_repository::get(&*lock(&state)?)
}

#[tauri::command]
pub fn update_deepseek_settings(state: State<'_, DatabaseState>, input: DeepSeekSettingsInput) -> Result<DeepSeekSettings, String> {
    deepseek_repository::update(&*lock(&state)?, &input)
}

#[tauri::command]
pub async fn list_deepseek_models(state: State<'_, DatabaseState>) -> Result<Vec<String>, String> {
    let (key, _) = deepseek_repository::credentials(&*lock(&state)?)?;
    let client = Client::builder().timeout(Duration::from_secs(20)).build().map_err(|_| "Não foi possível preparar o cliente do DeepSeek".to_string())?;
    let response = client.get(format!("{BASE_URL}/models")).bearer_auth(key).send().await.map_err(|_| "Não foi possível conectar ao DeepSeek. Verifique a internet.".to_string())?;
    match response.status().as_u16() {
        200..=299 => {}
        401 | 403 => return Err("Chave do DeepSeek inválida ou sem permissão".into()),
        status => return Err(format!("O DeepSeek respondeu com status {status} ao listar os modelos")),
    }
    let payload: Value = response.json().await.map_err(|_| "Lista de modelos inválida recebida do DeepSeek".to_string())?;
    Ok(parse_models(&payload))
}

#[tauri::command]
pub async fn deepseek_agent_chat(state: State<'_, DatabaseState>, messages: Vec<OllamaAgentMessage>, tools: Vec<Value>, timeout_seconds: u64) -> Result<OllamaAgentResult, String> {
    let (api_key, model) = deepseek_repository::credentials(&*lock(&state)?)?;
    gemini::chat_compatible(&DEEPSEEK, &api_key, &model, messages, tools, timeout_seconds).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn model_list_is_sorted_and_safe() {
        let payload = json!({"object": "list", "data": [{"id": "deepseek-reasoner", "object": "model"}, {"id": "deepseek-chat", "object": "model"}, {"id": "bad id; x"}]});
        assert_eq!(parse_models(&payload), vec!["deepseek-chat", "deepseek-reasoner"]);
        assert!(parse_models(&json!({})).is_empty());
    }
}
