use crate::utils::error::MultihookResult;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::net::{SocketAddr, ToSocketAddrs};
use std::pin::Pin;
use std::sync::Arc;

pub struct HTTPCallback<T1, T2> {
    inner: Arc<
        dyn Fn(
                Request<T1>,
            )
                -> Pin<Box<dyn Future<Output = MultihookResult<Response<T2>>> + Send + Sync>>
            + Send
            + Sync,
    >,
}

impl HTTPCallback<Body, Body> {
    pub fn new<F>(cb: F) -> Self
    where
        F: 'static
            + Fn(
                Request<Body>,
            )
                -> Pin<Box<dyn Future<Output = MultihookResult<Response<Body>>> + Send + Sync>>
            + Send
            + Sync,
    {
        Self {
            inner: Arc::new(cb),
        }
    }

    pub async fn execute(&self, req: Request<Body>) -> MultihookResult<Response<Body>> {
        self.inner.as_ref()(req).await
    }
}

#[derive(Default)]
pub struct HTTPServer {
    routes: HashMap<String, Arc<HTTPCallback<Body, Body>>>,
}

impl HTTPServer {
    pub fn add_callback<S: ToString>(&mut self, route: S, cb: HTTPCallback<Body, Body>) {
        self.routes.insert(route.to_string(), Arc::new(cb));
    }

    async fn execute_callback(&self, req: Request<Body>) -> Result<Response<Body>, Infallible> {
        let path = req.uri().path();
        let response = if let Some(cb) = self.routes.get(path) {
            match cb.as_ref().execute(req).await {
                Ok(res) => res,
                Err(e) => Response::builder()
                    .status(500)
                    .body(Body::from(format!("{:?}", e)))
                    .unwrap(),
            }
        } else {
            Response::builder()
                .status(404)
                .body(Body::from("Unknown endpoint"))
                .unwrap()
        };

        Ok(response)
    }

    pub async fn start<A: ToSocketAddrs>(self, addr: A) -> MultihookResult<()> {
        let address: SocketAddr = addr
            .to_socket_addrs()
            .expect("Failed to convert address to socket address.")
            .next()
            .expect("No socket address specified");
        let self_ref = Arc::new(self);
        let service_fn = make_service_fn(|_conn| {
            let self_ref = Arc::clone(&self_ref);
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let self_ref = Arc::clone(&self_ref);
                    async move { self_ref.execute_callback(req).await }
                }))
            }
        });
        let server = Server::bind(&address).serve(service_fn);
        server.await?;

        Ok(())
    }
}
