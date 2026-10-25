//! Motor autônomo: a cada ciclo lê mercado e conta, pede a decisão à IA, aplica os
//! limites de risco, executa na demo da BingX e registra tudo no SQLite.

use super::bingx::BingxClient;
use super::strategy::{self, Action, ContextInput, Decision, RiskConfig, RiskState, Side};
use crate::database::{ai_repository, trading_models::*, trading_repository, DatabaseState};
use crate::{gemini, ollama};
use serde::Serialize;
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

const SYSTEM_PROMPT: &str = include_str!("../../prompts/trading-agent-v2.txt");
const MAX_ENTRIES_PER_DAY: i64 = 12;
pub const UPDATE_EVENT: &str = "trading-update";

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub running: bool,
    pub cycle_in_progress: bool,
    pub last_cycle_at: Option<String>,
    pub next_cycle_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Default)]
pub struct TradingState {
    running: AtomicBool,
    wake: Notify,
    cycle: tokio::sync::Mutex<()>,
    info: Mutex<EngineInfo>,
}

impl TradingState {
    pub fn info(&self) -> EngineInfo {
        let mut info = self.info.lock().map(|info| info.clone()).unwrap_or_default();
        info.running = self.running.load(Ordering::Acquire);
        info
    }
    fn update(&self, change: impl FnOnce(&mut EngineInfo)) {
        if let Ok(mut info) = self.info.lock() {
            change(&mut info);
        }
    }
}

fn database<T>(app: &AppHandle, action: impl FnOnce(&rusqlite::Connection) -> Result<T, String>) -> Result<T, String> {
    let state = app.state::<DatabaseState>();
    let connection = state.connection.lock().map_err(|_| "O banco de dados está indisponível".to_string())?;
    action(&connection)
}

pub fn start(app: AppHandle) -> Result<(), String> {
    database(&app, |connection| trading_repository::credentials(connection).map(|_| ()))?;
    let state = app.state::<TradingState>();
    if state.running.swap(true, Ordering::AcqRel) {
        return Ok(());
    }
    database(&app, |connection| trading_repository::set_running(connection, true))?;
    state.update(|info| info.last_error = None);
    tauri::async_runtime::spawn(run_loop(app.clone()));
    let _ = app.emit(UPDATE_EVENT, ());
    Ok(())
}

pub fn stop(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<TradingState>();
    state.running.store(false, Ordering::Release);
    state.update(|info| info.next_cycle_at = None);
    state.wake.notify_one();
    database(app, |connection| trading_repository::set_running(connection, false))?;
    let _ = app.emit(UPDATE_EVENT, ());
    Ok(())
}

async fn run_loop(app: AppHandle) {
    loop {
        if !app.state::<TradingState>().running.load(Ordering::Acquire) {
            break;
        }
        let _ = run_cycle(&app).await;
        let state = app.state::<TradingState>();
        if !state.running.load(Ordering::Acquire) {
            break;
        }
        let minutes = database(&app, trading_repository::get_settings).map(|s| s.cycle_minutes).unwrap_or(15);
        let delay = delay_to_next_cycle(chrono::Utc::now().timestamp_millis(), minutes);
        let next = chrono::Local::now() + chrono::Duration::milliseconds(delay as i64);
        state.update(|info| info.next_cycle_at = Some(next.to_rfc3339()));
        let _ = app.emit(UPDATE_EVENT, ());
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(delay)) => {}
            _ = state.wake.notified() => {}
        }
    }
}

/// Próximo múltiplo do ciclo no relógio (ex.: :00, :15, :30, :45) mais 15 s para o candle fechar.
pub fn delay_to_next_cycle(now_ms: i64, cycle_minutes: i64) -> u64 {
    let period = cycle_minutes.max(1) * 60_000;
    let next = (now_ms / period + 1) * period + 15_000;
    (next - now_ms).max(1_000) as u64
}

