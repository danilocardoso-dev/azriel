use super::study_material_models::*;
use chrono::{DateTime, Utc};
use lopdf::Document;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

const PDF_MAX_BYTES: u64 = 100 * 1024 * 1024;
const PDF_PROCESS_MAX_BYTES: u64 = 32 * 1024 * 1024;
const IMAGE_MAX_BYTES: u64 = 25 * 1024 * 1024;
const TEXT_MAX_BYTES: u64 = 5 * 1024 * 1024;
const PREVIEW_MAX_BYTES: u64 = 32 * 1024 * 1024;
const PDF_EXTRACTOR_VERSION: &str = "STUDY_MATERIAL_PDF_TEXT_V1";
const TEXT_EXTRACTOR_VERSION: &str = "STUDY_MATERIAL_PLAIN_TEXT_V1";

struct DetectedFile {
    material_type: &'static str,
    mime_type: &'static str,
    extension: &'static str,
    size: u64,
    filename: String,
    modified_at: Option<String>,
    checksum: String,
}

struct ExtractedContent {
    status: &'static str,
    text: Option<String>,
    pages: Vec<String>,
    page_count: Option<i64>,
    extractor_version: Option<&'static str>,
    error_code: Option<&'static str>,
}

pub fn import_material(
    connection: &mut Connection,
    database_path: &Path,
    input: &ImportStudyMaterialInput,
) -> Result<ImportStudyMaterialResult, String> {
    validate_id(&input.id)?;
    validate_title(&input.title)?;
    validate_year(input.source_year)?;
    if !matches!(input.storage_kind.as_str(), "MANAGED_COPY" | "LINKED_LOCAL_FILE") {
        return Err("Storage local inválido. Use MANAGED_COPY ou LINKED_LOCAL_FILE.".into());
    }
    let source = PathBuf::from(input.source_path.trim())
        .canonicalize()
        .map_err(|_| "O arquivo selecionado não existe ou não está acessível.".to_string())?;
    if !source.is_file() {
        return Err("O material selecionado não é um arquivo.".into());
    }
    let detected = detect_file(&source)?;

    if let Some(existing_id) = connection
        .query_row(
            "SELECT id FROM study_materials WHERE checksum_sha256=?1 AND removed_at IS NULL LIMIT 1",
            [&detected.checksum],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
    {
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        save_relations(&transaction, &existing_id, &input.relations)?;
        transaction.commit().map_err(|error| error.to_string())?;
        return Ok(ImportStudyMaterialResult {
            material: get_material(connection, &existing_id)?.ok_or("Material duplicado não encontrado.")?,
            duplicate: true,
        });
    }

    let managed_root = managed_material_root(database_path)?;
    let (stored_path, cleanup_path) = if input.storage_kind == "MANAGED_COPY" {
        fs::create_dir_all(&managed_root).map_err(|error| format!("Não foi possível criar a biblioteca local: {error}"))?;
        let final_path = managed_root.join(format!("{}.{}", input.id, detected.extension));
        let temporary_path = managed_root.join(format!("{}.part", input.id));
        if final_path.exists() || temporary_path.exists() {
            return Err("Já existe um arquivo interno com este identificador.".into());
        }
        fs::copy(&source, &temporary_path).map_err(|error| format!("Falha ao copiar o material: {error}"))?;
        if let Err(error) = fs::rename(&temporary_path, &final_path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(format!("Falha ao finalizar a cópia gerenciada: {error}"));
        }
        (final_path.clone(), Some(final_path))
    } else {
        (source.clone(), None)
    };

    let extracted = match extract_content(&stored_path, &detected) {
        Ok(value) => value,
        Err(error) => {
            if let Some(path) = cleanup_path.as_ref() { let _ = fs::remove_file(path); }
            return Err(error);
        }
    };
    let now = Utc::now().to_rfc3339();
    let transaction = match connection.transaction() {
        Ok(value) => value,
        Err(error) => {
            if let Some(path) = cleanup_path.as_ref() { let _ = fs::remove_file(path); }
            return Err(error.to_string());
        }
    };
    let result = (|| {
        transaction.execute(
            "INSERT INTO study_materials(
               id,title,material_type,storage_kind,local_path,original_filename,mime_type,file_size,
               checksum_sha256,source_modified_at,source_author,source_title,source_year,description,page_count,
               status,created_at,updated_at
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,'ACTIVE',?16,?16)",
            params![
                input.id.trim(), input.title.trim(), detected.material_type, input.storage_kind,
                stored_path.to_string_lossy(), detected.filename, detected.mime_type, detected.size as i64,
                detected.checksum, detected.modified_at, clean(input.source_author.as_deref()),
                clean(input.source_title.as_deref()), input.source_year, clean(input.description.as_deref()),
                extracted.page_count, now,
            ],
        ).map_err(|error| error.to_string())?;
        save_text_content(&transaction, &input.id, &detected.checksum, &extracted, &now)?;
        save_relations(&transaction, &input.id, &input.relations)?;
        Ok::<(), String>(())
    })();
    if let Err(error) = result.and_then(|_| transaction.commit().map_err(|error| error.to_string())) {
        if let Some(path) = cleanup_path { let _ = fs::remove_file(path); }
        return Err(error);
    }
    Ok(ImportStudyMaterialResult {
        material: get_material(connection, &input.id)?.ok_or("Material importado não encontrado.")?,
        duplicate: false,
    })
}

