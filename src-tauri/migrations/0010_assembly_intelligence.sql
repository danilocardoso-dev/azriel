CREATE TABLE engineering_models (
  model_identity TEXT PRIMARY KEY,
  file_name TEXT NOT NULL,
  format TEXT NOT NULL CHECK(format IN ('GLB','GLTF')),
  byte_size INTEGER NOT NULL CHECK(byte_size >= 0),
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE engineering_model_components (
  model_identity TEXT NOT NULL REFERENCES engineering_models(model_identity) ON DELETE CASCADE,
  component_identity TEXT NOT NULL,
  original_name TEXT NOT NULL,
  structural_path TEXT NOT NULL,
  component_type TEXT NOT NULL,
  selectable INTEGER NOT NULL DEFAULT 0 CHECK(selectable IN (0,1)),
  last_seen_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(model_identity, component_identity)
);

CREATE TABLE engineering_subsystems (
  id TEXT PRIMARY KEY,
  model_identity TEXT NOT NULL REFERENCES engineering_models(model_identity) ON DELETE CASCADE,
  name TEXT NOT NULL COLLATE NOCASE,
  description TEXT NOT NULL DEFAULT '',
  parent_subsystem_id TEXT REFERENCES engineering_subsystems(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(model_identity, name),
  CHECK(parent_subsystem_id IS NULL OR parent_subsystem_id <> id)
);

CREATE TABLE engineering_component_semantics (
  model_identity TEXT NOT NULL,
  component_identity TEXT NOT NULL,
  semantic_label TEXT NOT NULL DEFAULT '',
  subsystem_id TEXT REFERENCES engineering_subsystems(id) ON DELETE SET NULL,
  role TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  notes TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY(model_identity, component_identity),
  FOREIGN KEY(model_identity, component_identity)
    REFERENCES engineering_model_components(model_identity, component_identity) ON DELETE CASCADE
);

CREATE TABLE engineering_component_relationships (
  id TEXT PRIMARY KEY,
  model_identity TEXT NOT NULL REFERENCES engineering_models(model_identity) ON DELETE CASCADE,
  source_component_identity TEXT NOT NULL,
  target_component_identity TEXT NOT NULL,
  relationship_type TEXT NOT NULL CHECK(relationship_type IN (
    'connected_to','contains','supports','drives','mounted_on','adjacent_to','depends_on','custom'
  )),
  description TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  CHECK(source_component_identity <> target_component_identity),
  UNIQUE(model_identity, source_component_identity, target_component_identity, relationship_type),
  FOREIGN KEY(model_identity, source_component_identity)
    REFERENCES engineering_model_components(model_identity, component_identity) ON DELETE CASCADE,
  FOREIGN KEY(model_identity, target_component_identity)
    REFERENCES engineering_model_components(model_identity, component_identity) ON DELETE CASCADE
);

CREATE INDEX idx_engineering_components_model ON engineering_model_components(model_identity);
CREATE INDEX idx_engineering_semantics_subsystem ON engineering_component_semantics(subsystem_id);
CREATE INDEX idx_engineering_relationships_model ON engineering_component_relationships(model_identity);
