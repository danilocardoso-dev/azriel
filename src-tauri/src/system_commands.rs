use crate::system_monitor::{ProcessSnapshot, SystemMonitorState, SystemSnapshot};
use tauri::State;

#[tauri::command]
pub fn system_snapshot(state: State<'_, SystemMonitorState>) -> Result<SystemSnapshot, String> {
    let mut monitor = state.0.lock().map_err(|_| "Monitor do sistema indisponível".to_string())?;
    Ok(monitor.snapshot())
}

#[tauri::command]
pub fn list_processes(state: State<'_, SystemMonitorState>) -> Result<Vec<ProcessSnapshot>, String> {
    let mut monitor = state.0.lock().map_err(|_| "Monitor de processos indisponível".to_string())?;
    Ok(monitor.processes())
}
