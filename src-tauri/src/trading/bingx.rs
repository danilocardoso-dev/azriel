//! Cliente dos perpétuos USDT-M da BingX.
//! - Dados de mercado (candles, cotação, contratos): API pública real, só leitura e sem chave.
//!   O volume da demo é o do ambiente simulado e não acompanha o mercado (correlação ~0).
//! - Conta, posição e ordens (tudo que é assinado): somente a conta DEMO (VST).
//! Requisições assinadas nunca vão para o endpoint real: dinheiro real exige uma mudança deliberada.

use super::indicators::Candle;
use super::strategy::{OpenPosition, Side};
use hmac::{Hmac, Mac};
use reqwest::{Client, Method};
use serde_json::{json, Value};
use sha2::Sha256;
use std::time::Duration;

pub const VST_BASE_URL: &str = "https://open-api-vst.bingx.com";
pub const MARKET_DATA_BASE_URL: &str = "https://open-api.bingx.com";

fn public_url(path: &str, query: &str) -> String {
    if query.is_empty() { format!("{MARKET_DATA_BASE_URL}{path}") } else { format!("{MARKET_DATA_BASE_URL}{path}?{query}") }
}

fn signed_url(path: &str, query: &str) -> String {
    format!("{VST_BASE_URL}{path}?{query}")
}

#[derive(Debug, Clone)]
pub struct ContractRules {
    pub quantity_precision: u32,
    pub price_precision: u32,
    pub min_quantity: f64,
    pub min_notional: f64,
}

#[derive(Debug, Clone)]
pub struct Ticker {
    pub last_price: f64,
    pub change_pct: f64,
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub equity: f64,
    pub balance: f64,
    pub unrealized: f64,
    pub available_margin: f64,
}

pub struct BingxClient {
    http: Client,
    api_key: String,
    secret: String,
    time_offset_ms: i64,
}

