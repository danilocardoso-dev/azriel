use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::{
    net::IpAddr,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

const LIFECYCLE_CONFIRM_ATTEMPTS: usize = 4;
const LIFECYCLE_CONFIRM_DELAY_MS: u64 = 450;
static LIFECYCLE_OPERATION_ACTIVE: AtomicBool = AtomicBool::new(false);
static MODEL_MANUALLY_PINNED: AtomicBool = AtomicBool::new(false);

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaToolFunction {
    pub name: String,
    #[serde(default = "empty_arguments")]
    pub arguments: serde_json::Value,
}

fn empty_arguments() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaToolCall {
    pub function: OllamaToolFunction,
    /// Campos do provedor remoto que precisam voltar intactos na rodada seguinte
    /// (ex.: `thought_signature` do Gemini). O Ollama nunca os preenche.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_content: Option<serde_json::Value>,
}

/// Mensagem do loop de agente: alem de system/user/assistant aceita o papel
/// `tool` e as chamadas de ferramenta no formato nativo do Ollama.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaAgentMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<OllamaToolCall>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaAgentResult {
    pub content: String,
    pub model: String,
    pub truncated: bool,
    pub tool_calls: Vec<OllamaToolCall>,
}

const AGENT_MESSAGE_LIMIT: usize = 60;
const AGENT_TOOL_LIMIT: usize = 40;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAiModelStatus {
    pub provider: String,
    pub model: String,
    pub server_status: String,
    pub model_status: String,
    pub loaded: bool,
    pub expires_at: Option<String>,
    pub last_checked_at: String,
    pub error: Option<String>,
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
#[derive(Deserialize)]
struct AgentChatResponse {
    model: String,
    message: OllamaAgentMessage,
    done_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RunningModelsResponse {
    models: Vec<RunningModel>,
}

#[derive(Debug, Deserialize)]
struct RunningModel {
    #[serde(default)]
    name: String,
    #[serde(default)]
    model: String,
    expires_at: Option<String>,
}

struct LifecycleGuard;

impl LifecycleGuard {
    fn acquire() -> Result<Self, String> {
        LIFECYCLE_OPERATION_ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| Self)
            .map_err(|_| {
                "Ja existe uma operacao de ativacao ou desativacao da IA em andamento.".into()
            })
    }
}

impl Drop for LifecycleGuard {
    fn drop(&mut self) {
        LIFECYCLE_OPERATION_ACTIVE.store(false, Ordering::Release);
    }
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

pub async fn model_lifecycle_status(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
) -> Result<LocalAiModelStatus, String> {
    let started = std::time::Instant::now();
    let result = query_model_status(endpoint, model, timeout_seconds).await;
    log_lifecycle(
        "STATUS",
        model,
        started.elapsed().as_millis(),
        &Ok(result.clone()),
    );
    Ok(result)
}

pub async fn load_model(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
) -> Result<LocalAiModelStatus, String> {
    let _guard = LifecycleGuard::acquire()?;
    let started = std::time::Instant::now();
    let result = change_model_lifecycle(endpoint, model, timeout_seconds, -1, true).await;
    if result.as_ref().is_ok_and(|status| status.loaded) {
        MODEL_MANUALLY_PINNED.store(true, Ordering::Release);
    }
    log_lifecycle("LOAD", model, started.elapsed().as_millis(), &result);
    result
}

pub async fn unload_model(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
) -> Result<LocalAiModelStatus, String> {
    let _guard = LifecycleGuard::acquire()?;
    let started = std::time::Instant::now();
    let result = change_model_lifecycle(endpoint, model, timeout_seconds, 0, false).await;
    if result.as_ref().is_ok_and(|status| !status.loaded) {
        MODEL_MANUALLY_PINNED.store(false, Ordering::Release);
    }
    log_lifecycle("UNLOAD", model, started.elapsed().as_millis(), &result);
    result
}

async fn change_model_lifecycle(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
    keep_alive: i64,
    expected_loaded: bool,
) -> Result<LocalAiModelStatus, String> {
    change_model_lifecycle_with_policy(
        endpoint,
        model,
        timeout_seconds,
        keep_alive,
        expected_loaded,
        LIFECYCLE_CONFIRM_ATTEMPTS,
        Duration::from_millis(LIFECYCLE_CONFIRM_DELAY_MS),
    )
    .await
}

async fn change_model_lifecycle_with_policy(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
    keep_alive: i64,
    expected_loaded: bool,
    confirmation_attempts: usize,
    confirmation_delay: Duration,
) -> Result<LocalAiModelStatus, String> {
    let endpoint = normalize_endpoint(endpoint)?;
    validate_model(model)?;
    let response = client(timeout_seconds)?
        .post(format!("{endpoint}/api/generate"))
        .json(&serde_json::json!({
            "model": model.trim(),
            "keep_alive": keep_alive,
            "stream": false
        }))
        .send()
        .await
        .map_err(|error| request_error(&error))?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        return Err(response_error(status.as_u16(), &detail, model));
    }

    let mut last = None;
    let confirmation_attempts = confirmation_attempts.max(1);
    for attempt in 0..confirmation_attempts {
        let status = query_model_status(&endpoint, model, timeout_seconds).await;
        if status.server_status == "ONLINE" && status.loaded == expected_loaded {
            return Ok(status);
        }
        last = Some(status);
        if attempt + 1 < confirmation_attempts {
            tokio::time::sleep(confirmation_delay).await;
        }
    }
    let detail = last.and_then(|status| status.error).unwrap_or_else(|| {
        if expected_loaded {
            "o modelo nao apareceu em /api/ps".into()
        } else {
            "o modelo continua presente em /api/ps".into()
        }
    });
    Err(if expected_loaded {
        format!("A ativacao nao foi confirmada: {detail}")
    } else {
        format!("A desativacao nao foi confirmada: {detail}")
    })
}

async fn query_model_status(
    endpoint: &str,
    model: &str,
    timeout_seconds: u64,
) -> LocalAiModelStatus {
    let now = chrono::Utc::now().to_rfc3339();
    let endpoint = match normalize_endpoint(endpoint) {
        Ok(value) => value,
        Err(error) => return lifecycle_error(model, "OFFLINE", "ERROR", now, error),
    };
    if let Err(error) = validate_model(model) {
        return lifecycle_error(model, "UNKNOWN", "ERROR", now, error);
    }
    let response = match client(timeout_seconds) {
        Ok(client) => client.get(format!("{endpoint}/api/ps")).send().await,
        Err(error) => return lifecycle_error(model, "UNKNOWN", "ERROR", now, error),
    };
    match response {
        Ok(response) if response.status().is_success() => {
            match response.json::<RunningModelsResponse>().await {
                Ok(payload) => {
                    let running = payload.models.into_iter().find(|candidate| {
                        model_matches(model, &candidate.name)
                            || model_matches(model, &candidate.model)
                    });
                    let installed = if running.is_some() {
                        Some(true)
                    } else {
                        query_model_installed(&endpoint, model, timeout_seconds).await
                    };
                    LocalAiModelStatus {
                        provider: "OLLAMA".into(),
                        model: model.trim().into(),
                        server_status: "ONLINE".into(),
                        model_status: if running.is_some() {
                            "LOADED"
                        } else if installed == Some(false) {
                            "NOT_AVAILABLE"
                        } else {
                            "UNLOADED"
                        }
                        .into(),
                        loaded: running.is_some(),
                        expires_at: running.and_then(|candidate| candidate.expires_at),
                        last_checked_at: now,
                        error: None,
                    }
                }
                Err(_) => lifecycle_error(
                    model,
                    "ONLINE",
                    "ERROR",
                    now,
                    "Resposta invalida recebida de /api/ps.".into(),
                ),
            }
        }
        Ok(response) => lifecycle_error(
            model,
            "OFFLINE",
            "ERROR",
            now,
            format!(
                "Ollama respondeu com status {} ao consultar /api/ps.",
                response.status()
            ),
        ),
        Err(error) => lifecycle_error(model, "OFFLINE", "ERROR", now, request_error(&error)),
    }
}

async fn query_model_installed(endpoint: &str, model: &str, timeout_seconds: u64) -> Option<bool> {
    let response = client(timeout_seconds)
        .ok()?
        .get(format!("{endpoint}/api/tags"))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let payload = response.json::<TagsResponse>().await.ok()?;
    Some(
        payload
            .models
            .into_iter()
            .any(|candidate| model_matches(model, &candidate.name)),
    )
}

fn lifecycle_error(
    model: &str,
    server_status: &str,
    model_status: &str,
    last_checked_at: String,
    error: String,
) -> LocalAiModelStatus {
    LocalAiModelStatus {
        provider: "OLLAMA".into(),
        model: model.trim().into(),
        server_status: server_status.into(),
        model_status: model_status.into(),
        loaded: false,
        expires_at: None,
        last_checked_at,
        error: Some(error),
    }
}

fn model_matches(configured: &str, running: &str) -> bool {
    let configured = configured.trim().to_ascii_lowercase();
    let running = running.trim().to_ascii_lowercase();
    configured == running
        || (!configured.contains(':') && running == format!("{configured}:latest"))
        || (!running.contains(':') && configured == format!("{running}:latest"))
}

fn log_lifecycle(
    action: &str,
    model: &str,
    duration_ms: u128,
    result: &Result<LocalAiModelStatus, String>,
) {
    let (status, category) = match result {
        Ok(value) => (
            value.model_status.as_str(),
            value.error.as_deref().map(error_category).unwrap_or("none"),
        ),
        Err(error) => ("ERROR", error_category(error)),
    };
    eprintln!("[AI_MODEL] action={action} provider=OLLAMA model={} duration_ms={duration_ms} result={status} error={category}", safe_log_value(model));
}

fn error_category(value: &str) -> &'static str {
    let value = value.to_lowercase();
    if value.contains("tempo limite") || value.contains("timeout") {
        "timeout"
    } else if value.contains("instalado") || value.contains("not found") {
        "model_unavailable"
    } else if value.contains("confirmada") {
        "confirmation_failed"
    } else if value.contains("resposta invalida") {
        "malformed_response"
    } else if value.contains("andamento") {
        "concurrent_operation"
    } else if value.contains("status") {
        "http_error"
    } else {
        "connection_or_configuration"
    }
}

