use super::system_models::{Workspace, WorkspaceInput};
use rusqlite::{params, Connection, OptionalExtension};

pub fn list_workspaces(connection: &Connection) -> Result<Vec<Workspace>, String> {
    let mut statement = connection.prepare(
        "SELECT id,name,path,project_id,enabled,created_at,updated_at FROM workspaces ORDER BY enabled DESC,name COLLATE NOCASE"
    ).map_err(err)?;
    let rows = statement.query_map([], map_workspace).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_workspace(connection: &Connection, id: &str) -> Result<Option<Workspace>, String> {
    connection.query_row(
        "SELECT id,name,path,project_id,enabled,created_at,updated_at FROM workspaces WHERE id=?1",
        [id], map_workspace,
    ).optional().map_err(err)
}

pub fn save_workspace(
    connection: &Connection,
    input: &WorkspaceInput,
) -> Result<Workspace, String> {
    validate_id(&input.id)?;
    let name = input.name.trim();
    if name.is_empty() || name.chars().count() > 100 {
        return Err("O nome do workspace deve ter entre 1 e 100 caracteres".into());
    }
    let current = get_workspace(connection, &input.id)?;
    let normalized_path = match current.as_ref() {
        Some(workspace) if !input.enabled && workspace.path == input.path.trim() => {
            workspace.path.clone()
        }
        _ => {
            let canonical = std::fs::canonicalize(input.path.trim())
                .map_err(|_| "A pasta selecionada não existe ou não está acessível".to_string())?;
            if !canonical.is_dir() {
                return Err("O workspace precisa apontar para uma pasta".into());
            }
            normalized_canonical_path(&canonical)
        }
    };
    if let Some(project_id) = input.project_id.as_deref() {
        let exists = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM projects WHERE id=?1)",
                [project_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("O projeto relacionado não existe".into());
        }
    }
    connection.execute(
        "INSERT INTO workspaces(id,name,path,project_id,enabled) VALUES (?1,?2,?3,?4,?5)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,path=excluded.path,project_id=excluded.project_id,enabled=excluded.enabled,updated_at=CURRENT_TIMESTAMP",
        params![input.id, name, normalized_path, input.project_id, input.enabled],
    ).map_err(|error| match error {
        rusqlite::Error::SqliteFailure(_, Some(message)) if message.contains("workspaces.path") => "Esta pasta já está cadastrada".into(),
        other => other.to_string(),
    })?;
    get_workspace(connection, &input.id)?
        .ok_or_else(|| "Workspace não encontrado após salvar".into())
}

pub fn delete_workspace(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM workspaces WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

fn map_workspace(row: &rusqlite::Row<'_>) -> rusqlite::Result<Workspace> {
    Ok(Workspace {
        id: row.get(0)?,
        name: row.get(1)?,
        path: row.get(2)?,
        project_id: row.get(3)?,
        enabled: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn validate_id(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.len() <= 100
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        Ok(())
    } else {
        Err("ID de workspace inválido".into())
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

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    #[test]
    fn workspace_registry_persists_and_validates_paths() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        let path = std::env::temp_dir();
        let saved = save_workspace(
            &connection,
            &WorkspaceInput {
                id: "workspace-test".into(),
                name: "Teste".into(),
                path: path.display().to_string(),
                project_id: None,
                enabled: true,
            },
        )
        .unwrap();
        assert!(std::path::Path::new(&saved.path).is_absolute());
        assert_eq!(list_workspaces(&connection).unwrap().len(), 1);
        delete_workspace(&connection, "workspace-test").unwrap();
        assert!(list_workspaces(&connection).unwrap().is_empty());
    }

    #[test]
    fn missing_workspace_path_is_rejected() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        let result = save_workspace(
            &connection,
            &WorkspaceInput {
                id: "missing".into(),
                name: "Ausente".into(),
                path: "Z:\\azriel-path-that-does-not-exist".into(),
                project_id: None,
                enabled: true,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn windows_extended_prefix_is_not_exposed() {
        assert_eq!(
            normalized_canonical_path(std::path::Path::new(r"\\?\C:\Projetos\Azriel")),
            r"C:\Projetos\Azriel"
        );
        assert_eq!(
            normalized_canonical_path(std::path::Path::new(r"\\?\UNC\servidor\dados")),
            r"\\servidor\dados"
        );
    }
}
