mod ai_commands;
mod commands;
mod daily_commands;
pub mod database;
mod gemini;
mod media;
mod memory_commands;
mod money_commands;
mod ollama;
mod deepseek;
mod quick_code;
mod spotify;
mod system_commands;
mod system_monitor;
mod voice;
mod workspace;

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
        .manage(voice::VoiceState::default())
        .setup(|app| {
            let database_path = app.path().app_data_dir()?.join("azriel.db");
            let connection = database::open(&database_path)
                .map_err(|error| format!("erro ao iniciar o banco: {error}"))?;
            app.manage(DatabaseState {
                connection: std::sync::Mutex::new(connection),
                path: database_path,
            });
            // As janelas só nascem depois do banco registrado: senão a interface chama
            // comandos (get_ai_settings etc.) antes do DatabaseState existir.
            for window_config in app.config().app.windows.clone() {
                tauri::WebviewWindowBuilder::from_config(app.handle(), &window_config)?.build()?;
            }
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
            commands::open_external,
            workspace::fs_list,
            workspace::fs_read,
            workspace::fs_search,
            workspace::fs_write,
            workspace::fs_edit,
            workspace::fs_mkdir,
            workspace::fs_delete,
            workspace::run_command,
            workspace::list_actions,
            workspace::cancel_commands,
            workspace::open_in_browser,
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
            ai_commands::delete_message,
            ai_commands::list_messages,
            ai_commands::add_message,
            ai_commands::save_ai_task_references,
            ai_commands::complete_ai_referenced_task,
            ai_commands::ollama_status,
            ai_commands::get_local_ai_model_status,
            ai_commands::load_local_ai_model,
            ai_commands::unload_local_ai_model,
            ai_commands::ollama_chat,
            ai_commands::ollama_agent_chat,
            ai_commands::get_remote_ai_settings,
            ai_commands::update_remote_ai_settings,
            ai_commands::list_remote_ai_models,
            ai_commands::remote_agent_chat,
            deepseek::get_deepseek_settings,
            deepseek::update_deepseek_settings,
            deepseek::list_deepseek_models,
            deepseek::deepseek_agent_chat,
            quick_code::quick_code_stream,
            quick_code::quick_code_cancel,
            ai_commands::web_search,
            memory_commands::get_azriel_identity,
            memory_commands::save_azriel_identity,
            memory_commands::reset_azriel_identity,
            memory_commands::list_memories,
            memory_commands::search_memories,
            memory_commands::save_memory,
            memory_commands::delete_memory,
            money_commands::money_month,
            money_commands::money_save_item,
            money_commands::money_delete_item,
            money_commands::money_set_paid,
            money_commands::money_projects,
            media::media_now_playing,
            media::media_control,
            spotify::spotify_status,
            spotify::spotify_set_client_id,
            spotify::spotify_connect,
            spotify::spotify_disconnect,
            spotify::spotify_token,
            spotify::spotify_transfer,
            spotify::spotify_player,
            spotify::spotify_control,
            spotify::spotify_search,
            spotify::spotify_queue,
            spotify::spotify_tracks,
            spotify::spotify_playlists,
            spotify::spotify_play,
            voice::voice_start,
            voice::voice_send_audio,
            voice::voice_tool_response,
            voice::voice_stop,
            system_commands::system_snapshot,
            system_commands::list_processes,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao executar Azriel");
}