pub fn add_link_material(
    connection: &mut Connection,
    input: &AddLinkStudyMaterialInput,
) -> Result<StudyMaterial, String> {
    validate_id(&input.id)?;
    validate_title(&input.title)?;
    validate_year(input.source_year)?;
    let url = reqwest::Url::parse(input.external_url.trim()).map_err(|_| "URL inválida.".to_string())?;
    if !matches!(url.scheme(), "http" | "https") || !url.username().is_empty() || url.password().is_some() {
        return Err("Somente URLs HTTP/HTTPS sem credenciais são permitidas.".into());
    }
    let now = Utc::now().to_rfc3339();
    let transaction = connection.transaction().map_err(|error| error.to_string())?;
    transaction.execute(
        "INSERT INTO study_materials(id,title,material_type,storage_kind,external_url,source_author,source_title,source_year,description,status,created_at,updated_at)
         VALUES (?1,?2,'LINK','EXTERNAL_URL',?3,?4,?5,?6,?7,'ACTIVE',?8,?8)",
        params![input.id.trim(), input.title.trim(), url.as_str(), clean(input.source_author.as_deref()), clean(input.source_title.as_deref()), input.source_year, clean(input.description.as_deref()), now],
    ).map_err(|error| error.to_string())?;
    let empty = ExtractedContent { status: "NOT_ATTEMPTED", text: None, pages: vec![], page_count: None, extractor_version: None, error_code: None };
    save_text_content(&transaction, &input.id, "", &empty, &now)?;
    save_relations(&transaction, &input.id, &input.relations)?;
    transaction.commit().map_err(|error| error.to_string())?;
    get_material(connection, &input.id)?.ok_or_else(|| "Link salvo não encontrado.".into())
}

pub fn list_materials(connection: &Connection, input: &StudyMaterialListInput) -> Result<Vec<StudyMaterialSummary>, String> {
    let limit = input.limit.clamp(1, 100);
    let offset = input.offset.max(0);
    let kind = clean(input.material_type.as_deref());
    let status = clean(input.status.as_deref());
    let relation_type = clean(input.relation_type.as_deref());
    let relation_id = clean(input.relation_id.as_deref());
    if relation_type.is_some() != relation_id.is_some() {
        return Err("Tipo e ID da relação devem ser informados juntos.".into());
    }
    if let Some(value) = relation_type.as_deref() { validate_relation_type(value)?; }
    let pattern = clean(input.query.as_deref()).map(|value| format!("%{}%", escape_like(&value)));
    let mut statement = connection.prepare(
        "SELECT material.id,material.title,material.material_type,material.storage_kind,material.original_filename,
                material.source_author,material.status,COALESCE(text.status,'NOT_ATTEMPTED'),COALESCE(text.is_stale,0),material.updated_at
         FROM study_materials material
         LEFT JOIN material_text_content text ON text.material_id=material.id
         WHERE material.removed_at IS NULL
           AND (?1 IS NULL OR material.material_type=?1)
           AND (?2 IS NULL OR material.status=?2)
           AND (?3 IS NULL OR material.title LIKE ?3 ESCAPE '\\' OR COALESCE(material.description,'') LIKE ?3 ESCAPE '\\'
                OR COALESCE(material.original_filename,'') LIKE ?3 ESCAPE '\\' OR COALESCE(material.source_author,'') LIKE ?3 ESCAPE '\\'
                OR COALESCE(text.text_content,'') LIKE ?3 ESCAPE '\\')
           AND (?4 IS NULL OR EXISTS(SELECT 1 FROM study_material_relations relation WHERE relation.material_id=material.id AND relation.relation_type=?4 AND relation.relation_id=?5))
         ORDER BY material.updated_at DESC, material.id
         LIMIT ?6 OFFSET ?7"
    ).map_err(|error| error.to_string())?;
    let rows = statement.query_map(params![kind, status, pattern, relation_type, relation_id, limit, offset], |row| {
        Ok(StudyMaterialSummary {
            id: row.get(0)?, title: row.get(1)?, material_type: row.get(2)?, storage_kind: row.get(3)?,
            original_filename: row.get(4)?, source_author: row.get(5)?, status: row.get(6)?, extraction_status: row.get(7)?,
            extraction_stale: row.get::<_, i64>(8)? == 1, relation_labels: vec![], updated_at: row.get(9)?,
        })
    }).map_err(|error| error.to_string())?;
    let mut materials = rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
    for material in &mut materials {
        material.relation_labels = list_relations(connection, &material.id)?.into_iter().map(|relation| relation.label).take(3).collect();
    }
    Ok(materials)
}

