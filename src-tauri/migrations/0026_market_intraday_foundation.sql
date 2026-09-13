ALTER TABLE market_datasets ADD COLUMN market TEXT NOT NULL DEFAULT 'UNSPECIFIED';
ALTER TABLE market_datasets ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
ALTER TABLE market_datasets ADD COLUMN session_type TEXT NOT NULL DEFAULT 'DAILY';
ALTER TABLE market_datasets ADD COLUMN session_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_datasets ADD COLUMN expected_gap_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_datasets ADD COLUMN unexpected_gap_count INTEGER NOT NULL DEFAULT 0;

ALTER TABLE market_candles ADD COLUMN timestamp_utc TEXT;
ALTER TABLE market_candles ADD COLUMN session_id TEXT;
ALTER TABLE market_candles ADD COLUMN session_state TEXT;

ALTER TABLE market_experiments ADD COLUMN market TEXT NOT NULL DEFAULT 'UNSPECIFIED';
ALTER TABLE market_experiments ADD COLUMN timezone TEXT NOT NULL DEFAULT 'UTC';
ALTER TABLE market_experiments ADD COLUMN session_type TEXT NOT NULL DEFAULT 'DAILY';
ALTER TABLE market_experiments ADD COLUMN annualization_factor REAL NOT NULL DEFAULT 252;
ALTER TABLE market_experiments ADD COLUMN execution_model_version TEXT NOT NULL DEFAULT 'EXECUTION_MODEL_V1';
ALTER TABLE market_experiments ADD COLUMN trigger_engine_version TEXT;
ALTER TABLE market_experiments ADD COLUMN trigger_config_json TEXT;

ALTER TABLE market_trade_lifecycles ADD COLUMN entry_session_id TEXT;
ALTER TABLE market_trade_lifecycles ADD COLUMN exit_session_id TEXT;
ALTER TABLE market_trade_lifecycles ADD COLUMN holding_market_minutes INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_trade_lifecycles ADD COLUMN overnight INTEGER NOT NULL DEFAULT 0 CHECK(overnight IN (0,1));

CREATE TABLE market_decision_trigger_audits (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  candle_index INTEGER NOT NULL,
  timestamp_utc TEXT NOT NULL,
  session_id TEXT NOT NULL,
  should_evaluate INTEGER NOT NULL CHECK(should_evaluate IN (0,1)),
  trigger_reason TEXT,
  skip_reason TEXT,
  cooldown_remaining INTEGER NOT NULL DEFAULT 0,
  call_index INTEGER,
  UNIQUE(experiment_id, agent_id, candle_index)
);

CREATE TABLE market_session_metrics (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  session_id TEXT NOT NULL,
  start_equity REAL NOT NULL,
  end_equity REAL NOT NULL,
  return_pct REAL NOT NULL,
  max_drawdown_pct REAL NOT NULL,
  trade_count INTEGER NOT NULL,
  ai_call_count INTEGER NOT NULL,
  trigger_count INTEGER NOT NULL,
  no_call_count INTEGER NOT NULL,
  PRIMARY KEY(experiment_id, agent_id, session_id)
);

CREATE INDEX idx_market_candles_session ON market_candles(dataset_id, session_id, candle_index);
CREATE INDEX idx_market_trigger_audit ON market_decision_trigger_audits(experiment_id, agent_id, candle_index);
CREATE INDEX idx_market_session_metrics ON market_session_metrics(experiment_id, agent_id, session_id);
