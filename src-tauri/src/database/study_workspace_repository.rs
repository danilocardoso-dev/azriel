use super::study_workspace_models::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};

const NOTE_FULL_SELECT: &str =
    "SELECT note.id,note.notebook_id,notebook.title,note.title,note.content,note.content_format,
            note.roadmap_id,roadmap.name,note.stage_id,stage.name,note.topic_id,topic.name,
            note.activity_id,activity.title,note.study_session_id,note.created_at,note.updated_at
     FROM study_notes note
     JOIN study_notebooks notebook ON notebook.id=note.notebook_id
     LEFT JOIN study_roadmaps roadmap ON roadmap.id=note.roadmap_id
     LEFT JOIN roadmap_stages stage ON stage.id=note.stage_id
     LEFT JOIN roadmap_topics topic ON topic.id=note.topic_id
     LEFT JOIN roadmap_activities activity ON activity.id=note.activity_id";

const NOTE_SUMMARY_SELECT: &str = "SELECT note.id,note.notebook_id,notebook.title,note.title,
            substr(replace(replace(note.content,char(13),' '),char(10),' '),1,180),
            note.roadmap_id,roadmap.name,note.activity_id,activity.title,note.study_session_id,
            note.created_at,note.updated_at
     FROM study_notes note
     JOIN study_notebooks notebook ON notebook.id=note.notebook_id
     LEFT JOIN study_roadmaps roadmap ON roadmap.id=note.roadmap_id
     LEFT JOIN roadmap_activities activity ON activity.id=note.activity_id";

#[derive(Default)]
struct ContextIds {
    roadmap_id: Option<String>,
    stage_id: Option<String>,
    topic_id: Option<String>,
    activity_id: Option<String>,
}