pub fn get_material(connection: &Connection, id: &str) -> Result<Option<StudyMaterial>, String> {
    validate_id(id)?;
    refresh_linked_state(connection, id)?;
    let row = connection.query_row(
        "SELECT id,title,material_type,storage_kind,external_url,original_filename,mime_type,file_size,checksum_sha256,
                source_author,source_title,source_year,description,page_count,status,removed_at,local_path,created_at,updated_at
         FROM study_materials WHERE id=?1",
        [id],
        |row| Ok((
            row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?, row.get::<_, Option<String>>(5)?, row.get::<_, Option<String>>(6)?, row.get::<_, Option<i64>>(7)?,
            row.get::<_, Option<String>>(8)?, row.get::<_, Option<String>>(9)?, row.get::<_, Option<String>>(10)?, row.get::<_, Option<i64>>(11)?,
            row.get::<_, Option<String>>(12)?, row.get::<_, Option<i64>>(13)?, row.get::<_, String>(14)?, row.get::<_, Option<String>>(15)?,
            row.get::<_, Option<String>>(16)?, row.get::<_, String>(17)?, row.get::<_, String>(18)?,
        )),
    ).optional().map_err(|error| error.to_string())?;
    let Some(row) = row else { return Ok(None); };
    Ok(Some(StudyMaterial {
        id: row.0.clone(), title: row.1, material_type: row.2, storage_kind: row.3, external_url: row.4,
        original_filename: row.5, mime_type: row.6, file_size: row.7, checksum_sha256: row.8,
        source_author: row.9, source_title: row.10, source_year: row.11, description: row.12, page_count: row.13,
        status: row.14, removed: row.15.is_some(), managed_file_available: row.16.as_ref().is_some_and(|path| Path::new(path).is_file()),
        relations: list_relations(connection, &row.0)?, text_content: get_text_content(connection, &row.0)?,
        created_at: row.17, updated_at: row.18,
    }))
}

pub fn update_material(connection: &Connection, input: &UpdateStudyMaterialInput) -> Result<StudyMaterial, String> {
    validate_id(&input.id)?;
    validate_title(&input.title)?;
    validate_year(input.source_year)?;
    let changed = connection.execute(
        "UPDATE study_materials SET title=?2,description=?3,source_author=?4,source_title=?5,source_year=?6,updated_at=?7 WHERE id=?1 AND removed_at IS NULL",
        params![input.id, input.title.trim(), clean(input.description.as_deref()), clean(input.source_author.as_deref()), clean(input.source_title.as_deref()), input.source_year, Utc::now().to_rfc3339()],
    ).map_err(|error| error.to_string())?;
    if changed == 0 { return Err("Material não encontrado na biblioteca.".into()); }
    get_material(connection, &input.id)?.ok_or_else(|| "Material atualizado não encontrado.".into())
}

pub fn set_status(connection: &Connection, id: &str, status: &str) -> Result<StudyMaterial, String> {
    validate_id(id)?;
    if !matches!(status, "ACTIVE" | "ARCHIVED") { return Err("Status de material inválido.".into()); }
    let changed = connection.execute("UPDATE study_materials SET status=?2,updated_at=?3 WHERE id=?1 AND removed_at IS NULL", params![id, status, Utc::now().to_rfc3339()]).map_err(|error| error.to_string())?;
    if changed == 0 { return Err("Material não encontrado na biblioteca.".into()); }
    get_material(connection, id)?.ok_or_else(|| "Material atualizado não encontrado.".into())
}

pub fn remove_from_library(connection: &Connection, id: &str) -> Result<(), String> {
    validate_id(id)?;
    let now = Utc::now().to_rfc3339();
    let changed = connection.execute("UPDATE study_materials SET status='ARCHIVED',removed_at=?2,updated_at=?2 WHERE id=?1 AND removed_at IS NULL", params![id, now]).map_err(|error| error.to_string())?;
    if changed == 0 { return Err("Material não encontrado na biblioteca.".into()); }
    Ok(())
}

pub fn delete_managed_file(connection: &Connection, database_path: &Path, id: &str) -> Result<StudyMaterial, String> {
    validate_id(id)?;
    let (storage_kind, local_path): (String, Option<String>) = connection.query_row("SELECT storage_kind,local_path FROM study_materials WHERE id=?1", [id], |row| Ok((row.get(0)?, row.get(1)?))).optional().map_err(|error| error.to_string())?.ok_or("Material não encontrado.")?;
    if storage_kind != "MANAGED_COPY" { return Err("Somente cópias gerenciadas podem ser excluídas fisicamente.".into()); }
    if let Some(path) = local_path {
        let root = managed_material_root(database_path)?.canonicalize().map_err(|_| "Diretório gerenciado indisponível.".to_string())?;
        let candidate = PathBuf::from(path);
        if candidate.exists() {
            let canonical = candidate.canonicalize().map_err(|error| error.to_string())?;
            if !canonical.starts_with(&root) { return Err("O arquivo não pertence ao armazenamento gerenciado.".into()); }
            fs::remove_file(&canonical).map_err(|error| format!("Não foi possível excluir a cópia gerenciada: {error}"))?;
        }
    }
    let now = Utc::now().to_rfc3339();
    connection.execute("UPDATE study_materials SET local_path=NULL,status='MISSING',managed_file_deleted_at=?2,updated_at=?2 WHERE id=?1", params![id, now]).map_err(|error| error.to_string())?;
    get_material(connection, id)?.ok_or_else(|| "Material atualizado não encontrado.".into())
}

