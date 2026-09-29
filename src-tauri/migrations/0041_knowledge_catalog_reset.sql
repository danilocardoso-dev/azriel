-- Substitui o catálogo legado pelo conjunto canônico aprovado para o Study Lab.
-- Referências antigas são removidas antes dos nós para respeitar as FKs RESTRICT.
DELETE FROM activity_knowledge_nodes;
DELETE FROM knowledge_events;
UPDATE roadmap_topics SET knowledge_node_id = NULL;
UPDATE research_items SET knowledge_node_id = NULL;
UPDATE tasks SET knowledge_area_id = NULL;
UPDATE notes SET knowledge_area_id = NULL;
DELETE FROM project_knowledge;
UPDATE knowledge_areas SET parent_id = NULL;
DELETE FROM knowledge_areas;

UPDATE app_metrics SET value = 0, formula_note = 'Reiniciado com o catálogo canônico do Study Lab', updated_at = CURRENT_TIMESTAMP
WHERE key = 'integration';
UPDATE learning_engine_state
SET integration_baseline = 0, last_recalculated_at = NULL, status = 'ready', last_error = NULL
WHERE id = 1;

INSERT INTO knowledge_areas(
  id, name, category, description, coverage, depth, priority, node_type, parent_id
) VALUES
  ('software-engineering', 'Engenharia de Software', 'Tecnologia', 'Arquitetura de software, código, testes, concorrência, performance, APIs e sistemas distribuídos.', 0, 0, 'high', 'area', NULL),
  ('systems-infrastructure', 'Sistemas e Infraestrutura', 'Tecnologia', 'Linux, servidores, containers, redes, cloud, platform engineering e deploy.', 0, 0, 'high', 'area', NULL),
  ('cybersecurity', 'Cybersecurity', 'Segurança', 'Security engineering, AppSec, hardening, threat modeling e segurança ofensiva e defensiva.', 0, 0, 'high', 'area', NULL),
  ('dfir', 'Forense Digital & Incident Response', 'Segurança', 'Evidências, investigação, memória, logs, timelines e resposta a incidentes.', 0, 0, 'high', 'area', NULL),
  ('reliability-observability', 'Reliability & Observabilidade', 'Operação', 'SRE, métricas, logs, traces, SLO/SLI, incidentes e performance operacional.', 0, 0, 'high', 'area', NULL),
  ('ai-engineering', 'Engenharia de IA', 'Inteligência Artificial', 'LLMs, inferência, contexto, tools, agents, RAG, avaliação, deployment e AI security.', 0, 0, 'high', 'area', NULL),
  ('automation', 'Automação', 'Operação', 'Integrações, workflows, scripting, IoT, tarefas, orquestração e automação operacional.', 0, 0, 'high', 'area', NULL),
  ('english', 'Inglês', 'Idiomas', 'Compreensão, listening, speaking, writing, pronúncia e inglês técnico.', 0, 0, 'high', 'area', NULL),
  ('technology-business', 'Empreendedorismo Tecnológico', 'Negócios', 'Produto, problema, cliente, vendas B2B, validação, operação, métricas e estratégia.', 0, 0, 'high', 'area', NULL),
  ('engineering-leadership', 'Liderança e Decisão Técnica', 'Liderança', 'Trade-offs, ADRs, comunicação técnica, priorização, mentoring e decisões de engenharia.', 0, 0, 'high', 'area', NULL);

INSERT INTO knowledge_baselines(knowledge_id, coverage, depth)
SELECT id, 0, 0 FROM knowledge_areas;

INSERT INTO knowledge_history(knowledge_id, coverage, depth, reason)
SELECT id, 0, 0, 'Catálogo canônico do Study Lab' FROM knowledge_areas;

INSERT INTO project_knowledge(project_id, knowledge_id)
SELECT 'azriel', knowledge_id
FROM (
  SELECT 'software-engineering' AS knowledge_id
  UNION ALL SELECT 'ai-engineering'
  UNION ALL SELECT 'automation'
)
WHERE EXISTS(SELECT 1 FROM projects WHERE id = 'azriel');
