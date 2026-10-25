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
    let position = client.position(&settings.symbol).await?.map(|position| LivePosition {
        side: if position.side == Side::Long { "LONG" } else { "SHORT" },
        quantity: position.quantity,
        entry_price: position.entry_price,
        unrealized: position.unrealized,
        leverage: position.leverage,
    });
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
