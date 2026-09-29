use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpaceState {
    Active,
    Archived,
    Deleting,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    InvalidTransition,
}

impl SpaceState {
    pub fn transition_to(self, next: Self) -> Result<Self, DomainError> {
        match (self, next) {
            (Self::Active, Self::Archived)
            | (Self::Archived, Self::Deleting)
            | (Self::Deleting, Self::Deleted) => Ok(next),
            _ => Err(DomainError::InvalidTransition),
        }
    }

    pub const fn accepts_writes(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipState {
    Active,
    Left,
    Removed,
}

impl MembershipState {
    pub fn transition_to(self, next: Self) -> Result<Self, DomainError> {
        match (self, next) {
            (Self::Active, Self::Left) | (Self::Active, Self::Removed) => Ok(next),
            _ => Err(DomainError::InvalidTransition),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitationState {
    Active,
    Revoked,
    Exhausted,
}

impl InvitationState {
    pub fn transition_to(self, next: Self) -> Result<Self, DomainError> {
        match (self, next) {
            (Self::Active, Self::Revoked) | (Self::Active, Self::Exhausted) => Ok(next),
            _ => Err(DomainError::InvalidTransition),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    View,
    Upload,
    Download,
    DownloadOriginal,
    DeleteOwn,
    DeleteAny,
    Invite,
    ManageMembers,
    ManageSpace,
    EnableAi,
    ViewLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolePreset {
    Owner,
    Member,
    Guest,
}

impl RolePreset {
    pub const fn allows(self, capability: Capability) -> bool {
        match self {
            Self::Owner => match capability {
                Capability::View
                | Capability::Upload
                | Capability::Download
                | Capability::DownloadOriginal
                | Capability::DeleteOwn
                | Capability::DeleteAny
                | Capability::Invite
                | Capability::ManageMembers
                | Capability::ManageSpace
                | Capability::EnableAi
                | Capability::ViewLocation => true,
            },
            Self::Member => match capability {
                Capability::View
                | Capability::Upload
                | Capability::Download
                | Capability::DownloadOriginal
                | Capability::DeleteOwn => true,
                Capability::DeleteAny
                | Capability::Invite
                | Capability::ManageMembers
                | Capability::ManageSpace
                | Capability::EnableAi
                | Capability::ViewLocation => false,
            },
            Self::Guest => match capability {
                Capability::View | Capability::Upload => true,
                Capability::Download
                | Capability::DownloadOriginal
                | Capability::DeleteOwn
                | Capability::DeleteAny
                | Capability::Invite
                | Capability::ManageMembers
                | Capability::ManageSpace
                | Capability::EnableAi
                | Capability::ViewLocation => false,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionValidity {
    Valid,
    Expired,
    Revoked,
}

pub const fn session_validity(is_expired: bool, is_revoked: bool) -> SessionValidity {
    if is_revoked {
        SessionValidity::Revoked
    } else if is_expired {
        SessionValidity::Expired
    } else {
        SessionValidity::Valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_lifecycle_only_moves_forward() {
        assert_eq!(
            SpaceState::Active.transition_to(SpaceState::Archived),
            Ok(SpaceState::Archived)
        );
        assert_eq!(
            SpaceState::Archived.transition_to(SpaceState::Deleting),
            Ok(SpaceState::Deleting)
        );
        assert_eq!(
            SpaceState::Deleting.transition_to(SpaceState::Deleted),
            Ok(SpaceState::Deleted)
        );
        assert_eq!(
            SpaceState::Archived.transition_to(SpaceState::Active),
            Err(DomainError::InvalidTransition)
        );
        assert_eq!(
            SpaceState::Active.transition_to(SpaceState::Deleted),
            Err(DomainError::InvalidTransition)
        );
    }

    #[test]
    fn only_active_spaces_accept_writes() {
        assert!(SpaceState::Active.accepts_writes());
        assert!(!SpaceState::Archived.accepts_writes());
        assert!(!SpaceState::Deleting.accepts_writes());
        assert!(!SpaceState::Deleted.accepts_writes());
    }

    #[test]
    fn membership_exit_is_terminal() {
        assert_eq!(
            MembershipState::Active.transition_to(MembershipState::Left),
            Ok(MembershipState::Left)
        );
        assert_eq!(
            MembershipState::Active.transition_to(MembershipState::Removed),
            Ok(MembershipState::Removed)
        );
        assert_eq!(
            MembershipState::Left.transition_to(MembershipState::Active),
            Err(DomainError::InvalidTransition)
        );
    }

    #[test]
    fn session_revocation_takes_precedence_over_expiry() {
        assert_eq!(session_validity(false, false), SessionValidity::Valid);
        assert_eq!(session_validity(true, false), SessionValidity::Expired);
        assert_eq!(session_validity(false, true), SessionValidity::Revoked);
        assert_eq!(session_validity(true, true), SessionValidity::Revoked);
    }

    #[test]
    fn invitation_terminal_states_cannot_be_reopened() {
        assert_eq!(
            InvitationState::Active.transition_to(InvitationState::Revoked),
            Ok(InvitationState::Revoked)
        );
        assert_eq!(
            InvitationState::Active.transition_to(InvitationState::Exhausted),
            Ok(InvitationState::Exhausted)
        );
        assert_eq!(
            InvitationState::Revoked.transition_to(InvitationState::Active),
            Err(DomainError::InvalidTransition)
        );
        assert_eq!(
            InvitationState::Exhausted.transition_to(InvitationState::Active),
            Err(DomainError::InvalidTransition)
        );
    }

    #[test]
    fn role_presets_are_capability_mappings_not_authorization_identity() {
        assert!(RolePreset::Owner.allows(Capability::ManageSpace));
        assert!(RolePreset::Member.allows(Capability::DeleteOwn));
        assert!(!RolePreset::Member.allows(Capability::DeleteAny));
        assert!(RolePreset::Guest.allows(Capability::View));
        assert!(!RolePreset::Guest.allows(Capability::ManageMembers));
    }
}

#[cfg(test)]
mod database_tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    fn assert_pg_error_code<T>(result: Result<T, sqlx::Error>, expected_code: &str, message: &str) {
        let error = match result {
            Ok(_) => panic!("{message}"),
            Err(error) => error,
        };
        let database_error = error
            .as_database_error()
            .expect("expected PostgreSQL database error");
        assert_eq!(
            database_error.code().as_deref(),
            Some(expected_code),
            "{message}: unexpected PostgreSQL error: {database_error}"
        );
    }

    async fn create_space(pool: &PgPool, id: Uuid, creator: Uuid) {
        sqlx::query("INSERT INTO identities (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
            .bind(creator)
            .execute(pool)
            .await
            .expect("creator identity should exist");

        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id) VALUES ($1, 'Test Space', $2)",
        )
        .bind(id)
        .bind(creator)
        .execute(pool)
        .await
        .expect("space should be created");
    }

    async fn create_membership(pool: &PgPool, id: Uuid, space_id: Uuid, identity_id: Uuid) {
        sqlx::query("INSERT INTO identities (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
            .bind(identity_id)
            .execute(pool)
            .await
            .expect("membership identity should exist");

        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role) VALUES ($1, $2, $3, 'owner')",
        )
        .bind(id)
        .bind(space_id)
        .bind(identity_id)
        .execute(pool)
        .await
        .expect("membership should be created");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn identity_role_is_defined_by_membership_not_identity_record(pool: PgPool) {
        let identity = Uuid::new_v4();
        let other_creator = Uuid::new_v4();
        let owned_space = Uuid::new_v4();
        let guest_space = Uuid::new_v4();

        create_space(&pool, owned_space, identity).await;
        create_membership(&pool, Uuid::new_v4(), owned_space, identity).await;
        create_space(&pool, guest_space, other_creator).await;

        let result = sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'guest')",
        )
        .bind(Uuid::new_v4())
        .bind(guest_space)
        .bind(identity)
        .execute(&pool)
        .await;

        assert!(
            result.is_ok(),
            "the same neutral identity may hold different roles in different Spaces"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn invitation_creator_must_belong_to_same_space(pool: PgPool) {
        let identity = Uuid::new_v4();
        let first_space = Uuid::new_v4();
        let second_space = Uuid::new_v4();
        let membership = Uuid::new_v4();

        create_space(&pool, first_space, identity).await;
        create_space(&pool, second_space, Uuid::new_v4()).await;
        create_membership(&pool, membership, first_space, identity).await;

        let result = sqlx::query(
            "INSERT INTO invitations
             (id, space_id, created_by_membership_id, secret_hash, expires_at)
             VALUES ($1, $2, $3, $4, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(second_space)
        .bind(membership)
        .bind(vec![7_u8; 32])
        .execute(&pool)
        .await;

        assert_pg_error_code(
            result,
            "23503",
            "cross-Space invitation creator must be rejected by a foreign key",
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn only_one_active_owner_is_allowed_per_space(pool: PgPool) {
        let space = Uuid::new_v4();
        create_space(&pool, space, Uuid::new_v4()).await;
        create_membership(&pool, Uuid::new_v4(), space, Uuid::new_v4()).await;

        let second_identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(second_identity)
            .execute(&pool)
            .await
            .expect("second owner identity should exist");

        let duplicate = sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'owner')",
        )
        .bind(Uuid::new_v4())
        .bind(space)
        .bind(second_identity)
        .execute(&pool)
        .await;

        assert_pg_error_code(duplicate, "23505", "a Space cannot have two active owners");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn archived_space_requires_archive_timestamp(pool: PgPool) {
        let identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(identity)
            .execute(&pool)
            .await
            .expect("creator identity should exist");

        let result = sqlx::query(
            "INSERT INTO spaces (id, name, state, created_by_identity_id)
             VALUES ($1, 'Invalid Archive', 'archived', $2)",
        )
        .bind(Uuid::new_v4())
        .bind(identity)
        .execute(&pool)
        .await;

        assert_pg_error_code(
            result,
            "23514",
            "archived state without timestamp must be rejected by a check constraint",
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn invitation_hash_must_be_unique_and_nontrivial(pool: PgPool) {
        let identity = Uuid::new_v4();
        let space = Uuid::new_v4();
        let membership = Uuid::new_v4();
        create_space(&pool, space, identity).await;
        create_membership(&pool, membership, space, identity).await;

        let weak = sqlx::query(
            "INSERT INTO invitations
             (id, space_id, created_by_membership_id, secret_hash, expires_at)
             VALUES ($1, $2, $3, $4, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(space)
        .bind(membership)
        .bind(vec![1_u8; 8])
        .execute(&pool)
        .await;
        assert_pg_error_code(
            weak,
            "23514",
            "short invitation verifier must be rejected by a check constraint",
        );

        let hash = vec![9_u8; 32];
        for attempt in 0..2 {
            let result = sqlx::query(
                "INSERT INTO invitations
                 (id, space_id, created_by_membership_id, secret_hash, expires_at)
                 VALUES ($1, $2, $3, $4, now() + interval '1 hour')",
            )
            .bind(Uuid::new_v4())
            .bind(space)
            .bind(membership)
            .bind(hash.clone())
            .execute(&pool)
            .await;

            if attempt == 0 {
                assert!(result.is_ok());
            } else {
                assert_pg_error_code(
                    result,
                    "23505",
                    "reused invitation verifier must be rejected by uniqueness",
                );
            }
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn guest_session_membership_must_belong_to_same_space(pool: PgPool) {
        let first_identity = Uuid::new_v4();
        let second_identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1), ($2)")
            .bind(first_identity)
            .bind(second_identity)
            .execute(&pool)
            .await
            .expect("identities should be created");

        let first_space = Uuid::new_v4();
        let second_space = Uuid::new_v4();
        create_space(&pool, first_space, first_identity).await;
        create_space(&pool, second_space, second_identity).await;

        let membership = Uuid::new_v4();
        create_membership(&pool, membership, first_space, first_identity).await;

        let result = sqlx::query(
            "INSERT INTO sessions
             (id, identity_id, secret_hash, space_id, membership_id, expires_at)
             VALUES ($1, $2, $3, $4, $5, now() + interval '1 hour')",
        )
        .bind(Uuid::new_v4())
        .bind(first_identity)
        .bind(vec![3_u8; 32])
        .bind(second_space)
        .bind(membership)
        .execute(&pool)
        .await;

        assert_pg_error_code(
            result,
            "23503",
            "session membership must be scoped to its Space",
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn session_verifier_is_unique_and_expiry_must_be_future(pool: PgPool) {
        let identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(identity)
            .execute(&pool)
            .await
            .expect("identity should be created");

        let expired = sqlx::query(
            "INSERT INTO sessions (id, identity_id, secret_hash, expires_at)
             VALUES ($1, $2, $3, now() - interval '1 minute')",
        )
        .bind(Uuid::new_v4())
        .bind(identity)
        .bind(vec![4_u8; 32])
        .execute(&pool)
        .await;
        assert_pg_error_code(expired, "23514", "session expiry must be after creation");

        let verifier = vec![5_u8; 32];
        for attempt in 0..2 {
            let result = sqlx::query(
                "INSERT INTO sessions (id, identity_id, secret_hash, expires_at)
                 VALUES ($1, $2, $3, now() + interval '1 hour')",
            )
            .bind(Uuid::new_v4())
            .bind(identity)
            .bind(verifier.clone())
            .execute(&pool)
            .await;

            if attempt == 0 {
                assert!(result.is_ok());
            } else {
                assert_pg_error_code(
                    result,
                    "23505",
                    "session verifier reuse must be rejected by uniqueness",
                );
            }
        }
    }
}
