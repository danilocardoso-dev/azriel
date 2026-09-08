INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES (
  'ai-technical-v2',
  'AI Technical V2',
  'llm',
  'AI_TECHNICAL_V2',
  '{"provider":"ollama","model":"qwen2.5:0.5b","promptVersion":"MARKET_AI_AGENT_V2","temperature":0.1,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1}',
  1
);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-technical-v2','ollama','qwen2.5:0.5b','MARKET_AI_AGENT_V2',0.1,5,30000,1);

ALTER TABLE market_ai_runtime_metrics ADD COLUMN prompt_version TEXT NOT NULL DEFAULT 'MARKET_AI_AGENT_V1';
ALTER TABLE market_ai_runtime_metrics ADD COLUMN buy_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN sell_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN llm_hold_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN no_llm_call_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN average_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN average_buy_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN average_sell_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN average_hold_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN min_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN max_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN median_confidence REAL;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN confidence_00_20 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN confidence_20_40 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN confidence_40_60 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN confidence_60_80 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_ai_runtime_metrics ADD COLUMN confidence_80_100 INTEGER NOT NULL DEFAULT 0;

ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN prompt_version TEXT NOT NULL DEFAULT 'MARKET_AI_AGENT_V1';
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN buy_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN sell_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN llm_hold_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN no_llm_call_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN average_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN average_buy_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN average_sell_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN average_hold_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN min_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN max_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN median_confidence REAL;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN confidence_00_20 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN confidence_20_40 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN confidence_40_60 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN confidence_60_80 INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_validation_ai_runtime_metrics ADD COLUMN confidence_80_100 INTEGER NOT NULL DEFAULT 0;

CREATE TABLE market_ai_decision_context (
  decision_id INTEGER PRIMARY KEY REFERENCES market_ai_decisions(decision_id) ON DELETE CASCADE,
  input_snapshot_json TEXT NOT NULL
);

CREATE TABLE market_ai_decision_evaluations (
  decision_id INTEGER PRIMARY KEY REFERENCES market_ai_decisions(decision_id) ON DELETE CASCADE,
  forward_return_1 REAL,
  forward_return_5 REAL,
  forward_return_10 REAL
);

CREATE INDEX idx_market_ai_decisions_prompt ON market_ai_decisions(prompt_version);
