use super::models::*;
use rusqlite::{params, Connection, OptionalExtension};

const KNOWLEDGE_SEED: &[(&str, &str, &str, &str, i64, i64, &str)] = &[
    ("cyber", "Cibersegurança", "Computação", "Segurança de sistemas, redes e aplicações.", 90, 80, "low"),
    ("programming", "Programação", "Computação", "Desenvolvimento de software e arquitetura de aplicações.", 80, 65, "low"),
    ("ai", "Inteligência Artificial", "Computação", "Modelos, agentes e aplicações de inteligência artificial.", 60, 40, "medium"),
    ("biology", "Biologia", "Biociências", "Fundamentos de sistemas biológicos.", 45, 30, "medium"),
    ("genetics", "Genética", "Biociências", "Herança, variação e organização genética.", 45, 30, "medium"),
    ("biomedicine", "Biomedicina", "Biociências", "Base biomédica aplicada à investigação e diagnóstico.", 35, 20, "medium"),
    ("biotechnology", "Biotecnologia", "Biociências", "Aplicação tecnológica de processos biológicos.", 35, 20, "medium"),
    ("molecular-biology", "Biologia Molecular", "Biociências", "Processos moleculares, DNA, RNA e proteínas.", 35, 20, "medium"),
    ("bioinformatics", "Bioinformática", "Biociências", "Computação aplicada a dados biológicos.", 35, 20, "medium"),
    ("physics", "Física Aplicada", "Fundamentos", "Fundamentos físicos voltados a problemas de engenharia.", 30, 20, "high"),
    ("mathematics", "Matemática", "Fundamentos", "Base quantitativa para ciência e engenharia.", 30, 20, "high"),
    ("statistics", "Estatística", "Fundamentos", "Análise estatística e inferência.", 0, 0, "high"),
    ("probability", "Probabilidade", "Fundamentos", "Modelagem de incerteza e eventos.", 0, 0, "high"),
    ("energy", "Energia", "Engenharia", "Captura, armazenamento e gestão de energia.", 25, 15, "medium"),
    ("robotics", "Robótica", "Engenharia", "Integração de percepção, controle e atuação.", 20, 10, "medium"),
    ("electronics", "Eletrônica", "Engenharia", "Circuitos e sistemas eletrônicos.", 20, 10, "high"),
    ("control", "Controle", "Engenharia", "Modelagem e controle de sistemas dinâmicos.", 20, 10, "medium"),
    ("automation", "Automação", "Engenharia", "Automação de processos e sistemas.", 20, 10, "medium"),
    ("iot", "Internet das Coisas", "Engenharia", "Sensores, conectividade e sistemas físicos conectados.", 20, 10, "medium"),
    ("materials", "Materiais", "Engenharia", "Propriedades e seleção de materiais.", 15, 8, "medium"),
    ("electrical", "Engenharia Elétrica", "Engenharia", "Sistemas elétricos e eletromagnetismo aplicado.", 15, 5, "critical"),
    ("mechanical", "Engenharia Mecânica", "Engenharia", "Mecânica, projeto e sistemas térmicos.", 15, 5, "critical"),
    ("big-data", "Big Data", "Dados", "Processamento e análise de grandes volumes de dados.", 0, 0, "high"),
    ("transcriptomics", "Transcriptômica", "Biociências", "Análise do conjunto de transcritos e expressão gênica.", 0, 0, "high"),
    ("data-analysis", "Análise de Dados", "Dados", "Preparação, exploração e interpretação de dados.", 0, 0, "high"),
    ("computer-graphics", "Computação Gráfica", "Computação", "Renderização e visualização tridimensional.", 0, 0, "medium"),
    ("3d-modeling", "Modelagem 3D", "Computação", "Criação e manipulação de modelos tridimensionais.", 0, 0, "medium"),
];

