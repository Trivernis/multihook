use crate::secret_validation::SecretValidator;
use hmac::{Hmac, Mac, NewMac};
use hyper::HeaderMap;
use sha2::Sha256;

pub struct GithubSecretValidator;

static X_HUB_SIGNATURE_256_HEADER: &str = "X-Hub-Signature-256";

impl SecretValidator for GithubSecretValidator {
    fn validate(&self, headers: &HeaderMap, body: &[u8], secret: &[u8]) -> bool {
        log::debug!("Validating GitHub Secret");
        if let Some(github_sum) = headers.get(X_HUB_SIGNATURE_256_HEADER) {
            let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
            mac.update(body);

            let decoded_secret = if let Ok(decoded) = hex::decode(github_sum) {
                decoded
            } else {
                return false;
            };
            mac.verify(&decoded_secret).is_ok()
        } else {
            log::debug!("Missing Signature Header");
            false
        }
    }
}
