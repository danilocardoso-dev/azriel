CREATE TABLE market_datasets (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  asset TEXT NOT NULL,
  timeframe TEXT NOT NULL,
  currency TEXT NOT NULL DEFAULT 'BRL',
  start_at TEXT NOT NULL,
  end_at TEXT NOT NULL,
  candle_count INTEGER NOT NULL CHECK (candle_count > 0),
  fingerprint TEXT NOT NULL UNIQUE,
  source_path TEXT NOT NULL,
  imported_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE market_candles (
  dataset_id TEXT NOT NULL REFERENCES market_datasets(id) ON DELETE CASCADE,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  open REAL NOT NULL,
  high REAL NOT NULL,
  low REAL NOT NULL,
  close REAL NOT NULL,
  volume REAL,
  PRIMARY KEY(dataset_id, candle_index),
  UNIQUE(dataset_id, timestamp)
);

CREATE TABLE market_agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  strategy_type TEXT NOT NULL,
  strategy_version TEXT NOT NULL,
  default_config_json TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0,1))
);

CREATE TABLE market_risk_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  max_position_pct REAL NOT NULL,
  max_total_exposure_pct REAL NOT NULL,
  max_daily_loss_pct REAL NOT NULL,
  max_drawdown_pct REAL NOT NULL,
  max_trades_per_day INTEGER,
  allow_leverage INTEGER NOT NULL DEFAULT 0 CHECK (allow_leverage IN (0,1)),
  allow_short INTEGER NOT NULL DEFAULT 0 CHECK (allow_short IN (0,1)),
  allowed_assets_json TEXT NOT NULL DEFAULT '[]'
);

CREATE TABLE market_experiments (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  dataset_id TEXT NOT NULL REFERENCES market_datasets(id),
  asset TEXT NOT NULL,
  timeframe TEXT NOT NULL,
  currency TEXT NOT NULL,
  initial_capital REAL NOT NULL,
  risk_profile_id TEXT NOT NULL REFERENCES market_risk_profiles(id),
  random_seed INTEGER NOT NULL,
  fee_pct REAL NOT NULL,
  slippage_pct REAL NOT NULL,
  config_json TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending','running','completed','failed','aborted')),
  error TEXT,
  started_at TEXT,
  completed_at TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE market_experiment_agents (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL REFERENCES market_agents(id),
  config_json TEXT NOT NULL,
  status TEXT NOT NULL,
  error TEXT,
  PRIMARY KEY(experiment_id, agent_id)
);

CREATE TABLE market_decisions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  observed_price REAL NOT NULL,
  action TEXT NOT NULL,
  desired_position_pct REAL,
  confidence REAL,
  reasoning TEXT NOT NULL,
  cash_before REAL NOT NULL,
  equity_before REAL NOT NULL,
  exposure_before REAL NOT NULL
);

CREATE TABLE market_risk_evaluations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  decision_id INTEGER NOT NULL REFERENCES market_decisions(id) ON DELETE CASCADE,
  result TEXT NOT NULL,
  reason TEXT NOT NULL,
  requested_position_pct REAL,
  approved_position_pct REAL,
  risk_state_json TEXT NOT NULL
);

CREATE TABLE market_orders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  decision_id INTEGER NOT NULL REFERENCES market_decisions(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  asset TEXT NOT NULL,
  side TEXT NOT NULL,
  quantity REAL NOT NULL,
  requested_price REAL NOT NULL,
  status TEXT NOT NULL
);

CREATE TABLE market_executions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  order_id INTEGER NOT NULL REFERENCES market_orders(id) ON DELETE CASCADE,
  timestamp TEXT NOT NULL,
  execution_price REAL NOT NULL,
  quantity REAL NOT NULL,
  gross_value REAL NOT NULL,
  fees REAL NOT NULL,
  slippage REAL NOT NULL,
  net_value REAL NOT NULL
);

CREATE TABLE market_portfolio_snapshots (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  candle_index INTEGER NOT NULL,
  timestamp TEXT NOT NULL,
  cash REAL NOT NULL,
  quantity REAL NOT NULL,
  market_value REAL NOT NULL,
  equity REAL NOT NULL,
  exposure_pct REAL NOT NULL,
  realized_pnl REAL NOT NULL,
  unrealized_pnl REAL NOT NULL
);

CREATE TABLE market_agent_metrics (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  final_equity REAL NOT NULL,
  total_return_pct REAL NOT NULL,
  max_drawdown_pct REAL NOT NULL,
  decision_count INTEGER NOT NULL,
  hold_count INTEGER NOT NULL,
  trade_count INTEGER NOT NULL,
  win_rate_pct REAL NOT NULL,
  profit_factor REAL,
  realized_pnl REAL NOT NULL,
  unrealized_pnl REAL NOT NULL,
  average_exposure_pct REAL NOT NULL,
  max_exposure_pct REAL NOT NULL,
  PRIMARY KEY(experiment_id, agent_id)
);

CREATE INDEX idx_market_candles_dataset ON market_candles(dataset_id, candle_index);
CREATE INDEX idx_market_decisions_experiment ON market_decisions(experiment_id, agent_id, candle_index);
CREATE INDEX idx_market_snapshots_experiment ON market_portfolio_snapshots(experiment_id, agent_id, candle_index);

INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json) VALUES
 ('cash','Cash Control','cash','1.0.0','{}'),
 ('buy-hold','Buy & Hold','buy_hold','1.0.0','{"desiredPositionPct":100}'),
 ('simple-trend','Simple Trend','simple_trend','1.0.0','{"shortSmaPeriod":5,"longSmaPeriod":20,"desiredPositionPct":80}'),
 ('simple-momentum','Simple Momentum','simple_momentum','1.0.0','{"momentumPeriod":5,"desiredPositionPct":60}'),
 ('random-controlled','Random Controlled','random_controlled','1.0.0','{"desiredPositionPct":40}');

INSERT INTO market_risk_profiles(id,name,max_position_pct,max_total_exposure_pct,max_daily_loss_pct,max_drawdown_pct,max_trades_per_day,allow_leverage,allow_short,allowed_assets_json)
VALUES ('balanced-v1','Balanced Lab v1',50,50,5,20,10,0,0,'[]');