const PROJECT_SEED: &[(&str, &str, &str, &str, &str, &str, i64, &str)] = &[
    ("azriel", "Azriel", "Sistema pessoal", "Central pessoal de inteligência, pesquisa, engenharia e evolução.", "Orquestrar projetos, formação, conhecimento e lacunas em uma interface única.", "active", 40, "Consolidar a persistência local da v0.5."),
    ("arccore", "ArcCore", "Energia", "Pesquisa experimental sobre captura, armazenamento e gestão inteligente de energia.", "Transformar fundamentos de energia em um sistema experimental realista.", "research", 18, "Aprofundar circuitos e armazenamento."),
    ("mendel-lab", "Mendel Lab", "Genética educacional", "Laboratório virtual para simular herança genética e cruzamentos mendelianos.", "Visualizar gametas, genótipos, fenótipos e probabilidades.", "planned", 12, "Definir o primeiro modelo de cruzamento."),
    ("gene-expression", "Gene Expression Explorer", "Bioinformática", "Visualização e comparação de dados de expressão gênica.", "Tornar dados de expressão gênica visualmente compreensíveis.", "research", 16, "Selecionar um conjunto de dados educacional."),
    ("pcr-simulator", "PCR Simulator", "Biologia molecular", "Simulador educacional das etapas e variáveis conceituais da PCR.", "Explicar primers, ciclos, orientação das fitas e amplicons.", "planned", 8, "Modelar as etapas do ciclo térmico."),
    ("genescope", "GeneScope", "Variantes genéticas", "Explorador educacional para comparar sequências e classificar alterações.", "Investigar substituições, inserções, deleções, códons e aminoácidos.", "planned", 10, "Definir o formato das sequências de entrada."),
    ("atlas3d", "Atlas3D", "Visualização científica", "Projeto exploratório de visualização tridimensional.", "Avaliar aplicações futuras em visualização científica ou biomédica.", "paused", 28, "Aguardar um problema científico que justifique a retomada."),
];

const RELATION_SEED: &[(&str, &[&str])] = &[
    ("azriel", &["ai", "programming", "automation", "big-data"]),
    ("arccore", &["energy", "electronics", "control", "automation", "materials", "electrical", "physics"]),
    ("mendel-lab", &["genetics", "biology", "biomedicine", "biotechnology", "probability", "statistics", "programming"]),
    ("gene-expression", &["genetics", "molecular-biology", "biotechnology", "bioinformatics", "transcriptomics", "data-analysis", "programming"]),
    ("pcr-simulator", &["molecular-biology", "genetics", "biotechnology", "bioinformatics", "programming"]),
    ("genescope", &["genetics", "molecular-biology", "biotechnology", "bioinformatics", "programming"]),
    ("atlas3d", &["programming", "computer-graphics", "3d-modeling"]),
];

const EDUCATION_SEED: &[(&str, &str, &str, &str, &str, Option<&str>, Option<&str>, &str, &str, &str)] = &[
    ("tech-base", "Tecnologia, Software e Cibersegurança", "course", "", "completed", None, None, "BASE ATUAL", "Fundação existente em desenvolvimento, computação e investigação tecnológica.", "Software|Cibersegurança|Computação"),
    ("biotech", "Pós-graduação em Biotecnologia", "postgraduate", "", "in_progress", None, Some("2026-11-30"), "ATÉ NOV/2026", "Ampliação da base em biologia aplicada e tecnologias biológicas.", "Biotecnologia|Biologia"),
    ("iot", "Pós-graduação em Internet das Coisas", "postgraduate", "", "in_progress", None, Some("2026-11-30"), "ATÉ NOV/2026", "Ponte entre software, sensores, dispositivos e sistemas físicos.", "IoT|Eletrônica|Automação"),
    ("big-data", "Pós-graduação em Big Data Analytics", "postgraduate", "", "in_progress", None, Some("2026-11-30"), "ATÉ NOV/2026", "Dados, processamento, análise, estatística e visualização.", "Big Data|Dados|Bioinformática"),
    ("biomedicine", "Biomedicina", "graduation", "", "planned", Some("2027-01-01"), None, "2027", "Próxima formação, com foco em genética, molecular, fisiologia e pesquisa.", "Genética|Biologia Molecular|Bioquímica"),
    ("mechatronics", "Engenharia Mecatrônica", "graduation", "", "planned", None, None, "ETAPA FUTURA", "Integração de mecânica, eletrônica, controle, robótica e sistemas embarcados.", "Mecânica|Eletrônica|Robótica"),
    ("masters", "Mestrado interdisciplinar", "masters", "", "planned", None, None, "POSTERIOR", "Tema definido por uma lacuna ou problema real identificado durante o percurso.", "Pesquisa|Integração"),
];

