mod github;

use crate::secret_validation::github::GithubSecretValidator;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SecretFormat {
    GitHub,
}

impl SecretFormat {
    pub fn validator(&self) -> impl SecretValidator {
        match self {
            SecretFormat::GitHub => GithubSecretValidator,
        }
    }
}

pub trait SecretValidator {
    fn validate(&self, headers: &HeaderMap, body: &[u8], secret: &[u8]) -> bool;
}
