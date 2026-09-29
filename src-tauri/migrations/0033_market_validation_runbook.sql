ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN reused_experiments INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN rebuilt_artifacts INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN new_llm_runs INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN failed_periods INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN skipped_periods INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN duration_ms INTEGER NOT NULL DEFAULT 0;
ALTER TABLE market_lifecycle_validation_batches
  ADD COLUMN audit_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN period_status TEXT NOT NULL DEFAULT 'PENDING'
    CHECK(period_status IN ('PENDING','RUNNING','COMPLETED','FAILED','SKIPPED'));
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN quality_status TEXT NOT NULL DEFAULT 'PENDING'
    CHECK(quality_status IN ('PENDING','PASS','PASS_WITH_WARNINGS','FAIL'));
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN quality_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN artifact_mode TEXT NOT NULL DEFAULT 'PENDING'
    CHECK(artifact_mode IN ('PENDING','REUSED','REBUILT'));
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN error TEXT;
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN started_at TEXT;
ALTER TABLE market_lifecycle_validation_periods
  ADD COLUMN completed_at TEXT;

CREATE TABLE market_lifecycle_validation_artifacts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  batch_id TEXT NOT NULL REFERENCES market_lifecycle_validation_batches(id) ON DELETE CASCADE,
  artifact_type TEXT NOT NULL CHECK(artifact_type IN (
    'REPORT_MD','REPORT_PDF','REPORT_HTML','REPORT_JSON','PERIOD_SUMMARY_CSV',
    'LIFECYCLE_SUMMARY_CSV','LATE_REDUCTION_EVENTS_CSV','COMPONENT_ANALYSIS_CSV',
    'EVIDENCE_MATRIX_CSV','WARNINGS_JSON'
  )),
  path TEXT NOT NULL,
  byte_size INTEGER NOT NULL CHECK(byte_size >= 0),
  checksum TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(batch_id, artifact_type)
);

CREATE INDEX idx_market_lifecycle_validation_period_status
  ON market_lifecycle_validation_periods(batch_id, period_status, quality_status);
CREATE INDEX idx_market_lifecycle_validation_artifacts_batch
  ON market_lifecycle_validation_artifacts(batch_id, artifact_type);
