use crate::utils::error::MultihookResult;
use config::File;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub server: ServerSettings,
    pub endpoints: HashMap<String, EndpointSettings>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerSettings {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EndpointSettings {
    pub path: String,
    pub action: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            endpoints: HashMap::new(),
            server: ServerSettings {
                address: String::from("127.0.0.1:8080"),
            },
        }
    }
}

pub fn get_settings() -> &'static Settings {
    lazy_static! {
        static ref SETTINGS: Settings = load_settings().expect("Failed to get settings");
    }

    &*SETTINGS
}

fn load_settings() -> MultihookResult<Settings> {
    let config_dir = dirs::config_dir()
        .map(|c| c.join("multihook"))
        .unwrap_or(PathBuf::from(".config"));
    if !Path::new(&config_dir).exists() {
        fs::create_dir(&config_dir)?;
    }
    write_toml_pretty(
        &config_dir.clone().join("default-config.toml"),
        &Settings::default(),
    )?;

    let mut settings = config::Config::default();
    settings
        .merge(
            glob::glob(&format!("{}/*.toml", config_dir.to_string_lossy()))
                .unwrap()
                .map(|path| File::from(path.unwrap()))
                .collect::<Vec<_>>(),
        )?
        .merge(config::Environment::with_prefix("MULTIHOOK"))?;

    let settings: Settings = settings.try_into()?;

    Ok(settings)
}

fn write_toml_pretty<T: Serialize>(path: &PathBuf, value: &T) -> MultihookResult<()> {
    let mut buf_str = String::new();
    let mut serializer = toml::Serializer::pretty(&mut buf_str);
    serializer.pretty_array(true);
    value.serialize(&mut serializer)?;
    fs::write(path, buf_str.as_bytes())?;

    Ok(())
}
