use super::stark_models::{KnowledgeEvent, LearningEngineStatus, LearningMutation};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

pub const FORMULA_VERSION: &str = "LEARNING_ENGINE_V1";
const BASE_IMPACT: f64 = 5.0;
const HIERARCHY_FACTORS: [f64; 3] = [1.0, 0.5, 0.25];

#[derive(Clone, Copy, Debug, Serialize)]
pub struct EvidenceProfile { pub coverage: f64, pub depth: f64, pub integration: f64 }

pub fn evidence_profile(kind: &str) -> Option<EvidenceProfile> {
    Some(match kind {
        "READING" => EvidenceProfile { coverage: 1.0, depth: 0.1, integration: 0.0 },
        "LESSON" => EvidenceProfile { coverage: 0.9, depth: 0.2, integration: 0.0 },
        "QUIZ" => EvidenceProfile { coverage: 0.6, depth: 0.35, integration: 0.0 },
        "EXERCISE" => EvidenceProfile { coverage: 0.45, depth: 0.7, integration: 0.1 },
        "SIMULATION" => EvidenceProfile { coverage: 0.35, depth: 0.85, integration: 0.3 },
        "EXPERIMENT" => EvidenceProfile { coverage: 0.3, depth: 1.0, integration: 0.45 },
        "PROJECT" => EvidenceProfile { coverage: 0.4, depth: 1.0, integration: 1.0 },
        "DOCUMENTATION" => EvidenceProfile { coverage: 0.55, depth: 0.6, integration: 0.35 },
        "RESEARCH" => EvidenceProfile { coverage: 0.7, depth: 0.45, integration: 0.3 },
        "OTHER" => EvidenceProfile { coverage: 0.3, depth: 0.3, integration: 0.1 },
        _ => return None,
    })
}

pub fn repetition_factor(index: usize) -> f64 {
    match index { 0 => 1.0, 1 => 0.75, 2 => 0.55, 3 => 0.40, 4 => 0.30, value => (0.30 / ((value - 3) as f64).sqrt()).max(0.10) }
}
pub fn diversity_factor(distinct_types: usize) -> f64 { (1.0 + distinct_types.saturating_sub(1) as f64 * 0.04).min(1.24) }
pub fn saturation_factor(current: f64) -> f64 { ((100.0 - current.clamp(0.0, 100.0)) / 100.0).max(0.02) }

fn active_evidence_filter() -> &'static str {
    "event_type='activity_completed' AND NOT EXISTS (SELECT 1 FROM knowledge_events reversal WHERE reversal.reversal_of_event_id=knowledge_events.id)"
}

fn current_metric(connection: &Connection, knowledge_id: &str) -> Result<(f64, f64), String> {
    connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id=?1", [knowledge_id], |row| Ok((row.get(0)?, row.get(1)?))).map_err(|e| e.to_string())
}

fn ancestors(connection: &Connection, node_id: &str) -> Result<Vec<(String, usize)>, String> {
    let mut result = vec![(node_id.to_string(), 0)];
    let mut current = node_id.to_string();
    let mut seen = HashSet::from([current.clone()]);
    for depth in 1..HIERARCHY_FACTORS.len() {
        let parent = connection.query_row("SELECT parent_id FROM knowledge_areas WHERE id=?1", [&current], |row| row.get::<_, Option<String>>(0)).optional().map_err(|e| e.to_string())?.flatten();
        let Some(parent) = parent else { break };
        if !seen.insert(parent.clone()) { return Err("Hierarquia de conhecimento contém ciclo".into()); }
        result.push((parent.clone(), depth)); current = parent;
    }
    Ok(result)
}

fn load_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<KnowledgeEvent> {
    Ok(KnowledgeEvent { id: row.get(0)?, knowledge_node_id: row.get(1)?, source_type: row.get(2)?, source_id: row.get(3)?, event_type: row.get(4)?, coverage_delta: row.get(5)?, depth_delta: row.get(6)?, integration_delta: row.get(7)?, description: row.get(8)?, created_at: row.get(9)?, activity_type: row.get(10)?, roadmap_id: row.get(11)?, topic_id: row.get(12)?, evidence_cycle: row.get(13)?, formula_version: row.get(14)?, coverage_impact: row.get(15)?, depth_impact: row.get(16)?, integration_impact: row.get(17)?, metadata_json: row.get(18)?, reversal_of_event_id: row.get(19)? })
}

