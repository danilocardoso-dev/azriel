pub mod ai_models;
pub mod ai_repository;
pub mod automation_models;
pub mod automation_repository;
pub mod daily_models;
pub mod daily_repository;
pub mod engineering_models;
pub mod engineering_repository;
pub mod learning_engine;
pub mod market_ai;
pub mod market_ai_intraday;
pub mod market_ai_reliability;
pub mod market_hold_diagnostics;
pub mod market_hold_repository;
pub mod market_identity;
pub mod market_intraday;
pub mod market_intraday_observatory;
pub mod market_lifecycle_intelligence;
pub mod market_lifecycle_repository;
pub mod market_lifecycle_validation;
pub mod market_lifecycle_validation_report;
pub mod market_lifecycle_validation_repository;
pub mod market_models;
pub mod market_observatory;
pub mod market_positions;
pub mod market_regimes;
pub mod market_repository;
pub mod market_risk;
pub mod market_signals;
pub mod market_statistics;
#[cfg(test)]
mod market_tests;
pub mod market_time;
pub mod market_triggers;
pub mod market_validation;
pub mod models;
pub mod repository;
pub mod routine_models;
pub mod routine_repository;
pub mod stark_models;
pub mod stark_repository;
pub mod study_models;
pub mod study_material_models;
pub mod study_material_repository;
pub mod study_repository;
pub mod study_review_models;
pub mod study_review_repository;
pub mod study_review_scheduler;
pub mod study_workspace_models;
pub mod study_workspace_repository;
pub mod system_models;
pub mod system_repository;

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
    (
        13,
        "interactive_roadmaps",
        include_str!("../../migrations/0013_interactive_roadmaps.sql"),
    ),
    (
        14,
        "market_lab",
        include_str!("../../migrations/0014_market_lab.sql"),
    ),
    (
        15,
        "market_observatory",
        include_str!("../../migrations/0015_market_observatory.sql"),
    ),
    (
        16,
        "market_validation",
        include_str!("../../migrations/0016_market_validation.sql"),
    ),
    (
        17,
        "market_ai_agent",
        include_str!("../../migrations/0017_market_ai_agent.sql"),
    ),
    (
        18,
        "market_ai_calibration",
        include_str!("../../migrations/0018_market_ai_calibration.sql"),
    ),
    (
        19,
        "market_signal_engine",
        include_str!("../../migrations/0019_market_signal_engine.sql"),
    ),
    (
        20,
        "market_position_lifecycle",
        include_str!("../../migrations/0020_market_position_lifecycle.sql"),
    ),
    (
        21,
        "market_position_execution",
        include_str!("../../migrations/0021_market_position_execution.sql"),
    ),
    (
        22,
        "market_ai_reliability",
        include_str!("../../migrations/0022_market_ai_reliability.sql"),
    ),
    (
        23,
        "market_ai_attempt_error_value",
        include_str!("../../migrations/0023_market_ai_attempt_error_value.sql"),
    ),
    (
        24,
        "market_ai_contract_v42",
        include_str!("../../migrations/0024_market_ai_contract_v42.sql"),
    ),
    (
        25,
        "market_risk_reduction_policy",
        include_str!("../../migrations/0025_market_risk_reduction_policy.sql"),
    ),
    (
        26,
        "market_intraday_foundation",
        include_str!("../../migrations/0026_market_intraday_foundation.sql"),
    ),
    (
        27,
        "market_intraday_strategy_lab",
        include_str!("../../migrations/0027_market_intraday_strategy_lab.sql"),
    ),
    (
        28,
        "market_ai_intraday",
        include_str!("../../migrations/0028_market_ai_intraday.sql"),
    ),
    (
        29,
        "market_hold_diagnostics",
        include_str!("../../migrations/0029_market_hold_diagnostics.sql"),
    ),
    (
        30,
        "market_dataset_identity",
        include_str!("../../migrations/0030_market_dataset_identity.sql"),
    ),
    (
        31,
        "market_position_lifecycle_intelligence",
        include_str!("../../migrations/0031_market_position_lifecycle_intelligence.sql"),
    ),
    (
        32,
        "market_multi_period_lifecycle_validation",
        include_str!("../../migrations/0032_market_multi_period_lifecycle_validation.sql"),
    ),
    (
        33,
        "market_validation_runbook",
        include_str!("../../migrations/0033_market_validation_runbook.sql"),
    ),
    (
        34,
        "study_lab_foundation",
        include_str!("../../migrations/0034_study_lab_foundation.sql"),
    ),
    (
        35,
        "study_knowledge_workspace",
        include_str!("../../migrations/0035_study_knowledge_workspace.sql"),
    ),
    (
        36,
        "study_review_active_recall",
        include_str!("../../migrations/0036_study_review_active_recall.sql"),
    ),
    (
        37,
        "study_library",
        include_str!("../../migrations/0037_study_library.sql"),
    ),
    (
        38,
        "ai_response_timeout",
        include_str!("../../migrations/0038_ai_response_timeout.sql"),
    ),
    (
        39,
        "ai_task_references",
        include_str!("../../migrations/0039_ai_task_references.sql"),
    ),
    (
        40,
        "study_roadmap_learning_model",
        include_str!("../../migrations/0040_study_roadmap_learning_model.sql"),
    ),
    (
        41,
        "knowledge_catalog_reset",
        include_str!("../../migrations/0041_knowledge_catalog_reset.sql"),
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
    market_identity::repair_legacy_asset_identities(connection)?;
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
        connection
            .execute(
                "INSERT INTO routines(id,name) VALUES ('keep-routine-v82','Preservar rotina')",
                [],
            )
            .unwrap();
        let old_tables = [
            "projects",
            "knowledge_areas",
            "knowledge_history",
            "education",
            "tasks",
            "notes",
            "workspaces",
            "applications",
            "routines",
            "engineering_calibration",
        ];
        let before: Vec<i64> = old_tables
            .iter()
            .map(|table| {
                connection
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap()
            })
            .collect();

        apply_migrations_through(&mut connection, 11).unwrap();
        stark_repository::ensure_baselines(&connection).unwrap();
        stark_repository::ensure_research_seed(&mut connection).unwrap();
        let after: Vec<i64> = old_tables
            .iter()
            .map(|table| {
                connection
                    .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                        row.get(0)
                    })
                    .unwrap()
            })
            .collect();
        assert_eq!(before, after);
        let knowledge_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_baselines", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            knowledge_count
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM research_items", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        stark_repository::ensure_baselines(&connection).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_baselines", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            knowledge_count
        );
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
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,knowledge_node_id,topic_order) VALUES ('keep-topic-v83','keep-stage-v83','Tópico','software-engineering',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('keep-activity-v83','keep-topic-v83','Atividade legada','READING','completed',1)", []).unwrap();
        let before: i64 = connection
            .query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get(0))
            .unwrap();
        apply_migrations_through(&mut connection, 12).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            before
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM study_roadmaps WHERE id='keep-v83'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(connection.query_row("SELECT knowledge_node_id FROM activity_knowledge_nodes WHERE activity_id='keep-activity-v83' AND role='primary'", [], |row| row.get::<_,String>(0)).unwrap(), "software-engineering");
        assert_eq!(
            connection
                .query_row(
                    "SELECT formula_version FROM learning_engine_state WHERE id=1",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "LEARNING_ENGINE_V1"
        );
        assert_eq!(schema_version(&connection).unwrap(), 12);
    }

    #[test]
    fn migration_thirteen_preserves_roadmaps_and_promotes_legacy_prerequisite() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 12).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-v84','Roadmap v0.8.4','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('stage-v84','keep-v84','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,description,knowledge_node_id,topic_order) VALUES ('topic-a-v84','stage-v84','Base','','software-engineering',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,description,knowledge_node_id,topic_order) VALUES ('topic-b-v84','stage-v84','Avançado','Conteúdo. Pré-requisitos: topic-a-v84','software-engineering',2)", []).unwrap();

        apply_migrations_through(&mut connection, 13).unwrap();

        assert_eq!(connection.query_row("SELECT prerequisite_topic_id FROM roadmap_topic_prerequisites WHERE topic_id='topic-b-v84'", [], |row| row.get::<_,String>(0)).unwrap(), "topic-a-v84");
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM study_roadmaps WHERE id='keep-v84'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(schema_version(&connection).unwrap(), 13);
    }

    #[test]
    fn migration_fourteen_preserves_roadmaps_and_seeds_market_registry() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 13).unwrap();
        repository::seed(&mut connection).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-market','Preservar roadmap','active')", []).unwrap();

        apply_migrations_through(&mut connection, 14).unwrap();

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM study_roadmaps WHERE id='keep-market'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_agents", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            5
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_risk_profiles", [], |row| row
                    .get::<_, i64>(
                    0
                ))
                .unwrap(),
            1
        );
        assert_eq!(schema_version(&connection).unwrap(), 14);
    }

    #[test]
    fn migration_fifteen_preserves_market_lab_and_adds_observatory() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 14).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES ('keep-dataset','Preservar','TST','1D','2026-01-01','2026-01-02',2,'keep-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 15).unwrap();

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-dataset'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_behavior_metrics", [], |row| {
                    row.get::<_, i64>(0)
                })
                .unwrap(),
            0
        );
        assert_eq!(schema_version(&connection).unwrap(), 15);
    }

    #[test]
    fn migration_sixteen_preserves_observatory_and_adds_validation() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 15).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES ('keep-validation','Preservar','TST','1D','2026-01-01','2026-01-02',2,'validation-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 16).unwrap();

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-validation'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row("SELECT COUNT(*) FROM market_validation_runs", [], |row| row
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(schema_version(&connection).unwrap(), 16);
    }

    #[test]
    fn migration_seventeen_preserves_validation_and_adds_single_ai_agent() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 16).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES ('keep-ai','Preservar','TST','1D','2026-01-01','2026-01-02',2,'ai-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 17).unwrap();

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-ai'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_agents WHERE strategy_type='llm'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-technical-v1'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_AGENT_V1");
        assert_eq!(schema_version(&connection).unwrap(), 17);
    }

    #[test]
    fn migration_eighteen_preserves_v1_and_adds_versioned_ai_diagnostics() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 17).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES ('keep-ai-v1','Preservar','TST','1D','2026-01-01','2026-01-02',2,'ai-v1-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 18).unwrap();

        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-ai-v1'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_agents WHERE strategy_type='llm'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-technical-v1'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_AGENT_V1");
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-technical-v2'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_AGENT_V2");
        assert_eq!(schema_version(&connection).unwrap(), 18);
    }

    #[test]
    fn migration_nineteen_preserves_v1_v2_and_adds_v3_signal_engine() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 18).unwrap();
        connection.execute("UPDATE market_ai_agent_configs SET decision_interval=7 WHERE agent_id='ai-technical-v2'", []).unwrap();
        apply_migrations_through(&mut connection, 19).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 19);
        assert_eq!(connection.query_row("SELECT decision_interval FROM market_ai_agent_configs WHERE agent_id='ai-technical-v2'", [], |row| row.get::<_,i64>(0)).unwrap(), 7);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_agents WHERE id='ai-technical-v3'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(connection.query_row("SELECT engine_version FROM market_signal_engine_configs WHERE version='SIGNAL_CONFIG_V1'", [], |row| row.get::<_,String>(0)).unwrap(), "SIGNAL_ENGINE_V1");
    }

    #[test]
    fn migration_twenty_preserves_v3_and_adds_position_lifecycle_v4() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 19).unwrap();
        connection.execute("UPDATE market_ai_agent_configs SET decision_interval=7 WHERE agent_id='ai-technical-v3'",[]).unwrap();
        apply_migrations_through(&mut connection, 20).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 20);
        assert_eq!(connection.query_row("SELECT decision_interval FROM market_ai_agent_configs WHERE agent_id='ai-technical-v3'",[],|row|row.get::<_,usize>(0)).unwrap(),7);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_agents WHERE id='ai-technical-v4'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
        for table in [
            "market_trade_lifecycles",
            "market_position_events",
            "market_position_metrics",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_twenty_one_adds_execution_link_without_losing_position_events() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 20).unwrap();
        let execution_column_before = connection
            .prepare("PRAGMA table_info(market_position_events)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .any(|column| column == "execution_id");
        assert!(!execution_column_before);
        connection.execute_batch("PRAGMA foreign_keys=OFF;
            INSERT INTO market_trade_lifecycles(id,experiment_id,agent_id,asset,lifecycle_index,opened_at,entry_price,average_entry_price,initial_exposure_pct,max_exposure_pct,holding_candles,realized_pnl,realized_pnl_pct,mfe_pct,mae_pct,profit_giveback_pct,status,reentry,engine_version)
            VALUES('migration-event','legacy-experiment','ai-technical-v4','TST',1,'2026-01-01',100,100,30,30,1,0,0,1,-1,0,'OPEN',0,'POSITION_ENGINE_V1');
            INSERT INTO market_position_events(lifecycle_id,timestamp,action,previous_exposure_pct,target_exposure_pct,new_exposure_pct,risk_result)
            VALUES('migration-event','2026-01-01','ENTER_LONG',0,30,30,'APPROVED');
            PRAGMA foreign_keys=ON;").unwrap();

        apply_migrations_through(&mut connection, 21).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 21);
        let columns = connection
            .prepare("PRAGMA table_info(market_position_events)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(columns.iter().any(|column| column == "execution_id"));
        assert_eq!(connection.query_row("SELECT action FROM market_position_events WHERE lifecycle_id='migration-event'", [], |row| row.get::<_, String>(0)).unwrap(), "ENTER_LONG");
    }

    #[test]
    fn migration_twenty_two_preserves_v4_and_adds_v4_1_reliability_audit() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 21).unwrap();
        connection.execute(
            "UPDATE market_ai_agent_configs SET decision_interval=7 WHERE agent_id='ai-technical-v4'",
            [],
        ).unwrap();

        apply_migrations_through(&mut connection, 22).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 22);
        assert_eq!(connection.query_row("SELECT decision_interval FROM market_ai_agent_configs WHERE agent_id='ai-technical-v4'", [], |row| row.get::<_, usize>(0)).unwrap(), 7);
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-technical-v4-1'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_AGENT_V4_1");
        for table in ["market_ai_output_attempts", "market_ai_validation_stages"] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
        let decision_columns = connection
            .prepare("PRAGMA table_info(market_ai_decisions)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(decision_columns
            .iter()
            .any(|column| column == "first_failure_type"));
        assert!(decision_columns
            .iter()
            .any(|column| column == "fallback_reason"));
        let runtime_columns = connection
            .prepare("PRAGMA table_info(market_ai_runtime_metrics)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(runtime_columns
            .iter()
            .any(|column| column == "first_pass_valid_count"));
        assert!(runtime_columns
            .iter()
            .any(|column| column == "p95_latency_ms"));
    }

    #[test]
    fn migration_twenty_three_adds_error_value_without_losing_attempts() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 22).unwrap();
        connection
            .execute_batch(
                "PRAGMA foreign_keys=OFF;
                 INSERT INTO market_ai_output_attempts(
                   decision_id, attempt_number, raw_response, status, error_type,
                   error_field, error_message, normalized_from_wrapped_json, latency_ms
                 ) VALUES(999, 1, '{\"action\":\"INVALID\"}', 'INVALID',
                   'INVALID_ENUM', 'action', 'valor inválido', 0, 12);
                 PRAGMA foreign_keys=ON;",
            )
            .unwrap();

        let columns_before = connection
            .prepare("PRAGMA table_info(market_ai_output_attempts)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(!columns_before.iter().any(|column| column == "error_value"));

        apply_migrations_through(&mut connection, 23).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 23);
        let columns_after = connection
            .prepare("PRAGMA table_info(market_ai_output_attempts)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert!(columns_after.iter().any(|column| column == "error_value"));
        assert_eq!(
            connection
                .query_row(
                    "SELECT status FROM market_ai_output_attempts WHERE decision_id=999",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .unwrap(),
            "INVALID"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT error_value FROM market_ai_output_attempts WHERE decision_id=999",
                    [],
                    |row| row.get::<_, Option<String>>(0),
                )
                .unwrap(),
            None
        );
    }

    #[test]
    fn migration_twenty_four_preserves_v4_1_and_adds_v4_2_contract() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 23).unwrap();
        connection.execute(
            "UPDATE market_ai_agent_configs SET decision_interval=7 WHERE agent_id='ai-technical-v4-1'",
            [],
        ).unwrap();

        apply_migrations_through(&mut connection, 24).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 24);
        assert_eq!(connection.query_row("SELECT decision_interval FROM market_ai_agent_configs WHERE agent_id='ai-technical-v4-1'", [], |row| row.get::<_, usize>(0)).unwrap(), 7);
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-technical-v4-2'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_AGENT_V4_2");
        let sizing: (f64, f64, f64) = connection.query_row(
            "SELECT default_entry_exposure_pct,default_increase_step_pct,default_reduce_step_pct FROM market_position_sizing_configs WHERE version='POSITION_SIZING_V1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap();
        assert_eq!(sizing, (25.0, 10.0, 15.0));
        let columns = connection
            .prepare("PRAGMA table_info(market_ai_decisions)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for expected in [
            "intent",
            "generated_target_exposure_pct",
            "position_sizing_version",
        ] {
            assert!(columns.iter().any(|column| column == expected));
        }
    }

    #[test]
    fn migration_twenty_five_adds_risk_trace_without_losing_history() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 24).unwrap();
        connection
            .execute_batch("PRAGMA foreign_keys = OFF;")
            .unwrap();
        connection.execute(
            "INSERT INTO market_risk_evaluations(decision_id,result,reason,risk_state_json) VALUES(999,'APPROVED','legacy','{}')",
            [],
        ).unwrap();
        apply_migrations_through(&mut connection, 25).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 25);
        assert_eq!(
            connection
                .query_row(
                    "SELECT reason FROM market_risk_evaluations WHERE decision_id=999",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "legacy"
        );
        let columns = connection
            .prepare("PRAGMA table_info(market_risk_evaluations)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        for expected in [
            "exposure_change_class",
            "current_exposure_pct",
            "target_exposure_pct",
            "exposure_delta_pct",
            "policy_version",
        ] {
            assert!(columns.iter().any(|column| column == expected));
        }
        connection.execute(
            "INSERT INTO market_risk_rule_evaluations(decision_id,rule_order,rule,status,reason) VALUES(999,0,'MAX_OPERATIONS','NOT_APPLICABLE','RISK_REDUCING_ACTION')",
            [],
        ).unwrap();
        assert_eq!(
            connection
                .query_row(
                    "SELECT status FROM market_risk_rule_evaluations WHERE decision_id=999",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "NOT_APPLICABLE"
        );
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .unwrap();
    }

    #[test]
    fn migration_twenty_six_preserves_market_history_and_adds_intraday_audit() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 25).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-intraday','Preservar','AAPL','1D','2026-01-01','2026-01-02',2,'keep-intraday-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 26).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 26);
        let metadata: (String, String, String) = connection
            .query_row(
                "SELECT market,timezone,session_type FROM market_datasets WHERE id='keep-intraday'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(
            metadata,
            ("UNSPECIFIED".into(), "UTC".into(), "DAILY".into())
        );
        for table in ["market_decision_trigger_audits", "market_session_metrics"] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_twenty_seven_adds_versioned_intraday_strategies_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 26).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-v051','Preservar v0.5','AAPL','15M','2026-01-01','2026-01-02',2,'keep-v051-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 27).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 27);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-v051'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_agents WHERE id LIKE 'intraday-%-v1'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            3
        );
        for table in [
            "market_intraday_feature_traces",
            "market_intraday_strategy_decisions",
            "market_intraday_strategy_metrics",
            "market_session_phase_performance",
            "market_holding_time_distribution",
            "market_intraday_agent_overlap",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_twenty_eight_adds_intraday_ai_and_repeatability_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 27).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-v052','Preservar v0.5.1','AAPL','15M','2026-01-01','2026-01-02',2,'keep-v052-fingerprint','fixture.csv')", []).unwrap();
        apply_migrations_through(&mut connection, 28).unwrap();
        assert_eq!(schema_version(&connection).unwrap(), 28);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-v052'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-intraday-v1'",[],|row|row.get::<_,String>(0)).unwrap(),"MARKET_AI_INTRADAY_V1");
        for table in [
            "market_ai_execution_metadata",
            "market_repeatability_groups",
            "market_repeatability_runs",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_twenty_nine_adds_hold_diagnostics_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 28).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-v0521','Preservar v0.5.2','AAPL','15M','2026-01-01','2026-01-02',2,'keep-v0521-fingerprint','fixture.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 29).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 29);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM market_datasets WHERE id='keep-v0521'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
        for table in ["market_hold_diagnostic_runs", "market_hold_diagnostics"] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
        assert_eq!(connection.query_row("SELECT prompt_version FROM market_ai_agent_configs WHERE agent_id='ai-intraday-v1'", [], |row| row.get::<_, String>(0)).unwrap(), "MARKET_AI_INTRADAY_V1");
    }

    #[test]
    fn migration_thirty_adds_identity_repair_audit_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 29).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-identity','AAPL_real_15m','AAPL - REAL','15M','2026-01-01','2026-01-02',2,'keep-identity-fingerprint','AAPL_real_15m.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 30).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 30);
        assert_eq!(
            connection
                .query_row(
                    "SELECT fingerprint FROM market_datasets WHERE id='keep-identity'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "keep-identity-fingerprint"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='market_dataset_identity_repairs'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
    }

    #[test]
    fn migration_thirty_one_adds_lifecycle_intelligence_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 30).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-lifecycle','AAPL 15M','AAPL','15M','2026-01-01','2026-01-02',2,'keep-lifecycle-fingerprint','AAPL_15m.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 31).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 31);
        assert_eq!(
            connection
                .query_row(
                    "SELECT fingerprint FROM market_datasets WHERE id='keep-lifecycle'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "keep-lifecycle-fingerprint"
        );
        for table in [
            "market_lifecycle_analysis_runs",
            "market_lifecycle_analyses",
            "market_lifecycle_events",
            "market_lifecycle_health_trace",
            "market_lifecycle_post_decision_outcomes",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_thirty_two_adds_multi_period_validation_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 31).unwrap();
        connection.execute("INSERT INTO market_datasets(id,name,asset,timeframe,start_at,end_at,candle_count,fingerprint,source_path) VALUES('keep-multi-period','AAPL 15M','AAPL','15M','2026-01-01','2026-01-02',2,'keep-multi-period-fingerprint','AAPL_15m.csv')", []).unwrap();

        apply_migrations_through(&mut connection, 32).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 32);
        assert_eq!(
            connection
                .query_row(
                    "SELECT fingerprint FROM market_datasets WHERE id='keep-multi-period'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "keep-multi-period-fingerprint"
        );
        for table in [
            "market_lifecycle_validation_batches",
            "market_lifecycle_validation_periods",
            "market_lifecycle_validation_lifecycles",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, usize>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_thirty_three_adds_runbook_checkpoints_without_data_loss() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 32).unwrap();
        connection.execute(
            "INSERT INTO market_lifecycle_validation_batches(id,name,asset,timeframe,agent_id,agent_version,lifecycle_config_version,deterioration_config_version,hold_diagnostics_version,execution_model_version,status,config_json) VALUES('keep-runbook','Keep','AAPL','15M','ai-intraday-v1','AI_INTRADAY_V1','POSITION_LIFECYCLE_CONFIG_V1','POSITION_DETERIORATION_CONFIG_V1','HOLD_DIAGNOSTICS_V1','EXECUTION_MODEL_V1','CREATED','{}')",
            [],
        ).unwrap();

        apply_migrations_through(&mut connection, 33).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 33);
        assert_eq!(
            connection
                .query_row(
                    "SELECT name FROM market_lifecycle_validation_batches WHERE id='keep-runbook'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "Keep"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='market_lifecycle_validation_artifacts'",
                    [],
                    |row| row.get::<_, usize>(0)
                )
                .unwrap(),
            1
        );
    }

    #[test]
    fn migration_thirty_four_preserves_roadmaps_and_adds_study_sessions() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 33).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-study-lab','Roadmap preservado','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('keep-study-stage','keep-study-lab','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,topic_order) VALUES ('keep-study-topic','keep-study-stage','Tópico',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('keep-study-activity','keep-study-topic','Atividade','EXERCISE','pending',1)", []).unwrap();

        apply_migrations_through(&mut connection, 34).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 34);
        assert_eq!(
            connection
                .query_row(
                    "SELECT name FROM study_roadmaps WHERE id='keep-study-lab'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "Roadmap preservado"
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT focus_minutes FROM study_settings WHERE id=1",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            25
        );
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='study_sessions'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn migration_thirty_five_preserves_study_sessions_and_adds_knowledge_workspace() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 34).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-workspace-roadmap','Roadmap v0.1','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('keep-workspace-stage','keep-workspace-roadmap','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,topic_order) VALUES ('keep-workspace-topic','keep-workspace-stage','Tópico',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('keep-workspace-activity','keep-workspace-topic','Atividade','EXERCISE','pending',1)", []).unwrap();
        connection.execute("INSERT INTO study_sessions(id,roadmap_id,stage_id,topic_id,activity_id,roadmap_name,stage_name,topic_name,activity_title,planned_focus_minutes,status,started_at,running_since,open_slot,created_at,updated_at) VALUES ('keep-workspace-session','keep-workspace-roadmap','keep-workspace-stage','keep-workspace-topic','keep-workspace-activity','Roadmap v0.1','Etapa','Tópico','Atividade',25,'ACTIVE','2026-09-25T10:00:00Z','2026-09-25T10:00:00Z',1,'2026-09-25T10:00:00Z','2026-09-25T10:00:00Z')", []).unwrap();

        apply_migrations_through(&mut connection, 35).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 35);
        assert_eq!(
            connection
                .query_row(
                    "SELECT status FROM study_sessions WHERE id='keep-workspace-session'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "ACTIVE"
        );
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='study_notebooks'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='study_notes'",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            1
        );
    }

    #[test]
    fn migration_thirty_six_preserves_study_workspace_and_adds_active_recall() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 35).unwrap();
        connection.execute("INSERT INTO study_notebooks(id,title,status,created_at,updated_at) VALUES ('keep-review-notebook','Caderno v0.2','ACTIVE','2026-09-25T10:00:00Z','2026-09-25T10:00:00Z')", []).unwrap();
        connection.execute("INSERT INTO study_notes(id,notebook_id,title,content,created_at,updated_at) VALUES ('keep-review-note','keep-review-notebook','Nota v0.2','Conteúdo preservado','2026-09-25T10:00:00Z','2026-09-25T10:00:00Z')", []).unwrap();

        apply_migrations_through(&mut connection, 36).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 36);
        assert_eq!(
            connection
                .query_row(
                    "SELECT content FROM study_notes WHERE id='keep-review-note'",
                    [],
                    |row| row.get::<_, String>(0)
                )
                .unwrap(),
            "Conteúdo preservado"
        );
        for table in [
            "study_cards",
            "study_review_states",
            "study_review_sessions",
            "study_review_session_items",
            "study_review_events",
        ] {
            assert_eq!(
                connection
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [table],
                        |row| row.get::<_, i64>(0)
                    )
                    .unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_thirty_seven_preserves_review_data_and_adds_study_library() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 36).unwrap();
        connection.execute("INSERT INTO study_cards(id,front,back,status,created_at,updated_at) VALUES ('keep-library-card','Pergunta','Resposta','ACTIVE','2026-09-25T10:00:00Z','2026-09-25T10:00:00Z')", []).unwrap();

        apply_migrations_through(&mut connection, 37).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 37);
        assert_eq!(connection.query_row("SELECT front FROM study_cards WHERE id='keep-library-card'", [], |row| row.get::<_, String>(0)).unwrap(), "Pergunta");
        for table in ["study_materials", "study_material_relations", "material_text_content"] {
            assert_eq!(connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1", [table], |row| row.get::<_, i64>(0)).unwrap(), 1);
        }
    }

    #[test]
    fn migration_thirty_eight_updates_only_the_previous_default_ai_timeout() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 37).unwrap();
        connection
            .execute(
                "UPDATE ai_settings SET endpoint='http://azriel-ai.ts.net:11434',model='qwen3.5:4b',timeout_seconds=45 WHERE id=1",
                [],
            )
            .unwrap();

        apply_migrations_through(&mut connection, 38).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 38);
        let migrated = connection
            .query_row(
                "SELECT endpoint,model,timeout_seconds FROM ai_settings WHERE id=1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, i64>(2)?,
                    ))
                },
            )
            .unwrap();
        assert_eq!(
            migrated,
            (
                "http://azriel-ai.ts.net:11434".into(),
                "qwen3.5:4b".into(),
                90
            )
        );

        let mut customized = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&customized).unwrap();
        apply_migrations_through(&mut customized, 37).unwrap();
        customized
            .execute(
                "UPDATE ai_settings SET timeout_seconds=120 WHERE id=1",
                [],
            )
            .unwrap();
        apply_migrations_through(&mut customized, 38).unwrap();
        assert_eq!(
            customized
                .query_row(
                    "SELECT timeout_seconds FROM ai_settings WHERE id=1",
                    [],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            120
        );
    }

    #[test]
    fn migration_thirty_nine_adds_ai_references_and_task_audit() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 38).unwrap();
        apply_migrations_through(&mut connection, 39).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 39);
        for table in ["ai_entity_references", "task_action_history"] {
            assert_eq!(
                connection.query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |row| row.get::<_, i64>(0),
                ).unwrap(),
                1
            );
        }
    }

    #[test]
    fn migration_forty_preserves_legacy_roadmaps_and_adds_learning_resources() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 39).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,description,status) VALUES ('legacy-roadmap','Legado','','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,description,stage_order) VALUES ('legacy-stage','legacy-roadmap','Etapa','',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,description,knowledge_node_id,topic_state,topic_order) VALUES ('legacy-topic','legacy-stage','Tópico','',NULL,'NOT_STARTED',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,description,activity_type,status,activity_order) VALUES ('legacy-activity','legacy-topic','Atividade','','READING','pending',1)", []).unwrap();

        apply_migrations_through(&mut connection, 40).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 40);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM roadmap_activities WHERE id='legacy-activity' AND is_validation=0 AND estimated_minutes IS NULL", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='roadmap_activity_resources'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    }

    #[test]
    fn migration_forty_one_replaces_knowledge_catalog_and_preserves_azriel() {
        let mut connection = Connection::open_in_memory().unwrap();
        prepare_migration_registry(&connection).unwrap();
        apply_migrations_through(&mut connection, 40).unwrap();
        connection.execute("INSERT INTO projects(id,name,category,status,progress) VALUES ('azriel','Azriel','Sistema pessoal','active',40)", []).unwrap();
        connection.execute("INSERT INTO knowledge_areas(id,name,category,coverage,depth,priority,node_type) VALUES ('programming','Programação','Computação',80,65,'high','area')", []).unwrap();
        connection.execute("INSERT INTO knowledge_baselines(knowledge_id,coverage,depth) VALUES ('programming',80,65)", []).unwrap();
        connection.execute("INSERT INTO knowledge_history(knowledge_id,coverage,depth,reason) VALUES ('programming',80,65,'Legado')", []).unwrap();
        connection.execute("INSERT INTO project_knowledge(project_id,knowledge_id) VALUES ('azriel','programming')", []).unwrap();
        connection.execute("INSERT INTO study_roadmaps(id,name,status) VALUES ('keep-roadmap','Roadmap preservado','active')", []).unwrap();
        connection.execute("INSERT INTO roadmap_stages(id,roadmap_id,name,stage_order) VALUES ('keep-stage','keep-roadmap','Etapa',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_topics(id,stage_id,name,knowledge_node_id,topic_state,topic_order) VALUES ('keep-topic','keep-stage','Tópico','programming','NOT_STARTED',1)", []).unwrap();
        connection.execute("INSERT INTO roadmap_activities(id,topic_id,title,activity_type,status,activity_order) VALUES ('keep-activity','keep-topic','Atividade','READING','pending',1)", []).unwrap();
        connection.execute("INSERT INTO activity_knowledge_nodes(activity_id,knowledge_node_id,role) VALUES ('keep-activity','programming','primary')", []).unwrap();
        connection.execute("INSERT INTO knowledge_events(id,knowledge_node_id,source_type,event_type,description) VALUES ('legacy-event','programming','manual','manual_adjustment','Legado')", []).unwrap();

        apply_migrations_through(&mut connection, 41).unwrap();

        assert_eq!(schema_version(&connection).unwrap(), 41);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_areas", [], |row| row.get::<_, i64>(0)).unwrap(), 10);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_areas WHERE id IN ('software-engineering','systems-infrastructure','cybersecurity','dfir','reliability-observability','ai-engineering','automation','english','technology-business','engineering-leadership')", [], |row| row.get::<_, i64>(0)).unwrap(), 10);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_areas WHERE id='programming'", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM projects WHERE id='azriel'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM project_knowledge WHERE project_id='azriel'", [], |row| row.get::<_, i64>(0)).unwrap(), 3);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM study_roadmaps WHERE id='keep-roadmap'", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM roadmap_topics WHERE id='keep-topic' AND knowledge_node_id IS NULL", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM activity_knowledge_nodes", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
        assert_eq!(connection.query_row("SELECT COUNT(*) FROM knowledge_events", [], |row| row.get::<_, i64>(0)).unwrap(), 0);
    }
}
