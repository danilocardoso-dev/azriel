use super::market_models::{
    MarketPositionEvent, MarketPositionLifecycleReport, MarketPositionMetric, MarketTradeLifecycle,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const POSITION_ENGINE_VERSION: &str = "POSITION_ENGINE_V1";
pub const POSITION_SIZING_VERSION: &str = "POSITION_SIZING_V1";
pub const RAPID_WINDOW_CANDLES: usize = 5;
const EPSILON: f64 = 0.01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionIntent {
    Enter,
    Hold,
    Reduce,
    Exit,
}

impl PositionIntent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enter => "ENTER",
            Self::Hold => "HOLD",
            Self::Reduce => "REDUCE",
            Self::Exit => "EXIT",
        }
    }
}

impl std::str::FromStr for PositionIntent {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ENTER" => Ok(Self::Enter),
            "HOLD" => Ok(Self::Hold),
            "REDUCE" => Ok(Self::Reduce),
            "EXIT" => Ok(Self::Exit),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionSizingConfig {
    pub version: String,
    pub default_entry_exposure_pct: f64,
    pub default_increase_step_pct: f64,
    pub default_reduce_step_pct: f64,
}

impl Default for PositionSizingConfig {
    fn default() -> Self {
        Self {
            version: POSITION_SIZING_VERSION.into(),
            default_entry_exposure_pct: 25.0,
            default_increase_step_pct: 10.0,
            default_reduce_step_pct: 15.0,
        }
    }
}

impl PositionSizingConfig {
    pub fn validate(&self) -> Result<(), String> {
        let values = [
            self.default_entry_exposure_pct,
            self.default_increase_step_pct,
            self.default_reduce_step_pct,
        ];
        if self.version != POSITION_SIZING_VERSION
            || values
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0 || *value > 100.0)
        {
            return Err("configuração de sizing de posição inválida".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedPositionDecision {
    pub intent: PositionIntent,
    pub lifecycle_action: LifecycleAction,
    pub previous_exposure_pct: f64,
    pub generated_target_exposure_pct: f64,
    pub sizing_config_version: String,
}

pub fn intent_allowed(state: PositionState, intent: PositionIntent) -> bool {
    match state {
        PositionState::Flat => matches!(intent, PositionIntent::Enter | PositionIntent::Hold),
        PositionState::LongOpen | PositionState::LongReduced => true,
        PositionState::ExitPending => matches!(intent, PositionIntent::Hold),
    }
}

pub fn generate_from_intent(
    state: PositionState,
    current_exposure_pct: f64,
    intent: PositionIntent,
    config: &PositionSizingConfig,
) -> Result<GeneratedPositionDecision, String> {
    config.validate()?;
    if !current_exposure_pct.is_finite() || !(0.0..=100.0).contains(&current_exposure_pct) {
        return Err("POSITION_CONTEXT_ERROR: exposição atual inválida".into());
    }
    if !intent_allowed(state, intent) {
        return Err("INVALID_POSITION_INTENT".into());
    }
    let (lifecycle_action, target) = match (state, intent) {
        (PositionState::Flat, PositionIntent::Enter) => {
            (LifecycleAction::EnterLong, config.default_entry_exposure_pct)
        }
        (PositionState::Flat, PositionIntent::Hold) => (LifecycleAction::StayFlat, 0.0),
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Enter)
            if current_exposure_pct >= 100.0 - EPSILON =>
        {
            (LifecycleAction::HoldPosition, current_exposure_pct)
        }
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Enter) => (
            LifecycleAction::IncreaseLong,
            (current_exposure_pct + config.default_increase_step_pct).min(100.0),
        ),
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Hold) => {
            (LifecycleAction::HoldPosition, current_exposure_pct)
        }
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Reduce)
            if current_exposure_pct <= EPSILON * 2.0 =>
        {
            (LifecycleAction::HoldPosition, current_exposure_pct)
        }
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Reduce) => {
            let reduction = config
                .default_reduce_step_pct
                .min(current_exposure_pct / 2.0);
            (LifecycleAction::ReduceLong, current_exposure_pct - reduction)
        }
        (PositionState::LongOpen | PositionState::LongReduced, PositionIntent::Exit) => {
            (LifecycleAction::ExitLong, 0.0)
        }
        (PositionState::ExitPending, PositionIntent::Hold) => {
            (LifecycleAction::HoldPosition, current_exposure_pct)
        }
        _ => return Err("INVALID_POSITION_INTENT".into()),
    };
    Ok(GeneratedPositionDecision {
        intent,
        lifecycle_action,
        previous_exposure_pct: current_exposure_pct,
        generated_target_exposure_pct: target,
        sizing_config_version: config.version.clone(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionState {
    Flat,
    LongOpen,
    LongReduced,
    ExitPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleAction {
    EnterLong,
    HoldPosition,
    IncreaseLong,
    ReduceLong,
    ExitLong,
    StayFlat,
}

impl LifecycleAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EnterLong => "ENTER_LONG",
            Self::HoldPosition => "HOLD_POSITION",
            Self::IncreaseLong => "INCREASE_LONG",
            Self::ReduceLong => "REDUCE_LONG",
            Self::ExitLong => "EXIT_LONG",
            Self::StayFlat => "STAY_FLAT",
        }
    }
    pub fn internal_action(self) -> &'static str {
        match self {
            Self::EnterLong | Self::IncreaseLong => "BUY",
            Self::ReduceLong | Self::ExitLong => "SELL",
            Self::HoldPosition | Self::StayFlat => "HOLD",
        }
    }
}

