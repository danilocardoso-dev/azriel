//! Motor remoto do AI Core: Gemini pela API compatível com OpenAI do Google AI Studio.
//! O endpoint é fixo no código e a chave nunca é registrada em log nem devolvida à interface.

use crate::ollama::{OllamaAgentMessage, OllamaAgentResult, OllamaToolCall, OllamaToolFunction};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::VecDeque, time::Duration};

const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";
const MESSAGE_LIMIT: usize = 60;
const TOOL_LIMIT: usize = 40;

#[derive(Deserialize)]
struct CompletionResponse {
    #[serde(default)]
    model: String,
    choices: Vec<CompletionChoice>,
}
#[derive(Deserialize)]
struct CompletionChoice {
    message: CompletionMessage,
    finish_reason: Option<String>,
}
#[derive(Deserialize)]
struct CompletionMessage {
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<CompletionToolCall>,
}
#[derive(Deserialize)]
struct CompletionToolCall {
    #[serde(default)]
    id: Option<String>,
    function: CompletionFunction,
    /// Contém `google.thought_signature`, obrigatório de volta nos modelos Gemini 3.
    #[serde(default)]
    extra_content: Option<Value>,
}
#[derive(Deserialize)]
struct CompletionFunction {
    name: String,
    #[serde(default)]
    arguments: String,
}
#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}
#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

pub async fn chat_with_tools(
    api_key: &str,
    model: &str,
    messages: Vec<OllamaAgentMessage>,
    tools: Vec<Value>,
    timeout_seconds: u64,
) -> Result<OllamaAgentResult, String> {
    validate_model(model)?;
    if messages.is_empty() || messages.len() > MESSAGE_LIMIT || tools.len() > TOOL_LIMIT {
        return Err("Quantidade de mensagens ou ferramentas inválida para o AI Core".into());
    }
    if messages.iter().any(|message| !["system", "user", "assistant", "tool"].contains(&message.role.as_str()) || message.content.chars().count() > 50_000) {
        return Err("Conteúdo de conversa inválido para o AI Core".into());
    }
    let mut body = json!({
        "model": model.trim(),
        "messages": to_openai_messages(&messages),
        "temperature": 0.2,
        "max_tokens": 1_200
    });
    if !tools.is_empty() {
        body["tools"] = json!(tools.into_iter().map(gemini_tool).collect::<Vec<_>>());
    }
    let started = std::time::Instant::now();
    let response = client(timeout_seconds)?
        .post(format!("{GEMINI_BASE_URL}/chat/completions"))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| request_error(&error))?;
    let status = response.status();
    if !status.is_success() {
        let detail = provider_detail(&response.text().await.unwrap_or_default());
        eprintln!("[AI_CORE] provider=GEMINI status=http_{} latency_ms={} detail={}", status.as_u16(), started.elapsed().as_millis(), detail.as_deref().unwrap_or("none"));
        return Err(with_detail(status_error(status.as_u16()), detail));
    }
    let payload = response
        .json::<CompletionResponse>()
        .await
        .map_err(|_| "Resposta inválida recebida do Gemini".to_string())?;
    let result = parse_completion(payload, model)?;
    eprintln!(
        "[AI_CORE] provider=GEMINI status=success latency_ms={} messages={} tool_calls={}",
        started.elapsed().as_millis(),
        messages.len(),
        result.tool_calls.len(),
    );
    Ok(result)
}

pub async fn list_models(api_key: &str, timeout_seconds: u64) -> Result<Vec<String>, String> {
    let response = client(timeout_seconds)?
        .get(format!("{GEMINI_BASE_URL}/models"))
        .bearer_auth(api_key)
        .send()
        .await
        .map_err(|error| request_error(&error))?;
    let status = response.status();
    if !status.is_success() {
        let detail = provider_detail(&response.text().await.unwrap_or_default());
        return Err(with_detail(status_error(status.as_u16()), detail));
    }
    let payload = response
        .json::<ModelsResponse>()
        .await
        .map_err(|_| "Lista de modelos inválida recebida do Gemini".to_string())?;
    Ok(chat_models(payload.data.into_iter().map(|entry| entry.id)))
}

/// O Gemini rejeita (400) parâmetros do tipo objeto sem propriedades; ferramentas
/// sem argumentos são enviadas sem o campo `parameters`.
fn gemini_tool(mut tool: Value) -> Value {
    let empty = tool["function"]["parameters"]["properties"]
        .as_object()
        .is_some_and(|properties| properties.is_empty());
    if empty {
        if let Some(function) = tool["function"].as_object_mut() {
            function.remove("parameters");
        }
    }
    tool
}

/// Extrai a mensagem de erro do Google (formato `[{error:{message}}]` ou `{error:{message}}`),
/// limitada e sem quebras de linha, para exibir ao operador.
fn provider_detail(body: &str) -> Option<String> {
    let value: Value = serde_json::from_str(body).ok()?;
    let error = if value.is_array() { &value[0]["error"] } else { &value["error"] };
    let message = error["message"].as_str()?.split_whitespace().collect::<Vec<_>>().join(" ");
    (!message.is_empty()).then(|| message.chars().take(240).collect())
}

