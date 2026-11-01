//! Spotify pela Web API oficial: login PKCE (sem segredo), tokens no SQLite local e
//! renovação automática. O som sai do app do Spotify (ou outro dispositivo Connect);
//! o Azriel é a tela e o controle.

use crate::database::DatabaseState;
use reqwest::{Client, Method};
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};

const PORT: u16 = 8789;
pub const REDIRECT_URI: &str = "http://127.0.0.1:8789/callback";
// `streaming` + e-mail/perfil permitem que o próprio Azriel seja um dispositivo (Web Playback SDK).
const SCOPES: &str = "user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private playlist-read-collaborative user-library-read streaming user-read-email user-read-private";
const API: &str = "https://api.spotify.com/v1";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotifyStatus {
    client_id_configured: bool,
    connected: bool,
    redirect_uri: &'static str,
}

struct Tokens {
    client_id: String,
    access: String,
    refresh: String,
    expires_at: i64,
}

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

fn load(connection: &Connection) -> Result<Tokens, String> {
    connection
        .query_row("SELECT client_id,access_token,refresh_token,expires_at FROM spotify_settings WHERE id=1", [], |row| {
            Ok(Tokens { client_id: row.get(0)?, access: row.get(1)?, refresh: row.get(2)?, expires_at: row.get(3)? })
        })
        .map_err(|error| error.to_string())
}

