use super::{ai_models::*, daily_models::Task, daily_repository};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;

pub fn get_settings(connection: &Connection) -> Result<AiSettings, String> {
    connection.query_row(
        "SELECT provider,endpoint,model,context_message_limit,timeout_seconds,updated_at FROM ai_settings WHERE id=1",
        [],
        |row| Ok(AiSettings { provider: row.get(0)?, endpoint: row.get(1)?, model: row.get(2)?, context_message_limit: row.get(3)?, timeout_seconds: row.get(4)?, updated_at: row.get(5)? }),
    ).map_err(err)
}

pub fn update_settings(
    connection: &Connection,
    input: &AiSettingsInput,
) -> Result<AiSettings, String> {
    validate_settings(input)?;
    connection.execute(
        "UPDATE ai_settings SET endpoint=?1,model=?2,context_message_limit=?3,timeout_seconds=?4,updated_at=CURRENT_TIMESTAMP WHERE id=1",
        params![input.endpoint.trim().trim_end_matches('/'), input.model.trim(), input.context_message_limit, input.timeout_seconds],
    ).map_err(err)?;
    get_settings(connection)
}

pub fn list_conversations(connection: &Connection) -> Result<Vec<Conversation>, String> {
    let mut statement = connection
        .prepare(
            "SELECT id,title,created_at,updated_at FROM conversations ORDER BY updated_at DESC,id",
        )
        .map_err(err)?;
    let rows = statement.query_map([], map_conversation).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn get_conversation(connection: &Connection, id: &str) -> Result<Option<Conversation>, String> {
    connection
        .query_row(
            "SELECT id,title,created_at,updated_at FROM conversations WHERE id=?1",
            [id],
            map_conversation,
        )
        .optional()
        .map_err(err)
}

pub fn create_conversation(
    connection: &Connection,
    input: &ConversationInput,
) -> Result<Conversation, String> {
    validate_id(&input.id)?;
    let title = input.title.trim();
    if title.is_empty() {
        return Err("O título da conversa é obrigatório".into());
    }
    if title.chars().count() > 120 {
        return Err("O título da conversa excede 120 caracteres".into());
    }
    connection
        .execute(
            "INSERT INTO conversations(id,title) VALUES (?1,?2)",
            params![input.id, title],
        )
        .map_err(err)?;
    get_conversation(connection, &input.id)?
        .ok_or_else(|| "Conversa não encontrada apó criar".into())
}

pub fn delete_conversation(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM conversations WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn clear_conversation_messages(
    connection: &mut Connection,
    conversation_id: &str,
) -> Result<usize, String> {
    validate_id(conversation_id)?;
    let transaction = connection.transaction().map_err(err)?;
    let exists = transaction
        .query_row(
            "SELECT 1 FROM conversations WHERE id=?1",
            [conversation_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(err)?
        .is_some();
    if !exists {
        return Err("Conversa não encontrada".into());
    }
    let removed = transaction
        .execute(
            "DELETE FROM messages WHERE conversation_id=?1",
            [conversation_id],
        )
        .map_err(err)?;
    transaction
        .execute(
            "UPDATE conversations SET updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            [conversation_id],
        )
        .map_err(err)?;
    transaction.commit().map_err(err)?;
    Ok(removed)
}

pub fn list_messages(
    connection: &Connection,
    conversation_id: &str,
) -> Result<Vec<Message>, String> {
    let mut statement = connection.prepare(
        "SELECT id,conversation_id,role,content,created_at FROM messages WHERE conversation_id=?1 ORDER BY rowid"
    ).map_err(err)?;
    let rows = statement
        .query_map([conversation_id], map_message)
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn add_message(connection: &mut Connection, input: &MessageInput) -> Result<Message, String> {
    validate_id(&input.id)?;
    validate_id(&input.conversation_id)?;
    if !["user", "assistant", "system"].contains(&input.role.as_str()) {
        return Err("Papel de mensagem inválido".into());
    }
    let content = input.content.trim();
    if content.is_empty() {
        return Err("O conteúdo da mensagem é obrigatório".into());
    }
    if content.chars().count() > 50_000 {
        return Err("A mensagem excede o limite permitido".into());
    }
    let transaction = connection.transaction().map_err(err)?;
    transaction
        .execute(
            "INSERT INTO messages(id,conversation_id,role,content) VALUES (?1,?2,?3,?4)",
            params![input.id, input.conversation_id, input.role, content],
        )
        .map_err(err)?;
    transaction
        .execute(
            "UPDATE conversations SET updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            [&input.conversation_id],
        )
        .map_err(err)?;
    transaction.commit().map_err(err)?;
    connection
        .query_row(
            "SELECT id,conversation_id,role,content,created_at FROM messages WHERE id=?1",
            [&input.id],
            map_message,
        )
        .map_err(err)
}

pub fn save_task_references(
    connection: &mut Connection,
    input: &SaveTaskReferencesInput,
) -> Result<usize, String> {
    validate_id(&input.conversation_id)?;
    validate_id(&input.source_message_id)?;
    if input.task_ids.is_empty() || input.task_ids.len() > 50 {
        return Err("A lista de referências deve possuir entre 1 e 50 tarefas".into());
    }
    let unique = input.task_ids.iter().collect::<HashSet<_>>();
    if unique.len() != input.task_ids.len() {
        return Err("A lista de referências contém tarefas duplicadas".into());
    }
    let transaction = connection.transaction().map_err(err)?;
    let message = transaction
        .query_row(
            "SELECT conversation_id,role FROM messages WHERE id=?1",
            [&input.source_message_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(err)?
        .ok_or_else(|| "Mensagem de referência não encontrada".to_string())?;
    if message.0 != input.conversation_id || message.1 != "assistant" {
        return Err("A mensagem não pode registrar referências desta conversa".into());
    }
    transaction
        .execute(
            "DELETE FROM ai_entity_references WHERE source_message_id=?1",
            [&input.source_message_id],
        )
        .map_err(err)?;
    for (index, task_id) in input.task_ids.iter().enumerate() {
        validate_id(task_id)?;
        let title = transaction
            .query_row("SELECT title FROM tasks WHERE id=?1", [task_id], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(err)?
            .ok_or_else(|| format!("Tarefa de referência não encontrada: {task_id}"))?;
        transaction
            .execute(
                "INSERT INTO ai_entity_references(conversation_id,source_message_id,position,entity_type,entity_id,entity_label) VALUES(?1,?2,?3,'task',?4,?5)",
                params![input.conversation_id, input.source_message_id, index as i64 + 1, task_id, title],
            )
            .map_err(err)?;
    }
    transaction.commit().map_err(err)?;
    Ok(input.task_ids.len())
}

pub fn complete_referenced_task(
    connection: &mut Connection,
    conversation_id: &str,
    position: i64,
) -> Result<Task, String> {
    validate_id(conversation_id)?;
    if !(1..=50).contains(&position) {
        return Err("A posição informada não existe na lista".into());
    }
    let transaction = connection.transaction().map_err(err)?;
    let task_id = transaction
        .query_row(
            "SELECT reference.entity_id
             FROM ai_entity_references reference
             JOIN messages message ON message.id=reference.source_message_id
             WHERE reference.conversation_id=?1 AND reference.entity_type='task' AND reference.position=?2
             ORDER BY message.rowid DESC LIMIT 1",
            params![conversation_id, position],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(err)?
        .ok_or_else(|| "Não encontrei uma lista recente com essa posição nesta conversa".to_string())?;
    let current = daily_repository::get_task(&transaction, &task_id)?
        .ok_or_else(|| "A tarefa referenciada não existe mais".to_string())?;
    if current.status == "completed" {
        return Err(format!("A demanda '{}' já está concluída", current.title));
    }
    if current.status == "cancelled" {
        return Err(format!(
            "A demanda '{}' está cancelada e não pode ser concluída",
            current.title
        ));
    }
    transaction
        .execute(
            "UPDATE tasks SET status='completed',completed_at=COALESCE(completed_at,CURRENT_TIMESTAMP),updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND status NOT IN ('completed','cancelled')",
            [&task_id],
        )
        .map_err(err)?;
    transaction
        .execute(
            "INSERT INTO task_action_history(task_id,task_title,action,source,conversation_id,previous_status,new_status) VALUES(?1,?2,'completed','ai',?3,?4,'completed')",
            params![task_id, current.title, conversation_id, current.status],
        )
        .map_err(err)?;
    let completed = daily_repository::get_task(&transaction, &task_id)?
        .ok_or_else(|| "A tarefa concluída não foi encontrada".to_string())?;
    transaction.commit().map_err(err)?;
    Ok(completed)
}

fn validate_settings(input: &AiSettingsInput) -> Result<(), String> {
    if input.endpoint.trim().is_empty() {
        return Err("Endpoint do Ollama é obrigatório".into());
    }
    if input.model.trim().is_empty() || input.model.chars().count() > 120 {
        return Err("Modelo do Ollama inválido".into());
    }
    if !(1..=20).contains(&input.context_message_limit) {
        return Err("O limite de contexto deve estar entre 1 e 20 mensagens".into());
    }
    if !(5..=180).contains(&input.timeout_seconds) {
        return Err("O timeout deve estar entre 5 e 180 segundos".into());
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

fn map_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        title: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}

fn map_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: row.get(0)?,
        conversation_id: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        created_at: row.get(4)?,
    })
}

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    #[test]
    fn settings_and_conversations_are_persistent() {
        let path = std::env::temp_dir().join(format!("azriel-v060-{}.db", std::process::id()));
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            let settings = update_settings(
                &connection,
                &AiSettingsInput {
                    endpoint: "http://localhost:11434/".into(),
                    model: "qwen2.5:0.5b".into(),
                    context_message_limit: 8,
                    timeout_seconds: 30,
                },
            )
            .unwrap();
            assert_eq!(settings.context_message_limit, 8);
            create_conversation(
                &connection,
                &ConversationInput {
                    id: "conversation-1".into(),
                    title: "Situação".into(),
                },
            )
            .unwrap();
            add_message(
                &mut connection,
                &MessageInput {
                    id: "message-1".into(),
                    conversation_id: "conversation-1".into(),
                    role: "user".into(),
                    content: "Azriel, situação".into(),
                },
            )
            .unwrap();
            add_message(
                &mut connection,
                &MessageInput {
                    id: "aaa-message-2".into(),
                    conversation_id: "conversation-1".into(),
                    role: "assistant".into(),
                    content: "Situação recuperada".into(),
                },
            )
            .unwrap();
        }
        {
            let mut connection = Connection::open(&path).unwrap();
            database::initialize(&mut connection).unwrap();
            assert_eq!(get_settings(&connection).unwrap().context_message_limit, 8);
            assert_eq!(list_conversations(&connection).unwrap().len(), 1);
            let messages = list_messages(&connection, "conversation-1").unwrap();
            assert_eq!(messages.len(), 2);
            assert_eq!(
                messages
                    .iter()
                    .map(|message| message.role.as_str())
                    .collect::<Vec<_>>(),
                vec!["user", "assistant"]
            );
            assert_eq!(
                clear_conversation_messages(&mut connection, "conversation-1").unwrap(),
                2
            );
            assert!(list_messages(&connection, "conversation-1")
                .unwrap()
                .is_empty());
            assert!(get_conversation(&connection, "conversation-1")
                .unwrap()
                .is_some());
            assert!(clear_conversation_messages(&mut connection, "missing").is_err());
            delete_conversation(&connection, "conversation-1").unwrap();
            assert!(list_messages(&connection, "conversation-1")
                .unwrap()
                .is_empty());
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn task_references_are_conversation_scoped_and_completion_is_audited() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection.execute("INSERT INTO tasks(id,title,status,priority) VALUES('critical-1','Demanda crítica','pending','critical')", []).unwrap();
        create_conversation(
            &connection,
            &ConversationInput {
                id: "conversation-ref".into(),
                title: "Demandas".into(),
            },
        )
        .unwrap();
        add_message(
            &mut connection,
            &MessageInput {
                id: "assistant-list".into(),
                conversation_id: "conversation-ref".into(),
                role: "assistant".into(),
                content: "1. Demanda crítica".into(),
            },
        )
        .unwrap();
        assert_eq!(
            save_task_references(
                &mut connection,
                &SaveTaskReferencesInput {
                    conversation_id: "conversation-ref".into(),
                    source_message_id: "assistant-list".into(),
                    task_ids: vec!["critical-1".into()],
                },
            )
            .unwrap(),
            1
        );

        let task = complete_referenced_task(&mut connection, "conversation-ref", 1).unwrap();
        assert_eq!(task.status, "completed");
        assert_eq!(
            connection
                .query_row(
                    "SELECT source FROM task_action_history WHERE task_id='critical-1'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "ai"
        );
        assert!(
            complete_referenced_task(&mut connection, "conversation-ref", 1)
                .unwrap_err()
                .contains("já está concluída")
        );

        clear_conversation_messages(&mut connection, "conversation-ref").unwrap();
        assert!(
            complete_referenced_task(&mut connection, "conversation-ref", 1)
                .unwrap_err()
                .contains("lista recente")
        );
    }
}
