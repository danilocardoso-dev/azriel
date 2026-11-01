//! Motor autônomo: a cada ciclo lê mercado e conta, pede a decisão à IA, aplica os
//! limites de risco, executa na demo da BingX e registra tudo no SQLite.

use super::bingx::BingxClient;
use super::review::{self, ReviewInput};
use super::strategy::{self, Action, ContextInput, Decision, ExitRules, RiskConfig, RiskState, Side};
use crate::database::{ai_repository, trading_models::*, trading_repository, DatabaseState};
use crate::{gemini, ollama};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

const SYSTEM_PROMPT: &str = include_str!("../../prompts/trading-agent-v5.txt");
const REVIEW_PROMPT: &str = include_str!("../../prompts/trading-review-v1.txt");
const MAX_ENTRIES_PER_DAY: i64 = 40;
/// Entre as análises da IA, o vigia confere a posição neste intervalo.
const WATCH_SECONDS: u64 = 15;
pub const UPDATE_EVENT: &str = "trading-update";

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub running: bool,
    pub cycle_in_progress: bool,
    pub last_cycle_at: Option<String>,
    pub next_cycle_at: Option<String>,
    pub last_error: Option<String>,
    pub review_in_progress: bool,
    pub last_review_error: Option<String>,
}

#[derive(Default)]
pub struct TradingState {
    running: AtomicBool,
    wake: Notify,
    cycle: tokio::sync::Mutex<()>,
    review: tokio::sync::Mutex<()>,
    info: Mutex<EngineInfo>,
    /// Melhor movimento a favor já visto em cada trade aberto (id do diário → %).
    peaks: Mutex<HashMap<i64, f64>>,
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
        // Até a próxima análise, o vigia confere a posição a cada WATCH_SECONDS.
        let deadline = Instant::now() + Duration::from_millis(delay);
        loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            tokio::select! {
                _ = tokio::time::sleep(left.min(Duration::from_secs(WATCH_SECONDS))) => {}
                _ = state.wake.notified() => break,
            }
            if !state.running.load(Ordering::Acquire) {
                break;
            }
            if let Err(error) = watch_position(&app).await {
                eprintln!("[TRADING] vigia: {error}");
            }
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
    let price = candles.last().map(|candle| candle.close).ok_or("Sem candles para operar")?;
    let mut trade_closed = reconcile_journal(app, symbol, position.as_ref(), balance.balance, price, &settings)?;
    let minutes_open = match position.as_ref() {
        Some(_) => database(app, |connection| trading_repository::open_trade_for(connection, symbol))?.map(|trade| minutes_since(&trade.opened_at)),
        None => None,
    };

