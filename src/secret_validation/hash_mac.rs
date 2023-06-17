use crate::secret_validation::SecretValidator;
use hmac::{Hmac, Mac};
use hyper::HeaderMap;
use sha2::Sha256;

pub struct HMacSecretValidator;

static SUM_HEADERS: &[&str] = &[
    "X-Forgejo-Signature",
    "X-Gitea-Signature",
    "X-Gogs-Signature",
    "X-Hub-Signature-256",
];

impl SecretValidator for HMacSecretValidator {
    fn validate(&self, headers: &HeaderMap, body: &[u8], secret: &[u8]) -> bool {
        log::debug!("Validating HMac Secret");
        let header = headers
            .iter()
            .filter(|(name, _)| SUM_HEADERS.iter().find(|h| **name == **h).is_some())
            .next();

        if let Some((_, sum)) = header {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
            mac.update(body);
            let Ok(sum) = sum.to_str() else {
                log::error!("Received signature is not a valid string");
                return false;
            };

            let Ok(decoded_secret) = hex::decode(sum.trim_start_matches("sha256=")) else {
                log::error!("Received signature cannot be decoded from hex");
                return false;
            };
            log::debug!("Verifying found signature");

            mac.verify_slice(&decoded_secret).is_ok()
        } else {
            log::error!("Missing Signature Header");
            false
        }
    }
}
