//! Programação do Azriel: ler, buscar, criar e editar arquivos e rodar comandos no disco C.
//! Regra do Senhor: tudo por conta própria, exceto excluir (a interface pede permissão antes).
//! Proteções: só caminhos absolutos no C:, backup antes de sobrescrever ou editar, exclusão vai
//! para a Lixeira, tempo limite nos comandos e histórico de tudo o que foi feito.

use crate::database::DatabaseState;
use serde::Serialize;
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager, State};

const MAX_READ_BYTES: u64 = 5_000_000;
const MAX_READ_CHARS: usize = 60_000;
const MAX_WRITE_CHARS: usize = 2_000_000;
const MAX_OUTPUT_CHARS: usize = 12_000;
const MAX_LIST: usize = 500;
const MAX_SEARCH_RESULTS: usize = 100;
const MAX_SEARCH_FILES: usize = 20_000;
const MAX_SEARCH_FILE_BYTES: u64 = 1_000_000;
const DEFAULT_TIMEOUT_SECONDS: u64 = 120;
const MAX_TIMEOUT_SECONDS: u64 = 600;
/// Pastas grandes e geradas que a busca pula (o Senhor pode listar e ler dentro delas normalmente).
const SKIP_DIRS: [&str; 8] = ["node_modules", ".git", "target", "dist", "build", "$Recycle.Bin", "System Volume Information", "Windows"];

fn err(error: impl std::fmt::Display) -> String { error.to_string() }

// ---------- caminhos ----------

/// Aceita só caminhos absolutos no disco C (ex.: C:\Projetos\app). Normaliza "." e ".." sem sair do disco.
pub fn resolve(path: &str) -> Result<PathBuf, String> {
    let raw = path.trim().trim_matches('"').replace('/', "\\");
    let invalid = || format!("Caminho inválido: \"{}\". Use um caminho completo no disco C, como C:\\Projetos\\pasta.", path.trim());
    if raw.len() < 3 || !raw[..3].eq_ignore_ascii_case("C:\\") || raw.starts_with("\\\\") {
        return Err(invalid());
    }
    if raw[2..].contains(':') || raw.chars().any(|c| c.is_control() || matches!(c, '<' | '>' | '|' | '?' | '*')) {
        return Err(invalid());
    }
    let mut parts: Vec<String> = Vec::new();
    for component in Path::new(&raw[3..]).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::ParentDir => { parts.pop(); }
            Component::CurDir => {}
            _ => return Err(invalid()),
        }
    }
    let mut resolved = PathBuf::from("C:\\");
    resolved.extend(parts);
    Ok(resolved)
}

fn is_drive_root(path: &Path) -> bool {
    path.components().count() <= 2
}

fn tail(text: &str, max: usize) -> String {
    let count = text.chars().count();
    if count <= max { return text.to_string(); }
    format!("[...{} caracteres anteriores omitidos...]\n{}", count - max, text.chars().skip(count - max).collect::<String>())
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8_000).any(|byte| *byte == 0)
}

// ---------- histórico ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRecord {
    pub id: i64,
    pub created_at: String,
    pub kind: String,
    pub target: String,
    pub detail: String,
    pub ok: bool,
}

fn log(app: &AppHandle, kind: &str, target: &str, detail: &str, ok: bool) {
    let Some(state) = app.try_state::<DatabaseState>() else { return };
    let Ok(connection) = state.connection.lock() else { return };
    let detail: String = detail.chars().take(2_000).collect();
    let _ = connection.execute(
        "INSERT INTO azriel_actions(kind, target, detail, ok) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![kind, target, detail, ok],
    );
}

