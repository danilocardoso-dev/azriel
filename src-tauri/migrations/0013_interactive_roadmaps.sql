CREATE TABLE roadmap_topic_prerequisites (
  topic_id TEXT NOT NULL REFERENCES roadmap_topics(id) ON DELETE CASCADE,
  prerequisite_topic_id TEXT NOT NULL REFERENCES roadmap_topics(id) ON DELETE CASCADE,
  PRIMARY KEY (topic_id, prerequisite_topic_id),
  CHECK (topic_id <> prerequisite_topic_id)
);

CREATE INDEX idx_roadmap_topic_prerequisite
  ON roadmap_topic_prerequisites(prerequisite_topic_id, topic_id);

-- Roadmaps importados antes da v0.8.4 podiam registrar um único pré-requisito
-- no final da descrição. A relação é promovida somente quando o ID é válido.
WITH legacy AS (
  SELECT
    id AS topic_id,
    trim(substr(description, instr(description, 'Pré-requisitos:') + length('Pré-requisitos:'))) AS raw_prerequisite
  FROM roadmap_topics
  WHERE instr(description, 'Pré-requisitos:') > 0
), candidates AS (
  SELECT
    topic_id,
    trim(CASE
      WHEN instr(raw_prerequisite, ';') > 0 THEN substr(raw_prerequisite, 1, instr(raw_prerequisite, ';') - 1)
      ELSE raw_prerequisite
    END) AS prerequisite_topic_id
  FROM legacy
)
INSERT OR IGNORE INTO roadmap_topic_prerequisites(topic_id, prerequisite_topic_id)
SELECT candidate.topic_id, candidate.prerequisite_topic_id
FROM candidates candidate
JOIN roadmap_topics prerequisite ON prerequisite.id = candidate.prerequisite_topic_id
WHERE candidate.topic_id <> candidate.prerequisite_topic_id;
