use crate::{
    automation_commands::execute_with_routine_context,
    automation_executor::{ActionExecutor, NativeActionExecutor},
    automation_policy::{
        self, ActionPermission, ActionRequest, ActionSource, PolicyEngine, OPEN_APPLICATION,
        OPEN_WORKSPACE,
    },
    database::{
        routine_models::{
            Routine, RoutineActionSummary, RoutineConfirmation, RoutineExecutionResult,
            RoutineHistory, RoutineInput, RoutineStep, RunRoutineRequest,
        },
        routine_repository, system_repository, DatabaseState,
    },
};
use rusqlite::Connection;
use tauri::State;

trait DelayProvider {
    fn wait(&self, milliseconds: u64);
}

struct ThreadDelay;
impl DelayProvider for ThreadDelay {
    fn wait(&self, milliseconds: u64) {
        std::thread::sleep(std::time::Duration::from_millis(milliseconds));
    }
}

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

fn execution_connection(path: &std::path::Path) -> Result<Connection, String> {
    let connection = Connection::open(path).map_err(|error| error.to_string())?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")
        .map_err(|error| error.to_string())?;
    Ok(connection)
}

#[tauri::command]
pub fn list_routines(state: State<'_, DatabaseState>) -> Result<Vec<Routine>, String> {
    let connection = lock(&state)?;
    routine_repository::list_routines(&connection)
}

#[tauri::command]
pub fn save_routine(
    state: State<'_, DatabaseState>,
    input: RoutineInput,
) -> Result<Vec<Routine>, String> {
    let mut connection = lock(&state)?;
    routine_repository::save_routine(&mut connection, &input)?;
    routine_repository::list_routines(&connection)
}

#[tauri::command]
pub fn delete_routine(state: State<'_, DatabaseState>, id: String) -> Result<Vec<Routine>, String> {
    let connection = lock(&state)?;
    routine_repository::delete_routine(&connection, &id)?;
    routine_repository::list_routines(&connection)
}

#[tauri::command]
pub fn list_routine_history(
    state: State<'_, DatabaseState>,
    limit: Option<i64>,
) -> Result<Vec<RoutineHistory>, String> {
    let connection = lock(&state)?;
    routine_repository::list_history(&connection, limit.unwrap_or(100))
}

#[tauri::command]
pub async fn run_routine(
    state: State<'_, DatabaseState>,
    request: RunRoutineRequest,
) -> Result<RoutineExecutionResult, String> {
    let database_path = state.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let connection = execution_connection(&database_path)?;
        run_with(&connection, request, &NativeActionExecutor, &ThreadDelay)
    })
    .await
    .map_err(|error| format!("Falha na tarefa de execução da rotina: {error}"))?
}

#[tauri::command]
pub async fn confirm_routine_execution(
    state: State<'_, DatabaseState>,
    history_id: i64,
) -> Result<RoutineExecutionResult, String> {
    let database_path = state.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let connection = execution_connection(&database_path)?;
        confirm_with(&connection, history_id, &NativeActionExecutor, &ThreadDelay)
    })
    .await
    .map_err(|error| format!("Falha na tarefa de confirmação da rotina: {error}"))?
}

#[tauri::command]
pub fn cancel_routine_execution(
    state: State<'_, DatabaseState>,
    history_id: i64,
) -> Result<(), String> {
    let connection = lock(&state)?;
    if routine_repository::cancel_waiting(&connection, history_id)? {
        Ok(())
    } else {
        Err("A rotina não está aguardando confirmação".into())
    }
}

fn run_with(
    connection: &Connection,
    request: RunRoutineRequest,
    executor: &dyn ActionExecutor,
    delay: &dyn DelayProvider,
) -> Result<RoutineExecutionResult, String> {
    crate::database::automation_repository::validate_id(&request.routine_id, "rotina")?;
    let routine = routine_repository::get_routine(connection, &request.routine_id)?
        .ok_or_else(|| "Rotina não encontrada".to_string())?;
    let summaries = validate_runtime(connection, &routine, &request.source)?;
    let confirmation_required = routine.confirmation_required
        || summaries.iter().any(|summary| {
            automation_policy::action(&summary.action_id)
                .is_some_and(|action| action.permission == ActionPermission::ConfirmWrite)
        })
        || matches!(request.source, ActionSource::Ai) && summaries.len() > 1;
    let history_id = routine_repository::start_history(
        connection,
        &routine,
        request.source.as_str(),
        confirmation_required,
    )?;
    if confirmation_required {
        return Ok(waiting_result(&routine, history_id, summaries));
    }
    execute_validated(
        connection,
        &routine,
        request.source,
        history_id,
        executor,
        delay,
    )
}

