use crate::database::{automation_repository, system_repository};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const OPEN_APPLICATION: &str = "open_application";
pub const OPEN_WORKSPACE: &str = "open_workspace";
pub const OPEN_PROJECT: &str = "open_project";
pub const REVEAL_WORKSPACE: &str = "reveal_workspace";
pub const OPEN_REGISTERED_URL: &str = "open_registered_url";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
pub enum ActionPermission {
    Read,
    SafeWrite,
    ConfirmWrite,
    Blocked,
}

impl ActionPermission {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::SafeWrite => "safe_write",
            Self::ConfirmWrite => "confirm_write",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredAction {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub permission: ActionPermission,
    pub target_type: &'static str,
}

const ACTIONS: [RegisteredAction; 5] = [
    RegisteredAction {
        id: OPEN_APPLICATION,
        name: "Abrir aplicativo",
        description: "Abre um aplicativo previamente autorizado",
        permission: ActionPermission::SafeWrite,
        target_type: "application",
    },
    RegisteredAction {
        id: OPEN_WORKSPACE,
        name: "Abrir workspace",
        description: "Abre um workspace no aplicativo autorizado relacionado",
        permission: ActionPermission::SafeWrite,
        target_type: "workspace",
    },
    RegisteredAction {
        id: OPEN_PROJECT,
        name: "Abrir projeto",
        description: "Resolve um projeto para seu único workspace autorizado",
        permission: ActionPermission::SafeWrite,
        target_type: "project",
    },
    RegisteredAction {
        id: REVEAL_WORKSPACE,
        name: "Revelar workspace",
        description: "Mostra um workspace autorizado no Explorador",
        permission: ActionPermission::SafeWrite,
        target_type: "workspace",
    },
    RegisteredAction {
        id: OPEN_REGISTERED_URL,
        name: "Abrir URL registrada",
        description: "Abre uma URL previamente autorizada",
        permission: ActionPermission::SafeWrite,
        target_type: "url",
    },
];

pub fn actions() -> &'static [RegisteredAction] {
    &ACTIONS
}
pub fn action(id: &str) -> Option<&'static RegisteredAction> {
    ACTIONS.iter().find(|item| item.id == id)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionSource {
    User,
    Ai,
    Ui,
}

impl ActionSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Ai => "ai",
            Self::Ui => "ui",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action_id: String,
    pub source: ActionSource,
    pub target_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationRequest {
    pub action_id: String,
    pub target_name: String,
    pub description: String,
    pub impact: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub error_code: Option<String>,
    pub action_id: String,
    pub target_name: Option<String>,
    pub history_id: i64,
    pub confirmation: Option<ConfirmationRequest>,
}

#[derive(Debug, Clone)]
pub enum ExecutionTarget {
    Application {
        path: String,
    },
    Workspace {
        application_path: String,
        workspace_path: String,
    },
    Reveal {
        workspace_path: String,
    },
    Url {
        url: String,
    },
}

pub struct AuthorizedAction {
    pub target_type: &'static str,
    pub target_name: String,
    pub execution: ExecutionTarget,
}

pub struct PolicyError {
    pub code: &'static str,
    pub message: String,
}
impl PolicyError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

pub enum ConfirmationDecision {
    Execute,
    Request,
}
pub struct ConfirmationGate;
impl ConfirmationGate {
    pub fn evaluate(permission: ActionPermission) -> ConfirmationDecision {
        match permission {
            ActionPermission::SafeWrite => ConfirmationDecision::Execute,
            ActionPermission::ConfirmWrite => ConfirmationDecision::Request,
            _ => ConfirmationDecision::Request,
        }
    }
}

pub struct PolicyEngine;
impl PolicyEngine {
    pub fn authorize(
        connection: &Connection,
        request: &ActionRequest,
        registered: &RegisteredAction,
    ) -> Result<AuthorizedAction, PolicyError> {
        if registered.permission != ActionPermission::SafeWrite {
            return Err(PolicyError::new(
                "ACTION_BLOCKED",
                "A ação não está autorizada para execução",
            ));
        }
        let target_id = request
            .target_id
            .as_deref()
            .ok_or_else(|| PolicyError::new("TARGET_REQUIRED", "Informe um alvo registrado"))?;
        automation_repository::validate_id(target_id, "alvo")
            .map_err(|message| PolicyError::new("INVALID_TARGET", message))?;
        match registered.id {
            OPEN_APPLICATION => Self::application(connection, target_id),
            OPEN_WORKSPACE => Self::workspace(connection, target_id),
            OPEN_PROJECT => Self::project(connection, target_id),
            REVEAL_WORKSPACE => Self::reveal(connection, target_id),
            OPEN_REGISTERED_URL => Self::url(connection, target_id),
            _ => Err(PolicyError::new("ACTION_BLOCKED", "Ação não registrada")),
        }
    }

