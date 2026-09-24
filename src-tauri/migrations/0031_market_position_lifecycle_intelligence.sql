CREATE TABLE market_lifecycle_analysis_runs (
  id TEXT PRIMARY KEY,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL REFERENCES market_agents(id),
  lifecycle_engine_version TEXT NOT NULL,
  deterioration_engine_version TEXT NOT NULL,
  deterioration_config_json TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('running','completed','failed')),
  total_lifecycles INTEGER NOT NULL DEFAULT 0,
  late_reduction_events INTEGER NOT NULL DEFAULT 0,
  error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT,
  UNIQUE(experiment_id, agent_id, lifecycle_engine_version, deterioration_engine_version)
);

CREATE TABLE market_lifecycle_analyses (
  lifecycle_id TEXT PRIMARY KEY REFERENCES market_trade_lifecycles(id) ON DELETE CASCADE,
  run_id TEXT NOT NULL REFERENCES market_lifecycle_analysis_runs(id) ON DELETE CASCADE,
  asset TEXT NOT NULL,
  timeframe TEXT NOT NULL,
  entry_decision_timestamp TEXT,
  entry_execution_timestamp TEXT NOT NULL,
  entry_price REAL NOT NULL,
  initial_exposure_pct REAL NOT NULL,
  max_exposure_pct REAL NOT NULL,
  exit_decision_timestamp TEXT,
  exit_execution_timestamp TEXT,
  exit_price REAL,
  duration_candles INTEGER NOT NULL,
  duration_market_minutes INTEGER NOT NULL,
  overnight INTEGER NOT NULL CHECK(overnight IN (0,1)),
  status TEXT NOT NULL CHECK(status IN ('OPEN','CLOSED')),
  realized_pnl REAL NOT NULL,
  realized_pnl_pct REAL NOT NULL,
  mfe_pct REAL NOT NULL,
  mae_pct REAL NOT NULL,
  giveback_pct REAL NOT NULL,
  exit_efficiency_pct REAL,
  entry_reason TEXT,
  exit_reason TEXT,
  entry_atr REAL,
  entry_vwap REAL,
  entry_ema9 REAL,
  entry_ema21 REAL,
  entry_rsi REAL,
  entry_relative_volume REAL,
  entry_market_state_json TEXT,
  entry_session_phase TEXT,
  first_low_health_timestamp TEXT,
  first_moderate_deterioration_timestamp TEXT,
  first_high_deterioration_timestamp TEXT,
  response_delay_candles INTEGER,
  response_delay_market_minutes INTEGER,
  response_censored INTEGER NOT NULL DEFAULT 0 CHECK(response_censored IN (0,1))
);

CREATE TABLE market_lifecycle_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL REFERENCES market_lifecycle_analysis_runs(id) ON DELETE CASCADE,
  lifecycle_id TEXT NOT NULL REFERENCES market_trade_lifecycles(id) ON DELETE CASCADE,
  decision_id INTEGER REFERENCES market_decisions(id) ON DELETE SET NULL,
  execution_id INTEGER REFERENCES market_executions(id) ON DELETE SET NULL,
  timestamp TEXT NOT NULL,
  candle_index INTEGER NOT NULL,
  event_type TEXT NOT NULL CHECK(event_type IN ('OPEN','INCREASE','HOLD','REDUCE','EXIT','FORCED_EXIT')),
  state_before TEXT NOT NULL,
  state_after TEXT NOT NULL,
  position_before TEXT NOT NULL,
  position_after TEXT NOT NULL,
  exposure_before_pct REAL NOT NULL,
  exposure_after_pct REAL NOT NULL,
  market_price REAL NOT NULL,
  unrealized_pnl_pct REAL NOT NULL,
  mfe_so_far_pct REAL NOT NULL,
  mae_so_far_pct REAL NOT NULL,
  ai_intent TEXT,
  confidence REAL,
  reason_code TEXT,
  trigger_reason TEXT,
  risk_result TEXT NOT NULL,
  execution_result TEXT NOT NULL
);

CREATE TABLE market_lifecycle_health_trace (
  run_id TEXT NOT NULL REFERENCES market_lifecycle_analysis_runs(id) ON DELETE CASCADE,
  lifecycle_id TEXT NOT NULL REFERENCES market_trade_lifecycles(id) ON DELETE CASCADE,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  position_age_candles INTEGER NOT NULL,
  position_age_market_minutes INTEGER NOT NULL,
  market_price REAL NOT NULL,
  exposure_pct REAL NOT NULL,
  return_since_entry_pct REAL NOT NULL,
  mfe_since_entry_pct REAL NOT NULL,
  mae_since_entry_pct REAL NOT NULL,
  distance_from_entry_atr REAL,
  distance_from_mfe_pct REAL NOT NULL,
  giveback_from_mfe_pct REAL NOT NULL,
  giveback_relative_pct REAL,
  vwap_change_since_entry REAL,
  ema_spread_change_since_entry REAL,
  rsi_change_since_entry REAL,
  trend_changed INTEGER NOT NULL CHECK(trend_changed IN (0,1)),
  momentum_changed INTEGER NOT NULL CHECK(momentum_changed IN (0,1)),
  trend_component REAL NOT NULL,
  momentum_component REAL NOT NULL,
  giveback_component REAL NOT NULL,
  vwap_component REAL NOT NULL,
  volatility_component REAL NOT NULL,
  time_component REAL NOT NULL,
  deterioration_score REAL NOT NULL CHECK(deterioration_score BETWEEN 0 AND 1),
  deterioration_level TEXT NOT NULL CHECK(deterioration_level IN ('NONE','LOW','MODERATE','HIGH','CRITICAL')),
  position_health TEXT NOT NULL CHECK(position_health IN ('STRONG','HEALTHY','WEAKENING','DETERIORATING','CRITICAL')),
  available_at_t INTEGER NOT NULL DEFAULT 1 CHECK(available_at_t=1),
  PRIMARY KEY(run_id, lifecycle_id, candle_index)
);

CREATE TABLE market_lifecycle_post_decision_outcomes (
  run_id TEXT NOT NULL REFERENCES market_lifecycle_analysis_runs(id) ON DELETE CASCADE,
  lifecycle_id TEXT NOT NULL REFERENCES market_trade_lifecycles(id) ON DELETE CASCADE,
  candle_index INTEGER NOT NULL,
  decision_id INTEGER REFERENCES market_decisions(id) ON DELETE SET NULL,
  hold_quality TEXT,
  forward_return_1 REAL,
  forward_return_5 REAL,
  mfe_5 REAL,
  mae_5 REAL,
  missed_reduction_window INTEGER NOT NULL DEFAULT 0 CHECK(missed_reduction_window IN (0,1)),
  post_decision_only INTEGER NOT NULL DEFAULT 1 CHECK(post_decision_only=1),
  PRIMARY KEY(run_id, lifecycle_id, candle_index)
);

CREATE INDEX idx_market_lifecycle_runs_experiment
  ON market_lifecycle_analysis_runs(experiment_id, agent_id);
CREATE INDEX idx_market_lifecycle_events_timeline
  ON market_lifecycle_events(run_id, lifecycle_id, candle_index, id);
CREATE INDEX idx_market_lifecycle_health
  ON market_lifecycle_health_trace(run_id, position_health, deterioration_level);
CREATE INDEX idx_market_lifecycle_outcomes_quality
  ON market_lifecycle_post_decision_outcomes(run_id, hold_quality, missed_reduction_window);
