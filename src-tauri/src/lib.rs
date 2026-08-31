mod commands;
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
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar o Azriel");
}
