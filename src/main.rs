use crate::logging::init_logger;
use crate::server::HookServer;
use crate::settings::get_settings;

mod action;
mod error;
mod logging;
mod server;
mod settings;

#[tokio::main]
async fn main() {
    init_logger();
    let settings = get_settings();
    let mut server = HookServer::new();
    for (name, endpoint) in &settings.endpoints {
        log::info!("Adding endpoint {} with path {}", name, &endpoint.path);
        server.add_hook(endpoint.path.clone(), endpoint.action.clone().into())
    }

    server.start(&settings.server.address).await
}