    let (day_start, entries_today, recent, lessons) = database(app, |connection| {
        let recent: Vec<(String, String, String)> = trading_repository::list_decisions(connection, 8)?
            .into_iter()
            .filter(|decision| decision.final_action != "ERROR")
            .take(3)
            .map(|decision| (decision.created_at, decision.final_action, decision.reason))
            .collect();
        Ok((trading_repository::day_start_equity(connection)?, trading_repository::entries_today(connection)?, recent, trading_repository::active_lessons(connection)?))
    })?;
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
        lessons: &lessons,
        exit_rules: exit_rules(&settings),
        minutes_open,
    })?;

    let (decision, model) = ask_ai(app, &settings.engine, &context.to_string()).await?;
    let risk = strategy::apply_risk(
        &decision,
        &RiskState {
            position: position.as_ref().map(|p| p.side),
            equity: balance.equity,
            day_start_equity: day_start,
            trades_today: entries_today,
        },
        &RiskConfig { min_confidence: settings.min_confidence, max_daily_loss_pct: settings.max_daily_loss_pct, max_trades_per_day: MAX_ENTRIES_PER_DAY },
    );

    let (order_status, order_detail) = match risk.action {
        Action::OpenLong | Action::OpenShort => {
            let side = if risk.action == Action::OpenLong { Side::Long } else { Side::Short };
            match open(&client, symbol, side, hedge, price, &settings, &rules).await {
                Ok((detail, quantity)) => {
                    database(app, |connection| {
                        trading_repository::open_trade(connection, &NewTradingTrade {
                            symbol: symbol.into(),
                            side: if side == Side::Long { "LONG" } else { "SHORT" }.into(),
                            entry_price: price,
                            quantity,
                            stop_loss_pct: settings.stop_loss_pct,
                            take_profit_pct: settings.take_profit_pct,
                            candle_interval: settings.candle_interval.clone(),
                            balance_before: balance.balance,
                            confidence: decision.confidence,
                            entry_reason: decision.reason.clone(),
                            entry_context: entry_context(&context),
                        })
                    })?;
                    ("FILLED", detail)
                }
                Err(error) => ("FAILED", error),
            }
        }
        Action::Close => match client.close_all(symbol).await {
            Ok(()) => {
                // Se o saldo não vier agora, o próximo ciclo fecha o trade pelo resultado.
                if let Ok(after) = client.balance().await {
                    trade_closed |= close_journal(app, symbol, after.balance, price, if risk.halt { "DAILY_LIMIT" } else { "AGENT" })?;
                }
                ("FILLED", "Posição fechada a mercado".to_string())
            }
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
    if trade_closed {
        review_if_due(app);
    }
    Ok(())
}

fn exit_rules(settings: &TradingSettings) -> ExitRules {
    ExitRules {
        stop_loss_pct: settings.stop_loss_pct,
        take_profit_pct: settings.take_profit_pct,
        profit_lock_pct: settings.profit_lock_pct,
        max_hold_minutes: settings.max_hold_minutes,
    }
}

/// Minutos desde um horário do SQLite (UTC, "AAAA-MM-DD HH:MM:SS").
fn minutes_since(sqlite_utc: &str) -> i64 {
    chrono::NaiveDateTime::parse_from_str(sqlite_utc, "%Y-%m-%d %H:%M:%S")
        .map(|opened| (chrono::Utc::now().naive_utc() - opened).num_minutes().max(0))
        .unwrap_or(0)
}

/// Vigia entre as análises: com um trade aberto, lê o preço e aplica alvo, stop, trava de
/// lucro e tempo máximo. Não chama a IA. Se um ciclo estiver rodando, espera o próximo toque.
async fn watch_position(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<TradingState>();
    let Ok(_guard) = state.cycle.try_lock() else { return Ok(()) };
    let (settings, (key, secret)) = database(app, |connection| Ok((trading_repository::get_settings(connection)?, trading_repository::credentials(connection)?)))?;
    let symbol = settings.symbol.as_str();
    let Some(trade) = database(app, |connection| trading_repository::open_trade_for(connection, symbol))? else {
        if let Ok(mut peaks) = state.peaks.lock() {
            peaks.clear();
        }
        return Ok(());
    };
    let mut client = BingxClient::new(&key, &secret)?;
    let _ = client.sync_time().await;
    let price = client.klines(symbol, "1m", 1).await?.last().map(|candle| candle.close).ok_or("Sem preço para o vigia")?;
    let side = if trade.side == "LONG" { Side::Long } else { Side::Short };
    let moved = strategy::move_pct(side, trade.entry_price, price);
    let peak = state
        .peaks
        .lock()
        .map(|mut peaks| {
            let best = peaks.entry(trade.id).or_insert(moved);
            *best = best.max(moved);
            *best
        })
        .unwrap_or(moved);
    let Some((reason, note)) = strategy::quick_exit(moved, peak, minutes_since(&trade.opened_at), &exit_rules(&settings)) else { return Ok(()) };
    // A corretora pode já ter fechado pelo stop/alvo; o próximo ciclo registra pelo saldo.
    let Some(position) = client.position(symbol).await? else { return Ok(()) };
    let result = client.close_all(symbol).await;
    let mut closed = false;
    if result.is_ok() {
        if let Ok(after) = client.balance().await {
            closed = close_journal(app, symbol, after.balance, price, reason)?;
        }
        if let Ok(mut peaks) = state.peaks.lock() {
            peaks.remove(&trade.id);
        }
    }
    database(app, |connection| {
        trading_repository::insert_decision(connection, &NewTradingDecision {
            symbol: symbol.into(),
            price,
            position_before: if position.side == Side::Long { "LONG" } else { "SHORT" }.into(),
            proposed_action: "CLOSE".into(),
            final_action: "CLOSE".into(),
            confidence: 100,
            reason: note.clone(),
            engine: "vigia".into(),
            order_status: if result.is_ok() { "FILLED" } else { "FAILED" }.into(),
            order_detail: result.as_ref().err().cloned().unwrap_or_else(|| "Posição fechada a mercado".into()),
            ..Default::default()
        })
    })?;
    let _ = app.emit(UPDATE_EVENT, ());
    if closed {
        review_if_due(app);
    }
    result
}

/// Só o que explica a entrada: direção e indicadores, sem velas nem conta.
fn entry_context(context: &serde_json::Value) -> String {
    json!({"alignment": context["alignment"], "indicators": context["indicators"], "trend1h": context["trend1h"]}).to_string()
}

/// Mantém o diário igual à corretora: fecha o trade cuja posição sumiu (stop/alvo) e adota
/// uma posição aberta que o diário não conhece. Devolve true quando fechou algum trade.
fn reconcile_journal(app: &AppHandle, symbol: &str, position: Option<&strategy::OpenPosition>, balance: f64, price: f64, settings: &TradingSettings) -> Result<bool, String> {
    database(app, |connection| match (trading_repository::open_trade_for(connection, symbol)?, position) {
        (Some(trade), None) => {
            let pnl = balance - trade.balance_before;
            let exit = review::implied_exit_price(&trade, pnl, price);
            trading_repository::close_trade(connection, trade.id, exit, pnl, review::exchange_exit_reason(&trade, pnl))?;
            Ok(true)
        }
        (None, Some(position)) => {
            trading_repository::open_trade(connection, &NewTradingTrade {
                symbol: symbol.into(),
                side: if position.side == Side::Long { "LONG" } else { "SHORT" }.into(),
                entry_price: position.entry_price,
                quantity: position.quantity,
                stop_loss_pct: settings.stop_loss_pct,
                take_profit_pct: settings.take_profit_pct,
                candle_interval: settings.candle_interval.clone(),
                balance_before: balance,
                entry_reason: "Posição já aberta quando o diário começou.".into(),
                entry_context: "{}".into(),
                ..Default::default()
            })?;
            Ok(false)
        }
        _ => Ok(false),
    })
}

/// Fecha o trade aberto do diário com o saldo já realizado depois do fechamento.
pub fn close_journal(app: &AppHandle, symbol: &str, balance_after: f64, price: f64, reason: &str) -> Result<bool, String> {
    database(app, |connection| match trading_repository::open_trade_for(connection, symbol)? {
        Some(trade) => {
            let pnl = balance_after - trade.balance_before;
            trading_repository::close_trade(connection, trade.id, review::implied_exit_price(&trade, pnl, price), pnl, reason)?;
            Ok(true)
        }
        None => Ok(false),
    })
}

/// Dispara a revisão em segundo plano a cada REVIEW_EVERY trades fechados, se o autoajuste estiver ligado.
pub fn review_if_due(app: &AppHandle) {
    let due = database(app, |connection| Ok(trading_repository::auto_tune(connection)? && trading_repository::closed_since_last_review(connection)? >= review::REVIEW_EVERY)).unwrap_or(false);
    if due {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = run_review(&app).await;
        });
    }
}

