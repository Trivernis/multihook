mod hash_mac;

use crate::secret_validation::hash_mac::HMacSecretValidator;
use hyper::HeaderMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SecretFormat {
    HMac,
}

impl SecretFormat {
    pub fn validator(&self) -> impl SecretValidator {
        match self {
            SecretFormat::HMac => HMacSecretValidator,
        }
    }
}

pub trait SecretValidator {
    fn validate(&self, headers: &HeaderMap, body: &[u8], secret: &[u8]) -> bool;
}