#[tauri::command]
pub fn list_actions(state: State<'_, DatabaseState>, limit: Option<u32>) -> Result<Vec<ActionRecord>, String> {
    let connection = state.connection.lock().map_err(|_| "O banco de dados está indisponível".to_string())?;
    let mut statement = connection
        .prepare("SELECT id, created_at, kind, target, detail, ok FROM azriel_actions ORDER BY id DESC LIMIT ?1")
        .map_err(err)?;
    let rows = statement
        .query_map([limit.unwrap_or(100).min(500)], |row| Ok(ActionRecord { id: row.get(0)?, created_at: row.get(1)?, kind: row.get(2)?, target: row.get(3)?, detail: row.get(4)?, ok: row.get(5)? }))
        .map_err(err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(err);
    rows
}

// ---------- backup ----------

/// Copia o arquivo para a pasta de backups do Azriel antes de mudá-lo. Devolve onde ficou.
fn backup(backups_root: &Path, file: &Path) -> Result<PathBuf, String> {
    let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S-%3f").to_string();
    let relative: PathBuf = file.components().filter_map(|component| match component {
        Component::Prefix(_) => Some(PathBuf::from("C")),
        Component::Normal(part) => Some(PathBuf::from(part)),
        _ => None,
    }).collect();
    let target = backups_root.join(stamp).join(relative);
    if let Some(parent) = target.parent() { fs::create_dir_all(parent).map_err(err)?; }
    fs::copy(file, &target).map_err(|error| format!("Não foi possível fazer o backup antes de alterar: {error}"))?;
    Ok(target)
}

fn backups_root(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_data_dir().map_err(err)?.join("backups"))
}

// ---------- operações (puras, testáveis) ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntry { name: String, path: String, kind: &'static str, size: Option<u64> }

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing { path: String, entries: Vec<DirEntry>, truncated: bool }

pub fn list_dir(path: &Path) -> Result<Listing, String> {
    if !path.is_dir() { return Err(format!("Pasta não encontrada: {}", path.display())); }
    let mut entries: Vec<DirEntry> = fs::read_dir(path).map_err(err)?.filter_map(Result::ok).map(|entry| {
        let meta = entry.metadata().ok();
        let dir = meta.as_ref().is_some_and(|m| m.is_dir());
        DirEntry { name: entry.file_name().to_string_lossy().into_owned(), path: entry.path().display().to_string(), kind: if dir { "pasta" } else { "arquivo" }, size: if dir { None } else { meta.map(|m| m.len()) } }
    }).collect();
    entries.sort_by(|a, b| a.kind.cmp(b.kind).reverse().then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    let truncated = entries.len() > MAX_LIST;
    entries.truncate(MAX_LIST);
    Ok(Listing { path: path.display().to_string(), entries, truncated })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileText { path: String, content: String, start_line: usize, end_line: usize, total_lines: usize, truncated: bool }

/// Lê um arquivo de texto, opcionalmente a partir de uma linha (1 = primeira) e com limite de linhas.
pub fn read_text(path: &Path, start: Option<usize>, limit: Option<usize>) -> Result<FileText, String> {
    let meta = fs::metadata(path).map_err(|_| format!("Arquivo não encontrado: {}", path.display()))?;
    if meta.is_dir() { return Err("Isso é uma pasta; use a listagem.".into()); }
    if meta.len() > MAX_READ_BYTES { return Err(format!("Arquivo grande demais para ler ({} MB).", meta.len() / 1_000_000)); }
    let bytes = fs::read(path).map_err(err)?;
    if is_binary(&bytes) { return Err("Arquivo binário: não dá para ler como texto.".into()); }
    let text = String::from_utf8_lossy(&bytes);
    let lines: Vec<&str> = text.lines().collect();
    let first = start.unwrap_or(1).max(1);
    let mut content = String::new();
    let mut end = first.saturating_sub(1);
    let mut truncated = false;
    for (index, line) in lines.iter().enumerate().skip(first - 1).take(limit.unwrap_or(usize::MAX)) {
        if content.len() + line.len() > MAX_READ_CHARS { truncated = true; break; }
        content.push_str(line);
        content.push('\n');
        end = index + 1;
    }
    Ok(FileText { path: path.display().to_string(), content, start_line: first, end_line: end, total_lines: lines.len(), truncated: truncated || end < lines.len() })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit { path: String, line: usize, text: String }

/// Procura texto (sem diferenciar maiúsculas) nos arquivos e nos nomes, pulando pastas geradas.
pub fn search(root: &Path, query: &str, extension: Option<&str>) -> Result<Vec<SearchHit>, String> {
    let needle = query.trim().to_lowercase();
    if needle.chars().count() < 2 { return Err("Busque ao menos 2 caracteres.".into()); }
    if !root.is_dir() { return Err(format!("Pasta não encontrada: {}", root.display())); }
    let extension = extension.map(|ext| ext.trim().trim_start_matches('.').to_lowercase()).filter(|ext| !ext.is_empty());
    let mut hits = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    let mut seen = 0usize;
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else { continue };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(meta) = entry.metadata() else { continue };
            if meta.is_dir() {
                if !SKIP_DIRS.iter().any(|skip| skip.eq_ignore_ascii_case(&name)) { stack.push(path); }
                continue;
            }
            seen += 1;
            if seen > MAX_SEARCH_FILES || hits.len() >= MAX_SEARCH_RESULTS { return Ok(hits); }
            if let Some(ext) = &extension {
                if path.extension().map(|e| e.to_string_lossy().to_lowercase()) != Some(ext.clone()) { continue; }
            }
            if name.to_lowercase().contains(&needle) {
                hits.push(SearchHit { path: path.display().to_string(), line: 0, text: "(nome do arquivo)".into() });
            }
            if meta.len() > MAX_SEARCH_FILE_BYTES { continue; }
            let Ok(bytes) = fs::read(&path) else { continue };
            if is_binary(&bytes) { continue; }
            for (index, line) in String::from_utf8_lossy(&bytes).lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    hits.push(SearchHit { path: path.display().to_string(), line: index + 1, text: line.trim().chars().take(200).collect() });
                    if hits.len() >= MAX_SEARCH_RESULTS { return Ok(hits); }
                }
            }
        }
    }
    Ok(hits)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult { path: String, created: bool, backup: Option<String>, replacements: usize }

