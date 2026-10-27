//! Chamada de voz com o Gemini Live. O WebSocket é aberto aqui no backend para a
//! chave do AI Studio nunca chegar à interface; o front só troca áudio e eventos.

use crate::database::{ai_repository, DatabaseState};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Mutex;
use tauri::{ipc::Channel, AppHandle, Manager, State};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

const LIVE_URL: &str = "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent";
pub const LIVE_MODEL: &str = "gemini-3.8-live";
const VOICE_NAME: &str = "Charon";
const SYSTEM_PROMPT: &str = include_str!("../prompts/voice-v1.txt");

#[derive(Default)]
pub struct VoiceState {
    outbox: Mutex<Option<mpsc::UnboundedSender<String>>>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum VoiceEvent {
    Ready,
    Audio { data: String },
    InputText { text: String },
    OutputText { text: String },
    Interrupted,
    TurnComplete,
    ToolCall { calls: Vec<ToolCall> },
    Closed { reason: String },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub args: Value,
}

/// Primeira mensagem da sessão: modelo, voz, persona, ferramentas e legendas.
pub fn setup_message(tools: &[Value], date: &str) -> Value {
    // A busca usa a ferramenta search_web (executada fora da sessão), e não o googleSearch nativo do Live:
    // com o nativo, estourar a cota de busca derrubava a chamada inteira (close 1011).
    let declarations: Vec<Value> = tools
        .iter()
        .filter_map(|tool| {
            let function = &tool["function"];
            let mut declaration = json!({"name": function["name"].as_str()?, "description": function["description"].as_str().unwrap_or("")});
            if function["parameters"]["properties"].as_object().is_some_and(|properties| !properties.is_empty()) {
                declaration["parameters"] = function["parameters"].clone();
            }
            Some(declaration)
        })
        .collect();
    let mut setup = json!({
        "setup": {
            "model": format!("models/{LIVE_MODEL}"),
            "generationConfig": {
                "responseModalities": ["AUDIO"],
                "speechConfig": {"voiceConfig": {"prebuiltVoiceConfig": {"voiceName": VOICE_NAME}}}
            },
            "systemInstruction": {"parts": [{"text": format!("{SYSTEM_PROMPT}\nData local atual: {date}.")}]},
            "inputAudioTranscription": {},
            "outputAudioTranscription": {}
        }
    });
    if !declarations.is_empty() {
        setup["setup"]["tools"] = json!([{"functionDeclarations": declarations}]);
    }
    setup
}

/// Converte uma mensagem do servidor em eventos para a interface (uma mensagem pode trazer vários).
pub fn parse_server_message(raw: &str) -> Vec<VoiceEvent> {
    let Ok(message) = serde_json::from_str::<Value>(raw) else {
        return vec![];
    };
    let mut events = Vec::new();
    if message.get("setupComplete").is_some() {
        events.push(VoiceEvent::Ready);
    }
    let content = &message["serverContent"];
    if let Some(parts) = content["modelTurn"]["parts"].as_array() {
        for part in parts {
            if let Some(data) = part["inlineData"]["data"].as_str() {
                events.push(VoiceEvent::Audio { data: data.into() });
            }
        }
    }
    if let Some(text) = content["inputTranscription"]["text"].as_str() {
        events.push(VoiceEvent::InputText { text: text.into() });
    }
    if let Some(text) = content["outputTranscription"]["text"].as_str() {
        events.push(VoiceEvent::OutputText { text: text.into() });
    }
    if content["interrupted"].as_bool() == Some(true) {
        events.push(VoiceEvent::Interrupted);
    }
    if content["turnComplete"].as_bool() == Some(true) {
        events.push(VoiceEvent::TurnComplete);
    }
    if let Some(calls) = message["toolCall"]["functionCalls"].as_array() {
        let calls: Vec<ToolCall> = calls
            .iter()
            .filter_map(|call| Some(ToolCall { id: call["id"].as_str().unwrap_or_default().into(), name: call["name"].as_str()?.into(), args: call["args"].clone() }))
            .collect();
        if !calls.is_empty() {
            events.push(VoiceEvent::ToolCall { calls });
        }
    }
    if let Some(text) = message["error"]["message"].as_str() {
        events.push(VoiceEvent::Error { message: text.chars().take(240).collect() });
    }
    if message.get("goAway").is_some() {
        events.push(VoiceEvent::Closed { reason: "O Google vai encerrar a sessão em instantes (limite de tempo).".into() });
    }
    events
}

fn close_reason(code: u16, reason: &str) -> String {
    let reason = reason.split_whitespace().collect::<Vec<_>>().join(" ").chars().take(200).collect::<String>();
    if reason.to_lowercase().contains("quota") {
        return "A cota do Gemini Live acabou por agora (plano gratuito). Tente mais tarde ou veja os limites em Rate limits no Google AI Studio.".into();
    }
    match code {
        1000 if reason.is_empty() => "Chamada encerrada.".into(),
        1007 | 1008 => format!("O Gemini recusou a sessão ({code}). {reason}"),
        _ if reason.is_empty() => format!("Conexão encerrada ({code})."),
        _ => format!("Conexão encerrada ({code}): {reason}"),
    }
}

#[tauri::command]
pub async fn voice_start(app: AppHandle, state: State<'_, DatabaseState>, tools: Vec<Value>, date: String, events: Channel<VoiceEvent>) -> Result<(), String> {
    let (api_key, _) = {
        let connection = state.connection.lock().map_err(|_| "O banco de dados está indisponível".to_string())?;
        ai_repository::remote_credentials(&connection)?
    };
    voice_stop(app.state::<VoiceState>());
    let (socket, _) = tokio_tungstenite::connect_async(format!("{LIVE_URL}?key={api_key}"))
        .await
        .map_err(|_| "Não foi possível conectar ao Gemini Live. Verifique a internet.".to_string())?;
    let (mut writer, mut reader) = socket.split();
    writer
        .send(Message::text(setup_message(&tools, &date).to_string()))
        .await
        .map_err(|_| "Falha ao iniciar a sessão de voz".to_string())?;

    let (outbox, mut inbox) = mpsc::unbounded_channel::<String>();
    if let Ok(mut slot) = app.state::<VoiceState>().outbox.lock() {
        *slot = Some(outbox);
    }

    tauri::async_runtime::spawn(async move {
        while let Some(text) = inbox.recv().await {
            if writer.send(Message::text(text)).await.is_err() {
                break;
            }
        }
        let _ = writer.close().await;
    });

    tauri::async_runtime::spawn(async move {
        let mut reason = "Chamada encerrada.".to_string();
        while let Some(message) = reader.next().await {
            let raw = match message {
                Ok(Message::Text(text)) => text.to_string(),
                Ok(Message::Binary(bytes)) => String::from_utf8_lossy(&bytes).into_owned(),
                Ok(Message::Close(frame)) => {
                    if let Some(frame) = frame {
                        reason = close_reason(frame.code.into(), &frame.reason);
                    }
                    break;
                }
                Ok(_) => continue,
                Err(_) => {
                    reason = "A conexão de voz caiu.".into();
                    break;
                }
            };
            for event in parse_server_message(&raw) {
                if events.send(event).is_err() {
                    return;
                }
            }
        }
        if let Ok(mut slot) = app.state::<VoiceState>().outbox.lock() {
            *slot = None;
        }
        let _ = events.send(VoiceEvent::Closed { reason });
    });
    Ok(())
}

fn send(state: &VoiceState, message: Value) -> Result<(), String> {
    let slot = state.outbox.lock().map_err(|_| "Sessão de voz indisponível".to_string())?;
    let outbox = slot.as_ref().ok_or("Nenhuma chamada em andamento")?;
    outbox.send(message.to_string()).map_err(|_| "A chamada já foi encerrada".to_string())
}

/// Bloco de áudio do microfone: PCM 16-bit, 16 kHz, mono, em base64.
#[tauri::command]
pub fn voice_send_audio(state: State<'_, VoiceState>, data: String) -> Result<(), String> {
    if data.len() > 200_000 {
        return Err("Bloco de áudio grande demais".into());
    }
    send(&state, json!({"realtimeInput": {"audio": {"data": data, "mimeType": "audio/pcm;rate=16000"}}}))
}

#[tauri::command]
pub fn voice_tool_response(state: State<'_, VoiceState>, id: String, name: String, result: Value) -> Result<(), String> {
    send(&state, json!({"toolResponse": {"functionResponses": [{"id": id, "name": name, "response": {"result": result}}]}}))
}

#[tauri::command]
pub fn voice_stop(state: State<'_, VoiceState>) {
    if let Ok(mut slot) = state.outbox.lock() {
        // Soltar o remetente encerra a tarefa de escrita, que fecha o WebSocket.
        *slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setup_declares_voice_tools_and_transcription() {
        let tools = vec![
            json!({"type":"function","function":{"name":"get_today_tasks","description":"Tarefas de hoje","parameters":{"type":"object","properties":{}}}}),
            json!({"type":"function","function":{"name":"search_web","description":"Busca","parameters":{"type":"object","properties":{"query":{"type":"string"}}}}}),
        ];
        let setup = setup_message(&tools, "2026-10-26");
        let body = &setup["setup"];
        assert_eq!(body["model"], "models/gemini-3.8-live");
        assert_eq!(body["generationConfig"]["responseModalities"][0], "AUDIO");
        assert_eq!(body["generationConfig"]["speechConfig"]["voiceConfig"]["prebuiltVoiceConfig"]["voiceName"], VOICE_NAME);
        assert_eq!(body["tools"][0]["functionDeclarations"][0], json!({"name":"get_today_tasks","description":"Tarefas de hoje"}));
        assert_eq!(body["tools"][0]["functionDeclarations"][1]["parameters"]["properties"]["query"]["type"], "string");
        assert!(body["tools"].as_array().unwrap().iter().all(|tool| tool.get("googleSearch").is_none()), "busca nativa derruba a chamada quando a cota estoura");
        assert!(body["inputAudioTranscription"].is_object() && body["outputAudioTranscription"].is_object());
        assert!(body["systemInstruction"]["parts"][0]["text"].as_str().unwrap().contains("2026-10-26"));
        assert!(setup_message(&[], "x")["setup"].get("tools").is_none());
    }

    #[test]
    fn server_messages_become_ui_events() {
        assert_eq!(parse_server_message(r#"{"setupComplete":{}}"#), vec![VoiceEvent::Ready]);
        let turn = parse_server_message(r#"{"serverContent":{"modelTurn":{"parts":[{"inlineData":{"mimeType":"audio/pcm;rate=24000","data":"AAA="}}]},"outputTranscription":{"text":"Olá"},"turnComplete":true}}"#);
        assert_eq!(turn, vec![VoiceEvent::Audio { data: "AAA=".into() }, VoiceEvent::OutputText { text: "Olá".into() }, VoiceEvent::TurnComplete]);
        assert_eq!(parse_server_message(r#"{"serverContent":{"interrupted":true}}"#), vec![VoiceEvent::Interrupted]);
        let tool = parse_server_message(r#"{"toolCall":{"functionCalls":[{"id":"c1","name":"get_trading_status","args":{}}]}}"#);
        assert_eq!(tool, vec![VoiceEvent::ToolCall { calls: vec![ToolCall { id: "c1".into(), name: "get_trading_status".into(), args: json!({}) }] }]);
        assert!(matches!(parse_server_message(r#"{"goAway":{"timeLeft":"10s"}}"#)[0], VoiceEvent::Closed { .. }));
        assert!(parse_server_message("not json").is_empty());
        assert_eq!(parse_server_message(r#"{"error":{"message":"quota"}}"#), vec![VoiceEvent::Error { message: "quota".into() }]);
    }

    #[test]
    fn close_reasons_are_readable() {
        assert_eq!(close_reason(1000, ""), "Chamada encerrada.");
        assert!(close_reason(1008, "API key not valid").contains("recusou"));
        assert!(close_reason(1011, "").contains("1011"));
        assert!(close_reason(1011, "You exceeded your current quota, please check your plan").contains("cota do Gemini Live"));
    }
}
