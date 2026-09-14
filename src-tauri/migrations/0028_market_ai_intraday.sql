INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES ('ai-intraday-v1','AI Intraday V1','llm','AI_INTRADAY_V1','{"provider":"ollama","model":"qwen2.5:3b","modelVersion":"qwen2.5:3b","promptVersion":"MARKET_AI_INTRADAY_V1","contextVersion":"MARKET_AI_INTRADAY_CONTEXT_V1","triggerVersion":"MARKET_DECISION_TRIGGER_V2","temperature":0.1,"seedSupported":false,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1,"timeframe":"15M","style":"INTRADAY","featureEngineVersion":"INTRADAY_FEATURE_ENGINE_V1","signalEngineVersion":"SIGNAL_ENGINE_V1","positionEngineVersion":"POSITION_ENGINE_V1","positionSizingVersion":"POSITION_SIZING_V1","riskPolicyVersion":"RISK_POLICY_V2","executionModelVersion":"EXECUTION_MODEL_V1"}',1);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-intraday-v1','ollama','qwen2.5:3b','MARKET_AI_INTRADAY_V1',0.1,5,30000,1);

CREATE TABLE market_ai_execution_metadata (
  decision_id INTEGER PRIMARY KEY REFERENCES market_decisions(id) ON DELETE CASCADE,
  model_version TEXT NOT NULL,
  temperature REAL NOT NULL,
  random_seed INTEGER NOT NULL,
  seed_supported INTEGER NOT NULL CHECK(seed_supported IN (0,1)),
  context_version TEXT NOT NULL,
  trigger_version TEXT NOT NULL,
  context_bytes INTEGER NOT NULL CHECK(context_bytes >= 0)
);

CREATE TABLE market_repeatability_groups (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  source_experiment_id TEXT NOT NULL REFERENCES market_experiments(id),
  repetitions INTEGER NOT NULL CHECK(repetitions BETWEEN 2 AND 5),
  frozen_config_json TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('running','completed','failed','aborted')),
  error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);

CREATE TABLE market_repeatability_runs (
  group_id TEXT NOT NULL REFERENCES market_repeatability_groups(id) ON DELETE CASCADE,
  run_index INTEGER NOT NULL CHECK(run_index >= 1),
  experiment_id TEXT NOT NULL UNIQUE REFERENCES market_experiments(id),
  PRIMARY KEY(group_id,run_index)
);

CREATE INDEX idx_market_ai_metadata_context ON market_ai_execution_metadata(context_version,trigger_version);