pub fn write_text(path: &Path, content: &str, overwrite: bool, backups: &Path) -> Result<WriteResult, String> {
    if content.chars().count() > MAX_WRITE_CHARS { return Err("Conteúdo grande demais para gravar de uma vez.".into()); }
    if path.is_dir() { return Err("Já existe uma pasta com esse nome.".into()); }
    let exists = path.exists();
    if exists && !overwrite { return Err("O arquivo já existe. Para substituir, use overwrite=true (um backup é feito antes).".into()); }
    let saved = if exists { Some(backup(backups, path)?) } else { None };
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(err)?; }
    fs::write(path, content).map_err(err)?;
    Ok(WriteResult { path: path.display().to_string(), created: !exists, backup: saved.map(|p| p.display().to_string()), replacements: 0 })
}

/// Troca um trecho exato. Se aparecer mais de uma vez, exige replace_all para não alterar o lugar errado.
pub fn edit_text(path: &Path, old: &str, new: &str, replace_all: bool, backups: &Path) -> Result<WriteResult, String> {
    if old.is_empty() { return Err("Informe o trecho a substituir.".into()); }
    let bytes = fs::read(path).map_err(|_| format!("Arquivo não encontrado: {}", path.display()))?;
    if is_binary(&bytes) { return Err("Arquivo binário: não dá para editar como texto.".into()); }
    let text = String::from_utf8(bytes).map_err(|_| "O arquivo não está em UTF-8; edite com write_file.".to_string())?;
    // Arquivos com CRLF: o modelo costuma mandar o trecho com LF.
    let crlf = text.contains("\r\n");
    let (old, new) = if crlf && !old.contains("\r\n") { (old.replace('\n', "\r\n"), new.replace("\r\n", "\n").replace('\n', "\r\n")) } else { (old.to_string(), new.to_string()) };
    let count = text.matches(&old).count();
    if count == 0 { return Err("Trecho não encontrado. Leia o arquivo de novo e copie o trecho exato.".into()); }
    if count > 1 && !replace_all { return Err(format!("O trecho aparece {count} vezes. Inclua mais contexto para ficar único, ou use replace_all=true.")); }
    let saved = backup(backups, path)?;
    let updated = if replace_all { text.replace(&old, &new) } else { text.replacen(&old, &new, 1) };
    fs::write(path, updated).map_err(err)?;
    Ok(WriteResult { path: path.display().to_string(), created: false, backup: Some(saved.display().to_string()), replacements: if replace_all { count } else { 1 } })
}

