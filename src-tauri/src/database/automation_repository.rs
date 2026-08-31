use super::automation_models::{
    ActionHistory, Application, ApplicationInput, RegisteredUrl, RegisteredUrlInput,
};
use reqwest::Url;
use rusqlite::{params, Connection, OptionalExtension};

pub fn list_applications(connection: &Connection) -> Result<Vec<Application>, String> {
    let mut statement = connection.prepare("SELECT id,name,path,enabled,created_at,updated_at FROM applications ORDER BY enabled DESC,name COLLATE NOCASE").map_err(err)?;
    let rows = statement.query_map([], map_application).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_application(connection: &Connection, id: &str) -> Result<Option<Application>, String> {
    connection
        .query_row(
            "SELECT id,name,path,enabled,created_at,updated_at FROM applications WHERE id=?1",
            [id],
            map_application,
        )
        .optional()
        .map_err(err)
}

pub fn save_application(
    connection: &Connection,
    input: &ApplicationInput,
) -> Result<Application, String> {
    validate_id(&input.id, "aplicativo")?;
    let name = required_name(&input.name, "aplicativo")?;
    let canonical = std::fs::canonicalize(input.path.trim())
        .map_err(|_| "O executável selecionado não existe ou não está acessível".to_string())?;
    if !canonical.is_file() {
        return Err("O aplicativo precisa apontar para um arquivo executável".into());
    }
    if canonical
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("exe"))
        != Some(true)
    {
        return Err("O aplicativo autorizado precisa ser um executável .exe".into());
    }
    let path = normalized_canonical_path(&canonical);
    connection.execute(
        "INSERT INTO applications(id,name,path,enabled) VALUES (?1,?2,?3,?4)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,path=excluded.path,enabled=excluded.enabled,updated_at=CURRENT_TIMESTAMP",
        params![input.id, name, path, input.enabled],
    ).map_err(|error| unique_error(error, "Este executável já está cadastrado"))?;
    get_application(connection, &input.id)?
        .ok_or_else(|| "Aplicativo não encontrado após salvar".into())
}

pub fn delete_application(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM applications WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn list_urls(connection: &Connection) -> Result<Vec<RegisteredUrl>, String> {
    let mut statement = connection.prepare("SELECT id,name,url,enabled,created_at,updated_at FROM registered_urls ORDER BY enabled DESC,name COLLATE NOCASE").map_err(err)?;
    let rows = statement.query_map([], map_url).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_url(connection: &Connection, id: &str) -> Result<Option<RegisteredUrl>, String> {
    connection
        .query_row(
            "SELECT id,name,url,enabled,created_at,updated_at FROM registered_urls WHERE id=?1",
            [id],
            map_url,
        )
        .optional()
        .map_err(err)
}

pub fn save_url(
    connection: &Connection,
    input: &RegisteredUrlInput,
) -> Result<RegisteredUrl, String> {
    validate_id(&input.id, "URL")?;
    let name = required_name(&input.name, "URL")?;
    let parsed = Url::parse(input.url.trim()).map_err(|_| "Informe uma URL válida".to_string())?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err("Somente URLs HTTP ou HTTPS com host são permitidas".into());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs com credenciais embutidas não são permitidas".into());
    }
    let url = parsed.to_string();
    connection.execute(
        "INSERT INTO registered_urls(id,name,url,enabled) VALUES (?1,?2,?3,?4)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,url=excluded.url,enabled=excluded.enabled,updated_at=CURRENT_TIMESTAMP",
        params![input.id, name, url, input.enabled],
    ).map_err(|error| unique_error(error, "Esta URL já está cadastrada"))?;
    get_url(connection, &input.id)?.ok_or_else(|| "URL não encontrada após salvar".into())
}

pub fn delete_url(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM registered_urls WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn start_history(
    connection: &Connection,
    action_id: &str,
    source: &str,
    target_id: Option<&str>,
    permission: &str,
    confirmation_required: bool,
) -> Result<i64, String> {
    connection.execute(
        "INSERT INTO action_history(action_id,source,target_id,permission,confirmation_required) VALUES (?1,?2,?3,?4,?5)",
        params![action_id, source, target_id, permission, confirmation_required],
    ).map_err(err)?;
    Ok(connection.last_insert_rowid())
}

pub fn complete_history(
    connection: &Connection,
    id: i64,
    target_type: Option<&str>,
    target_name: Option<&str>,
    confirmed: bool,
    success: bool,
    error_message: Option<&str>,
) -> Result<(), String> {
    connection.execute(
        "UPDATE action_history SET target_type=?1,target_name=?2,confirmed=?3,success=?4,error=?5,completed_at=CURRENT_TIMESTAMP WHERE id=?6",
        params![target_type, target_name, confirmed, success, error_message, id],
    ).map_err(err)?;
    Ok(())
}

pub fn list_history(connection: &Connection, limit: i64) -> Result<Vec<ActionHistory>, String> {
    let safe_limit = limit.clamp(1, 500);
    let mut statement = connection.prepare(
        "SELECT id,action_id,source,target_type,target_id,target_name,permission,confirmation_required,confirmed,success,error,created_at,completed_at FROM action_history ORDER BY id DESC LIMIT ?1"
    ).map_err(err)?;
    let rows = statement
        .query_map([safe_limit], |row| {
            Ok(ActionHistory {
                id: row.get(0)?,
                action_id: row.get(1)?,
                source: row.get(2)?,
                target_type: row.get(3)?,
                target_id: row.get(4)?,
                target_name: row.get(5)?,
                permission: row.get(6)?,
                confirmation_required: row.get(7)?,
                confirmed: row.get(8)?,
                success: row.get(9)?,
                error: row.get(10)?,
                created_at: row.get(11)?,
                completed_at: row.get(12)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn map_application(row: &rusqlite::Row<'_>) -> rusqlite::Result<Application> {
    Ok(Application {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        enabled: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn map_url(row: &rusqlite::Row<'_>) -> rusqlite::Result<RegisteredUrl> {
    Ok(RegisteredUrl {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        enabled: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn required_name<'a>(value: &'a str, kind: &str) -> Result<&'a str, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 120 {
        Err(format!(
            "O nome do {kind} deve ter entre 1 e 120 caracteres"
        ))
    } else {
        Ok(trimmed)
    }
}

pub fn validate_id(value: &str, kind: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.len() <= 100
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err(format!("ID de {kind} inválido"))
    }
}

fn normalized_canonical_path(path: &std::path::Path) -> String {
    let value = path.to_string_lossy();
    if let Some(network_path) = value.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{network_path}")
    } else {
        value.strip_prefix(r"\\?\").unwrap_or(&value).to_string()
    }
}

fn unique_error(error: rusqlite::Error, message: &str) -> String {
    match error {
        rusqlite::Error::SqliteFailure(_, Some(detail))
            if detail.contains("UNIQUE constraint failed") =>
        {
            message.into()
        }
        other => other.to_string(),
    }
}
fn err(error: rusqlite::Error) -> String {
    error.to_string()
}
