use crate::database::{stark_models::*, stark_repository, DatabaseState};
use tauri::State;

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn list_knowledge_baselines(state: State<'_, DatabaseState>) -> Result<Vec<KnowledgeBaseline>, String> {
    let connection = lock(&state)?;
    stark_repository::list_baselines(&connection)
}

#[tauri::command]
pub fn list_knowledge_events(state: State<'_, DatabaseState>) -> Result<Vec<KnowledgeEvent>, String> {
    let connection = lock(&state)?;
    stark_repository::list_events(&connection)
}

#[tauri::command]
pub fn list_study_roadmaps(state: State<'_, DatabaseState>) -> Result<Vec<StudyRoadmap>, String> {
    let connection = lock(&state)?;
    stark_repository::list_roadmaps(&connection)
}

#[tauri::command]
pub fn save_study_roadmap(state: State<'_, DatabaseState>, input: StudyRoadmapInput) -> Result<RoadmapSaveResult, String> {
    let mut connection = lock(&state)?;
    let learning=stark_repository::save_roadmap(&mut connection, &input)?;
    Ok(RoadmapSaveResult { roadmaps: stark_repository::list_roadmaps(&connection)?, learning })
}

#[tauri::command]
pub fn delete_study_roadmap(state: State<'_, DatabaseState>, id: String) -> Result<Vec<StudyRoadmap>, String> {
    let connection = lock(&state)?;
    stark_repository::delete_roadmap(&connection, &id)?;
    stark_repository::list_roadmaps(&connection)
}

#[tauri::command]
pub fn list_research_items(state: State<'_, DatabaseState>) -> Result<Vec<ResearchItem>, String> {
    let connection = lock(&state)?;
    stark_repository::list_research(&connection)
}

#[tauri::command]
pub fn save_research_item(state: State<'_, DatabaseState>, input: ResearchItem) -> Result<Vec<ResearchItem>, String> {
    let connection = lock(&state)?;
    stark_repository::save_research(&connection, &input)?;
    stark_repository::list_research(&connection)
}

#[tauri::command]
pub fn delete_research_item(state: State<'_, DatabaseState>, id: String) -> Result<Vec<ResearchItem>, String> {
    let connection = lock(&state)?;
    stark_repository::delete_research(&connection, &id)?;
    stark_repository::list_research(&connection)
}

#[tauri::command]
pub fn get_stark_summary(state: State<'_, DatabaseState>) -> Result<StarkSummary, String> {
    let connection = lock(&state)?;
    stark_repository::summary(&connection)
}

#[tauri::command]
pub fn get_learning_engine_status(state: State<'_, DatabaseState>) -> Result<LearningEngineStatus,String>{
    let connection=lock(&state)?; crate::database::learning_engine::status(&connection)
}

#[tauri::command]
pub fn rebuild_learning_engine(state: State<'_, DatabaseState>) -> Result<LearningMutation,String>{
    let mut connection=lock(&state)?;
    let transaction=connection.transaction().map_err(|e|e.to_string())?;
    let result=crate::database::learning_engine::recalculate(&transaction,"Learning Engine: reconstrução administrativa")?;
    transaction.commit().map_err(|e|e.to_string())?; Ok(result)
}
