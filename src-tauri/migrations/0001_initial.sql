CREATE TABLE knowledge_areas (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  category TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  coverage INTEGER NOT NULL DEFAULT 0 CHECK (coverage BETWEEN 0 AND 100),
  depth INTEGER NOT NULL DEFAULT 0 CHECK (depth BETWEEN 0 AND 100),
  priority TEXT NOT NULL CHECK (priority IN ('critical', 'high', 'medium', 'low')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  objective TEXT NOT NULL DEFAULT '',
  category TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('active', 'research', 'paused', 'planned', 'completed')),
  progress INTEGER NOT NULL DEFAULT 0 CHECK (progress BETWEEN 0 AND 100),
  next_step TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE project_knowledge (
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  knowledge_id TEXT NOT NULL REFERENCES knowledge_areas(id) ON DELETE CASCADE,
  PRIMARY KEY (project_id, knowledge_id)
);

CREATE TABLE education (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  type TEXT NOT NULL CHECK (type IN ('graduation', 'postgraduate', 'masters', 'doctorate', 'course', 'certification')),
  institution TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL CHECK (status IN ('completed', 'in_progress', 'planned')),
  start_date TEXT,
  expected_end_date TEXT,
  completed_at TEXT,
  description TEXT NOT NULL DEFAULT '',
  period TEXT NOT NULL DEFAULT '',
  domains TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE knowledge_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  knowledge_id TEXT NOT NULL REFERENCES knowledge_areas(id) ON DELETE CASCADE,
  coverage INTEGER NOT NULL CHECK (coverage BETWEEN 0 AND 100),
  depth INTEGER NOT NULL CHECK (depth BETWEEN 0 AND 100),
  recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  reason TEXT NOT NULL
);

CREATE TABLE app_metrics (
  key TEXT PRIMARY KEY,
  value REAL NOT NULL,
  formula_note TEXT NOT NULL DEFAULT '',
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_project_knowledge_knowledge ON project_knowledge(knowledge_id);
CREATE INDEX idx_knowledge_history_area_date ON knowledge_history(knowledge_id, recorded_at DESC);
CREATE INDEX idx_projects_status ON projects(status);
