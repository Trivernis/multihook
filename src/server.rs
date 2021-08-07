use crate::action::HookAction;
use crate::error::MultihookError;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::Response;
use warp::hyper::body::Bytes;
use warp::{Filter, Rejection};

pub struct HookServer {
    endpoints: HashMap<String, HookAction>,
}

impl HookServer {
    pub fn new() -> Self {
        Self {
            endpoints: Default::default(),
        }
    }

    pub fn add_hook(&mut self, point: String, action: HookAction) {
        self.endpoints.insert(point, action);
    }

    async fn execute_action(
        body: Bytes,
        point: String,
        action: Arc<HookAction>,
    ) -> Result<Response<String>, Rejection> {
        let body = String::from_utf8(body.to_vec()).map_err(MultihookError::from)?;
        action.execute(&body).await?;
        log::info!("Hook '{}' executed", point);
        Ok(Response::builder()
            .body(format!("Hook '{}' executed", point))
            .unwrap())
    }

    async fn not_found_response() -> Result<Response<String>, Rejection> {
        log::info!("Endpoint not found");
        Ok(Response::builder()
            .status(404)
            .body(String::from("Endpoint not found"))
            .unwrap())
    }

    pub async fn start(self, address: &str) {
        let routes = self
            .endpoints
            .into_iter()
            .map(|(p, a)| (p, Arc::new(a)))
            .map(|(point, action)| {
                warp::post()
                    .and(warp::path(point.clone()))
                    .and(warp::body::bytes())
                    .and_then(move |b| {
                        let action = Arc::clone(&action);
                        let point = point.clone();
                        async move { Self::execute_action(b, point, action).await }
                    })
                    .boxed()
            })
            .fold(
                warp::get()
                    .and_then(|| async { Self::not_found_response().await })
                    .boxed(),
                |routes, route| routes.or(route).unify().boxed(),
            );

        log::info!("Starting server on {}", address);
        warp::serve(routes)
            .bind(
                address
                    .parse::<SocketAddr>()
                    .expect("Invalid address in settings"),
            )
            .await;
    }
}
