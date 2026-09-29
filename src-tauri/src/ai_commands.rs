use crate::{
    database::{ai_models::*, ai_repository, daily_models::Task, DatabaseState},
    ollama::{self, LocalAiModelStatus, OllamaChatResult, OllamaMessage, OllamaStatus},
};
use serde::Deserialize;
use serde_json::Value;
use std::time::Instant;
use tauri::State;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiRequestMetadata {
    domain: String,
    action: String,
    prompt_version: String,
    context_truncated: bool,
    #[serde(default)]
    source_material_count: usize,
}

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn get_ai_settings(state: State<'_, DatabaseState>) -> Result<AiSettings, String> {
    let connection = lock(&state)?;
    ai_repository::get_settings(&connection)
}
#[tauri::command]
pub fn update_ai_settings(
    state: State<'_, DatabaseState>,
    input: AiSettingsInput,
) -> Result<AiSettings, String> {
    ollama::normalize_endpoint(&input.endpoint)?;
    let connection = lock(&state)?;
    ai_repository::update_settings(&connection, &input)
}
#[tauri::command]
pub fn list_conversations(state: State<'_, DatabaseState>) -> Result<Vec<Conversation>, String> {
    let connection = lock(&state)?;
    ai_repository::list_conversations(&connection)
}
#[tauri::command]
pub fn create_conversation(
    state: State<'_, DatabaseState>,
    input: ConversationInput,
) -> Result<Conversation, String> {
    let connection = lock(&state)?;
    ai_repository::create_conversation(&connection, &input)
}
#[tauri::command]
pub fn delete_conversation(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    ai_repository::delete_conversation(&connection, &id)
}
#[tauri::command]
pub fn clear_conversation_messages(
    state: State<'_, DatabaseState>,
    conversation_id: String,
) -> Result<usize, String> {
    let mut connection = lock(&state)?;
    ai_repository::clear_conversation_messages(&mut connection, &conversation_id)
}
#[tauri::command]
pub fn list_messages(
    state: State<'_, DatabaseState>,
    conversation_id: String,
) -> Result<Vec<Message>, String> {
    let connection = lock(&state)?;
    ai_repository::list_messages(&connection, &conversation_id)
}
#[tauri::command]
pub fn add_message(
    state: State<'_, DatabaseState>,
    input: MessageInput,
) -> Result<Message, String> {
    let mut connection = lock(&state)?;
    ai_repository::add_message(&mut connection, &input)
}

#[tauri::command]
pub fn save_ai_task_references(
    state: State<'_, DatabaseState>,
    input: SaveTaskReferencesInput,
) -> Result<usize, String> {
    let mut connection = lock(&state)?;
    ai_repository::save_task_references(&mut connection, &input)
}

#[tauri::command]
pub fn complete_ai_referenced_task(
    state: State<'_, DatabaseState>,
    conversation_id: String,
    position: i64,
) -> Result<Task, String> {
    let mut connection = lock(&state)?;
    ai_repository::complete_referenced_task(&mut connection, &conversation_id, position)
}

#[tauri::command]
pub async fn ollama_status(endpoint: String, timeout_seconds: u64) -> Result<OllamaStatus, String> {
    ollama::status(&endpoint, timeout_seconds).await
}

fn configured_ollama(state: &State<'_, DatabaseState>) -> Result<(String, String, u64), String> {
    let settings = {
        let connection = lock(state)?;
        ai_repository::get_settings(&connection)?
    };
    Ok((
        settings.endpoint,
        settings.model,
        settings.timeout_seconds.clamp(5, 180) as u64,
    ))
}

#[tauri::command]
pub async fn get_local_ai_model_status(
    state: State<'_, DatabaseState>,
) -> Result<LocalAiModelStatus, String> {
    let (endpoint, model, timeout) = configured_ollama(&state)?;
    ollama::model_lifecycle_status(&endpoint, &model, timeout.min(12)).await
}

#[tauri::command]
pub async fn load_local_ai_model(
    state: State<'_, DatabaseState>,
) -> Result<LocalAiModelStatus, String> {
    let (endpoint, model, timeout) = configured_ollama(&state)?;
    ollama::load_model(&endpoint, &model, timeout).await
}

#[tauri::command]
pub async fn unload_local_ai_model(
    state: State<'_, DatabaseState>,
) -> Result<LocalAiModelStatus, String> {
    let (endpoint, model, timeout) = configured_ollama(&state)?;
    ollama::unload_model(&endpoint, &model, timeout).await
}
#[tauri::command]
pub async fn ollama_chat(
    endpoint: String,
    model: String,
    messages: Vec<OllamaMessage>,
    timeout_seconds: u64,
    generation_profile: String,
    structured_output_schema: Option<Value>,
    request_metadata: Option<AiRequestMetadata>,
) -> Result<OllamaChatResult, String> {
    let started = Instant::now();
    let result = match structured_output_schema {
        Some(schema) => {
            ollama::chat_with_format(
                &endpoint,
                &model,
                messages,
                timeout_seconds,
                &generation_profile,
                Some(schema),
            )
            .await
        }
        None => {
            ollama::chat(
                &endpoint,
                &model,
                messages,
                timeout_seconds,
                &generation_profile,
            )
            .await
        }
    };
    if let Some(metadata) = request_metadata {
        let safe = |value: &str| {
            value
                .chars()
                .filter(|character| {
                    character.is_ascii_alphanumeric() || ['_', '-'].contains(character)
                })
                .take(80)
                .collect::<String>()
        };
        let (status, error) = match &result {
            Ok(_) => ("SUCCESS", "none"),
            Err(message)
                if message.to_lowercase().contains("tempo limite")
                    || message.to_lowercase().contains("timeout") =>
            {
                ("TIMEOUT", "timeout")
            }
            Err(_) => ("PROVIDER_ERROR", "provider_error"),
        };
        eprintln!(
            "[AI_CORE] domain={} action={} prompt_version={} status={} latency_ms={} context_truncated={} source_material_count={} error={}",
            safe(&metadata.domain),
            safe(&metadata.action),
            safe(&metadata.prompt_version),
            status,
            started.elapsed().as_millis(),
            metadata.context_truncated,
            metadata.source_material_count,
            error,
        );
    }
    result
}
