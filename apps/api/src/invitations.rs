use rand::RngCore;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

pub const INVITATION_SECRET_BYTES: usize = 32;

pub fn generate_secret() -> [u8; INVITATION_SECRET_BYTES] {
    let mut secret = [0_u8; INVITATION_SECRET_BYTES];
    rand::rng().fill_bytes(&mut secret);
    secret
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
    fn generated_invitation_secrets_are_not_reused() {
        assert_ne!(generate_secret(), generate_secret());
    }

    #[test]
    fn verifier_rejects_a_different_secret() {
        let secret = generate_secret();
        let hash = hash_secret(&secret);
        assert!(secret_matches(&secret, &hash));

        let mut different = secret;
        different[0] ^= 1;
        assert!(!secret_matches(&different, &hash));
    }
}
