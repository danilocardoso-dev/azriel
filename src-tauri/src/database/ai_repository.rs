use super::ai_models::*;
use rusqlite::{params, Connection, OptionalExtension};

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
            delete_conversation(&connection, "conversation-1").unwrap();
            assert!(list_messages(&connection, "conversation-1")
                .unwrap()
                .is_empty());
        }
        std::fs::remove_file(path).unwrap();
    }
}
