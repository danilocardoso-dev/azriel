CREATE TABLE market_position_sizing_configs(
  version TEXT PRIMARY KEY,
  default_entry_exposure_pct REAL NOT NULL CHECK(default_entry_exposure_pct > 0 AND default_entry_exposure_pct <= 100),
  default_increase_step_pct REAL NOT NULL CHECK(default_increase_step_pct > 0 AND default_increase_step_pct <= 100),
  default_reduce_step_pct REAL NOT NULL CHECK(default_reduce_step_pct > 0 AND default_reduce_step_pct <= 100),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO market_position_sizing_configs(version,default_entry_exposure_pct,default_increase_step_pct,default_reduce_step_pct)
VALUES ('POSITION_SIZING_V1',25,10,15);

INSERT INTO market_agents(id,name,strategy_type,strategy_version,default_config_json,enabled)
VALUES ('ai-technical-v4-2','AI Technical V4.2','llm','AI_TECHNICAL_V4_2','{"provider":"ollama","model":"qwen2.5:3b","promptVersion":"MARKET_AI_AGENT_V4_2","temperature":0.1,"decisionInterval":5,"timeoutMs":30000,"maxRetries":1,"signalEngineVersion":"SIGNAL_ENGINE_V1","signalConfigVersion":"SIGNAL_CONFIG_V1","positionEngineVersion":"POSITION_ENGINE_V1","positionSizingVersion":"POSITION_SIZING_V1","defaultEntryExposurePct":25,"defaultIncreaseStepPct":10,"defaultReduceStepPct":15}',1);

INSERT INTO market_ai_agent_configs(agent_id,provider,model,prompt_version,temperature,decision_interval,timeout_ms,max_retries)
VALUES ('ai-technical-v4-2','ollama','qwen2.5:3b','MARKET_AI_AGENT_V4_2',0.1,5,30000,1);

ALTER TABLE market_ai_decisions ADD COLUMN intent TEXT;
ALTER TABLE market_ai_decisions ADD COLUMN generated_target_exposure_pct REAL;
ALTER TABLE market_ai_decisions ADD COLUMN position_sizing_version TEXT;
