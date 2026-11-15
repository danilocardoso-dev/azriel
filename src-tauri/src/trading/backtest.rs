//! Backtest das regras do motor sobre velas históricas de 1 minuto, sem chamar a IA.
//! A decisão acontece no fechamento de cada vela de 5 min, como no motor. As saídas usam as
//! mesmas funções do vigia (`strategy::quick_exit`) e o stop/alvo da corretora pela máxima e
//! mínima de cada minuto. Quando os dois cabem no mesmo minuto, conta o stop (conservador).
//!
//! Rodar: `AZRIEL_BACKTEST_CANDLES=caminho/btc_1m.json cargo test --lib backtest_report -- --ignored --nocapture`
//! O arquivo é uma lista de `[openTimeMs, open, high, low, close, volume]` de 1 minuto.

use super::indicators::{atr, ema, rsi, Candle};
use super::strategy::{self, Action, Decision, ExitRules, RiskConfig, RiskState, Side};

const MINUTE_MS: i64 = 60_000;
const DECISION_MS: i64 = 5 * MINUTE_MS;
const HOUR_MS: i64 = 60 * MINUTE_MS;
/// Mesmo histórico que o motor pede à corretora a cada ciclo.
const CONTEXT_CANDLES: usize = 150;
const CONTEXT_HOURS: usize = 120;

