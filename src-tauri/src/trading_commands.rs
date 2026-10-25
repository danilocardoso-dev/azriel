use crate::{
    database::{trading_models::*, trading_repository, DatabaseState},
    trading::{
        bingx::BingxClient,
        engine::{self, EngineInfo, TradingState},
        strategy::Side,
    },
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

fn lock<'a>(state: &'a State<'_, DatabaseState>) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state.connection.lock().map_err(|_| "O banco de dados está indisponível".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingOverview {
    settings: TradingSettings,
    engine: EngineInfo,
    decisions: Vec<TradingDecisionRecord>,
    equity: Vec<EquityPoint>,
    first_equity: Option<f64>,
    day_start_equity: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePosition {
    side: &'static str,
    quantity: f64,
    entry_price: f64,
    unrealized: f64,
    leverage: i64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartCandle {
    time: i64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSnapshot {
    symbol: String,
    interval: String,
    candles: Vec<ChartCandle>,
    last_price: f64,
    change_pct: f64,
    high_24h: f64,
    low_24h: f64,
}

const CHART_INTERVALS: [&str; 9] = ["1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "1d"];
static SYMBOLS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// Dados públicos para o gráfico; não usa as chaves.
#[tauri::command]
pub async fn get_market_snapshot(symbol: String, interval: String) -> Result<MarketSnapshot, String> {
    trading_repository::validate_symbol(&symbol)?;
    if !CHART_INTERVALS.contains(&interval.as_str()) {
        return Err("Tempo gráfico inválido".into());
    }
    let client = BingxClient::public_only()?;
    let candles = client.live_klines(&symbol, &interval, 200).await?;
    let ticker = client.ticker(&symbol).await?;
    Ok(MarketSnapshot {
        candles: candles.into_iter().map(|c| ChartCandle { time: c.open_time, open: c.open, high: c.high, low: c.low, close: c.close, volume: c.volume }).collect(),
        last_price: ticker.last_price,
        change_pct: ticker.change_pct,
        high_24h: ticker.high,
        low_24h: ticker.low,
        symbol,
        interval,
    })
}

#[tauri::command]
pub async fn list_market_symbols() -> Result<Vec<String>, String> {
    if let Ok(cached) = SYMBOLS.lock() {
        if !cached.is_empty() {
            return Ok(cached.clone());
        }
    }
    let symbols = BingxClient::public_only()?.symbols().await?;
    if let Ok(mut cached) = SYMBOLS.lock() {
        cached.clone_from(&symbols);
    }
    Ok(symbols)
}

/// Troca o par operado pelo agente. Bloqueado com posição aberta para não abandonar um trade.
#[tauri::command]
pub async fn set_trading_symbol(app: AppHandle, state: State<'_, DatabaseState>, symbol: String) -> Result<TradingSettings, String> {
    trading_repository::validate_symbol(&symbol)?;
    let (settings, keys) = {
        let connection = lock(&state)?;
        (trading_repository::get_settings(&connection)?, trading_repository::credentials(&connection).ok())
    };
    if settings.symbol == symbol {
        return Ok(settings);
    }
    if let Some((key, secret)) = keys {
        let mut client = BingxClient::new(&key, &secret)?;
        let _ = client.sync_time().await;
        if client.position(&settings.symbol).await?.is_some() {
            return Err(format!("Feche a posição em {} antes de trocar de par", settings.symbol));
        }
    }
    let updated = {
        let connection = lock(&state)?;
        trading_repository::update_symbol(&connection, &symbol)?
    };
    let _ = app.emit(engine::UPDATE_EVENT, ());
    Ok(updated)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingLive {
    equity: f64,
    balance: f64,
    unrealized: f64,
    available_margin: f64,
    position: Option<LivePosition>,
}

#[tauri::command]
pub fn get_trading_overview(state: State<'_, DatabaseState>, trading: State<'_, TradingState>) -> Result<TradingOverview, String> {
    let connection = lock(&state)?;
    Ok(TradingOverview {
        settings: trading_repository::get_settings(&connection)?,
        engine: trading.info(),
        decisions: trading_repository::list_decisions(&connection, 100)?,
        equity: trading_repository::list_equity(&connection, 1_000)?,
        first_equity: trading_repository::first_equity(&connection)?,
        day_start_equity: trading_repository::day_start_equity(&connection)?,
    })
}

#[tauri::command]
pub fn update_trading_settings(state: State<'_, DatabaseState>, input: TradingSettingsInput) -> Result<TradingSettings, String> {
    let connection = lock(&state)?;
    trading_repository::update_settings(&connection, &input)
}

#[tauri::command]
pub fn update_trading_keys(state: State<'_, DatabaseState>, input: TradingKeysInput) -> Result<TradingSettings, String> {
    let connection = lock(&state)?;
    trading_repository::update_keys(&connection, &input)
}

/// Saldo e posição ao vivo na demo; também serve para testar as chaves.
#[tauri::command]
pub async fn get_trading_live(state: State<'_, DatabaseState>) -> Result<TradingLive, String> {
    let (settings, (key, secret)) = {
        let connection = lock(&state)?;
        (trading_repository::get_settings(&connection)?, trading_repository::credentials(&connection)?)
    };
    let mut client = BingxClient::new(&key, &secret)?;
    let _ = client.sync_time().await;
    let balance = client.balance().await?;
    let position = match client.position(&settings.symbol).await? {
        None => None,
        Some(position) => {
            let (stop_loss, take_profit) = client.protection_orders(&settings.symbol).await.unwrap_or((None, None));
            Some(LivePosition {
                side: if position.side == Side::Long { "LONG" } else { "SHORT" },
                quantity: position.quantity,
                entry_price: position.entry_price,
                unrealized: position.unrealized,
                leverage: position.leverage,
                stop_loss,
                take_profit,
            })
        }
    };
    Ok(TradingLive { equity: balance.equity, balance: balance.balance, unrealized: balance.unrealized, available_margin: balance.available_margin, position })
}

#[tauri::command]
pub fn start_trading(app: AppHandle) -> Result<(), String> {
    engine::start(app)
}

#[tauri::command]
pub fn stop_trading(app: AppHandle) -> Result<(), String> {
    engine::stop(&app)
}

#[tauri::command]
pub async fn run_trading_cycle_now(app: AppHandle) -> Result<(), String> {
    engine::run_cycle(&app).await
}

/// Botão de pânico: fecha tudo na demo e registra a ação manual.
#[tauri::command]
pub async fn close_trading_position(app: AppHandle, state: State<'_, DatabaseState>) -> Result<(), String> {
    let (settings, (key, secret)) = {
        let connection = lock(&state)?;
        (trading_repository::get_settings(&connection)?, trading_repository::credentials(&connection)?)
    };
    let mut client = BingxClient::new(&key, &secret)?;
    let _ = client.sync_time().await;
    let result = client.close_all(&settings.symbol).await;
    {
        let connection = lock(&state)?;
        trading_repository::insert_decision(&connection, &NewTradingDecision {
            symbol: settings.symbol.clone(),
            position_before: "?".into(),
            proposed_action: "MANUAL_CLOSE".into(),
            final_action: "CLOSE".into(),
            reason: "Fechamento manual pelo operador.".into(),
            order_status: if result.is_ok() { "FILLED" } else { "FAILED" }.into(),
            order_detail: result.as_ref().err().cloned().unwrap_or_default(),
            engine: "manual".into(),
            ..Default::default()
        })?;
    }
    let _ = app.emit(engine::UPDATE_EVENT, ());
    result
}
