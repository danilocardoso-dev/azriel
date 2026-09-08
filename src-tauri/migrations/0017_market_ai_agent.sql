INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES (
  'ai-technical-v1',
  'AI Technical V1',
  'llm',
  'AI_TECHNICAL_V1',
  '{"provider":"ollama","model":"qwen2.5:0.5b","promptVersion":"MARKET_AI_AGENT_V1","temperature":0.1,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1}',
  1
);

CREATE TABLE market_ai_agent_configs (
  agent_id TEXT PRIMARY KEY REFERENCES market_agents(id),
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  prompt_version TEXT NOT NULL,
  temperature REAL NOT NULL CHECK(temperature >= 0 AND temperature <= 0.3),
  decision_interval INTEGER NOT NULL CHECK(decision_interval BETWEEN 1 AND 100),
  timeout_ms INTEGER NOT NULL CHECK(timeout_ms BETWEEN 5000 AND 180000),
  max_retries INTEGER NOT NULL CHECK(max_retries BETWEEN 0 AND 1),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-technical-v1','ollama','qwen2.5:0.5b','MARKET_AI_AGENT_V1',0.1,5,30000,1);

CREATE TABLE market_ai_decisions (
  decision_id INTEGER PRIMARY KEY REFERENCES market_decisions(id) ON DELETE CASCADE,
  call_status TEXT NOT NULL CHECK(call_status IN ('VALID','INVALID','TIMEOUT','OFFLINE','NO_LLM_CALL')),
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  prompt_version TEXT NOT NULL,
  latency_ms INTEGER NOT NULL DEFAULT 0,
  attempts INTEGER NOT NULL DEFAULT 0,
  fallback_used INTEGER NOT NULL DEFAULT 0 CHECK(fallback_used IN (0,1))
);

CREATE TABLE market_ai_runtime_metrics (
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL REFERENCES market_agents(id),
  call_count INTEGER NOT NULL,
  successful_call_count INTEGER NOT NULL,
  invalid_response_count INTEGER NOT NULL,
  timeout_count INTEGER NOT NULL,
  retry_count INTEGER NOT NULL,
  fallback_count INTEGER NOT NULL,
  average_latency_ms REAL NOT NULL,
  max_latency_ms INTEGER NOT NULL,
  total_latency_ms INTEGER NOT NULL,
  input_tokens INTEGER,
  output_tokens INTEGER,
  PRIMARY KEY(experiment_id,agent_id)
);

CREATE TABLE market_validation_ai_runtime_metrics (
  validation_run_id TEXT NOT NULL REFERENCES market_validation_runs(id) ON DELETE CASCADE,
  agent_id TEXT NOT NULL REFERENCES market_agents(id),
  call_count INTEGER NOT NULL,
  successful_call_count INTEGER NOT NULL,
  invalid_response_count INTEGER NOT NULL,
  timeout_count INTEGER NOT NULL,
  retry_count INTEGER NOT NULL,
  fallback_count INTEGER NOT NULL,
  average_latency_ms REAL NOT NULL,
  max_latency_ms INTEGER NOT NULL,
  total_latency_ms INTEGER NOT NULL,
  input_tokens INTEGER,
  output_tokens INTEGER,
  PRIMARY KEY(validation_run_id,agent_id)
);

CREATE INDEX idx_market_ai_decisions_status ON market_ai_decisions(call_status);