impl BingxClient {
    pub fn new(api_key: &str, secret: &str) -> Result<Self, String> {
        if api_key.trim().is_empty() || secret.trim().is_empty() {
            return Err("Chaves da BingX não configuradas".into());
        }
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| "Não foi possível preparar o cliente da BingX".to_string())?;
        Ok(Self { http, api_key: api_key.trim().into(), secret: secret.trim().into(), time_offset_ms: 0 })
    }

    /// Cliente só para dados de mercado públicos (gráfico), sem chaves.
    pub fn public_only() -> Result<Self, String> {
        let http = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| "Não foi possível preparar o cliente da BingX".to_string())?;
        Ok(Self { http, api_key: String::new(), secret: String::new(), time_offset_ms: 0 })
    }

    /// Candles para o gráfico, incluindo o candle em formação.
    pub async fn live_klines(&self, symbol: &str, interval: &str, limit: u32) -> Result<Vec<Candle>, String> {
        let data = self
            .public("/openApi/swap/v3/quote/klines", &[("symbol", symbol.into()), ("interval", interval.into()), ("limit", limit.to_string())])
            .await?;
        parse_klines(&data, 0, i64::MAX)
    }

    pub async fn ticker(&self, symbol: &str) -> Result<Ticker, String> {
        parse_ticker(&self.public("/openApi/swap/v2/quote/ticker", &[("symbol", symbol.into())]).await?, symbol)
    }

    /// Pares perpétuos USDT ativos, em ordem alfabética.
    pub async fn symbols(&self) -> Result<Vec<String>, String> {
        Ok(parse_symbols(&self.public("/openApi/swap/v2/quote/contracts", &[]).await?))
    }

    /// Alinha o relógio local ao da BingX; requisições assinadas são recusadas com mais de 5 s de diferença.
    pub async fn sync_time(&mut self) -> Result<(), String> {
        let data = self.public("/openApi/swap/v2/server/time", &[]).await?;
        if let Some(server) = number(&data["serverTime"]) {
            self.time_offset_ms = server as i64 - chrono::Utc::now().timestamp_millis();
        }
        Ok(())
    }

    pub async fn klines(&self, symbol: &str, interval: &str, limit: u32) -> Result<Vec<Candle>, String> {
        let data = self
            .public("/openApi/swap/v3/quote/klines", &[("symbol", symbol.into()), ("interval", interval.into()), ("limit", limit.to_string())])
            .await?;
        parse_klines(&data, interval_ms(interval), chrono::Utc::now().timestamp_millis())
    }

    pub async fn contract(&self, symbol: &str) -> Result<ContractRules, String> {
        let data = self.public("/openApi/swap/v2/quote/contracts", &[("symbol", symbol.into())]).await?;
        parse_contract(&data, symbol)
    }

    pub async fn balance(&self) -> Result<Balance, String> {
        parse_balance(&self.signed(Method::GET, "/openApi/swap/v3/user/balance", &[]).await?)
    }

    pub async fn position(&self, symbol: &str) -> Result<Option<OpenPosition>, String> {
        let data = self.signed(Method::GET, "/openApi/swap/v2/user/positions", &[("symbol", symbol.into())]).await?;
        Ok(parse_position(&data))
    }

    /// Preços de stop loss e take profit das ordens pendentes do par (para desenhar no gráfico).
    pub async fn protection_orders(&self, symbol: &str) -> Result<(Option<f64>, Option<f64>), String> {
        let data = self.signed(Method::GET, "/openApi/swap/v2/trade/openOrders", &[("symbol", symbol.into())]).await?;
        Ok(parse_protection(&data))
    }

    pub async fn hedge_mode(&self) -> Result<bool, String> {
        let data = self.signed(Method::GET, "/openApi/swap/v1/positionSide/dual", &[]).await?;
        Ok(match &data["dualSidePosition"] {
            Value::Bool(value) => *value,
            Value::String(value) => value == "true",
            _ => false,
        })
    }

    pub async fn set_leverage(&self, symbol: &str, side: Side, hedge: bool, leverage: i64) -> Result<(), String> {
        self.signed(Method::POST, "/openApi/swap/v2/trade/leverage", &[("symbol", symbol.into()), ("side", position_side(side, hedge).into()), ("leverage", leverage.to_string())])
            .await
            .map(|_| ())
    }

    /// Ordem a mercado com stop loss e take profit anexados (gatilho pelo preço de marcação).
    #[allow(clippy::too_many_arguments)]
    pub async fn open_market(&self, symbol: &str, side: Side, hedge: bool, quantity: f64, quantity_precision: u32, stop_loss: f64, take_profit: f64) -> Result<String, String> {
        let stop = json!({"type": "STOP_MARKET", "stopPrice": stop_loss, "workingType": "MARK_PRICE"});
        let target = json!({"type": "TAKE_PROFIT_MARKET", "stopPrice": take_profit, "workingType": "MARK_PRICE"});
        let data = self
            .signed(Method::POST, "/openApi/swap/v2/trade/order", &[
                ("symbol", symbol.into()),
                ("side", if side == Side::Long { "BUY" } else { "SELL" }.into()),
                ("positionSide", position_side(side, hedge).into()),
                ("type", "MARKET".into()),
                ("quantity", format!("{quantity:.prec$}", prec = quantity_precision as usize)),
                ("stopLoss", stop.to_string()),
                ("takeProfit", target.to_string()),
            ])
            .await?;
        let order = &data["order"];
        Ok(order["orderID"].as_str().map(String::from).or_else(|| order["orderId"].as_i64().map(|id| id.to_string())).unwrap_or_default())
    }

    pub async fn close_all(&self, symbol: &str) -> Result<(), String> {
        self.signed(Method::POST, "/openApi/swap/v2/trade/closeAllPositions", &[("symbol", symbol.into())]).await?;
        self.cancel_open_orders(symbol).await
    }

    /// Remove stops/alvos que sobraram depois que a posição foi encerrada.
    pub async fn cancel_open_orders(&self, symbol: &str) -> Result<(), String> {
        self.signed(Method::DELETE, "/openApi/swap/v2/trade/allOpenOrders", &[("symbol", symbol.into())]).await.map(|_| ())
    }

    async fn public(&self, path: &str, params: &[(&str, String)]) -> Result<Value, String> {
        let query = params.iter().map(|(key, value)| format!("{key}={}", encode(value))).collect::<Vec<_>>().join("&");
        let url = public_url(path, &query);
        // Endpoints públicos aceitam 1 req/s por IP; o gráfico e o motor podem colidir, então tenta de novo uma vez.
        match self.send(self.http.get(&url)).await {
            Ok(data) => Ok(data),
            Err(_) => {
                tokio::time::sleep(Duration::from_millis(1_100)).await;
                self.send(self.http.get(&url)).await
            }
        }
    }

    async fn signed(&self, method: Method, path: &str, params: &[(&str, String)]) -> Result<Value, String> {
        let timestamp = chrono::Utc::now().timestamp_millis() + self.time_offset_ms;
        let query = signed_query(params, &self.secret, timestamp)?;
        let request = self.http.request(method, signed_url(path, &query)).header("X-BX-APIKEY", &self.api_key);
        self.send(request).await
    }

    async fn send(&self, request: reqwest::RequestBuilder) -> Result<Value, String> {
        let response = request.send().await.map_err(|error| {
            if error.is_timeout() { "A BingX excedeu o tempo limite".to_string() } else { "Não foi possível conectar à BingX".to_string() }
        })?;
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        let payload: Value = serde_json::from_str(&text).map_err(|_| format!("Resposta inválida da BingX (HTTP {})", status.as_u16()))?;
        let code = payload["code"].as_i64().unwrap_or(-1);
        if code != 0 {
            let message = payload["msg"].as_str().unwrap_or("erro desconhecido").chars().take(200).collect::<String>();
            return Err(format!("BingX recusou ({code}): {message}"));
        }
        Ok(payload["data"].clone())
    }
}

