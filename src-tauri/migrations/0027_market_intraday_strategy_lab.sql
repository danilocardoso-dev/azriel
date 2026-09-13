INSERT OR IGNORE INTO market_agents(id,name,strategy_type,strategy_version,default_config_json) VALUES
 ('intraday-trend-v1','Intraday Trend','intraday_trend','INTRADAY_TREND_V1','{"configVersion":"INTRADAY_TREND_CONFIG_V1","fastEma":9,"slowEma":21}'),
 ('intraday-momentum-v1','Intraday Momentum','intraday_momentum','INTRADAY_MOMENTUM_V1','{"configVersion":"INTRADAY_MOMENTUM_CONFIG_V1","breakoutLookback":8,"atrPeriod":14,"relativeVolumePeriod":20,"minimumRelativeVolume":1.2,"minimumRangeAtr":0.8}'),
 ('intraday-mean-reversion-v1','Intraday Mean Reversion','intraday_mean_reversion','INTRADAY_MEAN_REVERSION_V1','{"configVersion":"INTRADAY_MEAN_REVERSION_CONFIG_V1","rsiPeriod":14,"atrPeriod":14,"entryDistanceAtr":-1.0,"oversoldRsi":30,"exitRsi":55,"vwapReset":"SESSION"}');

CREATE TABLE market_intraday_feature_traces (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  candle_index INTEGER NOT NULL,
  timestamp_utc TEXT NOT NULL,
  session_id TEXT NOT NULL,
  session_phase TEXT NOT NULL,
  session_progress REAL NOT NULL,
  ready INTEGER NOT NULL CHECK(ready IN (0,1)),
  features_json TEXT NOT NULL,
  engine_version TEXT NOT NULL,
  PRIMARY KEY(experiment_id,candle_index)
);

CREATE TABLE market_intraday_strategy_decisions (
  decision_id INTEGER PRIMARY KEY REFERENCES market_decisions(id) ON DELETE CASCADE,
  style TEXT NOT NULL,
  intent TEXT NOT NULL,
  reason_code TEXT NOT NULL,
  reason TEXT NOT NULL,
  indicators_json TEXT NOT NULL,
  thresholds_json TEXT NOT NULL,
  position_before TEXT NOT NULL,
  generated_target_exposure_pct REAL NOT NULL,
  position_sizing_version TEXT NOT NULL
);

CREATE TABLE market_intraday_strategy_metrics (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  style TEXT NOT NULL,
  trend_entries INTEGER NOT NULL DEFAULT 0,
  ema_cross_entries INTEGER NOT NULL DEFAULT 0,
  avg_trend_duration_minutes REAL NOT NULL DEFAULT 0,
  breakout_attempts INTEGER NOT NULL DEFAULT 0,
  breakout_entries INTEGER NOT NULL DEFAULT 0,
  failed_breakouts INTEGER NOT NULL DEFAULT 0,
  vwap_deviation_events INTEGER NOT NULL DEFAULT 0,
  mean_reversion_entries INTEGER NOT NULL DEFAULT 0,
  successful_reversions INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(experiment_id,agent_id)
);

CREATE TABLE market_session_phase_performance (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  session_phase TEXT NOT NULL,
  trades INTEGER NOT NULL,
  return_pct REAL NOT NULL,
  win_rate_pct REAL NOT NULL,
  average_pnl REAL NOT NULL,
  PRIMARY KEY(experiment_id,agent_id,session_phase)
);

CREATE TABLE market_holding_time_distribution (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL,
  bucket TEXT NOT NULL,
  trade_count INTEGER NOT NULL,
  PRIMARY KEY(experiment_id,agent_id,bucket)
);

CREATE TABLE market_intraday_agent_overlap (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_a_id TEXT NOT NULL,
  agent_b_id TEXT NOT NULL,
  same_direction_decision_rate REAL NOT NULL,
  same_entry_window_count INTEGER NOT NULL,
  same_exit_window_count INTEGER NOT NULL,
  PRIMARY KEY(experiment_id,agent_a_id,agent_b_id)
);

CREATE INDEX idx_market_intraday_decisions_style ON market_intraday_strategy_decisions(style,reason_code);
CREATE INDEX idx_market_intraday_features_session ON market_intraday_feature_traces(experiment_id,session_id,candle_index);
