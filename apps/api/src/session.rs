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

        let result = sqlx::query(
            "INSERT INTO sessions
             (id, identity_id, secret_hash, space_id, membership_id, expires_at)
             VALUES ($1, $2, $3, $4, $5, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(second_identity)
        .bind(vec![10_u8; 32])
        .bind(space)
        .bind(membership)
        .execute(&pool)
        .await;

        assert!(result.is_err(), "session identity must own its scoped membership");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn removed_membership_invalidates_scoped_session(pool: PgPool) {
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

        let hash = vec![11_u8; 32];
        sqlx::query(
            "INSERT INTO sessions
             (id, identity_id, secret_hash, space_id, membership_id, expires_at)
             VALUES ($1, $2, $3, $4, $5, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(identity)
        .bind(hash.clone())
        .bind(space)
        .bind(membership)
        .execute(&pool)
        .await
        .unwrap();

        assert!(resolve_session(&pool, &hash).await.is_ok());

        sqlx::query(
            "UPDATE memberships SET state = 'removed', ended_at = now() WHERE id = $1",
        )
        .bind(membership)
        .execute(&pool)
        .await
        .unwrap();

        assert_eq!(
            resolve_session(&pool, &hash).await,
            Err(SessionResolutionError::InvalidCredential)
        );
    }

}