fn position_side(side: Side, hedge: bool) -> &'static str {
    match (hedge, side) {
        (false, _) => "BOTH",
        (true, Side::Long) => "LONG",
        (true, Side::Short) => "SHORT",
    }
}

/// Assinatura BingX: parâmetros + timestamp em ordem ASCII, `k=v` sem encoding,
/// HMAC-SHA256 em hex minúsculo. Valores só são percent-encoded na URL quando há `{` ou `[`.
pub fn signed_query(params: &[(&str, String)], secret: &str, timestamp: i64) -> Result<String, String> {
    // Impede injeção de parâmetros (o JSON de stop/alvo não usa nenhum desses caracteres).
    if params.iter().any(|(_, value)| value.contains(['&', '=', '?', '#', '\n'])) {
        return Err("Parâmetro inválido para a BingX".into());
    }
    let mut pairs: Vec<(String, String)> = params.iter().map(|(key, value)| (key.to_string(), value.clone())).collect();
    pairs.push(("timestamp".into(), timestamp.to_string()));
    pairs.sort_by(|left, right| left.0.cmp(&right.0));
    let signing = pairs.iter().map(|(key, value)| format!("{key}={value}")).collect::<Vec<_>>().join("&");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| "Chave secreta inválida".to_string())?;
    mac.update(signing.as_bytes());
    let signature: String = mac.finalize().into_bytes().iter().map(|byte| format!("{byte:02x}")).collect();
    let needs_encoding = signing.contains(['{', '[']);
    let query = pairs
        .iter()
        .map(|(key, value)| format!("{key}={}", if needs_encoding { encode(value) } else { value.clone() }))
        .collect::<Vec<_>>()
        .join("&");
    Ok(format!("{query}&signature={signature}"))
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => (byte as char).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

fn number(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_str().and_then(|text| text.trim().parse().ok()))
}

pub fn interval_ms(interval: &str) -> i64 {
    let (amount, unit) = interval.split_at(interval.len().saturating_sub(1));
    let amount: i64 = amount.parse().unwrap_or(1);
    amount * match unit {
        "m" => 60_000,
        "h" => 3_600_000,
        "d" => 86_400_000,
        "w" => 604_800_000,
        _ => 60_000,
    }
}

/// Aceita candles como arrays ou objetos, em qualquer ordem; devolve só candles fechados, do mais antigo ao mais novo.
fn parse_klines(data: &Value, interval_ms: i64, now_ms: i64) -> Result<Vec<Candle>, String> {
    let rows = data.as_array().ok_or("Candles inválidos recebidos da BingX")?;
    let mut candles: Vec<Candle> = rows
        .iter()
        .filter_map(|row| {
            let field = |index: usize, name: &str| if row.is_array() { number(&row[index]) } else { number(&row[name]) };
            Some(Candle {
                open_time: field(0, "time")? as i64,
                open: field(1, "open")?,
                high: field(2, "high")?,
                low: field(3, "low")?,
                close: field(4, "close")?,
                volume: field(5, "volume").unwrap_or(0.0),
            })
        })
        .filter(|candle| candle.open_time + interval_ms <= now_ms)
        .collect();
    candles.sort_by_key(|candle| candle.open_time);
    candles.dedup_by_key(|candle| candle.open_time);
    if candles.is_empty() {
        return Err("A BingX não retornou candles fechados".into());
    }
    Ok(candles)
}

