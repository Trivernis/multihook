use std::collections::HashMap;

use crate::secret_validation::SecretValidator;
use crate::utils::error::{LogErr, MultihookError, MultihookResult};
use crate::utils::settings::{EndpointSettings, SecretSettings, Settings};
use hyper::http::request::Parts;
use hyper::{Body, Request};
use serde_json::Value;

use super::action::Action;

#[derive(Clone)]
pub struct HookEndpoint {
    name: String,
    action: Action,
    global_hooks: ActionHooks,
    hooks: ActionHooks,
    run_detached: bool,
    secret: Option<SecretSettings>,
}

#[derive(Clone, Default)]
struct ActionHooks {
    pre: Option<Action>,
    post: Option<Action>,
    error: Option<Action>,
}

impl HookEndpoint {
    pub fn from_config<S: Into<String>>(
        name: S,
        global: &Settings,
        endpoint: &EndpointSettings,
    ) -> Self {
        let global_hooks = global
            .hooks
            .as_ref()
            .map(|hooks_cfg| ActionHooks {
                pre: hooks_cfg.pre_action.as_ref().map(|a| Action::new(a, true)),
                post: hooks_cfg.post_action.as_ref().map(|a| Action::new(a, true)),
                error: hooks_cfg.err_action.as_ref().map(|a| Action::new(a, true)),
            })
            .unwrap_or_default();

        let hooks = endpoint
            .hooks
            .as_ref()
            .map(|hooks_cfg| ActionHooks {
                pre: hooks_cfg
                    .pre_action
                    .as_ref()
                    .map(|a| Action::new(a, endpoint.allow_parallel)),
                post: hooks_cfg
                    .post_action
                    .as_ref()
                    .map(|a| Action::new(a, endpoint.allow_parallel)),
                error: hooks_cfg
                    .err_action
                    .as_ref()
                    .map(|a| Action::new(a, endpoint.allow_parallel)),
            })
            .unwrap_or_default();

        Self {
            name: name.into(),
            action: Action::new(&endpoint.action, endpoint.allow_parallel),
            run_detached: endpoint.run_detached,
            secret: endpoint.secret.clone(),
            global_hooks,
            hooks,
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
        let mut env = HashMap::new();
        env.insert("HOOK_NAME", self.name.to_owned());
        env.insert("HOOK_BODY", body.to_string());

        if let Some(global_pre) = &self.global_hooks.pre {
            global_pre
                .run(&json_body, &env)
                .await
                .log_err("Global Pre-Hook failed {e}");
        }
        if let Some(pre_hook) = &self.hooks.pre {
            pre_hook
                .run(&json_body, &env)
                .await
                .log_err("Endpoint Pre-Hook failed {e}");
        }
        if let Err(e) = self.action.run(&json_body, &env).await {
            env.insert("HOOK_ERROR", format!("{e}"));

            if let Some(global_err_action) = &self.global_hooks.error {
                global_err_action
                    .run(&json_body, &env)
                    .await
                    .log_err("Global Error-Hook failed {e}");
            }
            if let Some(err_hook) = &self.hooks.error {
                err_hook
                    .run(&json_body, &env)
                    .await
                    .log_err("Endpoint Error-Hook failed");
            }

            Err(e)
        } else {
            if let Some(global_post_hook) = &self.global_hooks.post {
                global_post_hook
                    .run(&json_body, &env)
                    .await
                    .log_err("Global Post-Hook failed");
            }
            if let Some(post_hook) = &self.hooks.post {
                post_hook
                    .run(&json_body, &env)
                    .await
                    .log_err("Endpoint Post-Hook failed")
            }

            Ok(())
        }
    }
}