pub fn seed(connection: &mut Connection) -> Result<(), String> {
    let seeded = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM _azriel_seeds WHERE key='v0.5-initial')", [], |row| row.get::<_, bool>(0),
    ).map_err(err)?;
    if seeded { return Ok(()); }
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
    transaction.execute("INSERT INTO _azriel_seeds(key) VALUES ('v0.5-initial')", []).map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn list_knowledge(connection: &Connection) -> Result<Vec<KnowledgeArea>, String> {
    let mut statement = connection.prepare("SELECT id,name,category,description,coverage,depth,priority,created_at,updated_at FROM knowledge_areas ORDER BY category,name").map_err(err)?;
    let rows = statement.query_map([], |row| Ok(KnowledgeArea {
        id: row.get(0)?, name: row.get(1)?, category: row.get(2)?, description: row.get(3)?, coverage: row.get(4)?, depth: row.get(5)?, priority: row.get(6)?, project_ids: vec![], created_at: row.get(7)?, updated_at: row.get(8)?,
    })).map_err(err)?;
    let mut areas = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for area in &mut areas {
        area.project_ids = collect_strings(connection, "SELECT project_id FROM project_knowledge WHERE knowledge_id=?1 ORDER BY project_id", &area.id)?;
    }
    Ok(areas)
}

pub fn get_knowledge(connection: &Connection, id: &str) -> Result<Option<KnowledgeArea>, String> {
    Ok(list_knowledge(connection)?.into_iter().find(|area| area.id == id))
}

pub fn list_history(connection: &Connection, knowledge_id: &str) -> Result<Vec<KnowledgeHistory>, String> {
    let mut statement = connection.prepare("SELECT id,knowledge_id,coverage,depth,recorded_at,reason FROM knowledge_history WHERE knowledge_id=?1 ORDER BY recorded_at DESC,id DESC").map_err(err)?;
    let rows = statement.query_map([knowledge_id], |row| Ok(KnowledgeHistory { id: row.get(0)?, knowledge_id: row.get(1)?, coverage: row.get(2)?, depth: row.get(3)?, recorded_at: row.get(4)?, reason: row.get(5)? })).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

pub fn save_knowledge(connection: &mut Connection, input: &KnowledgeInput) -> Result<(), String> {
    validate_knowledge(input)?;
    let current = connection.query_row("SELECT coverage,depth FROM knowledge_areas WHERE id=?1", [&input.id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?))).optional().map_err(err)?;
    if let Some((coverage, depth)) = current {
        if coverage != input.coverage || depth != input.depth {
            return Err("Use updateKnowledgeMetrics para alterar cobertura ou profundidade".into());
        }
        connection.execute("UPDATE knowledge_areas SET name=?1,category=?2,description=?3,priority=?4,updated_at=CURRENT_TIMESTAMP WHERE id=?5", params![input.name,input.category,input.description,input.priority,input.id]).map_err(err)?;
        return Ok(());
    }
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute("INSERT INTO knowledge_areas(id,name,category,description,coverage,depth,priority) VALUES (?1,?2,?3,?4,?5,?6,?7)", params![input.id,input.name,input.category,input.description,input.coverage,input.depth,input.priority]).map_err(err)?;
    transaction.execute("INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason) VALUES (?1,?2,?3,'Criação da área')", params![input.id,input.coverage,input.depth]).map_err(err)?;
    transaction.commit().map_err(err)
}

pub fn delete_knowledge(connection: &Connection, id: &str) -> Result<(), String> {
    connection.execute("DELETE FROM knowledge_areas WHERE id=?1", [id]).map_err(err)?;
    Ok(())
}

pub fn update_metrics(connection: &mut Connection, input: &MetricsInput) -> Result<KnowledgeArea, String> {
    percent(input.coverage, "cobertura")?; percent(input.depth, "profundidade")?;
    if input.reason.trim().is_empty() { return Err("Informe o motivo da atualização".into()); }
    let transaction = connection.transaction().map_err(err)?;
    let changed = transaction.execute("UPDATE knowledge_areas SET coverage=?1,depth=?2,updated_at=CURRENT_TIMESTAMP WHERE id=?3", params![input.coverage,input.depth,input.knowledge_id]).map_err(err)?;
    if changed == 0 { return Err("Área de conhecimento não encontrada".into()); }
    transaction.execute("INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason) VALUES (?1,?2,?3,?4)", params![input.knowledge_id,input.coverage,input.depth,input.reason.trim()]).map_err(err)?;
    transaction.commit().map_err(err)?;
    list_knowledge(connection)?.into_iter().find(|area| area.id == input.knowledge_id).ok_or_else(|| "Área atualizada não encontrada".into())
}