// ---------- PowerShell ----------

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { TABLE[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[n as usize & 63] as char } else { '=' });
    }
    out
}

/// -EncodedCommand (UTF-16LE em base64) evita qualquer problema de aspas ao passar o comando.
fn encoded(script: &str) -> String {
    let wrapped = format!(
        "[Console]::OutputEncoding=[Text.Encoding]::UTF8; $OutputEncoding=[Text.Encoding]::UTF8; $ProgressPreference='SilentlyContinue'\n{script}\nif ($null -ne $LASTEXITCODE) {{ exit $LASTEXITCODE }}"
    );
    base64(&wrapped.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<u8>>())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult { exit_code: Option<i32>, stdout: String, stderr: String, timed_out: bool, cancelled: bool, duration_ms: u128, cwd: String }

/// Comandos rodando agora (PID) e os que o Senhor mandou parar (ESC no AI Core).
static RUNNING: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static CANCELLED: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn kill_tree(pid: u32) {
    let mut taskkill = Command::new("taskkill");
    taskkill.args(["/T", "/F", "/PID", &pid.to_string()]).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        taskkill.creation_flags(0x0800_0000);
    }
    let _ = taskkill.status();
}

/// Para todos os comandos em andamento, com tudo o que eles abriram. Devolve quantos parou.
pub fn cancel_running() -> usize {
    let pids: Vec<u32> = RUNNING.lock().map(|running| running.clone()).unwrap_or_default();
    if let Ok(mut cancelled) = CANCELLED.lock() { cancelled.extend(&pids); }
    for pid in &pids { kill_tree(*pid); }
    pids.len()
}

#[tauri::command]
pub async fn cancel_commands(app: AppHandle) -> Result<usize, String> {
    let stopped = blocking(|| Ok(cancel_running())).await?;
    if stopped > 0 { log(&app, "comando interrompido", &format!("{stopped} comando(s)"), "parado pelo Senhor (ESC)", true); }
    Ok(stopped)
}

pub fn run_powershell(script: &str, cwd: &Path, timeout: Duration) -> Result<CommandResult, String> {
    let mut command = Command::new("powershell.exe");
    command
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", &encoded(script)])
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: sem janela de console piscando
    }
    let started = Instant::now();
    let mut child = command.spawn().map_err(|error| format!("Não foi possível iniciar o PowerShell: {error}"))?;
    let pid = child.id();
    if let Ok(mut running) = RUNNING.lock() { running.push(pid); }
    let mut stdout = child.stdout.take().ok_or("Saída indisponível")?;
    let mut stderr = child.stderr.take().ok_or("Saída de erro indisponível")?;
    let out_reader = std::thread::spawn(move || { let mut buffer = Vec::new(); let _ = stdout.read_to_end(&mut buffer); buffer });
    let err_reader = std::thread::spawn(move || { let mut buffer = Vec::new(); let _ = stderr.read_to_end(&mut buffer); buffer });
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(err)? { break Some(status); }
        if started.elapsed() > timeout {
            timed_out = true;
            // Mata a árvore inteira (npm, node...), não só o PowerShell.
            kill_tree(pid);
            let _ = child.kill();
            break child.wait().ok();
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    if let Ok(mut running) = RUNNING.lock() { running.retain(|item| *item != pid); }
    let cancelled = CANCELLED.lock().map(|mut list| { let hit = list.contains(&pid); list.retain(|item| *item != pid); hit }).unwrap_or(false);
    let stdout = String::from_utf8_lossy(&out_reader.join().unwrap_or_default()).into_owned();
    let stderr = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).into_owned();
    Ok(CommandResult { exit_code: status.and_then(|s| s.code()), stdout: tail(&stdout, MAX_OUTPUT_CHARS), stderr: tail(&stderr, MAX_OUTPUT_CHARS / 2), timed_out, cancelled, duration_ms: started.elapsed().as_millis(), cwd: cwd.display().to_string() })
}

