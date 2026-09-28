use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

const SESSION_SECRET_BYTES: usize = 32;
const SESSION_SECRET_ENCODED_LEN: usize = 43;

pub struct IssuedSession {
    session_id: Uuid,
    secret: SessionSecret,
}

impl IssuedSession {
    pub const fn session_id(&self) -> Uuid {
        self.session_id
    }

    pub fn into_parts(self) -> (Uuid, String) {
        (self.session_id, self.secret.into_string())
    }
}

pub enum SessionScope {
    Identity,
    SpaceMembership {
        space_id: Uuid,
        membership_id: Uuid,
    },
}

struct SessionSecret(String);

impl SessionSecret {
    fn generate() -> Result<Self, rand::Error> {
        let mut bytes = [0_u8; SESSION_SECRET_BYTES];
        OsRng.try_fill_bytes(&mut bytes)?;
        Ok(Self(URL_SAFE_NO_PAD.encode(bytes)))
    }

    fn into_string(self) -> String {
        self.0
    }
}

struct SessionVerifier([u8; 32]);

impl SessionVerifier {
    fn from_raw_secret(raw_secret: &str) -> Option<Self> {
        if raw_secret.len() != SESSION_SECRET_ENCODED_LEN {
            return None;
        }

        let decoded = URL_SAFE_NO_PAD.decode(raw_secret.as_bytes()).ok()?;
        if decoded.len() != SESSION_SECRET_BYTES
            || URL_SAFE_NO_PAD.encode(&decoded) != raw_secret
        {
            return None;
        }

        let digest = Sha256::digest(decoded);
        let mut verifier = [0_u8; 32];
        verifier.copy_from_slice(&digest);
        Some(Self(verifier))
    }

    fn from_secret(secret: &SessionSecret) -> Self {
        Self::from_raw_secret(&secret.0).expect("generated session secret must be valid")
    }

    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSession {
    pub session_id: Uuid,
    pub identity_id: Uuid,
    pub space_id: Option<Uuid>,
    pub membership_id: Option<Uuid>,
}

#[derive(Debug)]
pub enum SessionIssuanceError {
    InvalidTtl,
    InvalidScope,
    Entropy(rand::Error),
    Storage(sqlx::Error),
}

#[derive(Debug)]
pub enum SessionResolutionError {
    InvalidCredential,
    Storage(sqlx::Error),
}

pub async fn issue_session(
    pool: &PgPool,
    identity_id: Uuid,
    scope: SessionScope,
    ttl: Duration,
) -> Result<IssuedSession, SessionIssuanceError> {
    let ttl_seconds =
        i32::try_from(ttl.as_secs()).map_err(|_| SessionIssuanceError::InvalidTtl)?;
    if ttl_seconds <= 0 {
        return Err(SessionIssuanceError::InvalidTtl);
    }

    let secret = SessionSecret::generate().map_err(SessionIssuanceError::Entropy)?;
    let verifier = SessionVerifier::from_secret(&secret);
    let session_id = Uuid::new_v4();

    let (space_id, membership_id) = match scope {
        SessionScope::Identity => (None, None),
        SessionScope::SpaceMembership {
            space_id,
            membership_id,
        } => (Some(space_id), Some(membership_id)),
    };

    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO sessions
         (id, identity_id, secret_hash, space_id, membership_id, expires_at)
         SELECT $1, $2, $3, $4, $5, now() + make_interval(secs => $6)
         WHERE $4::uuid IS NULL
            OR EXISTS (
                SELECT 1
                FROM memberships
                WHERE space_id = $4
                  AND id = $5
                  AND identity_id = $2
                  AND state = 'active'
            )
         RETURNING id",
    )
    .bind(session_id)
    .bind(identity_id)
    .bind(verifier.as_bytes())
    .bind(space_id)
    .bind(membership_id)
    .bind(f64::from(ttl_seconds))
    .fetch_optional(pool)
    .await
    .map_err(SessionIssuanceError::Storage)?;

    if inserted.is_none() {
        return Err(SessionIssuanceError::InvalidScope);
    }

    Ok(IssuedSession { session_id, secret })
}

pub async fn resolve_session(
    pool: &PgPool,
    raw_secret: &str,
) -> Result<AuthenticatedSession, SessionResolutionError> {
    let verifier = SessionVerifier::from_raw_secret(raw_secret)
        .ok_or(SessionResolutionError::InvalidCredential)?;

    let row = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, Option<Uuid>)>(
        "SELECT s.id, s.identity_id, s.space_id, s.membership_id
         FROM sessions s
         LEFT JOIN memberships m
           ON m.space_id = s.space_id
          AND m.id = s.membership_id
          AND m.identity_id = s.identity_id
         WHERE s.secret_hash = $1
           AND s.revoked_at IS NULL
           AND s.expires_at > now()
           AND (
               s.membership_id IS NULL
               OR m.state = 'active'
           )",
    )
    .bind(verifier.as_bytes())
    .fetch_optional(pool)
    .await
    .map_err(SessionResolutionError::Storage)?;

    row.map(
        |(session_id, identity_id, space_id, membership_id)| AuthenticatedSession {
            session_id,
            identity_id,
            space_id,
            membership_id,
        },
    )
    .ok_or(SessionResolutionError::InvalidCredential)
}

