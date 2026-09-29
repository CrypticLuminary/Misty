use sqlx::PgConnection;
use uuid::Uuid;

use crate::{
    domain::{Capability, RolePreset},
    session::AuthenticatedSession,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthorizationGrant {
    membership_id: Uuid,
    space_id: Uuid,
}

impl AuthorizationGrant {
    pub const fn membership_id(self) -> Uuid {
        self.membership_id
    }

    pub const fn space_id(self) -> Uuid {
        self.space_id
    }
}

#[derive(Debug)]
pub enum AuthorizationError {
    Denied,
    InvariantViolation,
    Storage(sqlx::Error),
}

pub async fn authorize_space_capability(
    connection: &mut PgConnection,
    session: &AuthenticatedSession,
    space_id: Uuid,
    capability: Capability,
) -> Result<AuthorizationGrant, AuthorizationError> {
    let scoped_membership_id = match (session.space_id(), session.membership_id()) {
        (None, None) => None,
        (Some(scoped_space_id), Some(membership_id)) if scoped_space_id == space_id => {
            Some(membership_id)
        }
        (Some(_), Some(_)) | (Some(_), None) | (None, Some(_)) => {
            return Err(AuthorizationError::Denied);
        }
    };

    let row = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, role::text
         FROM memberships
         WHERE space_id = $1
           AND identity_id = $2
           AND state = 'active'
           AND ($3::uuid IS NULL OR id = $3)
         FOR SHARE",
    )
    .bind(space_id)
    .bind(session.identity_id())
    .bind(scoped_membership_id)
    .fetch_optional(connection)
    .await
    .map_err(AuthorizationError::Storage)?;

    let (membership_id, role) = row.ok_or(AuthorizationError::Denied)?;
    let role = role_preset_from_storage(&role).ok_or(AuthorizationError::InvariantViolation)?;

    if !role.allows(capability) {
        return Err(AuthorizationError::Denied);
    }

    Ok(AuthorizationGrant {
        membership_id,
        space_id,
    })
}

pub async fn list_space_capability_grants(
    connection: &mut PgConnection,
    session: &AuthenticatedSession,
    capability: Capability,
) -> Result<Vec<AuthorizationGrant>, AuthorizationError> {
    let rows = match (session.space_id(), session.membership_id()) {
        (None, None) => sqlx::query_as::<_, (Uuid, Uuid, String)>(
            "SELECT id, space_id, role::text
                 FROM memberships
                 WHERE identity_id = $1
                   AND state = 'active'
                 FOR SHARE",
        )
        .bind(session.identity_id())
        .fetch_all(&mut *connection)
        .await
        .map_err(AuthorizationError::Storage)?,
        (Some(space_id), Some(membership_id)) => sqlx::query_as::<_, (Uuid, Uuid, String)>(
            "SELECT id, space_id, role::text
                 FROM memberships
                 WHERE id = $1
                   AND space_id = $2
                   AND identity_id = $3
                   AND state = 'active'
                 FOR SHARE",
        )
        .bind(membership_id)
        .bind(space_id)
        .bind(session.identity_id())
        .fetch_all(&mut *connection)
        .await
        .map_err(AuthorizationError::Storage)?,
        (Some(_), None) | (None, Some(_)) => return Err(AuthorizationError::Denied),
    };

    let mut grants = Vec::with_capacity(rows.len());
    for (membership_id, space_id, role) in rows {
        let role = role_preset_from_storage(&role).ok_or(AuthorizationError::InvariantViolation)?;
        if role.allows(capability) {
            grants.push(AuthorizationGrant {
                membership_id,
                space_id,
            });
        }
    }

    Ok(grants)
}