fn quote(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "''"))
}

// ---------- abrir no navegador ----------

/// O que abrir: um endereço web (aceita "localhost:5173" sem http) ou um arquivo/pasta do disco C.
#[derive(Debug, PartialEq)]
pub enum OpenTarget { Url(String), Path(PathBuf) }

pub fn open_target(target: &str) -> Result<OpenTarget, String> {
    let target = target.trim().trim_matches('"');
    if target.is_empty() { return Err("Diga o que abrir (arquivo, pasta ou endereço).".into()); }
    let lower = target.to_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return if crate::commands::is_web_url(target) { Ok(OpenTarget::Url(target.into())) } else { Err("Endereço inválido".into()) };
    }
    if lower.starts_with("localhost") || lower.starts_with("127.0.0.1") {
        return Ok(OpenTarget::Url(format!("http://{target}")));
    }
    let path = resolve(target)?;
    if !path.exists() { return Err(format!("Não encontrado: {}", path.display())); }
    Ok(OpenTarget::Path(path))
}

/// Navegadores conhecidos pelo nome do executável no Windows.
pub fn browser_exe(browser: &str) -> Option<&'static str> {
    match browser.trim().to_lowercase().as_str() {
        "chrome" | "google chrome" => Some("chrome"),
        "edge" | "microsoft edge" => Some("msedge"),
        "firefox" => Some("firefox"),
        "brave" => Some("brave"),
        "opera" => Some("opera"),
        _ => None,
    }
}

/// Abre um site, o localhost do projeto ou um arquivo (ex.: index.html) no navegador.
#[tauri::command]
pub async fn open_in_browser(app: AppHandle, target: String, browser: Option<String>) -> Result<String, String> {
    use tauri_plugin_opener::OpenerExt;
    let what = open_target(&target)?;
    let shown = match &what { OpenTarget::Url(url) => url.clone(), OpenTarget::Path(path) => path.display().to_string() };
    let result = match browser.as_deref().and_then(browser_exe) {
        Some(exe) => {
            let argument = match &what { OpenTarget::Url(url) => url.clone(), OpenTarget::Path(path) => path.display().to_string() };
            let script = format!("Start-Process {exe} -ArgumentList '\"{}\"'", argument.replace('\'', "''"));
            let cwd = PathBuf::from("C:\\");
            match blocking(move || run_powershell(&script, &cwd, Duration::from_secs(30))).await {
                Ok(done) if done.exit_code == Some(0) => Ok(format!("Aberto no {exe}: {shown}")),
                Ok(done) => Err(format!("Não consegui abrir no {exe}. {}", done.stderr.trim())),
                Err(error) => Err(error),
            }
        }
        None => match &what {
            OpenTarget::Url(url) => app.opener().open_url(url, None::<&str>).map(|_| format!("Aberto no navegador padrão: {shown}")).map_err(|_| "Não foi possível abrir o navegador".to_string()),
            OpenTarget::Path(path) => app.opener().open_path(path.display().to_string(), None::<&str>).map(|_| format!("Aberto: {shown}")).map_err(|_| "Não foi possível abrir o arquivo".to_string()),
        },
    };
    log(&app, "aberto no navegador", &shown, result.as_ref().err().map(String::as_str).unwrap_or(""), result.is_ok());
    result
}

