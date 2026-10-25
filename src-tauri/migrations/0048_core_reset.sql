-- Core-only schema.  Applied on every startup so an existing installation is
-- normalized after the retired modules are removed.
CREATE TABLE IF NOT EXISTS tasks (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL,
  priority TEXT NOT NULL,
  due_date TEXT,
  project_id TEXT,
  knowledge_area_id TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  title TEXT,
  content TEXT NOT NULL,
  status TEXT NOT NULL,
  project_id TEXT,
  knowledge_area_id TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS tasks_core (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL,
  priority TEXT NOT NULL,
  due_date TEXT,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  completed_at TEXT
);
INSERT OR IGNORE INTO tasks_core(id,title,description,status,priority,due_date,created_at,updated_at,completed_at)
SELECT id,title,description,status,priority,due_date,created_at,updated_at,completed_at FROM tasks;
DROP TABLE tasks;
ALTER TABLE tasks_core RENAME TO tasks;

CREATE TABLE IF NOT EXISTS notes_core (
  id TEXT PRIMARY KEY,
  title TEXT,
  content TEXT NOT NULL,
  status TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO notes_core(id,title,content,status,created_at,updated_at)
SELECT id,title,content,status,created_at,updated_at FROM notes;
DROP TABLE notes;
ALTER TABLE notes_core RENAME TO notes;

CREATE TABLE IF NOT EXISTS ai_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  provider TEXT NOT NULL DEFAULT 'ollama',
  endpoint TEXT NOT NULL,
  model TEXT NOT NULL,
  context_message_limit INTEGER NOT NULL DEFAULT 12,
  timeout_seconds INTEGER NOT NULL DEFAULT 90,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
INSERT OR IGNORE INTO ai_settings(id, endpoint, model) VALUES (1, 'http://127.0.0.1:11434', '');
CREATE TABLE IF NOT EXISTS conversations (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE IF NOT EXISTS messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  role TEXT NOT NULL,
  content TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS ai_entity_references (
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  source_message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position BETWEEN 1 AND 50),
  entity_type TEXT NOT NULL CHECK (entity_type = 'task'),
  entity_id TEXT NOT NULL,
  entity_label TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (source_message_id, position)
);
CREATE TABLE IF NOT EXISTS task_action_history (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL,
  source_message_id TEXT NOT NULL,
  task_id TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ai_entity_references_conversation ON ai_entity_references(conversation_id, entity_type, source_message_id, position);
PRAGMA user_version = 48;
