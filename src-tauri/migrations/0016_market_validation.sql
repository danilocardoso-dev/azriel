CREATE TABLE market_validation_runs (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  dataset_id TEXT NOT NULL REFERENCES market_datasets(id),
  dataset_hash TEXT NOT NULL,
  risk_profile_id TEXT NOT NULL REFERENCES market_risk_profiles(id),
  agent_ids_json TEXT NOT NULL,
  frozen_config_json TEXT NOT NULL,
  split_config_json TEXT NOT NULL,
  walk_forward_config_json TEXT NOT NULL,
  regime_config_json TEXT NOT NULL,
  rolling_window INTEGER NOT NULL,
  annualization_factor REAL NOT NULL,
  validation_engine_version TEXT NOT NULL,
  metric_formula_version TEXT NOT NULL,
  regime_engine_version TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('draft','running','completed','failed','aborted')),
  error TEXT,
  started_at TEXT,
  completed_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE market_validation_windows (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  window_index INTEGER NOT NULL,
  window_type TEXT NOT NULL CHECK(window_type IN ('in_sample','validation','out_of_sample','walk_forward')),
  start_index INTEGER NOT NULL,
  end_index INTEGER NOT NULL,
  start_at TEXT NOT NULL,
  end_at TEXT NOT NULL,
  train_start_index INTEGER,
  train_end_index INTEGER,
  train_start_at TEXT,
  train_end_at TEXT,
  UNIQUE(validation_run_id, window_type, window_index)
);

CREATE TABLE market_agent_validation_metrics (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  window_id INTEGER NOT NULL REFERENCES market_validation_windows(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  total_return_pct REAL NOT NULL,
  max_drawdown_pct REAL NOT NULL,
  sharpe REAL,
  sortino REAL,
  calmar REAL,
  trade_count INTEGER NOT NULL,
  hold_rate_pct REAL NOT NULL,
  exposure_pct REAL NOT NULL,
  benchmark_cash_excess_pct REAL NOT NULL,
  benchmark_buy_hold_excess_pct REAL NOT NULL,
  formula_version TEXT NOT NULL,
  PRIMARY KEY(validation_run_id, window_id, agent_id)
);

CREATE TABLE market_regimes (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  trend_regime TEXT NOT NULL CHECK(trend_regime IN ('bull','bear','sideways')),
  volatility_regime TEXT NOT NULL CHECK(volatility_regime IN ('high_volatility','normal_volatility','low_volatility')),
  formula_version TEXT NOT NULL,
  PRIMARY KEY(validation_run_id, candle_index)
);

CREATE TABLE market_regime_metrics (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  regime_type TEXT NOT NULL CHECK(regime_type IN ('trend','volatility')),
  regime TEXT NOT NULL,
  candle_count INTEGER NOT NULL,
  total_return_pct REAL NOT NULL,
  max_drawdown_pct REAL NOT NULL,
  trade_count INTEGER NOT NULL,
  hold_rate_pct REAL NOT NULL,
  exposure_pct REAL NOT NULL,
  profit_factor REAL,
  win_rate_pct REAL,
  low_sample_size INTEGER NOT NULL CHECK(low_sample_size IN (0,1)),
  PRIMARY KEY(validation_run_id, agent_id, regime_type, regime)
);

CREATE TABLE market_rolling_metrics (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  rolling_return_pct REAL,
  rolling_volatility_pct REAL,
  rolling_sharpe REAL,
  rolling_drawdown_pct REAL,
  window_size INTEGER NOT NULL,
  formula_version TEXT NOT NULL,
  PRIMARY KEY(validation_run_id, agent_id, candle_index)
);

CREATE TABLE market_robustness_reports (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  in_sample_return_pct REAL NOT NULL,
  validation_return_pct REAL NOT NULL,
  out_of_sample_return_pct REAL NOT NULL,
  out_of_sample_drawdown_pct REAL NOT NULL,
  positive_window_ratio_pct REAL NOT NULL,
  average_window_return_pct REAL NOT NULL,
  median_window_return_pct REAL NOT NULL,
  best_window_return_pct REAL NOT NULL,
  worst_window_return_pct REAL NOT NULL,
  return_std_across_windows REAL NOT NULL,
  drawdown_std_across_windows REAL NOT NULL,
  benchmark_excess_pct REAL NOT NULL,
  overfitting_gap_pct REAL NOT NULL,
  possible_overfitting INTEGER NOT NULL CHECK(possible_overfitting IN (0,1)),
  robustness_status TEXT NOT NULL CHECK(robustness_status IN ('insufficient_data','unstable','mixed','robust_candidate')),
  rule_version TEXT NOT NULL,
  PRIMARY KEY(validation_run_id, agent_id)
);

CREATE INDEX idx_market_validation_dataset ON market_validation_runs(dataset_id, created_at);
CREATE INDEX idx_market_validation_windows ON market_validation_windows(validation_run_id, window_type, window_index);
CREATE INDEX idx_market_regimes_run ON market_regimes(validation_run_id, candle_index);
CREATE INDEX idx_market_rolling_run ON market_rolling_metrics(validation_run_id, agent_id, candle_index);
