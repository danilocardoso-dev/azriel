CREATE TABLE ai_settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  provider TEXT NOT NULL DEFAULT 'ollama' CHECK (provider = 'ollama'),
  endpoint TEXT NOT NULL DEFAULT 'http://localhost:11434',
  model TEXT NOT NULL DEFAULT 'qwen2.5:0.5b',
  context_message_limit INTEGER NOT NULL DEFAULT 6 CHECK (context_message_limit BETWEEN 1 AND 20),
  timeout_seconds INTEGER NOT NULL DEFAULT 45 CHECK (timeout_seconds BETWEEN 5 AND 180),
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO ai_settings(id) VALUES (1);

CREATE TABLE conversations (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE messages (
  id TEXT PRIMARY KEY,
  conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
  role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
  content TEXT NOT NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);
CREATE INDEX idx_messages_conversation_date ON messages(conversation_id, created_at, id);
