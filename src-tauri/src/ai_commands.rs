//! Esqueleto do AI Core: sem motor de IA por enquanto. Ficam as conversas salvas (consulta e
//! limpeza) e a chave do Google usada pela chamada de voz (Gemini Live).

use crate::database::{ai_models::*, ai_repository, DatabaseState};
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
pub fn list_conversations(state: State<'_, DatabaseState>) -> Result<Vec<Conversation>, String> {
    let connection = lock(&state)?;
    ai_repository::list_conversations(&connection)
}

#[tauri::command]
pub fn delete_conversation(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    ai_repository::delete_conversation(&connection, &id)
}

#[tauri::command]
pub fn clear_conversation_messages(state: State<'_, DatabaseState>, conversation_id: String) -> Result<usize, String> {
    let mut connection = lock(&state)?;
    ai_repository::clear_conversation_messages(&mut connection, &conversation_id)
}

#[tauri::command]
pub fn list_messages(state: State<'_, DatabaseState>, conversation_id: String) -> Result<Vec<Message>, String> {
    let connection = lock(&state)?;
    ai_repository::list_messages(&connection, &conversation_id)
}

/// Chave do Google (Gemini Live) usada pela chamada de voz.
#[tauri::command]
pub fn get_remote_ai_settings(state: State<'_, DatabaseState>) -> Result<RemoteAiSettings, String> {
    let connection = lock(&state)?;
    ai_repository::get_remote_settings(&connection)
}

#[tauri::command]
pub fn update_remote_ai_settings(state: State<'_, DatabaseState>, input: RemoteAiSettingsInput) -> Result<RemoteAiSettings, String> {
    let connection = lock(&state)?;
    ai_repository::update_remote_settings(&connection, &input)
}