pub fn list_projects(connection: &Connection) -> Result<Vec<Project>, String> {
    let mut statement = connection.prepare("SELECT id,name,description,objective,category,status,progress,next_step,created_at,updated_at FROM projects ORDER BY name").map_err(err)?;
    let rows = statement.query_map([], |row| Ok(Project { id: row.get(0)?, name: row.get(1)?, description: row.get(2)?, objective: row.get(3)?, category: row.get(4)?, status: row.get(5)?, progress: row.get(6)?, next_step: row.get(7)?, knowledge_area_ids: vec![], created_at: row.get(8)?, updated_at: row.get(9)? })).map_err(err)?;
    let mut projects = rows.collect::<Result<Vec<_>, _>>().map_err(err)?;
    for project in &mut projects {
        project.knowledge_area_ids = collect_strings(connection, "SELECT knowledge_id FROM project_knowledge WHERE project_id=?1 ORDER BY knowledge_id", &project.id)?;
    }
    Ok(projects)
}

pub fn get_project(connection: &Connection, id: &str) -> Result<Option<Project>, String> {
    Ok(list_projects(connection)?.into_iter().find(|project| project.id == id))
}

pub fn save_project(connection: &mut Connection, input: &ProjectInput) -> Result<(), String> {
    validate_project(input)?;
    let transaction = connection.transaction().map_err(err)?;
    transaction.execute(
        "INSERT INTO projects(id,name,description,objective,category,status,progress,next_step) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,description=excluded.description,objective=excluded.objective,category=excluded.category,status=excluded.status,progress=excluded.progress,next_step=excluded.next_step,updated_at=CURRENT_TIMESTAMP",
        params![input.id,input.name,input.description,input.objective,input.category,input.status,input.progress,input.next_step],
    ).map_err(err)?;
    transaction.execute("DELETE FROM project_knowledge WHERE project_id=?1", [&input.id]).map_err(err)?;
    for knowledge_id in &input.knowledge_area_ids {
        transaction.execute("INSERT INTO project_knowledge(project_id,knowledge_id) VALUES (?1,?2)", params![input.id,knowledge_id]).map_err(err)?;
    }
    transaction.commit().map_err(err)
}

pub fn delete_project(connection: &Connection, id: &str) -> Result<(), String> { connection.execute("DELETE FROM projects WHERE id=?1", [id]).map_err(err)?; Ok(()) }