/// Executa um ciclo completo. Falhas viram um registro ERROR em vez de derrubar o motor.
pub async fn run_cycle(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<TradingState>();
    let _guard = state.cycle.try_lock().map_err(|_| "Já existe um ciclo em andamento".to_string())?;
    state.update(|info| info.cycle_in_progress = true);
    let _ = app.emit(UPDATE_EVENT, ());
    let result = cycle(app).await;
    let now = chrono::Local::now().to_rfc3339();
    if let Err(error) = &result {
        let symbol = database(app, trading_repository::get_settings).map(|s| s.symbol).unwrap_or_default();
        let _ = database(app, |connection| {
            trading_repository::insert_decision(connection, &NewTradingDecision {
                symbol,
                position_before: "?".into(),
                proposed_action: "ERROR".into(),
                final_action: "ERROR".into(),
                reason: error.clone(),
                order_status: "NONE".into(),
                ..Default::default()
            })
        });
    }
    state.update(|info| {
        info.cycle_in_progress = false;
        info.last_cycle_at = Some(now);
        info.last_error = result.as_ref().err().cloned();
    });
    let _ = app.emit(UPDATE_EVENT, ());
    result
}

async fn cycle(app: &AppHandle) -> Result<(), String> {
    let (settings, (key, secret)) = database(app, |connection| Ok((trading_repository::get_settings(connection)?, trading_repository::credentials(connection)?)))?;
    let mut client = BingxClient::new(&key, &secret)?;
    let _ = client.sync_time().await;
    let symbol = settings.symbol.as_str();
    let rules = client.contract(symbol).await?;
    let candles = client.klines(symbol, &settings.candle_interval, 150).await?;
    let hourly = client.klines(symbol, "1h", 120).await?;
    let balance = client.balance().await?;
    let position = client.position(symbol).await?;
    let hedge = client.hedge_mode().await.unwrap_or(false);
    database(app, |connection| trading_repository::insert_equity(connection, balance.equity, balance.balance, balance.unrealized))?;
    if position.is_none() {
        // Sem posição, stops e alvos antigos ficariam pendurados.
        let _ = client.cancel_open_orders(symbol).await;
    }

    let (day_start, entries_today, recent) = database(app, |connection| {
        let recent: Vec<(String, String, String)> = trading_repository::list_decisions(connection, 8)?
            .into_iter()
            .filter(|decision| decision.final_action != "ERROR")
            .take(3)
            .map(|decision| (decision.created_at, decision.final_action, decision.reason))
            .collect();
        Ok((trading_repository::day_start_equity(connection)?, trading_repository::entries_today(connection)?, recent))
    })?;
    let price = candles.last().map(|candle| candle.close).ok_or("Sem candles para operar")?;
    let context = strategy::build_context(&ContextInput {
        symbol,
        interval: &settings.candle_interval,
        candles: &candles,
        hourly: &hourly,
        position: position.as_ref(),
        equity: balance.equity,
        stop_loss_pct: settings.stop_loss_pct,
        take_profit_pct: settings.take_profit_pct,
        leverage: settings.leverage,
        margin_usdt: settings.margin_usdt,
        recent_decisions: &recent,
    })?;

    let (decision, model) = ask_ai(app, &settings.engine, &context.to_string()).await?;
    let risk = strategy::apply_risk(
        &decision,
        &RiskState { position: position.as_ref().map(|p| p.side), equity: balance.equity, day_start_equity: day_start, trades_today: entries_today },
        &RiskConfig { min_confidence: settings.min_confidence, max_daily_loss_pct: settings.max_daily_loss_pct, max_trades_per_day: MAX_ENTRIES_PER_DAY },
    );

    let (order_status, order_detail) = match risk.action {
        Action::OpenLong | Action::OpenShort => {
            let side = if risk.action == Action::OpenLong { Side::Long } else { Side::Short };
            match open(&client, symbol, side, hedge, price, &settings, &rules).await {
                Ok(detail) => ("FILLED", detail),
                Err(error) => ("FAILED", error),
            }
        }
        Action::Close => match client.close_all(symbol).await {
            Ok(()) => ("FILLED", "Posição fechada a mercado".to_string()),
            Err(error) => ("FAILED", error),
        },
        Action::Hold => ("NONE", String::new()),
    };

    database(app, |connection| {
        trading_repository::insert_decision(connection, &NewTradingDecision {
            symbol: symbol.into(),
            price,
            position_before: match position.as_ref().map(|p| p.side) {
                None => "FLAT",
                Some(Side::Long) => "LONG",
                Some(Side::Short) => "SHORT",
            }
            .into(),
            proposed_action: decision.action.as_str().into(),
            final_action: risk.action.as_str().into(),
            confidence: decision.confidence,
            reason: decision.reason.clone(),
            risk_note: risk.note.clone(),
            engine: settings.engine.clone(),
            model,
            order_status: order_status.into(),
            order_detail,
            equity: Some(balance.equity),
        })
    })?;
    if risk.halt {
        stop(app)?;
    }
    Ok(())
}

