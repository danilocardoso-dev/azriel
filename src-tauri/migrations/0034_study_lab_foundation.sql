CREATE TABLE study_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  focus_minutes INTEGER NOT NULL DEFAULT 25 CHECK (focus_minutes BETWEEN 1 AND 240),
  short_break_minutes INTEGER NOT NULL DEFAULT 5 CHECK (short_break_minutes BETWEEN 1 AND 60),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO study_settings(id, focus_minutes, short_break_minutes)
VALUES (1, 25, 5);

CREATE TABLE study_sessions (
  id TEXT PRIMARY KEY,
  roadmap_id TEXT REFERENCES study_roadmaps(id) ON DELETE SET NULL,
  stage_id TEXT REFERENCES roadmap_stages(id) ON DELETE SET NULL,
  topic_id TEXT REFERENCES roadmap_topics(id) ON DELETE SET NULL,
  activity_id TEXT REFERENCES roadmap_activities(id) ON DELETE SET NULL,
  roadmap_name TEXT NOT NULL DEFAULT '',
  stage_name TEXT NOT NULL DEFAULT '',
  topic_name TEXT NOT NULL DEFAULT '',
  activity_title TEXT NOT NULL DEFAULT '',
  planned_focus_minutes INTEGER NOT NULL CHECK (planned_focus_minutes BETWEEN 1 AND 240),
  actual_focus_seconds INTEGER NOT NULL DEFAULT 0 CHECK (actual_focus_seconds >= 0),
  break_seconds INTEGER NOT NULL DEFAULT 0 CHECK (break_seconds >= 0),
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'PAUSED', 'COMPLETED', 'CANCELLED')),
  started_at TEXT NOT NULL,
  running_since TEXT,
  paused_at TEXT,
  ended_at TEXT,
  open_slot INTEGER,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (open_slot IS NULL OR open_slot = 1),
  CHECK (
    (status IN ('ACTIVE', 'PAUSED') AND open_slot = 1 AND ended_at IS NULL)
    OR
    (status IN ('COMPLETED', 'CANCELLED') AND open_slot IS NULL AND ended_at IS NOT NULL)
  ),
  CHECK (
    (status = 'ACTIVE' AND running_since IS NOT NULL AND paused_at IS NULL)
    OR
    (status = 'PAUSED' AND running_since IS NULL AND paused_at IS NOT NULL)
    OR
    (status IN ('COMPLETED', 'CANCELLED') AND running_since IS NULL AND paused_at IS NULL)
  )
);

CREATE UNIQUE INDEX idx_study_sessions_single_open
  ON study_sessions(open_slot)
  WHERE open_slot IS NOT NULL;

CREATE INDEX idx_study_sessions_started
  ON study_sessions(started_at DESC);

CREATE INDEX idx_study_sessions_roadmap_started
  ON study_sessions(roadmap_id, started_at DESC);

CREATE INDEX idx_study_sessions_status_updated
  ON study_sessions(status, updated_at DESC);
