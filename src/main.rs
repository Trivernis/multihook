use crate::logging::init_logger;
use crate::server::HookServer;
use crate::settings::get_settings;
use std::path::{Path, PathBuf};

mod action;
mod command_template;
mod error;
mod logging;
mod server;
mod settings;

#[tokio::main]
async fn main() {
    init_logger();
    let data_dir = dirs::data_dir()
        .map(|d| d.join("multihook"))
        .unwrap_or(PathBuf::from("."));
    if !Path::new(&data_dir).exists() {
        std::fs::create_dir(data_dir).expect("Failed to create data dir");
    }
    let settings = get_settings();
    let mut server = HookServer::new();
    for (name, endpoint) in &settings.endpoints {
        log::info!("Adding endpoint {} with path {}", name, &endpoint.path);
        server.add_hook(endpoint.path.clone(), endpoint.action.clone().into())
    }

    server.start(&settings.server.address).await
}
