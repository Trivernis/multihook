use std::sync::Arc;

use hyper::{Body, Method, Response};

use endpoint::HookEndpoint;

use crate::server::http::{HTTPCallback, HTTPServer};
use crate::utils::error::MultihookResult;

pub mod action;
pub mod endpoint;
mod http;

pub struct HookServer {
    server: HTTPServer,
}

impl HookServer {
    pub fn new() -> Self {
        Self {
            server: HTTPServer::default(),
        }
    }

    pub fn add_hook(&mut self, point: String, action: HookEndpoint) {
        let action = Arc::new(action);

        let cb = HTTPCallback::new({
            let point = point.clone();
            move |req| {
                let action = Arc::clone(&action);
                let point = point.clone();
                Box::pin(async move {
                    log::debug!("Executing hook {}", point);
                    action.execute(req).await?;
                    log::debug!("Hook {} executed", point);

                    Ok(Response::new(Body::from(format!(
                        "Hook '{}' executed.",
                        point
                    ))))
                })
            }
        })
        .allow_method(Method::POST);
        self.server.add_callback(point, cb);
    }

    pub async fn start(self, address: &str) -> MultihookResult<()> {
        log::info!("Starting server on {}", address);
        self.server.start(address).await
    }
}