pub fn associate(connection: &Connection, material_id: &str, input: &StudyMaterialRelationInput) -> Result<Vec<StudyMaterialRelation>, String> {
    validate_id(material_id)?;
    save_relations(connection, material_id, std::slice::from_ref(input))?;
    list_relations(connection, material_id)
}

pub fn dissociate(connection: &Connection, material_id: &str, relation_type: &str, relation_id: &str) -> Result<Vec<StudyMaterialRelation>, String> {
    validate_id(material_id)?;
    validate_relation_type(relation_type)?;
    validate_id(relation_id)?;
    connection.execute("DELETE FROM study_material_relations WHERE material_id=?1 AND relation_type=?2 AND relation_id=?3", params![material_id, relation_type, relation_id]).map_err(|error| error.to_string())?;
    list_relations(connection, material_id)
}

pub fn reprocess_text(connection: &Connection, id: &str) -> Result<MaterialTextContent, String> {
    validate_id(id)?;
    let row: (String, String, Option<String>) = connection.query_row("SELECT material_type,mime_type,local_path FROM study_materials WHERE id=?1 AND removed_at IS NULL", [id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional().map_err(|error| error.to_string())?.ok_or("Material não encontrado na biblioteca.")?;
    let path = row.2.ok_or("O material não possui arquivo local disponível.")?;
    let canonical = PathBuf::from(path).canonicalize().map_err(|_| "O arquivo local não está disponível.".to_string())?;
    let detected = detect_file(&canonical)?;
    if detected.material_type != row.0 { return Err("O tipo atual do arquivo diverge da metadata persistida.".into()); }
    let extracted = extract_content(&canonical, &detected)?;
    let now = Utc::now().to_rfc3339();
    let transaction = connection.unchecked_transaction().map_err(|error| error.to_string())?;
    save_text_content(&transaction, id, &detected.checksum, &extracted, &now)?;
    transaction.execute("UPDATE study_materials SET checksum_sha256=?2,file_size=?3,source_modified_at=?4,page_count=?5,status='ACTIVE',updated_at=?6 WHERE id=?1", params![id, detected.checksum, detected.size as i64, detected.modified_at, extracted.page_count, now]).map_err(|error| error.to_string())?;
    transaction.commit().map_err(|error| error.to_string())?;
    get_text_content(connection, id)
}

pub fn preview(connection: &Connection, id: &str) -> Result<MaterialPreview, String> {
    validate_id(id)?;
    let (path, mime): (Option<String>, Option<String>) = connection.query_row("SELECT local_path,mime_type FROM study_materials WHERE id=?1 AND removed_at IS NULL", [id], |row| Ok((row.get(0)?, row.get(1)?))).optional().map_err(|error| error.to_string())?.ok_or("Material não encontrado na biblioteca.")?;
    let path = path.ok_or("Este material não possui preview local.")?;
    let metadata = fs::metadata(&path).map_err(|_| "O arquivo local não está disponível.".to_string())?;
    if metadata.len() > PREVIEW_MAX_BYTES { return Err("Arquivo grande demais para preview interno. Use ABRIR EXTERNAMENTE.".into()); }
    Ok(MaterialPreview { bytes: fs::read(path).map_err(|error| error.to_string())?, mime_type: mime.unwrap_or_else(|| "application/octet-stream".into()) })
}

pub fn open_target(connection: &Connection, id: &str) -> Result<MaterialOpenTarget, String> {
    validate_id(id)?;
    connection.query_row("SELECT storage_kind,local_path,external_url FROM study_materials WHERE id=?1 AND removed_at IS NULL", [id], |row| Ok(MaterialOpenTarget { storage_kind: row.get(0)?, local_path: row.get(1)?, external_url: row.get(2)? })).optional().map_err(|error| error.to_string())?.ok_or_else(|| "Material não encontrado na biblioteca.".into())
}

fn detect_file(path: &Path) -> Result<DetectedFile, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    let size = metadata.len();
    if size == 0 { return Err("O arquivo está vazio.".into()); }
    let filename = path.file_name().and_then(|value| value.to_str()).ok_or("Nome de arquivo inválido.")?.chars().filter(|ch| !ch.is_control()).take(240).collect::<String>();
    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
    let declared_limit = match extension.as_str() {
        "pdf" => Some(PDF_MAX_BYTES),
        "png" | "jpg" | "jpeg" | "gif" | "webp" => Some(IMAGE_MAX_BYTES),
        "txt" | "md" | "markdown" => Some(TEXT_MAX_BYTES),
        _ => None,
    };
    if let Some(limit) = declared_limit {
        if size > limit { return Err(format!("O arquivo excede o limite de {} MB para este tipo.", limit / 1024 / 1024)); }
    }
    let mut header = [0u8; 8192];
    let count = File::open(path).and_then(|mut file| file.read(&mut header)).map_err(|error| error.to_string())?;
    let inferred = infer::get(&header[..count]).map(|kind| kind.mime_type().to_string());
    let (material_type, mime_type, safe_extension, limit) = if inferred.as_deref() == Some("application/pdf") && extension == "pdf" {
        Document::load(path).map_err(|_| "O arquivo não é um PDF válido.".to_string())?;
        ("PDF", "application/pdf", "pdf", PDF_MAX_BYTES)
    } else if inferred.as_deref().is_some_and(|mime| mime.starts_with("image/")) && matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
        let mime = match extension.as_str() { "png" => "image/png", "gif" => "image/gif", "webp" => "image/webp", _ => "image/jpeg" };
        let ext = match extension.as_str() { "png" => "png", "gif" => "gif", "webp" => "webp", _ => "jpg" };
        ("IMAGE", mime, ext, IMAGE_MAX_BYTES)
    } else if matches!(extension.as_str(), "txt" | "md" | "markdown") {
        let bytes = fs::read(path).map_err(|error| error.to_string())?;
        if bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() { return Err("O arquivo de texto não possui conteúdo UTF-8 válido.".into()); }
        if matches!(extension.as_str(), "md" | "markdown") { ("MARKDOWN", "text/markdown", "md", TEXT_MAX_BYTES) } else { ("TEXT", "text/plain", "txt", TEXT_MAX_BYTES) }
    } else {
        return Err("Tipo de arquivo não suportado ou assinatura incompatível com a extensão.".into());
    };
    if size > limit { return Err(format!("O arquivo excede o limite de {} MB para este tipo.", limit / 1024 / 1024)); }
    Ok(DetectedFile { material_type, mime_type, extension: safe_extension, size, filename, modified_at: modified_at(&metadata), checksum: checksum(path)? })
}

