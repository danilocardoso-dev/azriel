CREATE TABLE market_dataset_identity_repairs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  dataset_id TEXT NOT NULL UNIQUE REFERENCES market_datasets(id) ON DELETE CASCADE,
  previous_asset TEXT NOT NULL,
  normalized_asset TEXT NOT NULL,
  evidence TEXT NOT NULL,
  repaired_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_market_dataset_identity_repairs_asset
  ON market_dataset_identity_repairs(normalized_asset, repaired_at);