fn safe_log_value(value: &str) -> String {
    value
        .chars()
        .filter(|character| {
            character.is_ascii_alphanumeric() || ['-', '_', '.', ':'].contains(character)
        })
        .take(120)
        .collect()
}

pub async fn chat(
    endpoint: &str,
    model: &str,
    messages: Vec<OllamaMessage>,
    timeout_seconds: u64,
    generation_profile: &str,
) -> Result<OllamaChatResult, String> {
    chat_with_format(
        endpoint,
        model,
        messages,
        timeout_seconds,
        generation_profile,
        None,
    )
    .await
}

pub async fn chat_with_format(
    endpoint: &str,
    model: &str,
    messages: Vec<OllamaMessage>,
    timeout_seconds: u64,
    generation_profile: &str,
    structured_output_schema: Option<serde_json::Value>,
) -> Result<OllamaChatResult, String> {
    if LIFECYCLE_OPERATION_ACTIVE.load(Ordering::Acquire) {
        return Err("Aguarde a operacao de ativacao ou desativacao da IA terminar.".into());
    }
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
    let options = generation_options(generation_profile)?;
    let message_count = messages.len();
    let input_characters = messages
        .iter()
        .map(|message| message.content.chars().count())
        .sum::<usize>();
    let request_started = std::time::Instant::now();
    let mut request_body = serde_json::json!({
        "model": model.trim(), "messages": messages, "stream": false,
        "think": false,
        "options": options
    });
    if MODEL_MANUALLY_PINNED.load(Ordering::Acquire) {
        request_body["keep_alive"] = serde_json::json!(-1);
    }
    if let Some(schema) = structured_output_schema {
        request_body["format"] = schema;
    }
    let response = send_chat(
        &endpoint,
        model,
        &request_body,
        timeout_seconds,
        request_started,
        message_count,
        input_characters,
    )
    .await?;
    let payload = response
        .json::<ChatResponse>()
        .await
        .map_err(|_| "Resposta de chat inválida recebida do Ollama".to_string())?;
    if payload.message.content.trim().is_empty() {
        return Err("Ollama retornou uma resposta vazia".into());
    }
    eprintln!(
        "[AI_CORE] chat status=success latency_ms={} messages={} input_chars={}",
        request_started.elapsed().as_millis(),
        message_count,
        input_characters,
    );
    Ok(OllamaChatResult {
        content: payload.message.content.trim().into(),
        model: payload.model,
        truncated: payload.done_reason.as_deref() == Some("length"),
    })
}