pub fn list_events(connection: &Connection) -> Result<Vec<KnowledgeEvent>, String> {
    let mut statement = connection.prepare("SELECT id,knowledge_node_id,source_type,source_id,event_type,coverage_delta,depth_delta,integration_delta,description,created_at,activity_type,roadmap_id,topic_id,evidence_cycle,formula_version,coverage_impact,depth_impact,integration_impact,metadata_json,reversal_of_event_id FROM knowledge_events ORDER BY created_at DESC,rowid DESC").map_err(|e| e.to_string())?;
    let result=statement.query_map([], load_event).map_err(|e| e.to_string())?.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string());
    result
}

fn insert_event(connection: &Connection, node: &str, activity_id: &str, event_type: &str, kind: &str, roadmap_id: &str, topic_id: &str, cycle: i64, coverage: f64, depth: f64, integration: f64, description: &str, metadata: &str, reversal: Option<&str>) -> Result<(), String> {
    connection.execute("INSERT INTO knowledge_events(id,knowledge_node_id,source_type,source_id,event_type,coverage_delta,depth_delta,integration_delta,description,activity_type,roadmap_id,topic_id,evidence_cycle,formula_version,coverage_impact,depth_impact,integration_impact,metadata_json,reversal_of_event_id) VALUES (lower(hex(randomblob(16))),?1,'roadmap',?2,?3,round(?4),round(?5),round(?6),?7,?8,?9,?10,?11,?12,?4,?5,?6,?13,?14)", params![node,activity_id,event_type,coverage,depth,integration,description,kind,roadmap_id,topic_id,cycle,FORMULA_VERSION,metadata,reversal]).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn complete_activity(connection: &Connection, activity_id: &str) -> Result<(), String> {
    let (title, kind, topic_id, roadmap_id): (String, String, String, String) = connection.query_row("SELECT activity.title,activity.activity_type,topic.id,stage.roadmap_id FROM roadmap_activities activity JOIN roadmap_topics topic ON topic.id=activity.topic_id JOIN roadmap_stages stage ON stage.id=topic.stage_id WHERE activity.id=?1", [activity_id], |row| Ok((row.get(0)?,row.get(1)?,row.get(2)?,row.get(3)?))).map_err(|e| e.to_string())?;
    let profile = evidence_profile(&kind).ok_or_else(|| "Tipo de evidência inválido".to_string())?;
    let mut statement = connection.prepare("SELECT knowledge_node_id,role FROM activity_knowledge_nodes WHERE activity_id=?1 ORDER BY CASE role WHEN 'primary' THEN 0 ELSE 1 END,knowledge_node_id").map_err(|e| e.to_string())?;
    let direct = statement.query_map([activity_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?))).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    if direct.is_empty() { return Err(format!("A atividade {title} não possui conhecimento relacionado")); }
    let cycle: i64 = connection.query_row("SELECT COALESCE(MAX(evidence_cycle),0)+1 FROM knowledge_events WHERE source_type='roadmap' AND source_id=?1 AND event_type='activity_completed'", [activity_id], |row| row.get(0)).map_err(|e| e.to_string())?;
    let secondary_count = direct.iter().filter(|(_,role)| role == "secondary").count();
    let secondary_factor = if secondary_count <= 2 { 0.5 } else { 1.0 / secondary_count as f64 };
    let mut targets: HashMap<String,(f64,bool)> = HashMap::new();
    for (node, role) in &direct {
        let role_factor = if role == "primary" { 1.0 } else { secondary_factor };
        for (target, depth) in ancestors(connection, node)? {
            let factor = role_factor * HIERARCHY_FACTORS[depth];
            let direct_target = depth == 0;
            targets.entry(target).and_modify(|entry| { if factor > entry.0 { *entry = (factor,direct_target); } }).or_insert((factor,direct_target));
        }
    }
    for (node,(distribution,is_direct)) in targets {
        let repetition: i64 = connection.query_row(&format!("SELECT COUNT(*) FROM knowledge_events WHERE knowledge_node_id=?1 AND activity_type=?2 AND {}", active_evidence_filter()), params![node,kind], |row| row.get(0)).map_err(|e| e.to_string())?;
        let distinct: i64 = connection.query_row(&format!("SELECT COUNT(DISTINCT activity_type) FROM knowledge_events WHERE knowledge_node_id=?1 AND {}", active_evidence_filter()), [&node], |row| row.get(0)).map_err(|e| e.to_string())?;
        let rep = repetition_factor(repetition as usize); let diversity = diversity_factor((distinct + 1) as usize);
        let (current_coverage,current_depth) = current_metric(connection,&node)?;
        let coverage = profile.coverage * BASE_IMPACT * rep * diversity * distribution * saturation_factor(current_coverage);
        let depth = profile.depth * BASE_IMPACT * rep * diversity * distribution * saturation_factor(current_depth);
        let integration = if direct.len() > 1 && is_direct { profile.integration * BASE_IMPACT * rep * diversity * distribution / direct.iter().map(|(_,role)| if role == "primary" {1.0} else {secondary_factor}).sum::<f64>() } else { 0.0 };
        let metadata = serde_json::json!({"activityType":kind,"repetitionIndex":repetition+1,"repetitionFactor":rep,"diversityFactor":diversity,"distributionFactor":distribution,"formulaVersion":FORMULA_VERSION,"profile":profile}).to_string();
        insert_event(connection,&node,activity_id,"activity_completed",&kind,&roadmap_id,&topic_id,cycle,coverage,depth,integration,&title,&metadata,None)?;
    }
    Ok(())
}