fn confirm_with(
    connection: &Connection,
    history_id: i64,
    executor: &dyn ActionExecutor,
    delay: &dyn DelayProvider,
) -> Result<RoutineExecutionResult, String> {
    let history = routine_repository::get_history(connection, history_id)?
        .ok_or_else(|| "Solicitação de confirmação não encontrada".to_string())?;
    if history.status != "waiting_confirmation" {
        return Err("Esta solicitação não está aguardando confirmação".into());
    }
    let Some(routine_id) = history.routine_id.as_deref() else {
        routine_repository::finish_history(
            connection,
            history_id,
            false,
            0,
            None,
            Some("A rotina foi excluída"),
        )?;
        return Err("A rotina foi excluída".into());
    };
    let Some(routine) = routine_repository::get_routine(connection, routine_id)? else {
        routine_repository::finish_history(
            connection,
            history_id,
            false,
            0,
            None,
            Some("A rotina foi excluída"),
        )?;
        return Err("A rotina foi excluída".into());
    };
    if routine.revision != history.routine_revision {
        routine_repository::finish_history(
            connection,
            history_id,
            false,
            0,
            None,
            Some("A rotina foi alterada após a solicitação de confirmação"),
        )?;
        return Err(
            "A rotina foi alterada. Solicite uma nova execução para confirmar a versão atual"
                .into(),
        );
    }
    let source = source_from_history(&history.source)?;
    if let Err(message) = validate_runtime(connection, &routine, &source) {
        routine_repository::finish_history(connection, history_id, false, 0, None, Some(&message))?;
        return Err(message);
    }
    if !routine_repository::begin_confirmed_execution(connection, history_id, routine.revision)? {
        return Err("A confirmação já foi usada ou expirou".into());
    }
    execute_validated(connection, &routine, source, history_id, executor, delay)
}

fn validate_runtime(
    connection: &Connection,
    routine: &Routine,
    source: &ActionSource,
) -> Result<Vec<RoutineActionSummary>, String> {
    if !routine.enabled {
        return Err("A rotina está desativada".into());
    }
    for (index, step) in routine.steps.iter().enumerate() {
        if step.order != (index + 1) as i64 {
            return Err("A ordem persistida dos passos é inválida".into());
        }
    }
    let active = routine
        .steps
        .iter()
        .filter(|step| step.enabled)
        .collect::<Vec<_>>();
    if active.is_empty() {
        return Err("A rotina não possui passos ativos".into());
    }
    let mut summaries = Vec::with_capacity(active.len());
    for (active_index, step) in active.iter().enumerate() {
        let action = automation_policy::action(&step.action_id)
            .ok_or_else(|| format!("Passo {}: ação não registrada", step.order))?;
        if action.target_type != step.target_type {
            return Err(format!(
                "Passo {}: associação entre ação e alvo inválida",
                step.order
            ));
        }
        let authorized = PolicyEngine::authorize(
            connection,
            &ActionRequest {
                action_id: step.action_id.clone(),
                source: source.clone(),
                target_id: Some(step.target_id.clone()),
            },
            action,
        )
        .map_err(|error| format!("Passo {}: {}", step.order, error.message))?;
        summaries.push(RoutineActionSummary {
            order: step.order,
            action_id: step.action_id.clone(),
            action_name: action.name.into(),
            target_name: authorized.target_name,
            delay_ms: transition_delay(connection, step, active.get(active_index + 1).copied())?,
        });
    }
    Ok(summaries)
}

