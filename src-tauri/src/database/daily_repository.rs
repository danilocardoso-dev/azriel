use super::daily_models::*;
use rusqlite::{params, Connection, OptionalExtension};

pub fn list_tasks(connection: &Connection) -> Result<Vec<Task>, String> {
    query_tasks(connection, "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks ORDER BY CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END, COALESCE(due_date,'9999-12-31'),created_at DESC", [])
}

pub fn get_task(connection: &Connection, id: &str) -> Result<Option<Task>, String> {
    connection.query_row(
        "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks WHERE id=?1",
        [id], map_task,
    ).optional().map_err(err)
}

pub fn list_today_tasks(connection: &Connection, today: &str) -> Result<Vec<Task>, String> {
    validate_date(today)?;
    query_tasks(connection,
        "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks
         WHERE status NOT IN ('completed','cancelled') AND (due_date<=?1 OR (due_date IS NULL AND status IN ('pending','in_progress') AND priority IN ('high','critical')))
         ORDER BY CASE WHEN due_date<?1 THEN 0 ELSE 1 END,CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END,due_date,title", [today])
}

pub fn list_upcoming_tasks(connection: &Connection, today: &str) -> Result<Vec<Task>, String> {
    validate_date(today)?;
    query_tasks(connection,
        "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks
         WHERE status NOT IN ('completed','cancelled') AND due_date>?1 ORDER BY due_date,CASE priority WHEN 'critical' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END,title", [today])
}

pub fn list_inbox_tasks(connection: &Connection) -> Result<Vec<Task>, String> {
    query_tasks(connection, "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks WHERE status='inbox' ORDER BY created_at DESC", [])
}

pub fn list_completed_tasks(connection: &Connection) -> Result<Vec<Task>, String> {
    query_tasks(connection, "SELECT id,title,description,status,priority,due_date,project_id,knowledge_area_id,created_at,updated_at,completed_at FROM tasks WHERE status='completed' ORDER BY completed_at DESC,updated_at DESC", [])
}

pub fn save_task(connection: &Connection, input: &TaskInput) -> Result<Task, String> {
    validate_task(input)?;
    connection.execute(
        "INSERT INTO tasks(id,title,description,status,priority,due_date,project_id,knowledge_area_id,completed_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,CASE WHEN ?4='completed' THEN CURRENT_TIMESTAMP END)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title,description=excluded.description,status=excluded.status,priority=excluded.priority,due_date=excluded.due_date,project_id=excluded.project_id,knowledge_area_id=excluded.knowledge_area_id,updated_at=CURRENT_TIMESTAMP,completed_at=CASE WHEN excluded.status='completed' THEN COALESCE(tasks.completed_at,CURRENT_TIMESTAMP) ELSE NULL END",
        params![input.id,input.title.trim(),input.description.trim(),input.status,input.priority,input.due_date,input.project_id,input.knowledge_area_id],
    ).map_err(err)?;
    get_task(connection, &input.id)?.ok_or_else(|| "Tarefa não encontrada após salvar".into())
}

pub fn complete_task(connection: &Connection, id: &str) -> Result<Task, String> {
    let changed = connection.execute(
        "UPDATE tasks SET status='completed',completed_at=COALESCE(completed_at,CURRENT_TIMESTAMP),updated_at=CURRENT_TIMESTAMP WHERE id=?1",
        [id],
    ).map_err(err)?;
    if changed == 0 {
        return Err("Tarefa não encontrada".into());
    }
    get_task(connection, id)?.ok_or_else(|| "Tarefa concluída não encontrada".into())
}

pub fn delete_task(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM tasks WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn list_notes(connection: &Connection, include_archived: bool) -> Result<Vec<Note>, String> {
    let sql = if include_archived {
        "SELECT id,title,content,status,project_id,knowledge_area_id,created_at,updated_at FROM notes ORDER BY updated_at DESC"
    } else {
        "SELECT id,title,content,status,project_id,knowledge_area_id,created_at,updated_at FROM notes WHERE status='active' ORDER BY updated_at DESC"
    };
    let mut statement = connection.prepare(sql).map_err(err)?;
    let rows = statement.query_map([], map_note).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_note(connection: &Connection, id: &str) -> Result<Option<Note>, String> {
    connection.query_row("SELECT id,title,content,status,project_id,knowledge_area_id,created_at,updated_at FROM notes WHERE id=?1", [id], map_note)
        .optional().map_err(err)
}

pub fn save_note(connection: &Connection, input: &NoteInput) -> Result<Note, String> {
    validate_note(input)?;
    let title = input
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    connection.execute(
        "INSERT INTO notes(id,title,content,status,project_id,knowledge_area_id) VALUES (?1,?2,?3,?4,?5,?6)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title,content=excluded.content,status=excluded.status,project_id=excluded.project_id,knowledge_area_id=excluded.knowledge_area_id,updated_at=CURRENT_TIMESTAMP",
        params![input.id,title,input.content.trim(),input.status,input.project_id,input.knowledge_area_id],
    ).map_err(err)?;
    get_note(connection, &input.id)?.ok_or_else(|| "Nota não encontrada após salvar".into())
}

pub fn archive_note(connection: &Connection, id: &str) -> Result<Note, String> {
    let changed = connection
        .execute(
            "UPDATE notes SET status='archived',updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            [id],
        )
        .map_err(err)?;
    if changed == 0 {
        return Err("Nota não encontrada".into());
    }
    get_note(connection, id)?.ok_or_else(|| "Nota arquivada não encontrada".into())
}

pub fn delete_note(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM notes WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn counters(connection: &Connection, today: &str) -> Result<DailyCounters, String> {
    validate_date(today)?;
    connection.query_row(
        "SELECT
           (SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled')),
           (SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND (due_date=?1 OR (due_date IS NULL AND status IN ('pending','in_progress') AND priority IN ('high','critical')))),
           (SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND due_date<?1),
           (SELECT COUNT(*) FROM tasks WHERE status NOT IN ('completed','cancelled') AND priority IN ('high','critical')),
           (SELECT COUNT(*) FROM notes WHERE status='active')",
        [today],
        |row| Ok(DailyCounters { pending: row.get(0)?, today: row.get(1)?, overdue: row.get(2)?, priority: row.get(3)?, notes: row.get(4)? }),
    ).map_err(err)
}

fn query_tasks<P: rusqlite::Params>(
    connection: &Connection,
    sql: &str,
    params: P,
) -> Result<Vec<Task>, String> {
    let mut statement = connection.prepare(sql).map_err(err)?;
    let rows = statement.query_map(params, map_task).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn map_task(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: row.get(3)?,
        priority: row.get(4)?,
        due_date: row.get(5)?,
        project_id: row.get(6)?,
        knowledge_area_id: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
        completed_at: row.get(10)?,
    })
}

fn map_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        status: row.get(3)?,
        project_id: row.get(4)?,
        knowledge_area_id: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn validate_task(input: &TaskInput) -> Result<(), String> {
    validate_id(&input.id)?;
    if input.title.trim().is_empty() {
        return Err("Título da tarefa é obrigatório".into());
    }
    if !["inbox", "pending", "in_progress", "completed", "cancelled"]
        .contains(&input.status.as_str())
    {
        return Err("Status da tarefa inválido".into());
    }
    if !["low", "medium", "high", "critical"].contains(&input.priority.as_str()) {
        return Err("Prioridade inválida".into());
    }
    if let Some(date) = &input.due_date {
        validate_date(date)?;
    }
    Ok(())
}

fn validate_note(input: &NoteInput) -> Result<(), String> {
    validate_id(&input.id)?;
    if input.content.trim().is_empty() {
        return Err("Conteúdo da nota é obrigatório".into());
    }
    if !["active", "archived"].contains(&input.status.as_str()) {
        return Err("Status da nota inválido".into());
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        Ok(())
    } else {
        Err("ID inválido".into())
    }
}

fn validate_date(value: &str) -> Result<(), String> {
    let parts = value
        .split('-')
        .map(|part| part.parse::<u32>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "Data inválida; use AAAA-MM-DD".to_string())?;
    if parts.len() != 3 || value.len() != 10 {
        return Err("Data inválida; use AAAA-MM-DD".into());
    }
    let (year, month, day) = (parts[0], parts[1], parts[2]);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year >= 1900 && day >= 1 && day <= max_day {
        Ok(())
    } else {
        Err("Data inválida; use AAAA-MM-DD".into())
    }
}

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection
    }
    fn task(id: &str) -> TaskInput {
        TaskInput {
            id: id.into(),
            title: "Revisar GeneScope".into(),
            description: "Comparar variantes".into(),
            status: "inbox".into(),
            priority: "high".into(),
            due_date: Some("2026-08-31".into()),
            project_id: Some("genescope".into()),
            knowledge_area_id: Some("genetics".into()),
        }
    }

    #[test]
    fn task_crud_completion_relations_and_filters() {
        let connection = database();
        let created = save_task(&connection, &task("task-1")).unwrap();
        assert_eq!(created.project_id.as_deref(), Some("genescope"));
        assert_eq!(created.knowledge_area_id.as_deref(), Some("genetics"));
        assert_eq!(list_inbox_tasks(&connection).unwrap().len(), 1);
        assert_eq!(
            list_today_tasks(&connection, "2026-08-31").unwrap().len(),
            1
        );
        let completed = complete_task(&connection, "task-1").unwrap();
        assert_eq!(completed.status, "completed");
        assert!(completed.completed_at.is_some());
        let mut reopened = task("task-1");
        reopened.status = "in_progress".into();
        reopened.title = "Revisar lógica".into();
        let reopened = save_task(&connection, &reopened).unwrap();
        assert_eq!(reopened.title, "Revisar lógica");
        assert!(reopened.completed_at.is_none());
        delete_task(&connection, "task-1").unwrap();
        assert!(get_task(&connection, "task-1").unwrap().is_none());
    }

    #[test]
    fn upcoming_and_counters_follow_local_date() {
        let connection = database();
        let mut future = task("future");
        future.status = "pending".into();
        future.due_date = Some("2026-09-01".into());
        save_task(&connection, &future).unwrap();
        let mut overdue = task("overdue");
        overdue.status = "pending".into();
        overdue.due_date = Some("2026-08-30".into());
        save_task(&connection, &overdue).unwrap();
        let mut current = task("current");
        current.status = "in_progress".into();
        current.due_date = None;
        save_task(&connection, &current).unwrap();
        assert_eq!(
            list_upcoming_tasks(&connection, "2026-08-31")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            list_today_tasks(&connection, "2026-08-31").unwrap().len(),
            2
        );
        let counts = counters(&connection, "2026-08-31").unwrap();
        assert_eq!((counts.pending, counts.overdue, counts.priority), (3, 1, 3));
    }

    #[test]
    fn note_crud_and_archive() {
        let connection = database();
        let input = NoteInput {
            id: "note-1".into(),
            title: Some("MQTT".into()),
            content: "Investigar sensores".into(),
            status: "active".into(),
            project_id: Some("arccore".into()),
            knowledge_area_id: Some("iot".into()),
        };
        let note = save_note(&connection, &input).unwrap();
        assert_eq!(note.project_id.as_deref(), Some("arccore"));
        assert_eq!(list_notes(&connection, false).unwrap().len(), 1);
        assert_eq!(
            archive_note(&connection, "note-1").unwrap().status,
            "archived"
        );
        assert!(list_notes(&connection, false).unwrap().is_empty());
        delete_note(&connection, "note-1").unwrap();
        assert!(get_note(&connection, "note-1").unwrap().is_none());
    }

    #[test]
    fn task_and_note_survive_file_reopen() {
        let path = std::env::temp_dir().join(format!("azriel-v051-{}.db", std::process::id()));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            save_task(&connection, &task("persistent-task")).unwrap();
            save_note(
                &connection,
                &NoteInput {
                    id: "persistent-note".into(),
                    title: None,
                    content: "Nota persistente".into(),
                    status: "active".into(),
                    project_id: None,
                    knowledge_area_id: None,
                },
            )
            .unwrap();
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            assert!(get_task(&connection, "persistent-task").unwrap().is_some());
            assert!(get_note(&connection, "persistent-note").unwrap().is_some());
        }
        std::fs::remove_file(path).unwrap();
    }
}