pub fn reopen_activity(connection: &Connection, activity_id: &str) -> Result<(), String> {
    let mut statement = connection.prepare("SELECT id,knowledge_node_id,activity_type,roadmap_id,topic_id,evidence_cycle,coverage_impact,depth_impact,integration_impact,description FROM knowledge_events event WHERE source_type='roadmap' AND source_id=?1 AND event_type='activity_completed' AND NOT EXISTS (SELECT 1 FROM knowledge_events reversal WHERE reversal.reversal_of_event_id=event.id)").map_err(|e| e.to_string())?;
    let originals = statement.query_map([activity_id], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,Option<String>>(2)?.unwrap_or_else(|| "OTHER".into()),row.get::<_,Option<String>>(3)?.unwrap_or_default(),row.get::<_,Option<String>>(4)?.unwrap_or_default(),row.get::<_,i64>(5)?,row.get::<_,f64>(6)?,row.get::<_,f64>(7)?,row.get::<_,f64>(8)?,row.get::<_,String>(9)?))).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    // Atividades concluídas antes da v0.8.3 não possuíam eventos; reabri-las é seguro e não exige reversão.
    if originals.is_empty() { return Ok(()); }
    for (original,node,kind,roadmap,topic,cycle,coverage,depth,integration,title) in originals {
        let metadata = serde_json::json!({"reversalOf":original,"formulaVersion":FORMULA_VERSION}).to_string();
        insert_event(connection,&node,activity_id,"activity_reopened",&kind,&roadmap,&topic,cycle,-coverage,-depth,-integration,&format!("Reabertura: {title}"),&metadata,Some(&original))?;
    }
    Ok(())
}

pub fn manual_adjustment(connection:&Connection,knowledge_id:&str,coverage:i64,depth:i64,reason:&str)->Result<LearningMutation,String>{
    let previous:HashSet<String>=list_events(connection)?.into_iter().map(|event|event.id).collect();
    let (current_coverage,current_depth)=current_metric(connection,knowledge_id)?;
    let coverage_impact=coverage as f64-current_coverage; let depth_impact=depth as f64-current_depth;
    let metadata=serde_json::json!({"requestedCoverage":coverage,"requestedDepth":depth,"previousCoverage":current_coverage,"previousDepth":current_depth,"formulaVersion":FORMULA_VERSION}).to_string();
    connection.execute("INSERT INTO knowledge_events(id,knowledge_node_id,source_type,source_id,event_type,coverage_delta,depth_delta,integration_delta,description,formula_version,coverage_impact,depth_impact,integration_impact,metadata_json) VALUES (lower(hex(randomblob(16))),?1,'manual',lower(hex(randomblob(16))),'manual_adjustment',?2,?3,0,?4,?5,?2,?3,0,?6)",params![knowledge_id,coverage_impact,depth_impact,reason,FORMULA_VERSION,metadata]).map_err(|e|e.to_string())?;
    let mut result=recalculate(connection,reason)?;
    result.created_events=list_events(connection)?.into_iter().filter(|event|!previous.contains(&event.id)).collect();
    Ok(result)
}