fn extract_content(path: &Path, detected: &DetectedFile) -> Result<ExtractedContent, String> {
    match detected.material_type {
        "PDF" if detected.size > PDF_PROCESS_MAX_BYTES => Ok(ExtractedContent { status: "NOT_ATTEMPTED", text: None, pages: vec![], page_count: pdf_page_count(path).ok(), extractor_version: Some(PDF_EXTRACTOR_VERSION), error_code: Some("EXTRACTION_SIZE_LIMIT") }),
        "PDF" => {
            let document = Document::load(path).map_err(|_| "O arquivo não é um PDF válido.".to_string())?;
            let page_numbers = document.get_pages().keys().copied().collect::<Vec<_>>();
            let mut pages = Vec::with_capacity(page_numbers.len());
            for page in &page_numbers {
                match document.extract_text(&[*page]) {
                    Ok(text) => pages.push(text.trim().to_string()),
                    Err(_) => return Ok(ExtractedContent { status: "FAILED", text: None, pages: vec![], page_count: Some(page_numbers.len() as i64), extractor_version: Some(PDF_EXTRACTOR_VERSION), error_code: Some("PDF_TEXT_EXTRACTION_FAILED") }),
                }
            }
            let combined = pages.iter().filter(|page| !page.is_empty()).cloned().collect::<Vec<_>>().join("\n\n");
            if combined.trim().is_empty() {
                Ok(ExtractedContent { status: "TEXT_UNAVAILABLE", text: None, pages, page_count: Some(page_numbers.len() as i64), extractor_version: Some(PDF_EXTRACTOR_VERSION), error_code: None })
            } else {
                Ok(ExtractedContent { status: "AVAILABLE", text: Some(combined), pages, page_count: Some(page_numbers.len() as i64), extractor_version: Some(PDF_EXTRACTOR_VERSION), error_code: None })
            }
        }
        "TEXT" | "MARKDOWN" => {
            let value = fs::read_to_string(path).map_err(|error| format!("Falha ao ler o texto: {error}"))?;
            Ok(ExtractedContent { status: "AVAILABLE", text: Some(value.clone()), pages: vec![value], page_count: None, extractor_version: Some(TEXT_EXTRACTOR_VERSION), error_code: None })
        }
        _ => Ok(ExtractedContent { status: "NOT_ATTEMPTED", text: None, pages: vec![], page_count: None, extractor_version: None, error_code: None }),
    }
}

fn pdf_page_count(path: &Path) -> Result<i64, String> {
    Document::load(path).map(|document| document.get_pages().len() as i64).map_err(|error| error.to_string())
}

fn save_text_content(connection: &Connection, material_id: &str, checksum: &str, extracted: &ExtractedContent, now: &str) -> Result<(), String> {
    let pages = if extracted.pages.is_empty() { None } else { Some(serde_json::to_string(&extracted.pages).map_err(|error| error.to_string())?) };
    let char_count = extracted.text.as_deref().map_or(0, |text| text.chars().count() as i64);
    connection.execute(
        "INSERT INTO material_text_content(material_id,status,text_content,pages_json,extracted_at,extractor_version,char_count,source_checksum_sha256,is_stale,error_code,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,0,?9,?5)
         ON CONFLICT(material_id) DO UPDATE SET status=excluded.status,text_content=excluded.text_content,pages_json=excluded.pages_json,
           extracted_at=excluded.extracted_at,extractor_version=excluded.extractor_version,char_count=excluded.char_count,
           source_checksum_sha256=excluded.source_checksum_sha256,is_stale=0,error_code=excluded.error_code,updated_at=excluded.updated_at",
        params![material_id, extracted.status, extracted.text, pages, now, extracted.extractor_version, char_count, if checksum.is_empty() { None } else { Some(checksum) }, extracted.error_code],
    ).map_err(|error| error.to_string())?;
    Ok(())
}