impl std::str::FromStr for LifecycleAction {
    type Err = ();
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ENTER_LONG" => Ok(Self::EnterLong),
            "HOLD_POSITION" => Ok(Self::HoldPosition),
            "INCREASE_LONG" => Ok(Self::IncreaseLong),
            "REDUCE_LONG" => Ok(Self::ReduceLong),
            "EXIT_LONG" => Ok(Self::ExitLong),
            "STAY_FLAT" => Ok(Self::StayFlat),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionContext {
    pub state: PositionState,
    pub quantity: f64,
    pub exposure_pct: f64,
    pub entry_price: Option<f64>,
    pub average_entry_price: Option<f64>,
    pub current_price: f64,
    pub unrealized_pnl_pct: Option<f64>,
    pub holding_candles: usize,
    pub max_favorable_excursion_pct: Option<f64>,
    pub max_adverse_excursion_pct: Option<f64>,
    pub last_entry_timestamp: Option<String>,
    pub last_exit_timestamp: Option<String>,
    pub time_since_last_exit: Option<usize>,
    pub reentry_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedPositionDecision {
    pub requested_action: LifecycleAction,
    pub effective_action: LifecycleAction,
    pub target_exposure_pct: f64,
    pub validation_code: Option<String>,
}

pub fn validate_decision(
    state: PositionState,
    current: f64,
    action: LifecycleAction,
    target: f64,
) -> ValidatedPositionDecision {
    let finite = target.is_finite() && (0.0..=100.0).contains(&target);
    let coherent = finite
        && match (state, action) {
            (PositionState::Flat, LifecycleAction::StayFlat) => target.abs() <= EPSILON,
            (PositionState::Flat, LifecycleAction::EnterLong) => target > EPSILON,
            (
                PositionState::LongOpen | PositionState::LongReduced,
                LifecycleAction::HoldPosition,
            ) => (target - current).abs() <= EPSILON,
            (
                PositionState::LongOpen | PositionState::LongReduced,
                LifecycleAction::IncreaseLong,
            ) => target > current + EPSILON,
            (PositionState::LongOpen | PositionState::LongReduced, LifecycleAction::ReduceLong) => {
                target > EPSILON && target < current - EPSILON
            }
            (PositionState::LongOpen | PositionState::LongReduced, LifecycleAction::ExitLong) => {
                target.abs() <= EPSILON
            }
            _ => false,
        };
    if coherent {
        ValidatedPositionDecision {
            requested_action: action,
            effective_action: action,
            target_exposure_pct: target,
            validation_code: None,
        }
    } else {
        let fallback = if state == PositionState::Flat {
            LifecycleAction::StayFlat
        } else {
            LifecycleAction::HoldPosition
        };
        ValidatedPositionDecision {
            requested_action: action,
            effective_action: fallback,
            target_exposure_pct: if state == PositionState::Flat {
                0.0
            } else {
                current
            },
            validation_code: Some("INVALID_POSITION_ACTION".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionEventRecord {
    pub lifecycle_index: usize,
    pub decision_index: usize,
    pub timestamp: String,
    pub action: String,
    pub previous_exposure_pct: f64,
    pub target_exposure_pct: f64,
    pub new_exposure_pct: f64,
    pub signal_bias: Option<String>,
    pub confidence: Option<f64>,
    pub reason_code: Option<String>,
    pub risk_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeLifecycleRecord {
    pub lifecycle_index: usize,
    pub opened_at: String,
    pub closed_at: Option<String>,
    pub entry_price: f64,
    pub average_entry_price: f64,
    pub exit_price: Option<f64>,
    pub initial_exposure_pct: f64,
    pub max_exposure_pct: f64,
    pub holding_candles: usize,
    pub realized_pnl: f64,
    pub realized_pnl_pct: f64,
    pub mfe_pct: f64,
    pub mae_pct: f64,
    pub exit_efficiency_pct: Option<f64>,
    pub profit_giveback_pct: f64,
    pub entry_reason_code: Option<String>,
    pub exit_reason_code: Option<String>,
    pub status: String,
    pub reentry: bool,
}

#[derive(Debug, Clone)]
struct ActiveLifecycle {
    lifecycle_index: usize,
    opened_at: String,
    entry_price: f64,
    average_entry_price: f64,
    initial_exposure: f64,
    current_exposure: f64,
    max_exposure: f64,
    holding: usize,
    highest: f64,
    lowest: f64,
    realized_start: f64,
    capital_basis: f64,
    entry_reason: Option<String>,
    reentry: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PositionManager {
    active: Option<ActiveLifecycle>,
    pub lifecycles: Vec<TradeLifecycleRecord>,
    pub events: Vec<PositionEventRecord>,
    last_exit_index: Option<usize>,
    last_exit_timestamp: Option<String>,
    last_entry_timestamp: Option<String>,
    pub reentry_count: usize,
    pub rapid_reentry_count: usize,
    pub rapid_exit_count: usize,
    pub invalid_position_action_count: usize,
    pub flat_sell_attempt_count: usize,
    pub redundant_exit_count: usize,
}

impl PositionManager {
    pub fn state(&self) -> PositionState {
        match &self.active {
            Some(active) if active.current_exposure + EPSILON < active.max_exposure => {
                PositionState::LongReduced
            }
            Some(_) => PositionState::LongOpen,
            None => PositionState::Flat,
        }
    }
    pub fn context(
        &self,
        quantity: f64,
        exposure: f64,
        average_entry: f64,
        current_price: f64,
        current_index: usize,
    ) -> PositionContext {
        let active = self.active.as_ref();
        PositionContext {
            state: self.state(),
            quantity,
            exposure_pct: exposure,
            entry_price: active.map(|v| v.entry_price),
            average_entry_price: (quantity > 0.0).then_some(average_entry),
            current_price,
            unrealized_pnl_pct: (quantity > 0.0 && average_entry > 0.0)
                .then_some((current_price / average_entry - 1.0) * 100.0),
            holding_candles: active.map_or(0, |v| v.holding),
            max_favorable_excursion_pct: active
                .map(|v| (v.highest / v.average_entry_price - 1.0) * 100.0),
            max_adverse_excursion_pct: active
                .map(|v| (v.lowest / v.average_entry_price - 1.0) * 100.0),
            last_entry_timestamp: self.last_entry_timestamp.clone(),
            last_exit_timestamp: self.last_exit_timestamp.clone(),
            time_since_last_exit: self
                .last_exit_index
                .map(|v| current_index.saturating_sub(v)),
            reentry_count: self.reentry_count,
        }
    }
    pub fn observe(&mut self, high: f64, low: f64) {
        if let Some(active) = &mut self.active {
            active.highest = active.highest.max(high);
            active.lowest = active.lowest.min(low);
            active.holding += 1;
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn execution(
        &mut self,
        decision_index: usize,
        candle_index: usize,
        timestamp: &str,
        price: f64,
        previous_exposure: f64,
        target_exposure: f64,
        new_exposure: f64,
        average_entry: f64,
        realized_before: f64,
        realized_after: f64,
        action: &str,
        reason_code: Option<String>,
        confidence: Option<f64>,
        signal_bias: Option<String>,
        risk_result: &str,
        gross: f64,
    ) {
        let was_flat = previous_exposure <= EPSILON;
        let is_flat = new_exposure <= EPSILON;
        if was_flat && !is_flat {
            let reentry = self.last_exit_index.is_some();
            if reentry {
                self.reentry_count += 1;
                if candle_index.saturating_sub(self.last_exit_index.unwrap_or(0))
                    <= RAPID_WINDOW_CANDLES
                {
                    self.rapid_reentry_count += 1;
                }
            }
            let next = self.lifecycles.len() + 1;
            self.active = Some(ActiveLifecycle {
                lifecycle_index: next,
                opened_at: timestamp.into(),
                entry_price: price,
                average_entry_price: average_entry,
                initial_exposure: new_exposure,
                current_exposure: new_exposure,
                max_exposure: new_exposure,
                holding: 0,
                highest: price,
                lowest: price,
                realized_start: realized_before,
                capital_basis: gross,
                entry_reason: reason_code.clone(),
                reentry,
            });
            self.last_entry_timestamp = Some(timestamp.into());
        }
        if let Some(active) = &mut self.active {
            if !is_flat {
                active.average_entry_price = average_entry.max(f64::EPSILON);
            }
            active.current_exposure = new_exposure;
            active.max_exposure = active.max_exposure.max(new_exposure);
            if action == "INCREASE_LONG" {
                active.capital_basis += gross;
            }
        }
        let lifecycle_index = self
            .active
            .as_ref()
            .map_or(self.lifecycles.len() + 1, |v| v.lifecycle_index);
        self.events.push(PositionEventRecord {
            lifecycle_index,
            decision_index,
            timestamp: timestamp.into(),
            action: action.into(),
            previous_exposure_pct: previous_exposure,
            target_exposure_pct: target_exposure,
            new_exposure_pct: new_exposure,
            signal_bias,
            confidence,
            reason_code: reason_code.clone(),
            risk_result: risk_result.into(),
        });
        if !was_flat && is_flat {
            if let Some(active) = self.active.take() {
                if active.holding <= RAPID_WINDOW_CANDLES {
                    self.rapid_exit_count += 1;
                }
                let realized = realized_after - active.realized_start;
                let realized_pct = if active.capital_basis > 0.0 {
                    realized / active.capital_basis * 100.0
                } else {
                    0.0
                };
                let mfe = (active.highest / active.average_entry_price - 1.0) * 100.0;
                let mae = (active.lowest / active.average_entry_price - 1.0) * 100.0;
                let efficiency = (mfe > 0.0 && realized_pct >= 0.0)
                    .then_some((realized_pct / mfe * 100.0).clamp(0.0, 100.0));
                self.lifecycles.push(TradeLifecycleRecord {
                    lifecycle_index: active.lifecycle_index,
                    opened_at: active.opened_at,
                    closed_at: Some(timestamp.into()),
                    entry_price: active.entry_price,
                    average_entry_price: active.average_entry_price,
                    exit_price: Some(price),
                    initial_exposure_pct: active.initial_exposure,
                    max_exposure_pct: active.max_exposure,
                    holding_candles: active.holding,
                    realized_pnl: realized,
                    realized_pnl_pct: realized_pct,
                    mfe_pct: mfe,
                    mae_pct: mae,
                    exit_efficiency_pct: efficiency,
                    profit_giveback_pct: (mfe - realized_pct).max(0.0),
                    entry_reason_code: active.entry_reason,
                    exit_reason_code: reason_code,
                    status: "CLOSED".into(),
                    reentry: active.reentry,
                });
                self.last_exit_index = Some(candle_index);
                self.last_exit_timestamp = Some(timestamp.into());
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn hold(
        &mut self,
        decision_index: usize,
        timestamp: &str,
        exposure: f64,
        signal_bias: Option<String>,
        confidence: Option<f64>,
        reason_code: Option<String>,
        risk_result: &str,
    ) {
        if let Some(active) = &self.active {
            self.events.push(PositionEventRecord {
                lifecycle_index: active.lifecycle_index,
                decision_index,
                timestamp: timestamp.into(),
                action: "HOLD_POSITION".into(),
                previous_exposure_pct: exposure,
                target_exposure_pct: exposure,
                new_exposure_pct: exposure,
                signal_bias,
                confidence,
                reason_code,
                risk_result: risk_result.into(),
            });
        }
    }
    pub fn finish_open(&mut self, current_price: f64, average_entry: f64, realized: f64) {
        if let Some(active) = self.active.take() {
            let realized_delta = realized - active.realized_start;
            let realized_pct = if active.capital_basis > 0.0 {
                realized_delta / active.capital_basis * 100.0
            } else {
                0.0
            };
            let mfe = (active.highest / average_entry.max(f64::EPSILON) - 1.0) * 100.0;
            let mae = (active.lowest / average_entry.max(f64::EPSILON) - 1.0) * 100.0;
            self.lifecycles.push(TradeLifecycleRecord {
                lifecycle_index: active.lifecycle_index,
                opened_at: active.opened_at,
                closed_at: None,
                entry_price: active.entry_price,
                average_entry_price: average_entry,
                exit_price: None,
                initial_exposure_pct: active.initial_exposure,
                max_exposure_pct: active.max_exposure,
                holding_candles: active.holding,
                realized_pnl: realized_delta,
                realized_pnl_pct: realized_pct,
                mfe_pct: mfe,
                mae_pct: mae,
                exit_efficiency_pct: None,
                profit_giveback_pct: (mfe
                    - (current_price / average_entry.max(f64::EPSILON) - 1.0) * 100.0)
                    .max(0.0),
                entry_reason_code: active.entry_reason,
                exit_reason_code: None,
                status: "OPEN".into(),
                reentry: active.reentry,
            });
        }
    }
}

pub fn load_report(
    connection: &Connection,
    experiment_id: &str,
) -> Result<MarketPositionLifecycleReport, String> {
    let mut lifecycle_statement=connection.prepare("SELECT id,agent_id,asset,lifecycle_index,opened_at,closed_at,entry_price,average_entry_price,exit_price,initial_exposure_pct,max_exposure_pct,holding_candles,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,exit_efficiency_pct,profit_giveback_pct,entry_reason_code,exit_reason_code,status,reentry FROM market_trade_lifecycles WHERE experiment_id=?1 ORDER BY agent_id,lifecycle_index").map_err(|error|error.to_string())?;
    let lifecycles = lifecycle_statement
        .query_map([experiment_id], |row| {
            Ok(MarketTradeLifecycle {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                asset: row.get(2)?,
                lifecycle_index: row.get(3)?,
                opened_at: row.get(4)?,
                closed_at: row.get(5)?,
                entry_price: row.get(6)?,
                average_entry_price: row.get(7)?,
                exit_price: row.get(8)?,
                initial_exposure_pct: row.get(9)?,
                max_exposure_pct: row.get(10)?,
                holding_candles: row.get(11)?,
                realized_pnl: row.get(12)?,
                realized_pnl_pct: row.get(13)?,
                mfe_pct: row.get(14)?,
                mae_pct: row.get(15)?,
                exit_efficiency_pct: row.get(16)?,
                profit_giveback_pct: row.get(17)?,
                entry_reason_code: row.get(18)?,
                exit_reason_code: row.get(19)?,
                status: row.get(20)?,
                reentry: row.get::<_, i64>(21)? != 0,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut event_statement=connection.prepare("SELECT e.id,e.lifecycle_id,e.decision_id,e.execution_id,e.timestamp,e.action,e.previous_exposure_pct,e.target_exposure_pct,e.new_exposure_pct,e.signal_bias,e.confidence,e.reason_code,e.risk_result FROM market_position_events e JOIN market_trade_lifecycles l ON l.id=e.lifecycle_id WHERE l.experiment_id=?1 ORDER BY e.id").map_err(|error|error.to_string())?;
    let events = event_statement
        .query_map([experiment_id], |row| {
            Ok(MarketPositionEvent {
                id: row.get(0)?,
                lifecycle_id: row.get(1)?,
                decision_id: row.get(2)?,
                execution_id: row.get(3)?,
                timestamp: row.get(4)?,
                action: row.get(5)?,
                previous_exposure_pct: row.get(6)?,
                target_exposure_pct: row.get(7)?,
                new_exposure_pct: row.get(8)?,
                signal_bias: row.get(9)?,
                confidence: row.get(10)?,
                reason_code: row.get(11)?,
                risk_result: row.get(12)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    let mut metric_statement=connection.prepare("SELECT agent_id,closed_trades,winning_trades,losing_trades,average_holding_candles,median_holding_candles,average_mfe_pct,average_mae_pct,average_exit_efficiency_pct,average_profit_giveback_pct,rapid_reentry_count,rapid_exit_count,reentry_count,average_entry_exposure_pct,average_max_exposure_pct,invalid_position_action_count,flat_sell_attempt_count,redundant_exit_count,engine_version FROM market_position_metrics WHERE experiment_id=?1 ORDER BY agent_id").map_err(|error|error.to_string())?;
    let metrics = metric_statement
        .query_map([experiment_id], |row| {
            Ok(MarketPositionMetric {
                agent_id: row.get(0)?,
                closed_trades: row.get(1)?,
                winning_trades: row.get(2)?,
                losing_trades: row.get(3)?,
                average_holding_candles: row.get(4)?,
                median_holding_candles: row.get(5)?,
                average_mfe_pct: row.get(6)?,
                average_mae_pct: row.get(7)?,
                average_exit_efficiency_pct: row.get(8)?,
                average_profit_giveback_pct: row.get(9)?,
                rapid_reentry_count: row.get(10)?,
                rapid_exit_count: row.get(11)?,
                reentry_count: row.get(12)?,
                average_entry_exposure_pct: row.get(13)?,
                average_max_exposure_pct: row.get(14)?,
                invalid_position_action_count: row.get(15)?,
                flat_sell_attempt_count: row.get(16)?,
                redundant_exit_count: row.get(17)?,
                engine_version: row.get(18)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(MarketPositionLifecycleReport {
        lifecycles,
        events,
        metrics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v42_position_sizing_maps_intents_deterministically() {
        let config = PositionSizingConfig::default();
        let enter_flat = generate_from_intent(PositionState::Flat, 0.0, PositionIntent::Enter, &config).unwrap();
        assert_eq!(enter_flat.lifecycle_action, LifecycleAction::EnterLong);
        assert_eq!(enter_flat.generated_target_exposure_pct, 25.0);
        let enter_long = generate_from_intent(PositionState::LongOpen, 25.0, PositionIntent::Enter, &config).unwrap();
        assert_eq!(enter_long.lifecycle_action, LifecycleAction::IncreaseLong);
        assert_eq!(enter_long.generated_target_exposure_pct, 35.0);
        let hold = generate_from_intent(PositionState::LongOpen, 40.0, PositionIntent::Hold, &config).unwrap();
        assert_eq!(hold.generated_target_exposure_pct, 40.0);
        let reduce = generate_from_intent(PositionState::LongOpen, 40.0, PositionIntent::Reduce, &config).unwrap();
        assert_eq!(reduce.lifecycle_action, LifecycleAction::ReduceLong);
        assert_eq!(reduce.generated_target_exposure_pct, 25.0);
        let exit = generate_from_intent(PositionState::LongOpen, 40.0, PositionIntent::Exit, &config).unwrap();
        assert_eq!(exit.lifecycle_action, LifecycleAction::ExitLong);
        assert_eq!(exit.generated_target_exposure_pct, 0.0);
    }

    #[test]
    fn v42_flat_rejects_reduce_and_exit_and_reduce_never_becomes_exit() {
        let config = PositionSizingConfig::default();
        for intent in [PositionIntent::Reduce, PositionIntent::Exit] {
            assert_eq!(
                generate_from_intent(PositionState::Flat, 0.0, intent, &config).unwrap_err(),
                "INVALID_POSITION_INTENT"
            );
        }
        let reduced = generate_from_intent(PositionState::LongReduced, 10.0, PositionIntent::Reduce, &config).unwrap();
        assert_eq!(reduced.generated_target_exposure_pct, 5.0);
        let saturated = generate_from_intent(PositionState::LongOpen, 100.0, PositionIntent::Enter, &config).unwrap();
        assert_eq!(saturated.lifecycle_action, LifecycleAction::HoldPosition);
        let tiny = generate_from_intent(PositionState::LongReduced, 0.02, PositionIntent::Reduce, &config).unwrap();
        assert_eq!(tiny.lifecycle_action, LifecycleAction::HoldPosition);
    }
    #[test]
    fn validates_state_and_target_semantics() {
        assert!(
            validate_decision(PositionState::Flat, 0.0, LifecycleAction::ExitLong, 0.0)
                .validation_code
                .is_some()
        );
        assert_eq!(
            validate_decision(PositionState::Flat, 0.0, LifecycleAction::EnterLong, 30.0)
                .effective_action,
            LifecycleAction::EnterLong
        );
        assert!(validate_decision(
            PositionState::LongOpen,
            50.0,
            LifecycleAction::ReduceLong,
            20.0
        )
        .validation_code
        .is_none());
        assert!(validate_decision(
            PositionState::LongOpen,
            20.0,
            LifecycleAction::IncreaseLong,
            40.0
        )
        .validation_code
        .is_none());
    }
    #[test]
    fn lifecycle_tracks_partial_events_and_excursions() {
        let mut manager = PositionManager::default();
        manager.execution(
            0,
            10,
            "T10",
            100.0,
            0.0,
            30.0,
            30.0,
            100.0,
            0.0,
            0.0,
            "ENTER_LONG",
            Some("ALIGNED_BULLISH_SIGNAL".into()),
            Some(0.8),
            Some("BULLISH".into()),
            "APPROVED",
            300.0,
        );
        manager.observe(102.0, 99.0);
        manager.execution(
            1,
            11,
            "T11",
            102.0,
            30.0,
            50.0,
            50.0,
            101.0,
            0.0,
            0.0,
            "INCREASE_LONG",
            None,
            None,
            None,
            "APPROVED",
            200.0,
        );
        manager.observe(110.0, 95.0);
        manager.execution(
            2,
            12,
            "T12",
            105.0,
            50.0,
            20.0,
            20.0,
            101.0,
            0.0,
            4.0,
            "REDUCE_LONG",
            None,
            None,
            None,
            "APPROVED",
            300.0,
        );
        manager.execution(
            3,
            20,
            "T20",
            108.0,
            20.0,
            0.0,
            0.0,
            0.0,
            4.0,
            8.0,
            "EXIT_LONG",
            Some("EXIT_SIGNAL".into()),
            Some(0.7),
            Some("BEARISH".into()),
            "APPROVED",
            200.0,
        );
        assert_eq!(manager.events.len(), 4);
        assert_eq!(manager.lifecycles.len(), 1);
        assert!((manager.lifecycles[0].mfe_pct - (110.0 / 101.0 * 100.0 - 100.0)).abs() < 1e-10);
        assert!((manager.lifecycles[0].mae_pct - (95.0 / 101.0 * 100.0 - 100.0)).abs() < 1e-10);
    }
}