fn save_tokens(connection: &Connection, access: &str, refresh: &str, expires_in: i64) -> Result<(), String> {
    let expires_at = chrono::Utc::now().timestamp() + expires_in - 60;
    connection
        .execute(
            "UPDATE spotify_settings SET access_token=?1, refresh_token=CASE WHEN ?2='' THEN refresh_token ELSE ?2 END, expires_at=?3, updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![access, refresh, expires_at],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

// ---------------------------------------------------------------- utilidades puras

fn base64url(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..=chunk.len() {
            out.push(TABLE[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

/// PKCE: code_challenge = BASE64URL(SHA256(verifier)), sem padding.
pub fn code_challenge(verifier: &str) -> String {
    base64url(&Sha256::digest(verifier.as_bytes()))
}

fn random_token(bytes: usize) -> Result<String, String> {
    let mut buffer = vec![0u8; bytes];
    getrandom::fill(&mut buffer).map_err(|_| "Não foi possível gerar o código de segurança do login".to_string())?;
    Ok(base64url(&buffer))
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn form(pairs: &[(&str, &str)]) -> String {
    pairs.iter().map(|(key, value)| format!("{key}={}", encode(value))).collect::<Vec<_>>().join("&")
}

pub fn authorize_url(client_id: &str, challenge: &str, state: &str) -> String {
    format!(
        "https://accounts.spotify.com/authorize?{}",
        form(&[("client_id", client_id), ("response_type", "code"), ("redirect_uri", REDIRECT_URI), ("code_challenge_method", "S256"), ("code_challenge", challenge), ("scope", SCOPES), ("state", state)])
    )
}

/// Lê `code`/`state`/`error` da primeira linha da requisição HTTP do retorno do login.
pub fn parse_callback(request_line: &str) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let target = request_line.split_whitespace().nth(1)?;
    let query = target.strip_prefix("/callback?")?;
    let mut code = None;
    let mut state = None;
    let mut error = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = Some(value.to_string());
        match key {
            "code" => code = value,
            "state" => state = value,
            "error" => error = value,
            _ => {}
        }
    }
    Some((code, state, error))
}

/// Caminho da API para listar as faixas de um item. Playlist e álbum vêm de 50 em 50; o artista usa a
/// busca (o top-tracks foi removido do modo de desenvolvimento) em páginas de 10.
pub fn tracks_path(uri: &str, name: Option<&str>, offset: u32) -> Result<String, String> {
    let mut parts = uri.split(':');
    let (_, kind, id) = (parts.next(), parts.next(), parts.next());
    let (Some(kind), Some(id)) = (kind, id) else {
        return Err("Item do Spotify inválido".into());
    };
    match kind {
        "playlist" => Ok(format!("/playlists/{id}/items?limit=50&offset={}&additional_types=track", offset.min(9_950))),
        "album" => Ok(format!("/albums/{id}/tracks?limit=50&offset={}", offset.min(9_950))),
        "artist" => {
            let name = name
                .map(|value| value.replace('"', "").trim().to_string())
                .filter(|value| !value.is_empty() && value.chars().count() <= 100)
                .ok_or("Nome do artista inválido")?;
            Ok(format!("/search?{}", form(&[("q", &format!("artist:\"{name}\"")), ("type", "track"), ("limit", "10"), ("offset", &offset.min(990).to_string())])))
        }
        _ => Err("Só playlists, álbuns e artistas têm lista de faixas".into()),
    }
}

pub fn valid_device(id: &str) -> bool {
    !id.is_empty() && id.len() <= 80 && id.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Tipo de busca → (tipos da API, itens por tipo). A API limita a 10 por tipo; na busca geral
/// vêm 8 de cada para a tela não ficar enorme, e as buscas filtradas paginam por offset.
pub fn search_params(kind: &str) -> (&'static str, u32) {
    match kind {
        "track" => ("track", 10),
        "playlist" => ("playlist", 10),
        "album" => ("album", 10),
        "artist" => ("artist", 10),
        _ => ("track,playlist,artist,album", 8),
    }
}

/// Corpo do play: uma música dentro do contexto (álbum/playlist) continua tocando o resto dele;
/// sem contexto toca só a faixa; qualquer outro item é tocado como contexto.
pub fn play_body(uri: &str, context: Option<&str>) -> Value {
    let is_track = uri.starts_with("spotify:track:");
    let context = context.filter(|c| valid_uri(c) && !c.starts_with("spotify:track:"));
    match (is_track, context) {
        (true, Some(context)) => json!({"context_uri": context, "offset": {"uri": uri}}),
        (true, None) => json!({"uris": [uri]}),
        (false, _) => json!({"context_uri": uri}),
    }
}

pub fn valid_uri(uri: &str) -> bool {
    let mut parts = uri.split(':');
    matches!((parts.next(), parts.next(), parts.next(), parts.next()), (Some("spotify"), Some("track" | "playlist" | "album" | "artist"), Some(id), None) if !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric()))
}

// ---------------------------------------------------------------- login

fn wait_for_callback(expected_state: &str) -> Result<String, String> {
    let listener = TcpListener::bind(("127.0.0.1", PORT)).map_err(|_| format!("A porta {PORT} está em uso; feche o outro login do Spotify e tente de novo"))?;
    listener.set_nonblocking(true).map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        if Instant::now() > deadline {
            return Err("O login do Spotify não foi concluído em 3 minutos".into());
        }
        let (mut stream, _) = match listener.accept() {
            Ok(connection) => connection,
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(150));
                continue;
            }
            Err(error) => return Err(error.to_string()),
        };
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut buffer = [0u8; 4096];
        let read = stream.read(&mut buffer).unwrap_or(0);
        let request = String::from_utf8_lossy(&buffer[..read]);
        let Some((code, state, error)) = parse_callback(request.lines().next().unwrap_or("")) else {
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            continue;
        };
        let ok = error.is_none() && state.as_deref() == Some(expected_state) && code.is_some();
        let page = if ok { "Azriel conectado ao Spotify. Pode fechar esta aba." } else { "O login do Spotify não foi autorizado. Volte ao Azriel e tente de novo." };
        let body = format!("<!doctype html><meta charset=utf-8><body style=\"background:#010508;color:#dff9ff;font:16px sans-serif;display:grid;place-items:center;height:100vh\"><p>{page}</p>");
        let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes());
        if let Some(error) = error {
            return Err(format!("O Spotify recusou o login: {error}"));
        }
        if state.as_deref() != Some(expected_state) {
            return Err("Resposta de login inválida (state não confere)".into());
        }
        return code.ok_or_else(|| "O Spotify não enviou o código de login".to_string());
    }
}

async fn token_request(body: String) -> Result<Value, String> {
    let response = Client::new()
        .post(TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|_| "Não foi possível falar com o Spotify".to_string())?;
    let status = response.status();
    let payload: Value = response.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        let detail = payload["error_description"].as_str().or(payload["error"].as_str()).unwrap_or("erro desconhecido");
        return Err(format!("O Spotify recusou o token ({}): {detail}", status.as_u16()));
    }
    Ok(payload)
}

#[tauri::command]
pub fn spotify_status(state: State<'_, DatabaseState>) -> Result<SpotifyStatus, String> {
    let tokens = load(&*lock(&state)?)?;
    Ok(SpotifyStatus { client_id_configured: !tokens.client_id.is_empty(), connected: !tokens.refresh.is_empty(), redirect_uri: REDIRECT_URI })
}