// ---------- comandos Tauri ----------

async fn blocking<T: Send + 'static>(job: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(job).await.map_err(|_| "A operação foi interrompida".to_string())?
}

#[tauri::command]
pub async fn fs_list(path: String) -> Result<Listing, String> {
    let path = resolve(&path)?;
    blocking(move || list_dir(&path)).await
}

#[tauri::command]
pub async fn fs_read(path: String, start_line: Option<usize>, max_lines: Option<usize>) -> Result<FileText, String> {
    let path = resolve(&path)?;
    blocking(move || read_text(&path, start_line, max_lines)).await
}

#[tauri::command]
pub async fn fs_search(root: String, query: String, extension: Option<String>) -> Result<Vec<SearchHit>, String> {
    let root = resolve(&root)?;
    blocking(move || search(&root, &query, extension.as_deref())).await
}

#[tauri::command]
pub async fn fs_write(app: AppHandle, path: String, content: String, overwrite: Option<bool>) -> Result<WriteResult, String> {
    let path = resolve(&path)?;
    let backups = backups_root(&app)?;
    let target = path.display().to_string();
    let result = blocking(move || write_text(&path, &content, overwrite.unwrap_or(false), &backups)).await;
    log(&app, "arquivo gravado", &target, &match &result { Ok(done) => if done.created { "criado".into() } else { format!("sobrescrito; backup: {}", done.backup.clone().unwrap_or_default()) }, Err(error) => error.clone() }, result.is_ok());
    result
}

#[tauri::command]
pub async fn fs_edit(app: AppHandle, path: String, old_text: String, new_text: String, replace_all: Option<bool>) -> Result<WriteResult, String> {
    let path = resolve(&path)?;
    let backups = backups_root(&app)?;
    let target = path.display().to_string();
    let result = blocking(move || edit_text(&path, &old_text, &new_text, replace_all.unwrap_or(false), &backups)).await;
    log(&app, "arquivo editado", &target, &match &result { Ok(done) => format!("{} troca(s); backup: {}", done.replacements, done.backup.clone().unwrap_or_default()), Err(error) => error.clone() }, result.is_ok());
    result
}

#[tauri::command]
pub async fn fs_mkdir(app: AppHandle, path: String) -> Result<String, String> {
    let path = resolve(&path)?;
    let target = path.display().to_string();
    let result = fs::create_dir_all(&path).map(|_| target.clone()).map_err(err);
    log(&app, "pasta criada", &target, result.as_ref().err().map(String::as_str).unwrap_or(""), result.is_ok());
    result
}

/// Manda para a Lixeira (recuperável). A interface só chama depois da permissão do Senhor.
#[tauri::command]
pub async fn fs_delete(app: AppHandle, path: String) -> Result<String, String> {
    let path = resolve(&path)?;
    if is_drive_root(&path) { return Err("A raiz do disco não pode ser apagada.".into()); }
    if !path.exists() { return Err(format!("Não encontrado: {}", path.display())); }
    let target = path.display().to_string();
    let method = if path.is_dir() { "DeleteDirectory" } else { "DeleteFile" };
    let script = format!("Add-Type -AssemblyName Microsoft.VisualBasic; [Microsoft.VisualBasic.FileIO.FileSystem]::{method}({}, 'OnlyErrorDialogs', 'SendToRecycleBin')", quote(&path));
    let cwd = PathBuf::from("C:\\");
    let outcome = blocking(move || run_powershell(&script, &cwd, Duration::from_secs(120))).await;
    let result = match outcome {
        Ok(done) if done.exit_code == Some(0) && !path.exists() => Ok(format!("{target} foi para a Lixeira.")),
        Ok(done) => Err(format!("Não foi possível mandar para a Lixeira. {}", done.stderr.trim())),
        Err(error) => Err(error),
    };
    log(&app, "enviado para a Lixeira", &target, result.as_ref().err().map(String::as_str).unwrap_or(""), result.is_ok());
    result
}