/// Uma rodada do loop de agente: envia as ferramentas disponiveis e devolve
/// texto ou as chamadas de ferramenta escolhidas pelo modelo. A execucao das
/// ferramentas fica no AI Core, nunca aqui.
pub async fn chat_with_tools(
    endpoint: &str,
    model: &str,
    messages: Vec<OllamaAgentMessage>,
    tools: Vec<serde_json::Value>,
    timeout_seconds: u64,
) -> Result<OllamaAgentResult, String> {
    if LIFECYCLE_OPERATION_ACTIVE.load(Ordering::Acquire) {
        return Err("Aguarde a operacao de ativacao ou desativacao da IA terminar.".into());
    }
    let endpoint = normalize_endpoint(endpoint)?;
    validate_model(model)?;
    if messages.is_empty() || messages.len() > AGENT_MESSAGE_LIMIT || tools.len() > AGENT_TOOL_LIMIT {
        return Err("Quantidade de mensagens ou ferramentas inválida para o AI Core".into());
    }
    if messages.iter().any(|message| {
        let empty_allowed = message.role == "assistant" && !message.tool_calls.is_empty();
        !["system", "user", "assistant", "tool"].contains(&message.role.as_str())
            || (message.content.trim().is_empty() && !empty_allowed)
            || message.content.chars().count() > 50_000
    }) {
        return Err("Conteúdo de conversa inválido para o AI Core".into());
    }
    let message_count = messages.len();
    let input_characters = messages
        .iter()
        .map(|message| message.content.chars().count())
        .sum::<usize>();
    let request_started = std::time::Instant::now();
    let mut request_body = serde_json::json!({
        "model": model.trim(), "messages": messages, "stream": false,
        "think": false,
        "options": generation_options("agent")?
    });
    if !tools.is_empty() {
        request_body["tools"] = serde_json::json!(tools);
    }
    if MODEL_MANUALLY_PINNED.load(Ordering::Acquire) {
        request_body["keep_alive"] = serde_json::json!(-1);
    }
    let response = send_chat(
        &endpoint,
        model,
        &request_body,
        timeout_seconds,
        request_started,
        message_count,
        input_characters,
    )
    .await?;
    let payload = response
        .json::<AgentChatResponse>()
        .await
        .map_err(|_| "Resposta de chat inválida recebida do Ollama".to_string())?;
    if payload.message.content.trim().is_empty() && payload.message.tool_calls.is_empty() {
        return Err("Ollama retornou uma resposta vazia".into());
    }
    eprintln!(
        "[AI_CORE] agent status=success latency_ms={} messages={} input_chars={} tool_calls={}",
        request_started.elapsed().as_millis(),
        message_count,
        input_characters,
        payload.message.tool_calls.len(),
    );
    Ok(OllamaAgentResult {
        content: payload.message.content.trim().into(),
        model: payload.model,
        truncated: payload.done_reason.as_deref() == Some("length"),
        tool_calls: payload.message.tool_calls,
    })
}