#[tauri::command]
pub fn spotify_set_client_id(state: State<'_, DatabaseState>, client_id: String) -> Result<SpotifyStatus, String> {
    let client_id = client_id.trim();
    if client_id.len() != 32 || !client_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("O Client ID do Spotify tem 32 caracteres (0-9, a-f)".into());
    }
    lock(&state)?
        .execute("UPDATE spotify_settings SET client_id=?1, access_token='', refresh_token='', expires_at=0 WHERE id=1", [client_id])
        .map_err(|error| error.to_string())?;
    spotify_status(state)
}

#[tauri::command]
pub async fn spotify_connect(app: AppHandle, state: State<'_, DatabaseState>) -> Result<SpotifyStatus, String> {
    let client_id = load(&*lock(&state)?)?.client_id;
    if client_id.is_empty() {
        return Err("Cadastre o Client ID do Spotify primeiro".into());
    }
    let verifier = random_token(48)?;
    let csrf = random_token(16)?;
    let url = authorize_url(&client_id, &code_challenge(&verifier), &csrf);
    let expected = csrf.clone();
    let waiter = tauri::async_runtime::spawn_blocking(move || wait_for_callback(&expected));
    {
        use tauri_plugin_opener::OpenerExt;
        app.opener().open_url(url, None::<&str>).map_err(|_| "Não foi possível abrir o navegador para o login".to_string())?;
    }
    let code = waiter.await.map_err(|_| "Falha no login do Spotify".to_string())??;
    let payload = token_request(form(&[("grant_type", "authorization_code"), ("code", &code), ("redirect_uri", REDIRECT_URI), ("client_id", &client_id), ("code_verifier", &verifier)])).await?;
    save_tokens(
        &*lock(&state)?,
        payload["access_token"].as_str().unwrap_or_default(),
        payload["refresh_token"].as_str().unwrap_or_default(),
        payload["expires_in"].as_i64().unwrap_or(3600),
    )?;
    spotify_status(state)
}

#[tauri::command]
pub fn spotify_disconnect(state: State<'_, DatabaseState>) -> Result<SpotifyStatus, String> {
    lock(&state)?
        .execute("UPDATE spotify_settings SET access_token='', refresh_token='', expires_at=0 WHERE id=1", [])
        .map_err(|error| error.to_string())?;
    spotify_status(state)
}

// ---------------------------------------------------------------- chamadas da API

async fn access_token(app: &AppHandle, force_refresh: bool) -> Result<String, String> {
    let tokens = {
        let database = app.state::<DatabaseState>();
        let connection = database.connection.lock().map_err(|_| "O banco de dados está indisponível".to_string())?;
        load(&connection)?
    };
    if tokens.refresh.is_empty() {
        return Err("Conecte o Azriel ao Spotify primeiro".into());
    }
    if !force_refresh && !tokens.access.is_empty() && tokens.expires_at > chrono::Utc::now().timestamp() {
        return Ok(tokens.access);
    }
    let payload = token_request(form(&[("grant_type", "refresh_token"), ("refresh_token", &tokens.refresh), ("client_id", &tokens.client_id)])).await?;
    let access = payload["access_token"].as_str().unwrap_or_default().to_string();
    let database = app.state::<DatabaseState>();
    let connection = database.connection.lock().map_err(|_| "O banco de dados está indisponível".to_string())?;
    save_tokens(&connection, &access, payload["refresh_token"].as_str().unwrap_or_default(), payload["expires_in"].as_i64().unwrap_or(3600))?;
    Ok(access)
}

fn api_error(status: u16, payload: &Value) -> String {
    let reason = payload["error"]["reason"].as_str().unwrap_or_default();
    let message = payload["error"]["message"].as_str().unwrap_or("erro desconhecido");
    match (status, reason) {
        (404, "NO_ACTIVE_DEVICE") | (404, _) if message.to_lowercase().contains("device") => "Nenhum dispositivo do Spotify ativo. Abra o app do Spotify.".into(),
        (403, "PREMIUM_REQUIRED") => "Esse comando exige Spotify Premium.".into(),
        (403, _) => format!("O Spotify negou o acesso (403). Confira se seu e-mail está na lista de usuários do app. {message}"),
        (429, _) => "Muitas chamadas ao Spotify.".into(),
        _ => format!("Spotify respondeu {status}: {message}"),
    }
}

