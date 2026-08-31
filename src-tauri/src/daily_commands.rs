use crate::database::{daily_models::*, daily_repository, DatabaseState};
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
pub fn list_tasks(state: State<'_, DatabaseState>) -> Result<Vec<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::list_tasks(&connection)
}
#[tauri::command]
pub fn get_task(state: State<'_, DatabaseState>, id: String) -> Result<Option<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::get_task(&connection, &id)
}
#[tauri::command]
pub fn save_task(state: State<'_, DatabaseState>, input: TaskInput) -> Result<Task, String> {
    let connection = lock(&state)?;
    daily_repository::save_task(&connection, &input)
}
#[tauri::command]
pub fn complete_task(state: State<'_, DatabaseState>, id: String) -> Result<Task, String> {
    let connection = lock(&state)?;
    daily_repository::complete_task(&connection, &id)
}
#[tauri::command]
pub fn delete_task(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    daily_repository::delete_task(&connection, &id)
}
#[tauri::command]
pub fn list_today_tasks(
    state: State<'_, DatabaseState>,
    today: String,
) -> Result<Vec<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::list_today_tasks(&connection, &today)
}
#[tauri::command]
pub fn list_upcoming_tasks(
    state: State<'_, DatabaseState>,
    today: String,
) -> Result<Vec<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::list_upcoming_tasks(&connection, &today)
}
#[tauri::command]
pub fn list_inbox_tasks(state: State<'_, DatabaseState>) -> Result<Vec<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::list_inbox_tasks(&connection)
}
#[tauri::command]
pub fn list_completed_tasks(state: State<'_, DatabaseState>) -> Result<Vec<Task>, String> {
    let connection = lock(&state)?;
    daily_repository::list_completed_tasks(&connection)
}

#[tauri::command]
pub fn list_notes(
    state: State<'_, DatabaseState>,
    include_archived: bool,
) -> Result<Vec<Note>, String> {
    let connection = lock(&state)?;
    daily_repository::list_notes(&connection, include_archived)
}
#[tauri::command]
pub fn get_note(state: State<'_, DatabaseState>, id: String) -> Result<Option<Note>, String> {
    let connection = lock(&state)?;
    daily_repository::get_note(&connection, &id)
}
#[tauri::command]
pub fn save_note(state: State<'_, DatabaseState>, input: NoteInput) -> Result<Note, String> {
    let connection = lock(&state)?;
    daily_repository::save_note(&connection, &input)
}
#[tauri::command]
pub fn archive_note(state: State<'_, DatabaseState>, id: String) -> Result<Note, String> {
    let connection = lock(&state)?;
    daily_repository::archive_note(&connection, &id)
}
#[tauri::command]
pub fn delete_note(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    daily_repository::delete_note(&connection, &id)
}
#[tauri::command]
pub fn daily_counters(
    state: State<'_, DatabaseState>,
    today: String,
) -> Result<DailyCounters, String> {
    let connection = lock(&state)?;
    daily_repository::counters(&connection, &today)
}
