use crate::{
    database::{ai_models::*, ai_repository, DatabaseState},
    ollama::{self, OllamaChatResult, OllamaMessage, OllamaStatus},
};
use tauri::State;

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
pub async fn ollama_status(endpoint: String, timeout_seconds: u64) -> Result<OllamaStatus, String> {
    ollama::status(&endpoint, timeout_seconds).await
}
#[tauri::command]
pub async fn ollama_chat(
    endpoint: String,
    model: String,
    messages: Vec<OllamaMessage>,
    timeout_seconds: u64,
    generation_profile: String,
) -> Result<OllamaChatResult, String> {
    ollama::chat(
        &endpoint,
        &model,
        messages,
        timeout_seconds,
        &generation_profile,
    )
    .await
}
