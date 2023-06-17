use std::string::FromUtf8Error;
use thiserror::Error;

pub type MultihookResult<T> = Result<T, MultihookError>;

#[derive(Error, Debug)]
pub enum MultihookError {
    #[error("Failed to parse body as utf8 string {0}")]
    UTF8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error(transparent)]
    TomlDeserializeError(#[from] toml::de::Error),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),

    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    #[error("Secret validation failed.")]
    InvalidSecret,

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error("Action failed: {0}")]
    ActionError(String),
}

pub trait LogErr {
    fn log_err<S: AsRef<str>>(&self, template: S);
}

impl<T> LogErr for MultihookResult<T> {
    fn log_err<S: AsRef<str>>(&self, message: S) {
        if let Err(e) = self.as_ref() {
            log::error!("{}: {}", message.as_ref(), e);
        }
    }
}
