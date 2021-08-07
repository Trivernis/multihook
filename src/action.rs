use crate::error::MultihookResult;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub enum HookAction {
    Script(PathBuf),
    Command(String),
}

impl HookAction {
    pub async fn execute(&self, body: &str) -> MultihookResult<()> {
        match self {
            HookAction::Script(s) => Self::execute_script(s, body).await,
            HookAction::Command(c) => Self::execute_command(c, body).await,
        }
    }

    async fn execute_command(command: &str, body: &str) -> MultihookResult<()> {
        let output = Command::new("sh")
            .env("HOOK_BODY", body)
            .arg("-c")
            .arg(command)
            .output()
            .await?;
        let stderr = String::from_utf8_lossy(&output.stderr[..]);
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        log::debug!("Command output is: {}", stdout);

        if stderr.len() > 0 {
            log::error!("Errors occurred during command execution: {}", stderr);
        }

        Ok(())
    }

    async fn execute_script(script: &PathBuf, body: &str) -> MultihookResult<()> {
        let output = Command::new(script).env("HOOK_BODY", body).output().await?;
        let stderr = String::from_utf8_lossy(&output.stderr[..]);
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        log::debug!("Script output is: {}", stdout);

        if stderr.len() > 0 {
            log::error!("Errors occurred during script execution: {}", stderr);
        }

        Ok(())
    }
}

impl From<String> for HookAction {
    fn from(action: String) -> Self {
        let path = PathBuf::from(&action);
        if Path::new(&path).exists() {
            Self::Script(path)
        } else {
            Self::Command(action)
        }
    }
}
