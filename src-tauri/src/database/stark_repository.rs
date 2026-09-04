use super::{learning_engine, stark_models::*};
use rusqlite::{params, Connection};
use std::collections::{HashMap, HashSet};

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
    learning_engine::list_events(connection)
}

fn list_activities(connection: &Connection, topic_id: &str) -> Result<Vec<RoadmapActivity>, String> {
    let mut statement = connection.prepare("SELECT id,title,description,activity_type,status,completed_at,activity_order,project_id,research_id FROM roadmap_activities WHERE topic_id=?1 ORDER BY activity_order").map_err(err)?;
    let rows = statement.query_map([topic_id], |row| Ok(RoadmapActivity { id: row.get(0)?, title: row.get(1)?, description: row.get(2)?, activity_type: row.get(3)?, status: row.get(4)?, completed_at: row.get(5)?, order: row.get(6)?, project_id: row.get(7)?, research_id: row.get(8)?, primary_knowledge_node_id: None, secondary_knowledge_node_ids: vec![] })).map_err(err)?;
    let mut activities=rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for activity in &mut activities {
        let mut relations=connection.prepare("SELECT knowledge_node_id,role FROM activity_knowledge_nodes WHERE activity_id=?1 ORDER BY role,knowledge_node_id").map_err(err)?;
        let values=relations.query_map([&activity.id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
        activity.primary_knowledge_node_id=values.iter().find(|(_,role)|role=="primary").map(|(id,_)|id.clone());
        activity.secondary_knowledge_node_ids=values.into_iter().filter(|(_,role)|role=="secondary").map(|(id,_)|id).collect();
    }
    Ok(activities)
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

pub fn save_roadmap(connection: &mut Connection, input: &StudyRoadmapInput) -> Result<LearningMutation, String> {
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
                let primary=activity.primary_knowledge_node_id.as_ref().or(topic.knowledge_node_id.as_ref()).ok_or_else(||format!("A atividade {} exige um conhecimento primário",activity.title))?;
                let mut relation_ids=HashSet::from([primary.clone()]);
                for knowledge_id in &activity.secondary_knowledge_node_ids { if !relation_ids.insert(knowledge_id.clone()) { return Err("Conhecimentos relacionados não podem ser repetidos".into()); } }
                for knowledge_id in relation_ids { let exists=connection.query_row("SELECT EXISTS(SELECT 1 FROM knowledge_areas WHERE id=?1)",[knowledge_id],|row|row.get::<_,bool>(0)).map_err(err)?; if !exists{return Err("Conhecimento relacionado não encontrado".into());} }
            }
        }
    }
    let old_statuses:HashMap<String,(String,String)>={
        let mut statement=connection.prepare("SELECT activity.id,activity.status,activity.activity_type FROM roadmap_activities activity JOIN roadmap_topics topic ON topic.id=activity.topic_id JOIN roadmap_stages stage ON stage.id=topic.stage_id WHERE stage.roadmap_id=?1").map_err(err)?;
        let result=statement.query_map([&input.id],|row|Ok((row.get(0)?,(row.get(1)?,row.get(2)?)))).map_err(err)?.collect::<Result<HashMap<_,_>,_>>().map_err(err)?;
        result
    };
    let old_relations:HashMap<String,HashSet<String>>={
        let mut statement=connection.prepare("SELECT relation.activity_id,relation.role || ':' || relation.knowledge_node_id FROM activity_knowledge_nodes relation JOIN roadmap_activities activity ON activity.id=relation.activity_id JOIN roadmap_topics topic ON topic.id=activity.topic_id JOIN roadmap_stages stage ON stage.id=topic.stage_id WHERE stage.roadmap_id=?1").map_err(err)?;
        let rows=statement.query_map([&input.id],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).map_err(err)?.collect::<Result<Vec<_>,_>>().map_err(err)?;
        let mut relations:HashMap<String,HashSet<String>>=HashMap::new(); for (activity,value) in rows { relations.entry(activity).or_default().insert(value); } relations
    };
    let new_ids:HashSet<String>=input.stages.iter().flat_map(|stage|&stage.topics).flat_map(|topic|&topic.activities).map(|activity|activity.id.clone()).collect();
    for (old_id,(old_status,_)) in &old_statuses { if !new_ids.contains(old_id) && old_status=="completed" { return Err("Reabra atividades concluídas antes de removê-las".into()); } }
    for activity in input.stages.iter().flat_map(|stage|&stage.topics).flat_map(|topic|&topic.activities) {
        if let Some((old_status,old_type))=old_statuses.get(&activity.id) {
            if old_status=="completed" && activity.status=="completed" && old_type!=&activity.activity_type { return Err("Reabra a atividade concluída antes de alterar seu tipo de evidência".into()); }
        }
    }
    for topic in input.stages.iter().flat_map(|stage|&stage.topics) { for activity in &topic.activities {
        if old_statuses.get(&activity.id).is_some_and(|value|value.0=="completed") && activity.status=="completed" {
            let primary=activity.primary_knowledge_node_id.as_ref().or(topic.knowledge_node_id.as_ref()).expect("validated");
            let mut current=HashSet::from([format!("primary:{primary}")]); current.extend(activity.secondary_knowledge_node_ids.iter().map(|id|format!("secondary:{id}")));
            if old_relations.get(&activity.id).is_some_and(|old|old!=&current) { return Err("Reabra a atividade concluída antes de alterar seus conhecimentos relacionados".into()); }
        }
    }}
    let previous_events:HashSet<String>=learning_engine::list_events(connection)?.into_iter().map(|event|event.id).collect();
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute("INSERT INTO study_roadmaps(id,name,description,status) VALUES (?1,?2,?3,?4) ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,status=excluded.status,updated_at=CURRENT_TIMESTAMP", params![input.id,input.name.trim(),input.description.trim(),input.status]).map_err(err)?;
    transaction.execute("DELETE FROM roadmap_stages WHERE roadmap_id=?1", [&input.id]).map_err(err)?;
    for stage in &input.stages {
        transaction.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,description,stage_order) VALUES (?1,?2,?3,?4,?5)", params![stage.id,input.id,stage.name.trim(),stage.description.trim(),stage.order]).map_err(err)?;
        for topic in &stage.topics {
            transaction.execute("INSERT INTO roadmap_topics(id,stage_id,name,description,knowledge_node_id,topic_state,topic_order) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![topic.id,stage.id,topic.name.trim(),topic.description.trim(),topic.knowledge_node_id,topic.state,topic.order]).map_err(err)?;
            for activity in &topic.activities {
                let completed_at = if activity.status == "completed" { activity.completed_at.clone().or_else(|| Some("CURRENT_TIMESTAMP".into())) } else { None };
                transaction.execute("INSERT INTO roadmap_activities(id,topic_id,title,description,activity_type,status,completed_at,activity_order,project_id,research_id) VALUES (?1,?2,?3,?4,?5,?6,CASE WHEN ?7='CURRENT_TIMESTAMP' THEN CURRENT_TIMESTAMP ELSE ?7 END,?8,?9,?10)", params![activity.id,topic.id,activity.title.trim(),activity.description.trim(),activity.activity_type,activity.status,completed_at,activity.order,activity.project_id,activity.research_id]).map_err(err)?;
                let primary=activity.primary_knowledge_node_id.as_ref().or(topic.knowledge_node_id.as_ref()).expect("validated");
                transaction.execute("INSERT INTO activity_knowledge_nodes(activity_id,knowledge_node_id,role) VALUES (?1,?2,'primary')",params![activity.id,primary]).map_err(err)?;
                for secondary in &activity.secondary_knowledge_node_ids { transaction.execute("INSERT INTO activity_knowledge_nodes(activity_id,knowledge_node_id,role) VALUES (?1,?2,'secondary')",params![activity.id,secondary]).map_err(err)?; }
            }
        }
    }
    for activity in input.stages.iter().flat_map(|stage|&stage.topics).flat_map(|topic|&topic.activities) {
        let old=old_statuses.get(&activity.id).map(|value|value.0.as_str()).unwrap_or("pending");
        if old!="completed" && activity.status=="completed" { learning_engine::complete_activity(&transaction,&activity.id)?; }
        if old=="completed" && activity.status!="completed" { learning_engine::reopen_activity(&transaction,&activity.id)?; }
    }
    let mut result=learning_engine::recalculate(&transaction,"Learning Engine: atividade de roadmap")?;
    result.created_events=learning_engine::list_events(&transaction)?.into_iter().filter(|event|!previous_events.contains(&event.id)).collect();
    transaction.commit().map_err(err)?;
    Ok(result)
}

