pub mod ai_models;
pub mod ai_repository;
pub mod automation_models;
pub mod automation_repository;
pub mod daily_models;
pub mod daily_repository;
pub mod engineering_models;
pub mod engineering_repository;
pub mod learning_engine;
pub mod models;
pub mod repository;
pub mod routine_models;
pub mod routine_repository;
pub mod system_models;
pub mod system_repository;
pub mod stark_models;
pub mod stark_repository;

use rusqlite::{params, Connection};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct DatabaseState {
    pub connection: Mutex<Connection>,
    pub path: PathBuf,
}

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "initial",
        include_str!("../../migrations/0001_initial.sql"),
    ),
    (
        2,
        "education_contract",
        include_str!("../../migrations/0002_education_contract.sql"),
    ),
    (
        3,
        "seed_registry",
        include_str!("../../migrations/0003_seed_registry.sql"),
    ),
    (
        4,
        "daily_operations",
        include_str!("../../migrations/0004_daily_operations.sql"),
    ),
    (
        5,
        "ai_core",
        include_str!("../../migrations/0005_ai_core.sql"),
    ),
    (
        6,
        "system_core",
        include_str!("../../migrations/0006_system_core.sql"),
    ),
    (
        7,
        "automation_core",
        include_str!("../../migrations/0007_automation_core.sql"),
    ),
    (
        8,
        "routines",
        include_str!("../../migrations/0008_routines.sql"),
    ),
    (
        9,
        "engineering_core",
        include_str!("../../migrations/0009_engineering_core.sql"),
    ),
    (
        10,
        "assembly_intelligence",
        include_str!("../../migrations/0010_assembly_intelligence.sql"),
    ),
    (
        11,
        "stark_knowledge_system",
        include_str!("../../migrations/0011_stark_knowledge_system.sql"),
    ),
    (
        12,
        "learning_engine",
        include_str!("../../migrations/0012_learning_engine.sql"),
    ),
];

pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut connection = Connection::open(path).map_err(|error| error.to_string())?;
    initialize(&mut connection)?;
    routine_repository::cancel_stale_waiting(&connection)?;
    Ok(connection)
}

pub fn initialize(connection: &mut Connection) -> Result<(), String> {
    prepare_migration_registry(connection)?;
    apply_migrations_through(connection, i64::MAX)?;
    repository::seed(connection)?;
    stark_repository::ensure_baselines(connection)?;
    stark_repository::ensure_research_seed(connection)
}

fn prepare_migration_registry(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS _azriel_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
         );",
        )
        .map_err(|error| error.to_string())
}

