CREATE TABLE market_lifecycle_validation_batches (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  asset TEXT NOT NULL,
  timeframe TEXT NOT NULL,
  agent_id TEXT NOT NULL REFERENCES market_agents(id),
  agent_version TEXT NOT NULL,
  lifecycle_config_version TEXT NOT NULL,
  deterioration_config_version TEXT NOT NULL,
  hold_diagnostics_version TEXT NOT NULL,
  execution_model_version TEXT NOT NULL,
  dataset_count INTEGER NOT NULL DEFAULT 0,
  experiment_count INTEGER NOT NULL DEFAULT 0,
  lifecycle_count INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL CHECK(status IN ('CREATED','VALIDATING','COMPLETED','FAILED','PARTIAL')),
  config_json TEXT NOT NULL,
  result_json TEXT,
  warnings_json TEXT NOT NULL DEFAULT '[]',
  evidence_matrix_json TEXT,
  error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  started_at TEXT,
  completed_at TEXT
);

CREATE TABLE market_lifecycle_validation_periods (
  id TEXT PRIMARY KEY,
  batch_id TEXT NOT NULL REFERENCES market_lifecycle_validation_batches(id) ON DELETE CASCADE,
  experiment_id TEXT NOT NULL REFERENCES market_experiments(id),
  dataset_id TEXT NOT NULL REFERENCES market_datasets(id),
  source_role TEXT NOT NULL CHECK(source_role IN ('DEVELOPMENT','OOS','HOLDOUT','ADDITIONAL_VALIDATION')),
  asset TEXT NOT NULL,
  timeframe TEXT NOT NULL,
  start_at TEXT NOT NULL,
  end_at TEXT NOT NULL,
  candle_count INTEGER NOT NULL,
  session_count INTEGER NOT NULL,
  lifecycle_count INTEGER NOT NULL DEFAULT 0,
  lifecycle_run_id TEXT,
  overlap_status TEXT NOT NULL CHECK(overlap_status IN ('NON_OVERLAPPING','TOUCHING_BOUNDARY','OVERLAPPING','DUPLICATE_RANGE')),
  summary_json TEXT NOT NULL,
  UNIQUE(batch_id, experiment_id)
);

CREATE TABLE market_lifecycle_validation_lifecycles (
  batch_id TEXT NOT NULL REFERENCES market_lifecycle_validation_batches(id) ON DELETE CASCADE,
  period_id TEXT NOT NULL REFERENCES market_lifecycle_validation_periods(id) ON DELETE CASCADE,
  lifecycle_id TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('OPEN','CLOSED')),
  entry_at TEXT NOT NULL,
  exit_at TEXT,
  duration_minutes INTEGER NOT NULL,
  realized_pnl_pct REAL,
  mfe_pct REAL NOT NULL,
  mae_pct REAL NOT NULL,
  giveback_pct REAL NOT NULL,
  worst_health TEXT NOT NULL,
  dominant_health TEXT NOT NULL,
  max_deterioration_score REAL NOT NULL,
  max_deterioration_level TEXT NOT NULL,
  first_timestamp_at_max_level TEXT,
  late_reduction_events INTEGER NOT NULL DEFAULT 0,
  has_late_reduction INTEGER NOT NULL DEFAULT 0 CHECK(has_late_reduction IN (0,1)),
  response_delay_minutes INTEGER,
  response_status TEXT NOT NULL CHECK(response_status IN ('RESOLVED','UNRESOLVED','NOT_APPLICABLE')),
  response_from_first_deterioration_minutes INTEGER,
  response_from_high_minutes INTEGER,
  response_from_critical_minutes INTEGER,
  overnight INTEGER NOT NULL CHECK(overnight IN (0,1)),
  censored_at_dataset_end INTEGER NOT NULL CHECK(censored_at_dataset_end IN (0,1)),
  health_distribution_json TEXT NOT NULL,
  components_json TEXT NOT NULL,
  PRIMARY KEY(batch_id, period_id, lifecycle_id)
);

CREATE INDEX idx_market_lifecycle_validation_batches_created
  ON market_lifecycle_validation_batches(created_at DESC);
CREATE INDEX idx_market_lifecycle_validation_periods_batch
  ON market_lifecycle_validation_periods(batch_id, start_at, end_at);
CREATE INDEX idx_market_lifecycle_validation_lifecycles_batch
  ON market_lifecycle_validation_lifecycles(batch_id, status, max_deterioration_level, has_late_reduction);