/// Instante (unix) até o qual o Spotify pediu para não chamar de novo (cabeçalho Retry-After do 429).
static COOLDOWN_UNTIL: AtomicI64 = AtomicI64::new(0);

pub fn cooldown_message(remaining: i64) -> String {
    if remaining >= 90 {
        format!("O Spotify limitou as chamadas do Azriel; libera em {} min.", (remaining + 59) / 60)
    } else {
        format!("O Spotify limitou as chamadas do Azriel; libera em {remaining}s.")
    }
}

fn cooldown_remaining() -> i64 {
    (COOLDOWN_UNTIL.load(Ordering::Relaxed) - chrono::Utc::now().timestamp()).max(0)
}

async fn api(app: &AppHandle, method: Method, path: &str, body: Option<Value>) -> Result<Option<Value>, String> {
    let wait = cooldown_remaining();
    if wait > 0 {
        return Err(cooldown_message(wait));
    }
    let mut token = access_token(app, false).await?;
    for attempt in 0..2 {
        let mut request = Client::new().request(method.clone(), format!("{API}{path}")).bearer_auth(&token).timeout(Duration::from_secs(15));
        request = match &body {
            Some(body) => request.json(body),
            None if method != Method::GET => request.header("Content-Length", "0"),
            None => request,
        };
        let response = request.send().await.map_err(|_| "Não foi possível falar com o Spotify".to_string())?;
        let status = response.status().as_u16();
        if status == 401 && attempt == 0 {
            token = access_token(app, true).await?;
            continue;
        }
        if status == 429 {
            let seconds = response.headers().get("retry-after").and_then(|value| value.to_str().ok()).and_then(|value| value.trim().parse::<i64>().ok()).unwrap_or(30).clamp(1, 86_400);
            COOLDOWN_UNTIL.store(chrono::Utc::now().timestamp() + seconds, Ordering::Relaxed);
            return Err(cooldown_message(seconds));
        }
        let text = response.text().await.unwrap_or_default();
        let payload: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        if (200..300).contains(&status) {
            return Ok(if text.trim().is_empty() { None } else { Some(payload) });
        }
        return Err(api_error(status, &payload));
    }
    Err("Sessão do Spotify expirada; conecte de novo".into())
}

/// Token curto para o Web Playback SDK, que roda no WebView e precisa dele para tocar o áudio.
#[tauri::command]
pub async fn spotify_token(app: AppHandle) -> Result<String, String> {
    access_token(&app, false).await
}

/// Move a reprodução para um dispositivo (ex.: o player embutido do Azriel).
#[tauri::command]
pub async fn spotify_transfer(app: AppHandle, device_id: String, play: bool) -> Result<(), String> {
    if !valid_device(&device_id) {
        return Err("Dispositivo inválido".into());
    }
    api(&app, Method::PUT, "/me/player", Some(json!({"device_ids": [device_id], "play": play}))).await.map(|_| ())
}

#[tauri::command]
pub async fn spotify_player(app: AppHandle) -> Result<Option<Value>, String> {
    // Ler o estado sem dispositivo ativo não é erro: só não há nada tocando.
    match api(&app, Method::GET, "/me/player?additional_types=episode", None).await {
        Err(message) if message.starts_with("Nenhum dispositivo") => Ok(None),
        other => other,
    }
}

#[tauri::command]
pub async fn spotify_control(app: AppHandle, action: String, value: Option<f64>) -> Result<(), String> {
    let (method, path) = match action.as_str() {
        "play" => (Method::PUT, "/me/player/play".to_string()),
        "pause" => (Method::PUT, "/me/player/pause".to_string()),
        "next" => (Method::POST, "/me/player/next".to_string()),
        "previous" => (Method::POST, "/me/player/previous".to_string()),
        "shuffle" => (Method::PUT, format!("/me/player/shuffle?state={}", value.unwrap_or(0.0) > 0.0)),
        "volume" => (Method::PUT, format!("/me/player/volume?volume_percent={}", value.unwrap_or(50.0).clamp(0.0, 100.0).round())),
        "seek" => (Method::PUT, format!("/me/player/seek?position_ms={}", value.unwrap_or(0.0).max(0.0).round())),
        _ => return Err("Comando do Spotify inválido".into()),
    };
    api(&app, method, &path, None).await.map(|_| ())
}