fn parse_contract(data: &Value, symbol: &str) -> Result<ContractRules, String> {
    let entry = data
        .as_array()
        .and_then(|rows| rows.iter().find(|row| row["symbol"].as_str() == Some(symbol)))
        .or_else(|| (data["symbol"].as_str() == Some(symbol)).then_some(data))
        .ok_or_else(|| format!("Contrato {symbol} não encontrado na BingX"))?;
    Ok(ContractRules {
        quantity_precision: number(&entry["quantityPrecision"]).unwrap_or(4.0) as u32,
        price_precision: number(&entry["pricePrecision"]).unwrap_or(2.0) as u32,
        min_quantity: number(&entry["tradeMinQuantity"]).unwrap_or(0.0),
        min_notional: number(&entry["tradeMinUSDT"]).unwrap_or(0.0),
    })
}

fn parse_ticker(data: &Value, symbol: &str) -> Result<Ticker, String> {
    let entry = match data {
        Value::Array(rows) => rows.iter().find(|row| row["symbol"].as_str() == Some(symbol)).or_else(|| rows.first()),
        Value::Object(_) => Some(data),
        _ => None,
    }
    .ok_or("Cotação não encontrada na BingX")?;
    let percent = |value: &Value| number(value).or_else(|| value.as_str().and_then(|text| text.trim_end_matches('%').trim().parse().ok()));
    let last_price = number(&entry["lastPrice"]).ok_or("Cotação inválida recebida da BingX")?;
    Ok(Ticker {
        last_price,
        change_pct: percent(&entry["priceChangePercent"]).unwrap_or(0.0),
        high: number(&entry["highPrice"]).unwrap_or(last_price),
        low: number(&entry["lowPrice"]).unwrap_or(last_price),
    })
}

fn parse_protection(data: &Value) -> (Option<f64>, Option<f64>) {
    let orders = data["orders"].as_array().or_else(|| data.as_array()).cloned().unwrap_or_default();
    let price_of = |types: &[&str]| orders.iter().find(|order| order["type"].as_str().is_some_and(|kind| types.contains(&kind))).and_then(|order| number(&order["stopPrice"]));
    (price_of(&["STOP_MARKET", "STOP"]), price_of(&["TAKE_PROFIT_MARKET", "TAKE_PROFIT"]))
}

