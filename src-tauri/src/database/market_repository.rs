use super::{ai_repository, market_ai, market_models::*, market_observatory, market_regimes, market_signals};
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde_json::json;
use std::{collections::{HashMap, HashSet}, fs, path::Path, sync::atomic::{AtomicBool, Ordering}, time::{SystemTime, UNIX_EPOCH}};

pub const MAX_CONCURRENT_AGENTS: usize = 5;
const MONEY_SCALE: f64 = 1_000_000.0;
static MARKET_KILL_SWITCH: AtomicBool = AtomicBool::new(false);

pub fn set_kill_switch(active: bool) { MARKET_KILL_SWITCH.store(active, Ordering::SeqCst); }
pub fn kill_switch_active() -> bool { MARKET_KILL_SWITCH.load(Ordering::SeqCst) }

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Candle { pub(crate) timestamp: String, pub(crate) open: f64, pub(crate) high: f64, pub(crate) low: f64, pub(crate) close: f64, pub(crate) volume: Option<f64> }

#[derive(Debug, Clone, Default)]
struct Features { returns: Option<f64>, sma_short: Option<f64>, sma_long: Option<f64>, volatility: Option<f64>, momentum: Option<f64> }

#[derive(Debug, Clone)]
pub(crate) struct Decision { pub(crate) action: &'static str, pub(crate) desired_pct: Option<f64>, confidence: Option<f64>, reasoning: String }

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PendingOrder { decision_index: usize, action: &'static str, approved_pct: f64 }

#[derive(Debug, Clone)]
pub(crate) struct Portfolio {
    cash: f64, quantity: f64, average_entry: f64, realized: f64, peak_equity: f64,
    pub(crate) trades: usize, pub(crate) wins: usize, pub(crate) gross_profit: f64, pub(crate) gross_loss: f64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct AgentRun { pub(crate) id: String, pub(crate) name: String, status: String, portfolio: Portfolio, pending: Option<PendingOrder>, position_started_index: Option<usize>, pub(crate) decisions: Vec<DecisionRow>, pub(crate) snapshots: Vec<SnapshotRow>, pub(crate) max_drawdown: f64, exposure_sum: f64, max_exposure: f64, pub(crate) hold_count: usize, pub(crate) ai_runtime: market_ai::MarketAiRuntimeMetrics }

#[derive(Debug, Clone)]
pub(crate) struct DecisionRow { pub(crate) candle_index: usize, timestamp: String, observed_price: f64, pub(crate) decision: Decision, risk_result: String, risk_reason: String, validation_code: Option<String>, approved_pct: Option<f64>, cash_before: f64, equity_before: f64, exposure_before: f64, pub(crate) execution: Option<ExecutionRow>, ai: Option<market_ai::MarketAiDecision> }

#[derive(Debug, Clone)]
pub(crate) struct ExecutionRow { timestamp: String, side: String, requested_price: f64, price: f64, quantity: f64, gross: f64, fees: f64, slippage: f64, net: f64 }

#[derive(Debug, Clone)]
pub(crate) struct SnapshotRow { pub(crate) candle_index: usize, pub(crate) timestamp: String, cash: f64, quantity: f64, market_value: f64, pub(crate) equity: f64, pub(crate) exposure: f64, pub(crate) realized: f64, unrealized: f64 }

fn round(value: f64) -> f64 { (value * MONEY_SCALE).round() / MONEY_SCALE }
fn new_id(prefix: &str) -> String { format!("{prefix}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos()) }

fn split_csv(line: &str, delimiter: char) -> Result<Vec<String>, String> {
    let mut result=Vec::new(); let mut current=String::new(); let mut quoted=false; let mut chars=line.chars().peekable();
    while let Some(ch)=chars.next() { match ch { '"' if quoted && chars.peek()==Some(&'"') => { current.push('"'); chars.next(); }, '"' => quoted=!quoted, c if c==delimiter && !quoted => { result.push(current.trim().to_string()); current.clear(); }, _ => current.push(ch) } }
    if quoted { return Err("CSV inválido: aspas não encerradas".into()); }
    result.push(current.trim().to_string()); Ok(result)
}

fn timestamp_key(value: &str) -> Result<String, String> {
    let value=value.trim();
    if !value.is_empty() && value.chars().all(|c| c.is_ascii_digit()) { return Ok(format!("N{:0>20}", value)); }
    let core=value.strip_suffix('Z').unwrap_or(value); let parts: Vec<&str>=core.split(['T',' ']).collect();
    if parts.is_empty() || parts.len()>2 { return Err(format!("data/timestamp inválido: {value}")); }
    let date: Vec<&str>=parts[0].split('-').collect();
    if date.len()!=3 { return Err(format!("data/timestamp inválido: {value}")); }
    let date_numbers: Result<Vec<u32>,_>=date.iter().map(|part|part.parse()).collect();
    let date_values=date_numbers.map_err(|_|format!("data/timestamp inválido: {value}"))?;
    if date_values[1]==0 || date_values[1]>12 || date_values[2]==0 || date_values[2]>31 { return Err(format!("data/timestamp inválido: {value}")); }
    if parts.len()==1 { return Ok(format!("D{:04}{:02}{:02}000000",date_values[0],date_values[1],date_values[2])); }
    let time: Vec<&str>=parts[1].split(':').collect();
    if time.len()<2 || time.len()>3 { return Err(format!("data/timestamp inválido: {value}")); }
    let time_numbers: Result<Vec<u32>,_>=time.iter().map(|part|part.split('.').next().unwrap_or("").parse()).collect();
    let time_values=time_numbers.map_err(|_|format!("data/timestamp inválido: {value}"))?;
    if time_values[0]>23 || time_values[1]>59 || time_values.get(2).copied().unwrap_or(0)>59 { return Err(format!("data/timestamp inválido: {value}")); }
    Ok(format!("D{:04}{:02}{:02}{:02}{:02}{:02}",date_values[0],date_values[1],date_values[2],time_values[0],time_values[1],time_values.get(2).copied().unwrap_or(0)))
}

fn parse_csv(content: &str) -> Result<Vec<Candle>, String> {
    let mut lines=content.lines().filter(|line| !line.trim().is_empty()); let header_line=lines.next().ok_or("CSV vazio")?;
    let delimiter=if header_line.matches(';').count()>header_line.matches(',').count(){';'}else{','};
    let headers=split_csv(header_line.trim_start_matches('\u{feff}'),delimiter)?; let normalized: Vec<String>=headers.iter().map(|h|h.trim().to_ascii_lowercase()).collect();
    let index=|name:&str| normalized.iter().position(|h|h==name).ok_or_else(||format!("coluna obrigatória ausente: {name}"));
    let ti=normalized.iter().position(|h|h=="timestamp").or_else(||normalized.iter().position(|h|h=="date")).ok_or("coluna temporal obrigatória ausente: use timestamp ou date")?; let oi=index("open")?; let hi=index("high")?; let li=index("low")?; let ci=index("close")?; let vi=normalized.iter().position(|h|h=="volume");
    let mut candles=Vec::new(); let mut previous=None; let mut seen=HashSet::new();
    for (offset,line) in lines.enumerate() { let row=split_csv(line,delimiter)?; let row_number=offset+2; let get=|i:usize|row.get(i).map(String::as_str).ok_or_else(||format!("linha {row_number}: coluna ausente"));
        let timestamp=get(ti)?.trim().to_string(); let key=timestamp_key(&timestamp).map_err(|e|format!("linha {row_number}: {e}"))?;
        if !seen.insert(key.clone()) { return Err(format!("linha {row_number}: timestamp duplicado")); }
        if previous.as_ref().is_some_and(|p:&String| key<=*p) { return Err(format!("linha {row_number}: timestamps fora de ordem")); } previous=Some(key);
        let number=|i|->Result<f64,String>{get(i)?.replace(',', ".").parse::<f64>().map_err(|_|format!("linha {row_number}: número inválido"))};
        let open=number(oi)?; let high=number(hi)?; let low=number(li)?; let close=number(ci)?; let volume=vi.map(number).transpose()?;
        if ![open,high,low,close].iter().all(|v|v.is_finite()&&*v>0.0) || high<low || high<open || high<close || low>open || low>close { return Err(format!("linha {row_number}: OHLC inconsistente")); }
        if volume.is_some_and(|v|!v.is_finite()||v<0.0) { return Err(format!("linha {row_number}: volume inválido")); }
        candles.push(Candle{timestamp,open,high,low,close,volume});
    }
    if candles.is_empty(){return Err("dataset sem candles".into())} Ok(candles)
}

fn fingerprint(content:&str,asset:&str,timeframe:&str)->String { let mut hash:u64=0xcbf29ce484222325; for byte in content.bytes().chain(asset.bytes()).chain(timeframe.bytes()){hash^=byte as u64;hash=hash.wrapping_mul(0x100000001b3);} format!("fnv1a64:{hash:016x}") }

pub fn import_dataset(connection:&mut Connection,input:&ImportMarketDatasetInput)->Result<MarketDataset,String>{
    if input.name.trim().is_empty()||input.asset.trim().is_empty()||input.timeframe.trim().is_empty(){return Err("nome, ativo e timeframe são obrigatórios".into())}
    let path=Path::new(&input.path); if path.extension().and_then(|v|v.to_str()).map(|v|v.eq_ignore_ascii_case("csv"))!=Some(true){return Err("selecione um arquivo CSV".into())}
    let content=fs::read_to_string(path).map_err(|e|format!("não foi possível ler o CSV: {e}"))?; let candles=parse_csv(&content)?; let fp=fingerprint(&content,input.asset.trim(),input.timeframe.trim());
    if let Some(existing)=connection.query_row("SELECT id FROM market_datasets WHERE fingerprint=?1",[&fp],|r|r.get::<_,String>(0)).optional().map_err(|e|e.to_string())? { return get_dataset(connection,&existing); }
    let id=new_id("dataset"); let currency=input.currency.as_deref().unwrap_or("BRL").trim().to_ascii_uppercase(); let start=candles[0].timestamp.clone(); let end=candles.last().unwrap().timestamp.clone();
    let tx=connection.transaction().map_err(|e|e.to_string())?; tx.execute("INSERT INTO market_datasets(id,name,asset,timeframe,currency,start_at,end_at,candle_count,fingerprint,source_path) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![id,input.name.trim(),input.asset.trim().to_ascii_uppercase(),input.timeframe.trim(),currency,start,end,candles.len(),fp,input.path]).map_err(|e|e.to_string())?;
    {let mut statement=tx.prepare("INSERT INTO market_candles(dataset_id,candle_index,timestamp,open,high,low,close,volume) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)").map_err(|e|e.to_string())?; for (index,c) in candles.iter().enumerate(){statement.execute(params![id,index,c.timestamp,c.open,c.high,c.low,c.close,c.volume]).map_err(|e|e.to_string())?;}}
    tx.commit().map_err(|e|e.to_string())?; get_dataset(connection,&id)
}

fn map_dataset(row:&rusqlite::Row<'_>)->rusqlite::Result<MarketDataset>{Ok(MarketDataset{id:row.get(0)?,name:row.get(1)?,asset:row.get(2)?,timeframe:row.get(3)?,currency:row.get(4)?,start_at:row.get(5)?,end_at:row.get(6)?,candle_count:row.get(7)?,fingerprint:row.get(8)?,source_path:row.get(9)?,imported_at:row.get(10)?})}
pub fn list_datasets(connection:&Connection)->Result<Vec<MarketDataset>,String>{let mut s=connection.prepare("SELECT id,name,asset,timeframe,currency,start_at,end_at,candle_count,fingerprint,source_path,imported_at FROM market_datasets ORDER BY imported_at DESC").map_err(|e|e.to_string())?;let result=s.query_map([],map_dataset).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string());result}
fn get_dataset(connection:&Connection,id:&str)->Result<MarketDataset,String>{connection.query_row("SELECT id,name,asset,timeframe,currency,start_at,end_at,candle_count,fingerprint,source_path,imported_at FROM market_datasets WHERE id=?1",[id],map_dataset).optional().map_err(|e|e.to_string())?.ok_or("dataset não encontrado".into())}

pub fn list_agents(connection:&Connection)->Result<Vec<MarketAgentDefinition>,String>{let mut s=connection.prepare("SELECT id,name,strategy_type,strategy_version,default_config_json,enabled FROM market_agents ORDER BY rowid").map_err(|e|e.to_string())?;let result=s.query_map([],|r|Ok(MarketAgentDefinition{id:r.get(0)?,name:r.get(1)?,strategy_type:r.get(2)?,strategy_version:r.get(3)?,default_config_json:r.get(4)?,enabled:r.get(5)?})).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string());result}
pub fn list_risk_profiles(connection:&Connection)->Result<Vec<MarketRiskProfile>,String>{let mut s=connection.prepare("SELECT id,name,max_position_pct,max_total_exposure_pct,max_daily_loss_pct,max_drawdown_pct,max_trades_per_day,allow_leverage,allow_short,allowed_assets_json FROM market_risk_profiles ORDER BY name").map_err(|e|e.to_string())?;let result=s.query_map([],|r|{let raw:String=r.get(9)?;Ok(MarketRiskProfile{id:r.get(0)?,name:r.get(1)?,max_position_pct:r.get(2)?,max_total_exposure_pct:r.get(3)?,max_daily_loss_pct:r.get(4)?,max_drawdown_pct:r.get(5)?,max_trades_per_day:r.get(6)?,allow_leverage:r.get(7)?,allow_short:r.get(8)?,allowed_assets:serde_json::from_str(&raw).unwrap_or_default()})}).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string());result}

fn load_candles(connection:&Connection,dataset_id:&str)->Result<Vec<Candle>,String>{let mut s=connection.prepare("SELECT timestamp,open,high,low,close,volume FROM market_candles WHERE dataset_id=?1 ORDER BY candle_index").map_err(|e|e.to_string())?;let result=s.query_map([dataset_id],|r|Ok(Candle{timestamp:r.get(0)?,open:r.get(1)?,high:r.get(2)?,low:r.get(3)?,close:r.get(4)?,volume:r.get(5)?})).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string());result}

fn feature_at(candles:&[Candle],i:usize)->Features{let sma=|period:usize|if i+1>=period{Some(candles[i+1-period..=i].iter().map(|c|c.close).sum::<f64>()/period as f64)}else{None};let returns=if i>0{Some(candles[i].close/candles[i-1].close-1.0)}else{None};let volatility=if i>=5{let rs:Vec<f64>=(i+1-5..=i).filter(|&j|j>0).map(|j|candles[j].close/candles[j-1].close-1.0).collect();let mean=rs.iter().sum::<f64>()/rs.len() as f64;Some((rs.iter().map(|r|(r-mean).powi(2)).sum::<f64>()/rs.len() as f64).sqrt())}else{None};Features{returns,sma_short:sma(5),sma_long:sma(20),volatility,momentum:if i>=5{Some(candles[i].close/candles[i-5].close-1.0)}else{None}}}

fn return_at(candles: &[Candle], index: usize, period: usize) -> Option<f64> {
    (index >= period).then(|| candles[index].close / candles[index - period].close - 1.0)
}

fn sma_at(candles: &[Candle], index: usize, period: usize) -> Option<f64> {
    (index + 1 >= period).then(|| {
        candles[index + 1 - period..=index]
            .iter()
            .map(|candle| candle.close)
            .sum::<f64>()
            / period as f64
    })
}

fn rolling_volatility(candles: &[Candle], index: usize, period: usize) -> Option<f64> {
    if index + 1 < period || period < 2 {
        return None;
    }
    let start = index + 1 - period;
    let returns: Vec<f64> = candles[start..=index]
        .windows(2)
        .map(|pair| pair[1].close / pair[0].close - 1.0)
        .collect();
    let mean = returns.iter().sum::<f64>() / returns.len() as f64;
    Some(
        (returns
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / returns.len() as f64)
            .sqrt(),
    )
}

fn ratio_pct(value: Option<f64>, base: Option<f64>) -> Option<f64> {
    match (value, base) {
        (Some(value), Some(base)) if base != 0.0 => Some((value / base - 1.0) * 100.0),
        _ => None,
    }
}

fn ai_regime_config() -> MarketRegimeConfig {
    MarketRegimeConfig {
        trend_window: 20,
        volatility_window: 20,
        bull_threshold_pct: 2.0,
        bear_threshold_pct: -2.0,
        high_volatility_threshold_pct: 1.5,
        low_volatility_threshold_pct: 0.25,
        minimum_sample_candles: 3,
        minimum_sample_trades: 1,
    }
}
fn random_unit(seed:u64,index:usize,agent:&str)->f64{let mut x=seed^(index as u64).wrapping_mul(0x9E3779B97F4A7C15);for b in agent.bytes(){x^=b as u64;x=x.wrapping_mul(0x100000001b3);}x^=x>>12;x^=x<<25;x^=x>>27;(x.wrapping_mul(2685821657736338717)%1_000_000)as f64/1_000_000.0}
fn decide(agent:&str,index:usize,features:&Features,position:bool,seed:u64)->Decision{match agent{
    "cash"=>Decision{action:"HOLD",desired_pct:Some(0.0),confidence:Some(1.0),reasoning:"Controle permanece integralmente em caixa".into()},
    "buy-hold" if !position=>Decision{action:"BUY",desired_pct:Some(100.0),confidence:Some(1.0),reasoning:"Primeira oportunidade executável do período".into()},
    "buy-hold"=>Decision{action:"HOLD",desired_pct:Some(100.0),confidence:Some(1.0),reasoning:"Posição de benchmark mantida".into()},
    "simple-trend"=>match(features.sma_short,features.sma_long){(Some(s),Some(l))if s>l=>Decision{action:"BUY",desired_pct:Some(80.0),confidence:Some(((s/l-1.0)*10.0).clamp(0.0,1.0)),reasoning:"SMA curta acima da SMA longa".into()},(Some(s),Some(l))if s<l=>Decision{action:"SELL",desired_pct:Some(0.0),confidence:Some(((l/s-1.0)*10.0).clamp(0.0,1.0)),reasoning:"SMA curta abaixo da SMA longa".into()},_=>Decision{action:"HOLD",desired_pct:None,confidence:None,reasoning:"Warmup ou médias equivalentes".into()}},
    "simple-momentum"=>match features.momentum{Some(v)if v>0.0=>Decision{action:"BUY",desired_pct:Some(60.0),confidence:Some((v*10.0).clamp(0.0,1.0)),reasoning:"Momentum de 5 candles positivo".into()},Some(v)if v<0.0=>Decision{action:"SELL",desired_pct:Some(0.0),confidence:Some((-v*10.0).clamp(0.0,1.0)),reasoning:"Momentum de 5 candles negativo".into()},_=>Decision{action:"HOLD",desired_pct:None,confidence:None,reasoning:"Warmup ou momentum neutro".into()}},
    "random-controlled"=>{let r=random_unit(seed,index,agent);if r<0.15{Decision{action:"BUY",desired_pct:Some(40.0),confidence:Some(r),reasoning:"Amostra pseudoaleatória determinística: BUY".into()}}else if r<0.30{Decision{action:"SELL",desired_pct:Some(0.0),confidence:Some(r),reasoning:"Amostra pseudoaleatória determinística: SELL".into()}}else{Decision{action:"HOLD",desired_pct:None,confidence:Some(r),reasoning:"Amostra pseudoaleatória determinística: HOLD".into()}}},
    _=>Decision{action:"HOLD",desired_pct:None,confidence:None,reasoning:"Agente inválido".into()}
}}

fn equity(p:&Portfolio,price:f64)->f64{round(p.cash+p.quantity*price)}
fn exposure(p:&Portfolio,price:f64)->f64{let e=equity(p,price);if e<=0.0{0.0}else{p.quantity*price/e*100.0}}
fn execute(p:&mut Portfolio,pending:&PendingOrder,candle:&Candle,fee_pct:f64,slippage_pct:f64)->Option<ExecutionRow>{let current=equity(p,candle.open);let target=round(current*pending.approved_pct/100.0);let current_value=round(p.quantity*candle.open);let delta=target-current_value;if delta.abs()<0.000001{return None}let buying=delta>0.0;let price=round(candle.open*(if buying{1.0+slippage_pct/100.0}else{1.0-slippage_pct/100.0}));let mut quantity=round(delta.abs()/price);if buying{quantity=quantity.min(round(p.cash/(price*(1.0+fee_pct/100.0))))}else{quantity=quantity.min(p.quantity)}if quantity<=0.0{return None}let gross=round(quantity*price);let fees=round(gross*fee_pct/100.0);let net=round(if buying{gross+fees}else{gross-fees});if buying{let old_cost=p.quantity*p.average_entry;p.cash=round(p.cash-net);p.quantity=round(p.quantity+quantity);p.average_entry=if p.quantity>0.0{round((old_cost+gross+fees)/p.quantity)}else{0.0};}else{let pnl=round((price-p.average_entry)*quantity-fees);p.realized=round(p.realized+pnl);if pnl>=0.0{p.wins+=1;p.gross_profit=round(p.gross_profit+pnl)}else{p.gross_loss=round(p.gross_loss-pnl)}p.cash=round(p.cash+net);p.quantity=round(p.quantity-quantity);if p.quantity<=0.000001{p.quantity=0.0;p.average_entry=0.0;}}p.trades+=1;Some(ExecutionRow{timestamp:candle.timestamp.clone(),side:if buying{"BUY"}else{"SELL"}.into(),requested_price:candle.open,price,quantity,gross,fees,slippage:round((price-candle.open).abs()*quantity),net})}

fn risk(decision:&Decision,p:&Portfolio,price:f64,profile:&MarketRiskProfile,asset:&str,kill:bool)->(String,String,Option<f64>){if kill||kill_switch_active(){return("REJECTED".into(),"Kill switch ativo".into(),None)}if !profile.allowed_assets.is_empty()&&!profile.allowed_assets.iter().any(|a|a.eq_ignore_ascii_case(asset)){return("REJECTED".into(),"Ativo fora da allowlist".into(),None)}if decision.action=="HOLD"{return("APPROVED".into(),"HOLD não gera ordem".into(),None)}if profile.max_trades_per_day.is_some_and(|max|p.trades>=max){return("REJECTED".into(),"Limite de operações atingido".into(),None)}let e=equity(p,price);let drawdown=if p.peak_equity>0.0{(p.peak_equity-e)/p.peak_equity*100.0}else{0.0};if drawdown>=profile.max_drawdown_pct{return("REJECTED".into(),"Drawdown máximo atingido".into(),None)}let requested=decision.desired_pct.unwrap_or(if decision.action=="SELL"{0.0}else{exposure(p,price)}).clamp(0.0,100.0);let approved=requested.min(profile.max_position_pct).min(profile.max_total_exposure_pct);if (approved-requested).abs()>0.000001{("MODIFIED".into(),format!("Exposição limitada a {:.2}%",approved),Some(approved))}else{("APPROVED".into(),"Dentro dos limites do perfil".into(),Some(approved))}}

fn build_v2_snapshot(
    dataset: &MarketDataset,
    profile: &MarketRiskProfile,
    candles: &[Candle],
    index: usize,
    run: &AgentRun,
    equity_now: f64,
    exposure_now: f64,
) -> market_ai::MarketAiSnapshotV2 {
    let price = candles[index].close;
    let sma_short = sma_at(candles, index, 5);
    let sma_long = sma_at(candles, index, 20);
    let previous_short = index.checked_sub(5).and_then(|at| sma_at(candles, at, 5));
    let closes: Vec<f64> = candles.iter().map(|candle| candle.close).collect();
    let regime = market_regimes::classify(&closes, index, &ai_regime_config());
    let previous_decision = run.decisions.iter().rev().find_map(|row| {
        row.ai.as_ref().and_then(|decision| {
            (decision.call_status == "VALID").then(|| market_ai::MarketAiPreviousDecision {
                action: decision.action.into(),
                confidence: decision.confidence.unwrap_or(0.0),
            })
        })
    });
    market_ai::MarketAiSnapshotV2 {
        market: market_ai::MarketAiMarket {
            timestamp: candles[index].timestamp.clone(),
            asset: dataset.asset.clone(),
            timeframe: dataset.timeframe.clone(),
            price,
        },
        returns: market_ai::MarketAiReturns {
            return_1: return_at(candles, index, 1),
            return_5: return_at(candles, index, 5),
            return_10: return_at(candles, index, 10),
            return_20: return_at(candles, index, 20),
        },
        trend: market_ai::MarketAiTrend {
            sma_short,
            sma_long,
            price_vs_sma_short_pct: ratio_pct(Some(price), sma_short),
            price_vs_sma_long_pct: ratio_pct(Some(price), sma_long),
            sma_spread_pct: ratio_pct(sma_short, sma_long),
            short_sma_slope: ratio_pct(sma_short, previous_short).map(|value| value / 100.0),
        },
        momentum: market_ai::MarketAiMomentum {
            momentum_5: return_at(candles, index, 5),
            momentum_10: return_at(candles, index, 10),
        },
        volatility: market_ai::MarketAiVolatility {
            rolling_volatility: rolling_volatility(candles, index, 20),
            volatility_regime: regime.volatility.to_ascii_uppercase(),
        },
        regime: market_ai::MarketAiRegime {
            trend: regime.trend.to_ascii_uppercase(),
            volatility: regime.volatility.to_ascii_uppercase(),
            engine_version: market_regimes::REGIME_ENGINE_VERSION.into(),
        },
        portfolio: market_ai::MarketAiPortfolioV2 {
            cash: run.portfolio.cash,
            equity: equity_now,
            current_position: run.portfolio.quantity > 0.0,
            position_pct: exposure_now,
            entry_price: (run.portfolio.quantity > 0.0).then_some(run.portfolio.average_entry),
            unrealized_pnl_pct: (run.portfolio.quantity > 0.0 && run.portfolio.average_entry > 0.0)
                .then(|| (price / run.portfolio.average_entry - 1.0) * 100.0),
            holding_period: run
                .position_started_index
                .map(|started| index.saturating_sub(started)),
        },
        previous_decision,
        risk_context: market_ai::MarketAiRiskContext {
            max_position_pct: profile.max_position_pct,
            max_total_exposure_pct: profile.max_total_exposure_pct,
            allow_short: profile.allow_short,
            allow_leverage: profile.allow_leverage,
        },
    }
}

fn build_v3_snapshot(dataset:&MarketDataset,profile:&MarketRiskProfile,candles:&[Candle],index:usize,run:&AgentRun,equity_now:f64,exposure_now:f64)->Result<market_ai::MarketAiSnapshotV3,String>{
    let v2=build_v2_snapshot(dataset,profile,candles,index,run,equity_now,exposure_now);
    let previous_long=index.checked_sub(5).and_then(|at|sma_at(candles,at,20));
    let long_slope=ratio_pct(v2.trend.sma_long,previous_long);
    let signal_input=market_signals::MarketSignalInput{
        price_vs_sma_short_pct:v2.trend.price_vs_sma_short_pct,price_vs_sma_long_pct:v2.trend.price_vs_sma_long_pct,
        sma_spread_pct:v2.trend.sma_spread_pct,sma_short_slope_pct:v2.trend.short_sma_slope.map(|value|value*100.0),sma_long_slope_pct:long_slope,
        return_1_pct:v2.returns.return_1.map(|value|value*100.0),return_5_pct:v2.returns.return_5.map(|value|value*100.0),return_10_pct:v2.returns.return_10.map(|value|value*100.0),return_20_pct:v2.returns.return_20.map(|value|value*100.0),rolling_volatility:v2.volatility.rolling_volatility,
    };
    let signals=market_signals::evaluate(&signal_input,&Default::default())?;
    Ok(market_ai::MarketAiSnapshotV3{market:v2.market,regime:v2.regime,portfolio:v2.portfolio,key_features:market_ai::MarketAiKeyFeatures{return_5:v2.returns.return_5,return_20:v2.returns.return_20,price_vs_sma_short_pct:v2.trend.price_vs_sma_short_pct,sma_spread_pct:v2.trend.sma_spread_pct},signals,risk_context:v2.risk_context})
}

pub(crate) fn validate_input(connection:&Connection,input:&MarketExperimentInput)->Result<(MarketDataset,MarketRiskProfile,Vec<MarketAgentDefinition>,Vec<Candle>),String>{if input.agent_ids.len()<3{return Err("uma coorte exige entre 3 e 5 agentes".into())}if input.agent_ids.len()>MAX_CONCURRENT_AGENTS{return Err(format!("EXPERIMENT REJECTED — requested agents: {}; runtime limit: {}",input.agent_ids.len(),MAX_CONCURRENT_AGENTS))}let unique:HashSet<_>=input.agent_ids.iter().collect();if unique.len()!=input.agent_ids.len(){return Err("agentes duplicados".into())}if input.agent_ids.iter().filter(|id|market_ai::is_ai_agent(id)).count()>market_ai::MAX_AI_AGENTS{return Err("a v0.4.1 permite no máximo um AI Agent por coorte; compare V1 e V2 em experimentos separados".into())}if !input.initial_capital.is_finite()||input.initial_capital<=0.0{return Err("capital inicial inválido".into())}if input.fee_pct<0.0||input.slippage_pct<0.0||input.fee_pct>10.0||input.slippage_pct>10.0{return Err("fees/slippage devem estar entre 0 e 10%".into())}let dataset=get_dataset(connection,&input.dataset_id)?;let profile=list_risk_profiles(connection)?.into_iter().find(|p|p.id==input.risk_profile_id).ok_or("perfil de risco não encontrado")?;let registry=list_agents(connection)?;let agents=input.agent_ids.iter().map(|id|registry.iter().find(|a|a.id==*id&&a.enabled).cloned().ok_or_else(||format!("agente inválido ou desabilitado: {id}"))).collect::<Result<Vec<_>,_>>()?;let candles=load_candles(connection,&dataset.id)?;if candles.len()<2{return Err("dataset precisa de ao menos 2 candles".into())}Ok((dataset,profile,agents,candles))}

pub(crate) fn resolve_ai_config(connection:&Connection,input:&MarketExperimentInput)->Result<(market_ai::MarketAiConfig,String),String>{let settings=ai_repository::get_settings(connection)?;let agent_id=input.agent_ids.iter().find(|id|market_ai::is_ai_agent(id)).cloned().unwrap_or_else(||market_ai::AI_AGENT_V2_ID.into());let(provider,default_model,prompt_version,temperature,decision_interval,default_timeout,max_retries):(String,String,String,f64,usize,u64,usize)=connection.query_row("SELECT provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries FROM market_ai_agent_configs WHERE agent_id=?1",[&agent_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?,row.get(4)?,row.get(5)?,row.get(6)?))).map_err(|error|error.to_string())?;let config=market_ai::MarketAiConfig{agent_id,provider,model:if settings.model.trim().is_empty(){default_model}else{settings.model},prompt_version,temperature,decision_interval,timeout_ms:default_timeout.min((settings.timeout_seconds as u64)*1000),max_retries};config.validate()?;Ok((config,settings.endpoint))}

pub(crate) fn simulate_range_with_provider(dataset: &MarketDataset, profile: &MarketRiskProfile, agents: &[MarketAgentDefinition], candles: &[Candle], start: usize, end: usize, input: &MarketExperimentInput, provider:&dyn market_ai::MarketAiProvider, ai_config:&market_ai::MarketAiConfig) -> Result<Vec<AgentRun>, String> {
    if start > end || end >= candles.len() { return Err("intervalo de replay inválido".into()); }
    let mut runs: Vec<AgentRun> = agents
        .iter()
        .map(|agent| AgentRun {
            id: agent.id.clone(), name: agent.name.clone(), status: "completed".into(),
            portfolio: Portfolio { cash: round(input.initial_capital), quantity: 0.0, average_entry: 0.0, realized: 0.0, peak_equity: round(input.initial_capital), trades: 0, wins: 0, gross_profit: 0.0, gross_loss: 0.0 },
            pending: None, position_started_index: None, decisions: Vec::new(), snapshots: Vec::new(), max_drawdown: 0.0, exposure_sum: 0.0, max_exposure: 0.0, hold_count: 0, ai_runtime: Default::default(),
        })
        .collect();
    for index in start..=end {
        let candle = &candles[index];
        let features = feature_at(candles, index);
        for run in &mut runs {
            if run.status != "completed" { continue; }
            if let Some(pending) = run.pending.take() {
                let had_position = run.portfolio.quantity > 0.0;
                if let Some(execution) = execute(&mut run.portfolio, &pending, candle, input.fee_pct, input.slippage_pct) {
                    if let Some(row) = run.decisions.get_mut(pending.decision_index) { row.execution = Some(execution); }
                }
                let has_position = run.portfolio.quantity > 0.0;
                if !had_position && has_position { run.position_started_index = Some(index); }
                if had_position && !has_position { run.position_started_index = None; }
            }
            let before = equity(&run.portfolio, candle.close);
            run.portfolio.peak_equity = run.portfolio.peak_equity.max(before);
            let exp = exposure(&run.portfolio, candle.close);
            let ai = if market_ai::is_ai_agent(&run.id) {
                let outcome = if (index - start) % ai_config.decision_interval == 0 {
                    if ai_config.prompt_version == market_ai::PROMPT_V3_VERSION {
                        let snapshot = build_v3_snapshot(dataset, profile, candles, index, run, before, exp)?;
                        market_ai::MarketAIAgent::decide(provider, &snapshot, ai_config)
                    } else if ai_config.prompt_version == market_ai::PROMPT_V2_VERSION {
                        let snapshot = build_v2_snapshot(dataset, profile, candles, index, run, before, exp);
                        market_ai::MarketAIAgent::decide(provider, &snapshot, ai_config)
                    } else {
                        let snapshot = market_ai::MarketAiSnapshot {
                            timestamp: candle.timestamp.clone(), asset: dataset.asset.clone(), timeframe: dataset.timeframe.clone(), price: candle.close,
                            features: market_ai::MarketAiFeatures { return_1: features.returns, return_5: features.momentum, sma_short: features.sma_short, sma_long: features.sma_long, volatility: features.volatility },
                            portfolio: market_ai::MarketAiPortfolio { cash: run.portfolio.cash, equity: before, exposure_pct: exp, current_position: run.portfolio.quantity > 0.0, entry_price: (run.portfolio.quantity > 0.0).then_some(run.portfolio.average_entry) },
                            risk_context: market_ai::MarketAiRiskContext { max_position_pct: profile.max_position_pct, max_total_exposure_pct: profile.max_total_exposure_pct, allow_short: profile.allow_short, allow_leverage: profile.allow_leverage },
                            memory: market_ai::MarketAiMemory { last_action: run.decisions.last().map(|row| row.decision.action.into()), recent_decision_count: run.decisions.len().min(20) },
                        };
                        market_ai::MarketAIAgent::decide(provider, &snapshot, ai_config)
                    }
                } else {
                    market_ai::MarketAIAgent::no_call(ai_config)
                };
                run.ai_runtime.merge(&outcome.runtime);
                Some(outcome)
            } else { None };
            let decision = if let Some(value) = &ai {
                Decision { action: value.action, desired_pct: value.desired_position_pct, confidence: value.confidence, reasoning: value.reason.clone() }
            } else { decide(&run.id, index, &features, run.portfolio.quantity > 0.0, input.random_seed) };
            if decision.action == "HOLD" { run.hold_count += 1; }
            let invalid_position=market_ai::is_ai_agent(&run.id) && decision.action == "SELL" && run.portfolio.quantity <= 0.0 && !profile.allow_short;
            let (result, reason, approved) = if invalid_position {
                ("REJECTED".into(), "SELL sem posição rejeitado; short está desabilitado".into(), None)
            } else {
                risk(&decision, &run.portfolio, candle.close, profile, &dataset.asset, false)
            };
            let decision_index = run.decisions.len();
            run.decisions.push(DecisionRow { candle_index: index, timestamp: candle.timestamp.clone(), observed_price: candle.close, decision: decision.clone(), risk_result: result.clone(), risk_reason: reason, validation_code:invalid_position.then(||"INVALID_POSITION_ACTION".into()), approved_pct: approved, cash_before: run.portfolio.cash, equity_before: before, exposure_before: exp, execution: None, ai });
            if index < end && decision.action != "HOLD" && result != "REJECTED" { run.pending = Some(PendingOrder { decision_index, action: decision.action, approved_pct: approved.unwrap_or(exp) }); }
            let market = round(run.portfolio.quantity * candle.close); let eq = round(run.portfolio.cash + market); let unrealized = round((candle.close - run.portfolio.average_entry) * run.portfolio.quantity); let exposure_pct = if eq > 0.0 { round(market / eq * 100.0) } else { 0.0 }; let dd = if run.portfolio.peak_equity > 0.0 { round((run.portfolio.peak_equity - eq) / run.portfolio.peak_equity * 100.0) } else { 0.0 };
            run.max_drawdown = run.max_drawdown.max(dd); run.exposure_sum += exposure_pct; run.max_exposure = run.max_exposure.max(exposure_pct);
            run.snapshots.push(SnapshotRow { candle_index: index, timestamp: candle.timestamp.clone(), cash: run.portfolio.cash, quantity: run.portfolio.quantity, market_value: market, equity: eq, exposure: exposure_pct, realized: run.portfolio.realized, unrealized });
        }
    }
    Ok(runs)
}

pub fn run_experiment(connection:&mut Connection,input:&MarketExperimentInput)->Result<MarketExperimentResult,String>{let(ai_config,endpoint)=resolve_ai_config(connection,input)?;if input.agent_ids.iter().any(|id|market_ai::is_ai_agent(id)){let health=tauri::async_runtime::block_on(crate::ollama::status(&endpoint,(ai_config.timeout_ms/1000).clamp(5,8)))?;if health.available&&health.models.iter().any(|model|model==&ai_config.model){let provider=market_ai::OllamaMarketAiProvider::new(endpoint);run_experiment_with_provider(connection,input,&provider,&ai_config)}else{run_experiment_with_provider(connection,input,&market_ai::UnavailableMarketAiProvider,&ai_config)}}else{run_experiment_with_provider(connection,input,&market_ai::UnavailableMarketAiProvider,&ai_config)}}

pub(crate) fn run_experiment_with_provider(connection:&mut Connection,input:&MarketExperimentInput,provider:&dyn market_ai::MarketAiProvider,ai_config:&market_ai::MarketAiConfig)->Result<MarketExperimentResult,String>{let(dataset,profile,agents,candles)=validate_input(connection,input)?;let id=new_id("experiment");let signal_config=market_signals::SignalEngineConfig::default();let config=json!({"datasetFingerprint":dataset.fingerprint,"agentIds":input.agent_ids,"riskProfile":profile,"initialCapital":round(input.initial_capital),"randomSeed":input.random_seed,"feePct":input.fee_pct,"slippagePct":input.slippage_pct,"executionTiming":"decision-close-T/execution-open-T+1","precisionScale":6,"maxConcurrentAgents":MAX_CONCURRENT_AGENTS,"aiConfig":ai_config,"signalEngineVersion":(ai_config.prompt_version==market_ai::PROMPT_V3_VERSION).then_some(market_signals::SIGNAL_ENGINE_VERSION),"signalConfig":(ai_config.prompt_version==market_ai::PROMPT_V3_VERSION).then_some(signal_config)});connection.execute("INSERT INTO market_experiments(id,name,dataset_id,asset,timeframe,currency,initial_capital,risk_profile_id,random_seed,fee_pct,slippage_pct,config_json,status,started_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,'running',CURRENT_TIMESTAMP)",params![id,input.name.trim(),dataset.id,dataset.asset,dataset.timeframe,dataset.currency,round(input.initial_capital),profile.id,input.random_seed as i64,input.fee_pct,input.slippage_pct,config.to_string()]).map_err(|e|e.to_string())?;
    let runs=simulate_range_with_provider(&dataset,&profile,&agents,&candles,0,candles.len()-1,input,provider,ai_config)?;
    let tx=connection.transaction().map_err(|e|e.to_string())?;persist_run_with_ai_diagnostics(&tx,&id,&agents,&runs,input.initial_capital,&candles,ai_config)?;market_observatory::persist(&tx,&id)?;tx.execute("UPDATE market_experiments SET status='completed',completed_at=CURRENT_TIMESTAMP WHERE id=?1",[&id]).map_err(|e|e.to_string())?;tx.commit().map_err(|e|e.to_string())?;get_experiment(connection,&id)
}

fn persist_run(tx:&Transaction<'_>,experiment_id:&str,agents:&[MarketAgentDefinition],runs:&[AgentRun],initial:f64)->Result<(),String>{for(run,agent)in runs.iter().zip(agents){tx.execute("INSERT INTO market_experiment_agents(experiment_id,agent_id,config_json,status) VALUES(?1,?2,?3,?4)",params![experiment_id,agent.id,agent.default_config_json,run.status]).map_err(|e|e.to_string())?;for d in &run.decisions{tx.execute("INSERT INTO market_decisions(experiment_id,agent_id,candle_index,timestamp,observed_price,action,desired_position_pct,confidence,reasoning,cash_before,equity_before,exposure_before) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",params![experiment_id,run.id,d.candle_index,d.timestamp,d.observed_price,d.decision.action,d.decision.desired_pct,d.decision.confidence,d.decision.reasoning,d.cash_before,d.equity_before,d.exposure_before]).map_err(|e|e.to_string())?;let decision_id=tx.last_insert_rowid();if let Some(ai)=&d.ai{tx.execute("INSERT INTO market_ai_decisions(decision_id,call_status,provider,model,prompt_version,latency_ms,attempts,fallback_used,reason_code,validation_code) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",params![decision_id,ai.call_status,ai.provider,ai.model,ai.prompt_version,ai.latency_ms,ai.attempts,ai.fallback_used,ai.reason_code,d.validation_code]).map_err(|e|e.to_string())?;}tx.execute("INSERT INTO market_risk_evaluations(decision_id,result,reason,requested_position_pct,approved_position_pct,risk_state_json) VALUES(?1,?2,?3,?4,?5,?6)",params![decision_id,d.risk_result,d.risk_reason,d.decision.desired_pct,d.approved_pct,"{}"] ).map_err(|e|e.to_string())?;if let Some(ex)=&d.execution{tx.execute("INSERT INTO market_orders(decision_id,agent_id,asset,side,quantity,requested_price,status) SELECT ?1,?2,asset,?3,?4,?5,'filled' FROM market_experiments WHERE id=?6",params![decision_id,run.id,ex.side,ex.quantity,ex.requested_price,experiment_id]).map_err(|e|e.to_string())?;let order_id=tx.last_insert_rowid();tx.execute("INSERT INTO market_executions(order_id,timestamp,execution_price,quantity,gross_value,fees,slippage,net_value) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",params![order_id,ex.timestamp,ex.price,ex.quantity,ex.gross,ex.fees,ex.slippage,ex.net]).map_err(|e|e.to_string())?;}}for s in &run.snapshots{tx.execute("INSERT INTO market_portfolio_snapshots(experiment_id,agent_id,candle_index,timestamp,cash,quantity,market_value,equity,exposure_pct,realized_pnl,unrealized_pnl) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",params![experiment_id,run.id,s.candle_index,s.timestamp,s.cash,s.quantity,s.market_value,s.equity,s.exposure,s.realized,s.unrealized]).map_err(|e|e.to_string())?;}let last=run.snapshots.last().ok_or("snapshot ausente")?;let profit_factor=if run.portfolio.gross_loss>0.0{Some(round(run.portfolio.gross_profit/run.portfolio.gross_loss))}else{None};tx.execute("INSERT INTO market_agent_metrics(experiment_id,agent_id,final_equity,total_return_pct,max_drawdown_pct,decision_count,hold_count,trade_count,win_rate_pct,profit_factor,realized_pnl,unrealized_pnl,average_exposure_pct,max_exposure_pct) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",params![experiment_id,run.id,last.equity,round((last.equity-initial)/initial*100.0),run.max_drawdown,run.decisions.len(),run.hold_count,run.portfolio.trades,if run.portfolio.trades>0{round(run.portfolio.wins as f64/run.portfolio.trades as f64*100.0)}else{0.0},profit_factor,run.portfolio.realized,last.unrealized,round(run.exposure_sum/run.snapshots.len() as f64),run.max_exposure]).map_err(|e|e.to_string())?;if market_ai::is_ai_agent(&run.id){let m=&run.ai_runtime;tx.execute("INSERT INTO market_ai_runtime_metrics(experiment_id,agent_id,call_count,successful_call_count,invalid_response_count,timeout_count,retry_count,fallback_count,average_latency_ms,max_latency_ms,total_latency_ms,input_tokens,output_tokens) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13)",params![experiment_id,run.id,m.call_count,m.successful_call_count,m.invalid_response_count,m.timeout_count,m.retry_count,m.fallback_count,m.average_latency_ms,m.max_latency_ms,m.total_latency_ms,m.input_tokens,m.output_tokens]).map_err(|e|e.to_string())?;}}Ok(())}

fn forward_return(candles: &[Candle], candle_index: usize, horizon: usize) -> Option<f64> {
    candles.get(candle_index + horizon).map(|future| {
        round((future.close / candles[candle_index].close - 1.0) * 100.0)
    })
}

fn persist_run_with_ai_diagnostics(
    tx: &Transaction<'_>, experiment_id: &str, agents: &[MarketAgentDefinition], runs: &[AgentRun],
    initial: f64, candles: &[Candle], ai_config: &market_ai::MarketAiConfig,
) -> Result<(), String> {
    persist_run(tx, experiment_id, agents, runs, initial)?;
    for run in runs.iter().filter(|run| market_ai::is_ai_agent(&run.id)) {
        let metrics = &run.ai_runtime;
        let distribution = metrics.confidence_distribution();
        tx.execute(
            "UPDATE market_ai_runtime_metrics SET prompt_version=?1,buy_count=?2,sell_count=?3,llm_hold_count=?4,no_llm_call_count=?5,average_confidence=?6,average_buy_confidence=?7,average_sell_confidence=?8,average_hold_confidence=?9,min_confidence=?10,max_confidence=?11,median_confidence=?12,confidence_00_20=?13,confidence_20_40=?14,confidence_40_60=?15,confidence_60_80=?16,confidence_80_100=?17 WHERE experiment_id=?18 AND agent_id=?19",
            params![ai_config.prompt_version,metrics.buy_count,metrics.sell_count,metrics.llm_hold_count,metrics.no_llm_call_count,metrics.average_confidence(),metrics.average_buy_confidence(),metrics.average_sell_confidence(),metrics.average_hold_confidence(),metrics.min_confidence(),metrics.max_confidence(),metrics.median_confidence(),distribution[0],distribution[1],distribution[2],distribution[3],distribution[4],experiment_id,run.id],
        ).map_err(|error| error.to_string())?;
        for decision in &run.decisions {
            let Some(ai) = &decision.ai else { continue };
            let decision_id: i64 = tx.query_row(
                "SELECT id FROM market_decisions WHERE experiment_id=?1 AND agent_id=?2 AND candle_index=?3",
                params![experiment_id,run.id,decision.candle_index], |row| row.get(0),
            ).map_err(|error| error.to_string())?;
            if let Some(snapshot) = &ai.input_snapshot_json {
                tx.execute("INSERT INTO market_ai_decision_context(decision_id,input_snapshot_json) VALUES(?1,?2)",params![decision_id,snapshot]).map_err(|error|error.to_string())?;
                if ai.prompt_version==market_ai::PROMPT_V3_VERSION {
                    let parsed:serde_json::Value=serde_json::from_str(snapshot).map_err(|error|error.to_string())?;
                    let signals=&parsed["signals"];
                    let bias=signals["directionalBias"].as_str().unwrap_or("NEUTRAL");let strength=signals["signalStrength"].as_f64().unwrap_or(0.0);
                    let position=parsed.pointer("/portfolio/current_position").and_then(|value|value.as_bool()).unwrap_or(false);
                    let disagreement=ai.call_status=="VALID"&&((bias=="BULLISH"&&strength>=0.65&&ai.action!="BUY")||(bias=="BEARISH"&&strength>=0.65&&position&&ai.action!="SELL"));
                    tx.execute("INSERT INTO market_ai_signal_traces(decision_id,signal_engine_version,signal_config_version,signal_config_json,raw_features_json,trend_score,momentum_score,volatility_score,bullish_evidence,bearish_evidence,signal_strength,directional_bias,conflict_level,disagreement) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)",params![decision_id,signals["engineVersion"].as_str().unwrap_or(market_signals::SIGNAL_ENGINE_VERSION),signals["configVersion"].as_str().unwrap_or(market_signals::SIGNAL_CONFIG_VERSION),serde_json::to_string(&market_signals::SignalEngineConfig::default()).map_err(|error|error.to_string())?,parsed["key_features"].to_string(),signals["trendScore"].as_f64().unwrap_or(0.0),signals["momentumScore"].as_f64().unwrap_or(0.0),signals["volatilityScore"].as_f64().unwrap_or(0.0),signals["bullishEvidence"].as_f64().unwrap_or(0.0),signals["bearishEvidence"].as_f64().unwrap_or(0.0),strength,bias,signals["conflictLevel"].as_str().unwrap_or("LOW"),disagreement]).map_err(|error|error.to_string())?;
                }
            }
            if ai.call_status == "VALID" {
                tx.execute(
                    "INSERT INTO market_ai_decision_evaluations(decision_id,forward_return_1,forward_return_5,forward_return_10) VALUES(?1,?2,?3,?4)",
                    params![decision_id,forward_return(candles,decision.candle_index,1),forward_return(candles,decision.candle_index,5),forward_return(candles,decision.candle_index,10)],
                ).map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

fn summary_query(connection:&Connection,where_clause:&str,arg:Option<&str>)->Result<Vec<MarketExperimentSummary>,String>{let sql=format!("SELECT e.id,e.name,e.dataset_id,d.name,e.asset,e.timeframe,e.currency,e.initial_capital,e.risk_profile_id,e.random_seed,e.fee_pct,e.slippage_pct,e.status,e.error,e.started_at,e.completed_at,e.created_at FROM market_experiments e JOIN market_datasets d ON d.id=e.dataset_id {where_clause} ORDER BY e.created_at DESC");let mut s=connection.prepare(&sql).map_err(|e|e.to_string())?;let mapper=|r:&rusqlite::Row<'_>|->rusqlite::Result<MarketExperimentSummary>{let id:String=r.get(0)?;let mut a=connection.prepare("SELECT agent_id FROM market_experiment_agents WHERE experiment_id=?1 ORDER BY rowid")?;let ids=a.query_map([&id],|x|x.get(0))?.collect::<Result<Vec<String>,_>>()?;Ok(MarketExperimentSummary{id,name:r.get(1)?,dataset_id:r.get(2)?,dataset_name:r.get(3)?,asset:r.get(4)?,timeframe:r.get(5)?,currency:r.get(6)?,initial_capital:r.get(7)?,risk_profile_id:r.get(8)?,random_seed:r.get::<_,i64>(9)? as u64,fee_pct:r.get(10)?,slippage_pct:r.get(11)?,agent_ids:ids,status:r.get(12)?,error:r.get(13)?,started_at:r.get(14)?,completed_at:r.get(15)?,created_at:r.get(16)?})};if let Some(v)=arg{s.query_map([v],mapper).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())}else{s.query_map([],mapper).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())}}
pub fn list_experiments(connection:&Connection)->Result<Vec<MarketExperimentSummary>,String>{summary_query(connection,"",None)}
pub fn get_experiment(connection:&Connection,id:&str)->Result<MarketExperimentResult,String>{let experiment=summary_query(connection,"WHERE e.id=?1",Some(id))?.into_iter().next().ok_or("experimento não encontrado")?;let names:HashMap<String,String>=list_agents(connection)?.into_iter().map(|a|(a.id,a.name)).collect();let mut s=connection.prepare("SELECT m.agent_id,a.status,m.final_equity,m.total_return_pct,m.max_drawdown_pct,m.decision_count,m.hold_count,m.trade_count,m.win_rate_pct,m.profit_factor,m.realized_pnl,m.unrealized_pnl,m.average_exposure_pct,m.max_exposure_pct FROM market_agent_metrics m JOIN market_experiment_agents a ON a.experiment_id=m.experiment_id AND a.agent_id=m.agent_id WHERE m.experiment_id=?1 ORDER BY m.total_return_pct DESC").map_err(|e|e.to_string())?;let metrics=s.query_map([id],|r|{let aid:String=r.get(0)?;Ok(MarketAgentMetric{agent_name:names.get(&aid).cloned().unwrap_or_else(||aid.clone()),agent_id:aid,status:r.get(1)?,final_equity:r.get(2)?,total_return_pct:r.get(3)?,max_drawdown_pct:r.get(4)?,decision_count:r.get(5)?,hold_count:r.get(6)?,trade_count:r.get(7)?,win_rate_pct:r.get(8)?,profit_factor:r.get(9)?,realized_pnl:r.get(10)?,unrealized_pnl:r.get(11)?,average_exposure_pct:r.get(12)?,max_exposure_pct:r.get(13)?})}).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;let mut q=connection.prepare("SELECT timestamp,agent_id,equity,exposure_pct FROM market_portfolio_snapshots WHERE experiment_id=?1 ORDER BY candle_index,agent_id").map_err(|e|e.to_string())?;let equity=q.query_map([id],|r|Ok(MarketEquityPoint{timestamp:r.get(0)?,agent_id:r.get(1)?,equity:r.get(2)?,exposure_pct:r.get(3)?})).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;let mut q=connection.prepare("SELECT d.id,d.timestamp,d.agent_id,d.action,d.observed_price,d.desired_position_pct,d.reasoning,r.result,r.reason,r.approved_position_pct,x.execution_price,x.quantity,x.fees FROM market_decisions d JOIN market_risk_evaluations r ON r.decision_id=d.id LEFT JOIN market_orders o ON o.decision_id=d.id LEFT JOIN market_executions x ON x.order_id=o.id WHERE d.experiment_id=?1 ORDER BY d.candle_index DESC,d.agent_id LIMIT 1000").map_err(|e|e.to_string())?;let decisions=q.query_map([id],|r|Ok(MarketDecisionLog{id:r.get(0)?,timestamp:r.get(1)?,agent_id:r.get(2)?,action:r.get(3)?,observed_price:r.get(4)?,desired_position_pct:r.get(5)?,reasoning:r.get(6)?,risk_result:r.get(7)?,risk_reason:r.get(8)?,approved_position_pct:r.get(9)?,execution_price:r.get(10)?,quantity:r.get(11)?,fees:r.get(12)?})).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;let observatory=market_observatory::load(connection,id)?;Ok(MarketExperimentResult{experiment,metrics,equity,decisions,observatory})}
pub fn rerun(connection:&mut Connection,id:&str)->Result<MarketExperimentResult,String>{let old=get_experiment(connection,id)?.experiment;run_experiment(connection,&MarketExperimentInput{name:format!("{} / RERUN",old.name),dataset_id:old.dataset_id,risk_profile_id:old.risk_profile_id,agent_ids:old.agent_ids,initial_capital:old.initial_capital,random_seed:old.random_seed,fee_pct:old.fee_pct,slippage_pct:old.slippage_pct})}

pub fn update_ai_config(connection:&Connection,input:&UpdateMarketAiConfigInput)->Result<(),String>{if !market_ai::is_ai_agent(&input.agent_id)||!(1..=100).contains(&input.decision_interval)||!(5_000..=180_000).contains(&input.timeout_ms)||input.max_retries>1{return Err("configuração de runtime do AI Agent inválida".into())}let updated=connection.execute("UPDATE market_ai_agent_configs SET decision_interval=?1,timeout_ms=?2,max_retries=?3,updated_at=CURRENT_TIMESTAMP WHERE agent_id=?4",params![input.decision_interval,input.timeout_ms,input.max_retries,input.agent_id]).map_err(|error|error.to_string())?;if updated!=1{return Err("AI Agent não encontrado".into())}Ok(())}

fn map_ai_runtime(row: &rusqlite::Row<'_>) -> rusqlite::Result<MarketAiRuntimeMetric> {
    Ok(MarketAiRuntimeMetric {
        agent_id: row.get(0)?, prompt_version: row.get(1)?, call_count: row.get(2)?, successful_call_count: row.get(3)?, invalid_response_count: row.get(4)?, timeout_count: row.get(5)?, retry_count: row.get(6)?, fallback_count: row.get(7)?, average_latency_ms: row.get(8)?, max_latency_ms: row.get(9)?, total_latency_ms: row.get(10)?, input_tokens: row.get(11)?, output_tokens: row.get(12)?, buy_count: row.get(13)?, sell_count: row.get(14)?, llm_hold_count: row.get(15)?, no_llm_call_count: row.get(16)?, average_confidence: row.get(17)?, average_buy_confidence: row.get(18)?, average_sell_confidence: row.get(19)?, average_hold_confidence: row.get(20)?, min_confidence: row.get(21)?, max_confidence: row.get(22)?, median_confidence: row.get(23)?, confidence_distribution: vec![row.get(24)?,row.get(25)?,row.get(26)?,row.get(27)?,row.get(28)?],
    })
}

const AI_RUNTIME_SELECT: &str = "agent_id,prompt_version,call_count,successful_call_count,invalid_response_count,timeout_count,retry_count,fallback_count,average_latency_ms,max_latency_ms,total_latency_ms,input_tokens,output_tokens,buy_count,sell_count,llm_hold_count,no_llm_call_count,average_confidence,average_buy_confidence,average_sell_confidence,average_hold_confidence,min_confidence,max_confidence,median_confidence,confidence_00_20,confidence_20_40,confidence_40_60,confidence_60_80,confidence_80_100";

pub fn list_ai_runtime(connection:&Connection,experiment_id:&str)->Result<Vec<MarketAiRuntimeMetric>,String>{let mut statement=connection.prepare(&format!("SELECT {AI_RUNTIME_SELECT} FROM market_ai_runtime_metrics WHERE experiment_id=?1")).map_err(|error|error.to_string())?;let rows=statement.query_map([experiment_id],map_ai_runtime).map_err(|error|error.to_string())?;rows.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())}

pub fn list_validation_ai_runtime(connection:&Connection,validation_id:&str)->Result<Vec<MarketAiRuntimeMetric>,String>{let mut statement=connection.prepare(&format!("SELECT {AI_RUNTIME_SELECT} FROM market_validation_ai_runtime_metrics WHERE validation_run_id=?1")).map_err(|error|error.to_string())?;let rows=statement.query_map([validation_id],map_ai_runtime).map_err(|error|error.to_string())?;rows.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())}

pub fn list_ai_decisions(connection:&Connection,experiment_id:&str)->Result<Vec<MarketAiDecisionLog>,String>{let mut statement=connection.prepare("SELECT d.id,d.timestamp,d.action,d.desired_position_pct,d.confidence,d.reasoning,r.result,r.reason,r.approved_position_pct,x.execution_price,a.call_status,a.provider,a.model,a.prompt_version,a.latency_ms,a.attempts,a.fallback_used,c.input_snapshot_json,v.forward_return_1,v.forward_return_5,v.forward_return_10,a.reason_code,a.validation_code,COALESCE(s.disagreement,0) FROM market_decisions d JOIN market_risk_evaluations r ON r.decision_id=d.id JOIN market_ai_decisions a ON a.decision_id=d.id LEFT JOIN market_orders o ON o.decision_id=d.id LEFT JOIN market_executions x ON x.order_id=o.id LEFT JOIN market_ai_decision_context c ON c.decision_id=d.id LEFT JOIN market_ai_decision_evaluations v ON v.decision_id=d.id LEFT JOIN market_ai_signal_traces s ON s.decision_id=d.id WHERE d.experiment_id=?1 ORDER BY d.candle_index DESC LIMIT 1000").map_err(|error|error.to_string())?;let rows=statement.query_map([experiment_id],|row|{let raw:Option<String>=row.get(17)?;Ok(MarketAiDecisionLog{decision_id:row.get(0)?,timestamp:row.get(1)?,action:row.get(2)?,desired_position_pct:row.get(3)?,confidence:row.get(4)?,reason:row.get(5)?,risk_result:row.get(6)?,risk_reason:row.get(7)?,approved_position_pct:row.get(8)?,execution_price:row.get(9)?,call_status:row.get(10)?,provider:row.get(11)?,model:row.get(12)?,prompt_version:row.get(13)?,latency_ms:row.get(14)?,attempts:row.get(15)?,fallback_used:row.get(16)?,input_snapshot:raw.and_then(|value|serde_json::from_str(&value).ok()),forward_return_1:row.get(18)?,forward_return_5:row.get(19)?,forward_return_10:row.get(20)?,reason_code:row.get(21)?,validation_code:row.get(22)?,signal_disagreement:row.get(23)?})}).map_err(|error|error.to_string())?;rows.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())}

pub fn list_ai_experiment_comparisons(connection:&Connection)->Result<Vec<MarketAiExperimentComparison>,String>{
    let mut statement=connection.prepare("SELECT e.id,e.name,e.dataset_id,d.name,e.risk_profile_id,e.initial_capital,e.random_seed,e.fee_pct,e.slippage_pct,e.config_json,r.agent_id,r.prompt_version,r.call_count,r.successful_call_count,r.invalid_response_count,r.timeout_count,r.buy_count,r.sell_count,r.llm_hold_count,m.trade_count,m.total_return_pct,m.max_drawdown_pct,m.average_exposure_pct,r.average_confidence,r.average_latency_ms FROM market_experiments e JOIN market_datasets d ON d.id=e.dataset_id JOIN market_ai_runtime_metrics r ON r.experiment_id=e.id JOIN market_agent_metrics m ON m.experiment_id=e.id AND m.agent_id=r.agent_id WHERE e.status='completed' ORDER BY e.created_at DESC").map_err(|error|error.to_string())?;
    let rows=statement.query_map([],|row|{let config:String=row.get(9)?;let parsed:serde_json::Value=serde_json::from_str(&config).unwrap_or_default();let calls:usize=row.get(12)?;let llm_hold:usize=row.get(18)?;Ok(MarketAiExperimentComparison{experiment_id:row.get(0)?,experiment_name:row.get(1)?,dataset_id:row.get(2)?,dataset_name:row.get(3)?,risk_profile_id:row.get(4)?,initial_capital:row.get(5)?,random_seed:row.get::<_,i64>(6)? as u64,fee_pct:row.get(7)?,slippage_pct:row.get(8)?,decision_interval:parsed.pointer("/aiConfig/decisionInterval").and_then(|value|value.as_u64()).unwrap_or(5) as usize,agent_id:row.get(10)?,prompt_version:row.get(11)?,call_count:calls,successful_call_count:row.get(13)?,invalid_response_count:row.get(14)?,timeout_count:row.get(15)?,buy_count:row.get(16)?,sell_count:row.get(17)?,llm_hold_count:llm_hold,trade_count:row.get(19)?,hold_rate_pct:if calls==0{0.0}else{llm_hold as f64/calls as f64*100.0},total_return_pct:row.get(20)?,max_drawdown_pct:row.get(21)?,average_exposure_pct:row.get(22)?,average_confidence:row.get(23)?,average_latency_ms:row.get(24)?})}).map_err(|error|error.to_string())?;
    rows.collect::<Result<Vec<_>,_>>().map_err(|error|error.to_string())
}

pub fn get_signal_diagnostics(connection:&Connection,experiment_id:&str)->Result<Option<MarketSignalDiagnostics>,String>{
    let decisions=list_ai_decisions(connection,experiment_id)?.into_iter().filter(|item|item.prompt_version==market_ai::PROMPT_V3_VERSION&&item.call_status=="VALID").collect::<Vec<_>>();
    if decisions.is_empty(){return Ok(None)}
    let mut signal_rows=vec!["STRONG BULLISH","WEAK BULLISH","NEUTRAL","WEAK BEARISH","STRONG BEARISH"].into_iter().map(|label|MarketSignalMatrixRow{label:label.into(),buy_count:0,sell_count:0,hold_count:0,average_forward_5:None}).collect::<Vec<_>>();
    let mut conflict_rows=vec!["LOW","MEDIUM","HIGH"].into_iter().map(|label|MarketSignalMatrixRow{label:label.into(),buy_count:0,sell_count:0,hold_count:0,average_forward_5:None}).collect::<Vec<_>>();
    let mut signal_forwards:Vec<Vec<f64>>=vec![vec![];5];let mut conflict_forwards:Vec<Vec<f64>>=vec![vec![];3];let mut reasons:HashMap<String,usize>=HashMap::new();
    let mut bullish=0;let mut bearish=0;let mut neutral=0;let mut strong=0;let mut weak=0;let mut low=0;let mut medium=0;let mut high=0;let mut disagreements=0;
    for decision in &decisions { let Some(snapshot)=&decision.input_snapshot else{continue};let signals=&snapshot["signals"];let bias=signals["directionalBias"].as_str().unwrap_or("NEUTRAL");let strength=signals["signalStrength"].as_f64().unwrap_or(0.0);let conflict=signals["conflictLevel"].as_str().unwrap_or("LOW");
        match bias{"BULLISH"=>bullish+=1,"BEARISH"=>bearish+=1,_=>neutral+=1};if strength>=0.65{strong+=1}else{weak+=1};match conflict{"HIGH"=>high+=1,"MEDIUM"=>medium+=1,_=>low+=1};if decision.signal_disagreement{disagreements+=1};if let Some(code)=&decision.reason_code{*reasons.entry(code.clone()).or_default()+=1}
        let signal_index=match(bias,strength>=0.65){("BULLISH",true)=>0,("BULLISH",false)=>1,("BEARISH",false)=>3,("BEARISH",true)=>4,_=>2};let conflict_index=match conflict{"MEDIUM"=>1,"HIGH"=>2,_=>0};
        for row in [&mut signal_rows[signal_index],&mut conflict_rows[conflict_index]]{match decision.action.as_str(){"BUY"=>row.buy_count+=1,"SELL"=>row.sell_count+=1,_=>row.hold_count+=1}}
        if let Some(value)=decision.forward_return_5{signal_forwards[signal_index].push(value);conflict_forwards[conflict_index].push(value)}
    }
    for(index,row)in signal_rows.iter_mut().enumerate(){if !signal_forwards[index].is_empty(){row.average_forward_5=Some(signal_forwards[index].iter().sum::<f64>()/signal_forwards[index].len() as f64)}}
    for(index,row)in conflict_rows.iter_mut().enumerate(){if !conflict_forwards[index].is_empty(){row.average_forward_5=Some(conflict_forwards[index].iter().sum::<f64>()/conflict_forwards[index].len() as f64)}}
    let rate=|numerator:usize,denominator:usize|if denominator==0{None}else{Some(numerator as f64/denominator as f64*100.0)};let sb=&signal_rows[0];let sbd=sb.buy_count+sb.sell_count+sb.hold_count;let br=&signal_rows[4];let brd=br.buy_count+br.sell_count+br.hold_count;
    let collapse=market_ai::detect_degeneration(&decisions.iter().filter_map(|item|item.confidence.map(|confidence|(item.action.clone(),confidence))).collect::<Vec<_>>());
    let mut reason_codes=reasons.into_iter().map(|(reason_code,count)|MarketReasonCodeCount{reason_code,count}).collect::<Vec<_>>();reason_codes.sort_by(|a,b|b.count.cmp(&a.count).then(a.reason_code.cmp(&b.reason_code)));
    Ok(Some(MarketSignalDiagnostics{signal_engine_version:market_signals::SIGNAL_ENGINE_VERSION.into(),signal_config_version:market_signals::SIGNAL_CONFIG_VERSION.into(),bullish_count:bullish,bearish_count:bearish,neutral_count:neutral,strong_count:strong,weak_count:weak,low_conflict_count:low,medium_conflict_count:medium,high_conflict_count:high,action_collapse:collapse.action_collapse,collapsed_action:collapse.collapsed_action,confidence_collapse:collapse.confidence_collapse,collapsed_confidence:collapse.collapsed_confidence,disagreement_count:disagreements,disagreement_rate_pct:if decisions.is_empty(){0.0}else{disagreements as f64/decisions.len() as f64*100.0},strong_bullish_buy_rate_pct:rate(sb.buy_count,sbd),strong_bullish_hold_rate_pct:rate(sb.hold_count,sbd),strong_bearish_sell_rate_pct:rate(br.sell_count,brd),strong_bearish_hold_rate_pct:rate(br.hold_count,brd),signal_action_matrix:signal_rows,conflict_action_matrix:conflict_rows,reason_codes}))
}

#[cfg(test)] mod tests{use super::*;fn csv()->String{"timestamp,open,high,low,close,volume\n2026-01-01T00:00:00Z,10,11,9,10,100\n2026-01-02T00:00:00Z,11,12,10,11,110\n2026-01-03T00:00:00Z,12,13,11,12,120\n".into()}#[test]fn validates_ohlc_and_order(){assert_eq!(parse_csv(&csv()).unwrap().len(),3);assert!(parse_csv("timestamp,open,high,low,close\n2,10,9,8,10").is_err());assert!(parse_csv("timestamp,open,high,low,close\n2,10,11,9,10\n1,10,11,9,10").is_err())}#[test]fn features_have_real_warmup(){let c=parse_csv(&csv()).unwrap();let f=feature_at(&c,1);assert!(f.sma_long.is_none());assert_eq!(round(f.returns.unwrap()),0.1)}#[test]fn deterministic_random_is_repeatable(){assert_eq!(random_unit(42,7,"random-controlled"),random_unit(42,7,"random-controlled"));assert_ne!(random_unit(42,7,"random-controlled"),random_unit(43,7,"random-controlled"));}#[test]fn risk_modifies_excess_position(){let p=Portfolio{cash:100.0,quantity:0.0,average_entry:0.0,realized:0.0,peak_equity:100.0,trades:0,wins:0,gross_profit:0.0,gross_loss:0.0};let profile=MarketRiskProfile{id:"r".into(),name:"R".into(),max_position_pct:10.0,max_total_exposure_pct:20.0,max_daily_loss_pct:5.0,max_drawdown_pct:20.0,max_trades_per_day:None,allow_leverage:false,allow_short:false,allowed_assets:vec![]};let d=Decision{action:"BUY",desired_pct:Some(90.0),confidence:None,reasoning:"test".into()};assert_eq!(risk(&d,&p,10.0,&profile,"TEST",false).2,Some(10.0));}#[test]fn execution_uses_next_open(){let mut p=Portfolio{cash:100.0,quantity:0.0,average_entry:0.0,realized:0.0,peak_equity:100.0,trades:0,wins:0,gross_profit:0.0,gross_loss:0.0};let candle=Candle{timestamp:"T+1".into(),open:20.0,high:20.0,low:20.0,close:20.0,volume:None};let ex=execute(&mut p,&PendingOrder{decision_index:0,action:"BUY",approved_pct:50.0},&candle,0.0,0.0).unwrap();assert_eq!(ex.price,20.0);assert_eq!(ex.quantity,2.5);assert_eq!(equity(&p,20.0),100.0);}}

#[cfg(test)]
mod temporal_column_tests {
    use super::*;

    #[test]
    fn accepts_date_alias_with_iso_date_values() {
        let candles=parse_csv("Date,open,high,low,close,volume\n2026-01-01,10,11,9,10,100\n2026-01-02,11,12,10,11,110").unwrap();
        assert_eq!(candles.len(),2);
        assert_eq!(candles[0].timestamp,"2026-01-01");
    }

    #[test]
    fn accepts_date_only_values_in_timestamp_column() {
        let candles=parse_csv("timestamp,open,high,low,close\n2026-01-01,10,11,9,10\n2026-01-02,11,12,10,11").unwrap();
        assert_eq!(candles.len(),2);
    }

    #[test]
    fn timestamp_has_priority_when_both_temporal_columns_exist() {
        let candles=parse_csv("date,timestamp,open,high,low,close\nnot-a-date,2026-01-01T00:00:00Z,10,11,9,10").unwrap();
        assert_eq!(candles[0].timestamp,"2026-01-01T00:00:00Z");
    }

    #[test]
    fn rejects_missing_duplicate_and_unordered_temporal_values() {
        let missing=parse_csv("open,high,low,close\n10,11,9,10").unwrap_err();
        assert!(missing.contains("timestamp ou date"));
        assert!(parse_csv("date,open,high,low,close\n2026-01-01,10,11,9,10\n2026-01-01,11,12,10,11").unwrap_err().contains("duplicado"));
        assert!(parse_csv("date,open,high,low,close\n2026-01-02,10,11,9,10\n2026-01-01,11,12,10,11").unwrap_err().contains("fora de ordem"));
    }
}

#[cfg(test)]
mod v041_context_tests {
    use super::*;

    fn candles(values: &[f64]) -> Vec<Candle> {
        values.iter().enumerate().map(|(index, close)| Candle { timestamp: format!("T{index}"), open: *close, high: *close + 1.0, low: *close - 1.0, close: *close, volume: None }).collect()
    }

    fn run(position: bool, entry: f64, started: Option<usize>) -> AgentRun {
        AgentRun { id: market_ai::AI_AGENT_V2_ID.into(), name: "AI Technical V2".into(), status: "completed".into(), portfolio: Portfolio { cash: if position { 700.0 } else { 1_000.0 }, quantity: if position { 3.0 } else { 0.0 }, average_entry: if position { entry } else { 0.0 }, realized: 0.0, peak_equity: 1_000.0, trades: 0, wins: 0, gross_profit: 0.0, gross_loss: 0.0 }, pending: None, position_started_index: started, decisions: Vec::new(), snapshots: Vec::new(), max_drawdown: 0.0, exposure_sum: 0.0, max_exposure: 0.0, hold_count: 0, ai_runtime: Default::default() }
    }

    fn profile() -> MarketRiskProfile {
        MarketRiskProfile { id: "balanced-v1".into(), name: "Balanced".into(), max_position_pct: 50.0, max_total_exposure_pct: 50.0, max_daily_loss_pct: 5.0, max_drawdown_pct: 20.0, max_trades_per_day: None, allow_leverage: false, allow_short: false, allowed_assets: vec![] }
    }

    fn dataset() -> MarketDataset {
        MarketDataset { id: "d".into(), name: "Known".into(), asset: "AAPL".into(), timeframe: "1D".into(), currency: "USD".into(), start_at: "T0".into(), end_at: "T20".into(), candle_count: 21, fingerprint: "fixture".into(), source_path: "fixture.csv".into(), imported_at: "now".into() }
    }

    #[test]
    fn v2_context_contains_percentages_regime_and_no_future_returns() {
        let values: Vec<f64> = (90..=110).map(|value| value as f64).collect();
        let snapshot = build_v2_snapshot(&dataset(), &profile(), &candles(&values), 20, &run(false, 0.0, None), 1_000.0, 0.0);
        let json = serde_json::to_value(snapshot).unwrap();
        assert!(json.pointer("/returns/return_10").and_then(|value| value.as_f64()).unwrap() > 0.0);
        assert!(json.pointer("/trend/price_vs_sma_short_pct").and_then(|value| value.as_f64()).unwrap() > 0.0);
        assert!(json.pointer("/trend/price_vs_sma_long_pct").and_then(|value| value.as_f64()).unwrap() > 0.0);
        assert_eq!(json.pointer("/regime/trend").and_then(|value| value.as_str()), Some("BULL"));
        assert!(json.get("forward_return_1").is_none());
        assert!(json.to_string().find("forward_return").is_none());
    }

    #[test]
    fn v2_context_exposes_negative_trend_and_complete_position_state() {
        let values: Vec<f64> = (90..=110).rev().map(|value| value as f64).collect();
        let snapshot = build_v2_snapshot(&dataset(), &profile(), &candles(&values), 20, &run(true, 100.0, Some(8)), 1_030.0, 30.0);
        let json = serde_json::to_value(snapshot).unwrap();
        assert!(json.pointer("/returns/return_5").and_then(|value| value.as_f64()).unwrap() < 0.0);
        assert!(json.pointer("/trend/sma_spread_pct").and_then(|value| value.as_f64()).unwrap() < 0.0);
        assert_eq!(json.pointer("/regime/trend").and_then(|value| value.as_str()), Some("BEAR"));
        assert_eq!(json.pointer("/portfolio/position_pct").and_then(|value| value.as_f64()), Some(30.0));
        assert_eq!(json.pointer("/portfolio/entry_price").and_then(|value| value.as_f64()), Some(100.0));
        assert_eq!(json.pointer("/portfolio/holding_period").and_then(|value| value.as_u64()), Some(12));
    }

    #[test]
    fn forward_returns_use_only_exact_available_horizons() {
        let data = candles(&[100.0, 110.0, 120.0, 130.0, 140.0, 150.0]);
        assert_eq!(forward_return(&data, 0, 1), Some(10.0));
        assert_eq!(forward_return(&data, 0, 5), Some(50.0));
        assert_eq!(forward_return(&data, 0, 10), None);
        assert_eq!(forward_return(&data, 5, 1), None);
    }
}
