use super::trading_models::*;
use rusqlite::{params, Connection, OptionalExtension};

const INTERVALS: [&str; 6] = ["5m", "15m", "30m", "1h", "2h", "4h"];
const ENGINES: [&str; 2] = ["gemini", "ollama"];

pub fn get_settings(connection: &Connection) -> Result<TradingSettings, String> {
    connection
        .query_row(
            "SELECT api_key<>'' AND secret_key<>'',symbol,candle_interval,cycle_minutes,margin_usdt,leverage,stop_loss_pct,take_profit_pct,max_daily_loss_pct,min_confidence,engine,running,updated_at,profit_lock_pct,max_hold_minutes FROM trading_settings WHERE id=1",
            [],
            |row| {
                Ok(TradingSettings {
                    api_key_configured: row.get(0)?,
                    symbol: row.get(1)?,
                    candle_interval: row.get(2)?,
                    cycle_minutes: row.get(3)?,
                    margin_usdt: row.get(4)?,
                    leverage: row.get(5)?,
                    stop_loss_pct: row.get(6)?,
                    take_profit_pct: row.get(7)?,
                    max_daily_loss_pct: row.get(8)?,
                    min_confidence: row.get(9)?,
                    engine: row.get(10)?,
                    running: row.get(11)?,
                    updated_at: row.get(12)?,
                    profit_lock_pct: row.get(13)?,
                    max_hold_minutes: row.get(14)?,
                })
            },
        )
        .map_err(err)
}

pub fn validate_settings(input: &TradingSettingsInput) -> Result<(), String> {
    validate_symbol(input.symbol.trim())?;
    if !INTERVALS.contains(&input.candle_interval.as_str()) {
        return Err("Intervalo de candle inválido".into());
    }
    if !(5..=240).contains(&input.cycle_minutes) {
        return Err("O ciclo deve ficar entre 5 e 240 minutos".into());
    }
    if !(1.0..=10_000.0).contains(&input.margin_usdt) {
        return Err("Margem por operação deve ficar entre 1 e 10.000 USDT".into());
    }
    if !(1..=20).contains(&input.leverage) {
        return Err("Alavancagem deve ficar entre 1x e 20x".into());
    }
    if !(0.1..=10.0).contains(&input.stop_loss_pct) || !(0.1..=30.0).contains(&input.take_profit_pct) {
        return Err("Stop loss (0,1–10%) ou take profit (0,1–30%) fora do limite".into());
    }
    if !(0.5..=50.0).contains(&input.max_daily_loss_pct) {
        return Err("Perda diária máxima deve ficar entre 0,5% e 50%".into());
    }
    if !(0..=100).contains(&input.min_confidence) {
        return Err("Confiança mínima deve ficar entre 0 e 100".into());
    }
    if !ENGINES.contains(&input.engine.as_str()) {
        return Err("Motor de IA inválido".into());
    }
    if input.profit_lock_pct != 0.0 && !(0.15..=5.0).contains(&input.profit_lock_pct) {
        return Err("Trava de lucro: 0 (desligada) ou entre 0,15% e 5%".into());
    }
    if !(5..=1_440).contains(&input.max_hold_minutes) {
        return Err("Tempo máximo do trade deve ficar entre 5 min e 24 h".into());
    }
    Ok(())
}

pub fn update_settings(connection: &Connection, input: &TradingSettingsInput) -> Result<TradingSettings, String> {
    validate_settings(input)?;
    connection
        .execute(
            "UPDATE trading_settings SET symbol=?1,candle_interval=?2,cycle_minutes=?3,margin_usdt=?4,leverage=?5,stop_loss_pct=?6,take_profit_pct=?7,max_daily_loss_pct=?8,min_confidence=?9,engine=?10,profit_lock_pct=?11,max_hold_minutes=?12,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![input.symbol.trim(), input.candle_interval, input.cycle_minutes, input.margin_usdt, input.leverage, input.stop_loss_pct, input.take_profit_pct, input.max_daily_loss_pct, input.min_confidence, input.engine, input.profit_lock_pct, input.max_hold_minutes],
        )
        .map_err(err)?;
    get_settings(connection)
}

pub fn validate_symbol(symbol: &str) -> Result<(), String> {
    if !symbol.ends_with("-USDT") || symbol.len() > 24 || !symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-') {
        return Err("Par inválido. Use o formato BTC-USDT.".into());
    }
    Ok(())
}

pub fn update_symbol(connection: &Connection, symbol: &str) -> Result<TradingSettings, String> {
    validate_symbol(symbol)?;
    connection
        .execute("UPDATE trading_settings SET symbol=?1,updated_at=CURRENT_TIMESTAMP WHERE id=1", [symbol])
        .map_err(err)?;
    get_settings(connection)
}

