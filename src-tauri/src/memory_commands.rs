use crate::database::{memory_models::*, memory_repository, DatabaseState};
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
pub fn get_azriel_identity(state: State<'_, DatabaseState>) -> Result<Identity, String> {
    memory_repository::get_identity(&*lock(&state)?)
}
#[tauri::command]
pub fn save_azriel_identity(state: State<'_, DatabaseState>, content: String) -> Result<Identity, String> {
    memory_repository::save_identity(&*lock(&state)?, &content)
}
#[tauri::command]
pub fn reset_azriel_identity(state: State<'_, DatabaseState>) -> Result<Identity, String> {
    memory_repository::reset_identity(&*lock(&state)?)
}
#[tauri::command]
pub fn list_memories(state: State<'_, DatabaseState>) -> Result<Vec<Memory>, String> {
    memory_repository::list_memories(&*lock(&state)?)
}
#[tauri::command]
pub fn search_memories(state: State<'_, DatabaseState>, query: String) -> Result<Vec<Memory>, String> {
    memory_repository::search_memories(&*lock(&state)?, &query)
}
#[tauri::command]
pub fn save_memory(state: State<'_, DatabaseState>, input: MemoryInput) -> Result<Memory, String> {
    memory_repository::save_memory(&*lock(&state)?, &input)
}
#[tauri::command]
pub fn delete_memory(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    memory_repository::delete_memory(&*lock(&state)?, &id)
}
