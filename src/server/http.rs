use crate::utils::error::MultihookResult;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Future;
use std::net::{SocketAddr, ToSocketAddrs};
use std::pin::Pin;
use std::sync::Arc;

pub struct HTTPCallback<T1, T2> {
    methods: Vec<Method>,
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
            methods: Vec::new(),
        }
    }

    pub fn allow_method(mut self, method: Method) -> Self {
        self.methods.push(method);

        self
    }

    pub fn validate_method(&self, method: &Method) -> bool {
        self.methods.contains(method)
    }

    pub async fn execute(&self, req: Request<Body>) -> MultihookResult<Response<Body>> {
        if !self.validate_method(req.method()) {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("Method not allowed"))
                .unwrap());
        }
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
        let path = &req.uri().path()[1..];
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
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("404 - Not Found"))
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
