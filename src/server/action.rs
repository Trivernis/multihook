use crate::server::command_template::CommandTemplate;
use crate::utils::error::MultihookResult;
use serde_json::Value;
use std::fs::read_to_string;
use std::path::PathBuf;
use tokio::process::Command;

pub struct HookAction {
    command: CommandTemplate,
}

impl HookAction {
    pub fn new<S: ToString>(command: S) -> Self {
        Self {
            command: CommandTemplate::new(command),
        }
    }

    pub async fn execute(&self, body: &str) -> MultihookResult<()> {
        let json_body: Value = serde_json::from_str(body).unwrap_or_default();
        let command = self.command.evaluate(&json_body);

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
}

impl From<String> for HookAction {
    fn from(action: String) -> Self {
        let path = PathBuf::from(&action);
        let contents = read_to_string(path).unwrap_or(action);
        Self::new(contents)
    }
}