fn with_detail(message: String, detail: Option<String>) -> String {
    match detail {
        Some(detail) => format!("{message} Detalhe: {detail}"),
        None => message,
    }
}

/// Mantém só modelos de conversa: sem embeddings, voz, imagem etc.
fn chat_models(ids: impl Iterator<Item = String>) -> Vec<String> {
    let mut models: Vec<String> = ids
        .map(|id| id.trim_start_matches("models/").to_string())
        .filter(|id| id.starts_with("gemini") && !["embedding", "tts", "image", "audio", "live"].iter().any(|word| id.contains(word)))
        .collect();
    models.sort();
    models.dedup();
    models
}

/// Converte o histórico no formato do Ollama para o formato OpenAI, que exige
/// IDs ligando cada resultado de ferramenta à chamada correspondente.
fn to_openai_messages(messages: &[OllamaAgentMessage]) -> Vec<Value> {
    let mut pending: VecDeque<String> = VecDeque::new();
    let mut sequence = 0usize;
    messages
        .iter()
        .map(|message| match message.role.as_str() {
            "assistant" if !message.tool_calls.is_empty() => {
                let calls: Vec<Value> = message
                    .tool_calls
                    .iter()
                    .map(|call| {
                        sequence += 1;
                        let id = call.id.clone().unwrap_or_else(|| format!("call_{sequence}"));
                        pending.push_back(id.clone());
                        let mut converted = json!({"id": id, "type": "function", "function": {"name": call.function.name, "arguments": call.function.arguments.to_string()}});
                        if let Some(extra) = &call.extra_content {
                            converted["extra_content"] = extra.clone();
                        }
                        converted
                    })
                    .collect();
                json!({"role": "assistant", "content": if message.content.is_empty() { Value::Null } else { json!(message.content) }, "tool_calls": calls})
            }
            "tool" => {
                let id = pending.pop_front().unwrap_or_else(|| {
                    sequence += 1;
                    format!("call_{sequence}")
                });
                json!({"role": "tool", "tool_call_id": id, "content": message.content})
            }
            role => json!({"role": role, "content": message.content}),
        })
        .collect()
}

fn parse_completion(payload: CompletionResponse, requested_model: &str) -> Result<OllamaAgentResult, String> {
    let choice = payload
        .choices
        .into_iter()
        .next()
        .ok_or_else(|| "O Gemini não retornou nenhuma resposta".to_string())?;
    let content = choice.message.content.unwrap_or_default().trim().to_string();
    let tool_calls: Vec<OllamaToolCall> = choice
        .message
        .tool_calls
        .into_iter()
        .map(|call| OllamaToolCall {
            function: OllamaToolFunction {
                name: call.function.name,
                arguments: serde_json::from_str(&call.function.arguments).unwrap_or_else(|_| json!({})),
            },
            id: call.id.filter(|id| !id.is_empty()),
            extra_content: call.extra_content,
        })
        .collect();
    if content.is_empty() && tool_calls.is_empty() {
        return Err("O Gemini retornou uma resposta vazia".into());
    }
    Ok(OllamaAgentResult {
        content,
        model: if payload.model.is_empty() { requested_model.trim().into() } else { payload.model },
        truncated: choice.finish_reason.as_deref() == Some("length"),
        tool_calls,
    })
}

fn validate_model(model: &str) -> Result<(), String> {
    let model = model.trim();
    if model.is_empty() {
        return Err("Selecione um modelo do Gemini".into());
    }
    if model.chars().count() > 120 || !model.chars().all(|character| character.is_ascii_alphanumeric() || ['-', '_', '.', ':', '/'].contains(&character)) {
        return Err("Nome de modelo inválido".into());
    }
    Ok(())
}

fn client(timeout_seconds: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_seconds.clamp(5, 180)))
        .build()
        .map_err(|_| "Não foi possível preparar o cliente do Gemini".into())
}

fn request_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "O Gemini excedeu o tempo limite da solicitação".into()
    } else {
        "Não foi possível conectar ao Gemini. Verifique a internet.".into()
    }
}

