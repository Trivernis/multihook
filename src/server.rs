use crate::action::HookAction;
use crate::error::{MultihookError, MultihookResult};
use std::collections::HashMap;
use std::sync::Arc;
use warp::hyper::body::Bytes;
use warp::{Filter, Rejection};

pub struct HookServer {
    endpoints: HashMap<String, HookAction>,
}

impl HookServer {
    pub fn add_hook(&mut self, point: String, action: HookAction) {
        self.endpoints.insert(point, action);
    }

    async fn execute_action(
        body: Bytes,
        point: String,
        action: Arc<HookAction>,
    ) -> Result<impl warp::Reply, Rejection> {
        let body = String::from_utf8(body.to_vec()).map_err(MultihookError::from)?;
        action.execute(&body).await?;
        Ok(format!("Hook {} executed", point))
    }

    pub fn start(self) {
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
                        Self::execute_action(b, point.clone(), action)
                    })
                    .map(|_| warp::reply())
                    .boxed()
            })
            .fold(warp::any().map(warp::reply).boxed(), |routes, route| {
                routes.or(route).unify().boxed()
            });

        let routes = warp::serve(routes);
    }
}
