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

/// Só endereços web completos: nada de file:, javascript:, protocolos de app ou caminhos locais.
pub fn is_web_url(url: &str) -> bool {
    let url = url.trim();
    (url.starts_with("https://") || url.starts_with("http://"))
        && url.len() <= 2_048
        && url.split("://").nth(1).is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/'))
        && !url.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// Abre um link das respostas do Azriel no navegador padrão, fora da janela do app.
#[tauri::command]
pub fn open_external(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if !is_web_url(&url) {
        return Err("Só é possível abrir links da web (http ou https).".into());
    }
    app.opener().open_url(url.trim(), None::<&str>).map_err(|_| "Não foi possível abrir o navegador".to_string())
}

#[cfg(test)]
mod tests {
    use super::is_web_url;

    #[test]
    fn only_web_links_are_opened() {
        assert!(is_web_url("https://www.reuters.com/world/"));
        assert!(is_web_url("http://example.com"));
        assert!(!is_web_url("file:///C:/Windows/system32"));
        assert!(!is_web_url("javascript:alert(1)"));
        assert!(!is_web_url("spotify:track:1"));
        assert!(!is_web_url("https://"));
        assert!(!is_web_url("https:///etc/passwd"));
        assert!(!is_web_url("https://exemplo.com/a b"));
    }
}