fn apply_migrations_through(
    connection: &mut Connection,
    maximum_version: i64,
) -> Result<(), String> {
    for (version, name, sql) in MIGRATIONS {
        if *version > maximum_version {
            continue;
        }
        let applied = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM _azriel_migrations WHERE version = ?1)",
                [version],
                |row| row.get::<_, bool>(0),
            )
            .map_err(|error| error.to_string())?;
        if !applied {
            let transaction = connection
                .transaction()
                .map_err(|error| error.to_string())?;
            transaction
                .execute_batch(sql)
                .map_err(|error| error.to_string())?;
            transaction
                .execute(
                    "INSERT INTO _azriel_migrations(version, name) VALUES (?1, ?2)",
                    params![version, name],
                )
                .map_err(|error| error.to_string())?;
            transaction.commit().map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

pub fn schema_version(connection: &Connection) -> Result<i64, String> {
    connection
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM _azriel_migrations",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_four_preserves_a_version_five_database() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 3).unwrap();
        repository::seed(&mut connection).unwrap();
        let before = (
            connection
                .query_row("SELECT COUNT(*) FROM projects", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM education", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_history", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
        );

        apply_migrations_through(&mut connection, 4).unwrap();
        let after = (
            connection
                .query_row("SELECT COUNT(*) FROM projects", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM education", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_history", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
        );

        assert_eq!(before, after);
        assert_eq!(schema_version(&connection).unwrap(), 4);
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_five_preserves_version_051_data() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 4).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute("INSERT INTO tasks(id,title,status,priority) VALUES ('keep-task','Preservar','inbox','medium')", []).unwrap();
        connection
            .execute(
                "INSERT INTO notes(id,content,status) VALUES ('keep-note','Preservar','active')",
                [],
            )
            .unwrap();

        apply_migrations_through(&mut connection, 5).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 5);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM tasks WHERE id='keep-task'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM notes WHERE id='keep-note'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT model FROM ai_settings WHERE id=1", [], |row| row
                    .get::<_, String>(
                    0
                ))
                .unwrap(),
            "qwen2.5:0.5b"
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM conversations", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_six_preserves_ai_core_data() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 5).unwrap();
        repository::seed(&mut connection).unwrap();
        connection
            .execute(
                "INSERT INTO conversations(id,title) VALUES ('keep-chat','Preservar')",
                [],
            )
            .unwrap();

        apply_migrations_through(&mut connection, 6).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 6);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM conversations WHERE id='keep-chat'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM workspaces", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_seven_preserves_system_core_data() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 6).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute("INSERT INTO workspaces(id,name,path,enabled) VALUES ('keep-workspace','Preservar','C:\\Projetos',1)", []).unwrap();

        apply_migrations_through(&mut connection, 7).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 7);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM workspaces WHERE id='keep-workspace'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM applications", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM registered_urls", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM action_history", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_eight_preserves_automation_core_data() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 7).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute(
            "INSERT INTO registered_urls(id,name,url,enabled) VALUES ('keep-url','Preservar','https://example.com/',1)",
            [],
        ).unwrap();

        apply_migrations_through(&mut connection, 8).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 8);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM registered_urls WHERE id='keep-url'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM routines", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_nine_preserves_routines() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 8).unwrap();
        repository::seed(&mut connection).unwrap();
        connection
            .execute(
                "INSERT INTO routines(id,name) VALUES ('keep-routine','Preservar')",
                [],
            )
            .unwrap();

        apply_migrations_through(&mut connection, 9).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 9);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM routines WHERE id='keep-routine'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT calibrated FROM engineering_calibration WHERE id=1",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_ten_preserves_engineering_calibration() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 9).unwrap();
        connection
            .execute(
                "UPDATE engineering_calibration SET calibrated=1 WHERE id=1",
                [],
            )
            .unwrap();

        apply_migrations_through(&mut connection, 10).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 10);
        assert_eq!(
            connection
                .query_row(
                    "SELECT calibrated FROM engineering_calibration WHERE id=1",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM engineering_models", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }

    #[test]
    fn migration_eleven_preserves_existing_data_and_creates_one_baseline_per_knowledge() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 10).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute("INSERT INTO tasks(id,title,status,priority) VALUES ('keep-task-v82','Preservar tarefa','inbox','medium')", []).unwrap();
        connection.execute("INSERT INTO notes(id,content,status) VALUES ('keep-note-v82','Preservar nota','active')", []).unwrap();
        connection.execute("INSERT INTO applications(id,name,path,enabled) VALUES ('keep-app-v82','Preservar app','C:\\Azriel.exe',1)", []).unwrap();
        connection.execute("INSERT INTO workspaces(id,name,path,enabled,application_id) VALUES ('keep-workspace-v82','Preservar workspace','C:\\Projetos',1,'keep-app-v82')", []).unwrap();
        connection.execute("INSERT INTO routines(id,name) VALUES ('keep-routine-v82','Preservar rotina')", []).unwrap();
        let old_tables = ["projects", "knowledge_areas", "knowledge_history", "education", "tasks", "notes", "workspaces", "applications", "routines", "engineering_calibration"];
        let before: Vec<i64> = old_tables.iter().map(|table| connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0)).unwrap()).collect();

        apply_migrations_through(&mut connection, 11).unwrap();
        stark_repository::ensure_baselines(&connection).unwrap();
        stark_repository::ensure_research_seed(&mut connection).unwrap();
        let after: Vec<i64> = old_tables.iter().map(|table| connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| row.get(0)).unwrap()).collect();
        assert_eq!(before, after);
        let knowledge_count: i64 = connection.query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get(0)).unwrap();
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_baselines", [], |row| row.get::<_, i64>(0)).unwrap(), knowledge_count);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM research_items", [], |row| row.get::<_, i64>(0)).unwrap(), 6);
        stark_repository::ensure_baselines(&connection).unwrap();
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_baselines", [], |row| row.get::<_, i64>(0)).unwrap(), knowledge_count);
        assert_eq!(schema_version(&connection).unwrap(), 11);
    }

    #[test]
    fn migration_twelve_preserves_stark_data_and_initializes_learning_engine() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 11).unwrap();
        repository::seed(&mut connection).unwrap();
        stark_repository::ensure_baselines(&connection).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-v83','Preservar roadmap','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('keep-stage-v83','keep-v83','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,knowledge_node_id,topic_order) VALUES ('keep-topic-v83','keep-stage-v83','Tópico','electronics',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('keep-activity-v83','keep-topic-v83','Atividade legada','READING','completed',1)", []).unwrap();
        let before: i64 = connection.query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get(0)).unwrap();
        apply_migrations_through(&mut connection, 12).unwrap();
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get::<_,i64>(0)).unwrap(), before);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_roadmaps WHERE id='keep-v83'", [], |row| row.get::<_,i64>(0)).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT knowledge_node_id FROM activity_knowledge_nodes WHERE activity_id='keep-activity-v83' AND role='primary'", [], |row| row.get::<_,String>(0)).unwrap(), "electronics");
        assert_eq!(connection.query_row("SELECT formula_version FROM learning_engine_state WHERE id=1", [], |row| row.get::<_,String>(0)).unwrap(), "LEARNING_ENGINE_V1");
        assert_eq!(schema_version(&connection).unwrap(), 12);
    }
}
