use rand::Rng;
use sha2::{Digest, Sha256};

pub struct ApiKeyManager;

impl ApiKeyManager {
    pub fn generate() -> (String, String, String) {
        // Generates ff_live_<32 hex chars>
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = rng.gen();
        let random_hex = hex::encode(random_bytes);
        let raw_key = format!("ff_live_{}", random_hex);
        let prefix = "ff_live_".to_string();

        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        (raw_key, prefix, hash)
    }

    pub fn verify(raw_key: &str, expected_hash: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(raw_key.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        hash == expected_hash
    }
}
