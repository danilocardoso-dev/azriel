INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES ('ai-technical-v3','AI Technical V3','llm','AI_TECHNICAL_V3','{"provider":"ollama","model":"qwen2.5:0.5b","promptVersion":"MARKET_AI_AGENT_V3","temperature":0.1,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1,"signalEngineVersion":"SIGNAL_ENGINE_V1","signalConfigVersion":"SIGNAL_CONFIG_V1"}',1);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-technical-v3','ollama','qwen2.5:0.5b','MARKET_AI_AGENT_V3',0.1,5,30000,1);

CREATE TABLE market_signal_engine_configs (
  version TEXT PRIMARY KEY,
  engine_version TEXT NOT NULL,
  config_json TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO market_signal_engine_configs(version,engine_version,config_json)
VALUES ('SIGNAL_CONFIG_V1','SIGNAL_ENGINE_V1','{"version":"SIGNAL_CONFIG_V1","trendWeight":0.45,"momentumWeight":0.45,"volatilityWeight":0.10,"strongSignalThreshold":0.65,"weakSignalThreshold":0.25,"conflictThreshold":0.50,"trendNormalizationPct":3.0,"momentumNormalizationPct":5.0,"highVolatility":0.03}');

ALTER TABLE market_ai_decisions ADD COLUMN reason_code TEXT;
ALTER TABLE market_ai_decisions ADD COLUMN validation_code TEXT;

CREATE TABLE market_ai_signal_traces (
  decision_id INTEGER PRIMARY KEY REFERENCES market_ai_decisions(decision_id) ON DELETE CASCADE,
  signal_engine_version TEXT NOT NULL,
  signal_config_version TEXT NOT NULL,
  signal_config_json TEXT NOT NULL,
  raw_features_json TEXT NOT NULL,
  trend_score REAL NOT NULL,
  momentum_score REAL NOT NULL,
  volatility_score REAL NOT NULL,
  bullish_evidence REAL NOT NULL,
  bearish_evidence REAL NOT NULL,
  signal_strength REAL NOT NULL,
  directional_bias TEXT NOT NULL CHECK(directional_bias IN ('BULLISH','BEARISH','NEUTRAL')),
  conflict_level TEXT NOT NULL CHECK(conflict_level IN ('LOW','MEDIUM','HIGH')),
  disagreement INTEGER NOT NULL DEFAULT 0 CHECK(disagreement IN (0,1))
);

CREATE INDEX idx_market_signal_trace_bias ON market_ai_signal_traces(directional_bias,signal_strength);
