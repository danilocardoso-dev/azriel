//! Configuração do OpenRouter. A chave fica só no banco local e nunca volta para a interface.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

const MAX_FAVORITES: usize = 30;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterSettings {
    pub enabled: bool,
    pub model: String,
    /// Modelos que aparecem no seletor do AI Core (o Senhor escolhe em Configurações).
    pub favorites: Vec<String>,
    pub api_key_configured: bool,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRouterSettingsInput {
    pub enabled: bool,
    pub model: String,
    pub favorites: Vec<String>,
    /// None mantém a chave; Some("") apaga.
    pub api_key: Option<String>,
}

fn err(error: rusqlite::Error) -> String { error.to_string() }

/// Ids do OpenRouter têm o formato "empresa/modelo", às vezes com ":free".
pub fn valid_model(model: &str) -> bool {
    !model.is_empty()
        && model.chars().count() <= 120
        && model.chars().all(|c| c.is_ascii_alphanumeric() || ['-', '_', '.', ':', '/'].contains(&c))
}

pub fn get(connection: &Connection) -> Result<OpenRouterSettings, String> {
    let (enabled, model, favorites, configured, updated_at): (bool, String, String, bool, String) = connection
        .query_row("SELECT enabled, model, favorites, api_key<>'', updated_at FROM openrouter_settings WHERE id=1", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
        })
        .map_err(err)?;
    let favorites: Vec<String> = serde_json::from_str(&favorites).unwrap_or_default();
    Ok(OpenRouterSettings { enabled, model, favorites, api_key_configured: configured, updated_at })
}

pub fn update(connection: &Connection, input: &OpenRouterSettingsInput) -> Result<OpenRouterSettings, String> {
    let model = input.model.trim();
    if !model.is_empty() && !valid_model(model) { return Err("Modelo do OpenRouter inválido".into()); }
    let mut favorites: Vec<String> = Vec::new();
    for favorite in &input.favorites {
        let favorite = favorite.trim();
        if !valid_model(favorite) { return Err(format!("Modelo favorito inválido: {favorite}")); }
        if !favorites.iter().any(|item| item == favorite) { favorites.push(favorite.to_string()); }
    }
    if favorites.len() > MAX_FAVORITES { return Err(format!("No máximo {MAX_FAVORITES} modelos favoritos.")); }
    if let Some(key) = &input.api_key {
        let key = key.trim();
        if key.chars().count() > 300 || key.chars().any(char::is_whitespace) { return Err("Chave de API inválida".into()); }
        connection.execute("UPDATE openrouter_settings SET api_key=?1 WHERE id=1", [key]).map_err(err)?;
    }
    let configured: bool = connection.query_row("SELECT api_key<>'' FROM openrouter_settings WHERE id=1", [], |row| row.get(0)).map_err(err)?;
    if input.enabled && (!configured || model.is_empty()) {
        return Err("Configure a chave e escolha um modelo antes de ligar o OpenRouter".into());
    }
    let favorites_json = serde_json::to_string(&favorites).map_err(|error| error.to_string())?;
    connection
        .execute(
            "UPDATE openrouter_settings SET enabled=?1, model=?2, favorites=?3, updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![input.enabled && configured, model, favorites_json],
        )
        .map_err(err)?;
    // Um motor por vez: ligar o OpenRouter desliga o Gemini.
    if input.enabled {
        connection.execute("UPDATE ai_remote_settings SET enabled=0 WHERE id=1", []).map_err(err)?;
    }
    get(connection)
}

/// Uso exclusivo do backend para chamar o OpenRouter.
pub fn credentials(connection: &Connection) -> Result<(String, String), String> {
    let (key, model): (String, String) = connection
        .query_row("SELECT api_key, model FROM openrouter_settings WHERE id=1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(err)?;
    if key.is_empty() { return Err("Chave do OpenRouter não configurada".into()); }
    Ok((key, model))
}

pub fn api_key(connection: &Connection) -> Option<String> {
    connection.query_row("SELECT api_key FROM openrouter_settings WHERE id=1", [], |row| row.get::<_, String>(0)).ok().filter(|key| !key.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::database::initialize(&mut connection).unwrap();
        connection
    }

    fn input(enabled: bool, model: &str, favorites: &[&str], key: Option<&str>) -> OpenRouterSettingsInput {
        OpenRouterSettingsInput { enabled, model: model.into(), favorites: favorites.iter().map(|f| f.to_string()).collect(), api_key: key.map(String::from) }
    }

    #[test]
    fn key_is_hidden_required_to_enable_and_kept_when_omitted() {
        let connection = connection();
        assert!(!get(&connection).unwrap().api_key_configured);
        assert!(update(&connection, &input(true, "deepseek/deepseek-chat", &[], None)).is_err());
        let saved = update(&connection, &input(false, "deepseek/deepseek-chat", &["deepseek/deepseek-chat", "moonshotai/kimi-k2:free", "deepseek/deepseek-chat"], Some("sk-or-test"))).unwrap();
        assert!(saved.api_key_configured);
        assert_eq!(saved.favorites, vec!["deepseek/deepseek-chat", "moonshotai/kimi-k2:free"], "sem duplicatas");
        assert_eq!(credentials(&connection).unwrap(), ("sk-or-test".into(), "deepseek/deepseek-chat".into()));
        update(&connection, &input(true, "deepseek/deepseek-chat", &[], None)).unwrap();
        assert!(get(&connection).unwrap().enabled);
        assert!(update(&connection, &input(false, "x; DROP TABLE", &[], None)).is_err());
        update(&connection, &input(false, "", &[], Some(""))).unwrap();
        assert!(credentials(&connection).is_err());
    }

    #[test]
    fn only_one_remote_engine_is_on() {
        let connection = connection();
        connection.execute("UPDATE ai_remote_settings SET enabled=1, api_key='AIza', model='gemini-x' WHERE id=1", []).unwrap();
        update(&connection, &input(true, "deepseek/deepseek-chat", &[], Some("sk-or"))).unwrap();
        let gemini_on: bool = connection.query_row("SELECT enabled FROM ai_remote_settings WHERE id=1", [], |row| row.get(0)).unwrap();
        assert!(!gemini_on, "ligar o OpenRouter desliga o Gemini");
        crate::database::ai_repository::update_remote_settings(&connection, &crate::database::ai_models::RemoteAiSettingsInput { enabled: true, model: "gemini-x".into(), api_key: None }).unwrap();
        assert!(!get(&connection).unwrap().enabled, "ligar o Gemini desliga o OpenRouter");
    }
}