fn execute_validated(
    connection: &Connection,
    routine: &Routine,
    source: ActionSource,
    history_id: i64,
    executor: &dyn ActionExecutor,
    delay: &dyn DelayProvider,
) -> Result<RoutineExecutionResult, String> {
    let active = routine
        .steps
        .iter()
        .filter(|step| step.enabled)
        .collect::<Vec<_>>();
    let mut completed = 0;
    for (active_index, step) in active.iter().enumerate() {
        let result = execute_with_routine_context(
            connection,
            ActionRequest {
                action_id: step.action_id.clone(),
                source: source.clone(),
                target_id: Some(step.target_id.clone()),
            },
            executor,
            Some((history_id, step.order)),
        )?;
        if !result.success {
            routine_repository::finish_history(
                connection,
                history_id,
                false,
                completed,
                Some(step.order),
                Some(&result.message),
            )?;
            return Ok(RoutineExecutionResult {
                success: false,
                status: "failed".into(),
                routine_id: routine.id.clone(),
                routine_name: routine.name.clone(),
                history_id,
                completed_steps: completed,
                failed_step: Some(step.order),
                error: Some(result.message),
                confirmation: None,
            });
        }
        completed += 1;
        routine_repository::update_progress(connection, history_id, completed)?;
        if active_index + 1 < active.len() {
            let milliseconds =
                transition_delay(connection, step, active.get(active_index + 1).copied())?;
            if milliseconds > 0 {
                delay.wait(milliseconds as u64);
            }
        }
    }
    routine_repository::finish_history(connection, history_id, true, completed, None, None)?;
    Ok(RoutineExecutionResult {
        success: true,
        status: "completed".into(),
        routine_id: routine.id.clone(),
        routine_name: routine.name.clone(),
        history_id,
        completed_steps: completed,
        failed_step: None,
        error: None,
        confirmation: None,
    })
}

fn transition_delay(
    connection: &Connection,
    current: &RoutineStep,
    next: Option<&RoutineStep>,
) -> Result<i64, String> {
    let mut milliseconds = current.delay_ms;
    let Some(next) = next else {
        return Ok(milliseconds);
    };
    if current.action_id == OPEN_APPLICATION && next.action_id == OPEN_WORKSPACE {
        let workspace = system_repository::get_workspace(connection, &next.target_id)?;
        if workspace
            .as_ref()
            .and_then(|item| item.application_id.as_deref())
            == Some(current.target_id.as_str())
        {
            milliseconds = milliseconds.max(2_000);
        }
    }
    Ok(milliseconds)
}

fn waiting_result(
    routine: &Routine,
    history_id: i64,
    actions: Vec<RoutineActionSummary>,
) -> RoutineExecutionResult {
    RoutineExecutionResult {
        success: false,
        status: "waiting_confirmation".into(),
        routine_id: routine.id.clone(),
        routine_name: routine.name.clone(),
        history_id,
        completed_steps: 0,
        failed_step: None,
        error: None,
        confirmation: Some(RoutineConfirmation {
            history_id,
            routine_id: routine.id.clone(),
            routine_name: routine.name.clone(),
            revision: routine.revision,
            actions,
        }),
    }
}

