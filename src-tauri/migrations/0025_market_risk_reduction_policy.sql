ALTER TABLE market_risk_evaluations ADD COLUMN exposure_change_class TEXT;
ALTER TABLE market_risk_evaluations ADD COLUMN current_exposure_pct REAL;
ALTER TABLE market_risk_evaluations ADD COLUMN target_exposure_pct REAL;
ALTER TABLE market_risk_evaluations ADD COLUMN exposure_delta_pct REAL;
ALTER TABLE market_risk_evaluations ADD COLUMN policy_version TEXT;

CREATE TABLE market_risk_rule_evaluations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  decision_id INTEGER NOT NULL REFERENCES market_decisions(id) ON DELETE CASCADE,
  rule_order INTEGER NOT NULL,
  rule TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('PASSED', 'REJECTED', 'NOT_APPLICABLE')),
  reason TEXT,
  UNIQUE(decision_id, rule_order)
);

CREATE INDEX idx_market_risk_rules_decision
  ON market_risk_rule_evaluations(decision_id, rule_order);