/// Junta velas de 1 min em velas de `period_ms`, alinhadas ao relógio; descarta as incompletas.
pub fn aggregate(minutes: &[Candle], period_ms: i64) -> Vec<Candle> {
    let per = (period_ms / MINUTE_MS) as usize;
    let mut out = Vec::new();
    let mut index = 0;
    while index < minutes.len() {
        let bucket = minutes[index].open_time.div_euclid(period_ms) * period_ms;
        let end = minutes[index..].iter().position(|c| c.open_time >= bucket + period_ms).map_or(minutes.len(), |p| index + p);
        let group = &minutes[index..end];
        if group.len() == per && group[0].open_time == bucket {
            out.push(Candle {
                open_time: bucket,
                open: group[0].open,
                high: group.iter().map(|c| c.high).fold(f64::MIN, f64::max),
                low: group.iter().map(|c| c.low).fold(f64::MAX, f64::min),
                close: group[per - 1].close,
                volume: group.iter().map(|c| c.volume).sum(),
            });
        }
        index = end;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Signal {
    /// O que o prompt v6 pede: a favor do 1h, curto prazo confirmando, RSI no lado do trade,
    /// última vela na direção. Passa pelas travas de `apply_risk`, como no motor.
    Momentum,
    /// O lado oposto do Momentum, nos mesmos momentos e sem travas: testa se as entradas chegam atrasadas.
    Fade,
    /// Correção do atraso: a favor da tendência, mas no recuo até a EMA21 (até meio ATR dela),
    /// com a estrutura alinhada (EMA21 x EMA50), RSI vindo de baixo de 45 (acima de 55 na venda)
    /// e a última vela já virando a favor. Passa pelas travas de `apply_risk`.
    Pullback,
    /// Entrada aleatória (probabilidade por vela, lado no cara ou coroa), sem travas.
    Random { chance: f64, seed: u64 },
    /// Momento aleatório, lado sempre a favor do 1h, com as travas.
    TrendRandom { chance: f64, seed: u64 },
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub signal: Signal,
    /// Vela em que o motor decide (5 min no motor de hoje).
    pub decision_ms: i64,
    /// Vela da tendência (1h no motor de hoje).
    pub trend_ms: i64,
    pub exit: ExitRules,
    /// Taxa de ida e volta em % do preço (0 mostra o resultado bruto do sinal).
    pub fees_pct: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Trade {
    pub side: Side,
    pub opened_at: i64,
    pub minutes: i64,
    /// Resultado em % do valor da posição, já sem a taxa. Com 100 USDT de exposição, 1% = 1 USDT.
    pub net_pct: f64,
    pub reason: &'static str,
}

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Leituras do snapshot que o motor calcula a cada ciclo, sobre o histórico fechado até a decisão.
struct Snapshot {
    trend: Option<Side>,
    short_term: &'static str,
    rsi: Option<f64>,
    last_up: bool,
    ema21_above_ema50: Option<bool>,
    risk: RiskState,
}

fn snapshot(candles: &[Candle], hourly: &[Candle]) -> Snapshot {
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();
    let hourly_closes: Vec<f64> = hourly.iter().map(|c| c.close).collect();
    let price = *closes.last().unwrap();
    let direction = strategy::alignment(&closes, &hourly_closes);
    let trend = strategy::trend_side(direction.trend_1h);
    let recent = strategy::recent_rsi(&closes);
    let last = candles.last().unwrap();
    Snapshot {
        trend,
        short_term: direction.short_term,
        rsi: rsi(&closes, 14),
        last_up: last.close > last.open,
        ema21_above_ema50: ema(&closes, 21).zip(ema(&closes, 50)).map(|(fast, slow)| fast > slow),
        risk: RiskState {
            position: None,
            equity: 0.0,
            day_start_equity: None,
            trades_today: 0,
            trend,
            atr_pct: atr(candles, 14).map(|v| v / price * 100.0),
            price_vs_ema21_pct: ema(&closes, 21).map(|v| (price - v) / v * 100.0),
            position_move_pct: None,
            ema9_above_ema21: ema(&closes, 9).zip(ema(&closes, 21)).map(|(fast, slow)| fast > slow),
            rsi_recent_low: recent.map(|r| r.0),
            rsi_recent_high: recent.map(|r| r.1),
        },
    }
}

fn momentum_side(snap: &Snapshot) -> Option<Side> {
    let side = snap.trend?;
    let rsi = snap.rsi?;
    match side {
        Side::Long if snap.short_term == "ALTA" && (50.0..75.0).contains(&rsi) && snap.last_up => Some(Side::Long),
        Side::Short if snap.short_term == "BAIXA" && rsi > 25.0 && rsi <= 50.0 && !snap.last_up => Some(Side::Short),
        _ => None,
    }
}

fn pullback_side(snap: &Snapshot) -> Option<Side> {
    let side = snap.trend?;
    let (rsi, low, high) = (snap.rsi?, snap.risk.rsi_recent_low?, snap.risk.rsi_recent_high?);
    let (atr, distance) = (snap.risk.atr_pct?, snap.risk.price_vs_ema21_pct?);
    if distance.abs() > 0.5 * atr {
        return None;
    }
    match side {
        Side::Long if snap.ema21_above_ema50 == Some(true) && low < 45.0 && rsi > low + 3.0 && snap.last_up => Some(Side::Long),
        Side::Short if snap.ema21_above_ema50 == Some(false) && high > 55.0 && rsi < high - 3.0 && !snap.last_up => Some(Side::Short),
        _ => None,
    }
}

fn passes_risk(side: Side, snap: &Snapshot, exit: &ExitRules) -> bool {
    let config = RiskConfig { min_confidence: 60, max_daily_loss_pct: 100.0, max_trades_per_day: i64::MAX, stop_loss_pct: exit.stop_loss_pct, take_profit_pct: exit.take_profit_pct };
    let action = if side == Side::Long { Action::OpenLong } else { Action::OpenShort };
    let decision = Decision { action, confidence: 75, reason: String::new() };
    strategy::apply_risk(&decision, &snap.risk, &config).action == action
}

fn opposite(side: Side) -> Side {
    if side == Side::Long { Side::Short } else { Side::Long }
}

/// Segue a posição minuto a minuto até a saída. Devolve (índice do último minuto, preço, motivo).
fn follow(minutes: &[Candle], start: usize, entry_time: i64, side: Side, entry: f64, exit: &ExitRules) -> Option<(usize, f64, &'static str)> {
    let (stop, target) = strategy::protection_prices(side, entry, exit.stop_loss_pct, exit.take_profit_pct, 8);
    let mut peak = f64::MIN;
    for (index, minute) in minutes.iter().enumerate().skip(start) {
        let (stop_hit, target_hit) = match side {
            Side::Long => (minute.low <= stop, minute.high >= target),
            Side::Short => (minute.high >= stop, minute.low <= target),
        };
        if stop_hit {
            return Some((index, stop, "STOP"));
        }
        if target_hit {
            return Some((index, target, "TARGET"));
        }
        let moved = strategy::move_pct(side, entry, minute.close);
        peak = peak.max(moved);
        let open_minutes = (minute.open_time + MINUTE_MS - entry_time) / MINUTE_MS;
        if let Some((reason, _)) = strategy::quick_exit(moved, peak, open_minutes, exit) {
            return Some((index, minute.close, reason));
        }
    }
    None
}

pub fn run(minutes: &[Candle], config: &Config) -> Vec<Trade> {
    let candles = aggregate(minutes, config.decision_ms);
    let hourly = aggregate(minutes, config.trend_ms);
    let mut rng = match config.signal {
        Signal::Random { seed, .. } | Signal::TrendRandom { seed, .. } => Rng(seed.max(1)),
        _ => Rng(1),
    };
    let mut trades = Vec::new();
    let mut hour_end = 0;
    let mut free_at = i64::MIN;
    let mut minute_index = 0;
    for index in CONTEXT_CANDLES..candles.len() {
        let decided_at = candles[index].open_time + config.decision_ms;
        if decided_at < free_at {
            continue;
        }
        while hour_end < hourly.len() && hourly[hour_end].open_time + config.trend_ms <= decided_at {
            hour_end += 1;
        }
        if hour_end < CONTEXT_HOURS / 2 {
            continue;
        }
        let window = &candles[index + 1 - CONTEXT_CANDLES..=index];
        let hours = &hourly[hour_end.saturating_sub(CONTEXT_HOURS)..hour_end];
        let side = match config.signal {
            Signal::Momentum => {
                let snap = snapshot(window, hours);
                momentum_side(&snap).filter(|side| passes_risk(*side, &snap, &config.exit))
            }
            Signal::Pullback => {
                let snap = snapshot(window, hours);
                pullback_side(&snap).filter(|side| passes_risk(*side, &snap, &config.exit))
            }
            Signal::Fade => momentum_side(&snapshot(window, hours)).map(opposite),
            Signal::Random { chance, .. } => {
                let roll = rng.next();
                let side = if rng.next() < 0.5 { Side::Long } else { Side::Short };
                (roll < chance).then_some(side)
            }
            Signal::TrendRandom { chance, .. } => {
                if rng.next() >= chance {
                    None
                } else {
                    let snap = snapshot(window, hours);
                    snap.trend.filter(|side| passes_risk(*side, &snap, &config.exit))
                }
            }
        };
        let Some(side) = side else { continue };
        while minute_index < minutes.len() && minutes[minute_index].open_time < decided_at {
            minute_index += 1;
        }
        let entry = candles[index].close;
        let Some((last, price, reason)) = follow(minutes, minute_index, decided_at, side, entry, &config.exit) else { break };
        let closed_at = minutes[last].open_time + MINUTE_MS;
        trades.push(Trade {
            side,
            opened_at: decided_at,
            minutes: (closed_at - decided_at) / MINUTE_MS,
            net_pct: strategy::move_pct(side, entry, price) - config.fees_pct,
            reason,
        });
        free_at = closed_at;
    }
    trades
}

#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub count: usize,
    pub wins: usize,
    pub total_pct: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub max_drawdown: f64,
}

pub fn summarize(trades: &[Trade]) -> Summary {
    let wins: Vec<f64> = trades.iter().map(|t| t.net_pct).filter(|p| *p > 0.0).collect();
    let losses: Vec<f64> = trades.iter().map(|t| t.net_pct).filter(|p| *p <= 0.0).collect();
    let mean = |v: &[f64]| if v.is_empty() { 0.0 } else { v.iter().sum::<f64>() / v.len() as f64 };
    let (mut equity, mut top, mut drawdown) = (0.0f64, 0.0f64, 0.0f64);
    for trade in trades {
        equity += trade.net_pct;
        top = top.max(equity);
        drawdown = drawdown.max(top - equity);
    }
    Summary { count: trades.len(), wins: wins.len(), total_pct: equity, avg_win: mean(&wins), avg_loss: mean(&losses), max_drawdown: drawdown }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minute(i: i64, open: f64, high: f64, low: f64, close: f64) -> Candle {
        Candle { open_time: i * MINUTE_MS, open, high, low, close, volume: 1.0 }
    }

    const RULES: ExitRules = ExitRules { stop_loss_pct: 0.3, take_profit_pct: 0.5, profit_lock_pct: 0.25, max_hold_minutes: 30 };

    #[test]
    fn aggregates_only_complete_aligned_buckets() {
        let minutes: Vec<Candle> = (2..13).map(|i| minute(i, 100.0 + i as f64, 101.0 + i as f64, 99.0, 100.5 + i as f64)).collect();
        let five = aggregate(&minutes, DECISION_MS);
        assert_eq!(five.len(), 1, "2..4 incompleto, 5..9 completo, 10..12 incompleto");
        assert_eq!((five[0].open_time, five[0].open, five[0].close, five[0].high, five[0].volume), (5 * MINUTE_MS, 105.0, 109.5, 110.0, 5.0));
    }

    #[test]
    fn follow_uses_intrabar_stop_before_target_then_watcher_rules() {
        // Mesmo minuto toca stop e alvo: conta o stop.
        let both = [minute(0, 100.0, 101.0, 99.0, 100.0)];
        assert_eq!(follow(&both, 0, 0, Side::Long, 100.0, &RULES).map(|r| r.2), Some("STOP"));
        let target = [minute(0, 100.0, 100.6, 99.9, 100.4)];
        let (_, price, reason) = follow(&target, 0, 0, Side::Long, 100.0, &RULES).unwrap();
        assert_eq!((reason, (price * 100.0).round() / 100.0), ("TARGET", 100.5));
        // Venda que anda a favor até +0,3% e volta para +0,05%: trava de lucro.
        let lock = [minute(0, 100.0, 100.0, 99.69, 99.7), minute(1, 99.7, 99.96, 99.7, 99.95)];
        assert_eq!(follow(&lock, 0, 0, Side::Short, 100.0, &RULES).map(|r| r.2), Some("LOCK"));
        let flat: Vec<Candle> = (0..40).map(|i| minute(i, 100.0, 100.05, 99.95, 100.0)).collect();
        let (last, _, reason) = follow(&flat, 0, 0, Side::Long, 100.0, &RULES).unwrap();
        assert_eq!((reason, last), ("TIMEOUT", 29));
    }

    #[test]
    fn summary_tracks_drawdown_on_the_sum_of_results() {
        let trade = |net_pct| Trade { side: Side::Long, opened_at: 0, minutes: 1, net_pct, reason: "STOP" };
        let summary = summarize(&[trade(1.0), trade(-0.5), trade(-0.7), trade(0.4)]);
        assert_eq!((summary.count, summary.wins), (4, 2));
        assert!((summary.total_pct - 0.2).abs() < 1e-9 && (summary.max_drawdown - 1.2).abs() < 1e-9);
    }

    fn load() -> Vec<Candle> {
        let path = std::env::var("AZRIEL_BACKTEST_CANDLES").expect("defina AZRIEL_BACKTEST_CANDLES");
        let rows: Vec<[f64; 6]> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        rows.iter().map(|r| Candle { open_time: r[0] as i64, open: r[1], high: r[2], low: r[3], close: r[4], volume: r[5] }).collect()
    }

    /// Uma linha por estratégia: resultado por trade com e sem taxa, duração média e o total por trimestre.
    fn line(name: &str, trades: &[Trade], fees: f64) {
        let s = summarize(trades);
        let n = s.count.max(1) as f64;
        let rate = s.wins as f64 / n * 100.0;
        let mut reasons: Vec<(&str, usize)> = Vec::new();
        for t in trades {
            match reasons.iter_mut().find(|r| r.0 == t.reason) {
                Some(r) => r.1 += 1,
                None => reasons.push((t.reason, 1)),
            }
        }
        let minutes = trades.iter().map(|t| t.minutes).sum::<i64>() as f64 / n;
        let quarters: Vec<String> = trades.chunks(trades.len().div_ceil(4).max(1)).map(|c| format!("{:+.1}", summarize(c).total_pct)).collect();
        println!(
            "  {name:<26} n={:>5} acerto={:>4.1}% líquido/trade={:>+.3}% bruto/trade={:>+.3}% ganho/perda={:+.2}/{:+.2} total={:>+7.1}% dd={:>5.1}% duração={:>4.0} min quartos=[{}] {:?}",
            s.count, rate, s.total_pct / n, s.total_pct / n + fees, s.avg_win, s.avg_loss, s.total_pct, s.max_drawdown, minutes, quarters.join(" "), reasons
        );
    }

    /// Entradas aleatórias com a mesma frequência e as mesmas saídas: a régua do "sem vantagem".
    fn random_line(name: &str, minutes: &[Candle], config: Config, chance: f64, trend: bool, own: &[Trade]) {
        let own_avg = summarize(own).total_pct / own.len().max(1) as f64;
        let mut averages: Vec<f64> = (1..=100u64)
            .map(|seed| {
                let signal = if trend { Signal::TrendRandom { chance, seed: seed * 7919 } } else { Signal::Random { chance, seed: seed * 7919 } };
                let s = summarize(&run(minutes, &Config { signal, ..config }));
                s.total_pct / s.count.max(1) as f64
            })
            .collect();
        averages.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let beaten = averages.iter().filter(|a| **a < own_avg).count();
        println!(
            "  {name:<26} líquido/trade médio={:>+.3}% (5–95%: {:+.3} a {:+.3}) · Recuo supera {beaten}/100",
            averages.iter().sum::<f64>() / 100.0, averages[5], averages[94]
        );
    }

    struct Scenario {
        name: &'static str,
        decision_ms: i64,
        trend_ms: i64,
        exit: ExitRules,
    }

    #[test]
    #[ignore]
    fn backtest_report() {
        let minutes = load();
        let days = (minutes.last().unwrap().open_time - minutes[0].open_time) as f64 / 86_400_000.0;
        println!("\n{} velas de 1m, {:.0} dias. Resultados em % da exposição (com 100 USDT, 1% = 1 USDT).", minutes.len(), days);
        let rules = |stop_loss_pct, take_profit_pct, profit_lock_pct, max_hold_minutes| ExitRules { stop_loss_pct, take_profit_pct, profit_lock_pct, max_hold_minutes };
        let scenarios = [
            Scenario { name: "A · hoje: vela 5m, tendência 1h, stop/alvo 0,21/0,35, trava 0,25, até 30 min", decision_ms: DECISION_MS, trend_ms: HOUR_MS, exit: rules(0.21, 0.35, 0.25, 30) },
            Scenario { name: "B · vela 15m, tendência 1h, stop/alvo 0,8/1,6, trava 0,6, até 3h", decision_ms: 15 * MINUTE_MS, trend_ms: HOUR_MS, exit: rules(0.8, 1.6, 0.6, 180) },
            Scenario { name: "C · vela 1h, tendência 4h, stop/alvo 1,0/2,0, trava 0,8, até 3h", decision_ms: HOUR_MS, trend_ms: 4 * HOUR_MS, exit: rules(1.0, 2.0, 0.8, 180) },
        ];
        // Taker 0,05% por lado (hoje) e maker 0,02% por lado (ordem limitada).
        let fee_sets = [("taker 0,10%", strategy::FEES_PCT), ("maker 0,04%", 0.04)];
        for scenario in &scenarios {
            println!("\n=== {} ===", scenario.name);
            for (fee_name, fees) in fee_sets {
                println!(" [{fee_name}]");
                let config = |signal| Config { signal, decision_ms: scenario.decision_ms, trend_ms: scenario.trend_ms, exit: scenario.exit, fees_pct: fees };
                line("Momentum (regras v6)", &run(&minutes, &config(Signal::Momentum)), fees);
                line("Fade (oposto do v6)", &run(&minutes, &config(Signal::Fade)), fees);
                let pullback = run(&minutes, &config(Signal::Pullback));
                line("Recuo (entrada corrigida)", &pullback, fees);
                let decisions = days * 86_400_000.0 / scenario.decision_ms as f64;
                let chance = (pullback.len() as f64 * 2.0 / decisions).clamp(0.002, 0.5);
                random_line("Aleatório", &minutes, config(Signal::Momentum), chance, false, &pullback);
                random_line("Aleatório a favor da tend.", &minutes, config(Signal::Momentum), chance, true, &pullback);
            }
        }
    }
}
