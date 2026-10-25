mod ai_commands;
mod commands;
mod daily_commands;
pub mod database;
mod ollama;
mod system_commands;
mod system_monitor;

use database::DatabaseState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(system_monitor::SystemMonitorState(std::sync::Mutex::new(
            system_monitor::SystemMonitor::new(),
        )))
        .setup(|app| {
            let database_path = app.path().app_data_dir()?.join("azriel.db");
            let connection = database::open(&database_path)
                .map_err(|error| format!("erro ao iniciar o banco: {error}"))?;
            app.manage(DatabaseState {
                connection: std::sync::Mutex::new(connection),
                path: database_path,
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" { return; }
            match event {
                tauri::WindowEvent::Resized(_) if window.is_minimized().unwrap_or(false) => {
                    if let Some(orb) = window.app_handle().get_webview_window("orb") { let _ = orb.show(); }
                    let _ = window.hide();
                }
                tauri::WindowEvent::Destroyed => {
                    if let Some(orb) = window.app_handle().get_webview_window("orb") { let _ = orb.close(); }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::database_info,
            daily_commands::list_tasks,
            daily_commands::get_task,
            daily_commands::save_task,
            daily_commands::complete_task,
            daily_commands::delete_task,
            daily_commands::list_today_tasks,
            daily_commands::list_upcoming_tasks,
            daily_commands::list_inbox_tasks,
            daily_commands::list_completed_tasks,
            daily_commands::list_notes,
            daily_commands::get_note,
            daily_commands::save_note,
            daily_commands::archive_note,
            daily_commands::delete_note,
            daily_commands::daily_counters,
            ai_commands::get_ai_settings,
            ai_commands::update_ai_settings,
            ai_commands::list_conversations,
            ai_commands::create_conversation,
            ai_commands::delete_conversation,
            ai_commands::clear_conversation_messages,
            ai_commands::list_messages,
            ai_commands::add_message,
            ai_commands::save_ai_task_references,
            ai_commands::complete_ai_referenced_task,
            ai_commands::ollama_status,
            ai_commands::get_local_ai_model_status,
            ai_commands::load_local_ai_model,
            ai_commands::unload_local_ai_model,
            ai_commands::ollama_chat,
            system_commands::system_snapshot,
            system_commands::list_processes,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar Azriel");
}
