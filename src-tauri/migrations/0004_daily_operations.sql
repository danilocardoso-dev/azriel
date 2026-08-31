CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL CHECK (status IN ('inbox', 'pending', 'in_progress', 'completed', 'cancelled')),
  priority TEXT NOT NULL CHECK (priority IN ('low', 'medium', 'high', 'critical')),
  due_date TEXT CHECK (due_date IS NULL OR (length(due_date) = 10 AND due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]')),
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  knowledge_area_id TEXT REFERENCES knowledge_areas(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);

CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  title TEXT,
  content TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active', 'archived')),
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  knowledge_area_id TEXT REFERENCES knowledge_areas(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_tasks_status_due ON tasks(status, due_date);
CREATE INDEX idx_tasks_completed_at ON tasks(completed_at DESC);
CREATE INDEX idx_tasks_project ON tasks(project_id);
CREATE INDEX idx_tasks_knowledge ON tasks(knowledge_area_id);
CREATE INDEX idx_notes_status_updated ON notes(status, updated_at DESC);
CREATE INDEX idx_notes_project ON notes(project_id);
CREATE INDEX idx_notes_knowledge ON notes(knowledge_area_id);
