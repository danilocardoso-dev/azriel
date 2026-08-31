mod commands;
mod daily_commands;
mod database;

use database::DatabaseState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database_path = app.path().app_data_dir()?.join("azriel.db");
            let connection = database::open(&database_path)
                .map_err(|error| format!("erro ao iniciar o banco de dados: {error}"))?;
            app.manage(DatabaseState { connection: std::sync::Mutex::new(connection), path: database_path });
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
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Azriel");
}
