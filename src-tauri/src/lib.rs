mod ai_commands;
mod automation_commands;
mod automation_executor;
mod automation_policy;
mod commands;
mod daily_commands;
mod database;
mod git_monitor;
mod ollama;
mod routine_commands;
mod system_commands;
mod system_monitor;

use database::DatabaseState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(system_monitor::SystemMonitorState(std::sync::Mutex::new(
            system_monitor::SystemMonitor::new(),
        )))
        .setup(|app| {
            let database_path = app.path().app_data_dir()?.join("azriel.db");
            let connection = database::open(&database_path)
                .map_err(|error| format!("erro ao iniciar o banco de dados: {error}"))?;
            app.manage(DatabaseState {
                connection: std::sync::Mutex::new(connection),
                path: database_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::database_info,
            commands::list_knowledge,
            commands::get_knowledge,
            commands::save_knowledge,
            commands::delete_knowledge,
            commands::update_knowledge_metrics,
            commands::list_knowledge_history,
            commands::list_projects,
            commands::get_project,
            commands::save_project,
            commands::delete_project,
            commands::list_education,
            commands::save_education,
            commands::delete_education,
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
            ai_commands::list_messages,
            ai_commands::add_message,
            ai_commands::ollama_status,
            ai_commands::ollama_chat,
            system_commands::system_snapshot,
            system_commands::list_processes,
            system_commands::list_workspaces,
            system_commands::save_workspace,
            system_commands::delete_workspace,
            system_commands::get_workspace_status,
            automation_commands::list_registered_actions,
            automation_commands::list_applications,
            automation_commands::save_application,
            automation_commands::delete_application,
            automation_commands::list_registered_urls,
            automation_commands::save_registered_url,
            automation_commands::delete_registered_url,
            automation_commands::list_action_history,
            automation_commands::execute_automation_action,
            routine_commands::list_routines,
            routine_commands::save_routine,
            routine_commands::delete_routine,
            routine_commands::list_routine_history,
            routine_commands::run_routine,
            routine_commands::confirm_routine_execution,
            routine_commands::cancel_routine_execution,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Azriel");
}