fn parse_symbols(data: &Value) -> Vec<String> {
    let mut symbols: Vec<String> = data
        .as_array()
        .map(|rows| {
            rows.iter()
                .filter(|row| number(&row["status"]).is_none_or(|status| status == 1.0))
                .filter_map(|row| row["symbol"].as_str())
                .filter(|symbol| symbol.ends_with("-USDT"))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    symbols.sort();
    symbols.dedup();
    symbols
}

fn parse_balance(data: &Value) -> Result<Balance, String> {
    let entry = match data {
        Value::Array(rows) => rows.iter().find(|row| row["asset"].as_str() == Some("USDT")).or_else(|| rows.first()),
        Value::Object(_) if data["balance"].is_object() => Some(&data["balance"]),
        Value::Object(_) => Some(data),
        _ => None,
    }
    .ok_or("Saldo não encontrado na BingX")?;
    let equity = number(&entry["equity"]).ok_or("Saldo inválido recebido da BingX")?;
    Ok(Balance {
        equity,
        balance: number(&entry["balance"]).unwrap_or(equity),
        unrealized: number(&entry["unrealizedProfit"]).unwrap_or(0.0),
        available_margin: number(&entry["availableMargin"]).unwrap_or(0.0),
    })
}

fn parse_position(data: &Value) -> Option<OpenPosition> {
    data.as_array()?.iter().find_map(|row| {
        let amount = number(&row["positionAmt"])?;
        if amount == 0.0 {
            return None;
        }
        let side = if row["positionSide"].as_str() == Some("SHORT") || amount < 0.0 { Side::Short } else { Side::Long };
        Some(OpenPosition {
            side,
            quantity: amount.abs(),
            entry_price: number(&row["avgPrice"]).unwrap_or(0.0),
            unrealized: number(&row["unrealizedProfit"]).unwrap_or(0.0),
            leverage: number(&row["leverage"]).unwrap_or(1.0) as i64,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn market_data_is_real_but_signed_requests_only_reach_the_demo() {
        assert!(public_url("/openApi/swap/v3/quote/klines", "symbol=BTC-USDT").starts_with("https://open-api.bingx.com/"));
        assert!(signed_url("/openApi/swap/v2/trade/order", "x=1").starts_with("https://open-api-vst.bingx.com/"));
        assert!(signed_url("/openApi/swap/v3/user/balance", "x=1").contains("-vst."));
    }

    #[test]
    fn signature_matches_hmac_of_sorted_unencoded_params() {
        let query = signed_query(&[("symbol", "BTC-USDT".into()), ("recvWindow", "0".into())], "secret", 1696751141337).unwrap();
        let (unsigned, signature) = query.split_once("&signature=").unwrap();
        assert_eq!(unsigned, "recvWindow=0&symbol=BTC-USDT&timestamp=1696751141337");
        let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
        mac.update(unsigned.as_bytes());
        let expected: String = mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(signature, expected);
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn json_params_are_signed_raw_and_encoded_in_the_url() {
        let stop = json!({"type":"STOP_MARKET","stopPrice":63538.3}).to_string();
        let query = signed_query(&[("stopLoss", stop.clone()), ("symbol", "BTC-USDT".into())], "secret", 1).unwrap();
        assert!(query.starts_with("stopLoss=%7B%22"));
        assert!(query.contains("%22type%22%3A%22STOP_MARKET%22"));
        assert!(!query.contains('{'));
        assert!(signed_query(&[("symbol", "BTC-USDT&x=1".into())], "secret", 1).is_err());
    }

    #[test]
    fn klines_accept_arrays_or_objects_and_drop_the_open_candle() {
        let now = 10 * 60_000;
        let objects = json!([
            {"open":"3","close":"4","high":"5","low":"2","volume":"7","time": 9 * 60_000},
            {"open":"1","close":"2","high":"3","low":"0.5","volume":"9","time": 8 * 60_000},
        ]);
        let candles = parse_klines(&objects, 60_000, now).unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].close, 2.0);
        let open = json!([[now, "1", "2", "0.5", "1.5", "3"]]);
        assert!(parse_klines(&open, 60_000, now).is_err());
        assert_eq!(parse_klines(&open, 0, i64::MAX).unwrap().len(), 1, "o gráfico mostra o candle em formação");
        let arrays = json!([[0, 1.0, 2.0, 0.5, 1.5, 3.0, 59_999]]);
        assert_eq!(parse_klines(&arrays, 60_000, now).unwrap()[0].high, 2.0);
    }

    #[test]
    fn account_payloads_are_parsed() {
        let balance = parse_balance(&json!([{"asset":"USDT","balance":"194.82","equity":"196.74","unrealizedProfit":"1.92","availableMargin":"193.76"}])).unwrap();
        assert_eq!(balance.equity, 196.74);
        assert_eq!(parse_balance(&json!({"balance":{"equity":"10","balance":"9"}})).unwrap().balance, 9.0);
        let position = parse_position(&json!([{"positionSide":"SHORT","positionAmt":"0.20","avgPrice":"246.43","unrealizedProfit":"-0.06","leverage":5}])).unwrap();
        assert_eq!((position.side, position.quantity, position.leverage), (Side::Short, 0.2, 5));
        assert!(parse_position(&json!([{"positionSide":"LONG","positionAmt":"0"}])).is_none());
        assert_eq!(parse_protection(&json!({"orders":[{"type":"TAKE_PROFIT_MARKET","stopPrice":"82000"},{"type":"STOP_MARKET","stopPrice":"79000"}]})), (Some(79000.0), Some(82000.0)));
        assert_eq!(parse_protection(&json!([])), (None, None));
        let ticker = parse_ticker(&json!({"symbol":"BTC-USDT","lastPrice":"80633.2","priceChangePercent":"-1.25%","highPrice":"81900","lowPrice":"79800"}), "BTC-USDT").unwrap();
        assert_eq!((ticker.last_price, ticker.change_pct, ticker.high), (80633.2, -1.25, 81900.0));
        assert_eq!(parse_symbols(&json!([{"symbol":"SOL-USDT","status":1},{"symbol":"BTC-USDT","status":1},{"symbol":"OLD-USDT","status":0},{"symbol":"BTC-USDC","status":1}])), vec!["BTC-USDT", "SOL-USDT"]);
        let rules = parse_contract(&json!([{"symbol":"BTC-USDT","quantityPrecision":4,"pricePrecision":1,"tradeMinQuantity":0.0001,"tradeMinUSDT":2}]), "BTC-USDT").unwrap();
        assert_eq!((rules.quantity_precision, rules.price_precision), (4, 1));
        assert_eq!(interval_ms("15m"), 900_000);
        assert_eq!(interval_ms("1h"), 3_600_000);
    }
}
