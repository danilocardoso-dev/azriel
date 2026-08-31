use crate::database::{self, models::*, repository, DatabaseState};
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
pub fn database_info(state: State<'_, DatabaseState>) -> Result<DatabaseInfo, String> {
    let connection = lock(&state)?;
    let integration_value = connection
        .query_row(
            "SELECT value FROM app_metrics WHERE key='integration'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(DatabaseInfo {
        path: state.path.display().to_string(),
        schema_version: database::schema_version(&connection)?,
        integration_value,
    })
}

#[tauri::command]
pub fn list_knowledge(state: State<'_, DatabaseState>) -> Result<Vec<KnowledgeArea>, String> {
    let connection = lock(&state)?;
    repository::list_knowledge(&connection)
}

#[tauri::command]
pub fn get_knowledge(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<KnowledgeArea>, String> {
    let connection = lock(&state)?;
    repository::get_knowledge(&connection, &id)
}

#[tauri::command]
pub fn save_knowledge(
    state: State<'_, DatabaseState>,
    input: KnowledgeInput,
) -> Result<Vec<KnowledgeArea>, String> {
    let mut connection = lock(&state)?;
    repository::save_knowledge(&mut connection, &input)?;
    repository::list_knowledge(&connection)
}

#[tauri::command]
pub fn delete_knowledge(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Vec<KnowledgeArea>, String> {
    let connection = lock(&state)?;
    repository::delete_knowledge(&connection, &id)?;
    repository::list_knowledge(&connection)
}

#[tauri::command]
pub fn update_knowledge_metrics(
    state: State<'_, DatabaseState>,
    input: MetricsInput,
) -> Result<KnowledgeArea, String> {
    let mut connection = lock(&state)?;
    repository::update_metrics(&mut connection, &input)
}

#[tauri::command]
pub fn list_knowledge_history(
    state: State<'_, DatabaseState>,
    knowledge_id: String,
) -> Result<Vec<KnowledgeHistory>, String> {
    let connection = lock(&state)?;
    repository::list_history(&connection, &knowledge_id)
}

#[tauri::command]
pub fn list_projects(state: State<'_, DatabaseState>) -> Result<Vec<Project>, String> {
    let connection = lock(&state)?;
    repository::list_projects(&connection)
}

#[tauri::command]
pub fn get_project(state: State<'_, DatabaseState>, id: String) -> Result<Option<Project>, String> {
    let connection = lock(&state)?;
    repository::get_project(&connection, &id)
}

#[tauri::command]
pub fn save_project(
    state: State<'_, DatabaseState>,
    input: ProjectInput,
) -> Result<Vec<Project>, String> {
    let mut connection = lock(&state)?;
    repository::save_project(&mut connection, &input)?;
    repository::list_projects(&connection)
}

#[tauri::command]
pub fn delete_project(state: State<'_, DatabaseState>, id: String) -> Result<Vec<Project>, String> {
    let connection = lock(&state)?;
    repository::delete_project(&connection, &id)?;
    repository::list_projects(&connection)
}

#[tauri::command]
pub fn list_education(state: State<'_, DatabaseState>) -> Result<Vec<EducationItem>, String> {
    let connection = lock(&state)?;
    repository::list_education(&connection)
}

#[tauri::command]
pub fn save_education(
    state: State<'_, DatabaseState>,
    input: EducationInput,
) -> Result<Vec<EducationItem>, String> {
    let connection = lock(&state)?;
    repository::save_education(&connection, &input)?;
    repository::list_education(&connection)
}

#[tauri::command]
pub fn delete_education(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Vec<EducationItem>, String> {
    let connection = lock(&state)?;
    repository::delete_education(&connection, &id)?;
    repository::list_education(&connection)
}
