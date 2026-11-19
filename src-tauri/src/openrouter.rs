//! OpenRouter: um provedor com vários modelos (DeepSeek, Kimi, Claude, GPT...) pela mesma API
//! compatível com OpenAI que o Gemini usa. Trocar de modelo não exige mudar código.

use crate::database::{openrouter_repository::{self, OpenRouterSettings, OpenRouterSettingsInput}, DatabaseState};
use crate::gemini::{self, Compat};
use crate::ollama::{OllamaAgentMessage, OllamaAgentResult};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use tauri::State;

const BASE_URL: &str = "https://openrouter.ai/api/v1";
pub const OPENROUTER: Compat = Compat { name: "OpenRouter", base_url: BASE_URL, strip_empty_parameters: false };

/// Um modelo do catálogo, com preço em dólares por milhão de tokens.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub context_length: u64,
    pub prompt_per_million: f64,
    pub completion_per_million: f64,
    pub free: bool,
}

fn price_per_million(value: &Value) -> f64 {
    let per_token = match value {
        Value::String(text) => text.parse::<f64>().unwrap_or(0.0),
        Value::Number(number) => number.as_f64().unwrap_or(0.0),
        _ => 0.0,
    };
    (per_token * 1_000_000.0 * 100.0).round() / 100.0
}

/// Só modelos que aceitam ferramentas: sem elas o Azriel não consegue agir nem programar.
pub fn parse_models(payload: &Value) -> Vec<OpenRouterModel> {
    let mut models: Vec<OpenRouterModel> = payload["data"]
        .as_array()
        .map(|list| list.as_slice())
        .unwrap_or_default()
        .iter()
        .filter(|model| model["supported_parameters"].as_array().is_some_and(|params| params.iter().any(|p| p == "tools")))
        .filter_map(|model| {
            let id = model["id"].as_str()?.to_string();
            if !openrouter_repository::valid_model(&id) { return None; }
            let prompt = price_per_million(&model["pricing"]["prompt"]);
            let completion = price_per_million(&model["pricing"]["completion"]);
            Some(OpenRouterModel {
                name: model["name"].as_str().unwrap_or(&id).to_string(),
                context_length: model["context_length"].as_u64().unwrap_or(0),
                free: id.ends_with(":free") || (prompt == 0.0 && completion == 0.0),
                prompt_per_million: prompt,
                completion_per_million: completion,
                id,
            })
        })
        .collect();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    models
}

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn get_openrouter_settings(state: State<'_, DatabaseState>) -> Result<OpenRouterSettings, String> {
    openrouter_repository::get(&*lock(&state)?)
}

#[tauri::command]
pub fn update_openrouter_settings(state: State<'_, DatabaseState>, input: OpenRouterSettingsInput) -> Result<OpenRouterSettings, String> {
    openrouter_repository::update(&*lock(&state)?, &input)
}

/// Catálogo de modelos com ferramentas. Não exige chave, mas usa a do Senhor se houver.
#[tauri::command]
pub async fn list_openrouter_models(state: State<'_, DatabaseState>) -> Result<Vec<OpenRouterModel>, String> {
    let key = openrouter_repository::api_key(&*lock(&state)?);
    let client = Client::builder().timeout(Duration::from_secs(30)).build().map_err(|_| "Não foi possível preparar o cliente do OpenRouter".to_string())?;
    let mut request = client.get(format!("{BASE_URL}/models"));
    if let Some(key) = key { request = request.bearer_auth(key); }
    let response = request.send().await.map_err(|_| "Não foi possível conectar ao OpenRouter. Verifique a internet.".to_string())?;
    if !response.status().is_success() {
        return Err(format!("O OpenRouter respondeu com status {} ao listar os modelos", response.status().as_u16()));
    }
    let payload: Value = response.json().await.map_err(|_| "Lista de modelos inválida recebida do OpenRouter".to_string())?;
    Ok(parse_models(&payload))
}

#[tauri::command]
pub async fn openrouter_agent_chat(state: State<'_, DatabaseState>, messages: Vec<OllamaAgentMessage>, tools: Vec<Value>, timeout_seconds: u64) -> Result<OllamaAgentResult, String> {
    let (api_key, model) = openrouter_repository::credentials(&*lock(&state)?)?;
    gemini::chat_compatible(&OPENROUTER, &api_key, &model, messages, tools, timeout_seconds).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn catalog_keeps_only_tool_models_with_prices_per_million() {
        let payload = json!({"data": [
            {"id": "deepseek/deepseek-chat", "name": "DeepSeek V3", "context_length": 163840, "pricing": {"prompt": "0.00000027", "completion": "0.0000011"}, "supported_parameters": ["tools", "temperature"]},
            {"id": "moonshotai/kimi-k2:free", "name": "Kimi K2 (free)", "context_length": 32768, "pricing": {"prompt": "0", "completion": "0"}, "supported_parameters": ["tools"]},
            {"id": "some/image-model", "name": "Sem ferramentas", "pricing": {"prompt": "0.000001", "completion": "0.000001"}, "supported_parameters": ["temperature"]},
            {"id": "bad id; drop", "supported_parameters": ["tools"]}
        ]});
        let models = parse_models(&payload);
        assert_eq!(models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), vec!["deepseek/deepseek-chat", "moonshotai/kimi-k2:free"]);
        assert_eq!((models[0].prompt_per_million, models[0].completion_per_million, models[0].free), (0.27, 1.1, false));
        assert!(models[1].free);
        assert_eq!(models[0].context_length, 163_840);
        assert!(parse_models(&json!({})).is_empty());
    }
}