pub fn recalculate(connection: &Connection, reason: &str) -> Result<LearningMutation, String> {
    let mut affected = Vec::new();
    let mut statement = connection.prepare("SELECT baseline.knowledge_id,baseline.coverage,baseline.depth,area.coverage,area.depth FROM knowledge_baselines baseline JOIN knowledge_areas area ON area.id=baseline.knowledge_id ORDER BY baseline.knowledge_id").map_err(|e| e.to_string())?;
    let nodes = statement.query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,f64>(1)?,row.get::<_,f64>(2)?,row.get::<_,i64>(3)?,row.get::<_,i64>(4)?))).map_err(|e| e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e| e.to_string())?;
    for (node,base_c,base_d,old_c,old_d) in nodes {
        let (delta_c,delta_d):(f64,f64) = connection.query_row("SELECT COALESCE(SUM(coverage_impact),0),COALESCE(SUM(depth_impact),0) FROM knowledge_events WHERE knowledge_node_id=?1", [&node], |row| Ok((row.get(0)?,row.get(1)?))).map_err(|e| e.to_string())?;
        let new_c=(base_c+delta_c).clamp(0.0,100.0).round() as i64; let new_d=(base_d+delta_d).clamp(0.0,100.0).round() as i64;
        if new_c != old_c || new_d != old_d {
            connection.execute("UPDATE knowledge_areas SET coverage=?1,depth=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3",params![new_c,new_d,node]).map_err(|e|e.to_string())?;
            connection.execute("INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason) VALUES (?1,?2,?3,?4)",params![node,new_c,new_d,reason]).map_err(|e|e.to_string())?;
            affected.push(node);
        }
    }
    let (baseline,impact):(f64,f64)=connection.query_row("SELECT integration_baseline,(SELECT COALESCE(SUM(integration_impact),0) FROM knowledge_events) FROM learning_engine_state WHERE id=1",[],|row|Ok((row.get(0)?,row.get(1)?))).map_err(|e|e.to_string())?;
    let integration=(baseline+impact).clamp(0.0,100.0);
    connection.execute("UPDATE app_metrics SET value=?1 WHERE key='integration'",[integration]).map_err(|e|e.to_string())?;
    derive_topic_mastery(connection)?;
    connection.execute("UPDATE learning_engine_state SET status='ready',last_error=NULL,last_recalculated_at=CURRENT_TIMESTAMP,formula_version=?1 WHERE id=1",[FORMULA_VERSION]).map_err(|e|e.to_string())?;
    Ok(LearningMutation { created_events: vec![], affected_knowledge_ids: affected, integration })
}

fn derive_topic_mastery(connection: &Connection) -> Result<(), String> {
    let mut statement=connection.prepare("SELECT id,knowledge_node_id FROM roadmap_topics").map_err(|e|e.to_string())?;
    let topics=statement.query_map([],|row|Ok((row.get::<_,String>(0)?,row.get::<_,Option<String>>(1)?))).map_err(|e|e.to_string())?.collect::<Result<Vec<_>,_>>().map_err(|e|e.to_string())?;
    for (topic,node) in topics {
        let mut kinds_stmt=connection.prepare(&format!("SELECT DISTINCT activity_type FROM knowledge_events WHERE topic_id=?1 AND {}",active_evidence_filter())).map_err(|e|e.to_string())?;
        let kinds:HashSet<String>=kinds_stmt.query_map([&topic],|row|row.get(0)).map_err(|e|e.to_string())?.collect::<Result<_,_>>().map_err(|e|e.to_string())?;
        let mut state="NOT_STARTED";
        if kinds.iter().any(|k|["READING","LESSON","RESEARCH"].contains(&k.as_str())) { state="EXPOSED"; }
        if kinds.iter().any(|k|["QUIZ","EXERCISE"].contains(&k.as_str())) { state="UNDERSTOOD"; }
        if kinds.iter().any(|k|["SIMULATION","EXPERIMENT"].contains(&k.as_str())) { state="PRACTICED"; }
        if kinds.contains("PROJECT") { state="APPLIED"; }
        if let Some(node)=node { let (c,d)=current_metric(connection,&node)?; if c>=85.0 && d>=80.0 && kinds.len()>=5 && kinds.contains("PROJECT") { state="MASTERED"; } }
        connection.execute("UPDATE roadmap_topics SET topic_state=?1 WHERE id=?2",params![state,topic]).map_err(|e|e.to_string())?;
    }
    Ok(())
}

pub fn status(connection:&Connection)->Result<LearningEngineStatus,String>{
    connection.query_row("SELECT formula_version,integration_baseline,(SELECT value FROM app_metrics WHERE key='integration'),(SELECT COUNT(*) FROM knowledge_events),last_recalculated_at,status,last_error FROM learning_engine_state WHERE id=1",[],|row|Ok(LearningEngineStatus{formula_version:row.get(0)?,integration_baseline:row.get(1)?,current_integration:row.get(2)?,event_count:row.get(3)?,last_recalculated_at:row.get(4)?,status:row.get(5)?,last_error:row.get(6)?})).map_err(|e|e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn profiles_are_distinct(){assert!(evidence_profile("READING").unwrap().coverage>evidence_profile("READING").unwrap().depth);assert!(evidence_profile("EXPERIMENT").unwrap().depth>evidence_profile("EXPERIMENT").unwrap().coverage);assert_eq!(evidence_profile("PROJECT").unwrap().integration,1.0);}
    #[test] fn diminishing_diversity_and_saturation_are_bounded(){assert!(repetition_factor(0)>repetition_factor(9));assert!(diversity_factor(10)<=1.24);assert!(saturation_factor(90.0)<saturation_factor(20.0));}
}
