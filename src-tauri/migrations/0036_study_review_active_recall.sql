CREATE TABLE study_cards (
  id TEXT PRIMARY KEY,
  front TEXT NOT NULL CHECK (length(trim(front)) BETWEEN 1 AND 4000),
  back TEXT NOT NULL CHECK (length(trim(back)) BETWEEN 1 AND 12000),
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'SUSPENDED', 'ARCHIVED')),
  roadmap_id TEXT REFERENCES study_roadmaps(id) ON DELETE SET NULL,
  stage_id TEXT REFERENCES roadmap_stages(id) ON DELETE SET NULL,
  topic_id TEXT REFERENCES roadmap_topics(id) ON DELETE SET NULL,
  activity_id TEXT REFERENCES roadmap_activities(id) ON DELETE SET NULL,
  notebook_id TEXT REFERENCES study_notebooks(id) ON DELETE SET NULL,
  note_id TEXT REFERENCES study_notes(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE study_review_states (
  card_id TEXT PRIMARY KEY REFERENCES study_cards(id) ON DELETE CASCADE,
  due_at TEXT NOT NULL,
  last_reviewed_at TEXT,
  review_count INTEGER NOT NULL DEFAULT 0 CHECK (review_count >= 0),
  correct_count INTEGER NOT NULL DEFAULT 0 CHECK (correct_count >= 0),
  incorrect_count INTEGER NOT NULL DEFAULT 0 CHECK (incorrect_count >= 0),
  current_interval_seconds INTEGER NOT NULL DEFAULT 0 CHECK (current_interval_seconds >= 0),
  scheduler_version TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE study_review_sessions (
  id TEXT PRIMARY KEY,
  started_at TEXT NOT NULL,
  ended_at TEXT,
  status TEXT NOT NULL CHECK (status IN ('ACTIVE', 'COMPLETED', 'CANCELLED')),
  planned_cards INTEGER NOT NULL CHECK (planned_cards >= 0),
  reviewed_cards INTEGER NOT NULL DEFAULT 0 CHECK (reviewed_cards >= 0),
  study_session_id TEXT REFERENCES study_sessions(id) ON DELETE SET NULL,
  open_slot INTEGER,
  created_at TEXT NOT NULL,
  CHECK (open_slot IS NULL OR open_slot = 1),
  CHECK (
    (status = 'ACTIVE' AND open_slot = 1 AND ended_at IS NULL)
    OR
    (status IN ('COMPLETED', 'CANCELLED') AND open_slot IS NULL AND ended_at IS NOT NULL)
  )
);

CREATE UNIQUE INDEX idx_study_review_sessions_single_open
  ON study_review_sessions(open_slot)
  WHERE open_slot IS NOT NULL;

CREATE TABLE study_review_session_items (
  id TEXT PRIMARY KEY,
  review_session_id TEXT NOT NULL REFERENCES study_review_sessions(id) ON DELETE CASCADE,
  card_id TEXT NOT NULL REFERENCES study_cards(id) ON DELETE RESTRICT,
  item_order INTEGER NOT NULL CHECK (item_order >= 1),
  status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'REVIEWED', 'SKIPPED')),
  reviewed_at TEXT,
  UNIQUE(review_session_id, card_id),
  UNIQUE(review_session_id, item_order)
);

CREATE TABLE study_review_events (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES study_cards(id) ON DELETE RESTRICT,
  review_session_id TEXT NOT NULL REFERENCES study_review_sessions(id) ON DELETE RESTRICT,
  session_item_id TEXT NOT NULL UNIQUE REFERENCES study_review_session_items(id) ON DELETE RESTRICT,
  reviewed_at TEXT NOT NULL,
  result TEXT NOT NULL CHECK (result IN ('AGAIN', 'HARD', 'GOOD', 'EASY')),
  response_time_ms INTEGER CHECK (response_time_ms IS NULL OR response_time_ms BETWEEN 0 AND 86400000),
  previous_due_at TEXT NOT NULL,
  next_due_at TEXT NOT NULL,
  previous_interval_seconds INTEGER NOT NULL CHECK (previous_interval_seconds >= 0),
  next_interval_seconds INTEGER NOT NULL CHECK (next_interval_seconds >= 0),
  scheduler_version TEXT NOT NULL
);

CREATE INDEX idx_study_cards_status_updated
  ON study_cards(status, updated_at DESC);

CREATE INDEX idx_study_cards_note_updated
  ON study_cards(note_id, updated_at DESC);

CREATE INDEX idx_study_cards_roadmap_updated
  ON study_cards(roadmap_id, updated_at DESC);

CREATE INDEX idx_study_cards_activity_updated
  ON study_cards(activity_id, updated_at DESC);

CREATE INDEX idx_study_review_states_due
  ON study_review_states(due_at, card_id);

CREATE INDEX idx_study_review_events_card_reviewed
  ON study_review_events(card_id, reviewed_at DESC);

CREATE INDEX idx_study_review_events_session
  ON study_review_events(review_session_id, reviewed_at);

CREATE INDEX idx_study_review_sessions_started
  ON study_review_sessions(started_at DESC);
