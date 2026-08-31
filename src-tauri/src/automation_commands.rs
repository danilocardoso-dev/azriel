use crate::{
    automation_executor::{ActionExecutor, NativeActionExecutor},
    automation_policy::{
        self, ActionPermission, ActionRequest, ActionResult, ConfirmationDecision,
        ConfirmationGate, PolicyEngine,
    },
    database::{
        automation_models::{
            ActionHistory, Application, ApplicationInput, RegisteredUrl, RegisteredUrlInput,
        },
        automation_repository, DatabaseState,
    },
};
use tauri::State;

fn lock<'a>(
    state: &'a State<'_, DatabaseState>,
) -> Result<std::sync::MutexGuard<'a, rusqlite::Connection>, String> {
    state
        .connection
        .lock()
        .map_err(|_| "O banco de dados está indisponível".into())
}

#[tauri::command]
pub fn list_registered_actions() -> Vec<automation_policy::RegisteredAction> {
    automation_policy::actions().to_vec()
}

#[tauri::command]
pub fn list_applications(state: State<'_, DatabaseState>) -> Result<Vec<Application>, String> {
    let connection = lock(&state)?;
    automation_repository::list_applications(&connection)
}
#[tauri::command]
pub fn save_application(
    state: State<'_, DatabaseState>,
    input: ApplicationInput,
) -> Result<Vec<Application>, String> {
    let connection = lock(&state)?;
    automation_repository::save_application(&connection, &input)?;
    automation_repository::list_applications(&connection)
}
#[tauri::command]
pub fn delete_application(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Vec<Application>, String> {
    let connection = lock(&state)?;
    automation_repository::delete_application(&connection, &id)?;
    automation_repository::list_applications(&connection)
}

#[tauri::command]
pub fn list_registered_urls(state: State<'_, DatabaseState>) -> Result<Vec<RegisteredUrl>, String> {
    let connection = lock(&state)?;
    automation_repository::list_urls(&connection)
}
#[tauri::command]
pub fn save_registered_url(
    state: State<'_, DatabaseState>,
    input: RegisteredUrlInput,
) -> Result<Vec<RegisteredUrl>, String> {
    let connection = lock(&state)?;
    automation_repository::save_url(&connection, &input)?;
    automation_repository::list_urls(&connection)
}
#[tauri::command]
pub fn delete_registered_url(
    state: State<'_, DatabaseState>,
    id: String,
) -> Result<Vec<RegisteredUrl>, String> {
    let connection = lock(&state)?;
    automation_repository::delete_url(&connection, &id)?;
    automation_repository::list_urls(&connection)
}

#[tauri::command]
pub fn list_action_history(
    state: State<'_, DatabaseState>,
    limit: Option<i64>,
) -> Result<Vec<ActionHistory>, String> {
    let connection = lock(&state)?;
    automation_repository::list_history(&connection, limit.unwrap_or(100))
}

#[tauri::command]
pub fn execute_automation_action(
    state: State<'_, DatabaseState>,
    request: ActionRequest,
) -> Result<ActionResult, String> {
    let connection = lock(&state)?;
    execute_with(&connection, request, &NativeActionExecutor)
}

pub fn execute_with(
    connection: &rusqlite::Connection,
    request: ActionRequest,
    executor: &dyn ActionExecutor,
) -> Result<ActionResult, String> {
    eprintln!("[AUTOMATION] action requested");
    let safe_action_id = if request.action_id.len() <= 100
        && request
            .action_id
            .chars()
            .all(|value| value.is_ascii_lowercase() || value == '_')
    {
        request.action_id.as_str()
    } else {
        "invalid_action"
    };
    let registered = automation_policy::action(safe_action_id);
    let permission = registered
        .map(|item| item.permission)
        .unwrap_or(ActionPermission::Blocked);
    let confirmation_required = permission == ActionPermission::ConfirmWrite;
    let safe_target_id = request.target_id.as_deref().filter(|value| {
        value.len() <= 100
            && value.chars().all(|character| {
                character.is_ascii_alphanumeric() || character == '-' || character == '_'
            })
    });
    let history_id = automation_repository::start_history(
        connection,
        safe_action_id,
        request.source.as_str(),
        safe_target_id,
        permission.as_str(),
        confirmation_required,
    )?;
    let Some(action) = registered else {
        eprintln!("[AUTOMATION] policy result: blocked / unregistered action");
        return finish_denied(
            connection,
            history_id,
            safe_action_id,
            "ACTION_NOT_REGISTERED",
            "A ação solicitada não está registrada",
        );
    };
    if matches!(
        ConfirmationGate::evaluate(action.permission),
        ConfirmationDecision::Request
    ) {
        eprintln!("[AUTOMATION] policy result: confirmation required");
        automation_repository::complete_history(
            connection,
            history_id,
            Some(action.target_type),
            None,
            false,
            false,
            Some("A ação requer confirmação e não pode ser executada nesta versão"),
        )?;
        return Ok(ActionResult {
            success: false,
            message: "A ação requer confirmação explícita".into(),
            error_code: Some("CONFIRMATION_REQUIRED".into()),
            action_id: action.id.into(),
            target_name: None,
            history_id,
            confirmation: Some(automation_policy::ConfirmationRequest {
                action_id: action.id.into(),
                target_name: request
                    .target_id
                    .clone()
                    .unwrap_or_else(|| "Alvo não informado".into()),
                description: action.description.into(),
                impact: "Ação de escrita aguardando confirmação segura".into(),
            }),
        });
    }
    let authorized = match PolicyEngine::authorize(connection, &request, action) {
        Ok(value) => {
            eprintln!("[AUTOMATION] policy result: authorized ({})", action.id);
            value
        }
        Err(error) => {
            eprintln!("[AUTOMATION] policy result: denied ({})", error.code);
            return finish_denied(
                connection,
                history_id,
                action.id,
                error.code,
                &error.message,
            )
        }
    };
    eprintln!("[AUTOMATION] execution started ({})", action.id);
    match executor.execute(&authorized.execution) {
        Ok(()) => {
            eprintln!("[AUTOMATION] execution completed ({})", action.id);
            automation_repository::complete_history(
                connection,
                history_id,
                Some(authorized.target_type),
                Some(&authorized.target_name),
                false,
                true,
                None,
            )?;
            Ok(ActionResult {
                success: true,
                message: format!("{} executado com sucesso", action.name),
                error_code: None,
                action_id: action.id.into(),
                target_name: Some(authorized.target_name),
                history_id,
                confirmation: None,
            })
        }
        Err(message) => {
            eprintln!("[AUTOMATION] execution failed ({})", action.id);
            automation_repository::complete_history(
                connection,
                history_id,
                Some(authorized.target_type),
                Some(&authorized.target_name),
                false,
                false,
                Some(&message),
            )?;
            Ok(ActionResult {
                success: false,
                message: message.clone(),
                error_code: Some("NATIVE_EXECUTION_FAILED".into()),
                action_id: action.id.into(),
                target_name: Some(authorized.target_name),
                history_id,
                confirmation: None,
            })
        }
    }
}

fn finish_denied(
    connection: &rusqlite::Connection,
    history_id: i64,
    action_id: &str,
    code: &str,
    message: &str,
) -> Result<ActionResult, String> {
    automation_repository::complete_history(
        connection,
        history_id,
        None,
        None,
        false,
        false,
        Some(message),
    )?;
    Ok(ActionResult {
        success: false,
        message: message.into(),
        error_code: Some(code.into()),
        action_id: action_id.into(),
        target_name: None,
        history_id,
        confirmation: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        automation_policy::{ActionSource, ExecutionTarget},
        database::{
            self,
            automation_models::{ApplicationInput, RegisteredUrlInput},
            system_models::WorkspaceInput,
            system_repository,
        },
    };
    use rusqlite::Connection;
    use std::sync::{atomic::{AtomicUsize, Ordering}, Mutex};

    static EXECUTABLE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct FakeExecutor {
        calls: Mutex<Vec<&'static str>>,
        failure: Option<&'static str>,
    }
    impl FakeExecutor {
        fn success() -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                failure: None,
            }
        }
    }
    impl ActionExecutor for FakeExecutor {
        fn execute(&self, target: &ExecutionTarget) -> Result<(), String> {
            let kind = match target {
                ExecutionTarget::Application { .. } => "application",
                ExecutionTarget::Workspace { .. } => "workspace",
                ExecutionTarget::Reveal { .. } => "reveal",
                ExecutionTarget::Url { .. } => "url",
            };
            self.calls.lock().unwrap().push(kind);
            self.failure.map_or(Ok(()), |message| Err(message.into()))
        }
    }

    fn test_database() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        database::initialize(&mut connection).unwrap();
        connection
    }

    fn executable_file() -> std::path::PathBuf {
        let sequence = EXECUTABLE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("azriel-fake-{}-{sequence}.exe", std::process::id()));
        std::fs::write(&path, b"fake").unwrap();
        path
    }

    #[test]
    fn registry_exposes_only_the_five_safe_actions() {
        let actions = automation_policy::actions();
        assert_eq!(actions.len(), 5);
        assert!(actions
            .iter()
            .all(|action| action.permission == ActionPermission::SafeWrite));
        assert!(actions
            .iter()
            .all(|action| !action.id.contains("shell") && !action.id.contains("command")));
    }

    #[test]
    fn authorized_application_executes_through_fake_and_is_audited() {
        let connection = test_database();
        let path = executable_file();
        automation_repository::save_application(
            &connection,
            &ApplicationInput {
                id: "code".into(),
                name: "Code".into(),
                path: path.display().to_string(),
                enabled: true,
            },
        )
        .unwrap();
        let executor = FakeExecutor::success();
        let result = execute_with(
            &connection,
            ActionRequest {
                action_id: "open_application".into(),
                source: ActionSource::Ai,
                target_id: Some("code".into()),
            },
            &executor,
        )
        .unwrap();
        assert!(result.success);
        assert_eq!(executor.calls.lock().unwrap().as_slice(), ["application"]);
        let history = automation_repository::list_history(&connection, 10).unwrap();
        assert_eq!(history[0].success, Some(true));
        assert_eq!(history[0].source, "ai");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn blocked_unknown_and_disabled_targets_are_audited_without_execution() {
        let connection = test_database();
        let executor = FakeExecutor::success();
        let blocked = execute_with(
            &connection,
            ActionRequest {
                action_id: "run_shell".into(),
                source: ActionSource::Ai,
                target_id: Some("anything".into()),
            },
            &executor,
        )
        .unwrap();
        assert_eq!(blocked.error_code.as_deref(), Some("ACTION_NOT_REGISTERED"));
        let path = executable_file();
        automation_repository::save_application(
            &connection,
            &ApplicationInput {
                id: "disabled".into(),
                name: "Desativado".into(),
                path: path.display().to_string(),
                enabled: false,
            },
        )
        .unwrap();
        let disabled = execute_with(
            &connection,
            ActionRequest {
                action_id: "open_application".into(),
                source: ActionSource::Ui,
                target_id: Some("disabled".into()),
            },
            &executor,
        )
        .unwrap();
        assert_eq!(disabled.error_code.as_deref(), Some("TARGET_DISABLED"));
        assert!(executor.calls.lock().unwrap().is_empty());
        assert_eq!(
            automation_repository::list_history(&connection, 10)
                .unwrap()
                .len(),
            2
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn project_without_workspace_and_native_failure_return_useful_results() {
        let connection = test_database();
        let executor = FakeExecutor::success();
        let missing_workspace = execute_with(
            &connection,
            ActionRequest {
                action_id: "open_project".into(),
                source: ActionSource::User,
                target_id: Some("azriel".into()),
            },
            &executor,
        )
        .unwrap();
        assert_eq!(
            missing_workspace.error_code.as_deref(),
            Some("WORKSPACE_REQUIRED")
        );

        let path = executable_file();
        automation_repository::save_application(
            &connection,
            &ApplicationInput {
                id: "code".into(),
                name: "Code".into(),
                path: path.display().to_string(),
                enabled: true,
            },
        )
        .unwrap();
        system_repository::save_workspace(
            &connection,
            &WorkspaceInput {
                id: "workspace".into(),
                name: "Azriel".into(),
                path: std::env::temp_dir().display().to_string(),
                project_id: Some("azriel".into()),
                application_id: Some("code".into()),
                enabled: true,
            },
        )
        .unwrap();
        let failing = FakeExecutor {
            calls: Mutex::new(Vec::new()),
            failure: Some("falha nativa"),
        };
        let result = execute_with(
            &connection,
            ActionRequest {
                action_id: "open_project".into(),
                source: ActionSource::Ui,
                target_id: Some("azriel".into()),
            },
            &failing,
        )
        .unwrap();
        assert_eq!(
            result.error_code.as_deref(),
            Some("NATIVE_EXECUTION_FAILED")
        );
        assert_eq!(
            automation_repository::list_history(&connection, 1).unwrap()[0]
                .error
                .as_deref(),
            Some("falha nativa")
        );
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn registered_urls_are_validated_and_executed_only_by_id() {
        let connection = test_database();
        assert!(automation_repository::save_url(
            &connection,
            &RegisteredUrlInput {
                id: "bad".into(),
                name: "Credencial".into(),
                url: "https://user:secret@example.com".into(),
                enabled: true
            }
        )
        .is_err());
        automation_repository::save_url(
            &connection,
            &RegisteredUrlInput {
                id: "azriel_web".into(),
                name: "Azriel Web".into(),
                url: "https://example.com/azriel".into(),
                enabled: true,
            },
        )
        .unwrap();
        let executor = FakeExecutor::success();
        let result = execute_with(
            &connection,
            ActionRequest {
                action_id: "open_registered_url".into(),
                source: ActionSource::Ai,
                target_id: Some("azriel_web".into()),
            },
            &executor,
        )
        .unwrap();
        assert!(result.success);
        assert_eq!(executor.calls.lock().unwrap().as_slice(), ["url"]);
    }

    #[test]
    fn authorized_workspace_can_open_and_reveal_through_specific_actions() {
        let connection = test_database();
        let path = executable_file();
        automation_repository::save_application(
            &connection,
            &ApplicationInput {
                id: "editor".into(),
                name: "Editor".into(),
                path: path.display().to_string(),
                enabled: true,
            },
        )
        .unwrap();
        system_repository::save_workspace(
            &connection,
            &WorkspaceInput {
                id: "authorized_workspace".into(),
                name: "Workspace autorizado".into(),
                path: std::env::temp_dir().display().to_string(),
                project_id: None,
                application_id: Some("editor".into()),
                enabled: true,
            },
        )
        .unwrap();
        let executor = FakeExecutor::success();
        for action_id in ["open_workspace", "reveal_workspace"] {
            let result = execute_with(
                &connection,
                ActionRequest {
                    action_id: action_id.into(),
                    source: ActionSource::Ai,
                    target_id: Some("authorized_workspace".into()),
                },
                &executor,
            )
            .unwrap();
            assert!(result.success);
        }
        assert_eq!(executor.calls.lock().unwrap().as_slice(), ["workspace", "reveal"]);
        assert_eq!(automation_repository::list_history(&connection, 10).unwrap().len(), 2);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn confirmation_gate_never_executes_confirm_write_in_this_version() {
        assert!(matches!(
            ConfirmationGate::evaluate(ActionPermission::ConfirmWrite),
            ConfirmationDecision::Request
        ));
        assert!(matches!(
            ConfirmationGate::evaluate(ActionPermission::SafeWrite),
            ConfirmationDecision::Execute
        ));
    }
}
