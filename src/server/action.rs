use crate::secret_validation::SecretValidator;
use crate::server::command_template::CommandTemplate;
use crate::utils::error::{MultihookError, MultihookResult};
use crate::utils::settings::{EndpointSettings, SecretSettings};
use hyper::http::request::Parts;
use hyper::{Body, Request};
use serde_json::Value;
use std::fs::read_to_string;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Semaphore;

static MAX_CONCURRENCY: usize = 256;

#[derive(Clone)]
pub struct HookAction {
    command: CommandTemplate,
    parallel_lock: Arc<Semaphore>,
    run_detached: bool,
    secret: Option<SecretSettings>,
}

impl HookAction {
    pub fn new<S: ToString>(
        command: S,
        parallel: bool,
        detached: bool,
        secret: Option<SecretSettings>,
    ) -> Self {
        let parallel_lock = if parallel {
            Semaphore::new(MAX_CONCURRENCY)
        } else {
            Semaphore::new(1)
        };
        Self {
            command: CommandTemplate::new(command),
            parallel_lock: Arc::new(parallel_lock),
            run_detached: detached,
            secret,
        }
    }

    pub async fn execute(&self, req: Request<Body>) -> MultihookResult<()> {
        let (parts, body) = req.into_parts();
        let body = hyper::body::to_bytes(body).await?.to_vec();

        self.validate_secret(&parts, &body)?;
        let body = String::from_utf8(body)?;

        if self.run_detached {
            tokio::spawn({
                let action = self.clone();
                async move {
                    if let Err(e) = action.execute_command(&body).await {
                        log::error!("Detached hook threw an error: {:?}", e);
                    }
                }
            });

            Ok(())
        } else {
            self.execute_command(&body).await
        }
    }

    fn validate_secret(&self, parts: &Parts, body: &Vec<u8>) -> MultihookResult<()> {
        if let Some(secret) = &self.secret {
            let validator = secret.format.validator();
            if !validator.validate(&parts.headers, &body, &secret.value.as_bytes()) {
                return Err(MultihookError::InvalidSecret);
            }
        }
        Ok(())
    }

    async fn execute_command(&self, body: &str) -> MultihookResult<()> {
        let json_body: Value = serde_json::from_str(body).unwrap_or_default();
        let command = self.command.evaluate(&json_body);
        log::debug!("Acquiring lock for parallel runs...");
        let permit = self.parallel_lock.acquire().await.unwrap();
        log::debug!("Lock acquired. Running command...");

        let output = Command::new("sh")
            .env("HOOK_BODY", body)
            .arg("-c")
            .arg(command)
            .kill_on_drop(true)
            .output()
            .await?;
        log::debug!("Command finished. Releasing parallel lock...");
        mem::drop(permit);
        let stderr = String::from_utf8_lossy(&output.stderr[..]);
        let stdout = String::from_utf8_lossy(&output.stdout[..]);
        log::debug!("Command output is: {}", stdout);

        if stderr.len() > 0 {
            log::error!("Errors occurred during command execution: {}", stderr);
        }

        Ok(())
    }
}

impl From<&EndpointSettings> for HookAction {
    fn from(endpoint: &EndpointSettings) -> Self {
        let action = endpoint.action.clone();
        let path = PathBuf::from(&action);
        let contents = read_to_string(path).unwrap_or(action);
        Self::new(
            contents,
            endpoint.allow_parallel,
            endpoint.run_detached,
            endpoint.secret.clone(),
        )
    }
}
