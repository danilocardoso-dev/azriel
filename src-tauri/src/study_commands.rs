use crate::database::{
    self, study_material_models::*, study_material_repository, study_models::*, study_repository,
    study_review_models::*, study_review_repository,
    study_workspace_models::*, study_workspace_repository, DatabaseState,
};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn start_study_session(
    state: State<'_, DatabaseState>,
    input: StartStudySessionInput,
) -> Result<StudySession, String> {
    let mut connection = lock(&state)?;
    study_repository::start_session(&mut connection, &input)
}

#[tauri::command]
pub fn pause_study_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudySession, String> {
    let connection = lock(&state)?;
    study_repository::pause_session(&connection, &id)
}

#[tauri::command]
pub fn resume_study_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudySession, String> {
    let connection = lock(&state)?;
    study_repository::resume_session(&connection, &id)
}

#[tauri::command]
pub fn complete_study_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudySession, String> {
    let connection = lock(&state)?;
    study_repository::complete_session(&connection, &id)
}

#[tauri::command]
pub fn cancel_study_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudySession, String> {
    let connection = lock(&state)?;
    study_repository::cancel_session(&connection, &id)
}

#[tauri::command]
pub fn get_active_study_session(
    state: State<'_, DatabaseState>,
) -> Result<Option<StudySession>, String> {
    let connection = lock(&state)?;
    study_repository::get_active_session(&connection)
}

#[tauri::command]
pub fn list_study_sessions(
    state: State<'_, DatabaseState>,
    input: StudySessionListInput,
) -> Result<Vec<StudySession>, String> {
    let connection = lock(&state)?;
    study_repository::list_sessions(&connection, &input)
}

#[tauri::command]
pub fn get_study_today_summary(
    state: State<'_, DatabaseState>,
) -> Result<StudyTodaySummary, String> {
    let connection = lock(&state)?;
    study_repository::today_summary(&connection)
}

#[tauri::command]
pub fn get_study_settings(state: State<'_, DatabaseState>) -> Result<StudySettings, String> {
    let connection = lock(&state)?;
    study_repository::get_settings(&connection)
}

#[tauri::command]
pub fn update_study_settings(
    state: State<'_, DatabaseState>,
    input: StudySettingsInput,
) -> Result<StudySettings, String> {
    let connection = lock(&state)?;
    study_repository::update_settings(&connection, &input)
}

#[tauri::command]
pub fn list_study_notebooks(
    state: State<'_, DatabaseState>,
    include_archived: bool,
) -> Result<Vec<StudyNotebook>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::list_notebooks(&connection, include_archived)
}

#[tauri::command]
pub fn get_study_notebook(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<StudyNotebook>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::get_notebook(&connection, &id)
}

#[tauri::command]
pub fn save_study_notebook(
    state: State<'_, DatabaseState>,
    input: StudyNotebookInput,
) -> Result<StudyNotebook, String> {
    let connection = lock(&state)?;
    study_workspace_repository::save_notebook(&connection, &input)
}

#[tauri::command]
pub fn archive_study_notebook(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudyNotebook, String> {
    let connection = lock(&state)?;
    study_workspace_repository::set_notebook_status(&connection, &id, "ARCHIVED")
}

#[tauri::command]
pub fn restore_study_notebook(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudyNotebook, String> {
    let connection = lock(&state)?;
    study_workspace_repository::set_notebook_status(&connection, &id, "ACTIVE")
}

#[tauri::command]
pub fn delete_study_notebook(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let mut connection = lock(&state)?;
    study_workspace_repository::delete_notebook(&mut connection, &id)
}

#[tauri::command]
pub fn get_study_note(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<StudyNote>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::get_note(&connection, &id)
}

#[tauri::command]
pub fn save_study_note(
    state: State<'_, DatabaseState>,
    input: StudyNoteInput,
) -> Result<StudyNote, String> {
    let connection = lock(&state)?;
    study_workspace_repository::save_note(&connection, &input)
}

#[tauri::command]
pub fn create_study_note_from_material(
    state: State<'_, DatabaseState>,
    material_id: String,
    input: StudyNoteInput,
) -> Result<StudyNote, String> {
    let mut connection = lock(&state)?;
    let transaction = connection.transaction().map_err(|error| error.to_string())?;
    let note = study_workspace_repository::save_note(&transaction, &input)?;
    study_material_repository::associate(
        &transaction,
        &material_id,
        &StudyMaterialRelationInput {
            relation_type: "NOTE".into(),
            relation_id: note.id.clone(),
        },
    )?;
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(note)
}

#[tauri::command]
pub fn delete_study_note(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    study_workspace_repository::delete_note(&connection, &id)
}

#[tauri::command]
pub fn list_study_notes(
    state: State<'_, DatabaseState>,
    input: StudyNoteListInput,
) -> Result<Vec<StudyNoteSummary>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::list_notes(&connection, &input)
}

#[tauri::command]
pub fn search_study_notes(
    state: State<'_, DatabaseState>,
    input: StudyNoteSearchInput,
) -> Result<Vec<StudyNoteSummary>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::search_notes(&connection, &input)
}