pub fn delete_roadmap(connection: &Connection, id: &str) -> Result<(), String> {
    let completed=connection.query_row("SELECT EXISTS(SELECT 1 FROM roadmap_activities activity JOIN roadmap_topics topic ON topic.id=activity.topic_id JOIN roadmap_stages stage ON stage.id=topic.stage_id WHERE stage.roadmap_id=?1 AND activity.status='completed')",[id],|row|row.get::<_,bool>(0)).map_err(err)?;
    if completed { return Err("Reabra as atividades concluídas antes de excluir o roadmap".into()); }
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
        let activity = |id:&str,title:&str,kind:&str,status:&str,order:i64| RoadmapActivity { id:id.into(),title:title.into(),description:"".into(),activity_type:kind.into(),status:status.into(),completed_at:None,order,primary_knowledge_node_id:None,secondary_knowledge_node_ids:vec![],project_id:None,research_id:None };
        StudyRoadmapInput { id: "control-roadmap".into(), name: "Controle e Automação".into(), description: "Teste".into(), status: "active".into(), stages: vec![RoadmapStage { id: "electronics-stage".into(), name: "Eletrônica".into(), description: "".into(), order: 1, topics: vec![RoadmapTopic { id: "mosfet-topic".into(), name: "MOSFET".into(), description: "".into(), knowledge_node_id: Some("electronics".into()), state: "EXPOSED".into(), order: 1, activities: vec![activity("read-mosfet","Ler fundamentos","READING","completed",1),activity("simulate-mosfet","Simular circuito","SIMULATION","pending",2)] }] }] }
    }

    #[test]
    fn roadmap_completion_creates_evidence_and_updates_knowledge() {
        let mut connection = Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap();
        let before: (i64, i64) = connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id='electronics'", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        save_roadmap(&mut connection, &roadmap()).unwrap();
        let result = list_roadmaps(&connection).unwrap();
        assert_eq!((result[0].completed_activities, result[0].total_activities, result[0].progress), (1, 2, 50));
        let after: (i64, i64) = connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id='electronics'", [], |row| Ok((row.get(0)?, row.get(1)?))).unwrap();
        assert!(after.0 > before.0);
        assert!(!list_events(&connection).unwrap().is_empty());
        assert!(delete_roadmap(&connection, "control-roadmap").unwrap_err().contains("conclu"));
        let event_count=list_events(&connection).unwrap().len();
        assert!(save_roadmap(&mut connection,&roadmap()).unwrap().created_events.is_empty());
        assert_eq!(list_events(&connection).unwrap().len(),event_count);
        let mut reopened=roadmap(); reopened.stages[0].topics[0].activities[0].status="pending".into();
        let reversal=save_roadmap(&mut connection,&reopened).unwrap();
        assert!(reversal.created_events.iter().all(|event|event.event_type=="activity_reopened"));
        let restored:(i64,i64)=connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id='electronics'",[],|row|Ok((row.get(0)?,row.get(1)?))).unwrap();
        assert_eq!(restored,before);
        let recompleted=save_roadmap(&mut connection,&roadmap()).unwrap();
        assert!(recompleted.created_events.iter().all(|event|event.evidence_cycle==2));
    }

    #[test]
    fn interdisciplinary_project_distributes_evidence_and_increases_integration() {
        let mut connection=Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap();
        let mut input=roadmap(); let evidence=&mut input.stages[0].topics[0].activities[0];
        evidence.activity_type="PROJECT".into(); evidence.secondary_knowledge_node_ids=vec!["control".into()];
        let result=save_roadmap(&mut connection,&input).unwrap();
        assert!(result.integration>0.0);
        assert!(result.created_events.iter().any(|event|event.knowledge_node_id=="electronics"));
        assert!(result.created_events.iter().any(|event|event.knowledge_node_id=="control"));
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
        assert!(!list_events(&connection).unwrap().is_empty());
    }
}
