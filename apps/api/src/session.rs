use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedSession {
    pub session_id: Uuid,
    pub identity_id: Uuid,
    pub space_id: Option<Uuid>,
    pub membership_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionResolutionError {
    InvalidCredential,
}

pub async fn resolve_session(
    pool: &PgPool,
    secret_hash: &[u8],
) -> Result<AuthenticatedSession, SessionResolutionError> {
    let row = sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, Option<Uuid>)>(
        "SELECT id, identity_id, space_id, membership_id
         FROM sessions
         WHERE secret_hash = $1
           AND revoked_at IS NULL
           AND expires_at > now()",
    )
    .bind(secret_hash)
    .fetch_optional(pool)
    .await
    .map_err(|_| SessionResolutionError::InvalidCredential)?;

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
    use super::*;

    async fn create_identity(pool: &PgPool, identity: Uuid) {
        sqlx::query("INSERT INTO identities (id, kind) VALUES ($1, 'owner')")
            .bind(identity)
            .execute(pool)
            .await
            .expect("identity should be created");
    }

    async fn create_session(pool: &PgPool, identity: Uuid, hash: Vec<u8>) -> Uuid {
        let id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO sessions (id, identity_id, secret_hash, expires_at)
             VALUES ($1, $2, $3, now() + interval '1 hour')",
        )
        .bind(id)
        .bind(identity)
        .bind(hash)
        .execute(pool)
        .await
        .expect("session should be created");
        id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn unknown_credential_is_denied(pool: PgPool) {
        let result = resolve_session(&pool, &[9_u8; 32]).await;
        assert_eq!(result, Err(SessionResolutionError::InvalidCredential));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn valid_server_session_resolves_then_revocation_denies(pool: PgPool) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;
        let hash = vec![7_u8; 32];
        let session_id = create_session(&pool, identity, hash.clone()).await;

        let resolved = resolve_session(&pool, &hash)
            .await
            .expect("valid session should resolve");
        assert_eq!(resolved.identity_id, identity);

        assert!(revoke_session(&pool, session_id, identity).await.unwrap());
        assert_eq!(
            resolve_session(&pool, &hash).await,
            Err(SessionResolutionError::InvalidCredential)
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn expired_session_is_denied(pool: PgPool) {
        let identity = Uuid::new_v4();
        create_identity(&pool, identity).await;
        let hash = vec![8_u8; 32];
        let session_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO sessions (id, identity_id, secret_hash, created_at, expires_at)
             VALUES ($1, $2, $3, now() - interval '2 hours', now() - interval '1 hour')",
        )
        .bind(session_id)
        .bind(identity)
        .bind(hash.clone())
        .execute(&pool)
        .await
        .expect("historically valid expired session should be created");

        assert_eq!(
            resolve_session(&pool, &hash).await,
            Err(SessionResolutionError::InvalidCredential)
        );
    }
}