async fn send_chat(
    endpoint: &str,
    model: &str,
    request_body: &serde_json::Value,
    timeout_seconds: u64,
    request_started: std::time::Instant,
    message_count: usize,
    input_characters: usize,
) -> Result<reqwest::Response, String> {
    let response = client(timeout_seconds)?
        .post(format!("{endpoint}/api/chat"))
        .json(request_body)
        .send()
        .await
        .map_err(|error| {
            eprintln!(
                "[AI_CORE] chat status={} latency_ms={} messages={} input_chars={}",
                if error.is_timeout() {
                    "timeout"
                } else {
                    "connection_error"
                },
                request_started.elapsed().as_millis(),
                message_count,
                input_characters,
            );
            request_error(&error)
        })?;
    if !response.status().is_success() {
        let status = response.status();
        let detail = response.text().await.unwrap_or_default();
        eprintln!(
            "[AI_CORE] chat status=http_{} latency_ms={} messages={} input_chars={}",
            status.as_u16(),
            request_started.elapsed().as_millis(),
            message_count,
            input_characters,
        );
        return Err(response_error(status.as_u16(), &detail, model));
    }
    Ok(response)
}

fn generation_options(generation_profile: &str) -> Result<serde_json::Value, String> {
    Ok(match generation_profile {
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
        "study" => serde_json::json!({
            "temperature": 0.3,
            "top_p": 0.88,
            "top_k": 35,
            "repeat_penalty": 1.18,
            "repeat_last_n": 160,
            "num_predict": 900
        }),
        "study-structured" => serde_json::json!({
            "temperature": 0.18,
            "top_p": 0.82,
            "top_k": 25,
            "repeat_penalty": 1.15,
            "repeat_last_n": 160,
            "num_predict": 1400
        }),
        "market-agent" => serde_json::json!({
            "temperature": 0.1,
            "top_p": 0.8,
            "top_k": 20,
            "repeat_penalty": 1.15,
            "repeat_last_n": 96,
            "num_predict": 120,
            "seed": 42
        }),
        "agent" => serde_json::json!({
            "temperature": 0.2,
            "top_p": 0.85,
            "top_k": 30,
            "repeat_penalty": 1.12,
            "repeat_last_n": 128,
            "num_predict": 700
        }),
        _ => return Err("Perfil de geração inválido para o AI Core".into()),
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
    let loopback = allowed_ollama_host(host);
    if !loopback {
        return Err("Por seguranca, o endpoint do Ollama deve apontar para localhost, rede privada ou Tailscale".into());
    }
    if !url.path().is_empty() && url.path() != "/" {
        return Err("O endpoint deve apontar para a raiz da API do Ollama".into());
    }
    Ok(trimmed.into())
}

fn allowed_ollama_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    if let Ok(address) = host.parse::<IpAddr>() {
        return match address {
            IpAddr::V4(value) => {
                let octets = value.octets();
                value.is_loopback()
                    || value.is_private()
                    || value.is_link_local()
                    || (octets[0] == 100 && (64..=127).contains(&octets[1]))
            }
            IpAddr::V6(value) => {
                value.is_loopback() || value.is_unique_local() || value.is_unicast_link_local()
            }
        };
    }
    let lower = host.to_ascii_lowercase();
    let valid_name = lower
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '.');
    valid_name && !lower.is_empty() && (!lower.contains('.') || lower.ends_with(".ts.net"))
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
    } else if detail.contains("does not support tools") {
        format!("O modelo '{}' não suporta ferramentas", model.trim())
    } else {
        format!("Ollama respondeu com status {status}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{Arc, Mutex},
        thread,
    };

    fn mock_ollama(replies: Vec<(u16, &'static str)>) -> (String, Arc<Mutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        thread::spawn(move || {
            for (status, body) in replies {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut bytes = vec![0u8; 16_384];
                let count = stream.read(&mut bytes).unwrap_or_default();
                captured
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&bytes[..count]).into_owned());
                let reason = if status == 200 { "OK" } else { "ERROR" };
                let response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (endpoint, requests)
    }

    #[test]
    fn endpoint_is_restricted_to_local_ollama() {
        assert_eq!(
            normalize_endpoint("http://localhost:11434/").unwrap(),
            "http://localhost:11434"
        );
        assert!(normalize_endpoint("http://127.0.0.1:11434").is_ok());
        assert!(normalize_endpoint("http://100.100.20.30:11434").is_ok());
        assert!(normalize_endpoint("http://azriel-ollama.tailnet.ts.net:11434").is_ok());
        assert!(normalize_endpoint("http://ollama-server:11434").is_ok());
        assert!(normalize_endpoint("https://example.com").is_err());
        assert!(normalize_endpoint("http://8.8.8.8:11434").is_err());
        assert!(normalize_endpoint("http://example.com:11434").is_err());
        assert!(normalize_endpoint("http://localhost:11434/api").is_err());
    }

    #[test]
    fn configured_model_matching_never_accepts_another_model() {
        assert!(model_matches("qwen3.5:4b", "qwen3.5:4b"));
        assert!(model_matches("qwen3.5", "qwen3.5:latest"));
        assert!(!model_matches("qwen3.5:4b", "qwen2.5:4b"));
        assert!(!model_matches("qwen3.5:4b", "qwen3.5:8b"));
    }

    #[test]
    fn lifecycle_guard_rejects_concurrent_transitions_and_releases_after_drop() {
        let first = LifecycleGuard::acquire().unwrap();
        assert!(LifecycleGuard::acquire().is_err());
        drop(first);
        assert!(LifecycleGuard::acquire().is_ok());
    }

    #[test]
    fn lifecycle_status_distinguishes_loaded_unloaded_other_model_and_malformed() {
        let (loaded_endpoint, _) = mock_ollama(vec![(
            200,
            r#"{"models":[{"name":"qwen3.5:4b","model":"qwen3.5:4b","expires_at":"2099-01-01T00:00:00Z"}]}"#,
        )]);
        let loaded = tauri::async_runtime::block_on(model_lifecycle_status(
            &loaded_endpoint,
            "qwen3.5:4b",
            5,
        ))
        .unwrap();
        assert!(loaded.loaded);
        assert_eq!(loaded.model_status, "LOADED");

        let (other_endpoint, _) = mock_ollama(vec![
            (
                200,
                r#"{"models":[{"name":"llama3:8b","model":"llama3:8b","expires_at":null}]}"#,
            ),
            (200, r#"{"models":[{"name":"qwen3.5:4b"}]}"#),
        ]);
        let other = tauri::async_runtime::block_on(model_lifecycle_status(
            &other_endpoint,
            "qwen3.5:4b",
            5,
        ))
        .unwrap();
        assert!(!other.loaded);
        assert_eq!(other.model_status, "UNLOADED");

        let (missing_endpoint, _) =
            mock_ollama(vec![(200, r#"{"models":[]}"#), (200, r#"{"models":[]}"#)]);
        let missing = tauri::async_runtime::block_on(model_lifecycle_status(
            &missing_endpoint,
            "qwen3.5:4b",
            5,
        ))
        .unwrap();
        assert_eq!(missing.model_status, "NOT_AVAILABLE");

        let (malformed_endpoint, _) = mock_ollama(vec![(200, "not-json")]);
        let malformed = tauri::async_runtime::block_on(model_lifecycle_status(
            &malformed_endpoint,
            "qwen3.5:4b",
            5,
        ))
        .unwrap();
        assert_eq!(malformed.server_status, "ONLINE");
        assert_eq!(malformed.model_status, "ERROR");
    }

    #[test]
    fn load_and_unload_use_keep_alive_and_require_ps_confirmation() {
        let (load_endpoint, load_requests) = mock_ollama(vec![
            (200, "{}"),
            (
                200,
                r#"{"models":[{"name":"qwen3.5:4b","model":"qwen3.5:4b","expires_at":null}]}"#,
            ),
        ]);
        let loaded = tauri::async_runtime::block_on(change_model_lifecycle_with_policy(
            &load_endpoint,
            "qwen3.5:4b",
            5,
            -1,
            true,
            1,
            Duration::ZERO,
        ))
        .unwrap();
        assert!(loaded.loaded);
        assert!(load_requests.lock().unwrap()[0].contains(r#""keep_alive":-1"#));

        let (unload_endpoint, unload_requests) = mock_ollama(vec![
            (200, "{}"),
            (200, r#"{"models":[]}"#),
            (200, r#"{"models":[{"name":"qwen3.5:4b"}]}"#),
        ]);
        let unloaded = tauri::async_runtime::block_on(change_model_lifecycle_with_policy(
            &unload_endpoint,
            "qwen3.5:4b",
            5,
            0,
            false,
            1,
            Duration::ZERO,
        ))
        .unwrap();
        assert!(!unloaded.loaded);
        assert_eq!(unloaded.server_status, "ONLINE");
        assert!(unload_requests.lock().unwrap()[0].contains(r#""keep_alive":0"#));
    }

    #[test]
    fn lifecycle_rejects_post_failure_and_unconfirmed_transition() {
        let (post_failure, _) =
            mock_ollama(vec![(404, r#"{"error":"model 'missing' not found"}"#)]);
        let error = tauri::async_runtime::block_on(change_model_lifecycle_with_policy(
            &post_failure,
            "missing",
            5,
            -1,
            true,
            1,
            Duration::ZERO,
        ))
        .unwrap_err();
        assert!(error.contains("modelo") && error.contains("instalado"));

        let (unconfirmed, _) = mock_ollama(vec![
            (200, "{}"),
            (200, r#"{"models":[]}"#),
            (200, r#"{"models":[{"name":"qwen3.5:4b"}]}"#),
        ]);
        let error = tauri::async_runtime::block_on(change_model_lifecycle_with_policy(
            &unconfirmed,
            "qwen3.5:4b",
            5,
            -1,
            true,
            1,
            Duration::ZERO,
        ))
        .unwrap_err();
        assert!(error.contains("nao foi confirmada"));
    }

    #[test]
    fn model_not_found_has_a_clear_error() {
        let message = response_error(404, r#"{"error":"model 'missing' not found"}"#, "missing");
        assert_eq!(message, "O modelo 'missing' não está instalado no Ollama");
    }

    #[test]
    fn chat_disables_hidden_thinking_and_returns_visible_content() {
        let (endpoint, requests) = mock_ollama(vec![(
            200,
            r#"{"model":"qwen3.5:4b","message":{"role":"assistant","content":"Olá!"},"done_reason":"stop"}"#,
        )]);

        let result = tauri::async_runtime::block_on(chat(
            &endpoint,
            "qwen3.5:4b",
            vec![OllamaMessage {
                role: "user".into(),
                content: "Olá".into(),
            }],
            5,
            "standard",
        ))
        .unwrap();

        assert_eq!(result.content, "Olá!");
        assert!(requests.lock().unwrap()[0].contains(r#""think":false"#));
    }

    #[test]
    fn agent_chat_sends_tools_and_returns_tool_calls() {
        let (endpoint, requests) = mock_ollama(vec![(
            200,
            r#"{"model":"qwen3.5:4b","message":{"role":"assistant","content":"","tool_calls":[{"function":{"name":"get_today_tasks","arguments":{}}}]},"done_reason":"stop"}"#,
        )]);
        let tool = serde_json::json!({"type":"function","function":{"name":"get_today_tasks","description":"Tarefas de hoje","parameters":{"type":"object","properties":{}}}});
        let result = tauri::async_runtime::block_on(chat_with_tools(
            &endpoint,
            "qwen3.5:4b",
            vec![OllamaAgentMessage {
                role: "user".into(),
                content: "O que tenho hoje?".into(),
                tool_calls: vec![],
                tool_name: None,
            }],
            vec![tool],
            5,
        ))
        .unwrap();

        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.tool_calls[0].function.name, "get_today_tasks");
        let request = requests.lock().unwrap()[0].clone();
        assert!(request.contains(r#""tools":"#));
        assert!(!request.contains(r#""tool_calls""#));
    }

    #[test]
    fn agent_chat_accepts_tool_results_and_rejects_unknown_roles() {
        let (endpoint, requests) = mock_ollama(vec![(
            200,
            r#"{"model":"qwen3.5:4b","message":{"role":"assistant","content":"Você tem 2 tarefas."},"done_reason":"stop"}"#,
        )]);
        let call = OllamaToolCall {
            function: OllamaToolFunction {
                name: "get_today_tasks".into(),
                arguments: serde_json::json!({}),
            },
            id: None,
            extra_content: None,
        };
        let messages = vec![
            OllamaAgentMessage { role: "user".into(), content: "Hoje?".into(), tool_calls: vec![], tool_name: None },
            OllamaAgentMessage { role: "assistant".into(), content: String::new(), tool_calls: vec![call], tool_name: None },
            OllamaAgentMessage { role: "tool".into(), content: "[]".into(), tool_calls: vec![], tool_name: Some("get_today_tasks".into()) },
        ];
        let result = tauri::async_runtime::block_on(chat_with_tools(&endpoint, "qwen3.5:4b", messages, vec![], 5)).unwrap();
        assert_eq!(result.content, "Você tem 2 tarefas.");
        assert!(result.tool_calls.is_empty());
        let request = requests.lock().unwrap()[0].clone();
        assert!(!request.contains(r#""tools":"#));
        assert!(request.contains(r#""tool_name":"get_today_tasks""#));
        assert!(!request.contains("extra_content"));

        let invalid = vec![OllamaAgentMessage { role: "root".into(), content: "x".into(), tool_calls: vec![], tool_name: None }];
        assert!(tauri::async_runtime::block_on(chat_with_tools("http://127.0.0.1:9", "qwen3.5:4b", invalid, vec![], 5)).is_err());
    }

    #[test]
    fn model_without_tool_support_has_a_clear_error() {
        let message = response_error(400, r#"{"error":"registry.ollama.ai/library/gemma:2b does not support tools"}"#, "gemma:2b");
        assert_eq!(message, "O modelo 'gemma:2b' não suporta ferramentas");
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