pub async fn revoke_session(
    pool: &PgPool,
    session_id: Uuid,
    identity_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE sessions
         SET revoked_at = now()
         WHERE id = $1 AND identity_id = $2 AND revoked_at IS NULL",
    )
    .bind(session_id)
    .bind(identity_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() == 1)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    async fn create_identity(pool: &PgPool, identity: Uuid) {
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(identity)
            .execute(pool)
            .await
            .expect("identity should be created");
    }

    #[test]
    fn generated_credentials_are_unique_url_safe_256_bit_secrets() {
        let mut seen = HashSet::new();

        for _ in 0..64 {
            let secret = SessionSecret::generate().expect("OS entropy should be available");
            let decoded = URL_SAFE_NO_PAD
                .decode(secret.0.as_bytes())
                .expect("generated credential should be base64url");
            assert_eq!(decoded.len(), SESSION_SECRET_BYTES);
            assert_eq!(secret.0.len(), SESSION_SECRET_ENCODED_LEN);
            assert!(seen.insert(secret.0));
        }
    }

    #[test]
    fn verifier_is_deterministic_but_not_the_raw_credential() {
        let secret = SessionSecret::generate().expect("OS entropy should be available");
        let first = SessionVerifier::from_secret(&secret);
        let second = SessionVerifier::from_raw_secret(&secret.0).unwrap();

        assert_eq!(first.as_bytes(), second.as_bytes());
        assert_ne!(first.as_bytes(), secret.0.as_bytes());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn malformed_or_unknown_credential_is_denied(pool: PgPool) {
        assert!(matches!(
            resolve_session(&pool, "not-a-session-secret").await,
            Err(SessionResolutionError::InvalidCredential)
        ));

        let unknown = SessionSecret::generate().unwrap().into_string();
        assert!(matches!(
            resolve_session(&pool, &unknown).await,
            Err(SessionResolutionError::InvalidCredential)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn issued_session_stores_only_verifier_and_revocation_denies(pool: PgPool) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;

        let issued = issue_session(
            &pool,
            identity,
            SessionScope::Identity,
            Duration::from_secs(3600),
        )
        .await
        .expect("session should issue");
        let (session_id, raw_secret) = issued.into_parts();

        let stored_verifier =
            sqlx::query_scalar::<_, Vec<u8>>("SELECT secret_hash FROM sessions WHERE id = $1")
                .bind(session_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(stored_verifier.len(), 32);
        assert_ne!(stored_verifier, raw_secret.as_bytes());

        let resolved = resolve_session(&pool, &raw_secret)
            .await
            .expect("valid session should resolve");
        assert_eq!(resolved.identity_id, identity);

        assert!(revoke_session(&pool, session_id, identity).await.unwrap());
        assert!(matches!(
            resolve_session(&pool, &raw_secret).await,
            Err(SessionResolutionError::InvalidCredential)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn expired_session_is_denied(pool: PgPool) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;
        let secret = SessionSecret::generate().unwrap();
        let verifier = SessionVerifier::from_secret(&secret);
        let raw_secret = secret.into_string();

        sqlx::query(
            "INSERT INTO sessions (id, identity_id, secret_hash, created_at, expires_at)
             VALUES ($1, $2, $3, now() - interval '2 hours', now() - interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(identity)
        .bind(verifier.as_bytes())
        .execute(&pool)
        .await
        .expect("historically valid expired session should be created");

        assert!(matches!(
            resolve_session(&pool, &raw_secret).await,
            Err(SessionResolutionError::InvalidCredential)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_session_identity_must_match_membership_identity(pool: PgPool) {
        let first_identity = Uuid::new_v4();
        let second_identity = Uuid::new_v4();
        create_identity(&pool, first_identity).await;
        create_identity(&pool, second_identity).await;

        let space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id) VALUES ($1, 'Scoped', $2)",
        )
        .bind(space)
        .bind(first_identity)
        .execute(&pool)
        .await
        .unwrap();

        let membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'owner')",
        )
        .bind(membership)
        .bind(space)
        .bind(first_identity)
        .execute(&pool)
        .await
        .unwrap();

        let secret = SessionSecret::generate().unwrap();
        let verifier = SessionVerifier::from_secret(&secret);
        let result = sqlx::query(
            "INSERT INTO sessions
             (id, identity_id, secret_hash, space_id, membership_id, expires_at)
             VALUES ($1, $2, $3, $4, $5, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(second_identity)
        .bind(verifier.as_bytes())
        .bind(space)
        .bind(membership)
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "session identity must own its scoped membership"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_issuance_requires_active_membership_and_removal_invalidates_session(
        pool: PgPool,
    ) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;
        let space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id) VALUES ($1, 'Scoped', $2)",
        )
        .bind(space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'guest')",
        )
        .bind(membership)
        .bind(space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let issued = issue_session(
            &pool,
            identity,
            SessionScope::SpaceMembership {
                space_id: space,
                membership_id: membership,
            },
            Duration::from_secs(3600),
        )
        .await
        .expect("active membership should receive scoped session");
        let (_, raw_secret) = issued.into_parts();

        assert!(resolve_session(&pool, &raw_secret).await.is_ok());

        sqlx::query("UPDATE memberships SET state = 'removed', ended_at = now() WHERE id = $1")
            .bind(membership)
            .execute(&pool)
            .await
            .unwrap();

        assert!(matches!(
            resolve_session(&pool, &raw_secret).await,
            Err(SessionResolutionError::InvalidCredential)
        ));

        assert!(matches!(
            issue_session(
                &pool,
                identity,
                SessionScope::SpaceMembership {
                    space_id: space,
                    membership_id: membership,
                },
                Duration::from_secs(3600),
            )
            .await,
            Err(SessionIssuanceError::InvalidScope)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn zero_ttl_is_rejected_before_storage(pool: PgPool) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;

        assert!(matches!(
            issue_session(
                &pool,
                identity,
                SessionScope::Identity,
                Duration::ZERO,
            )
            .await,
            Err(SessionIssuanceError::InvalidTtl)
        ));
    }
}
