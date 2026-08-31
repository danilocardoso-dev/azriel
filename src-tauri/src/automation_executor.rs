use crate::automation_policy::ExecutionTarget;

pub trait ActionExecutor: Send + Sync {
    fn execute(&self, target: &ExecutionTarget) -> Result<(), String>;
}

pub struct NativeActionExecutor;

impl ActionExecutor for NativeActionExecutor {
    fn execute(&self, target: &ExecutionTarget) -> Result<(), String> {
        execute_native(target)
    }
}

#[cfg(target_os = "windows")]
fn execute_native(target: &ExecutionTarget) -> Result<(), String> {
    use std::path::Path;
    use std::process::Command;
    let spawn = |command: &mut Command| {
        command
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Falha ao iniciar a ação no Windows: {error}"))
    };
    match target {
        ExecutionTarget::Application { path } => {
            if !Path::new(path).is_file() {
                return Err("O executável autorizado não está disponível".into());
            }
            spawn(&mut Command::new(path))
        }
        ExecutionTarget::Workspace {
            application_path,
            workspace_path,
        } => {
            if !Path::new(application_path).is_file() {
                return Err("O aplicativo autorizado não está disponível".into());
            }
            if !Path::new(workspace_path).is_dir() {
                return Err("O workspace autorizado não está disponível".into());
            }
            spawn(Command::new(application_path).arg(workspace_path))
        }
        ExecutionTarget::Reveal { workspace_path } => {
            if !Path::new(workspace_path).is_dir() {
                return Err("O workspace autorizado não está disponível".into());
            }
            spawn(Command::new("explorer.exe").arg(workspace_path))
        }
        ExecutionTarget::Url { url } => spawn(
            Command::new("rundll32.exe")
                .arg("url.dll,FileProtocolHandler")
                .arg(url),
        ),
    }
}

#[cfg(not(target_os = "windows"))]
fn execute_native(_target: &ExecutionTarget) -> Result<(), String> {
    Err("A Automation Core v0.8.0 está disponível apenas no Windows".into())
}
