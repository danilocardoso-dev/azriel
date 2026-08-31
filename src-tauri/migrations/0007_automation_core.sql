CREATE TABLE applications (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_applications_enabled ON applications(enabled, name);

CREATE TABLE registered_urls (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  url TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_registered_urls_enabled ON registered_urls(enabled, name);

ALTER TABLE workspaces ADD COLUMN application_id TEXT REFERENCES applications(id) ON DELETE SET NULL;
CREATE INDEX idx_workspaces_application ON workspaces(application_id);

CREATE TABLE action_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  action_id TEXT NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('user', 'ai', 'ui')),
  target_type TEXT,
  target_id TEXT,
  target_name TEXT,
  permission TEXT NOT NULL CHECK (permission IN ('read', 'safe_write', 'confirm_write', 'blocked')),
  confirmation_required INTEGER NOT NULL DEFAULT 0 CHECK (confirmation_required IN (0, 1)),
  confirmed INTEGER NOT NULL DEFAULT 0 CHECK (confirmed IN (0, 1)),
  success INTEGER CHECK (success IS NULL OR success IN (0, 1)),
  error TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);

CREATE INDEX idx_action_history_created ON action_history(created_at DESC, id DESC);
CREATE INDEX idx_action_history_action ON action_history(action_id, created_at DESC);