#[tauri::command]
pub async fn run_command(app: AppHandle, command: String, cwd: String, timeout_seconds: Option<u64>) -> Result<CommandResult, String> {
    let script = command.trim().to_string();
    if script.is_empty() { return Err("Comando vazio.".into()); }
    if script.chars().count() > 8_000 { return Err("Comando longo demais.".into()); }
    let cwd = resolve(&cwd)?;
    if !cwd.is_dir() { return Err(format!("Pasta de trabalho não encontrada: {}", cwd.display())); }
    let timeout = Duration::from_secs(timeout_seconds.unwrap_or(DEFAULT_TIMEOUT_SECONDS).clamp(5, MAX_TIMEOUT_SECONDS));
    let target = format!("{} › {}", cwd.display(), script.chars().take(300).collect::<String>());
    let result = blocking(move || run_powershell(&script, &cwd, timeout)).await;
    let detail = match &result {
        Ok(done) if done.cancelled => "parado pelo Senhor".to_string(),
        Ok(done) if done.timed_out => format!("tempo esgotado após {} s", timeout.as_secs()),
        Ok(done) => format!("código {}", done.exit_code.map(|c| c.to_string()).unwrap_or_else(|| "?".into())),
        Err(error) => error.clone(),
    };
    log(&app, "comando", &target, &detail, matches!(&result, Ok(done) if done.exit_code == Some(0)));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sandbox(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("azriel-ws-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn only_absolute_paths_on_drive_c_are_accepted() {
        assert_eq!(resolve("C:\\Projetos\\azriel").unwrap(), PathBuf::from("C:\\Projetos\\azriel"));
        assert_eq!(resolve("c:/Projetos/./azriel/src/..").unwrap(), PathBuf::from("C:\\Projetos\\azriel"));
        assert_eq!(resolve("C:\\..\\..\\Windows").unwrap(), PathBuf::from("C:\\Windows"), "\"..\" não sai do disco");
        for bad in ["D:\\dados", "Projetos\\azriel", "\\\\servidor\\pasta", "C:", "C:\\a\\b:stream", "C:\\a*", ""] {
            assert!(resolve(bad).is_err(), "deveria recusar {bad}");
        }
        assert!(is_drive_root(&resolve("C:\\").unwrap()));
        assert!(!is_drive_root(&resolve("C:\\Projetos").unwrap()));
    }

    #[test]
    fn writing_backs_up_before_overwriting() {
        let dir = sandbox("write");
        let backups = dir.join("backups");
        let file = dir.join("app").join("main.ts");
        assert!(write_text(&file, "v1", false, &backups).unwrap().created);
        assert!(write_text(&file, "v2", false, &backups).is_err(), "sem overwrite não substitui");
        let done = write_text(&file, "v2", true, &backups).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "v2");
        assert_eq!(fs::read_to_string(done.backup.unwrap()).unwrap(), "v1");
    }

    #[test]
    fn edits_need_a_unique_snippet_and_keep_crlf() {
        let dir = sandbox("edit");
        let backups = dir.join("backups");
        let file = dir.join("a.txt");
        fs::write(&file, "um\r\ndois\r\ndois\r\n").unwrap();
        assert!(edit_text(&file, "tres", "x", false, &backups).unwrap_err().contains("não encontrado"));
        assert!(edit_text(&file, "dois", "2", false, &backups).unwrap_err().contains("2 vezes"));
        edit_text(&file, "um\ndois", "UM\nDOIS", false, &backups).unwrap();
        assert_eq!(fs::read_to_string(&file).unwrap(), "UM\r\nDOIS\r\ndois\r\n");
        assert_eq!(edit_text(&file, "dois", "2", true, &backups).unwrap().replacements, 1);
    }

    #[test]
    fn reads_by_line_range_and_refuses_binaries() {
        let dir = sandbox("read");
        let file = dir.join("b.txt");
        fs::write(&file, "1\n2\n3\n4\n").unwrap();
        let part = read_text(&file, Some(2), Some(2)).unwrap();
        assert_eq!((part.content.as_str(), part.start_line, part.end_line, part.total_lines, part.truncated), ("2\n3\n", 2, 3, 4, true));
        fs::write(dir.join("c.bin"), [0u8, 1, 2]).unwrap();
        assert!(read_text(&dir.join("c.bin"), None, None).is_err());
    }

    #[test]
    fn search_finds_lines_and_names_and_skips_generated_folders() {
        let dir = sandbox("search");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("node_modules")).unwrap();
        fs::write(dir.join("src").join("app.ts"), "const Azriel = 1;\n").unwrap();
        fs::write(dir.join("node_modules").join("lib.ts"), "Azriel\n").unwrap();
        fs::write(dir.join("azriel.md"), "nada\n").unwrap();
        let hits = search(&dir, "azriel", None).unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().any(|hit| hit.line == 1 && hit.path.ends_with("app.ts")));
        assert!(hits.iter().all(|hit| !hit.path.contains("node_modules")));
        assert_eq!(search(&dir, "azriel", Some(".ts")).unwrap().len(), 1);
    }

    #[test]
    fn open_targets_accept_sites_localhost_and_existing_files() {
        assert_eq!(open_target("https://example.com").unwrap(), OpenTarget::Url("https://example.com".into()));
        assert_eq!(open_target("localhost:5173").unwrap(), OpenTarget::Url("http://localhost:5173".into()));
        let dir = sandbox("open");
        let file = dir.join("index.html");
        fs::write(&file, "<p>oi</p>").unwrap();
        assert_eq!(open_target(&format!("\"{}\"", file.display())).unwrap(), OpenTarget::Path(file.clone()));
        assert!(open_target("C:\\nao\\existe.html").is_err());
        assert!(open_target("javascript:alert(1)").is_err());
        assert!(open_target("").is_err());
        assert_eq!(browser_exe("Chrome"), Some("chrome"));
        assert_eq!(browser_exe("edge"), Some("msedge"));
        assert_eq!(browser_exe("padrão"), None);
    }

    #[test]
    fn base64_matches_the_standard() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
    }

    #[cfg(windows)]
    #[test]
    fn powershell_runs_with_exit_code_output_and_timeout() {
        let dir = sandbox("ps");
        let done = run_powershell("Write-Output 'olá \"Senhor\"'; cmd /c exit 3", &dir, Duration::from_secs(60)).unwrap();
        assert_eq!(done.exit_code, Some(3));
        assert!(done.stdout.contains("olá \"Senhor\""), "saída: {}", done.stdout);
        let slow = run_powershell("Start-Sleep -Seconds 30", &dir, Duration::from_secs(2)).unwrap();
        assert!(slow.timed_out);
    }

    #[cfg(windows)]
    #[test]
    fn cancel_stops_a_running_command() {
        let dir = sandbox("cancel");
        let worker = std::thread::spawn(move || run_powershell("Start-Sleep -Seconds 60", &dir, Duration::from_secs(120)).unwrap());
        let started = Instant::now();
        while RUNNING.lock().unwrap().is_empty() && started.elapsed() < Duration::from_secs(20) { std::thread::sleep(Duration::from_millis(50)); }
        assert!(cancel_running() >= 1);
        let done = worker.join().unwrap();
        assert!(done.cancelled && !done.timed_out);
        assert!(started.elapsed() < Duration::from_secs(30), "parou na hora, não esperou os 60 s");
    }
}
