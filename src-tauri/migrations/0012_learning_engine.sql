ALTER TABLE knowledge_events ADD COLUMN activity_type TEXT;
ALTER TABLE knowledge_events ADD COLUMN roadmap_id TEXT;
ALTER TABLE knowledge_events ADD COLUMN topic_id TEXT;
ALTER TABLE knowledge_events ADD COLUMN evidence_cycle INTEGER NOT NULL DEFAULT 0;
ALTER TABLE knowledge_events ADD COLUMN formula_version TEXT NOT NULL DEFAULT 'PRE_V1';
ALTER TABLE knowledge_events ADD COLUMN coverage_impact REAL NOT NULL DEFAULT 0;
ALTER TABLE knowledge_events ADD COLUMN depth_impact REAL NOT NULL DEFAULT 0;
ALTER TABLE knowledge_events ADD COLUMN integration_impact REAL NOT NULL DEFAULT 0;
ALTER TABLE knowledge_events ADD COLUMN metadata_json TEXT NOT NULL DEFAULT '{}';
ALTER TABLE knowledge_events ADD COLUMN reversal_of_event_id TEXT;
ALTER TABLE roadmap_activities ADD COLUMN project_id TEXT REFERENCES projects(id) ON DELETE SET NULL;
ALTER TABLE roadmap_activities ADD COLUMN research_id TEXT REFERENCES research_items(id) ON DELETE SET NULL;

CREATE TABLE activity_knowledge_nodes (
  activity_id TEXT NOT NULL REFERENCES roadmap_activities(id) ON DELETE CASCADE,
  knowledge_node_id TEXT NOT NULL REFERENCES knowledge_areas(id) ON DELETE RESTRICT,
  role TEXT NOT NULL CHECK (role IN ('primary', 'secondary')),
  PRIMARY KEY (activity_id, knowledge_node_id)
);

CREATE UNIQUE INDEX idx_activity_primary_knowledge
  ON activity_knowledge_nodes(activity_id) WHERE role = 'primary';
CREATE INDEX idx_activity_knowledge_node ON activity_knowledge_nodes(knowledge_node_id, activity_id);
INSERT OR IGNORE INTO activity_knowledge_nodes(activity_id, knowledge_node_id, role)
SELECT activity.id, topic.knowledge_node_id, 'primary'
FROM roadmap_activities activity
JOIN roadmap_topics topic ON topic.id = activity.topic_id
WHERE topic.knowledge_node_id IS NOT NULL;
CREATE UNIQUE INDEX idx_knowledge_event_cycle
  ON knowledge_events(source_type, source_id, event_type, knowledge_node_id, evidence_cycle)
  WHERE source_type = 'roadmap' AND source_id IS NOT NULL;
CREATE UNIQUE INDEX idx_knowledge_event_reversal
  ON knowledge_events(reversal_of_event_id) WHERE reversal_of_event_id IS NOT NULL;

CREATE TABLE learning_engine_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  formula_version TEXT NOT NULL,
  integration_baseline REAL NOT NULL CHECK (integration_baseline BETWEEN 0 AND 100),
  last_recalculated_at TEXT,
  status TEXT NOT NULL DEFAULT 'ready' CHECK (status IN ('ready', 'recalculating', 'error')),
  last_error TEXT
);

INSERT INTO learning_engine_state(id, formula_version, integration_baseline)
VALUES (1, 'LEARNING_ENGINE_V1', MAX(0, MIN(100, COALESCE((SELECT value FROM app_metrics WHERE key = 'integration'), 0))));
