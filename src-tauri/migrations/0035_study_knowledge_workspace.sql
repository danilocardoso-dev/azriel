CREATE TABLE study_notebooks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 160),
  description TEXT,
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'ARCHIVED')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE study_notes (
  id TEXT PRIMARY KEY,
  notebook_id TEXT NOT NULL REFERENCES study_notebooks(id) ON DELETE RESTRICT,
  title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 200),
  content TEXT NOT NULL DEFAULT '',
  content_format TEXT NOT NULL DEFAULT 'MARKDOWN' CHECK (content_format = 'MARKDOWN'),
  roadmap_id TEXT REFERENCES study_roadmaps(id) ON DELETE SET NULL,
  stage_id TEXT REFERENCES roadmap_stages(id) ON DELETE SET NULL,
  topic_id TEXT REFERENCES roadmap_topics(id) ON DELETE SET NULL,
  activity_id TEXT REFERENCES roadmap_activities(id) ON DELETE SET NULL,
  study_session_id TEXT REFERENCES study_sessions(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_study_notebooks_status_updated
  ON study_notebooks(status, updated_at DESC);

CREATE INDEX idx_study_notes_notebook_updated
  ON study_notes(notebook_id, updated_at DESC);

CREATE INDEX idx_study_notes_roadmap_updated
  ON study_notes(roadmap_id, updated_at DESC);

CREATE INDEX idx_study_notes_topic_updated
  ON study_notes(topic_id, updated_at DESC);

CREATE INDEX idx_study_notes_activity_updated
  ON study_notes(activity_id, updated_at DESC);

CREATE INDEX idx_study_notes_session_updated
  ON study_notes(study_session_id, updated_at DESC);

CREATE INDEX idx_study_notes_updated
  ON study_notes(updated_at DESC);
