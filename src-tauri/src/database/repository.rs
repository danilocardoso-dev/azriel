use super::models::*;
use rusqlite::{params, Connection, OptionalExtension};

const KNOWLEDGE_SEED: &[(&str, &str, &str, &str, i64, i64, &str)] = &[
    (
        "software-engineering",
        "Engenharia de Software",
        "Tecnologia",
        "Arquitetura de software, código, testes, concorrência, performance, APIs e sistemas distribuídos.",
        0,
        0,
        "high",
    ),
    (
        "systems-infrastructure",
        "Sistemas e Infraestrutura",
        "Tecnologia",
        "Linux, servidores, containers, redes, cloud, platform engineering e deploy.",
        0,
        0,
        "high",
    ),
    (
        "cybersecurity",
        "Cybersecurity",
        "Segurança",
        "Security engineering, AppSec, hardening, threat modeling e segurança ofensiva e defensiva.",
        0,
        0,
        "high",
    ),
    (
        "dfir",
        "Forense Digital & Incident Response",
        "Segurança",
        "Evidências, investigação, memória, logs, timelines e resposta a incidentes.",
        0,
        0,
        "high",
    ),
    (
        "reliability-observability",
        "Reliability & Observabilidade",
        "Operação",
        "SRE, métricas, logs, traces, SLO/SLI, incidentes e performance operacional.",
        0,
        0,
        "high",
    ),
    (
        "ai-engineering",
        "Engenharia de IA",
        "Inteligência Artificial",
        "LLMs, inferência, contexto, tools, agents, RAG, avaliação, deployment e AI security.",
        0,
        0,
        "high",
    ),
    (
        "automation",
        "Automação",
        "Operação",
        "Integrações, workflows, scripting, IoT, tarefas, orquestração e automação operacional.",
        0,
        0,
        "high",
    ),
    (
        "english",
        "Inglês",
        "Idiomas",
        "Compreensão, listening, speaking, writing, pronúncia e inglês técnico.",
        0,
        0,
        "high",
    ),
    (
        "technology-business",
        "Empreendedorismo Tecnológico",
        "Negócios",
        "Produto, problema, cliente, vendas B2B, validação, operação, métricas e estratégia.",
        0,
        0,
        "high",
    ),
    (
        "engineering-leadership",
        "Liderança e Decisão Técnica",
        "Liderança",
        "Trade-offs, ADRs, comunicação técnica, priorização, mentoring e decisões de engenharia.",
        0,
        0,
        "high",
    ),
];

const PROJECT_SEED: &[(&str, &str, &str, &str, &str, &str, i64, &str)] = &[
    (
        "azriel",
        "Azriel",
        "Sistema pessoal",
        "Central pessoal de inteligência, pesquisa, engenharia e evolução.",
        "Orquestrar projetos, formação, conhecimento e lacunas em uma interface única.",
        "active",
        40,
        "Consolidar a persistência local da v0.5.",
    ),
];

const RELATION_SEED: &[(&str, &[&str])] = &[
    (
        "azriel",
        &["software-engineering", "ai-engineering", "automation"],
    ),
];

const EDUCATION_SEED: &[(
    &str,
    &str,
    &str,
    &str,
    &str,
    Option<&str>,
    Option<&str>,
    &str,
    &str,
    &str,
)] = &[
    (
        "tech-base",
        "Tecnologia, Software e Cibersegurança",
        "course",
        "",
        "completed",
        None,
        None,
        "BASE ATUAL",
        "Fundação existente em desenvolvimento, computação e investigação tecnológica.",
        "Software|Cibersegurança|Computação",
    ),
    (
        "biotech",
        "Pós-graduação em Biotecnologia",
        "postgraduate",
        "",
        "in_progress",
        None,
        Some("2026-11-30"),
        "ATÉ NOV/2026",
        "Ampliação da base em biologia aplicada e tecnologias biológicas.",
        "Biotecnologia|Biologia",
    ),
    (
        "iot",
        "Pós-graduação em Internet das Coisas",
        "postgraduate",
        "",
        "in_progress",
        None,
        Some("2026-11-30"),
        "ATÉ NOV/2026",
        "Ponte entre software, sensores, dispositivos e sistemas físicos.",
        "IoT|Eletrônica|Automação",
    ),
    (
        "big-data",
        "Pós-graduação em Big Data Analytics",
        "postgraduate",
        "",
        "in_progress",
        None,
        Some("2026-11-30"),
        "ATÉ NOV/2026",
        "Dados, processamento, análise, estatística e visualização.",
        "Big Data|Dados|Bioinformática",
    ),
    (
        "biomedicine",
        "Biomedicina",
        "graduation",
        "",
        "planned",
        Some("2027-01-01"),
        None,
        "2027",
        "Próxima formação, com foco em genética, molecular, fisiologia e pesquisa.",
        "Genética|Biologia Molecular|Bioquímica",
    ),
    (
        "mechatronics",
        "Engenharia Mecatrônica",
        "graduation",
        "",
        "planned",
        None,
        None,
        "ETAPA FUTURA",
        "Integração de mecânica, eletrônica, controle, robótica e sistemas embarcados.",
        "Mecânica|Eletrônica|Robótica",
    ),
    (
        "masters",
        "Mestrado interdisciplinar",
        "masters",
        "",
        "planned",
        None,
        None,
        "POSTERIOR",
        "Tema definido por uma lacuna ou problema real identificado durante o percurso.",
        "Pesquisa|Integração",
    ),
];