fn status_error(status: u16) -> String {
    match status {
        400 => "O Gemini recusou a solicitação (400). Verifique o modelo selecionado.".into(),
        401 | 403 => "Chave do Gemini inválida ou sem permissão".into(),
        404 => "Modelo do Gemini não encontrado".into(),
        429 => "Limite de uso do Gemini atingido. Aguarde um pouco e tente de novo.".into(),
        _ => format!("Gemini respondeu com status {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(role: &str, content: &str) -> OllamaAgentMessage {
        OllamaAgentMessage { role: role.into(), content: content.into(), tool_calls: vec![], tool_name: None }
    }

    #[test]
    fn tool_results_are_linked_to_their_calls() {
        let call = |name: &str| OllamaToolCall { function: OllamaToolFunction { name: name.into(), arguments: json!({}) }, id: None, extra_content: None };
        let mut assistant = message("assistant", "");
        assistant.tool_calls = vec![call("get_today_tasks"), call("get_cpu_status")];
        let converted = to_openai_messages(&[message("user", "Hoje?"), assistant, message("tool", "[]"), message("tool", "{\"usagePercent\":12}")]);
        assert_eq!(converted[1]["tool_calls"][0]["id"], "call_1");
        assert_eq!(converted[1]["tool_calls"][1]["function"]["arguments"], "{}");
        assert!(converted[1]["content"].is_null());
        assert_eq!(converted[2]["tool_call_id"], "call_1");
        assert_eq!(converted[3]["tool_call_id"], "call_2");
        assert_eq!(converted[0], json!({"role": "user", "content": "Hoje?"}));
    }

    #[test]
    fn completion_with_tool_calls_is_parsed() {
        let payload: CompletionResponse = serde_json::from_str(r#"{"model":"gemini-2.5-flash","choices":[{"message":{"role":"assistant","content":null,"tool_calls":[{"id":"x","type":"function","function":{"name":"get_today_tasks","arguments":"{}"}}]},"finish_reason":"tool_calls"}]}"#).unwrap();
        let result = parse_completion(payload, "gemini-2.5-flash").unwrap();
        assert_eq!(result.tool_calls[0].function.name, "get_today_tasks");
        assert_eq!(result.content, "");

        assert_eq!(result.tool_calls[0].id.as_deref(), Some("x"));

        // Gemini 3: a assinatura recebida precisa voltar igual, com o mesmo id, na próxima rodada.
        let signed: CompletionResponse = serde_json::from_str(r#"{"choices":[{"message":{"content":null,"tool_calls":[{"id":"fc-1","type":"function","function":{"name":"get_daily_operations_summary","arguments":"{}"},"extra_content":{"google":{"thought_signature":"SIG=="}}}]},"finish_reason":"tool_calls"}]}"#).unwrap();
        let signed = parse_completion(signed, "gemini-3-flash").unwrap();
        let assistant = OllamaAgentMessage { role: "assistant".into(), content: String::new(), tool_calls: signed.tool_calls, tool_name: None };
        let tool = OllamaAgentMessage { role: "tool".into(), content: "{}".into(), tool_calls: vec![], tool_name: Some("get_daily_operations_summary".into()) };
        let converted = to_openai_messages(&[assistant, tool]);
        assert_eq!(converted[0]["tool_calls"][0]["extra_content"]["google"]["thought_signature"], "SIG==");
        assert_eq!(converted[0]["tool_calls"][0]["id"], "fc-1");
        assert_eq!(converted[1]["tool_call_id"], "fc-1");

        let empty: CompletionResponse = serde_json::from_str(r#"{"choices":[{"message":{"content":"  "},"finish_reason":"stop"}]}"#).unwrap();
        assert!(parse_completion(empty, "gemini-2.5-flash").is_err());
    }

    #[test]
    fn tools_without_arguments_omit_parameters() {
        let empty = gemini_tool(json!({"type":"function","function":{"name":"get_today_tasks","description":"x","parameters":{"type":"object","properties":{}}}}));
        assert!(empty["function"].get("parameters").is_none());
        assert_eq!(empty["function"]["name"], "get_today_tasks");
        let with_args = gemini_tool(json!({"type":"function","function":{"name":"f","parameters":{"type":"object","properties":{"n":{"type":"number"}}}}}));
        assert!(with_args["function"]["parameters"]["properties"]["n"].is_object());
    }

    #[test]
    fn provider_error_detail_is_extracted() {
        let body = r#"[{"error":{"code":400,"message":"* GenerateContentRequest.tools[0].function_declarations[0].parameters.properties: should be non-empty for OBJECT type\n","status":"INVALID_ARGUMENT"}}]"#;
        assert!(provider_detail(body).unwrap().contains("should be non-empty"));
        assert_eq!(provider_detail(r#"{"error":{"message":"API key not valid."}}"#).unwrap(), "API key not valid.");
        assert!(provider_detail("<html>").is_none());
        assert_eq!(with_detail("Falha.".into(), None), "Falha.");
    }

    #[test]
    fn only_chat_models_are_listed() {
        let ids = ["models/gemini-2.5-flash", "models/gemini-embedding-001", "models/gemini-2.5-flash", "models/imagen-4", "gemini-2.5-pro", "models/gemini-2.5-flash-preview-tts"];
        assert_eq!(chat_models(ids.iter().map(|id| id.to_string())), vec!["gemini-2.5-flash", "gemini-2.5-pro"]);
    }

    #[test]
    fn errors_never_mention_the_key_and_explain_limits() {
        assert!(status_error(429).contains("Limite"));
        assert!(status_error(401).contains("Chave"));
        assert!(validate_model("").is_err());
        assert!(validate_model("gemini 2.5").is_err());
    }
}
