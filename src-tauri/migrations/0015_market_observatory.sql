CREATE TABLE market_behavior_metrics (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  buy_count INTEGER NOT NULL,
  sell_count INTEGER NOT NULL,
  hold_count INTEGER NOT NULL,
  hold_rate_pct REAL NOT NULL,
  trade_frequency_pct REAL NOT NULL,
  average_exposure_pct REAL NOT NULL,
  max_exposure_pct REAL NOT NULL,
  average_position_size_pct REAL NOT NULL,
  max_position_size_pct REAL NOT NULL,
  average_holding_candles REAL NOT NULL,
  median_holding_candles REAL NOT NULL,
  max_holding_candles INTEGER NOT NULL,
  turnover_pct REAL NOT NULL,
  time_in_market_pct REAL NOT NULL,
  time_in_cash_pct REAL NOT NULL,
  entry_count INTEGER NOT NULL,
  exit_count INTEGER NOT NULL,
  risk_rejection_count INTEGER NOT NULL,
  risk_modification_count INTEGER NOT NULL,
  risk_rejection_rate_pct REAL NOT NULL,
  risk_modification_rate_pct REAL NOT NULL,
  drawdown_trigger_count INTEGER NOT NULL,
  daily_loss_trigger_count INTEGER NOT NULL,
  formula_version TEXT NOT NULL,
  computed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(experiment_id, agent_id)
);

CREATE TABLE market_position_episodes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  episode_index INTEGER NOT NULL,
  opened_candle_index INTEGER NOT NULL,
  opened_at TEXT NOT NULL,
  closed_candle_index INTEGER,
  closed_at TEXT,
  duration_candles INTEGER NOT NULL,
  max_exposure_pct REAL NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('open','closed')),
  UNIQUE(experiment_id, agent_id, episode_index)
);

CREATE TABLE market_agent_correlations (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_a_id TEXT NOT NULL,
  agent_b_id TEXT NOT NULL,
  equity_return_correlation REAL,
  decision_similarity REAL NOT NULL,
  high_similarity INTEGER NOT NULL CHECK(high_similarity IN (0,1)),
  similarity_threshold REAL NOT NULL,
  formula_version TEXT NOT NULL,
  PRIMARY KEY(experiment_id, agent_a_id, agent_b_id)
);

CREATE TABLE market_comparison_snapshots (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  cash_return_pct REAL,
  buy_hold_return_pct REAL,
  excess_vs_cash_pct REAL NOT NULL,
  excess_vs_buy_hold_pct REAL,
  computed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(experiment_id, agent_id)
);

CREATE INDEX idx_market_behavior_experiment ON market_behavior_metrics(experiment_id);
CREATE INDEX idx_market_episodes_experiment ON market_position_episodes(experiment_id, agent_id);
CREATE INDEX idx_market_correlations_experiment ON market_agent_correlations(experiment_id);
