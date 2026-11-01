//! "Tocando agora": lê e controla a mídia do Windows (System Media Transport Controls),
//! a mesma que aparece no painel de volume. Funciona com o app do Spotify, YouTube no
//! navegador ou no mini player do Azriel, sem conta de desenvolvedor nem chave.

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub source: String,
    pub playing: bool,
    pub position_seconds: Option<f64>,
    pub duration_seconds: Option<f64>,
    /// Capa como data URL (ou `None` quando o player não fornece).
    pub artwork: Option<String>,
}

/// Nome amigável do app que está tocando, a partir do AppUserModelId do Windows.
pub fn friendly_source(app_id: &str) -> String {
    let id = app_id.to_lowercase();
    if id.contains("spotify") {
        "Spotify".into()
    } else if id.contains("chrome") {
        "Chrome".into()
    } else if id.contains("msedge") || id.contains("microsoftedge") {
        "Edge".into()
    } else if id.contains("firefox") {
        "Firefox".into()
    } else if id.contains("azriel") {
        "Azriel".into()
    } else {
        app_id.rsplit(['\\', '!']).next().unwrap_or(app_id).trim_end_matches(".exe").to_string()
    }
}

#[cfg(windows)]
mod platform {
    use super::{friendly_source, NowPlaying};
    use std::sync::Mutex;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session, GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Storage::Streams::DataReader;
    use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

    /// Capa da última faixa: evita reler a imagem a cada consulta (chave = título|artista).
    static ARTWORK: Mutex<Option<(String, Option<String>)>> = Mutex::new(None);

    fn session() -> windows::core::Result<Option<Session>> {
        // A thread do pool não tem apartamento COM; o erro "já inicializado" é inofensivo.
        unsafe {
            let _ = RoInitialize(RO_INIT_MULTITHREADED);
        }
        let manager = Manager::RequestAsync()?.get()?;
        Ok(manager.GetCurrentSession().ok())
    }

    fn artwork(session_props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties) -> Option<String> {
        let reference = session_props.Thumbnail().ok()?;
        let stream = reference.OpenReadAsync().ok()?.get().ok()?;
        let size = stream.Size().ok()? as u32;
        if size == 0 || size > 4_000_000 {
            return None;
        }
        let content_type = stream.ContentType().map(|t| t.to_string()).unwrap_or_else(|_| "image/png".into());
        let reader = DataReader::CreateDataReader(&stream).ok()?;
        reader.LoadAsync(size).ok()?.get().ok()?;
        let mut bytes = vec![0u8; size as usize];
        reader.ReadBytes(&mut bytes).ok()?;
        Some(format!("data:{};base64,{}", if content_type.is_empty() { "image/png" } else { &content_type }, super::base64(&bytes)))
    }

    pub fn now_playing() -> Result<Option<NowPlaying>, String> {
        let Some(session) = session().map_err(|error| format!("Controles de mídia do Windows indisponíveis: {error}"))? else {
            return Ok(None);
        };
        let props = session.TryGetMediaPropertiesAsync().and_then(|op| op.get()).map_err(|error| format!("Falha ao ler a mídia: {error}"))?;
        let title = props.Title().map(|t| t.to_string()).unwrap_or_default();
        if title.trim().is_empty() {
            return Ok(None);
        }
        let artist = props.Artist().map(|t| t.to_string()).unwrap_or_default();
        let key = format!("{title}|{artist}");
        let cached = ARTWORK.lock().ok().and_then(|slot| slot.as_ref().filter(|(k, _)| *k == key).map(|(_, art)| art.clone()));
        let art = match cached {
            Some(art) => art,
            None => {
                let art = artwork(&props);
                if let Ok(mut slot) = ARTWORK.lock() {
                    *slot = Some((key, art.clone()));
                }
                art
            }
        };
        let playing = session.GetPlaybackInfo().and_then(|info| info.PlaybackStatus()).map(|status| status == Status::Playing).unwrap_or(false);
        let timeline = session.GetTimelineProperties().ok();
        let seconds = |ticks: i64| ticks as f64 / 10_000_000.0;
        let duration = timeline.as_ref().and_then(|t| t.EndTime().ok()).map(|d| seconds(d.Duration)).filter(|d| *d > 0.0);
        let position = timeline.as_ref().and_then(|t| t.Position().ok()).map(|d| seconds(d.Duration)).filter(|_| duration.is_some());
        Ok(Some(NowPlaying {
            title,
            artist,
            album: props.AlbumTitle().map(|t| t.to_string()).unwrap_or_default(),
            source: friendly_source(&session.SourceAppUserModelId().map(|t| t.to_string()).unwrap_or_default()),
            playing,
            position_seconds: position,
            duration_seconds: duration,
            artwork: art,
        }))
    }

    pub fn control(action: &str) -> Result<(), String> {
        let session = session().map_err(|error| error.to_string())?.ok_or("Nada tocando no momento")?;
        let operation = match action {
            "toggle" => session.TryTogglePlayPauseAsync(),
            "next" => session.TrySkipNextAsync(),
            "previous" => session.TrySkipPreviousAsync(),
            _ => return Err("Comando de mídia inválido".into()),
        };
        let accepted = operation.and_then(|op| op.get()).map_err(|error| error.to_string())?;
        if accepted { Ok(()) } else { Err("O player recusou o comando".into()) }
    }
}

#[cfg(not(windows))]
mod platform {
    use super::NowPlaying;
    pub fn now_playing() -> Result<Option<NowPlaying>, String> { Ok(None) }
    pub fn control(_action: &str) -> Result<(), String> { Err("Controle de mídia disponível apenas no Windows".into()) }
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8) | u32::from(*chunk.get(2).unwrap_or(&0));
        for i in 0..4 {
            out.push(if i <= chunk.len() { TABLE[(n >> (18 - 6 * i) & 63) as usize] as char } else { '=' });
        }
    }
    out
}

#[tauri::command]
pub async fn media_now_playing() -> Result<Option<NowPlaying>, String> {
    tauri::async_runtime::spawn_blocking(platform::now_playing).await.map_err(|_| "Falha ao consultar a mídia".to_string())?
}

#[tauri::command]
pub async fn media_control(action: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || platform::control(&action)).await.map_err(|_| "Falha ao controlar a mídia".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_get_friendly_names() {
        assert_eq!(friendly_source("Spotify.exe"), "Spotify");
        assert_eq!(friendly_source("Chrome"), "Chrome");
        assert_eq!(friendly_source("MSEdge"), "Edge");
        assert_eq!(friendly_source("C:\\Apps\\vlc.exe"), "vlc");
        assert_eq!(friendly_source("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"), "Microsoft.ZuneMusic");
    }

    #[test]
    #[ignore = "lê a mídia real do Windows"]
    fn live_now_playing_smoke_test() {
        let result = platform::now_playing().expect("a API de mídia do Windows deveria responder");
        match result {
            Some(now) => println!("tocando: {} — {} [{}] playing={} capa={}", now.title, now.artist, now.source, now.playing, now.artwork.as_ref().map(|a| a.len()).unwrap_or(0)),
            None => println!("nada tocando"),
        }
    }

    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(&[0xff, 0xfe, 0x00, 0x10]), "//4AEA==");
    }
}
