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
            Self::Owner => true,
            Self::Member => matches!(
                capability,
                Capability::View
                    | Capability::Upload
                    | Capability::Download
                    | Capability::DownloadOriginal
                    | Capability::DeleteOwn
            ),
            Self::Guest => matches!(capability, Capability::View | Capability::Upload),
        }
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

    async fn create_space(pool: &PgPool, id: Uuid, creator: Uuid) {
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

        assert!(
            result.is_err(),
            "cross-Space invitation creator must be rejected"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn only_one_active_owner_is_allowed_per_space(pool: PgPool) {
        let space = Uuid::new_v4();
        create_space(&pool, space, Uuid::new_v4()).await;
        create_membership(&pool, Uuid::new_v4(), space, Uuid::new_v4()).await;

        let duplicate = sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'owner')",
        )
        .bind(Uuid::new_v4())
        .bind(space)
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await;

        assert!(duplicate.is_err(), "a Space cannot have two active owners");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn archived_space_requires_archive_timestamp(pool: PgPool) {
        let result = sqlx::query(
            "INSERT INTO spaces (id, name, state, created_by_identity_id)
             VALUES ($1, 'Invalid Archive', 'archived', $2)",
        )
        .bind(Uuid::new_v4())
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await;

        assert!(
            result.is_err(),
            "archived state without timestamp must be rejected"
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
        assert!(weak.is_err(), "short invitation verifier must be rejected");

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
                assert!(
                    result.is_err(),
                    "reused invitation verifier must be rejected"
                );
            }
        }
    }
}
