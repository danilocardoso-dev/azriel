ALTER TABLE roadmap_activities ADD COLUMN learning_objective TEXT;
ALTER TABLE roadmap_activities ADD COLUMN instructions TEXT;
ALTER TABLE roadmap_activities ADD COLUMN completion_criteria TEXT;
ALTER TABLE roadmap_activities ADD COLUMN deliverable TEXT;
ALTER TABLE roadmap_activities ADD COLUMN estimated_minutes INTEGER CHECK (estimated_minutes IS NULL OR (estimated_minutes >= 1 AND estimated_minutes <= 1440));
ALTER TABLE roadmap_activities ADD COLUMN learning_method_type TEXT CHECK (learning_method_type IS NULL OR learning_method_type IN ('ACTIVE_RECALL','FEYNMAN','SHADOWING','SPACED_REVIEW','HANDS_ON','PROBLEM_SOLVING','CASE_STUDY','BUILD','OBSERVE','EXPERIMENT','REFLECTION','RESEARCH','OTHER'));
ALTER TABLE roadmap_activities ADD COLUMN learning_method_instructions TEXT;
ALTER TABLE roadmap_activities ADD COLUMN is_validation INTEGER NOT NULL DEFAULT 0 CHECK (is_validation IN (0,1));
ALTER TABLE roadmap_activities ADD COLUMN reflection_prompt TEXT;

CREATE TABLE roadmap_activity_resources (
  id TEXT PRIMARY KEY,
  activity_id TEXT NOT NULL REFERENCES roadmap_activities(id) ON DELETE CASCADE,
  resource_order INTEGER NOT NULL CHECK (resource_order > 0),
  resource_type TEXT NOT NULL CHECK (resource_type IN ('VIDEO','ARTICLE','DOCUMENTATION','COURSE','BOOK','LAB','TOOL','PODCAST','DATASET','WEBSITE','OTHER')),
  title TEXT NOT NULL CHECK (length(trim(title)) > 0),
  url TEXT,
  provider TEXT,
  language TEXT,
  required INTEGER NOT NULL DEFAULT 0 CHECK (required IN (0,1)),
  study_material_id TEXT REFERENCES study_materials(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(activity_id, resource_order)
);

CREATE INDEX idx_roadmap_activity_resources_activity ON roadmap_activity_resources(activity_id, resource_order);
CREATE INDEX idx_roadmap_activity_resources_material ON roadmap_activity_resources(study_material_id);