/// Uma revisão completa: o Gemini estuda os trades fechados, o código aplica as travas e grava.
pub async fn run_review(app: &AppHandle) -> Result<TradingReview, String> {
    let state = app.state::<TradingState>();
    let _guard = state.review.try_lock().map_err(|_| "Já existe uma revisão em andamento".to_string())?;
    state.update(|info| info.review_in_progress = true);
    let _ = app.emit(UPDATE_EVENT, ());
    let result = review_once(app).await;
    state.update(|info| {
        info.review_in_progress = false;
        info.last_review_error = result.as_ref().err().cloned();
    });
    let _ = app.emit(UPDATE_EVENT, ());
    result
}

async fn review_once(app: &AppHandle) -> Result<TradingReview, String> {
    let (settings, trades, new_trades, lessons, previous, (api_key, model)) = database(app, |connection| {
        let trades = trading_repository::closed_trades(connection, 30)?;
        if trades.is_empty() {
            return Err("Ainda não há trades fechados para revisar".into());
        }
        let previous: Vec<(String, String, Vec<TuningChange>)> = trading_repository::list_reviews(connection, 3)?
            .into_iter()
            .filter(|review| !review.reverted)
            .map(|review| (review.created_at, review.diagnosis, review.changes))
            .collect();
        Ok((
            trading_repository::get_settings(connection)?,
            trades,
            trading_repository::closed_since_last_review(connection)?,
            trading_repository::active_lessons(connection)?,
            previous,
            ai_repository::remote_credentials(connection).map_err(|_| "O autoajuste usa o Gemini: configure a chave no AI Core".to_string())?,
        ))
    })?;
    let current = Tunable::of(&settings);
    let input = review::build_review_input(&ReviewInput {
        symbol: &settings.symbol,
        current: &current,
        margin_usdt: settings.margin_usdt,
        leverage: settings.leverage,
        trades: &trades,
        new_trades: new_trades.max(0) as usize,
        lessons: &lessons,
        previous: &previous,
    });
    let messages = vec![
        ollama::OllamaAgentMessage { role: "system".into(), content: REVIEW_PROMPT.into(), tool_calls: vec![], tool_name: None },
        ollama::OllamaAgentMessage { role: "user".into(), content: format!("DADOS DA REVISÃO:\n{input}"), tool_calls: vec![], tool_name: None },
    ];
    let result = gemini::chat_with_tools(&api_key, &model, messages, vec![], 120).await?;
    let proposal = review::parse_proposal(&result.content)?;
    let (tuned, changes) = review::apply_guardrails(&current, &proposal.adjustments, trades.len());
    let summary = review::stats(&trades);
    database(app, |connection| {
        trading_repository::save_review(
            connection,
            &NewTradingReview {
                last_trade_id: trades.iter().map(|trade| trade.id).max().unwrap_or(0),
                trades_count: summary.count as i64,
                pnl_usdt: summary.pnl,
                win_rate: summary.win_rate(),
                diagnosis: proposal.diagnosis.clone(),
                lessons: proposal.lessons.clone(),
                changes,
                settings_before: current.clone(),
                model: result.model.clone(),
            },
            &tuned,
        )
    })
}