async fn open(client: &BingxClient, symbol: &str, side: Side, hedge: bool, price: f64, settings: &TradingSettings, rules: &super::bingx::ContractRules) -> Result<String, String> {
    let quantity = strategy::order_quantity(settings.margin_usdt, settings.leverage, price, rules.quantity_precision, rules.min_quantity, rules.min_notional)?;
    let (stop_loss, take_profit) = strategy::protection_prices(side, price, settings.stop_loss_pct, settings.take_profit_pct, rules.price_precision);
    client.set_leverage(symbol, side, hedge, settings.leverage).await?;
    let order_id = client.open_market(symbol, side, hedge, quantity, rules.quantity_precision, stop_loss, take_profit).await?;
    Ok(format!("Qtd {quantity} · stop {stop_loss} · alvo {take_profit} · ordem {order_id}"))
}

async fn ask_ai(app: &AppHandle, engine: &str, context: &str) -> Result<(Decision, String), String> {
    let user = format!("SNAPSHOT DO CICLO:\n{context}");
    if engine == "ollama" {
        let settings = database(app, ai_repository::get_settings)?;
        let schema = json!({
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["OPEN_LONG", "OPEN_SHORT", "CLOSE", "HOLD"]},
                "confidence": {"type": "number"},
                "reason": {"type": "string"}
            },
            "required": ["action", "confidence", "reason"]
        });
        let messages = vec![
            ollama::OllamaMessage { role: "system".into(), content: SYSTEM_PROMPT.into() },
            ollama::OllamaMessage { role: "user".into(), content: user },
        ];
        let result = ollama::chat_with_format(&settings.endpoint, &settings.model, messages, settings.timeout_seconds.clamp(5, 180) as u64, "agent", Some(schema)).await?;
        return Ok((strategy::parse_decision(&result.content)?, result.model));
    }
    let (api_key, model) = database(app, ai_repository::remote_credentials)?;
    let messages = vec![
        ollama::OllamaAgentMessage { role: "system".into(), content: SYSTEM_PROMPT.into(), tool_calls: vec![], tool_name: None },
        ollama::OllamaAgentMessage { role: "user".into(), content: user, tool_calls: vec![], tool_name: None },
    ];
    let result = gemini::chat_with_tools(&api_key, &model, messages, vec![], 90).await?;
    Ok((strategy::parse_decision(&result.content)?, result.model))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_align_to_the_clock_after_the_candle_closes() {
        let quarter = 15 * 60_000;
        assert_eq!(delay_to_next_cycle(0, 15), (quarter + 15_000) as u64);
        assert_eq!(delay_to_next_cycle(quarter - 1_000, 15), 16_000);
        assert_eq!(delay_to_next_cycle(quarter + 14_500, 15), (quarter - 14_500 + 15_000) as u64);
    }

    #[test]
    fn trading_prompt_demands_a_single_json_decision() {
        assert!(SYSTEM_PROMPT.contains("DEMO"));
        assert!(SYSTEM_PROMPT.contains("\"action\""));
        assert!(SYSTEM_PROMPT.contains("igualmente válidas"), "o prompt precisa tratar compra e venda de forma simétrica");
        assert!(SYSTEM_PROMPT.contains("alignment"));
    }
}
