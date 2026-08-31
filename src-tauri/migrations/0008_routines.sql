CREATE TABLE routines (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  description TEXT NOT NULL DEFAULT '',
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  confirmation_required INTEGER NOT NULL DEFAULT 1 CHECK (confirmation_required IN (0, 1)),
  revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_routines_enabled_name ON routines(enabled, name);

CREATE TABLE routine_steps (
  id TEXT PRIMARY KEY,
  routine_id TEXT NOT NULL REFERENCES routines(id) ON DELETE CASCADE,
  step_order INTEGER NOT NULL CHECK (step_order > 0),
  action_id TEXT NOT NULL CHECK (action_id IN (
    'open_application',
    'open_workspace',
    'open_project',
    'reveal_workspace',
    'open_registered_url'
  )),
  target_type TEXT NOT NULL CHECK (target_type IN ('application', 'workspace', 'project', 'url')),
  target_id TEXT NOT NULL,
  delay_ms INTEGER NOT NULL DEFAULT 0 CHECK (delay_ms BETWEEN 0 AND 10000),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(routine_id, step_order)
);

CREATE INDEX idx_routine_steps_routine ON routine_steps(routine_id, step_order);

CREATE TABLE routine_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  routine_id TEXT REFERENCES routines(id) ON DELETE SET NULL,
  routine_name TEXT NOT NULL,
  routine_revision INTEGER NOT NULL,
  source TEXT NOT NULL CHECK (source IN ('user', 'ai', 'ui')),
  status TEXT NOT NULL CHECK (status IN (
    'waiting_confirmation', 'executing', 'completed', 'cancelled', 'failed'
  )),
  confirmation_required INTEGER NOT NULL DEFAULT 0 CHECK (confirmation_required IN (0, 1)),
  confirmed INTEGER NOT NULL DEFAULT 0 CHECK (confirmed IN (0, 1)),
  total_steps INTEGER NOT NULL,
  completed_steps INTEGER NOT NULL DEFAULT 0,
  failed_step INTEGER,
  error TEXT,
  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);

CREATE INDEX idx_routine_history_started ON routine_history(started_at DESC, id DESC);
CREATE INDEX idx_routine_history_routine ON routine_history(routine_id, started_at DESC);

ALTER TABLE action_history ADD COLUMN routine_history_id INTEGER REFERENCES routine_history(id) ON DELETE SET NULL;
ALTER TABLE action_history ADD COLUMN routine_step_order INTEGER;
CREATE INDEX idx_action_history_routine ON action_history(routine_history_id, routine_step_order);
