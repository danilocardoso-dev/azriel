CREATE TABLE engineering_calibration (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  pinch_start_threshold REAL NOT NULL DEFAULT 0.045 CHECK (pinch_start_threshold BETWEEN 0.015 AND 0.100),
  pinch_release_threshold REAL NOT NULL DEFAULT 0.065 CHECK (pinch_release_threshold BETWEEN 0.025 AND 0.140),
  smoothing_alpha REAL NOT NULL DEFAULT 0.38 CHECK (smoothing_alpha BETWEEN 0.05 AND 1.0),
  rotation_sensitivity REAL NOT NULL DEFAULT 3.2 CHECK (rotation_sensitivity BETWEEN 0.25 AND 8.0),
  min_scale REAL NOT NULL DEFAULT 0.45 CHECK (min_scale BETWEEN 0.20 AND 1.0),
  max_scale REAL NOT NULL DEFAULT 2.4 CHECK (max_scale BETWEEN 1.0 AND 5.0),
  comfortable_hand_distance REAL NOT NULL DEFAULT 0.42 CHECK (comfortable_hand_distance BETWEEN 0.05 AND 1.0),
  calibrated INTEGER NOT NULL DEFAULT 0 CHECK (calibrated IN (0, 1)),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK (pinch_release_threshold > pinch_start_threshold),
  CHECK (max_scale > min_scale)
);

INSERT INTO engineering_calibration(id) VALUES (1);
