INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES ('ai-technical-v4-1','AI Technical V4.1','llm','AI_TECHNICAL_V4_1','{"provider":"ollama","model":"qwen2.5:3b","promptVersion":"MARKET_AI_AGENT_V4_1","temperature":0.1,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1,"signalEngineVersion":"SIGNAL_ENGINE_V1","signalConfigVersion":"SIGNAL_CONFIG_V1","positionEngineVersion":"POSITION_ENGINE_V1"}',1);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-technical-v4-1','ollama','qwen2.5:3b','MARKET_AI_AGENT_V4_1',0.1,5,30000,1);

ALTER TABLE market_ai_decisions ADD COLUMN first_failure_type TEXT;
ALTER TABLE market_ai_decisions ADD COLUMN fallback_reason TEXT;

CREATE TABLE market_ai_output_attempts(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  decision_id INTEGER NOT NULL REFERENCES market_ai_decisions(decision_id) ON DELETE CASCADE,
  attempt_number INTEGER NOT NULL CHECK(attempt_number BETWEEN 1 AND 2),
  raw_response TEXT,
  status TEXT NOT NULL CHECK(status IN ('VALID','INVALID','PROVIDER_ERROR')),
  error_type TEXT,
  error_field TEXT,
  error_message TEXT,
  normalized_from_wrapped_json INTEGER NOT NULL DEFAULT 0 CHECK(normalized_from_wrapped_json IN (0,1)),
  latency_ms INTEGER NOT NULL DEFAULT 0,
  UNIQUE(decision_id,attempt_number)
);

CREATE TABLE market_ai_validation_stages(
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  attempt_id INTEGER NOT NULL REFERENCES market_ai_output_attempts(id) ON DELETE CASCADE,
  stage_order INTEGER NOT NULL,
  stage TEXT NOT NULL,
  success INTEGER NOT NULL CHECK(success IN (0,1)),
  error_code TEXT,
  message TEXT,
  UNIQUE(attempt_id,stage_order)
);

CREATE INDEX idx_market_ai_attempts_decision ON market_ai_output_attempts(decision_id,attempt_number);
CREATE INDEX idx_market_ai_attempts_error ON market_ai_output_attempts(error_type);

ALTER TABLE market_ai_runtime_metrics ADD COLUMN first_pass_valid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN retry_recovered_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN final_valid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN final_invalid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN system_fallback_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN first_attempt_total_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN retry_total_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN p50_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN p95_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN slow_call_count INTEGER NOT NULL DEFAULT 0;

ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN first_pass_valid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN retry_recovered_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN final_valid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN final_invalid_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN system_fallback_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN first_attempt_total_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN retry_total_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN p50_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN p95_latency_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN slow_call_count INTEGER NOT NULL DEFAULT 0;