fn get_text_content(connection: &Connection, material_id: &str) -> Result<MaterialTextContent, String> {
    connection.query_row(
        "SELECT status,text_content,pages_json,extracted_at,extractor_version,char_count,is_stale,error_code FROM material_text_content WHERE material_id=?1",
        [material_id],
        |row| {
            let pages_json: Option<String> = row.get(2)?;
            Ok(MaterialTextContent { material_id: material_id.to_string(), status: row.get(0)?, text: row.get(1)?, pages: pages_json.and_then(|value| serde_json::from_str(&value).ok()).unwrap_or_default(), extracted_at: row.get(3)?, extractor_version: row.get(4)?, char_count: row.get(5)?, is_stale: row.get::<_, i64>(6)? == 1, error_code: row.get(7)? })
        },
    ).optional().map_err(|error| error.to_string())?.ok_or_else(|| "Estado de extração não encontrado.".into())
}

fn save_relations(connection: &Connection, material_id: &str, relations: &[StudyMaterialRelationInput]) -> Result<(), String> {
    if relations.len() > 50 { return Err("Um material pode receber no máximo 50 relações por operação.".into()); }
    let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM study_materials WHERE id=?1)", [material_id], |row| row.get::<_, i64>(0)).map_err(|error| error.to_string())? == 1;
    if !exists { return Err("Material não encontrado.".into()); }
    let now = Utc::now().to_rfc3339();
    for relation in relations {
        validate_relation(connection, relation)?;
        let relation_id = format!("material-relation-{material_id}-{}-{}", relation.relation_type.to_ascii_lowercase(), relation.relation_id);
        connection.execute("INSERT OR IGNORE INTO study_material_relations(id,material_id,relation_type,relation_id,created_at) VALUES (?1,?2,?3,?4,?5)", params![relation_id, material_id, relation.relation_type, relation.relation_id, now]).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn validate_relation(connection: &Connection, relation: &StudyMaterialRelationInput) -> Result<(), String> {
    validate_relation_type(&relation.relation_type)?;
    validate_id(&relation.relation_id)?;
    let table = relation_table(&relation.relation_type)?;
    let query = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id=?1)");
    let exists = connection.query_row(&query, [&relation.relation_id], |row| row.get::<_, i64>(0)).map_err(|error| error.to_string())? == 1;
    if !exists { return Err(format!("Relação {} aponta para um registro inexistente.", relation.relation_type)); }
    Ok(())
}

fn list_relations(connection: &Connection, material_id: &str) -> Result<Vec<StudyMaterialRelation>, String> {
    let mut statement = connection.prepare("SELECT id,relation_type,relation_id,created_at FROM study_material_relations WHERE material_id=?1 ORDER BY created_at,id").map_err(|error| error.to_string())?;
    let rows = statement.query_map([material_id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?))).map_err(|error| error.to_string())?;
    let mut relations = Vec::new();
    for row in rows {
        let (id, relation_type, relation_id, created_at) = row.map_err(|error| error.to_string())?;
        relations.push(StudyMaterialRelation { id, label: relation_label(connection, &relation_type, &relation_id)?, relation_type, relation_id, created_at });
    }
    Ok(relations)
}

fn relation_label(connection: &Connection, relation_type: &str, relation_id: &str) -> Result<String, String> {
    let (table, column) = match relation_type {
        "ROADMAP" => ("study_roadmaps", "name"), "STAGE" => ("roadmap_stages", "name"), "TOPIC" => ("roadmap_topics", "name"),
        "ACTIVITY" => ("roadmap_activities", "title"), "NOTEBOOK" => ("study_notebooks", "title"), "NOTE" => ("study_notes", "title"), "CARD" => ("study_cards", "front"),
        _ => return Err("Tipo de relação inválido.".into()),
    };
    connection.query_row(&format!("SELECT {column} FROM {table} WHERE id=?1"), [relation_id], |row| row.get::<_, String>(0)).optional().map_err(|error| error.to_string()).map(|value| value.unwrap_or_else(|| format!("{relation_type} REMOVIDO")))
}