pub fn list_notebooks(
    connection: &Connection,
    include_archived: bool,
) -> Result<Vec<StudyNotebook>, String> {
    let mut statement = connection
        .prepare(
            "SELECT notebook.id,notebook.title,notebook.description,notebook.status,
                    COUNT(note.id),MAX(note.updated_at),notebook.created_at,notebook.updated_at
             FROM study_notebooks notebook
             LEFT JOIN study_notes note ON note.notebook_id=notebook.id
             WHERE (?1=1 OR notebook.status='ACTIVE')
             GROUP BY notebook.id
             ORDER BY notebook.status ASC,COALESCE(MAX(note.updated_at),notebook.updated_at) DESC,notebook.title ASC",
        )
        .map_err(err)?;
    let rows = statement
        .query_map([include_archived], map_notebook)
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_notebook(connection: &Connection, id: &str) -> Result<Option<StudyNotebook>, String> {
    connection
        .query_row(
            "SELECT notebook.id,notebook.title,notebook.description,notebook.status,
                    COUNT(note.id),MAX(note.updated_at),notebook.created_at,notebook.updated_at
             FROM study_notebooks notebook
             LEFT JOIN study_notes note ON note.notebook_id=notebook.id
             WHERE notebook.id=?1 GROUP BY notebook.id",
            [id],
            map_notebook,
        )
        .optional()
        .map_err(err)
}

pub fn save_notebook(
    connection: &Connection,
    input: &StudyNotebookInput,
) -> Result<StudyNotebook, String> {
    validate_id(&input.id)?;
    let title = input.title.trim();
    if title.is_empty() || title.chars().count() > 160 {
        return Err("O título do caderno deve ter entre 1 e 160 caracteres".into());
    }
    let description = input
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if description.is_some_and(|value| value.chars().count() > 1000) {
        return Err("A descrição do caderno deve ter no máximo 1000 caracteres".into());
    }
    let timestamp = Utc::now().to_rfc3339();
    connection
        .execute(
            "INSERT INTO study_notebooks(id,title,description,status,created_at,updated_at)
             VALUES (?1,?2,?3,'ACTIVE',?4,?4)
             ON CONFLICT(id) DO UPDATE SET title=excluded.title,description=excluded.description,updated_at=excluded.updated_at",
            params![input.id, title, description, timestamp],
        )
        .map_err(err)?;
    get_notebook(connection, &input.id)?.ok_or_else(|| "Caderno não encontrado após salvar".into())
}

pub fn set_notebook_status(
    connection: &Connection,
    id: &str,
    status: &str,
) -> Result<StudyNotebook, String> {
    validate_id(id)?;
    if !["ACTIVE", "ARCHIVED"].contains(&status) {
        return Err("Status de caderno inválido".into());
    }
    let changed = connection
        .execute(
            "UPDATE study_notebooks SET status=?1,updated_at=?2 WHERE id=?3",
            params![status, Utc::now().to_rfc3339(), id],
        )
        .map_err(err)?;
    if changed == 0 {
        return Err("Caderno não encontrado".into());
    }
    get_notebook(connection, id)?.ok_or_else(|| "Caderno não encontrado após atualizar".into())
}

pub fn delete_notebook(connection: &mut Connection, id: &str) -> Result<(), String> {
    validate_id(id)?;
    let transaction = connection.transaction().map_err(err)?;
    let exists = transaction
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM study_notebooks WHERE id=?1)",
            [id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(err)?;
    if !exists {
        return Err("Caderno não encontrado".into());
    }
    transaction
        .execute("DELETE FROM study_notes WHERE notebook_id=?1", [id])
        .map_err(err)?;
    transaction
        .execute("DELETE FROM study_notebooks WHERE id=?1", [id])
        .map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn get_note(connection: &Connection, id: &str) -> Result<Option<StudyNote>, String> {
    validate_id(id)?;
    connection
        .query_row(
            &format!("{NOTE_FULL_SELECT} WHERE note.id=?1"),
            [id],
            map_note,
        )
        .optional()
        .map_err(err)
}

pub fn save_note(connection: &Connection, input: &StudyNoteInput) -> Result<StudyNote, String> {
    validate_id(&input.id)?;
    validate_id(&input.notebook_id)?;
    let title = input.title.trim();
    if title.is_empty() || title.chars().count() > 200 {
        return Err("O título da nota deve ter entre 1 e 200 caracteres".into());
    }
    if input.content.len() > 2_000_000 {
        return Err("O conteúdo da nota excede o limite de 2 MB".into());
    }
    let notebook_status = connection
        .query_row(
            "SELECT status FROM study_notebooks WHERE id=?1",
            [&input.notebook_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(err)?
        .ok_or_else(|| "Caderno não encontrado".to_string())?;
    if notebook_status != "ACTIVE" {
        return Err("Restaure o caderno antes de editar suas notas".into());
    }
    let context = resolve_context(connection, input)?;
    let timestamp = Utc::now().to_rfc3339();
    connection
        .execute(
            "INSERT INTO study_notes(
               id,notebook_id,title,content,content_format,roadmap_id,stage_id,topic_id,activity_id,study_session_id,created_at,updated_at
             ) VALUES (?1,?2,?3,?4,'MARKDOWN',?5,?6,?7,?8,?9,?10,?10)
             ON CONFLICT(id) DO UPDATE SET
               notebook_id=excluded.notebook_id,title=excluded.title,content=excluded.content,
               roadmap_id=excluded.roadmap_id,stage_id=excluded.stage_id,topic_id=excluded.topic_id,
               activity_id=excluded.activity_id,study_session_id=excluded.study_session_id,updated_at=excluded.updated_at",
            params![
                input.id,
                input.notebook_id,
                title,
                input.content,
                context.roadmap_id,
                context.stage_id,
                context.topic_id,
                context.activity_id,
                input.study_session_id,
                timestamp
            ],
        )
        .map_err(err)?;
    get_note(connection, &input.id)?.ok_or_else(|| "Nota não encontrada após salvar".into())
}

pub fn delete_note(connection: &Connection, id: &str) -> Result<(), String> {
    validate_id(id)?;
    let changed = connection
        .execute("DELETE FROM study_notes WHERE id=?1", [id])
        .map_err(err)?;
    if changed == 0 {
        return Err("Nota não encontrada".into());
    }
    Ok(())
}

pub fn list_notes(
    connection: &Connection,
    input: &StudyNoteListInput,
) -> Result<Vec<StudyNoteSummary>, String> {
    validate_optional_id(input.notebook_id.as_deref())?;
    validate_optional_id(input.activity_id.as_deref())?;
    validate_optional_id(input.study_session_id.as_deref())?;
    let limit = input.limit.clamp(1, 100);
    let offset = input.offset.max(0);
    let mut statement = connection
        .prepare(&format!(
            "{NOTE_SUMMARY_SELECT}
             WHERE (?1 IS NULL OR note.notebook_id=?1)
               AND (?2 IS NULL OR note.activity_id=?2)
               AND (?3 IS NULL OR note.study_session_id=?3)
             ORDER BY note.updated_at DESC,note.id DESC LIMIT ?4 OFFSET ?5"
        ))
        .map_err(err)?;
    let rows = statement
        .query_map(
            params![
                input.notebook_id,
                input.activity_id,
                input.study_session_id,
                limit,
                offset
            ],
            map_note_summary,
        )
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn search_notes(
    connection: &Connection,
    input: &StudyNoteSearchInput,
) -> Result<Vec<StudyNoteSummary>, String> {
    let query = input.query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }
    if query.chars().count() > 200 {
        return Err("A busca deve ter no máximo 200 caracteres".into());
    }
    let pattern = like_pattern(query);
    let limit = input.limit.clamp(1, 50);
    let mut statement = connection
        .prepare(&format!(
            "{NOTE_SUMMARY_SELECT}
             WHERE note.title LIKE ?1 ESCAPE '\\' COLLATE NOCASE
                OR note.content LIKE ?1 ESCAPE '\\' COLLATE NOCASE
                OR notebook.title LIKE ?1 ESCAPE '\\' COLLATE NOCASE
             ORDER BY note.updated_at DESC,note.id DESC LIMIT ?2"
        ))
        .map_err(err)?;
    let rows = statement
        .query_map(params![pattern, limit], map_note_summary)
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn list_recent_notes(
    connection: &Connection,
    limit: i64,
) -> Result<Vec<StudyNoteSummary>, String> {
    list_notes(
        connection,
        &StudyNoteListInput {
            notebook_id: None,
            activity_id: None,
            study_session_id: None,
            limit: limit.clamp(1, 20),
            offset: 0,
        },
    )
}

fn resolve_context(connection: &Connection, input: &StudyNoteInput) -> Result<ContextIds, String> {
    validate_optional_id(input.roadmap_id.as_deref())?;
    validate_optional_id(input.stage_id.as_deref())?;
    validate_optional_id(input.topic_id.as_deref())?;
    validate_optional_id(input.activity_id.as_deref())?;
    validate_optional_id(input.study_session_id.as_deref())?;
    let mut context = ContextIds {
        roadmap_id: input.roadmap_id.clone(),
        stage_id: input.stage_id.clone(),
        topic_id: input.topic_id.clone(),
        activity_id: input.activity_id.clone(),
    };

    if let Some(session_id) = input.study_session_id.as_deref() {
        let session = connection
            .query_row(
                "SELECT roadmap_id,stage_id,topic_id,activity_id FROM study_sessions WHERE id=?1",
                [session_id],
                |row| {
                    Ok(ContextIds {
                        roadmap_id: row.get(0)?,
                        stage_id: row.get(1)?,
                        topic_id: row.get(2)?,
                        activity_id: row.get(3)?,
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Sessão de estudo não encontrada".to_string())?;
        merge_context(&mut context, session, "sessão")?;
    }

    if let Some(activity_id) = context.activity_id.clone() {
        let activity = connection
            .query_row(
                "SELECT roadmap.id,stage.id,topic.id,activity.id
                 FROM roadmap_activities activity
                 JOIN roadmap_topics topic ON topic.id=activity.topic_id
                 JOIN roadmap_stages stage ON stage.id=topic.stage_id
                 JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id
                 WHERE activity.id=?1",
                [activity_id],
                |row| {
                    Ok(ContextIds {
                        roadmap_id: Some(row.get(0)?),
                        stage_id: Some(row.get(1)?),
                        topic_id: Some(row.get(2)?),
                        activity_id: Some(row.get(3)?),
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Atividade vinculada não encontrada".to_string())?;
        merge_context(&mut context, activity, "atividade")?;
    } else if let Some(topic_id) = context.topic_id.clone() {
        let topic = connection
            .query_row(
                "SELECT roadmap.id,stage.id,topic.id
                 FROM roadmap_topics topic
                 JOIN roadmap_stages stage ON stage.id=topic.stage_id
                 JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id
                 WHERE topic.id=?1",
                [topic_id],
                |row| {
                    Ok(ContextIds {
                        roadmap_id: Some(row.get(0)?),
                        stage_id: Some(row.get(1)?),
                        topic_id: Some(row.get(2)?),
                        activity_id: None,
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Tópico vinculado não encontrado".to_string())?;
        merge_context(&mut context, topic, "tópico")?;
    } else if let Some(stage_id) = context.stage_id.clone() {
        let stage = connection
            .query_row(
                "SELECT roadmap.id,stage.id
                 FROM roadmap_stages stage
                 JOIN study_roadmaps roadmap ON roadmap.id=stage.roadmap_id
                 WHERE stage.id=?1",
                [stage_id],
                |row| {
                    Ok(ContextIds {
                        roadmap_id: Some(row.get(0)?),
                        stage_id: Some(row.get(1)?),
                        topic_id: None,
                        activity_id: None,
                    })
                },
            )
            .optional()
            .map_err(err)?
            .ok_or_else(|| "Etapa vinculada não encontrada".to_string())?;
        merge_context(&mut context, stage, "etapa")?;
    } else if let Some(roadmap_id) = context.roadmap_id.as_deref() {
        let exists = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM study_roadmaps WHERE id=?1)",
                [roadmap_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(err)?;
        if !exists {
            return Err("Roadmap vinculado não encontrado".into());
        }
    }
    Ok(context)
}

fn merge_context(current: &mut ContextIds, actual: ContextIds, source: &str) -> Result<(), String> {
    merge_id(
        &mut current.roadmap_id,
        actual.roadmap_id,
        "roadmap",
        source,
    )?;
    merge_id(&mut current.stage_id, actual.stage_id, "etapa", source)?;
    merge_id(&mut current.topic_id, actual.topic_id, "tópico", source)?;
    merge_id(
        &mut current.activity_id,
        actual.activity_id,
        "atividade",
        source,
    )
}

fn merge_id(
    current: &mut Option<String>,
    actual: Option<String>,
    label: &str,
    source: &str,
) -> Result<(), String> {
    if let (Some(expected), Some(value)) = (current.as_deref(), actual.as_deref()) {
        if expected != value {
            return Err(format!(
                "O {label} informado não pertence à {source} vinculada"
            ));
        }
    }
    if current.is_none() {
        *current = actual;
    }
    Ok(())
}

fn map_notebook(row: &Row<'_>) -> rusqlite::Result<StudyNotebook> {
    Ok(StudyNotebook {
        id: row.get(0)?,
        title: row.get(1)?,
        description: row.get(2)?,
        status: row.get(3)?,
        note_count: row.get(4)?,
        last_note_updated_at: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_note(row: &Row<'_>) -> rusqlite::Result<StudyNote> {
    Ok(StudyNote {
        id: row.get(0)?,
        notebook_id: row.get(1)?,
        notebook_title: row.get(2)?,
        title: row.get(3)?,
        content: row.get(4)?,
        content_format: row.get(5)?,
        roadmap_id: row.get(6)?,
        roadmap_name: row.get(7)?,
        stage_id: row.get(8)?,
        stage_name: row.get(9)?,
        topic_id: row.get(10)?,
        topic_name: row.get(11)?,
        activity_id: row.get(12)?,
        activity_title: row.get(13)?,
        study_session_id: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

fn map_note_summary(row: &Row<'_>) -> rusqlite::Result<StudyNoteSummary> {
    Ok(StudyNoteSummary {
        id: row.get(0)?,
        notebook_id: row.get(1)?,
        notebook_title: row.get(2)?,
        title: row.get(3)?,
        preview: row.get(4)?,
        roadmap_id: row.get(5)?,
        roadmap_name: row.get(6)?,
        activity_id: row.get(7)?,
        activity_title: row.get(8)?,
        study_session_id: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn validate_optional_id(value: Option<&str>) -> Result<(), String> {
    if let Some(value) = value {
        validate_id(value)?;
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

fn like_pattern(value: &str) -> String {
    format!(
        "%{}%",
        value
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn err(error: impl std::fmt::Display) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{self, study_models::StartStudySessionInput, study_repository};

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('roadmap-notes','Roadmap Notes','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('stage-notes','roadmap-notes','Fundamentos',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,topic_order) VALUES ('topic-notes','stage-notes','Conceitos',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('activity-notes','topic-notes','Atividade prática','EXERCISE','pending',1)", []).unwrap();
        connection
    }

    fn notebook(id: &str, title: &str) -> StudyNotebookInput {
        StudyNotebookInput {
            id: id.into(),
            title: title.into(),
            description: Some("Descrição".into()),
        }
    }

    fn note(id: &str, notebook_id: &str) -> StudyNoteInput {
        StudyNoteInput {
            id: id.into(),
            notebook_id: notebook_id.into(),
            title: "Lei de Ohm".into(),
            content: "# Tensão\n\n```rust\nlet voltage = 12;\n```".into(),
            roadmap_id: Some("roadmap-notes".into()),
            stage_id: Some("stage-notes".into()),
            topic_id: Some("topic-notes".into()),
            activity_id: Some("activity-notes".into()),
            study_session_id: None,
        }
    }

    #[test]
    fn notebook_create_update_archive_restore_and_note_count_are_persistent() {
        let connection = database();
        save_notebook(&connection, &notebook("electronics", "Eletrônica")).unwrap();
        save_note(&connection, &note("ohm", "electronics")).unwrap();
        let updated =
            save_notebook(&connection, &notebook("electronics", "Eletrônica Aplicada")).unwrap();
        assert_eq!(updated.note_count, 1);
        assert_eq!(updated.title, "Eletrônica Aplicada");
        assert_eq!(
            set_notebook_status(&connection, "electronics", "ARCHIVED")
                .unwrap()
                .status,
            "ARCHIVED"
        );
        assert!(list_notebooks(&connection, false).unwrap().is_empty());
        assert_eq!(list_notebooks(&connection, true).unwrap()[0].note_count, 1);
        assert_eq!(
            set_notebook_status(&connection, "electronics", "ACTIVE")
                .unwrap()
                .status,
            "ACTIVE"
        );
    }

    #[test]
    fn notebook_delete_removes_its_notes_and_is_atomic() {
        let mut connection = database();
        save_notebook(&connection, &notebook("empty", "Vazio")).unwrap();
        delete_notebook(&mut connection, "empty").unwrap();
        assert!(get_notebook(&connection, "empty").unwrap().is_none());

        save_notebook(&connection, &notebook("electronics", "Eletrônica")).unwrap();
        save_note(&connection, &note("ohm", "electronics")).unwrap();
        delete_notebook(&mut connection, "electronics").unwrap();
        assert!(get_notebook(&connection, "electronics").unwrap().is_none());
        assert!(get_note(&connection, "ohm").unwrap().is_none());
        assert!(delete_notebook(&mut connection, "missing")
            .unwrap_err()
            .contains("não encontrado"));

        save_notebook(&connection, &notebook("atomic", "Atômico")).unwrap();
        save_note(&connection, &note("atomic-note", "atomic")).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER prevent_notebook_delete BEFORE DELETE ON study_notebooks
             WHEN OLD.id='atomic' BEGIN SELECT RAISE(ABORT,'forced failure'); END;",
            )
            .unwrap();
        assert!(delete_notebook(&mut connection, "atomic").is_err());
        assert!(get_notebook(&connection, "atomic").unwrap().is_some());
        assert!(get_note(&connection, "atomic-note").unwrap().is_some());
    }

    #[test]
    fn note_relations_search_recent_update_and_delete_are_validated() {
        let connection = database();
        save_notebook(&connection, &notebook("electronics", "Eletrônica")).unwrap();
        let saved = save_note(&connection, &note("ohm", "electronics")).unwrap();
        assert_eq!(saved.activity_id.as_deref(), Some("activity-notes"));
        assert_eq!(
            search_notes(
                &connection,
                &StudyNoteSearchInput {
                    query: "Tensão".into(),
                    limit: 10
                }
            )
            .unwrap()
            .len(),
            1
        );
        let mut updated = note("ohm", "electronics");
        updated.title = "Circuitos DC".into();
        updated.content = "Conteúdo atualizado".into();
        assert_eq!(
            save_note(&connection, &updated).unwrap().title,
            "Circuitos DC"
        );
        assert_eq!(list_recent_notes(&connection, 5).unwrap()[0].id, "ohm");
        delete_note(&connection, "ohm").unwrap();
        assert!(get_note(&connection, "ohm").unwrap().is_none());
    }

    #[test]
    fn note_inherits_session_context_and_rejects_conflicting_or_missing_foreign_ids() {
        let mut connection = database();
        save_notebook(&connection, &notebook("electronics", "Eletrônica")).unwrap();
        study_repository::start_session(
            &mut connection,
            &StartStudySessionInput {
                id: "session-notes".into(),
                roadmap_id: Some("roadmap-notes".into()),
                activity_id: Some("activity-notes".into()),
                planned_focus_minutes: 25,
            },
        )
        .unwrap();
        let mut linked = note("session-note", "electronics");
        linked.roadmap_id = None;
        linked.stage_id = None;
        linked.topic_id = None;
        linked.activity_id = None;
        linked.study_session_id = Some("session-notes".into());
        let saved = save_note(&connection, &linked).unwrap();
        assert_eq!(saved.roadmap_id.as_deref(), Some("roadmap-notes"));
        assert_eq!(saved.activity_id.as_deref(), Some("activity-notes"));
        let mut conflict = linked;
        conflict.id = "conflict".into();
        conflict.activity_id = Some("missing".into());
        assert!(save_note(&connection, &conflict)
            .unwrap_err()
            .contains("não pertence"));
        let mut missing_notebook = note("missing-notebook", "absent");
        missing_notebook.roadmap_id = None;
        missing_notebook.stage_id = None;
        missing_notebook.topic_id = None;
        missing_notebook.activity_id = None;
        assert!(save_note(&connection, &missing_notebook)
            .unwrap_err()
            .contains("Caderno"));
    }
}
