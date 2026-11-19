//! Configuração do DeepSeek. A chave fica só no banco local e nunca volta para a interface.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekSettings {
    pub enabled: bool,
    pub model: String,
    pub api_key_configured: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeepSeekSettingsInput {
    pub enabled: bool,
    pub model: String,
    /// None mantém a chave; Some("") apaga.
    pub api_key: Option<String>,
}

fn err(error: rusqlite::Error) -> String { error.to_string() }

pub fn valid_model(model: &str) -> bool {
    !model.is_empty() && model.chars().count() <= 80 && model.chars().all(|c| c.is_ascii_alphanumeric() || ['-', '_', '.', ':'].contains(&c))
}

pub fn get(connection: &Connection) -> Result<DeepSeekSettings, String> {
    connection
        .query_row("SELECT enabled, model, api_key<>'', updated_at FROM deepseek_settings WHERE id=1", [], |row| {
            Ok(DeepSeekSettings { enabled: row.get(0)?, model: row.get(1)?, api_key_configured: row.get(2)?, updated_at: row.get(3)? })
        })
        .map_err(err)
}

pub fn update(connection: &Connection, input: &DeepSeekSettingsInput) -> Result<DeepSeekSettings, String> {
    let model = input.model.trim();
    if !model.is_empty() && !valid_model(model) { return Err("Modelo do DeepSeek inválido".into()); }
    if let Some(key) = &input.api_key {
        let key = key.trim();
        if key.chars().count() > 300 || key.chars().any(char::is_whitespace) { return Err("Chave de API inválida".into()); }
        connection.execute("UPDATE deepseek_settings SET api_key=?1 WHERE id=1", [key]).map_err(err)?;
    }
    let configured: bool = connection.query_row("SELECT api_key<>'' FROM deepseek_settings WHERE id=1", [], |row| row.get(0)).map_err(err)?;
    if input.enabled && (!configured || model.is_empty()) {
        return Err("Configure a chave e escolha um modelo antes de ligar o DeepSeek".into());
    }
    connection
        .execute("UPDATE deepseek_settings SET enabled=?1, model=?2, updated_at=CURRENT_TIMESTAMP WHERE id=1", params![input.enabled && configured, model])
        .map_err(err)?;
    // Um motor por vez: ligar o DeepSeek desliga o Gemini.
    if input.enabled {
        connection.execute("UPDATE ai_remote_settings SET enabled=0 WHERE id=1", []).map_err(err)?;
    }
    get(connection)
}

/// Uso exclusivo do backend para chamar o DeepSeek.
pub fn credentials(connection: &Connection) -> Result<(String, String), String> {
    let (key, model): (String, String) = connection
        .query_row("SELECT api_key, model FROM deepseek_settings WHERE id=1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(err)?;
    if key.is_empty() { return Err("Chave do DeepSeek não configurada".into()); }
    Ok((key, model))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::database::initialize(&mut connection).unwrap();
        connection
    }

    fn input(enabled: bool, model: &str, key: Option<&str>) -> DeepSeekSettingsInput {
        DeepSeekSettingsInput { enabled, model: model.into(), api_key: key.map(String::from) }
    }

    #[test]
    fn key_is_hidden_required_to_enable_and_kept_when_omitted() {
        let connection = connection();
        assert!(!get(&connection).unwrap().api_key_configured);
        assert!(update(&connection, &input(true, "deepseek-chat", None)).is_err());
        assert!(update(&connection, &input(false, "deepseek-chat", Some("sk-test"))).unwrap().api_key_configured);
        assert_eq!(credentials(&connection).unwrap(), ("sk-test".into(), "deepseek-chat".into()));
        assert!(update(&connection, &input(true, "deepseek-chat", None)).unwrap().enabled);
        assert!(update(&connection, &input(false, "x; DROP TABLE", None)).is_err());
        update(&connection, &input(false, "", Some(""))).unwrap();
        assert!(credentials(&connection).is_err());
    }

    #[test]
    fn only_one_remote_engine_is_on() {
        let connection = connection();
        connection.execute("UPDATE ai_remote_settings SET enabled=1, api_key='AIza', model='gemini-x' WHERE id=1", []).unwrap();
        update(&connection, &input(true, "deepseek-chat", Some("sk-test"))).unwrap();
        let gemini_on: bool = connection.query_row("SELECT enabled FROM ai_remote_settings WHERE id=1", [], |row| row.get(0)).unwrap();
        assert!(!gemini_on, "ligar o DeepSeek desliga o Gemini");
        crate::database::ai_repository::update_remote_settings(&connection, &crate::database::ai_models::RemoteAiSettingsInput { enabled: true, model: "gemini-x".into(), api_key: None }).unwrap();
        assert!(!get(&connection).unwrap().enabled, "ligar o Gemini desliga o DeepSeek");
    }

    #[test]
    fn the_old_openrouter_table_is_removed() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch("CREATE TABLE openrouter_settings (id INTEGER PRIMARY KEY, api_key TEXT);").unwrap();
        let mut connection = connection;
        crate::database::initialize(&mut connection).unwrap();
        let exists: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='openrouter_settings')", [], |row| row.get(0)).unwrap();
        assert!(!exists, "a tabela (e qualquer chave guardada nela) some do banco");
    }
}
