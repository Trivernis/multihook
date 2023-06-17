use crate::utils::error::{MultihookError, MultihookResult};

use self::template::ActionTemplate;
use std::{collections::HashMap, sync::Arc};
use tokio::{process::Command, sync::Semaphore};

mod template;

static MAX_CONCURRENCY: usize = 256;

#[derive(Clone)]
pub struct Action {
    template: ActionTemplate,
    semaphore: Arc<Semaphore>,
}

impl Action {
    /// Creates a new command that also checks for parallel runs
    pub fn new<S: Into<String>>(command: S, allow_parallel: bool) -> Self {
        let semaphore = if allow_parallel {
            Semaphore::new(MAX_CONCURRENCY)
        } else {
            Semaphore::new(1)
        };

        Self {
            template: ActionTemplate::new(command.into()),
            semaphore: Arc::new(semaphore),
        }
    }

    /// Executes the action
    pub async fn run(
        &self,
        body: &serde_json::Value,
        env: &HashMap<&str, String>,
    ) -> MultihookResult<()> {
        let command = self.template.evaluate(&body);
        log::debug!("Acquiring lock for parallel runs...");
        let permit = self.semaphore.acquire().await.unwrap();
        log::debug!("Lock acquired. Running command...");
        std::mem::drop(permit);

        let output = Command::new("sh")
            .envs(env)
            .arg("-c")
            .arg(command)
            .kill_on_drop(true)
            .output()
            .await?;
        log::debug!("Command finished. Releasing parallel lock...");

        let stderr = String::from_utf8_lossy(&output.stderr[..]);
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        log::debug!("Command output is: {}", stdout);

        if output.status.success() {
            Ok(())
        } else {
            log::error!("Errors occurred during command execution: {}", stderr);
            Err(MultihookError::ActionError(stderr.into_owned()))
        }
    }
}