pub fn seed(connection: &mut Connection) -> Result<(), String> {
    let seeded = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM _azriel_seeds WHERE key='v0.5-initial')",
            [],
            |row| row.get::<_, bool>(0),
        )
        .map_err(err)?;
    if seeded {
        return Ok(());
    }
    let transaction = connection.transaction().map_err(err)?;
    for area in KNOWLEDGE_SEED {
        transaction.execute(
            "INSERT OR IGNORE INTO knowledge_areas(id,name,category,description,coverage,depth,priority) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![area.0, area.1, area.2, area.3, area.4, area.5, area.6],
        ).map_err(err)?;
        transaction.execute(
            "INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason)
             SELECT ?1,?2,?3,'Seed inicial da v0.5' WHERE NOT EXISTS (SELECT 1 FROM knowledge_history WHERE knowledge_id=?1)",
            params![area.0, area.4, area.5],
        ).map_err(err)?;
    }
    for project in PROJECT_SEED {
        transaction.execute(
            "INSERT OR IGNORE INTO projects(id,name,category,description,objective,status,progress,next_step) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            params![project.0, project.1, project.2, project.3, project.4, project.5, project.6, project.7],
        ).map_err(err)?;
    }
    for (project_id, areas) in RELATION_SEED {
        for knowledge_id in *areas {
            transaction.execute("INSERT OR IGNORE INTO project_knowledge(project_id,knowledge_id) VALUES (?1,?2)", params![project_id, knowledge_id]).map_err(err)?;
        }
    }
    for item in EDUCATION_SEED {
        transaction.execute(
            "INSERT OR IGNORE INTO education(id,name,type,institution,status,start_date,expected_end_date,period,description,domains) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",
            params![item.0,item.1,item.2,item.3,item.4,item.5,item.6,item.7,item.8,item.9],
        ).map_err(err)?;
    }
    transaction.execute("INSERT OR IGNORE INTO app_metrics(key,value,formula_note) VALUES ('integration',0,'Fórmula será definida em versão futura')", []).map_err(err)?;
    transaction
        .execute("INSERT INTO _azriel_seeds(key) VALUES ('v0.5-initial')", [])
        .map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn list_knowledge(connection: &Connection) -> Result<Vec<KnowledgeArea>, String> {
    let mut statement = connection.prepare("SELECT id,name,category,description,coverage,depth,priority,node_type,parent_id,created_at,updated_at FROM knowledge_areas ORDER BY category,node_type,name").map_err(err)?;
    let rows = statement
        .query_map([], |row| {
            Ok(KnowledgeArea {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                coverage: row.get(4)?,
                depth: row.get(5)?,
                priority: row.get(6)?,
                node_type: row.get(7)?,
                parent_id: row.get(8)?,
                project_ids: vec![],
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(err)?;
    let mut areas = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for area in &mut areas {
        area.project_ids = collect_strings(
            connection,
            "SELECT project_id FROM project_knowledge WHERE knowledge_id=?1 ORDER BY project_id",
            &area.id,
        )?;
    }
    Ok(areas)
}

pub fn get_knowledge(connection: &Connection, id: &str) -> Result<Option<KnowledgeArea>, String> {
    Ok(list_knowledge(connection)?
        .into_iter()
        .find(|area| area.id == id))
}

pub fn list_history(
    connection: &Connection,
    knowledge_id: &str,
) -> Result<Vec<KnowledgeHistory>, String> {
    let mut statement = connection.prepare("SELECT id,knowledge_id,coverage,depth,recorded_at,reason FROM knowledge_history WHERE knowledge_id=?1 ORDER BY recorded_at DESC,id DESC").map_err(err)?;
    let rows = statement
        .query_map([knowledge_id], |row| {
            Ok(KnowledgeHistory {
                id: row.get(0)?,
                knowledge_id: row.get(1)?,
                coverage: row.get(2)?,
                depth: row.get(3)?,
                recorded_at: row.get(4)?,
                reason: row.get(5)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn save_knowledge(connection: &mut Connection, input: &KnowledgeInput) -> Result<(), String> {
    validate_knowledge(input)?;
    let current = connection
        .query_row(
            "SELECT coverage,depth FROM knowledge_areas WHERE id=?1",
            [&input.id],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()
        .map_err(err)?;
    if let Some((coverage, depth)) = current {
        if coverage != input.coverage || depth != input.depth {
            return Err("Use updateKnowledgeMetrics para alterar cobertura ou profundidade".into());
        }
        connection.execute("UPDATE knowledge_areas SET name=?1,category=?2,description=?3,priority=?4,node_type=?5,parent_id=?6,updated_at=CURRENT_TIMESTAMP WHERE id=?7", params![input.name,input.category,input.description,input.priority,input.node_type,input.parent_id,input.id]).map_err(err)?;
        return Ok(());
    }
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute("INSERT INTO knowledge_areas(id,name,category,description,coverage,depth,priority,node_type,parent_id) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", params![input.id,input.name,input.category,input.description,input.coverage,input.depth,input.priority,input.node_type,input.parent_id]).map_err(err)?;
    transaction.execute("INSERT OR IGNORE INTO knowledge_baselines(knowledge_id,coverage,depth) VALUES (?1,?2,?3)", params![input.id,input.coverage,input.depth]).map_err(err)?;
    transaction.execute("INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason) VALUES (?1,?2,?3,'Criação da área')", params![input.id,input.coverage,input.depth]).map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn delete_knowledge(connection: &Connection, id: &str) -> Result<(), String> {
    let activity_links = connection
        .query_row(
            "SELECT COUNT(*) FROM activity_knowledge_nodes WHERE knowledge_node_id=?1",
            [id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(err)?;
    let evidence_events = connection
        .query_row(
            "SELECT COUNT(*) FROM knowledge_events WHERE knowledge_node_id=?1",
            [id],
            |row| row.get::<_, i64>(0),
        )
        .map_err(err)?;
    if activity_links > 0 || evidence_events > 0 {
        return Err(format!(
            "O conhecimento possui {activity_links} vínculo(s) com atividades e {evidence_events} evento(s) de aprendizagem. Remova ou migre essas referências antes de excluir."
        ));
    }
    connection
        .execute("DELETE FROM knowledge_areas WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn update_metrics(
    connection: &mut Connection,
    input: &MetricsInput,
) -> Result<KnowledgeArea, String> {
    percent(input.coverage, "cobertura")?;
    percent(input.depth, "profundidade")?;
    if input.reason.trim().is_empty() {
        return Err("Informe o motivo da atualização".into());
    }
    let exists = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM knowledge_areas WHERE id=?1)",
            [&input.knowledge_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(err)?;
    if !exists {
        return Err("Área de conhecimento não encontrada".into());
    }
    let transaction = connection.transaction().map_err(err)?;
    super::learning_engine::manual_adjustment(
        &transaction,
        &input.knowledge_id,
        input.coverage,
        input.depth,
        input.reason.trim(),
    )?;
    transaction.commit().map_err(err)?;
    list_knowledge(connection)?
        .into_iter()
        .find(|area| area.id == input.knowledge_id)
        .ok_or_else(|| "Área atualizada não encontrada".into())
}

pub fn list_projects(connection: &Connection) -> Result<Vec<Project>, String> {
    let mut statement = connection.prepare("SELECT id,name,description,objective,category,status,progress,next_step,created_at,updated_at FROM projects ORDER BY name").map_err(err)?;
    let rows = statement
        .query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                objective: row.get(3)?,
                category: row.get(4)?,
                status: row.get(5)?,
                progress: row.get(6)?,
                next_step: row.get(7)?,
                knowledge_area_ids: vec![],
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(err)?;
    let mut projects = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for project in &mut projects {
        project.knowledge_area_ids = collect_strings(
            connection,
            "SELECT knowledge_id FROM project_knowledge WHERE project_id=?1 ORDER BY knowledge_id",
            &project.id,
        )?;
    }
    Ok(projects)
}

pub fn get_project(connection: &Connection, id: &str) -> Result<Option<Project>, String> {
    Ok(list_projects(connection)?
        .into_iter()
        .find(|project| project.id == id))
}

pub fn save_project(connection: &mut Connection, input: &ProjectInput) -> Result<(), String> {
    validate_project(input)?;
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute(
        "INSERT INTO projects(id,name,description,objective,category,status,progress,next_step) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,objective=excluded.objective,category=excluded.category,status=excluded.status,progress=excluded.progress,next_step=excluded.next_step,updated_at=CURRENT_TIMESTAMP",
        params![input.id,input.name,input.description,input.objective,input.category,input.status,input.progress,input.next_step],
    ).map_err(err)?;
    transaction
        .execute(
            "DELETE FROM project_knowledge WHERE project_id=?1",
            [&input.id],
        )
        .map_err(err)?;
    for knowledge_id in &input.knowledge_area_ids {
        transaction
            .execute(
                "INSERT INTO project_knowledge(project_id,knowledge_id) VALUES (?1,?2)",
                params![input.id, knowledge_id],
            )
            .map_err(err)?;
    }
    transaction.commit().map_err(err)
}

pub fn delete_project(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM projects WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

pub fn list_education(connection: &Connection) -> Result<Vec<EducationItem>, String> {
    let mut statement = connection.prepare("SELECT id,name,type,institution,status,start_date,expected_end_date,completed_at,description,period,domains,created_at,updated_at FROM education ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'planned' THEN 1 ELSE 2 END,name").map_err(err)?;
    let rows = statement
        .query_map([], |row| {
            let domains: String = row.get(10)?;
            Ok(EducationItem {
                id: row.get(0)?,
                name: row.get(1)?,
                kind: row.get(2)?,
                institution: row.get(3)?,
                status: row.get(4)?,
                start_date: row.get(5)?,
                expected_end_date: row.get(6)?,
                completed_at: row.get(7)?,
                description: row.get(8)?,
                period: row.get(9)?,
                domains: split_domains(&domains),
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn save_education(connection: &Connection, input: &EducationInput) -> Result<(), String> {
    validate_education(input)?;
    connection.execute(
        "INSERT INTO education(id,name,type,institution,status,start_date,expected_end_date,completed_at,description,period,domains) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,type=excluded.type,institution=excluded.institution,status=excluded.status,start_date=excluded.start_date,expected_end_date=excluded.expected_end_date,completed_at=excluded.completed_at,description=excluded.description,period=excluded.period,domains=excluded.domains,updated_at=CURRENT_TIMESTAMP",
        params![input.id,input.name,input.kind,input.institution,input.status,input.start_date,input.expected_end_date,input.completed_at,input.description,input.period,input.domains.join("|")],
    ).map_err(err)?;
    Ok(())
}

pub fn delete_education(connection: &Connection, id: &str) -> Result<(), String> {
    connection
        .execute("DELETE FROM education WHERE id=?1", [id])
        .map_err(err)?;
    Ok(())
}

fn collect_strings(connection: &Connection, sql: &str, id: &str) -> Result<Vec<String>, String> {
    let mut statement = connection.prepare(sql).map_err(err)?;
    let rows = statement.query_map([id], |row| row.get(0)).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn split_domains(value: &str) -> Vec<String> {
    value
        .split('|')
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}
fn err(error: rusqlite::Error) -> String {
    error.to_string()
}
fn percent(value: i64, field: &str) -> Result<(), String> {
    if (0..=100).contains(&value) {
        Ok(())
    } else {
        Err(format!("{field} deve estar entre 0 e 100"))
    }
}
fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}
fn required(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} é obrigatório"))
    } else {
        Ok(())
    }
}
fn validate_knowledge(input: &KnowledgeInput) -> Result<(), String> {
    if !valid_id(&input.id) {
        return Err("ID inválido".into());
    }
    required(&input.name, "Nome")?;
    required(&input.category, "Categoria")?;
    percent(input.coverage, "Cobertura")?;
    percent(input.depth, "Profundidade")?;
    if !["critical", "high", "medium", "low"].contains(&input.priority.as_str()) {
        return Err("Prioridade inválida".into());
    }
    if !["area", "discipline", "topic", "competency"].contains(&input.node_type.as_str()) {
        return Err("Tipo de conhecimento inválido".into());
    }
    if input.parent_id.as_deref() == Some(input.id.as_str()) {
        return Err("Um conhecimento não pode ser pai de si mesmo".into());
    }
    Ok(())
}
fn validate_project(input: &ProjectInput) -> Result<(), String> {
    if !valid_id(&input.id) {
        return Err("ID inválido".into());
    }
    required(&input.name, "Nome")?;
    required(&input.category, "Categoria")?;
    percent(input.progress, "Progresso")?;
    if !["active", "research", "paused", "planned", "completed"].contains(&input.status.as_str()) {
        return Err("Status inválido".into());
    }
    Ok(())
}
fn validate_education(input: &EducationInput) -> Result<(), String> {
    if !valid_id(&input.id) {
        return Err("ID inválido".into());
    }
    required(&input.name, "Nome")?;
    if ![
        "graduation",
        "postgraduate",
        "masters",
        "doctorate",
        "course",
        "certification",
    ]
    .contains(&input.kind.as_str())
    {
        return Err("Tipo inválido".into());
    }
    if !["completed", "in_progress", "planned"].contains(&input.status.as_str()) {
        return Err("Status inválido".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection
    }

    #[test]
    fn seed_is_idempotent() {
        let mut connection = database();
        seed(&mut connection).unwrap();
        assert_eq!(database::schema_version(&connection).unwrap(), 41);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM projects", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM education", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            7
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_history", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            KNOWLEDGE_SEED.len() as i64
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT expected_end_date FROM education WHERE id='biotech'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "2026-11-30"
        );
        assert_eq!(
            connection
                .query_row("SELECT type FROM education WHERE id='masters'", [], |row| {
                    row.get::<_, String>(0)
                })
                .unwrap(),
            "masters"
        );
        delete_project(&connection, "azriel").unwrap();
        seed(&mut connection).unwrap();
        assert!(get_project(&connection, "azriel").unwrap().is_none());
    }

    #[test]
    fn metric_update_writes_current_value_and_history() {
        let mut connection = database();
        let before = list_history(&connection, "ai-engineering").unwrap().len();
        let area = update_metrics(
            &mut connection,
            &MetricsInput {
                knowledge_id: "ai-engineering".into(),
                coverage: 67,
                depth: 44,
                reason: "Estudo validado".into(),
            },
        )
        .unwrap();
        assert_eq!((area.coverage, area.depth), (67, 44));
        let history = list_history(&connection, "ai-engineering").unwrap();
        assert_eq!(history.len(), before + 1);
        assert_eq!(history[0].reason, "Estudo validado");
    }

    #[test]
    fn project_relations_are_persisted() {
        let mut connection = database();
        let mut project = list_projects(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == "azriel")
            .unwrap();
        assert!(project
            .knowledge_area_ids
            .contains(&"ai-engineering".to_string()));
        project.knowledge_area_ids = vec!["cybersecurity".into(), "software-engineering".into()];
        save_project(
            &mut connection,
            &ProjectInput {
                id: project.id.clone(),
                name: project.name,
                description: project.description,
                objective: project.objective,
                category: project.category,
                status: project.status,
                progress: project.progress,
                next_step: project.next_step,
                knowledge_area_ids: project.knowledge_area_ids,
            },
        )
        .unwrap();
        let saved = list_projects(&connection)
            .unwrap()
            .into_iter()
            .find(|item| item.id == project.id)
            .unwrap();
        assert_eq!(
            saved.knowledge_area_ids,
            vec!["cybersecurity", "software-engineering"]
        );
    }

    #[test]
    fn custom_knowledge_can_be_created_and_deleted_but_evidence_is_protected() {
        let mut connection = database();
        let input = KnowledgeInput {
            id: "embedded-systems".into(),
            name: "Sistemas Embarcados".into(),
            category: "Tecnologia".into(),
            description: "Firmware, dispositivos e integração com hardware.".into(),
            coverage: 0,
            depth: 0,
            priority: "high".into(),
            node_type: "area".into(),
            parent_id: None,
        };

        save_knowledge(&mut connection, &input).unwrap();
        assert!(list_knowledge(&connection)
            .unwrap()
            .iter()
            .any(|area| area.id == input.id));

        connection.execute("INSERT INTO knowledge_events(id,knowledge_node_id,source_type,event_type,description) VALUES ('protected-event','embedded-systems','manual','manual_adjustment','Evidência')", []).unwrap();
        assert!(delete_knowledge(&connection, &input.id)
            .unwrap_err()
            .contains("evento(s) de aprendizagem"));

        connection
            .execute("DELETE FROM knowledge_events WHERE id='protected-event'", [])
            .unwrap();
        delete_knowledge(&connection, &input.id).unwrap();
        assert!(!list_knowledge(&connection)
            .unwrap()
            .iter()
            .any(|area| area.id == input.id));
    }
}
