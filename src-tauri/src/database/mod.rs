pub mod ai_models;
pub mod ai_repository;
pub mod daily_models;
pub mod daily_repository;
pub mod money_models;
pub mod money_repository;
pub mod trading_models;
pub mod trading_repository;

use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct DatabaseState {
    pub connection: Mutex<Connection>,
    pub path: PathBuf,
}

pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut connection = Connection::open(path).map_err(|error| error.to_string())?;
    initialize(&mut connection)?;
    Ok(connection)
}

pub fn initialize(connection: &mut Connection) -> Result<(), String> {
    connection
        .execute_batch("PRAGMA foreign_keys = OFF;")
        .map_err(|error| error.to_string())?;
    purge_retired_schema(connection)?;
    connection
        .execute_batch(include_str!("../../migrations/0048_core_reset.sql"))
        .map_err(|error| error.to_string())?;
    // Operação rápida (scalp): bancos antigos ganham as colunas novas e, uma única vez,
    // saem do stop/alvo de swing (1%/2%) para os de entradas e saídas curtas.
    if add_column(connection, "trading_settings", "max_hold_minutes", "INTEGER NOT NULL DEFAULT 30")? {
        connection
            .execute_batch("UPDATE trading_settings SET stop_loss_pct=0.3,take_profit_pct=0.5,candle_interval='5m',cycle_minutes=5 WHERE id=1;")
            .map_err(|error| error.to_string())?;
    }
    add_column(connection, "trading_settings", "profit_lock_pct", "REAL NOT NULL DEFAULT 0.25")?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// Acrescenta a coluna se ela ainda não existe. Devolve true quando acrescentou.
fn add_column(connection: &Connection, table: &str, column: &str, definition: &str) -> Result<bool, String> {
    let exists: bool = connection
        .query_row(&format!("SELECT EXISTS(SELECT 1 FROM pragma_table_info('{table}') WHERE name=?1)"), [column], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if exists {
        return Ok(false);
    }
    connection
        .execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column} {definition};"))
        .map_err(|error| error.to_string())?;
    Ok(true)
}

fn purge_retired_schema(connection: &Connection) -> Result<(), String> {
    let exact = [
        "projects", "project_knowledge", "knowledge_areas", "knowledge_history", "education",
        "registered_actions", "applications", "workspaces", "registered_urls", "action_history",
        "automation_confirmation_requests", "routine_history", "engineering_calibration",
    ];
    let mut statement = connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .map_err(|error| error.to_string())?;
    let names = statement
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    for name in names {
        let retired = exact.contains(&name.as_str())
            || ["market_", "finance_", "study_", "stark_", "routine_", "automation_"]
                .iter()
                .any(|prefix| name.starts_with(prefix));
        if retired {
            let escaped = name.replace('"', "\"\"");
            connection
                .execute_batch(&format!("DROP TABLE IF EXISTS \"{escaped}\";"))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

pub fn schema_version(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub path: String,
    pub schema_version: i64,
}
