//! Programação rápida: o modelo local escreve os arquivos de uma vez, em tempo real.
//! Sem identidade, memórias nem ferramentas do Azriel: só a instrução de código e o pedido.
//! Medido no PC do Senhor (i3, 4 threads): qwen3:1.7b gera ~11 tokens/s; hello world em ~16 s.

use crate::ollama;
use reqwest::Client;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::ipc::Channel;

/// Cada geração tem um número; ESC avança o contador e a geração em andamento para na hora.
static GENERATION: AtomicU64 = AtomicU64::new(0);
const MAX_PROMPT_CHARS: usize = 40_000;
/// Tempo máximo sem receber nenhum pedaço do Ollama antes de desistir.
const STALL_SECONDS: u64 = 120;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum QuickEvent {
    /// Texto novo do modelo (vai aparecendo na tela).
    Token { text: String },
    /// Fim: quantos tokens gerou e a que velocidade.
    Done { tokens: u64, tokens_per_second: f64, seconds: f64 },
}

/// Opções medidas para CPU fraca: contexto curto (gera bem mais rápido), todas as threads, modelo sempre carregado.
pub fn request_body(model: &str, system: &str, user: &str, prompt_chars: usize) -> Value {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 16);
    // ~3,5 caracteres por token; folga para a resposta. Contexto menor = geração mais rápida.
    let needed = prompt_chars / 3 + 2_500;
    let num_ctx = if needed <= 4_096 { 4_096 } else if needed <= 8_192 { 8_192 } else { 16_384 };
    json!({
        "model": model,
        "stream": true,
        "think": false,
        "keep_alive": -1,
        "messages": [{"role": "system", "content": system}, {"role": "user", "content": user}],
        "options": {"num_ctx": num_ctx, "num_predict": 4_096, "temperature": 0.2, "top_p": 0.9, "num_thread": threads}
    })
}

/// Lê as linhas NDJSON que já chegaram; devolve (textos, terminou, estatísticas) e guarda o resto incompleto.
pub fn drain_lines(buffer: &mut String) -> (Vec<String>, Option<(u64, u64)>, Option<String>) {
    let mut texts = Vec::new();
    let mut done = None;
    let mut error = None;
    while let Some(end) = buffer.find('\n') {
        let line: String = buffer.drain(..=end).collect();
        let Ok(value) = serde_json::from_str::<Value>(line.trim()) else { continue };
        if let Some(message) = value["error"].as_str() { error = Some(message.to_string()); }
        if let Some(text) = value["message"]["content"].as_str().filter(|text| !text.is_empty()) { texts.push(text.to_string()); }
        if value["done"].as_bool() == Some(true) {
            done = Some((value["eval_count"].as_u64().unwrap_or(0), value["eval_duration"].as_u64().unwrap_or(0)));
        }
    }
    (texts, done, error)
}

#[tauri::command]
pub async fn quick_code_stream(endpoint: String, model: String, system: String, prompt: String, events: Channel<QuickEvent>) -> Result<String, String> {
    let endpoint = ollama::normalize_endpoint(&endpoint)?;
    let model = model.trim();
    if model.is_empty() || model.chars().count() > 120 { return Err("Escolha um modelo local".into()); }
    if prompt.trim().is_empty() { return Err("Diga o que programar".into()); }
    if system.len() + prompt.len() > MAX_PROMPT_CHARS { return Err("Pedido grande demais para a programação rápida".into()); }
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let client = Client::builder().connect_timeout(Duration::from_secs(5)).build().map_err(|_| "Não foi possível preparar o cliente do Ollama".to_string())?;
    let mut response = client
        .post(format!("{endpoint}/api/chat"))
        .json(&request_body(model, &system, &prompt, system.len() + prompt.len()))
        .send()
        .await
        .map_err(|_| "Ollama não encontrado. Ele está aberto?".to_string())?;
    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        let detail = serde_json::from_str::<Value>(&body).ok().and_then(|value| value["error"].as_str().map(String::from)).unwrap_or_default();
        return Err(if detail.contains("not found") { format!("O modelo {model} não está instalado no Ollama") } else { format!("O Ollama recusou o pedido. {detail}") });
    }
    let started = Instant::now();
    let mut buffer = String::new();
    let mut full = String::new();
    loop {
        // ESC: larga a conexão; o Ollama percebe e para de gerar.
        if GENERATION.load(Ordering::SeqCst) != generation { return Err("Interrompido pelo Senhor.".into()); }
        let chunk = tokio::time::timeout(Duration::from_secs(STALL_SECONDS), response.chunk())
            .await
            .map_err(|_| format!("O Ollama ficou {STALL_SECONDS} s sem responder"))?
            .map_err(|_| "A conexão com o Ollama caiu".to_string())?;
        let Some(chunk) = chunk else { break };
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        let (texts, done, error) = drain_lines(&mut buffer);
        if let Some(error) = error { return Err(format!("Ollama: {error}")); }
        for text in texts {
            full.push_str(&text);
            let _ = events.send(QuickEvent::Token { text });
        }
        if let Some((tokens, duration_ns)) = done {
            let generation_seconds = duration_ns as f64 / 1e9;
            let _ = events.send(QuickEvent::Done {
                tokens,
                tokens_per_second: if generation_seconds > 0.0 { (tokens as f64 / generation_seconds * 10.0).round() / 10.0 } else { 0.0 },
                seconds: (started.elapsed().as_secs_f64() * 10.0).round() / 10.0,
            });
            break;
        }
    }
    eprintln!("[QUICK_CODE] model={model} chars={} latency_ms={}", full.len(), started.elapsed().as_millis());
    Ok(full)
}

#[tauri::command]
pub fn quick_code_cancel() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_tuned_for_speed() {
        let body = request_body("qwen3:1.7b", "sys", "faça um hello world", 200);
        assert_eq!(body["stream"], true);
        assert_eq!(body["think"], false, "sem raciocínio escondido: só o código");
        assert_eq!(body["keep_alive"], -1, "modelo fica carregado, sem esperar recarga");
        assert_eq!(body["options"]["num_ctx"], 4_096, "pedido pequeno usa contexto curto (gera mais rápido)");
        assert!(body["options"]["num_thread"].as_u64().unwrap() >= 1);
        assert_eq!(request_body("m", "s", "u", 20_000)["options"]["num_ctx"], 16_384);
        assert_eq!(request_body("m", "s", "u", 9_000)["options"]["num_ctx"], 8_192);
    }

    #[test]
    fn streamed_lines_are_split_even_when_cut_in_the_middle() {
        let mut buffer = String::from("{\"message\":{\"content\":\"ARQ\"},\"done\":false}\n{\"message\":{\"content\":\"UIVO\"},\"do");
        let (texts, done, error) = drain_lines(&mut buffer);
        assert_eq!(texts, vec!["ARQ"]);
        assert!(done.is_none() && error.is_none());
        buffer.push_str("ne\":false}\n{\"message\":{\"content\":\"\"},\"done\":true,\"eval_count\":120,\"eval_duration\":10000000000}\n");
        let (texts, done, _) = drain_lines(&mut buffer);
        assert_eq!(texts, vec!["UIVO"]);
        assert_eq!(done, Some((120, 10_000_000_000)));
        assert!(buffer.is_empty());
        let mut failed = String::from("{\"error\":\"model 'x' not found\"}\n");
        assert_eq!(drain_lines(&mut failed).2.as_deref(), Some("model 'x' not found"));
    }
}