fn source_from_history(value: &str) -> Result<ActionSource, String> {
    match value {
        "user" => Ok(ActionSource::User),
        "ai" => Ok(ActionSource::Ai),
        "ui" => Ok(ActionSource::Ui),
        _ => Err("Origem da rotina inválida".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        automation_policy::ExecutionTarget,
        database::{
            self,
            automation_models::ApplicationInput,
            automation_repository,
            routine_models::{RoutineInput, RoutineStepInput},
            routine_repository,
            system_models::WorkspaceInput,
            system_repository,
        },
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    static FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct FakeExecutor {
        calls: Mutex<Vec<String>>,
        fail_at: Option<usize>,
    }

    impl ActionExecutor for FakeExecutor {
        fn execute(&self, target: &ExecutionTarget) -> Result<(), String> {
            let mut calls = self.calls.lock().unwrap();
            let name = match target {
                ExecutionTarget::Application { path } => path.clone(),
                ExecutionTarget::Workspace { workspace_path, .. } => workspace_path.clone(),
                ExecutionTarget::Reveal { workspace_path } => workspace_path.clone(),
                ExecutionTarget::Url { url } => url.clone(),
            };
            calls.push(name);
            if self.fail_at == Some(calls.len()) {
                Err("falha simulada".into())
            } else {
                Ok(())
            }
        }
    }

    #[derive(Default)]
    struct FakeDelay(Mutex<Vec<u64>>);
    impl DelayProvider for FakeDelay {
        fn wait(&self, milliseconds: u64) {
            self.0.lock().unwrap().push(milliseconds);
        }
    }

    fn database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection
    }

    fn executable() -> std::path::PathBuf {
        let sequence = FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "azriel-routine-{}-{sequence}.exe",
            std::process::id()
        ));
        std::fs::write(&path, b"fake").unwrap();
        path
    }

    fn register_app(connection: &Connection, id: &str) -> std::path::PathBuf {
        let path = executable();
        automation_repository::save_application(
            connection,
            &ApplicationInput {
                id: id.into(),
                name: id.into(),
                path: path.display().to_string(),
                enabled: true,
            },
        )
        .unwrap();
        path
    }

    fn routine(id: &str, targets: &[&str], confirmation_required: bool) -> RoutineInput {
        RoutineInput {
            id: id.into(),
            name: format!("Rotina {id}"),
            description: "Teste".into(),
            enabled: true,
            confirmation_required,
            steps: targets
                .iter()
                .enumerate()
                .map(|(index, target)| RoutineStepInput {
                    id: format!("{id}-step-{}", index + 1),
                    order: (index + 1) as i64,
                    action_id: "open_application".into(),
                    target_type: "application".into(),
                    target_id: (*target).into(),
                    delay_ms: if index == 0 { 250 } else { 0 },
                    enabled: true,
                })
                .collect(),
        }
    }

    #[test]
    fn routine_persists_order_and_rejects_unregistered_targets() {
        let mut connection = database();
        let first = register_app(&connection, "first");
        let second = register_app(&connection, "second");
        routine_repository::save_routine(
            &mut connection,
            &routine("persist", &["first", "second"], false),
        )
        .unwrap();
        let stored = routine_repository::get_routine(&connection, "persist")
            .unwrap()
            .unwrap();
        assert_eq!(
            stored
                .steps
                .iter()
                .map(|step| step.order)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert!(routine_repository::save_routine(
            &mut connection,
            &routine("invalid", &["missing"], false)
        )
        .is_err());
        let mut bypass = routine("bypass", &["first"], false);
        bypass.steps[0].action_id = "run_shell".into();
        assert!(routine_repository::save_routine(&mut connection, &bypass).is_err());
        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
    }

    #[test]
    fn required_confirmation_executes_once_and_audits_each_step() {
        let mut connection = database();
        let first = register_app(&connection, "first");
        let second = register_app(&connection, "second");
        routine_repository::save_routine(
            &mut connection,
            &routine("confirmed", &["first", "second"], true),
        )
        .unwrap();
        let executor = FakeExecutor {
            calls: Mutex::new(Vec::new()),
            fail_at: None,
        };
        let delay = FakeDelay::default();
        let pending = run_with(
            &connection,
            RunRoutineRequest {
                routine_id: "confirmed".into(),
                source: ActionSource::Ui,
            },
            &executor,
            &delay,
        )
        .unwrap();
        assert_eq!(pending.status, "waiting_confirmation");
        assert!(executor.calls.lock().unwrap().is_empty());
        let result = confirm_with(&connection, pending.history_id, &executor, &delay).unwrap();
        assert!(result.success);
        assert_eq!(executor.calls.lock().unwrap().len(), 2);
        assert_eq!(delay.0.lock().unwrap().as_slice(), [250]);
        assert!(confirm_with(&connection, pending.history_id, &executor, &delay).is_err());
        assert_eq!(
            connection
                .query_row(
                    "SELECT COUNT(*) FROM action_history WHERE routine_history_id=?1",
                    [pending.history_id],
                    |row| row.get::<_, i64>(0)
                )
                .unwrap(),
            2
        );
        let cancelled = run_with(
            &connection,
            RunRoutineRequest {
                routine_id: "confirmed".into(),
                source: ActionSource::Ui,
            },
            &executor,
            &delay,
        )
        .unwrap();
        assert!(routine_repository::cancel_waiting(&connection, cancelled.history_id).unwrap());
        assert!(confirm_with(&connection, cancelled.history_id, &executor, &delay).is_err());
        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
    }

    #[test]
    fn linked_workspace_waits_for_application_startup() {
        let mut connection = database();
        let application_path = register_app(&connection, "code");
        system_repository::save_workspace(
            &connection,
            &WorkspaceInput {
                id: "workspace".into(),
                name: "Workspace".into(),
                path: std::env::temp_dir().display().to_string(),
                project_id: None,
                application_id: Some("code".into()),
                enabled: true,
            },
        )
        .unwrap();
        let input = RoutineInput {
            id: "startup".into(),
            name: "Inicialização segura".into(),
            description: String::new(),
            enabled: true,
            confirmation_required: false,
            steps: vec![
                RoutineStepInput {
                    id: "startup-app".into(),
                    order: 1,
                    action_id: OPEN_APPLICATION.into(),
                    target_type: "application".into(),
                    target_id: "code".into(),
                    delay_ms: 0,
                    enabled: true,
                },
                RoutineStepInput {
                    id: "startup-workspace".into(),
                    order: 2,
                    action_id: OPEN_WORKSPACE.into(),
                    target_type: "workspace".into(),
                    target_id: "workspace".into(),
                    delay_ms: 0,
                    enabled: true,
                },
            ],
        };
        routine_repository::save_routine(&mut connection, &input).unwrap();
        let executor = FakeExecutor {
            calls: Mutex::new(Vec::new()),
            fail_at: None,
        };
        let delay = FakeDelay::default();
        let result = run_with(
            &connection,
            RunRoutineRequest {
                routine_id: "startup".into(),
                source: ActionSource::Ui,
            },
            &executor,
            &delay,
        )
        .unwrap();
        assert!(result.success);
        assert_eq!(executor.calls.lock().unwrap().len(), 2);
        assert_eq!(delay.0.lock().unwrap().as_slice(), [2_000]);
        std::fs::remove_file(application_path).unwrap();
    }

    #[test]
    fn stop_on_error_prevents_later_steps_and_records_failure() {
        let mut connection = database();
        let paths = [
            register_app(&connection, "first"),
            register_app(&connection, "second"),
            register_app(&connection, "third"),
        ];
        routine_repository::save_routine(
            &mut connection,
            &routine("failure", &["first", "second", "third"], false),
        )
        .unwrap();
        let executor = FakeExecutor {
            calls: Mutex::new(Vec::new()),
            fail_at: Some(2),
        };
        let result = run_with(
            &connection,
            RunRoutineRequest {
                routine_id: "failure".into(),
                source: ActionSource::Ui,
            },
            &executor,
            &FakeDelay::default(),
        )
        .unwrap();
        assert!(!result.success);
        assert_eq!(result.completed_steps, 1);
        assert_eq!(result.failed_step, Some(2));
        assert_eq!(executor.calls.lock().unwrap().len(), 2);
        let history = routine_repository::get_history(&connection, result.history_id)
            .unwrap()
            .unwrap();
        assert_eq!(history.status, "failed");
        for path in paths {
            std::fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn all_steps_are_validated_before_any_execution() {
        let mut connection = database();
        let first = register_app(&connection, "first");
        let second = register_app(&connection, "second");
        routine_repository::save_routine(
            &mut connection,
            &routine("validate", &["first", "second"], false),
        )
        .unwrap();
        connection
            .execute("UPDATE applications SET enabled=0 WHERE id='second'", [])
            .unwrap();
        let executor = FakeExecutor {
            calls: Mutex::new(Vec::new()),
            fail_at: None,
        };
        let result = run_with(
            &connection,
            RunRoutineRequest {
                routine_id: "validate".into(),
                source: ActionSource::Ui,
            },
            &executor,
            &FakeDelay::default(),
        );
        assert!(result.is_err());
        assert!(executor.calls.lock().unwrap().is_empty());
        assert_eq!(
            routine_repository::list_history(&connection, 10)
                .unwrap()
                .len(),
            0
        );
        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
    }

    #[test]
    fn routine_survives_database_reopen() {
        let sequence = FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let db_path = std::env::temp_dir().join(format!(
            "azriel-routine-db-{}-{sequence}.sqlite",
            std::process::id()
        ));
        let app_path = executable();
        {
            let mut connection = database::open(&db_path).unwrap();
            automation_repository::save_application(
                &connection,
                &ApplicationInput {
                    id: "persistent-app".into(),
                    name: "Persistente".into(),
                    path: app_path.display().to_string(),
                    enabled: true,
                },
            )
            .unwrap();
            let stored = routine_repository::save_routine(
                &mut connection,
                &routine("reopen", &["persistent-app"], false),
            )
            .unwrap();
            routine_repository::start_history(&connection, &stored, "ui", true).unwrap();
        }
        let connection = database::open(&db_path).unwrap();
        let stored = routine_repository::get_routine(&connection, "reopen")
            .unwrap()
            .unwrap();
        assert_eq!(stored.name, "Rotina reopen");
        assert_eq!(stored.steps[0].target_id, "persistent-app");
        assert_eq!(
            routine_repository::list_history(&connection, 1).unwrap()[0].status,
            "cancelled"
        );
        drop(connection);
        std::fs::remove_file(db_path).unwrap();
        std::fs::remove_file(app_path).unwrap();
    }
}
