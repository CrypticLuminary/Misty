use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub const SESSION_SECRET_BYTES: usize = 32;

pub fn generate_secret() -> [u8; SESSION_SECRET_BYTES] {
    let mut secret = [0_u8; SESSION_SECRET_BYTES];
    rand::rng().fill_bytes(&mut secret);
    secret
}

pub fn encode_secret(secret: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(secret)
}

pub fn decode_secret(encoded: &str) -> Option<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(encoded).ok()
}

pub fn hash_secret(secret: &[u8]) -> [u8; 32] {
    Sha256::digest(secret).into()
}

pub fn secret_matches(candidate: &[u8], expected_hash: &[u8; 32]) -> bool {
    let candidate_hash = hash_secret(candidate);
    bool::from(candidate_hash.ct_eq(expected_hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_secret_encoding_round_trips() {
        let secret = generate_secret();
        assert_eq!(decode_secret(&encode_secret(&secret)).as_deref(), Some(secret.as_slice()));
    }

    #[test]
    fn verifier_accepts_only_the_original_secret() {
        let hash = hash_secret(b"correct-secret");
        assert!(secret_matches(b"correct-secret", &hash));
        assert!(!secret_matches(b"wrong-secret", &hash));
    }
}
