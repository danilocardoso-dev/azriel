use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, time::Duration};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    pub available: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaChatResult {
    pub content: String,
    pub model: String,
    pub truncated: bool,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<TagModel>,
}
#[derive(Deserialize)]
struct TagModel {
    name: String,
}
#[derive(Deserialize)]
struct ChatResponse {
    model: String,
    message: OllamaMessage,
    done_reason: Option<String>,
}

pub async fn status(endpoint: &str, timeout_seconds: u64) -> Result<OllamaStatus, String> {
    let endpoint = normalize_endpoint(endpoint)?;
    let client = client(timeout_seconds)?;
    match client.get(format!("{endpoint}/api/tags")).send().await {
        Ok(response) if response.status().is_success() => {
            let payload = response
                .json::<TagsResponse>()
                .await
                .map_err(|_| "Resposta inválida recebida do Ollama".to_string())?;
            Ok(OllamaStatus {
                available: true,
                models: payload.models.into_iter().map(|model| model.name).collect(),
                error: None,
            })
        }
        Ok(response) => Ok(OllamaStatus {
            available: false,
            models: vec![],
            error: Some(format!("Ollama respondeu com status {}", response.status())),
        }),
        Err(error) => {
            eprintln!(
                "[AI_CORE] Ollama indisponível: {}",
                if error.is_timeout() {
                    "timeout"
                } else {
                    "conexão"
                }
            );
            Ok(OllamaStatus {
                available: false,
                models: vec![],
                error: Some(request_error(&error)),
            })
        }
    }
}

pub async fn chat(
    endpoint: &str,
    model: &str,
    messages: Vec<OllamaMessage>,
    timeout_seconds: u64,
    generation_profile: &str,
) -> Result<OllamaChatResult, String> {
    let endpoint = normalize_endpoint(endpoint)?;
    validate_model(model)?;
    if messages.is_empty() || messages.len() > 30 {
        return Err("Quantidade de mensagens inválida para o AI Core".into());
    }
    if messages.iter().any(|message| {
        !["system", "user", "assistant"].contains(&message.role.as_str())
            || message.content.trim().is_empty()
            || message.content.chars().count() > 50_000
    }) {
        return Err("Conteúdo de conversa inválido para o AI Core".into());
    }
    let options = match generation_profile {
        "standard" => serde_json::json!({
            "temperature": 0.35,
            "top_p": 0.9,
            "top_k": 40,
            "repeat_penalty": 1.18,
            "repeat_last_n": 128,
            "num_predict": 420
        }),
        "repetition-retry" => serde_json::json!({
            "temperature": 0.45,
            "top_p": 0.85,
            "top_k": 30,
            "repeat_penalty": 1.28,
            "repeat_last_n": 192,
            "num_predict": 180
        }),
        _ => return Err("Perfil de geração inválido para o AI Core".into()),
    };
    let response = client(timeout_seconds)?
        .post(format!("{endpoint}/api/chat"))
        .json(&serde_json::json!({
            "model": model.trim(), "messages": messages, "stream": false,
            "options": options
        }))
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "[AI_CORE] Falha na chamada ao Ollama: {}",
                if error.is_timeout() {
                    "timeout"
                } else {
                    "conexão"
                }
            );
            request_error(&error)
        })?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        eprintln!("[AI_CORE] Ollama recusou a requisição com status {status}");
        return Err(response_error(status.as_u16(), &detail, model));
    }
    let payload = response
        .json::<ChatResponse>()
        .await
        .map_err(|_| "Resposta de chat inválida recebida do Ollama".to_string())?;
    if payload.message.content.trim().is_empty() {
        return Err("Ollama retornou uma resposta vazia".into());
    }
    Ok(OllamaChatResult {
        content: payload.message.content.trim().into(),
        model: payload.model,
        truncated: payload.done_reason.as_deref() == Some("length"),
    })
}

pub fn normalize_endpoint(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches('/');
    let url = Url::parse(trimmed).map_err(|_| "Endpoint do Ollama inválido".to_string())?;
    if url.scheme() != "http"
        || url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "O endpoint deve ser uma URL HTTP local sem credenciais, query ou fragmento".into(),
        );
    }
    let host = url
        .host_str()
        .ok_or_else(|| "Endpoint do Ollama sem host".to_string())?;
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .map(|address| address.is_loopback())
            .unwrap_or(false);
    if !loopback {
        return Err("Por segurança, o endpoint do Ollama deve apontar para localhost".into());
    }
    if !url.path().is_empty() && url.path() != "/" {
        return Err("O endpoint deve apontar para a raiz da API do Ollama".into());
    }
    Ok(trimmed.into())
}

fn validate_model(model: &str) -> Result<(), String> {
    let model = model.trim();
    if model.is_empty() || model.chars().count() > 120 || model.chars().any(char::is_whitespace) {
        Err("Nome de modelo inválido".into())
    } else {
        Ok(())
    }
}

fn client(timeout_seconds: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_seconds.clamp(5, 180)))
        .build()
        .map_err(|_| "Não foi possível preparar o cliente local do Ollama".into())
}

fn request_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "O Ollama excedeu o tempo limite da solicitação".into()
    } else {
        "Ollama não encontrado no endpoint configurado".into()
    }
}

fn response_error(status: u16, detail: &str, model: &str) -> String {
    let detail = detail.to_lowercase();
    if detail.contains("model") && detail.contains("not found") {
        format!("O modelo '{}' não está instalado no Ollama", model.trim())
    } else {
        format!("Ollama respondeu com status {status}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_is_restricted_to_local_ollama() {
        assert_eq!(
            normalize_endpoint("http://localhost:11434/").unwrap(),
            "http://localhost:11434"
        );
        assert!(normalize_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(normalize_endpoint("https://example.com").is_err());
        assert!(normalize_endpoint("http://localhost:11434/api").is_err());
    }

    #[test]
    fn model_not_found_has_a_clear_error() {
        let message = response_error(404, r#"{"error":"model 'missing' not found"}"#, "missing");
        assert_eq!(message, "O modelo 'missing' não está instalado no Ollama");
    }

    #[test]
    fn unavailable_ollama_is_reported_without_panicking() {
        let result = tauri::async_runtime::block_on(status("http://127.0.0.1:9", 5)).unwrap();
        assert!(!result.available);
        assert!(result.error.is_some());
    }

    #[test]
    #[ignore = "requer Ollama local e qwen2.5:0.5b"]
    fn live_ollama_chat_smoke_test() {
        let result = tauri::async_runtime::block_on(chat(
            "http://localhost:11434",
            "qwen2.5:0.5b",
            vec![OllamaMessage {
                role: "user".into(),
                content: "Responda apenas: ONLINE".into(),
            }],
            60,
            "standard",
        ))
        .unwrap();
        assert!(!result.content.trim().is_empty());
    }
}
