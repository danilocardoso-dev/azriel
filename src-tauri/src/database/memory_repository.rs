use super::memory_models::*;
use rusqlite::{params, Connection, OptionalExtension};

pub const DEFAULT_IDENTITY: &str = include_str!("../../prompts/identity-v2.txt");
/// Padrões anteriores: se o banco ainda tem um deles intacto, sobe para o atual. Texto editado pelo Senhor nunca é trocado.
const PREVIOUS_DEFAULTS: [&str; 1] = [include_str!("../../prompts/identity-v1.txt")];
pub const CATEGORIES: [&str; 6] = ["operador", "preferencia", "projeto", "pessoa", "decisao", "fato"];
const MAX_MEMORY_CHARS: usize = 1_000;
const MAX_IDENTITY_CHARS: usize = 20_000;
const COLUMNS: &str = "id,content,category,source,conversation_id,created_at,updated_at";

fn err(error: rusqlite::Error) -> String { error.to_string() }

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Memory> {
    Ok(Memory {
        id: row.get(0)?,
        content: row.get(1)?,
        category: row.get(2)?,
        source: row.get(3)?,
        conversation_id: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

// ---------- identidade ----------

/// Grava a identidade padrão só na primeira vez; a versão editada pelo operador nunca é sobrescrita.
pub fn seed_identity(connection: &Connection) -> Result<(), String> {
    connection
        .execute("INSERT OR IGNORE INTO azriel_identity(id, content) VALUES (1, ?1)", [DEFAULT_IDENTITY.trim()])
        .map_err(err)?;
    let stored: String = connection.query_row("SELECT content FROM azriel_identity WHERE id=1", [], |row| row.get(0)).map_err(err)?;
    if PREVIOUS_DEFAULTS.iter().any(|previous| same_text(previous, &stored)) {
        connection
            .execute("UPDATE azriel_identity SET content=?1, updated_at=CURRENT_TIMESTAMP WHERE id=1", [DEFAULT_IDENTITY.trim()])
            .map_err(err)?;
    }
    Ok(())
}

/// Compara ignorando fim de linha (o Git no Windows troca LF por CRLF) e espaços nas pontas.
fn same_text(a: &str, b: &str) -> bool {
    a.replace("\r\n", "\n").trim() == b.replace("\r\n", "\n").trim()
}

pub fn get_identity(connection: &Connection) -> Result<Identity, String> {
    let stored: Option<(String, String)> = connection
        .query_row("SELECT content, updated_at FROM azriel_identity WHERE id=1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
        .map_err(err)?;
    let (content, updated_at) = stored.unwrap_or_else(|| (DEFAULT_IDENTITY.trim().to_string(), String::new()));
    Ok(Identity { is_default: same_text(&content, DEFAULT_IDENTITY), content, updated_at })
}

pub fn save_identity(connection: &Connection, content: &str) -> Result<Identity, String> {
    let content = content.trim();
    if content.is_empty() { return Err("A identidade não pode ficar vazia.".into()); }
    if content.chars().count() > MAX_IDENTITY_CHARS { return Err(format!("A identidade pode ter no máximo {MAX_IDENTITY_CHARS} caracteres.")); }
    connection
        .execute(
            "INSERT INTO azriel_identity(id, content, updated_at) VALUES (1, ?1, CURRENT_TIMESTAMP)
             ON CONFLICT(id) DO UPDATE SET content=excluded.content, updated_at=CURRENT_TIMESTAMP",
            [content],
        )
        .map_err(err)?;
    get_identity(connection)
}

pub fn reset_identity(connection: &Connection) -> Result<Identity, String> {
    save_identity(connection, DEFAULT_IDENTITY)
}

// ---------- memórias ----------

fn validate(input: &MemoryInput) -> Result<(String, String), String> {
    if input.id.trim().is_empty() || input.id.chars().count() > 64 { return Err("Memória sem identificador válido".into()); }
    let content = input.content.split_whitespace().collect::<Vec<_>>().join(" ");
    if content.is_empty() { return Err("A memória está vazia.".into()); }
    if content.chars().count() > MAX_MEMORY_CHARS { return Err(format!("Uma memória pode ter no máximo {MAX_MEMORY_CHARS} caracteres.")); }
    let category = input.category.trim().to_lowercase();
    let category = if CATEGORIES.contains(&category.as_str()) { category } else { "fato".to_string() };
    if !["azriel", "operador"].contains(&input.source.as_str()) { return Err("Origem da memória inválida".into()); }
    Ok((content, category))
}

/// Cria ou corrige uma memória. A data de criação é preservada; a de atualização avança.
pub fn save_memory(connection: &Connection, input: &MemoryInput) -> Result<Memory, String> {
    let (content, category) = validate(input)?;
    connection
        .execute(
            "INSERT INTO azriel_memories(id, content, category, source, conversation_id) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET content=excluded.content, category=excluded.category, updated_at=CURRENT_TIMESTAMP",
            params![input.id.trim(), content, category, input.source, input.conversation_id],
        )
        .map_err(err)?;
    get_memory(connection, input.id.trim())?.ok_or_else(|| "Memória não encontrada após salvar".into())
}

pub fn get_memory(connection: &Connection, id: &str) -> Result<Option<Memory>, String> {
    connection
        .query_row(&format!("SELECT {COLUMNS} FROM azriel_memories WHERE id=?1"), [id], row)
        .optional()
        .map_err(err)
}

/// Mais recentes primeiro (atualizadas contam como recentes).
pub fn list_memories(connection: &Connection) -> Result<Vec<Memory>, String> {
    let mut statement = connection
        .prepare(&format!("SELECT {COLUMNS} FROM azriel_memories ORDER BY updated_at DESC, created_at DESC"))
        .map_err(err)?;
    let memories = statement.query_map([], row).map_err(err)?.collect::<Result<Vec<_>, _>>().map_err(err);
    memories
}

pub fn delete_memory(connection: &Connection, id: &str) -> Result<(), String> {
    let removed = connection.execute("DELETE FROM azriel_memories WHERE id=?1", [id]).map_err(err)?;
    if removed == 0 { return Err("Memória não encontrada.".into()); }
    Ok(())
}

/// Busca por palavras (3+ letras), ordenando pelas que casam mais termos.
pub fn search_memories(connection: &Connection, query: &str) -> Result<Vec<Memory>, String> {
    let terms: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.chars().count() >= 3)
        .map(String::from)
        .collect();
    let all = list_memories(connection)?;
    if terms.is_empty() { return Ok(all.into_iter().take(20).collect()); }
    let mut scored: Vec<(usize, Memory)> = all
        .into_iter()
        .map(|memory| {
            let text = format!("{} {}", memory.content.to_lowercase(), memory.category);
            (terms.iter().filter(|term| text.contains(term.as_str())).count(), memory)
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(scored.into_iter().take(20).map(|(_, memory)| memory).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connection() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::database::initialize(&mut connection).unwrap();
        connection
    }

    fn input(id: &str, content: &str, category: &str) -> MemoryInput {
        MemoryInput { id: id.into(), content: content.into(), category: category.into(), source: "azriel".into(), conversation_id: None }
    }

    #[test]
    fn identity_starts_with_the_default_and_keeps_the_operator_version() {
        let connection = connection();
        let identity = get_identity(&connection).unwrap();
        assert!(identity.is_default);
        assert!(identity.content.contains("Senhor Danilo"));
        let edited = save_identity(&connection, "Sou o Azriel, versão do Senhor.").unwrap();
        assert!(!edited.is_default);
        seed_identity(&connection).unwrap();
        assert_eq!(get_identity(&connection).unwrap().content, "Sou o Azriel, versão do Senhor.");
        assert!(reset_identity(&connection).unwrap().is_default);
        assert!(save_identity(&connection, "   ").is_err());
    }

    #[test]
    fn untouched_old_default_is_upgraded_but_edited_text_is_kept() {
        let connection = connection();
        save_identity(&connection, PREVIOUS_DEFAULTS[0]).unwrap();
        seed_identity(&connection).unwrap();
        let identity = get_identity(&connection).unwrap();
        assert!(identity.is_default);
        assert!(identity.content.contains("Excluir qualquer coisa: só com a permissão do Senhor"));
        save_identity(&connection, &PREVIOUS_DEFAULTS[0].replace('\n', "\r\n")).unwrap();
        seed_identity(&connection).unwrap();
        assert!(get_identity(&connection).unwrap().is_default, "CRLF do Git não impede a atualização");
        save_identity(&connection, &format!("{}\nNota minha.", PREVIOUS_DEFAULTS[0])).unwrap();
        seed_identity(&connection).unwrap();
        assert!(get_identity(&connection).unwrap().content.ends_with("Nota minha."));
    }

    #[test]
    fn memories_are_saved_corrected_searched_and_deleted() {
        let connection = connection();
        save_memory(&connection, &input("m1", "O Senhor  prefere café   sem açúcar", "preferencia")).unwrap();
        save_memory(&connection, &input("m2", "Projeto Azriel em Tauri e React", "projeto")).unwrap();
        let fixed = save_memory(&connection, &input("m1", "O Senhor prefere café com pouco açúcar", "preferencia")).unwrap();
        assert_eq!(fixed.content, "O Senhor prefere café com pouco açúcar");
        assert_eq!(list_memories(&connection).unwrap().len(), 2);
        assert_eq!(search_memories(&connection, "como ele gosta do café?").unwrap()[0].id, "m1");
        assert!(search_memories(&connection, "futebol").unwrap().is_empty());
        delete_memory(&connection, "m1").unwrap();
        assert!(delete_memory(&connection, "m1").is_err());
        assert_eq!(list_memories(&connection).unwrap().len(), 1);
    }

    #[test]
    fn invalid_memories_are_rejected_and_unknown_categories_fall_back() {
        let connection = connection();
        assert!(save_memory(&connection, &input("m1", "  ", "fato")).is_err());
        assert!(save_memory(&connection, &input("m2", &"x".repeat(1_001), "fato")).is_err());
        assert_eq!(save_memory(&connection, &input("m3", "Algo", "inventada")).unwrap().category, "fato");
        let mut foreign = input("m4", "Algo", "fato");
        foreign.source = "hacker".into();
        assert!(save_memory(&connection, &foreign).is_err());
    }
}
