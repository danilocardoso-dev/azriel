ALTER TABLE education RENAME TO education_v1;

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

INSERT INTO education (
  id, name, type, institution, status, start_date, expected_end_date, completed_at,
  description, period, domains, created_at, updated_at
)
SELECT
  id,
  name,
  CASE
    WHEN id = 'mechatronics' THEN 'graduation'
    WHEN id = 'masters' THEN 'masters'
    WHEN type IN ('graduation', 'postgraduate', 'masters', 'doctorate', 'course', 'certification') THEN type
    ELSE 'course'
  END,
  institution,
  REPLACE(status, 'in-progress', 'in_progress'),
  CASE WHEN id = 'biomedicine' AND start_date IS NULL THEN '2027-01-01' ELSE start_date END,
  CASE WHEN id IN ('biotech', 'iot', 'big-data') AND expected_end_date IS NULL THEN '2026-11-30' ELSE expected_end_date END,
  completed_at, description, period, domains, created_at, updated_at
FROM education_v1;

DROP TABLE education_v1;