    fn application(connection: &Connection, id: &str) -> Result<AuthorizedAction, PolicyError> {
        let app = automation_repository::get_application(connection, id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new(
                    "TARGET_NOT_FOUND",
                    "O aplicativo não está registrado como autorizado",
                )
            })?;
        if !app.enabled {
            return Err(PolicyError::new(
                "TARGET_DISABLED",
                "O aplicativo autorizado está desativado",
            ));
        }
        Ok(AuthorizedAction {
            target_type: "application",
            target_name: app.name,
            execution: ExecutionTarget::Application { path: app.path },
        })
    }

    fn workspace(connection: &Connection, id: &str) -> Result<AuthorizedAction, PolicyError> {
        let workspace = system_repository::get_workspace(connection, id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new("TARGET_NOT_FOUND", "O workspace não está registrado")
            })?;
        if !workspace.enabled {
            return Err(PolicyError::new(
                "TARGET_DISABLED",
                "O workspace está desativado",
            ));
        }
        let application_id = workspace.application_id.as_deref().ok_or_else(|| {
            PolicyError::new(
                "HANDLER_REQUIRED",
                "O workspace não possui aplicativo autorizado associado",
            )
        })?;
        let app = automation_repository::get_application(connection, application_id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new(
                    "HANDLER_NOT_FOUND",
                    "O aplicativo associado ao workspace não existe",
                )
            })?;
        if !app.enabled {
            return Err(PolicyError::new(
                "HANDLER_DISABLED",
                "O aplicativo associado ao workspace está desativado",
            ));
        }
        Ok(AuthorizedAction {
            target_type: "workspace",
            target_name: workspace.name,
            execution: ExecutionTarget::Workspace {
                application_path: app.path,
                workspace_path: workspace.path,
            },
        })
    }

    fn project(connection: &Connection, id: &str) -> Result<AuthorizedAction, PolicyError> {
        let project_name = connection
            .query_row("SELECT name FROM projects WHERE id=?1", [id], |row| {
                row.get::<_, String>(0)
            })
            .optional()
            .map_err(|error| db_error(error.to_string()))?
            .ok_or_else(|| PolicyError::new("TARGET_NOT_FOUND", "O projeto não está registrado"))?;
        let matching = system_repository::list_workspaces(connection)
            .map_err(db_error)?
            .into_iter()
            .filter(|workspace| workspace.enabled && workspace.project_id.as_deref() == Some(id))
            .collect::<Vec<_>>();
        if matching.is_empty() {
            return Err(PolicyError::new(
                "WORKSPACE_REQUIRED",
                "O projeto não possui workspace autorizado",
            ));
        }
        if matching.len() > 1 {
            return Err(PolicyError::new("AMBIGUOUS_TARGET", "O projeto possui mais de um workspace ativo; abra o workspace desejado diretamente"));
        }
        let workspace = &matching[0];
        let application_id = workspace.application_id.as_deref().ok_or_else(|| {
            PolicyError::new(
                "HANDLER_REQUIRED",
                "O workspace do projeto não possui aplicativo autorizado associado",
            )
        })?;
        let app = automation_repository::get_application(connection, application_id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new(
                    "HANDLER_NOT_FOUND",
                    "O aplicativo associado ao projeto não existe",
                )
            })?;
        if !app.enabled {
            return Err(PolicyError::new(
                "HANDLER_DISABLED",
                "O aplicativo associado ao projeto está desativado",
            ));
        }
        Ok(AuthorizedAction {
            target_type: "project",
            target_name: project_name,
            execution: ExecutionTarget::Workspace {
                application_path: app.path,
                workspace_path: workspace.path.clone(),
            },
        })
    }

    fn reveal(connection: &Connection, id: &str) -> Result<AuthorizedAction, PolicyError> {
        let workspace = system_repository::get_workspace(connection, id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new("TARGET_NOT_FOUND", "O workspace não está registrado")
            })?;
        if !workspace.enabled {
            return Err(PolicyError::new(
                "TARGET_DISABLED",
                "O workspace está desativado",
            ));
        }
        Ok(AuthorizedAction {
            target_type: "workspace",
            target_name: workspace.name,
            execution: ExecutionTarget::Reveal {
                workspace_path: workspace.path,
            },
        })
    }

    fn url(connection: &Connection, id: &str) -> Result<AuthorizedAction, PolicyError> {
        let url = automation_repository::get_url(connection, id)
            .map_err(db_error)?
            .ok_or_else(|| {
                PolicyError::new(
                    "TARGET_NOT_FOUND",
                    "A URL não está registrada como autorizada",
                )
            })?;
        if !url.enabled {
            return Err(PolicyError::new(
                "TARGET_DISABLED",
                "A URL registrada está desativada",
            ));
        }
        Ok(AuthorizedAction {
            target_type: "url",
            target_name: url.name,
            execution: ExecutionTarget::Url { url: url.url },
        })
    }
}

fn db_error(message: impl ToString) -> PolicyError {
    PolicyError::new("DATABASE_ERROR", message.to_string())
}

use rusqlite::OptionalExtension;
