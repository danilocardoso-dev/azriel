CREATE TABLE ai_entity_references (
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  source_message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position BETWEEN 1 AND 50),
  entity_type TEXT NOT NULL CHECK (entity_type = 'task'),
  entity_id TEXT NOT NULL,
  entity_label TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (source_message_id, position)
);

CREATE INDEX idx_ai_entity_references_conversation
  ON ai_entity_references(conversation_id, entity_type, source_message_id, position);

CREATE TABLE task_action_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
  task_title TEXT NOT NULL,
  action TEXT NOT NULL CHECK (action = 'completed'),
  source TEXT NOT NULL CHECK (source IN ('ui', 'user', 'ai')),
  conversation_id TEXT REFERENCES conversations(id) ON DELETE SET NULL,
  previous_status TEXT NOT NULL,
  new_status TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_task_action_history_task_date
  ON task_action_history(task_id, created_at DESC);
