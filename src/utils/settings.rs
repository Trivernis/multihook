use crate::secret_validation::SecretFormat;
use crate::utils::error::MultihookResult;
use config::{Config, File};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub server: ServerSettings,
    pub hooks: Option<Hooks>,
    pub endpoints: HashMap<String, EndpointSettings>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerSettings {
    pub address: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Hooks {
    pub pre_action: Option<String>,
    pub post_action: Option<String>,
    pub err_action: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EndpointSettings {
    pub path: String,
    pub action: String,
    pub hooks: Option<Hooks>,
    #[serde(default)]
    pub allow_parallel: bool,
    #[serde(default)]
    pub run_detached: bool,
    pub secret: Option<SecretSettings>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SecretSettings {
    pub value: String,
    pub format: SecretFormat,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            endpoints: HashMap::new(),
            server: ServerSettings { address: None },
            hooks: None,
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
        write_toml_pretty(
            &config_dir.clone().join("config.toml"),
            &Settings::default(),
        )?;
    }

    let settings = Config::builder()
        .add_source(
            glob::glob(&format!("{}/*.toml", config_dir.to_string_lossy()))
                .unwrap()
                .map(|path| File::from(path.unwrap()))
                .collect::<Vec<_>>(),
        )
        .add_source(config::Environment::with_prefix("MULTIHOOK"))
        .add_source(File::from(PathBuf::from(".multihook.toml")))
        .build()?;

    let settings: Settings = settings.try_deserialize()?;

    Ok(settings)
}

fn write_toml_pretty<T: Serialize>(path: &PathBuf, value: &T) -> MultihookResult<()> {
    fs::write(path, toml::to_string_pretty(value)?)?;

    Ok(())
}