pub fn update_keys(connection: &Connection, input: &TradingKeysInput) -> Result<TradingSettings, String> {
    let (key, secret) = (input.api_key.trim(), input.secret_key.trim());
    let invalid = |value: &str| value.chars().count() > 200 || value.chars().any(|c| c.is_whitespace() || c.is_control());
    if invalid(key) || invalid(secret) || key.is_empty() != secret.is_empty() {
        return Err("Informe a API Key e a Secret Key juntas, sem espaços".into());
    }
    connection
        .execute("UPDATE trading_settings SET api_key=?1,secret_key=?2,updated_at=CURRENT_TIMESTAMP WHERE id=1", [key, secret])
        .map_err(err)?;
    get_settings(connection)
}

/// Uso exclusivo do motor de trading.
pub fn credentials(connection: &Connection) -> Result<(String, String), String> {
    let pair: (String, String) = connection
        .query_row("SELECT api_key,secret_key FROM trading_settings WHERE id=1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(err)?;
    if pair.0.is_empty() || pair.1.is_empty() {
        return Err("Cadastre as chaves da BingX (conta demo) antes de operar".into());
    }
    Ok(pair)
}

pub fn set_running(connection: &Connection, running: bool) -> Result<(), String> {
    connection.execute("UPDATE trading_settings SET running=?1 WHERE id=1", [running]).map_err(err).map(|_| ())
}

pub fn insert_decision(connection: &Connection, decision: &NewTradingDecision) -> Result<i64, String> {
    connection
        .execute(
            "INSERT INTO trading_decisions(symbol,price,position_before,proposed_action,final_action,confidence,reason,risk_note,engine,model,order_status,order_detail,equity) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",
            params![decision.symbol, decision.price, decision.position_before, decision.proposed_action, decision.final_action, decision.confidence, decision.reason, decision.risk_note, decision.engine, decision.model, decision.order_status, decision.order_detail, decision.equity],
        )
        .map_err(err)?;
    Ok(connection.last_insert_rowid())
}

pub fn list_decisions(connection: &Connection, limit: i64) -> Result<Vec<TradingDecisionRecord>, String> {
    let mut statement = connection
        .prepare("SELECT id,created_at,symbol,price,position_before,proposed_action,final_action,confidence,reason,risk_note,engine,model,order_status,order_detail,equity FROM trading_decisions ORDER BY id DESC LIMIT ?1")
        .map_err(err)?;
    let rows = statement
        .query_map([limit.clamp(1, 500)], |row| {
            Ok(TradingDecisionRecord {
                id: row.get(0)?,
                created_at: row.get(1)?,
                symbol: row.get(2)?,
                price: row.get(3)?,
                position_before: row.get(4)?,
                proposed_action: row.get(5)?,
                final_action: row.get(6)?,
                confidence: row.get(7)?,
                reason: row.get(8)?,
                risk_note: row.get(9)?,
                engine: row.get(10)?,
                model: row.get(11)?,
                order_status: row.get(12)?,
                order_detail: row.get(13)?,
                equity: row.get(14)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn insert_equity(connection: &Connection, equity: f64, balance: f64, unrealized: f64) -> Result<(), String> {
    connection
        .execute("INSERT INTO trading_equity(equity,balance,unrealized) VALUES(?1,?2,?3)", params![equity, balance, unrealized])
        .map_err(err)
        .map(|_| ())
}

pub fn list_equity(connection: &Connection, limit: i64) -> Result<Vec<EquityPoint>, String> {
    let mut statement = connection
        .prepare("SELECT created_at,equity,balance,unrealized FROM (SELECT * FROM trading_equity ORDER BY id DESC LIMIT ?1) ORDER BY id")
        .map_err(err)?;
    let rows = statement
        .query_map([limit.clamp(1, 2_000)], |row| Ok(EquityPoint { created_at: row.get(0)?, equity: row.get(1)?, balance: row.get(2)?, unrealized: row.get(3)? }))
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

/// Patrimônio do primeiro registro do dia local; base do limite de perda diária.
pub fn day_start_equity(connection: &Connection) -> Result<Option<f64>, String> {
    connection
        .query_row(
            "SELECT equity FROM trading_equity WHERE date(created_at,'localtime')=date('now','localtime') ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(err)
}

pub fn first_equity(connection: &Connection) -> Result<Option<f64>, String> {
    connection.query_row("SELECT equity FROM trading_equity ORDER BY id LIMIT 1", [], |row| row.get(0)).optional().map_err(err)
}

pub fn entries_today(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM trading_decisions WHERE final_action IN ('OPEN_LONG','OPEN_SHORT') AND order_status='FILLED' AND date(created_at,'localtime')=date('now','localtime')",
            [],
            |row| row.get(0),
        )
        .map_err(err)
}

const TRADE_COLUMNS: &str = "id,symbol,side,opened_at,closed_at,entry_price,exit_price,quantity,stop_loss_pct,take_profit_pct,candle_interval,balance_before,pnl_usdt,exit_reason,confidence,entry_reason,entry_context";

fn trade_row(row: &rusqlite::Row) -> rusqlite::Result<TradingTrade> {
    Ok(TradingTrade {
        id: row.get(0)?,
        symbol: row.get(1)?,
        side: row.get(2)?,
        opened_at: row.get(3)?,
        closed_at: row.get(4)?,
        entry_price: row.get(5)?,
        exit_price: row.get(6)?,
        quantity: row.get(7)?,
        stop_loss_pct: row.get(8)?,
        take_profit_pct: row.get(9)?,
        candle_interval: row.get(10)?,
        balance_before: row.get(11)?,
        pnl_usdt: row.get(12)?,
        exit_reason: row.get(13)?,
        confidence: row.get(14)?,
        entry_reason: row.get(15)?,
        entry_context: row.get(16)?,
    })
}

pub fn open_trade(connection: &Connection, trade: &NewTradingTrade) -> Result<i64, String> {
    connection
        .execute(
            "INSERT INTO trading_trades(symbol,side,entry_price,quantity,stop_loss_pct,take_profit_pct,candle_interval,balance_before,confidence,entry_reason,entry_context) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            params![trade.symbol, trade.side, trade.entry_price, trade.quantity, trade.stop_loss_pct, trade.take_profit_pct, trade.candle_interval, trade.balance_before, trade.confidence, trade.entry_reason, trade.entry_context],
        )
        .map_err(err)?;
    Ok(connection.last_insert_rowid())
}

/// O trade ainda aberto no diário para o par, se houver.
pub fn open_trade_for(connection: &Connection, symbol: &str) -> Result<Option<TradingTrade>, String> {
    connection
        .query_row(&format!("SELECT {TRADE_COLUMNS} FROM trading_trades WHERE symbol=?1 AND closed_at IS NULL ORDER BY id DESC LIMIT 1"), [symbol], trade_row)
        .optional()
        .map_err(err)
}

pub fn close_trade(connection: &Connection, id: i64, exit_price: f64, pnl_usdt: f64, reason: &str) -> Result<(), String> {
    connection
        .execute(
            "UPDATE trading_trades SET closed_at=CURRENT_TIMESTAMP,exit_price=?2,pnl_usdt=?3,exit_reason=?4 WHERE id=?1 AND closed_at IS NULL",
            params![id, exit_price, pnl_usdt, reason],
        )
        .map_err(err)
        .map(|_| ())
}

/// Trades mais recentes primeiro (abertos e fechados).
pub fn list_trades(connection: &Connection, limit: i64) -> Result<Vec<TradingTrade>, String> {
    let mut statement = connection.prepare(&format!("SELECT {TRADE_COLUMNS} FROM trading_trades ORDER BY opened_at DESC,id DESC LIMIT ?1")).map_err(err)?;
    let rows = statement.query_map([limit.clamp(1, 500)], trade_row).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

/// Trades fechados, mais antigos primeiro, limitados aos `limit` mais recentes.
pub fn closed_trades(connection: &Connection, limit: i64) -> Result<Vec<TradingTrade>, String> {
    let mut statement = connection
        .prepare(&format!("SELECT {TRADE_COLUMNS} FROM (SELECT * FROM trading_trades WHERE closed_at IS NOT NULL ORDER BY id DESC LIMIT ?1) ORDER BY id"))
        .map_err(err)?;
    let rows = statement.query_map([limit.clamp(1, 500)], trade_row).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

/// Trades fechados depois da última revisão (desfeita ou não): é o que dispara a próxima.
pub fn closed_since_last_review(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT COUNT(*) FROM trading_trades WHERE closed_at IS NOT NULL AND id > COALESCE((SELECT MAX(last_trade_id) FROM trading_reviews),0)",
            [],
            |row| row.get(0),
        )
        .map_err(err)
}

fn review_row(row: &rusqlite::Row) -> rusqlite::Result<TradingReview> {
    let lessons: String = row.get(7)?;
    let changes: String = row.get(8)?;
    Ok(TradingReview {
        id: row.get(0)?,
        created_at: row.get(1)?,
        last_trade_id: row.get(2)?,
        trades_count: row.get(3)?,
        pnl_usdt: row.get(4)?,
        win_rate: row.get(5)?,
        diagnosis: row.get(6)?,
        lessons: serde_json::from_str(&lessons).unwrap_or_default(),
        changes: serde_json::from_str(&changes).unwrap_or_default(),
        model: row.get(9)?,
        reverted: row.get(10)?,
    })
}

const REVIEW_COLUMNS: &str = "id,created_at,last_trade_id,trades_count,pnl_usdt,win_rate,diagnosis,lessons,changes,model,reverted";

pub fn list_reviews(connection: &Connection, limit: i64) -> Result<Vec<TradingReview>, String> {
    let mut statement = connection.prepare(&format!("SELECT {REVIEW_COLUMNS} FROM trading_reviews ORDER BY id DESC LIMIT ?1")).map_err(err)?;
    let rows = statement.query_map([limit.clamp(1, 100)], review_row).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

/// Lições em vigor: as da revisão mais recente que não foi desfeita.
pub fn active_lessons(connection: &Connection) -> Result<Vec<String>, String> {
    Ok(connection
        .query_row(&format!("SELECT {REVIEW_COLUMNS} FROM trading_reviews WHERE reverted=0 ORDER BY id DESC LIMIT 1"), [], review_row)
        .optional()
        .map_err(err)?
        .map(|review| review.lessons)
        .unwrap_or_default())
}

/// Tira uma lição das que estão em vigor (a da revisão mais recente não desfeita).
pub fn remove_lesson(connection: &Connection, lesson: &str) -> Result<Vec<String>, String> {
    let (id, lessons): (i64, String) = connection
        .query_row("SELECT id,lessons FROM trading_reviews WHERE reverted=0 ORDER BY id DESC LIMIT 1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
        .map_err(err)?
        .ok_or("Não há lições em vigor")?;
    let mut lessons: Vec<String> = serde_json::from_str(&lessons).unwrap_or_default();
    let before = lessons.len();
    lessons.retain(|item| item != lesson);
    if lessons.len() == before {
        return Err("Lição não encontrada".into());
    }
    connection.execute("UPDATE trading_reviews SET lessons=?2 WHERE id=?1", params![id, json(&lessons)]).map_err(err)?;
    Ok(lessons)
}

pub fn apply_tunable(connection: &Connection, tunable: &Tunable) -> Result<TradingSettings, String> {
    let current = get_settings(connection)?;
    update_settings(connection, &TradingSettingsInput {
        symbol: current.symbol,
        candle_interval: tunable.candle_interval.clone(),
        cycle_minutes: tunable.cycle_minutes,
        margin_usdt: current.margin_usdt,
        leverage: current.leverage,
        stop_loss_pct: tunable.stop_loss_pct,
        take_profit_pct: tunable.take_profit_pct,
        max_daily_loss_pct: current.max_daily_loss_pct,
        min_confidence: tunable.min_confidence,
        engine: current.engine,
        profit_lock_pct: current.profit_lock_pct,
        max_hold_minutes: current.max_hold_minutes,
    })
}

/// Grava a revisão e aplica os novos parâmetros juntos.
pub fn save_review(connection: &Connection, review: &NewTradingReview, tuned: &Tunable) -> Result<TradingReview, String> {
    connection.execute_batch("BEGIN").map_err(err)?;
    let result = (|| {
        connection
            .execute(
                "INSERT INTO trading_reviews(last_trade_id,trades_count,pnl_usdt,win_rate,diagnosis,lessons,changes,settings_before,model) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![review.last_trade_id, review.trades_count, review.pnl_usdt, review.win_rate, review.diagnosis, json(&review.lessons), json(&review.changes), json(&review.settings_before), review.model],
            )
            .map_err(err)?;
        let id = connection.last_insert_rowid();
        if !review.changes.is_empty() {
            apply_tunable(connection, tuned)?;
        }
        connection.query_row(&format!("SELECT {REVIEW_COLUMNS} FROM trading_reviews WHERE id=?1"), [id], review_row).map_err(err)
    })();
    connection.execute_batch(if result.is_ok() { "COMMIT" } else { "ROLLBACK" }).map_err(err)?;
    result
}

/// Desfaz a revisão mais recente ainda em vigor: volta os parâmetros de antes e as lições anteriores.
pub fn revert_last_review(connection: &Connection) -> Result<TradingSettings, String> {
    let (id, before): (i64, String) = connection
        .query_row("SELECT id,settings_before FROM trading_reviews WHERE reverted=0 ORDER BY id DESC LIMIT 1", [], |row| Ok((row.get(0)?, row.get(1)?)))
        .optional()
        .map_err(err)?
        .ok_or("Não há revisão para desfazer")?;
    let before: Tunable = serde_json::from_str(&before).map_err(|_| "Revisão sem os parâmetros anteriores".to_string())?;
    let settings = apply_tunable(connection, &before)?;
    connection.execute("UPDATE trading_reviews SET reverted=1 WHERE id=?1", [id]).map_err(err)?;
    Ok(settings)
}

/// Monta o diário uma única vez (enquanto ele não tem trade fechado) a partir das decisões e do
/// patrimônio já gravados, para a primeira revisão já ter os trades anteriores ao autoajuste.
/// O resultado de cada trade é a variação do saldo realizado entre a abertura e o primeiro
/// ciclo seguinte sem posição.
pub fn backfill_trades(connection: &Connection) -> Result<usize, String> {
    let has_closed: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM trading_trades WHERE closed_at IS NOT NULL)", [], |row| row.get(0)).map_err(err)?;
    if has_closed {
        return Ok(0);
    }
    // Uma posição já adotada pelo motor (aberta antes do diário existir) é completada com a
    // decisão que de fato a abriu, em vez de ganhar uma linha duplicada.
    let adopted: Option<(i64, String, String)> = connection
        .query_row("SELECT id,side,opened_at FROM trading_trades ORDER BY id LIMIT 1", [], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .optional()
        .map_err(err)?;
    let settings = get_settings(connection)?;
    let mut statement = connection
        .prepare("SELECT created_at,symbol,price,position_before,final_action,order_status,confidence,reason,order_detail,engine FROM trading_decisions WHERE ?1 IS NULL OR created_at<?1 ORDER BY id")
        .map_err(err)?;
    #[allow(clippy::type_complexity)]
    let decisions: Vec<(String, String, f64, String, String, String, i64, String, String, String)> = statement
        .query_map([adopted.as_ref().map(|(_, _, opened_at)| opened_at.clone())], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?, row.get(8)?, row.get(9)?)))
        .map_err(err)?
        .collect::<Result<_, _>>()
        .map_err(err)?;
    let balance_at = |at: &str| -> Result<Option<f64>, String> {
        connection
            .query_row("SELECT balance FROM trading_equity WHERE created_at<=?1 ORDER BY id DESC LIMIT 1", [at], |row| row.get(0))
            .optional()
            .map_err(err)
    };
    let quantity_of = |detail: &str| detail.strip_prefix("Qtd ").and_then(|rest| rest.split_whitespace().next()).and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
    // "Qtd 0.0012 · stop 81448 · alvo 79028.8": o stop e o alvo reais daquela ordem, em % da entrada.
    let level_of = |detail: &str, label: &str| detail.split(" · ").find_map(|part| part.strip_prefix(label)).and_then(|value| value.trim().parse::<f64>().ok());
    let pct_from = |level: Option<f64>, entry: f64, fallback: f64| match level {
        Some(level) if entry > 0.0 => ((level - entry).abs() / entry * 10_000.0).round() / 100.0,
        _ => fallback,
    };
    struct Pending {
        trade: TradingTrade,
        reason: Option<&'static str>,
    }
    let mut pending: Option<Pending> = None;
    let mut inserted = 0;
    let mut insert = |trade: &TradingTrade| -> Result<(), String> {
        connection
            .execute(
                "INSERT INTO trading_trades(symbol,side,opened_at,closed_at,entry_price,exit_price,quantity,stop_loss_pct,take_profit_pct,candle_interval,balance_before,pnl_usdt,exit_reason,confidence,entry_reason) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                params![trade.symbol, trade.side, trade.opened_at, trade.closed_at, trade.entry_price, trade.exit_price, trade.quantity, trade.stop_loss_pct, trade.take_profit_pct, trade.candle_interval, trade.balance_before, trade.pnl_usdt, trade.exit_reason, trade.confidence, trade.entry_reason],
            )
            .map_err(err)?;
        inserted += 1;
        Ok(())
    };
    for (at, symbol, price, position_before, action, status, confidence, reason, detail, engine) in &decisions {
        if let Some(open) = pending.as_mut() {
            if action == "CLOSE" && status == "FILLED" {
                open.reason = Some(if engine == "manual" { "MANUAL" } else { "AGENT" });
            }
        }
        if position_before == "FLAT" {
            if let Some(open) = pending.take() {
                if let Some(balance) = balance_at(at)? {
                    let mut trade = open.trade;
                    let pnl = balance - trade.balance_before;
                    trade.exit_reason = Some(open.reason.unwrap_or_else(|| crate::trading::review::exchange_exit_reason(&trade, pnl)).into());
                    trade.exit_price = Some(crate::trading::review::implied_exit_price(&trade, pnl, *price));
                    trade.closed_at = Some(at.clone());
                    trade.pnl_usdt = Some(pnl);
                    insert(&trade)?;
                }
            }
        }
        if pending.is_none() && status == "FILLED" && (action == "OPEN_LONG" || action == "OPEN_SHORT") {
            if let Some(balance) = balance_at(at)? {
                pending = Some(Pending {
                    trade: TradingTrade {
                        id: 0,
                        symbol: symbol.clone(),
                        side: if action == "OPEN_LONG" { "LONG" } else { "SHORT" }.into(),
                        opened_at: at.clone(),
                        closed_at: None,
                        entry_price: *price,
                        exit_price: None,
                        quantity: quantity_of(detail),
                        stop_loss_pct: pct_from(level_of(detail, "stop "), *price, settings.stop_loss_pct),
                        take_profit_pct: pct_from(level_of(detail, "alvo "), *price, settings.take_profit_pct),
                        candle_interval: settings.candle_interval.clone(),
                        balance_before: balance,
                        pnl_usdt: None,
                        exit_reason: None,
                        confidence: *confidence,
                        entry_reason: reason.clone(),
                        entry_context: "{}".into(),
                    },
                    reason: None,
                });
            }
        }
    }
    // A posição ainda aberta entra aberta; o próximo ciclo a fecha pelo resultado.
    match (pending, adopted) {
        (Some(open), None) => insert(&open.trade)?,
        (Some(open), Some((id, side, _))) if side == open.trade.side => {
            let trade = open.trade;
            connection
                .execute(
                    "UPDATE trading_trades SET opened_at=?2,balance_before=?3,confidence=?4,entry_reason=?5,quantity=?6 WHERE id=?1",
                    params![id, trade.opened_at, trade.balance_before, trade.confidence, trade.entry_reason, trade.quantity],
                )
                .map_err(err)?;
        }
        _ => {}
    }
    Ok(inserted)
}

pub fn auto_tune(connection: &Connection) -> Result<bool, String> {
    connection.query_row("SELECT auto_tune FROM trading_tuning WHERE id=1", [], |row| row.get(0)).map_err(err)
}

pub fn set_auto_tune(connection: &Connection, enabled: bool) -> Result<(), String> {
    connection.execute("UPDATE trading_tuning SET auto_tune=?1 WHERE id=1", [enabled]).map_err(err).map(|_| ())
}

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

fn err(error: rusqlite::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn input() -> TradingSettingsInput {
        TradingSettingsInput {
            symbol: "BTC-USDT".into(), candle_interval: "15m".into(), cycle_minutes: 15, margin_usdt: 20.0, leverage: 5,
            stop_loss_pct: 1.0, take_profit_pct: 2.0, max_daily_loss_pct: 5.0, min_confidence: 60, engine: "gemini".into(),
            profit_lock_pct: 0.25, max_hold_minutes: 30,
        }
    }

    #[test]
    fn settings_validate_limits_and_hide_keys() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        assert!(!get_settings(&connection).unwrap().api_key_configured);
        assert!(credentials(&connection).is_err());
        assert!(update_settings(&connection, &TradingSettingsInput { leverage: 50, ..input() }).is_err());
        assert!(update_settings(&connection, &TradingSettingsInput { symbol: "btc".into(), ..input() }).is_err());
        assert_eq!(update_settings(&connection, &TradingSettingsInput { symbol: "ETH-USDT".into(), ..input() }).unwrap().symbol, "ETH-USDT");
        assert!(update_keys(&connection, &TradingKeysInput { api_key: "abc".into(), secret_key: String::new() }).is_err());
        let saved = update_keys(&connection, &TradingKeysInput { api_key: "abc".into(), secret_key: "s3cr3t".into() }).unwrap();
        assert!(saved.api_key_configured);
        assert!(!serde_json::to_string(&saved).unwrap().contains("s3cr3t"));
        assert_eq!(credentials(&connection).unwrap(), ("abc".into(), "s3cr3t".into()));
    }

    #[test]
    fn journal_reviews_apply_and_revert_tuning() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        update_settings(&connection, &TradingSettingsInput { candle_interval: "5m".into(), cycle_minutes: 5, ..input() }).unwrap();
        assert!(auto_tune(&connection).unwrap());
        assert_eq!(open_trade_for(&connection, "BTC-USDT").unwrap(), None);
        let trade = NewTradingTrade { symbol: "BTC-USDT".into(), side: "SHORT".into(), entry_price: 80_000.0, quantity: 0.0012, stop_loss_pct: 1.0, take_profit_pct: 2.0, candle_interval: "5m".into(), balance_before: 1000.0, ..Default::default() };
        for pnl in [-0.9, 0.3] {
            let id = open_trade(&connection, &trade).unwrap();
            assert_eq!(open_trade_for(&connection, "BTC-USDT").unwrap().unwrap().id, id);
            close_trade(&connection, id, 80_100.0, pnl, "STOP").unwrap();
        }
        open_trade(&connection, &trade).unwrap();
        assert_eq!(closed_trades(&connection, 10).unwrap().iter().map(|t| t.pnl_usdt).collect::<Vec<_>>(), vec![Some(-0.9), Some(0.3)]);
        assert_eq!(list_trades(&connection, 10).unwrap().len(), 3);
        assert_eq!(closed_since_last_review(&connection).unwrap(), 2);

        let before = Tunable::of(&get_settings(&connection).unwrap());
        let tuned = Tunable { stop_loss_pct: 1.3, candle_interval: "15m".into(), cycle_minutes: 15, ..before.clone() };
        let change = TuningChange { field: "stopLossPct".into(), label: "Stop loss".into(), from: "1,0%".into(), to: "1,3%".into() };
        let review = NewTradingReview { last_trade_id: 2, trades_count: 2, pnl_usdt: -0.6, win_rate: 50.0, diagnosis: "d".into(), lessons: vec!["l1".into()], changes: vec![change], settings_before: before.clone(), model: "m".into() };
        let saved = save_review(&connection, &review, &tuned).unwrap();
        assert_eq!(saved.lessons, vec!["l1".to_string()]);
        assert_eq!(Tunable::of(&get_settings(&connection).unwrap()), tuned);
        assert_eq!(active_lessons(&connection).unwrap(), vec!["l1".to_string()]);
        assert!(remove_lesson(&connection, "outra").is_err());
        assert!(remove_lesson(&connection, "l1").unwrap().is_empty());
        assert!(active_lessons(&connection).unwrap().is_empty());
        assert_eq!(closed_since_last_review(&connection).unwrap(), 0);

        assert_eq!(Tunable::of(&revert_last_review(&connection).unwrap()), before);
        assert!(active_lessons(&connection).unwrap().is_empty());
        assert!(list_reviews(&connection, 5).unwrap()[0].reverted);
        assert!(revert_last_review(&connection).is_err());
        set_auto_tune(&connection, false).unwrap();
        assert!(!auto_tune(&connection).unwrap());
    }

    #[test]
    fn old_databases_move_to_quick_trades_once() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        assert_eq!((get_settings(&connection).unwrap().stop_loss_pct, get_settings(&connection).unwrap().max_hold_minutes), (0.3, 30));
        // Banco de antes do modo rápido: sem as colunas novas e com stop/alvo de swing.
        connection.execute_batch("ALTER TABLE trading_settings DROP COLUMN max_hold_minutes; ALTER TABLE trading_settings DROP COLUMN profit_lock_pct; UPDATE trading_settings SET stop_loss_pct=1.0,take_profit_pct=2.0,candle_interval='15m',cycle_minutes=15;").unwrap();
        database::initialize(&mut connection).unwrap();
        let settings = get_settings(&connection).unwrap();
        assert_eq!((settings.stop_loss_pct, settings.take_profit_pct, settings.candle_interval.as_str(), settings.cycle_minutes), (0.3, 0.5, "5m", 5));
        assert_eq!((settings.profit_lock_pct, settings.max_hold_minutes), (0.25, 30));
        // Depois disso, o que você configurar fica.
        update_settings(&connection, &TradingSettingsInput { stop_loss_pct: 0.4, ..input() }).unwrap();
        database::initialize(&mut connection).unwrap();
        assert_eq!(get_settings(&connection).unwrap().stop_loss_pct, 0.4);
        assert!(update_settings(&connection, &TradingSettingsInput { profit_lock_pct: 0.05, ..input() }).is_err());
        assert!(update_settings(&connection, &TradingSettingsInput { max_hold_minutes: 2, ..input() }).is_err());
    }

    #[test]
    fn journal_is_rebuilt_from_the_old_history_once() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        let equity = |at: &str, balance: f64| connection.execute("INSERT INTO trading_equity(created_at,equity,balance,unrealized) VALUES(?1,?2,?2,0)", params![at, balance]).unwrap();
        let decision = |at: &str, before: &str, action: &str, status: &str, detail: &str, engine: &str| {
            connection.execute(
                "INSERT INTO trading_decisions(created_at,symbol,price,position_before,proposed_action,final_action,order_status,order_detail,engine,confidence,reason) VALUES(?1,'BTC-USDT',80000,?2,?3,?3,?4,?5,?6,75,'r')",
                params![at, before, action, status, detail, engine],
            ).unwrap();
        };
        // Dados reais de 25–29/10: manual com lucro, stop num LONG, fechamento do agente e um SHORT ainda aberto.
        equity("2026-10-25 18:40:00", 100_000.0);
        decision("2026-10-25 18:40:20", "FLAT", "OPEN_SHORT", "FILLED", "Qtd 0.0012 · stop 80800 · alvo 78400 · ordem 1", "gemini");
        decision("2026-10-25 21:26:33", "?", "CLOSE", "FILLED", "", "manual");
        equity("2026-10-26 12:53:20", 100_000.24);
        decision("2026-10-26 12:53:40", "FLAT", "HOLD", "NONE", "", "gemini");
        decision("2026-10-28 19:42:04", "FLAT", "OPEN_LONG", "FILLED", "Qtd 0.0012 · stop 80426", "gemini");
        equity("2026-10-29 01:50:18", 99_999.15);
        decision("2026-10-29 01:50:19", "FLAT", "HOLD", "NONE", "", "gemini");
        decision("2026-10-29 04:45:20", "FLAT", "OPEN_LONG", "FILLED", "Qtd 0.0012", "gemini");
        decision("2026-10-29 07:00:21", "LONG", "CLOSE", "FILLED", "", "gemini");
        equity("2026-10-29 07:05:00", 99_998.9);
        decision("2026-10-29 07:05:22", "FLAT", "OPEN_SHORT", "FILLED", "Qtd 0.0012", "gemini");

        assert_eq!(backfill_trades(&connection).unwrap(), 4);
        let trades = list_trades(&connection, 10).unwrap();
        let summary: Vec<_> = trades.iter().rev().map(|t| (t.side.as_str(), t.exit_reason.as_deref(), t.pnl_usdt.map(|p| (p * 100.0).round() / 100.0))).collect();
        assert_eq!(summary, vec![("SHORT", Some("MANUAL"), Some(0.24)), ("LONG", Some("STOP"), Some(-1.09)), ("LONG", Some("AGENT"), Some(-0.25)), ("SHORT", None, None)]);
        assert_eq!(trades[1].opened_at, "2026-10-29 04:45:20");
        assert_eq!(trades[3].quantity, 0.0012);
        // Stop e alvo vêm da própria ordem (1% e 2% de 80.000), não da configuração atual.
        assert_eq!((trades[3].stop_loss_pct, trades[3].take_profit_pct), (1.0, 2.0));
        assert_eq!(backfill_trades(&connection).unwrap(), 0, "só roda enquanto não há trade fechado");
    }

    #[test]
    fn backfill_completes_a_position_the_engine_already_adopted() {
        // Caso real de 01/11: SHORT aberto às 07:50; o motor novo só o adotou às 13:57.
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection.execute("INSERT INTO trading_equity(created_at,equity,balance,unrealized) VALUES('2026-11-01 10:50:17',99995.5,99995.5,0)", []).unwrap();
        connection.execute(
            "INSERT INTO trading_decisions(created_at,symbol,price,position_before,proposed_action,final_action,order_status,order_detail,confidence,reason) VALUES('2026-11-01 10:50:19','BTC-USDT',77978.8,'FLAT','OPEN_SHORT','OPEN_SHORT','FILLED','Qtd 0.0012',85,'Baixa alinhada')",
            [],
        ).unwrap();
        let adopted = NewTradingTrade { symbol: "BTC-USDT".into(), side: "SHORT".into(), entry_price: 77978.1, quantity: 0.0012, balance_before: 99995.48, entry_reason: "Posição já aberta quando o diário começou.".into(), ..Default::default() };
        open_trade(&connection, &adopted).unwrap();
        connection.execute("UPDATE trading_trades SET opened_at='2026-11-01 16:57:00'", []).unwrap();

        assert_eq!(backfill_trades(&connection).unwrap(), 0);
        let trades = list_trades(&connection, 10).unwrap();
        assert_eq!(trades.len(), 1);
        assert_eq!((trades[0].opened_at.as_str(), trades[0].confidence, trades[0].entry_reason.as_str()), ("2026-11-01 10:50:19", 85, "Baixa alinhada"));
        assert_eq!(trades[0].balance_before, 99995.5);
    }

    #[test]
    fn decisions_and_equity_feed_the_daily_limits() {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        assert_eq!(day_start_equity(&connection).unwrap(), None);
        insert_equity(&connection, 1000.0, 1000.0, 0.0).unwrap();
        insert_equity(&connection, 990.0, 1000.0, -10.0).unwrap();
        assert_eq!(day_start_equity(&connection).unwrap(), Some(1000.0));
        assert_eq!(list_equity(&connection, 10).unwrap().last().unwrap().equity, 990.0);
        let filled = NewTradingDecision { symbol: "BTC-USDT".into(), position_before: "FLAT".into(), proposed_action: "OPEN_LONG".into(), final_action: "OPEN_LONG".into(), order_status: "FILLED".into(), ..Default::default() };
        insert_decision(&connection, &filled).unwrap();
        insert_decision(&connection, &NewTradingDecision { final_action: "HOLD".into(), order_status: "NONE".into(), ..filled.clone() }).unwrap();
        assert_eq!(entries_today(&connection).unwrap(), 1);
        assert_eq!(list_decisions(&connection, 10).unwrap()[0].final_action, "HOLD");
    }
}