fn refresh_linked_state(connection: &Connection, id: &str) -> Result<(), String> {
    let linked = connection.query_row("SELECT storage_kind,local_path,file_size,source_modified_at,status FROM study_materials WHERE id=?1", [id], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?, row.get::<_, Option<i64>>(2)?, row.get::<_, Option<String>>(3)?, row.get::<_, String>(4)?))).optional().map_err(|error| error.to_string())?;
    let Some((storage_kind, Some(path), persisted_size, persisted_modified, status)) = linked else { return Ok(()); };
    if storage_kind != "LINKED_LOCAL_FILE" { return Ok(()); }
    let now = Utc::now().to_rfc3339();
    match fs::metadata(&path) {
        Ok(metadata) => {
            let current_modified = modified_at(&metadata);
            let stale = persisted_size != Some(metadata.len() as i64) || persisted_modified != current_modified;
            connection.execute("UPDATE material_text_content SET is_stale=?2,updated_at=CASE WHEN is_stale<>?2 THEN ?3 ELSE updated_at END WHERE material_id=?1", params![id, stale as i64, now]).map_err(|error| error.to_string())?;
            if status == "MISSING" { connection.execute("UPDATE study_materials SET status='ACTIVE',updated_at=?2 WHERE id=?1", params![id, now]).map_err(|error| error.to_string())?; }
        }
        Err(_) => {
            connection.execute("UPDATE study_materials SET status='MISSING',updated_at=?2 WHERE id=?1", params![id, now]).map_err(|error| error.to_string())?;
            connection.execute("UPDATE material_text_content SET is_stale=1,updated_at=?2 WHERE material_id=?1", params![id, now]).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn managed_material_root(database_path: &Path) -> Result<PathBuf, String> {
    database_path.parent().map(|parent| parent.join("study-materials")).ok_or_else(|| "Diretório de dados do Azriel inválido.".into())
}

fn checksum(path: &Path) -> Result<String, String> {
    let mut reader = BufReader::new(File::open(path).map_err(|error| error.to_string())?);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer).map_err(|error| error.to_string())?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn modified_at(metadata: &fs::Metadata) -> Option<String> {
    metadata.modified().ok().map(|value: SystemTime| DateTime::<Utc>::from(value).to_rfc3339())
}

fn validate_relation_type(value: &str) -> Result<(), String> {
    if matches!(value, "ROADMAP" | "STAGE" | "TOPIC" | "ACTIVITY" | "NOTEBOOK" | "NOTE" | "CARD") { Ok(()) } else { Err("Tipo de relação inválido.".into()) }
}

fn relation_table(value: &str) -> Result<&'static str, String> {
    match value { "ROADMAP" => Ok("study_roadmaps"), "STAGE" => Ok("roadmap_stages"), "TOPIC" => Ok("roadmap_topics"), "ACTIVITY" => Ok("roadmap_activities"), "NOTEBOOK" => Ok("study_notebooks"), "NOTE" => Ok("study_notes"), "CARD" => Ok("study_cards"), _ => Err("Tipo de relação inválido.".into()) }
}

fn validate_id(value: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > 128 || !value.chars().all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')) { return Err("Identificador inválido.".into()); }
    Ok(())
}

fn validate_title(value: &str) -> Result<(), String> {
    let length = value.trim().chars().count();
    if !(1..=240).contains(&length) { return Err("O título deve possuir entre 1 e 240 caracteres.".into()); }
    Ok(())
}

fn validate_year(value: Option<i64>) -> Result<(), String> {
    if value.is_some_and(|year| !(1000..=9999).contains(&year)) { return Err("Ano da fonte inválido.".into()); }
    Ok(())
}

fn clean(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(|value| value.chars().take(4_000).collect())
}

fn escape_like(value: &str) -> String {
    value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;
    use lopdf::{content::{Content, Operation}, dictionary, Object, Stream};

    fn database_and_root(label: &str) -> (Connection, PathBuf) {
        let root = std::env::temp_dir().join(format!("azriel-study-material-{label}-{}", Utc::now().timestamp_nanos_opt().unwrap_or_default()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("azriel.db");
        (database::open(&path).unwrap(), root)
    }

    fn text_input(id: &str, source_path: &Path, storage_kind: &str) -> ImportStudyMaterialInput {
        ImportStudyMaterialInput { id: id.into(), title: "Material de teste".into(), source_path: source_path.to_string_lossy().into_owned(), storage_kind: storage_kind.into(), source_author: None, source_title: None, source_year: None, description: None, relations: vec![] }
    }

    fn write_pdf(path: &Path, text: Option<&str>) {
        let mut document = Document::with_version("1.5");
        let pages_id = document.new_object_id();
        let font_id = document.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
        });
        let resources_id = document.add_object(dictionary! {
            "Font" => dictionary! { "F1" => font_id },
        });
        let operations = text.map(|value| vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), 14.into()]),
            Operation::new("Td", vec![72.into(), 720.into()]),
            Operation::new("Tj", vec![Object::string_literal(value)]),
            Operation::new("ET", vec![]),
        ]).unwrap_or_default();
        let content_id = document.add_object(Stream::new(dictionary! {}, Content { operations }.encode().unwrap()));
        let page_id = document.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content_id,
            "Resources" => resources_id, "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        document.objects.insert(pages_id, Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1,
        }));
        let catalog_id = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        document.trailer.set("Root", catalog_id);
        document.save(path).unwrap();
    }

    #[test]
    fn text_import_is_managed_searchable_and_deduplicated() {
        let (mut connection, root) = database_and_root("text");
        let source = root.join("conteúdo estranho 01.md");
        fs::write(&source, "# Eletrônica\n\nLei de Ohm").unwrap();
        let path = root.join("azriel.db");
        let imported = import_material(&mut connection, &path, &text_input("material-1", &source, "MANAGED_COPY")).unwrap();
        assert!(!imported.duplicate);
        assert_eq!(imported.material.text_content.status, "AVAILABLE");
        assert_eq!(list_materials(&connection, &StudyMaterialListInput { query: Some("Ohm".into()), material_type: None, status: None, relation_type: None, relation_id: None, limit: 50, offset: 0 }).unwrap().len(), 1);
        let duplicate = import_material(&mut connection, &path, &text_input("material-2", &source, "MANAGED_COPY")).unwrap();
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.material.id, "material-1");
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_materials", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn invalid_binary_and_oversized_text_are_rejected_without_rows() {
        let (mut connection, root) = database_and_root("invalid");
        let source = root.join("fake.txt");
        fs::write(&source, [0, 1, 2, 3]).unwrap();
        assert!(import_material(&mut connection, &root.join("azriel.db"), &text_input("material-invalid", &source, "MANAGED_COPY")).is_err());
        let oversized = root.join("oversized.txt");
        File::create(&oversized).unwrap().set_len(TEXT_MAX_BYTES + 1).unwrap();
        assert!(import_material(&mut connection, &root.join("azriel.db"), &text_input("material-oversized", &oversized, "MANAGED_COPY")).is_err());
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_materials", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn image_signature_is_accepted_without_text_extraction() {
        let (mut connection, root) = database_and_root("image");
        let source = root.join("imagem estranha [01].png");
        fs::write(&source, [
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
            0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
            0x89, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
            0x44, 0xae, 0x42, 0x60, 0x82,
        ]).unwrap();
        let imported = import_material(&mut connection, &root.join("azriel.db"), &text_input("image-1", &source, "LINKED_LOCAL_FILE")).unwrap();
        assert_eq!(imported.material.material_type, "IMAGE");
        assert_eq!(imported.material.text_content.status, "NOT_ATTEMPTED");
        assert_eq!(imported.material.original_filename.as_deref(), Some("imagem estranha [01].png"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn pdf_text_layer_and_no_text_are_distinguished_and_reprocessable() {
        let (mut connection, root) = database_and_root("pdf");
        let path = root.join("azriel.db");
        let text_pdf = root.join("text-layer.pdf");
        write_pdf(&text_pdf, Some("Circuitos e sensores"));
        let imported = import_material(&mut connection, &path, &text_input("pdf-text", &text_pdf, "MANAGED_COPY")).unwrap();
        assert_eq!(imported.material.text_content.status, "AVAILABLE");
        assert_eq!(imported.material.text_content.extractor_version.as_deref(), Some(PDF_EXTRACTOR_VERSION));
        assert!(imported.material.text_content.text.as_deref().unwrap_or_default().contains("Circuitos"));
        assert_eq!(reprocess_text(&connection, "pdf-text").unwrap().status, "AVAILABLE");

        let blank_pdf = root.join("blank.pdf");
        write_pdf(&blank_pdf, None);
        let blank = import_material(&mut connection, &path, &text_input("pdf-blank", &blank_pdf, "LINKED_LOCAL_FILE")).unwrap();
        assert_eq!(blank.material.text_content.status, "TEXT_UNAVAILABLE");
        assert_eq!(blank.material.page_count, Some(1));

        let invalid_pdf = root.join("invalid.pdf");
        fs::write(&invalid_pdf, b"%PDF-not-really-a-pdf").unwrap();
        assert!(import_material(&mut connection, &path, &text_input("pdf-invalid", &invalid_pdf, "MANAGED_COPY")).is_err());
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_materials WHERE id='pdf-invalid'", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn link_validation_and_relation_validation_are_strict() {
        let (mut connection, root) = database_and_root("links");
        let invalid = AddLinkStudyMaterialInput { id: "link-1".into(), title: "Inválido".into(), external_url: "file:///C:/secret.txt".into(), description: None, source_author: None, source_title: None, source_year: None, relations: vec![] };
        assert!(add_link_material(&mut connection, &invalid).is_err());
        let missing_relation = AddLinkStudyMaterialInput { id: "link-2".into(), title: "Documentação".into(), external_url: "https://example.com/docs".into(), description: None, source_author: None, source_title: None, source_year: None, relations: vec![StudyMaterialRelationInput { relation_type: "NOTE".into(), relation_id: "missing".into() }] };
        assert!(add_link_material(&mut connection, &missing_relation).is_err());
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_materials", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn managed_delete_never_deletes_linked_source() {
        let (mut connection, root) = database_and_root("delete");
        let source = root.join("linked.txt");
        fs::write(&source, "preservar").unwrap();
        let path = root.join("azriel.db");
        let imported = import_material(&mut connection, &path, &text_input("linked-material", &source, "LINKED_LOCAL_FILE")).unwrap();
        assert_eq!(imported.material.storage_kind, "LINKED_LOCAL_FILE");
        assert!(delete_managed_file(&connection, &path, "linked-material").is_err());
        assert!(source.exists());
        let _ = fs::remove_dir_all(root);
    }
}
