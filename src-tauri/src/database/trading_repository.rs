use super::trading_models::*;
use rusqlite::{params, Connection, OptionalExtension};

const INTERVALS: [&str; 6] = ["5m", "15m", "30m", "1h", "2h", "4h"];
const ENGINES: [&str; 2] = ["gemini", "ollama"];

pub fn get_settings(connection: &Connection) -> Result<TradingSettings, String> {
    connection
        .query_row(
            "SELECT api_key<>'' AND secret_key<>'',symbol,candle_interval,cycle_minutes,margin_usdt,leverage,stop_loss_pct,take_profit_pct,max_daily_loss_pct,min_confidence,engine,running,updated_at FROM trading_settings WHERE id=1",
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
    Ok(())
}

pub fn update_settings(connection: &Connection, input: &TradingSettingsInput) -> Result<TradingSettings, String> {
    validate_settings(input)?;
    connection
        .execute(
            "UPDATE trading_settings SET symbol=?1,candle_interval=?2,cycle_minutes=?3,margin_usdt=?4,leverage=?5,stop_loss_pct=?6,take_profit_pct=?7,max_daily_loss_pct=?8,min_confidence=?9,engine=?10,updated_at=CURRENT_TIMESTAMP WHERE id=1",
            params![input.symbol.trim(), input.candle_interval, input.cycle_minutes, input.margin_usdt, input.leverage, input.stop_loss_pct, input.take_profit_pct, input.max_daily_loss_pct, input.min_confidence, input.engine],
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
