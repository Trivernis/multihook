use std::path::{Path, PathBuf};

use utils::logging::init_logger;
use utils::settings::get_settings;

use crate::server::endpoint::HookEndpoint;
use crate::server::HookServer;

mod secret_validation;
mod server;
pub(crate) mod utils;

#[cfg(not(feature = "singlethreaded"))]
#[tokio::main]
async fn main() {
    init_and_start().await
}

#[cfg(feature = "singlethreaded")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_and_start().await
}

async fn init_and_start() {
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
        log::info!("Adding endpoint '{}' with path '{}'", name, &endpoint.path);
        server.add_hook(
            endpoint.path.clone(),
            HookEndpoint::from_config(name, &settings, &endpoint),
        )
    }

    let address = settings
        .server
        .address
        .clone()
        .unwrap_or(String::from("127.0.0.1:8080"));
    server
        .start(&address)
        .await
        .expect("Failed to start server");
}