#[tauri::command]
pub async fn spotify_search(app: AppHandle, query: String, kind: Option<String>, offset: Option<u32>) -> Result<Option<Value>, String> {
    let query = query.trim();
    if query.is_empty() || query.chars().count() > 200 {
        return Err("Digite o que buscar (até 200 caracteres)".into());
    }
    let (types, limit) = search_params(kind.as_deref().unwrap_or("all"));
    let offset = offset.unwrap_or(0).min(990);
    api(&app, Method::GET, &format!("/search?{}", form(&[("q", query), ("type", types), ("limit", &limit.to_string()), ("offset", &offset.to_string())])), None).await
}

#[tauri::command]
pub async fn spotify_playlists(app: AppHandle) -> Result<Option<Value>, String> {
    api(&app, Method::GET, "/me/playlists?limit=50", None).await
}

/// Faixas de uma playlist, álbum ou artista, para abrir o item antes de tocar.
#[tauri::command]
pub async fn spotify_tracks(app: AppHandle, uri: String, name: Option<String>, offset: Option<u32>) -> Result<Option<Value>, String> {
    if !valid_uri(&uri) {
        return Err("Item do Spotify inválido".into());
    }
    let path = tracks_path(&uri, name.as_deref(), offset.unwrap_or(0))?;
    api(&app, Method::GET, &path, None).await
}

/// Adiciona uma música ao fim da fila do dispositivo ativo (ou do indicado).
#[tauri::command]
pub async fn spotify_queue(app: AppHandle, uri: String, device_id: Option<String>) -> Result<(), String> {
    if !uri.starts_with("spotify:track:") || !valid_uri(&uri) {
        return Err("Só músicas podem entrar na fila".into());
    }
    let mut path = format!("/me/player/queue?uri={}", encode(&uri));
    if let Some(id) = device_id.filter(|id| valid_device(id)) {
        path.push_str(&format!("&device_id={id}"));
    }
    api(&app, Method::POST, &path, None).await.map(|_| ())
}