async fn open(client: &BingxClient, symbol: &str, side: Side, hedge: bool, price: f64, settings: &TradingSettings, rules: &super::bingx::ContractRules) -> Result<(String, f64), String> {
    let quantity = strategy::order_quantity(settings.margin_usdt, settings.leverage, price, rules.quantity_precision, rules.min_quantity, rules.min_notional)?;
    let (stop_loss, take_profit) = strategy::protection_prices(side, price, settings.stop_loss_pct, settings.take_profit_pct, rules.price_precision);
    client.set_leverage(symbol, side, hedge, settings.leverage).await?;
    let order_id = client.open_market(symbol, side, hedge, quantity, rules.quantity_precision, stop_loss, take_profit).await?;
    Ok((format!("Qtd {quantity} · stop {stop_loss} · alvo {take_profit} · ordem {order_id}"), quantity))
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
    fn minutes_are_counted_from_sqlite_utc_timestamps() {
        let ten_ago = (chrono::Utc::now() - chrono::Duration::minutes(10)).format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(minutes_since(&ten_ago), 10);
        assert_eq!(minutes_since("lixo"), 0);
    }

    #[test]
    fn trading_prompt_demands_a_single_json_decision() {
        assert!(SYSTEM_PROMPT.contains("DEMO"));
        assert!(SYSTEM_PROMPT.contains("\"action\""));
        assert!(SYSTEM_PROMPT.contains("igualmente válidas"), "o prompt precisa tratar compra e venda de forma simétrica");
        assert!(SYSTEM_PROMPT.contains("taxa"), "operação rápida precisa considerar a taxa");
        assert!(SYSTEM_PROMPT.contains("alignment"));
        assert!(SYSTEM_PROMPT.contains("lessons"), "o agente precisa seguir as lições do autoajuste");
        assert!(REVIEW_PROMPT.contains("parametersLocked") && REVIEW_PROMPT.contains("\"adjustments\""));
        assert!(SYSTEM_PROMPT.contains("minutesOpen") && !SYSTEM_PROMPT.contains("só quando o trend1h deixar de favorecer"), "saída rápida, não pela tendência de 1h");
    }
}