#[tauri::command]
pub fn list_recent_study_notes(
    state: State<'_, DatabaseState>,
    limit: i64,
) -> Result<Vec<StudyNoteSummary>, String> {
    let connection = lock(&state)?;
    study_workspace_repository::list_recent_notes(&connection, limit)
}

#[tauri::command]
pub fn save_study_card(
    state: State<'_, DatabaseState>,
    input: StudyCardInput,
) -> Result<StudyCard, String> {
    let connection = lock(&state)?;
    study_review_repository::save_card(&connection, &input)
}

#[tauri::command]
pub fn save_study_cards(
    state: State<'_, DatabaseState>,
    inputs: Vec<StudyCardInput>,
) -> Result<Vec<StudyCard>, String> {
    let mut connection = lock(&state)?;
    study_review_repository::save_cards(&mut connection, &inputs)
}

#[tauri::command]
pub fn get_study_card(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Option<StudyCard>, String> {
    let connection = lock(&state)?;
    study_review_repository::get_card(&connection, &id)
}

#[tauri::command]
pub fn list_study_cards(
    state: State<'_, DatabaseState>,
    input: StudyCardListInput,
) -> Result<Vec<StudyCard>, String> {
    let connection = lock(&state)?;
    study_review_repository::list_cards(&connection, &input)
}

#[tauri::command]
pub fn set_study_card_status(
    state: State<'_, DatabaseState>,
    id: String,
    status: String,
) -> Result<StudyCard, String> {
    let connection = lock(&state)?;
    study_review_repository::set_card_status(&connection, &id, &status)
}

#[tauri::command]
pub fn delete_study_card(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    study_review_repository::delete_card(&connection, &id)
}

#[tauri::command]
pub fn get_study_review_dashboard_summary(
    state: State<'_, DatabaseState>,
) -> Result<StudyReviewDashboardSummary, String> {
    let connection = lock(&state)?;
    study_review_repository::dashboard_summary(&connection)
}

#[tauri::command]
pub fn get_study_review_queue(
    state: State<'_, DatabaseState>,
    mode: String,
) -> Result<Vec<StudyReviewQueueItem>, String> {
    let connection = lock(&state)?;
    study_review_repository::get_review_queue(&connection, &mode)
}

#[tauri::command]
pub fn start_study_review_session(
    state: State<'_, DatabaseState>,
    input: StartStudyReviewSessionInput,
) -> Result<StudyReviewSession, String> {
    let mut connection = lock(&state)?;
    study_review_repository::start_review_session(&mut connection, &input)
}

#[tauri::command]
pub fn get_active_study_review_session(
    state: State<'_, DatabaseState>,
) -> Result<Option<StudyReviewSession>, String> {
    let connection = lock(&state)?;
    study_review_repository::get_active_review_session(&connection)
}

#[tauri::command]
pub fn submit_study_review_result(
    state: State<'_, DatabaseState>,
    input: SubmitStudyReviewResultInput,
) -> Result<StudyReviewResult, String> {
    let mut connection = lock(&state)?;
    study_review_repository::submit_review_result(&mut connection, &input)
}

#[tauri::command]
pub fn complete_study_review_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudyReviewSession, String> {
    let mut connection = lock(&state)?;
    study_review_repository::complete_review_session(&mut connection, &id)
}

#[tauri::command]
pub fn cancel_study_review_session(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudyReviewSession, String> {
    let mut connection = lock(&state)?;
    study_review_repository::cancel_review_session(&mut connection, &id)
}

#[tauri::command]
pub fn get_study_review_summary(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<StudyReviewSummary, String> {
    let connection = lock(&state)?;
    study_review_repository::get_review_summary(&connection, &id)
}

#[tauri::command]
pub fn list_study_review_sessions(
    state: State<'_, DatabaseState>,
    input: StudyReviewSessionListInput,
) -> Result<Vec<StudyReviewSession>, String> {
    let connection = lock(&state)?;
    study_review_repository::list_review_sessions(&connection, &input)
}

#[tauri::command]
pub async fn import_study_material(
    state: State<'_, DatabaseState>,
    input: ImportStudyMaterialInput,
) -> Result<ImportStudyMaterialResult, String> {
    let database_path = state.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let started = std::time::Instant::now();
        let mut connection = database::open(&database_path)?;
        let result = study_material_repository::import_material(&mut connection, &database_path, &input);
        eprintln!("[STUDY_MATERIAL] operation=import id={} status={} duration_ms={}", input.id, if result.is_ok() { "success" } else { "error" }, started.elapsed().as_millis());
        result
    }).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn add_study_link_material(state: State<'_, DatabaseState>, input: AddLinkStudyMaterialInput) -> Result<StudyMaterial, String> {
    let mut connection = lock(&state)?;
    study_material_repository::add_link_material(&mut connection, &input)
}

#[tauri::command]
pub fn list_study_materials(state: State<'_, DatabaseState>, input: StudyMaterialListInput) -> Result<Vec<StudyMaterialSummary>, String> {
    let connection = lock(&state)?;
    study_material_repository::list_materials(&connection, &input)
}

#[tauri::command]
pub fn get_study_material(state: State<'_, DatabaseState>, id: String) -> Result<Option<StudyMaterial>, String> {
    let connection = lock(&state)?;
    study_material_repository::get_material(&connection, &id)
}

#[tauri::command]
pub fn update_study_material(state: State<'_, DatabaseState>, input: UpdateStudyMaterialInput) -> Result<StudyMaterial, String> {
    let connection = lock(&state)?;
    study_material_repository::update_material(&connection, &input)
}

#[tauri::command]
pub fn set_study_material_status(state: State<'_, DatabaseState>, id: String, status: String) -> Result<StudyMaterial, String> {
    let connection = lock(&state)?;
    study_material_repository::set_status(&connection, &id, &status)
}

#[tauri::command]
pub fn remove_study_material(state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    study_material_repository::remove_from_library(&connection, &id)
}

#[tauri::command]
pub fn delete_managed_study_material_file(state: State<'_, DatabaseState>, id: String) -> Result<StudyMaterial, String> {
    let connection = lock(&state)?;
    study_material_repository::delete_managed_file(&connection, &state.path, &id)
}

#[tauri::command]
pub fn associate_study_material(state: State<'_, DatabaseState>, material_id: String, input: StudyMaterialRelationInput) -> Result<Vec<StudyMaterialRelation>, String> {
    let connection = lock(&state)?;
    study_material_repository::associate(&connection, &material_id, &input)
}

#[tauri::command]
pub fn dissociate_study_material(state: State<'_, DatabaseState>, material_id: String, relation_type: String, relation_id: String) -> Result<Vec<StudyMaterialRelation>, String> {
    let connection = lock(&state)?;
    study_material_repository::dissociate(&connection, &material_id, &relation_type, &relation_id)
}

#[tauri::command]
pub async fn reprocess_study_material_text(state: State<'_, DatabaseState>, id: String) -> Result<MaterialTextContent, String> {
    let database_path = state.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let started = std::time::Instant::now();
        let connection = database::open(&database_path)?;
        let result = study_material_repository::reprocess_text(&connection, &id);
        eprintln!("[STUDY_MATERIAL] operation=reprocess id={} status={} duration_ms={}", id, if result.is_ok() { "success" } else { "error" }, started.elapsed().as_millis());
        result
    }).await.map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn preview_study_material(state: State<'_, DatabaseState>, id: String) -> Result<tauri::ipc::Response, String> {
    let connection = lock(&state)?;
    let preview = study_material_repository::preview(&connection, &id)?;
    Ok(tauri::ipc::Response::new(preview.bytes))
}

#[tauri::command]
pub fn open_study_material(app: AppHandle, state: State<'_, DatabaseState>, id: String) -> Result<(), String> {
    let connection = lock(&state)?;
    let target = study_material_repository::open_target(&connection, &id)?;
    if target.storage_kind == "EXTERNAL_URL" {
        let url = target.external_url.ok_or("Link externo indisponível.")?;
        let parsed = reqwest::Url::parse(&url).map_err(|_| "URL persistida inválida.".to_string())?;
        if !matches!(parsed.scheme(), "http" | "https") { return Err("Protocolo de URL não permitido.".into()); }
        app.opener().open_url(parsed.as_str(), None::<&str>).map_err(|error| error.to_string())
    } else {
        let path = target.local_path.ok_or("Arquivo local indisponível.")?;
        if !std::path::Path::new(&path).is_file() { return Err("Arquivo local não encontrado.".into()); }
        app.opener().open_path(path, None::<&str>).map_err(|error| error.to_string())
    }
}