pub fn list_education(connection: &Connection) -> Result<Vec<EducationItem>, String> {
    let mut statement = connection.prepare("SELECT id,name,type,institution,status,start_date,expected_end_date,completed_at,description,period,domains,created_at,updated_at FROM education ORDER BY CASE status WHEN 'in_progress' THEN 0 WHEN 'planned' THEN 1 ELSE 2 END,name").map_err(err)?;
    let rows = statement.query_map([], |row| { let domains: String = row.get(10)?; Ok(EducationItem { id: row.get(0)?, name: row.get(1)?, kind: row.get(2)?, institution: row.get(3)?, status: row.get(4)?, start_date: row.get(5)?, expected_end_date: row.get(6)?, completed_at: row.get(7)?, description: row.get(8)?, period: row.get(9)?, domains: split_domains(&domains), created_at: row.get(11)?, updated_at: row.get(12)? }) }).map_err(err)?;
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

pub fn delete_education(connection: &Connection, id: &str) -> Result<(), String> { connection.execute("DELETE FROM education WHERE id=?1", [id]).map_err(err)?; Ok(()) }

fn collect_strings(connection: &Connection, sql: &str, id: &str) -> Result<Vec<String>, String> {
    let mut statement = connection.prepare(sql).map_err(err)?;
    let rows = statement.query_map([id], |row| row.get(0)).map_err(err)?;
    rows.collect::<Result<Vec<_>, _>>().map_err(err)
}

fn split_domains(value: &str) -> Vec<String> { value.split('|').filter(|value| !value.is_empty()).map(str::to_owned).collect() }
fn err(error: rusqlite::Error) -> String { error.to_string() }
fn percent(value: i64, field: &str) -> Result<(), String> { if (0..=100).contains(&value) { Ok(()) } else { Err(format!("{field} deve estar entre 0 e 100")) } }
fn valid_id(value: &str) -> bool { !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') }
fn required(value: &str, field: &str) -> Result<(), String> { if value.trim().is_empty() { Err(format!("{field} é obrigatório")) } else { Ok(()) } }
fn validate_knowledge(input: &KnowledgeInput) -> Result<(), String> {
    if !valid_id(&input.id) { return Err("ID inválido".into()); }
    required(&input.name,"Nome")?; required(&input.category,"Categoria")?; percent(input.coverage,"Cobertura")?; percent(input.depth,"Profundidade")?;
    if !["critical","high","medium","low"].contains(&input.priority.as_str()) { return Err("Prioridade inválida".into()); }
    Ok(())
}
fn validate_project(input: &ProjectInput) -> Result<(), String> {
    if !valid_id(&input.id) { return Err("ID inválido".into()); }
    required(&input.name,"Nome")?; required(&input.category,"Categoria")?; percent(input.progress,"Progresso")?;
    if !["active","research","paused","planned","completed"].contains(&input.status.as_str()) { return Err("Status inválido".into()); }
    Ok(())
}
fn validate_education(input: &EducationInput) -> Result<(), String> {
    if !valid_id(&input.id) { return Err("ID inválido".into()); }
    required(&input.name,"Nome")?;
    if !["graduation","postgraduate","masters","doctorate","course","certification"].contains(&input.kind.as_str()) { return Err("Tipo inválido".into()); }
    if !["completed","in_progress","planned"].contains(&input.status.as_str()) { return Err("Status inválido".into()); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn database() -> Connection { let mut connection = Connection::open_in_memory().unwrap(); database::initialize(&mut connection).unwrap(); connection }

    #[test]
    fn seed_is_idempotent() {
        let mut connection = database();
        seed(&mut connection).unwrap();
        assert_eq!(database::schema_version(&connection).unwrap(), 6);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM projects", [], |row| row.get::<_, i64>(0)).unwrap(), 7);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM education", [], |row| row.get::<_, i64>(0)).unwrap(), 7);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_history", [], |row| row.get::<_, i64>(0)).unwrap(), KNOWLEDGE_SEED.len() as i64);
        assert_eq!(connection.query_row("SELECT expected_end_date FROM education WHERE id='biotech'", [], |row| row.get::<_, String>(0)).unwrap(), "2026-11-30");
        assert_eq!(connection.query_row("SELECT type FROM education WHERE id='masters'", [], |row| row.get::<_, String>(0)).unwrap(), "masters");
        delete_project(&connection, "atlas3d").unwrap();
        seed(&mut connection).unwrap();
        assert!(get_project(&connection, "atlas3d").unwrap().is_none());
    }

    #[test]
    fn metric_update_writes_current_value_and_history() {
        let mut connection = database();
        let before = list_history(&connection, "ai").unwrap().len();
        let area = update_metrics(&mut connection, &MetricsInput { knowledge_id: "ai".into(), coverage: 67, depth: 44, reason: "Estudo validado".into() }).unwrap();
        assert_eq!((area.coverage, area.depth), (67, 44));
        let history = list_history(&connection, "ai").unwrap();
        assert_eq!(history.len(), before + 1);
        assert_eq!(history[0].reason, "Estudo validado");
    }

    #[test]
    fn project_relations_are_persisted() {
        let mut connection = database();
        let mut project = list_projects(&connection).unwrap().into_iter().find(|item| item.id == "azriel").unwrap();
        assert!(project.knowledge_area_ids.contains(&"ai".to_string()));
        project.knowledge_area_ids = vec!["cyber".into(), "programming".into()];
        save_project(&mut connection, &ProjectInput { id: project.id.clone(), name: project.name, description: project.description, objective: project.objective, category: project.category, status: project.status, progress: project.progress, next_step: project.next_step, knowledge_area_ids: project.knowledge_area_ids }).unwrap();
        let saved = list_projects(&connection).unwrap().into_iter().find(|item| item.id == project.id).unwrap();
        assert_eq!(saved.knowledge_area_ids, vec!["cyber", "programming"]);
    }
}