fn role_preset_from_storage(role: &str) -> Option<RolePreset> {
    match role {
        "owner" => Some(RolePreset::Owner),
        "member" => Some(RolePreset::Member),
        "guest" => Some(RolePreset::Guest),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use sqlx::PgPool;

    use crate::session::{SessionScope, issue_session, resolve_session};

    async fn authorize(
        pool: &PgPool,
        session: &AuthenticatedSession,
        space_id: Uuid,
        capability: Capability,
    ) -> Result<AuthorizationGrant, AuthorizationError> {
        let mut connection = pool
            .acquire()
            .await
            .expect("connection should be available");
        authorize_space_capability(&mut connection, session, space_id, capability).await
    }

    async fn create_identity(pool: &PgPool) -> Uuid {
        let identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(identity)
            .execute(pool)
            .await
            .expect("identity should be created");
        identity
    }

    async fn create_space(pool: &PgPool, creator: Uuid) -> Uuid {
        let space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id)
             VALUES ($1, 'Authorization Test', $2)",
        )
        .bind(space)
        .bind(creator)
        .execute(pool)
        .await
        .expect("space should be created");
        space
    }

    async fn create_membership(
        pool: &PgPool,
        space_id: Uuid,
        identity_id: Uuid,
        role: &str,
    ) -> Uuid {
        let membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, CAST($4 AS membership_role))",
        )
        .bind(membership)
        .bind(space_id)
        .bind(identity_id)
        .bind(role)
        .execute(pool)
        .await
        .expect("membership should be created");
        membership
    }

    async fn identity_session(pool: &PgPool, identity: Uuid) -> AuthenticatedSession {
        let issued = issue_session(
            pool,
            identity,
            SessionScope::Identity,
            Duration::from_secs(3600),
        )
        .await
        .expect("identity session should issue");
        let (_, secret) = issued.into_parts();
        resolve_session(pool, &secret)
            .await
            .expect("identity session should resolve")
    }

    async fn scoped_session(
        pool: &PgPool,
        identity: Uuid,
        space_id: Uuid,
        membership_id: Uuid,
    ) -> AuthenticatedSession {
        let issued = issue_session(
            pool,
            identity,
            SessionScope::SpaceMembership {
                space_id,
                membership_id,
            },
            Duration::from_secs(3600),
        )
        .await
        .expect("scoped session should issue");
        let (_, secret) = issued.into_parts();
        resolve_session(pool, &secret)
            .await
            .expect("scoped session should resolve")
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn owner_membership_allows_management_capability(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let space = create_space(&pool, identity).await;
        let membership = create_membership(&pool, space, identity, "owner").await;
        let session = identity_session(&pool, identity).await;

        let grant = authorize(&pool, &session, space, Capability::ManageSpace)
            .await
            .expect("owner should be authorized");

        assert_eq!(grant.membership_id(), membership);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn role_capability_mapping_denies_without_requested_capability(pool: PgPool) {
        let creator = create_identity(&pool).await;
        let guest = create_identity(&pool).await;
        let space = create_space(&pool, creator).await;
        let membership = create_membership(&pool, space, guest, "guest").await;
        let session = scoped_session(&pool, guest, space, membership).await;

        assert!(
            authorize(&pool, &session, space, Capability::View)
                .await
                .is_ok()
        );
        assert!(matches!(
            authorize(&pool, &session, space, Capability::ManageSpace).await,
            Err(AuthorizationError::Denied)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_session_cannot_use_another_space_membership(pool: PgPool) {
        let first_creator = create_identity(&pool).await;
        let second_creator = create_identity(&pool).await;
        let identity = create_identity(&pool).await;
        let first_space = create_space(&pool, first_creator).await;
        let second_space = create_space(&pool, second_creator).await;
        let first_membership = create_membership(&pool, first_space, identity, "guest").await;
        create_membership(&pool, second_space, identity, "member").await;

        let session = scoped_session(&pool, identity, first_space, first_membership).await;

        assert!(
            authorize(&pool, &session, first_space, Capability::View)
                .await
                .is_ok()
        );
        assert!(matches!(
            authorize(&pool, &session, second_space, Capability::View).await,
            Err(AuthorizationError::Denied)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_list_does_not_expand_to_other_memberships(pool: PgPool) {
        let first_creator = create_identity(&pool).await;
        let second_creator = create_identity(&pool).await;
        let identity = create_identity(&pool).await;
        let first_space = create_space(&pool, first_creator).await;
        let second_space = create_space(&pool, second_creator).await;
        let first_membership = create_membership(&pool, first_space, identity, "guest").await;
        create_membership(&pool, second_space, identity, "member").await;
        let session = scoped_session(&pool, identity, first_space, first_membership).await;

        let mut connection = pool.acquire().await.unwrap();
        let grants = list_space_capability_grants(&mut connection, &session, Capability::View)
            .await
            .expect("scoped list should authorize its own membership");

        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0].space_id(), first_space);
        assert_eq!(grants[0].membership_id(), first_membership);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn inactive_membership_is_denied(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let space = create_space(&pool, identity).await;
        let membership = create_membership(&pool, space, identity, "owner").await;
        let session = identity_session(&pool, identity).await;

        sqlx::query(
            "UPDATE memberships
             SET state = 'removed', ended_at = now()
             WHERE id = $1",
        )
        .bind(membership)
        .execute(&pool)
        .await
        .unwrap();

        assert!(matches!(
            authorize(&pool, &session, space, Capability::View).await,
            Err(AuthorizationError::Denied)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn creator_provenance_without_membership_grants_nothing(pool: PgPool) {
        let creator = create_identity(&pool).await;
        let space = create_space(&pool, creator).await;
        let session = identity_session(&pool, creator).await;

        assert!(matches!(
            authorize(&pool, &session, space, Capability::View).await,
            Err(AuthorizationError::Denied)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn transaction_authorization_locks_membership_against_role_change(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let space = create_space(&pool, identity).await;
        let membership = create_membership(&pool, space, identity, "owner").await;
        let session = identity_session(&pool, identity).await;

        let mut authorizing_tx = pool.begin().await.unwrap();
        authorize_space_capability(
            &mut authorizing_tx,
            &session,
            space,
            Capability::ManageSpace,
        )
        .await
        .expect("authorization should acquire membership lock");

        let mut competing_tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL lock_timeout = '100ms'")
            .execute(&mut *competing_tx)
            .await
            .unwrap();

        let update = sqlx::query(
            "UPDATE memberships
             SET role = 'guest'
             WHERE id = $1",
        )
        .bind(membership)
        .execute(&mut *competing_tx)
        .await;

        assert!(
            update.is_err(),
            "membership role must not change while a transaction relies on its authorization"
        );

        authorizing_tx.rollback().await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn identity_session_uses_role_of_each_space_membership(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let other_creator = create_identity(&pool).await;
        let owned_space = create_space(&pool, identity).await;
        let guest_space = create_space(&pool, other_creator).await;
        create_membership(&pool, owned_space, identity, "owner").await;
        create_membership(&pool, guest_space, identity, "guest").await;
        let session = identity_session(&pool, identity).await;

        assert!(
            authorize(&pool, &session, owned_space, Capability::ManageSpace)
                .await
                .is_ok()
        );

        assert!(matches!(
            authorize(&pool, &session, guest_space, Capability::ManageSpace).await,
            Err(AuthorizationError::Denied)
        ));
    }
}
