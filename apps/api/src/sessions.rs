use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub const SESSION_SECRET_BYTES: usize = 32;

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
    fn verifier_accepts_only_the_original_secret() {
        let hash = hash_secret(b"correct-secret");
        assert!(secret_matches(b"correct-secret", &hash));
        assert!(!secret_matches(b"wrong-secret", &hash));
    }
}
