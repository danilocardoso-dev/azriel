use super::stark_models::*;
use rusqlite::{params, Connection};
use std::collections::HashSet;

fn err(error: rusqlite::Error) -> String { error.to_string() }
fn required(value: &str, label: &str) -> Result<(), String> {
    if value.trim().is_empty() { Err(format!("Informe {label}")) } else { Ok(()) }
}

pub fn ensure_baselines(connection: &Connection) -> Result<(), String> {
    connection.execute("INSERT OR IGNORE INTO knowledge_baselines(knowledge_id,coverage,depth,recorded_at) SELECT id,coverage,depth,COALESCE(created_at,CURRENT_TIMESTAMP) FROM knowledge_areas", []).map_err(err)?;
    Ok(())
}

pub fn ensure_research_seed(connection: &mut Connection) -> Result<(), String> {
    let seeded = connection.query_row("SELECT EXISTS(SELECT 1 FROM _azriel_seeds WHERE key='v0.8.2-research')", [], |row| row.get::<_, bool>(0)).map_err(err)?;
    if seeded { return Ok(()); }
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute_batch("INSERT OR IGNORE INTO research_items(id,title,domain,objective,kind,status,impact,project_id) VALUES
      ('BIO-P01','Mendel Lab','Genética + Software','Simular cruzamentos e probabilidades','project','active','Genética educacional','mendel-lab'),
      ('BIO-P02','Gene Expression Explorer','Bioinformática','Comparar expressão gênica','project','active','Transcriptômica + dados','gene-expression'),
      ('BIO-P03','PCR Simulator','Biologia Molecular','Modelar primers, ciclos e amplicons','project','active','Molecular + software','pcr-simulator'),
      ('BIO-P04','GeneScope','Bioinformática','Explorar variantes genéticas','project','active','Genética aplicada','genescope'),
      ('FND-01','Matemática e Física','Fundamentos','Preparar a base para Mecatrônica','study','active','Redução de lacunas',NULL),
      ('ENE-01','ArcCore','Energia','Investigar armazenamento e gestão','research','planned','Projeto integrador','arccore');").map_err(err)?;
    transaction.execute("INSERT INTO _azriel_seeds(key) VALUES ('v0.8.2-research')", []).map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn list_baselines(connection: &Connection) -> Result<Vec<KnowledgeBaseline>, String> {
    let mut statement = connection.prepare("SELECT knowledge_id,coverage,depth,recorded_at FROM knowledge_baselines ORDER BY recorded_at DESC,knowledge_id").map_err(err)?;
    let rows = statement.query_map([], |row| Ok(KnowledgeBaseline { knowledge_area_id: row.get(0)?, coverage: row.get(1)?, depth: row.get(2)?, recorded_at: row.get(3)? })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn list_events(connection: &Connection) -> Result<Vec<KnowledgeEvent>, String> {
    let mut statement = connection.prepare("SELECT id,knowledge_node_id,source_type,source_id,event_type,coverage_delta,depth_delta,integration_delta,description,created_at FROM knowledge_events ORDER BY created_at DESC,id DESC").map_err(err)?;
    let rows = statement.query_map([], |row| Ok(KnowledgeEvent { id: row.get(0)?, knowledge_node_id: row.get(1)?, source_type: row.get(2)?, source_id: row.get(3)?, event_type: row.get(4)?, coverage_delta: row.get(5)?, depth_delta: row.get(6)?, integration_delta: row.get(7)?, description: row.get(8)?, created_at: row.get(9)? })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn list_activities(connection: &Connection, topic_id: &str) -> Result<Vec<RoadmapActivity>, String> {
    let mut statement = connection.prepare("SELECT id,title,description,activity_type,status,completed_at,activity_order FROM roadmap_activities WHERE topic_id=?1 ORDER BY activity_order").map_err(err)?;
    let rows = statement.query_map([topic_id], |row| Ok(RoadmapActivity { id: row.get(0)?, title: row.get(1)?, description: row.get(2)?, activity_type: row.get(3)?, status: row.get(4)?, completed_at: row.get(5)?, order: row.get(6)? })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn list_topics(connection: &Connection, stage_id: &str) -> Result<Vec<RoadmapTopic>, String> {
    let mut statement = connection.prepare("SELECT id,name,description,knowledge_node_id,topic_state,topic_order FROM roadmap_topics WHERE stage_id=?1 ORDER BY topic_order").map_err(err)?;
    let rows = statement.query_map([stage_id], |row| Ok(RoadmapTopic { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, knowledge_node_id: row.get(3)?, state: row.get(4)?, order: row.get(5)?, activities: vec![] })).map_err(err)?;
    let mut topics = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for topic in &mut topics { topic.activities = list_activities(connection, &topic.id)?; }
    Ok(topics)
}

fn list_stages(connection: &Connection, roadmap_id: &str) -> Result<Vec<RoadmapStage>, String> {
    let mut statement = connection.prepare("SELECT id,name,description,stage_order FROM roadmap_stages WHERE roadmap_id=?1 ORDER BY stage_order").map_err(err)?;
    let rows = statement.query_map([roadmap_id], |row| Ok(RoadmapStage { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, order: row.get(3)?, topics: vec![] })).map_err(err)?;
    let mut stages = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for stage in &mut stages { stage.topics = list_topics(connection, &stage.id)?; }
    Ok(stages)
}

pub fn list_roadmaps(connection: &Connection) -> Result<Vec<StudyRoadmap>, String> {
    let mut statement = connection.prepare("SELECT id,name,description,status,created_at,updated_at FROM study_roadmaps ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'planned' THEN 1 WHEN 'paused' THEN 2 ELSE 3 END,name").map_err(err)?;
    let rows = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?, row.get::<_, String>(5)?))).map_err(err)?;
    let mut result = Vec::new();
    for row in rows {
        let (id, name, description, status, created_at, updated_at) = row.map_err(err)?;
        let stages = list_stages(connection, &id)?;
        let total_activities = stages.iter().flat_map(|stage| &stage.topics).map(|topic| topic.activities.len() as i64).sum();
        let completed_activities = stages.iter().flat_map(|stage| &stage.topics).flat_map(|topic| &topic.activities).filter(|activity| activity.status == "completed").count() as i64;
        let progress = if total_activities == 0 { 0 } else { completed_activities * 100 / total_activities };
        result.push(StudyRoadmap { id, name, description, status, completed_activities, total_activities, progress, stages, created_at, updated_at });
    }
    Ok(result)
}

pub fn save_roadmap(connection: &mut Connection, input: &StudyRoadmapInput) -> Result<(), String> {
    required(&input.id, "o ID do roadmap")?; required(&input.name, "o nome do roadmap")?;
    if !["planned", "active", "paused", "completed"].contains(&input.status.as_str()) { return Err("Status de roadmap inválido".into()); }
    let mut ids = HashSet::new();
    for (stage_index, stage) in input.stages.iter().enumerate() {
        required(&stage.id, "o ID da etapa")?; required(&stage.name, "o nome da etapa")?;
        if stage.order != stage_index as i64 + 1 || !ids.insert(stage.id.clone()) { return Err("Etapas devem possuir IDs únicos e ordem contínua".into()); }
        for (topic_index, topic) in stage.topics.iter().enumerate() {
            required(&topic.id, "o ID do tópico")?; required(&topic.name, "o nome do tópico")?;
            if topic.order != topic_index as i64 + 1 || !ids.insert(topic.id.clone()) { return Err("Tópicos devem possuir IDs únicos e ordem contínua".into()); }
            if !["NOT_STARTED", "EXPOSED", "UNDERSTOOD", "PRACTICED", "APPLIED", "MASTERED"].contains(&topic.state.as_str()) { return Err("Estado de tópico inválido".into()); }
            if let Some(knowledge_id) = &topic.knowledge_node_id {
                let exists = connection.query_row("SELECT EXISTS(SELECT 1 FROM knowledge_areas WHERE id=?1)", [knowledge_id], |row| row.get::<_, bool>(0)).map_err(err)?;
                if !exists { return Err(format!("Conhecimento não encontrado no tópico {}", topic.name)); }
            }
            for (activity_index, activity) in topic.activities.iter().enumerate() {
                required(&activity.id, "o ID da atividade")?; required(&activity.title, "o título da atividade")?;
                if activity.order != activity_index as i64 + 1 || !ids.insert(activity.id.clone()) { return Err("Atividades devem possuir IDs únicos e ordem contínua".into()); }
                if !["READING", "LESSON", "QUIZ", "EXERCISE", "SIMULATION", "EXPERIMENT", "PROJECT", "DOCUMENTATION", "RESEARCH", "OTHER"].contains(&activity.activity_type.as_str()) { return Err("Tipo de atividade inválido".into()); }
                if !["pending", "in_progress", "completed"].contains(&activity.status.as_str()) { return Err("Status de atividade inválido".into()); }
            }
        }
    }
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute("INSERT INTO study_roadmaps(id,name,description,status) VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,status=excluded.status,updated_at=CURRENT_TIMESTAMP", params![input.id,input.name.trim(),input.description.trim(),input.status]).map_err(err)?;
    transaction.execute("DELETE FROM roadmap_stages WHERE roadmap_id=?1", [&input.id]).map_err(err)?;
    for stage in &input.stages {
        transaction.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,description,stage_order) VALUES (?1,?2,?3,?4,?5)", params![stage.id,input.id,stage.name.trim(),stage.description.trim(),stage.order]).map_err(err)?;
        for topic in &stage.topics {
            transaction.execute("INSERT INTO roadmap_topics(id,stage_id,name,description,knowledge_node_id,topic_state,topic_order) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![topic.id,stage.id,topic.name.trim(),topic.description.trim(),topic.knowledge_node_id,topic.state,topic.order]).map_err(err)?;
            for activity in &topic.activities {
                let completed_at = if activity.status == "completed" { activity.completed_at.clone().or_else(|| Some("CURRENT_TIMESTAMP".into())) } else { None };
                transaction.execute("INSERT INTO roadmap_activities(id,topic_id,title,description,activity_type,status,completed_at,activity_order) VALUES (?1,?2,?3,?4,?5,?6,CASE WHEN ?7='CURRENT_TIMESTAMP' THEN CURRENT_TIMESTAMP ELSE ?7 END,?8)", params![activity.id,topic.id,activity.title.trim(),activity.description.trim(),activity.activity_type,activity.status,completed_at,activity.order]).map_err(err)?;
            }
        }
    }
    transaction.commit().map_err(err)
}

pub fn delete_roadmap(connection: &Connection, id: &str) -> Result<(), String> {
    if connection.execute("DELETE FROM study_roadmaps WHERE id=?1", [id]).map_err(err)? == 0 { return Err("Roadmap não encontrado".into()); }
    Ok(())
}

pub fn list_research(connection: &Connection) -> Result<Vec<ResearchItem>, String> {
    let mut statement = connection.prepare("SELECT id,title,domain,objective,description,kind,status,impact,knowledge_node_id,roadmap_id,roadmap_topic_id,project_id,created_at,updated_at FROM research_items ORDER BY CASE status WHEN 'active' THEN 0 WHEN 'planned' THEN 1 WHEN 'paused' THEN 2 ELSE 3 END,updated_at DESC").map_err(err)?;
    let rows = statement.query_map([], |row| Ok(ResearchItem { id: row.get(0)?, title: row.get(1)?, domain: row.get(2)?, objective: row.get(3)?, description: row.get(4)?, kind: row.get(5)?, status: row.get(6)?, impact: row.get(7)?, knowledge_node_id: row.get(8)?, roadmap_id: row.get(9)?, roadmap_topic_id: row.get(10)?, project_id: row.get(11)?, created_at: row.get(12)?, updated_at: row.get(13)? })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn save_research(connection: &Connection, input: &ResearchItem) -> Result<(), String> {
    required(&input.id, "o ID da pesquisa")?; required(&input.title, "o título da pesquisa")?;
    if !["project", "study", "research"].contains(&input.kind.as_str()) { return Err("Tipo de pesquisa inválido".into()); }
    if !["planned", "active", "paused", "completed"].contains(&input.status.as_str()) { return Err("Status de pesquisa inválido".into()); }
    if input.roadmap_topic_id.is_some() && input.roadmap_id.is_none() { return Err("Um tópico de roadmap exige o roadmap relacionado".into()); }
    if let (Some(topic_id), Some(roadmap_id)) = (&input.roadmap_topic_id, &input.roadmap_id) {
        let association = connection.query_row("SELECT EXISTS(SELECT 1 FROM roadmap_topics topic JOIN roadmap_stages stage ON stage.id=topic.stage_id WHERE topic.id=?1 AND stage.roadmap_id=?2)", params![topic_id, roadmap_id], |row| row.get::<_, bool>(0)).map_err(err)?;
        if !association { return Err("O tópico selecionado não pertence ao roadmap informado".into()); }
    }
    connection.execute("INSERT INTO research_items(id,title,domain,objective,description,kind,status,impact,knowledge_node_id,roadmap_id,roadmap_topic_id,project_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12) ON CONFLICT(id) DO UPDATE SET title=excluded.title,domain=excluded.domain,objective=excluded.objective,description=excluded.description,kind=excluded.kind,status=excluded.status,impact=excluded.impact,knowledge_node_id=excluded.knowledge_node_id,roadmap_id=excluded.roadmap_id,roadmap_topic_id=excluded.roadmap_topic_id,project_id=excluded.project_id,updated_at=CURRENT_TIMESTAMP", params![input.id,input.title.trim(),input.domain.trim(),input.objective.trim(),input.description.trim(),input.kind,input.status,input.impact.trim(),input.knowledge_node_id,input.roadmap_id,input.roadmap_topic_id,input.project_id]).map_err(err)?;
    Ok(())
}

pub fn delete_research(connection: &Connection, id: &str) -> Result<(), String> {
    if connection.execute("DELETE FROM research_items WHERE id=?1", [id]).map_err(err)? == 0 { return Err("Pesquisa não encontrada".into()); }
    Ok(())
}

pub fn summary(connection: &Connection) -> Result<StarkSummary, String> {
    let count = |sql: &str| connection.query_row(sql, [], |row| row.get::<_, i64>(0)).map_err(err);
    Ok(StarkSummary { roadmap_count: count("SELECT COUNT(*) FROM study_roadmaps")?, active_roadmap_count: count("SELECT COUNT(*) FROM study_roadmaps WHERE status='active'")?, research_count: count("SELECT COUNT(*) FROM research_items")?, active_research_count: count("SELECT COUNT(*) FROM research_items WHERE status='active'")?, baseline_count: count("SELECT COUNT(*) FROM knowledge_baselines")?, event_count: count("SELECT COUNT(*) FROM knowledge_events")? })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn roadmap() -> StudyRoadmapInput {
        StudyRoadmapInput { id: "control-roadmap".into(), name: "Controle e Automação".into(), description: "Teste".into(), status: "active".into(), stages: vec![RoadmapStage { id: "electronics-stage".into(), name: "Eletrônica".into(), description: "".into(), order: 1, topics: vec![RoadmapTopic { id: "mosfet-topic".into(), name: "MOSFET".into(), description: "".into(), knowledge_node_id: Some("electronics".into()), state: "EXPOSED".into(), order: 1, activities: vec![RoadmapActivity { id: "read-mosfet".into(), title: "Ler fundamentos".into(), description: "".into(), activity_type: "READING".into(), status: "completed".into(), completed_at: None, order: 1 }, RoadmapActivity { id: "simulate-mosfet".into(), title: "Simular circuito".into(), description: "".into(), activity_type: "SIMULATION".into(), status: "pending".into(), completed_at: None, order: 2 }] }] }] }
    }

    #[test]
    fn roadmap_crud_calculates_progress_without_changing_knowledge() {
        let mut connection = Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap();
        let before: (i64, i64) = connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id='electronics'", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        save_roadmap(&mut connection, &roadmap()).unwrap();
        let result = list_roadmaps(&connection).unwrap();
        assert_eq!((result[0].completed_activities, result[0].total_activities, result[0].progress), (1, 2, 50));
        let after: (i64, i64) = connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id='electronics'", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        assert_eq!(before, after);
        assert_eq!(list_events(&connection).unwrap().len(), 0);
        delete_roadmap(&connection, "control-roadmap").unwrap();
        assert!(list_roadmaps(&connection).unwrap().is_empty());
    }

    #[test]
    fn roadmap_rejects_unknown_knowledge_before_writing() {
        let mut connection = Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap();
        let mut input = roadmap(); input.stages[0].topics[0].knowledge_node_id = Some("unknown".into());
        assert!(save_roadmap(&mut connection, &input).unwrap_err().contains("não encontrado"));
        assert!(list_roadmaps(&connection).unwrap().is_empty());
    }

    #[test]
    fn research_relationships_persist_without_creating_events() {
        let mut connection = Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap();
        save_roadmap(&mut connection, &roadmap()).unwrap();
        let input = ResearchItem { id: "pid-research".into(), title: "Controle PID".into(), domain: "Controle".into(), objective: "Investigar".into(), description: "".into(), kind: "research".into(), status: "completed".into(), impact: "".into(), knowledge_node_id: Some("control".into()), roadmap_id: Some("control-roadmap".into()), roadmap_topic_id: Some("mosfet-topic".into()), project_id: Some("arccore".into()), created_at: "".into(), updated_at: "".into() };
        save_research(&connection, &input).unwrap();
        let saved = list_research(&connection).unwrap().into_iter().find(|item| item.id == input.id).unwrap();
        assert_eq!(saved.knowledge_node_id.as_deref(), Some("control"));
        assert_eq!(saved.roadmap_id.as_deref(), Some("control-roadmap"));
        assert_eq!(saved.project_id.as_deref(), Some("arccore"));
        assert!(list_events(&connection).unwrap().is_empty());
    }
}
