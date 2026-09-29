CREATE TABLE study_materials (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL CHECK (length(trim(title)) BETWEEN 1 AND 240),
  material_type TEXT NOT NULL CHECK (material_type IN ('PDF', 'IMAGE', 'TEXT', 'MARKDOWN', 'LINK', 'OTHER')),
  storage_kind TEXT NOT NULL CHECK (storage_kind IN ('MANAGED_COPY', 'LINKED_LOCAL_FILE', 'EXTERNAL_URL')),
  local_path TEXT,
  external_url TEXT,
  original_filename TEXT,
  mime_type TEXT,
  file_size INTEGER CHECK (file_size IS NULL OR file_size >= 0),
  checksum_sha256 TEXT,
  source_modified_at TEXT,
  source_author TEXT,
  source_title TEXT,
  source_year INTEGER CHECK (source_year IS NULL OR source_year BETWEEN 1000 AND 9999),
  description TEXT,
  page_count INTEGER CHECK (page_count IS NULL OR page_count >= 0),
  status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'ARCHIVED', 'MISSING', 'ERROR')),
  removed_at TEXT,
  managed_file_deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  CHECK (
    (storage_kind IN ('MANAGED_COPY', 'LINKED_LOCAL_FILE') AND external_url IS NULL)
    OR (storage_kind = 'EXTERNAL_URL' AND local_path IS NULL AND external_url IS NOT NULL)
  )
);

CREATE UNIQUE INDEX idx_study_materials_checksum
  ON study_materials(checksum_sha256)
  WHERE checksum_sha256 IS NOT NULL AND removed_at IS NULL;

CREATE INDEX idx_study_materials_library
  ON study_materials(removed_at, status, updated_at DESC);

CREATE INDEX idx_study_materials_type
  ON study_materials(material_type, removed_at, updated_at DESC);

CREATE TABLE study_material_relations (
  id TEXT PRIMARY KEY,
  material_id TEXT NOT NULL REFERENCES study_materials(id) ON DELETE RESTRICT,
  relation_type TEXT NOT NULL CHECK (relation_type IN ('ROADMAP', 'STAGE', 'TOPIC', 'ACTIVITY', 'NOTEBOOK', 'NOTE', 'CARD')),
  relation_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(material_id, relation_type, relation_id)
);

CREATE INDEX idx_study_material_relations_material
  ON study_material_relations(material_id, created_at);

CREATE INDEX idx_study_material_relations_target
  ON study_material_relations(relation_type, relation_id, created_at DESC);

CREATE TABLE material_text_content (
  material_id TEXT PRIMARY KEY REFERENCES study_materials(id) ON DELETE CASCADE,
  status TEXT NOT NULL CHECK (status IN ('NOT_ATTEMPTED', 'AVAILABLE', 'TEXT_UNAVAILABLE', 'FAILED')),
  text_content TEXT,
  pages_json TEXT,
  extracted_at TEXT,
  extractor_version TEXT,
  char_count INTEGER NOT NULL DEFAULT 0 CHECK (char_count >= 0),
  source_checksum_sha256 TEXT,
  is_stale INTEGER NOT NULL DEFAULT 0 CHECK (is_stale IN (0, 1)),
  error_code TEXT,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_material_text_status
  ON material_text_content(status, is_stale, updated_at DESC);