/// Toca uma faixa ou um contexto (playlist, álbum, artista). Sem dispositivo ativo,
/// usa o primeiro disponível; sem nenhum, abre o app do Spotify e tenta de novo.
#[tauri::command]
pub async fn spotify_play(app: AppHandle, uri: String, device_id: Option<String>, context: Option<String>) -> Result<(), String> {
    if !valid_uri(&uri) {
        return Err("Item do Spotify inválido".into());
    }
    let body = play_body(&uri, context.as_deref());
    // Com o player embutido pronto, toca nele direto; sem ele, segue o fluxo do app do Spotify.
    if let Some(id) = device_id.filter(|id| valid_device(id)) {
        return api(&app, Method::PUT, &format!("/me/player/play?device_id={id}"), Some(body)).await.map(|_| ());
    }
    match api(&app, Method::PUT, "/me/player/play", Some(body.clone())).await {
        Ok(_) => return Ok(()),
        Err(error) if !error.contains("dispositivo") => return Err(error),
        Err(_) => {}
    }
    for attempt in 0..6 {
        let devices = api(&app, Method::GET, "/me/player/devices", None).await?.unwrap_or(Value::Null);
        let device = devices["devices"].as_array().and_then(|list| list.iter().find(|d| d["is_computer"].as_bool().unwrap_or(false) || d["type"] == "Computer").or_else(|| list.first())).and_then(|d| d["id"].as_str()).map(String::from);
        if let Some(id) = device {
            return api(&app, Method::PUT, &format!("/me/player/play?device_id={}", encode(&id)), Some(body)).await.map(|_| ());
        }
        if attempt == 0 {
            use tauri_plugin_opener::OpenerExt;
            let _ = app.opener().open_url("spotify:", None::<&str>);
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    Err("O app do Spotify não apareceu como dispositivo. Abra o Spotify e tente de novo.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_matches_the_rfc_example() {
        // RFC 7636, apêndice B.
        assert_eq!(code_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"), "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
        assert_eq!(base64url(&[0xff, 0xfe]), "__4");
    }

    #[test]
    fn authorize_url_uses_pkce_and_the_loopback_redirect() {
        let url = authorize_url("abc", "challenge", "xyz");
        assert!(url.starts_with("https://accounts.spotify.com/authorize?client_id=abc&response_type=code"));
        assert!(url.contains("redirect_uri=http%3A%2F%2F127.0.0.1%3A8789%2Fcallback"));
        assert!(url.contains("code_challenge_method=S256&code_challenge=challenge"));
        assert!(url.contains("user-modify-playback-state"));
        assert!(url.contains("streaming"), "o player embutido exige o escopo streaming");
    }

    #[test]
    fn callback_request_is_parsed() {
        assert_eq!(parse_callback("GET /callback?code=AQB1&state=xyz HTTP/1.1"), Some((Some("AQB1".into()), Some("xyz".into()), None)));
        assert_eq!(parse_callback("GET /callback?error=access_denied&state=xyz HTTP/1.1"), Some((None, Some("xyz".into()), Some("access_denied".into()))));
        assert_eq!(parse_callback("GET /favicon.ico HTTP/1.1"), None);
    }

    #[test]
    fn only_spotify_uris_are_playable() {
        assert!(valid_uri("spotify:track:4uLU6hMCjMI75M1A2tKUQC"));
        assert!(valid_uri("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M"));
        assert!(!valid_uri("spotify:track:../x"));
        assert!(!valid_uri("https://open.spotify.com/track/1"));
        assert!(!valid_uri("spotify:user:abc"));
    }

    #[test]
    fn search_is_paged_by_type_within_the_api_limit() {
        assert_eq!(search_params("track"), ("track", 10));
        assert_eq!(search_params("artist"), ("artist", 10));
        assert_eq!(search_params("all"), ("track,playlist,artist,album", 8));
        assert_eq!(search_params("qualquer"), ("track,playlist,artist,album", 8));
    }

    #[test]
    fn tracks_play_inside_their_album_and_the_rest_as_context() {
        let album = "spotify:album:2up3OPMp9Tb4dAKM2erWXQ";
        let track = "spotify:track:4uLU6hMCjMI75M1A2tKUQC";
        assert_eq!(play_body(track, Some(album)), json!({"context_uri": album, "offset": {"uri": track}}));
        assert_eq!(play_body(track, None), json!({"uris": [track]}));
        assert_eq!(play_body(track, Some("spotify:track:outra")), json!({"uris": [track]}), "uma faixa não serve de contexto");
        assert_eq!(play_body(album, Some("spotify:playlist:x")), json!({"context_uri": album}));
    }

    #[test]
    fn track_lists_use_the_current_endpoints() {
        assert_eq!(tracks_path("spotify:playlist:37i9dQZF1DXcBWIGoYBM5M", None, 0).unwrap(), "/playlists/37i9dQZF1DXcBWIGoYBM5M/items?limit=50&offset=0&additional_types=track");
        assert_eq!(tracks_path("spotify:album:2up3OPMp9Tb4dAKM2erWXQ", None, 50).unwrap(), "/albums/2up3OPMp9Tb4dAKM2erWXQ/tracks?limit=50&offset=50");
        let artist = tracks_path("spotify:artist:4gzpq5DPGxSnKTe4SA8HAU", Some("Cold\"play & Co"), 10).unwrap();
        assert_eq!(artist, "/search?q=artist%3A%22Coldplay%20%26%20Co%22&type=track&limit=10&offset=10");
        assert!(tracks_path("spotify:artist:x", None, 0).is_err(), "o artista exige o nome");
        assert!(tracks_path("spotify:track:x", None, 0).is_err());
        assert!(tracks_path("spotify:playlist", None, 0).is_err());
    }

    #[test]
    fn device_ids_are_validated() {
        assert!(valid_device("abc123DEF456"));
        assert!(!valid_device(""));
        assert!(!valid_device("a/b?c"));
    }

    #[test]
    fn api_errors_are_explained() {
        assert!(api_error(404, &json!({"error": {"status": 404, "message": "Player command failed: No active device found", "reason": "NO_ACTIVE_DEVICE"}})).contains("dispositivo"));
        assert!(api_error(403, &json!({"error": {"message": "x", "reason": "PREMIUM_REQUIRED"}})).contains("Premium"));
        assert!(api_error(500, &json!({"error": {"message": "boom"}})).contains("500"));
        assert!(cooldown_message(30).contains("30s"));
        assert!(cooldown_message(600).contains("10 min"));
    }
}
