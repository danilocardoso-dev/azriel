ALTER TABLE knowledge_areas ADD COLUMN node_type TEXT NOT NULL DEFAULT 'area'
  CHECK (node_type IN ('area', 'discipline', 'topic', 'competency'));
ALTER TABLE knowledge_areas ADD COLUMN parent_id TEXT REFERENCES knowledge_areas(id) ON DELETE SET NULL;
CREATE INDEX idx_knowledge_parent ON knowledge_areas(parent_id, node_type, name);

CREATE TABLE knowledge_baselines (
  knowledge_id TEXT PRIMARY KEY REFERENCES knowledge_areas(id) ON DELETE CASCADE,
  coverage INTEGER NOT NULL CHECK (coverage BETWEEN 0 AND 100),
  depth INTEGER NOT NULL CHECK (depth BETWEEN 0 AND 100),
  recorded_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO knowledge_baselines(knowledge_id, coverage, depth, recorded_at)
SELECT id, coverage, depth, COALESCE(created_at, CURRENT_TIMESTAMP)
FROM knowledge_areas;

CREATE TABLE study_roadmaps (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'planned'
    CHECK (status IN ('planned', 'active', 'paused', 'completed')),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE roadmap_stages (
  id TEXT PRIMARY KEY,
  roadmap_id TEXT NOT NULL REFERENCES study_roadmaps(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  stage_order INTEGER NOT NULL CHECK (stage_order > 0),
  UNIQUE(roadmap_id, stage_order)
);

CREATE TABLE roadmap_topics (
  id TEXT PRIMARY KEY,
  stage_id TEXT NOT NULL REFERENCES roadmap_stages(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  knowledge_node_id TEXT REFERENCES knowledge_areas(id) ON DELETE SET NULL,
  topic_state TEXT NOT NULL DEFAULT 'NOT_STARTED'
    CHECK (topic_state IN ('NOT_STARTED', 'EXPOSED', 'UNDERSTOOD', 'PRACTICED', 'APPLIED', 'MASTERED')),
  topic_order INTEGER NOT NULL CHECK (topic_order > 0),
  UNIQUE(stage_id, topic_order)
);

CREATE TABLE roadmap_activities (
  id TEXT PRIMARY KEY,
  topic_id TEXT NOT NULL REFERENCES roadmap_topics(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  activity_type TEXT NOT NULL DEFAULT 'OTHER'
    CHECK (activity_type IN ('READING', 'LESSON', 'QUIZ', 'EXERCISE', 'SIMULATION', 'EXPERIMENT', 'PROJECT', 'DOCUMENTATION', 'RESEARCH', 'OTHER')),
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'in_progress', 'completed')),
  completed_at TEXT,
  activity_order INTEGER NOT NULL CHECK (activity_order > 0),
  UNIQUE(topic_id, activity_order)
);

CREATE INDEX idx_roadmap_stages_roadmap ON roadmap_stages(roadmap_id, stage_order);
CREATE INDEX idx_roadmap_topics_stage ON roadmap_topics(stage_id, topic_order);
CREATE INDEX idx_roadmap_topics_knowledge ON roadmap_topics(knowledge_node_id);
CREATE INDEX idx_roadmap_activities_topic ON roadmap_activities(topic_id, activity_order);

CREATE TABLE research_items (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL UNIQUE,
  domain TEXT NOT NULL DEFAULT '',
  objective TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  kind TEXT NOT NULL DEFAULT 'research' CHECK (kind IN ('project', 'study', 'research')),
  status TEXT NOT NULL DEFAULT 'planned' CHECK (status IN ('planned', 'active', 'paused', 'completed')),
  impact TEXT NOT NULL DEFAULT '',
  knowledge_node_id TEXT REFERENCES knowledge_areas(id) ON DELETE SET NULL,
  roadmap_id TEXT REFERENCES study_roadmaps(id) ON DELETE SET NULL,
  roadmap_topic_id TEXT REFERENCES roadmap_topics(id) ON DELETE SET NULL,
  project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_research_status ON research_items(status, updated_at DESC);
CREATE INDEX idx_research_knowledge ON research_items(knowledge_node_id);
CREATE INDEX idx_research_roadmap ON research_items(roadmap_id);
CREATE INDEX idx_research_project ON research_items(project_id);

CREATE TABLE knowledge_events (
  id TEXT PRIMARY KEY,
  knowledge_node_id TEXT NOT NULL REFERENCES knowledge_areas(id) ON DELETE RESTRICT,
  source_type TEXT NOT NULL
    CHECK (source_type IN ('baseline', 'roadmap', 'project', 'research', 'manual', 'education')),
  source_id TEXT,
  event_type TEXT NOT NULL,
  coverage_delta INTEGER NOT NULL DEFAULT 0,
  depth_delta INTEGER NOT NULL DEFAULT 0,
  integration_delta INTEGER NOT NULL DEFAULT 0,
  description TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_knowledge_events_node_date ON knowledge_events(knowledge_node_id, created_at DESC);
CREATE INDEX idx_knowledge_events_source ON knowledge_events(source_type, source_id);
