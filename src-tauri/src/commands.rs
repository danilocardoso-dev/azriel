use crate::database::{self, DatabaseInfo, DatabaseState};
use tauri::State;

#[tauri::command]
pub fn database_info(state: State<'_, DatabaseState>) -> Result<DatabaseInfo, String> {
    let connection = state.connection.lock().map_err(|_| "Banco indisponível".to_string())?;
    Ok(DatabaseInfo {
        path: state.path.display().to_string(),
        schema_version: database::schema_version(&connection)?,
    })
}
