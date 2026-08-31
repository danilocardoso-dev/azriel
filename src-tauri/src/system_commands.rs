use crate::{
    database::{
        system_models::{Workspace, WorkspaceInput},
        system_repository, DatabaseState,
    },
    git_monitor::{self, GitStatus},
    system_monitor::{ProcessSnapshot, SystemMonitorState, SystemSnapshot},
};
use serde::Serialize;
use std::path::Path;
use tauri::State;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatus {
    pub workspace: Workspace,
    pub path_available: bool,
    pub entry_count: Option<usize>,
    pub git: Option<GitStatus>,
    pub error: Option<String>,
}

fn database_lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn system_snapshot(state: State<'_, SystemMonitorState>) -> Result<SystemSnapshot, String> {
    let mut monitor = state
        .0
        .lock()
        .map_err(|_| "O monitor do sistema está indisponível".to_string())?;
    Ok(monitor.snapshot())
}

#[tauri::command]
pub fn list_processes(
    state: State<'_, SystemMonitorState>,
) -> Result<Vec<ProcessSnapshot>, String> {
    let mut monitor = state
        .0
        .lock()
        .map_err(|_| "O monitor de processos está indisponível".to_string())?;
    Ok(monitor.processes())
}

#[tauri::command]
pub fn list_workspaces(state: State<'_, DatabaseState>) -> Result<Vec<Workspace>, String> {
    let connection = database_lock(&state)?;
    system_repository::list_workspaces(&connection)
}

#[tauri::command]
pub fn save_workspace(
    state: State<'_, DatabaseState>,
    input: WorkspaceInput,
) -> Result<Vec<Workspace>, String> {
    let connection = database_lock(&state)?;
    system_repository::save_workspace(&connection, &input)?;
    system_repository::list_workspaces(&connection)
}

#[tauri::command]
pub fn delete_workspace(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Vec<Workspace>, String> {
    let connection = database_lock(&state)?;
    system_repository::delete_workspace(&connection, &id)?;
    system_repository::list_workspaces(&connection)
}

#[tauri::command]
pub fn get_workspace_status(
    state: State<'_, DatabaseState>,
    workspace_id: String,
) -> Result<WorkspaceStatus, String> {
    let connection = database_lock(&state)?;
    let workspace = system_repository::get_workspace(&connection, &workspace_id)?
        .ok_or_else(|| "Workspace não encontrado".to_string())?;
    drop(connection);
    if !workspace.enabled {
        return Err("O workspace está desabilitado".into());
    }
    let path = Path::new(&workspace.path);
    if !path.is_dir() {
        return Ok(WorkspaceStatus {
            workspace,
            path_available: false,
            entry_count: None,
            git: None,
            error: Some("A pasta cadastrada não está disponível".into()),
        });
    }
    let entry_count = std::fs::read_dir(path)
        .ok()
        .map(|entries| entries.filter_map(Result::ok).take(10_001).count());
    let git = git_monitor::inspect(path);
    Ok(WorkspaceStatus {
        workspace,
        path_available: true,
        entry_count,
        git: Some(git),
        error: None,
    })
}
