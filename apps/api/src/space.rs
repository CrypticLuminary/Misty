use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    authorization::{AuthorizationError, authorize_space_capability, list_space_capability_grants},
    domain::{Capability, SpaceState},
    session::AuthenticatedSession,
};

const MAX_SPACE_NAME_CHARS: usize = 120;
const SPACE_CREATED_EVENT: &str = "space.created";
const SPACE_ARCHIVED_EVENT: &str = "space.archived";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedSpace {
    space_id: Uuid,
    owner_membership_id: Uuid,
    name: String,
}

impl CreatedSpace {
    pub const fn space_id(&self) -> Uuid {
        self.space_id
    }

    pub const fn owner_membership_id(&self) -> Uuid {
        self.owner_membership_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub enum CreateSpaceError {
    InvalidName,
    RequiresIdentitySession,
    Storage(sqlx::Error),
}

#[derive(Debug, Clone, Copy)]
struct NewSpaceIds {
    space_id: Uuid,
    owner_membership_id: Uuid,
    audit_event_id: Uuid,
}

impl NewSpaceIds {
    fn generate() -> Self {
        Self {
            space_id: Uuid::new_v4(),
            owner_membership_id: Uuid::new_v4(),
            audit_event_id: Uuid::new_v4(),
        }
    }
}

pub async fn create_space(
    pool: &PgPool,
    session: &AuthenticatedSession,
    raw_name: &str,
    correlation_id: Uuid,
) -> Result<CreatedSpace, CreateSpaceError> {
    create_space_with_ids(
        pool,
        session,
        raw_name,
        correlation_id,
        NewSpaceIds::generate(),
    )
    .await
}

async fn create_space_with_ids(
    pool: &PgPool,
    session: &AuthenticatedSession,
    raw_name: &str,
    correlation_id: Uuid,
    ids: NewSpaceIds,
) -> Result<CreatedSpace, CreateSpaceError> {
    if session.space_id().is_some() || session.membership_id().is_some() {
        return Err(CreateSpaceError::RequiresIdentitySession);
    }

    let name = normalize_space_name(raw_name)?;

    let mut transaction = pool.begin().await.map_err(CreateSpaceError::Storage)?;

    sqlx::query(
        "INSERT INTO spaces (id, name, created_by_identity_id)
         VALUES ($1, $2, $3)",
    )
    .bind(ids.space_id)
    .bind(&name)
    .bind(session.identity_id())
    .execute(&mut *transaction)
    .await
    .map_err(CreateSpaceError::Storage)?;

    sqlx::query(
        "INSERT INTO memberships
         (id, space_id, identity_id, role)
         VALUES ($1, $2, $3, 'owner')",
    )
    .bind(ids.owner_membership_id)
    .bind(ids.space_id)
    .bind(session.identity_id())
    .execute(&mut *transaction)
    .await
    .map_err(CreateSpaceError::Storage)?;

    sqlx::query(
        "INSERT INTO audit_events
         (id, space_id, actor_identity_id, actor_membership_id, event_type, correlation_id)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(ids.audit_event_id)
    .bind(ids.space_id)
    .bind(session.identity_id())
    .bind(ids.owner_membership_id)
    .bind(SPACE_CREATED_EVENT)
    .bind(correlation_id)
    .execute(&mut *transaction)
    .await
    .map_err(CreateSpaceError::Storage)?;

    transaction
        .commit()
        .await
        .map_err(CreateSpaceError::Storage)?;

    Ok(CreatedSpace {
        space_id: ids.space_id,
        owner_membership_id: ids.owner_membership_id,
        name,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpaceSummary {
    space_id: Uuid,
    name: String,
    state: SpaceState,
}

impl SpaceSummary {
    pub const fn space_id(&self) -> Uuid {
        self.space_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn state(&self) -> SpaceState {
        self.state
    }
}

#[derive(Debug)]
pub enum SpaceAccessError {
    Denied,
    InvariantViolation,
    Storage(sqlx::Error),
}

pub async fn read_space(
    pool: &PgPool,
    session: &AuthenticatedSession,
    space_id: Uuid,
) -> Result<SpaceSummary, SpaceAccessError> {
    let mut transaction = pool.begin().await.map_err(SpaceAccessError::Storage)?;

    authorize_space_capability(&mut transaction, session, space_id, Capability::View)
        .await
        .map_err(map_authorization_error)?;

    let row = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, name, state::text
         FROM spaces
         WHERE id = $1
           AND state <> 'deleted'",
    )
    .bind(space_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(SpaceAccessError::Storage)?
    .ok_or(SpaceAccessError::Denied)?;

    transaction
        .commit()
        .await
        .map_err(SpaceAccessError::Storage)?;

    summary_from_row(row)
}

pub async fn list_spaces(
    pool: &PgPool,
    session: &AuthenticatedSession,
) -> Result<Vec<SpaceSummary>, SpaceAccessError> {
    let mut transaction = pool.begin().await.map_err(SpaceAccessError::Storage)?;
    let grants = list_space_capability_grants(&mut transaction, session, Capability::View)
        .await
        .map_err(map_authorization_error)?;

    if grants.is_empty() {
        transaction
            .commit()
            .await
            .map_err(SpaceAccessError::Storage)?;
        return Ok(Vec::new());
    }

    let space_ids: Vec<Uuid> = grants.into_iter().map(|grant| grant.space_id()).collect();
    let rows = sqlx::query_as::<_, (Uuid, String, String)>(
        "SELECT id, name, state::text
         FROM spaces
         WHERE id = ANY($1)
           AND state <> 'deleted'
         ORDER BY name ASC, id ASC",
    )
    .bind(&space_ids)
    .fetch_all(&mut *transaction)
    .await
    .map_err(SpaceAccessError::Storage)?;

    transaction
        .commit()
        .await
        .map_err(SpaceAccessError::Storage)?;

    rows.into_iter().map(summary_from_row).collect()
}

fn summary_from_row(row: (Uuid, String, String)) -> Result<SpaceSummary, SpaceAccessError> {
    let state = space_state_from_storage(&row.2).ok_or(SpaceAccessError::InvariantViolation)?;

    Ok(SpaceSummary {
        space_id: row.0,
        name: row.1,
        state,
    })
}

fn space_state_from_storage(state: &str) -> Option<SpaceState> {
    match state {
        "active" => Some(SpaceState::Active),
        "archived" => Some(SpaceState::Archived),
        "deleting" => Some(SpaceState::Deleting),
        "deleted" => Some(SpaceState::Deleted),
        _ => None,
    }
}

fn map_authorization_error(error: AuthorizationError) -> SpaceAccessError {
    match error {
        AuthorizationError::Denied => SpaceAccessError::Denied,
        AuthorizationError::InvariantViolation => SpaceAccessError::InvariantViolation,
        AuthorizationError::Storage(error) => SpaceAccessError::Storage(error),
    }
}

#[derive(Debug)]
pub enum ArchiveSpaceError {
    Denied,
    NotActive,
    InvariantViolation,
    Storage(sqlx::Error),
}

pub async fn archive_space(
    pool: &PgPool,
    session: &AuthenticatedSession,
    space_id: Uuid,
    correlation_id: Uuid,
) -> Result<SpaceSummary, ArchiveSpaceError> {
    archive_space_with_audit_id(pool, session, space_id, correlation_id, Uuid::new_v4()).await
}

async fn archive_space_with_audit_id(
    pool: &PgPool,
    session: &AuthenticatedSession,
    space_id: Uuid,
    correlation_id: Uuid,
    audit_event_id: Uuid,
) -> Result<SpaceSummary, ArchiveSpaceError> {
    let mut transaction = pool.begin().await.map_err(ArchiveSpaceError::Storage)?;

    let grant =
        authorize_space_capability(&mut transaction, session, space_id, Capability::ManageSpace)
            .await
            .map_err(map_archive_authorization_error)?;

    let current_state = lock_space_state(&mut transaction, space_id)
        .await
        .map_err(map_archive_space_access_error)?;

    current_state
        .transition_to(SpaceState::Archived)
        .map_err(|_| ArchiveSpaceError::NotActive)?;

    let row = sqlx::query_as::<_, (Uuid, String, String)>(
        "UPDATE spaces
         SET state = 'archived',
             archived_at = now()
         WHERE id = $1
           AND state = 'active'
         RETURNING id, name, state::text",
    )
    .bind(space_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(ArchiveSpaceError::Storage)?
    .ok_or(ArchiveSpaceError::NotActive)?;

    sqlx::query(
        "INSERT INTO audit_events
         (id, space_id, actor_identity_id, actor_membership_id, event_type, correlation_id)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(audit_event_id)
    .bind(space_id)
    .bind(session.identity_id())
    .bind(grant.membership_id())
    .bind(SPACE_ARCHIVED_EVENT)
    .bind(correlation_id)
    .execute(&mut *transaction)
    .await
    .map_err(ArchiveSpaceError::Storage)?;

    transaction
        .commit()
        .await
        .map_err(ArchiveSpaceError::Storage)?;

    summary_from_row(row).map_err(|error| match error {
        SpaceAccessError::InvariantViolation => ArchiveSpaceError::InvariantViolation,
        SpaceAccessError::Storage(error) => ArchiveSpaceError::Storage(error),
        SpaceAccessError::Denied => ArchiveSpaceError::InvariantViolation,
    })
}

#[cfg(test)]
pub(crate) async fn lock_active_space_for_write(
    connection: &mut sqlx::PgConnection,
    space_id: Uuid,
) -> Result<(), SpaceAccessError> {
    let state = lock_space_state(connection, space_id).await?;
    if !state.accepts_writes() {
        return Err(SpaceAccessError::Denied);
    }
    Ok(())
}

async fn lock_space_state(
    connection: &mut sqlx::PgConnection,
    space_id: Uuid,
) -> Result<SpaceState, SpaceAccessError> {
    let state = sqlx::query_scalar::<_, String>(
        "SELECT state::text
         FROM spaces
         WHERE id = $1
         FOR UPDATE",
    )
    .bind(space_id)
    .fetch_optional(connection)
    .await
    .map_err(SpaceAccessError::Storage)?
    .ok_or(SpaceAccessError::Denied)?;

    space_state_from_storage(&state).ok_or(SpaceAccessError::InvariantViolation)
}

fn map_archive_authorization_error(error: AuthorizationError) -> ArchiveSpaceError {
    match error {
        AuthorizationError::Denied => ArchiveSpaceError::Denied,
        AuthorizationError::InvariantViolation => ArchiveSpaceError::InvariantViolation,
        AuthorizationError::Storage(error) => ArchiveSpaceError::Storage(error),
    }
}

fn map_archive_space_access_error(error: SpaceAccessError) -> ArchiveSpaceError {
    match error {
        SpaceAccessError::Denied => ArchiveSpaceError::Denied,
        SpaceAccessError::InvariantViolation => ArchiveSpaceError::InvariantViolation,
        SpaceAccessError::Storage(error) => ArchiveSpaceError::Storage(error),
    }
}

fn normalize_space_name(raw_name: &str) -> Result<String, CreateSpaceError> {
    let name = raw_name.trim();

    if name.is_empty()
        || name.chars().count() > MAX_SPACE_NAME_CHARS
        || name.chars().any(char::is_control)
    {
        return Err(CreateSpaceError::InvalidName);
    }

    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::session::{SessionScope, issue_session, resolve_session};

    async fn create_identity(pool: &PgPool) -> Uuid {
        let identity = Uuid::new_v4();
        sqlx::query("INSERT INTO identities (id) VALUES ($1)")
            .bind(identity)
            .execute(pool)
            .await
            .expect("identity should be created");
        identity
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

    #[test]
    fn space_name_is_trimmed_and_control_characters_are_rejected() {
        assert_eq!(
            normalize_space_name("  College Trip  ").unwrap(),
            "College Trip"
        );
        assert!(matches!(
            normalize_space_name("Trip\nHidden"),
            Err(CreateSpaceError::InvalidName)
        ));
        assert!(matches!(
            normalize_space_name("   "),
            Err(CreateSpaceError::InvalidName)
        ));
        assert!(matches!(
            normalize_space_name(&"x".repeat(MAX_SPACE_NAME_CHARS + 1)),
            Err(CreateSpaceError::InvalidName)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn create_space_atomically_creates_owner_and_audit_event(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let correlation_id = Uuid::new_v4();

        let created = create_space(&pool, &session, "  Weekend Hike  ", correlation_id)
            .await
            .expect("Space should be created");

        assert_eq!(created.name(), "Weekend Hike");

        let space = sqlx::query_as::<_, (String, Uuid)>(
            "SELECT name, created_by_identity_id
             FROM spaces
             WHERE id = $1",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(space.0, "Weekend Hike");
        assert_eq!(space.1, identity);

        let membership = sqlx::query_as::<_, (Uuid, Uuid, String, String)>(
            "SELECT space_id, identity_id, role::text, state::text
             FROM memberships
             WHERE id = $1",
        )
        .bind(created.owner_membership_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(membership.0, created.space_id());
        assert_eq!(membership.1, identity);
        assert_eq!(membership.2, "owner");
        assert_eq!(membership.3, "active");

        let audit = sqlx::query_as::<_, (String, Uuid, Uuid, Uuid)>(
            "SELECT event_type, correlation_id, actor_identity_id, actor_membership_id
             FROM audit_events
             WHERE space_id = $1",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audit.0, SPACE_CREATED_EVENT);
        assert_eq!(audit.1, correlation_id);
        assert_eq!(audit.2, identity);
        assert_eq!(audit.3, created.owner_membership_id());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_guest_session_cannot_create_a_new_space(pool: PgPool) {
        let creator = create_identity(&pool).await;
        let guest = create_identity(&pool).await;

        let existing_space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id)
             VALUES ($1, 'Existing', $2)",
        )
        .bind(existing_space)
        .bind(creator)
        .execute(&pool)
        .await
        .unwrap();

        let guest_membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'guest')",
        )
        .bind(guest_membership)
        .bind(existing_space)
        .bind(guest)
        .execute(&pool)
        .await
        .unwrap();

        let issued = issue_session(
            &pool,
            guest,
            SessionScope::SpaceMembership {
                space_id: existing_space,
                membership_id: guest_membership,
            },
            Duration::from_secs(3600),
        )
        .await
        .unwrap();
        let (_, secret) = issued.into_parts();
        let session = resolve_session(&pool, &secret).await.unwrap();

        assert!(matches!(
            create_space(&pool, &session, "Escape", Uuid::new_v4()).await,
            Err(CreateSpaceError::RequiresIdentitySession)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn failed_audit_insert_rolls_back_space_and_owner(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;

        let existing_space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id)
             VALUES ($1, 'Existing Audit', $2)",
        )
        .bind(existing_space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let existing_membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'owner')",
        )
        .bind(existing_membership)
        .bind(existing_space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let existing_audit = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO audit_events
             (id, space_id, actor_identity_id, actor_membership_id, event_type, correlation_id)
             VALUES ($1, $2, $3, $4, 'space.created', $5)",
        )
        .bind(existing_audit)
        .bind(existing_space)
        .bind(identity)
        .bind(existing_membership)
        .bind(Uuid::new_v4())
        .execute(&pool)
        .await
        .unwrap();

        let attempted_space = Uuid::new_v4();
        let attempted_membership = Uuid::new_v4();
        let result = create_space_with_ids(
            &pool,
            &session,
            "Audit Must Roll Back",
            Uuid::new_v4(),
            NewSpaceIds {
                space_id: attempted_space,
                owner_membership_id: attempted_membership,
                audit_event_id: existing_audit,
            },
        )
        .await;

        assert!(matches!(result, Err(CreateSpaceError::Storage(_))));

        let space_count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM spaces WHERE id = $1")
            .bind(attempted_space)
            .fetch_one(&pool)
            .await
            .unwrap();
        let membership_count =
            sqlx::query_scalar::<_, i64>("SELECT count(*) FROM memberships WHERE id = $1")
                .bind(attempted_membership)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(space_count, 0, "audit failure must roll back the Space");
        assert_eq!(
            membership_count, 0,
            "audit failure must roll back the owner membership"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn read_space_requires_view_capability(pool: PgPool) {
        let owner = create_identity(&pool).await;
        let outsider = create_identity(&pool).await;
        let owner_session = identity_session(&pool, owner).await;
        let outsider_session = identity_session(&pool, outsider).await;

        let created = create_space(&pool, &owner_session, "Protected", Uuid::new_v4())
            .await
            .unwrap();

        let summary = read_space(&pool, &owner_session, created.space_id())
            .await
            .expect("owner should read Space");
        assert_eq!(summary.space_id(), created.space_id());
        assert_eq!(summary.name(), "Protected");
        assert_eq!(summary.state(), SpaceState::Active);

        assert!(matches!(
            read_space(&pool, &outsider_session, created.space_id()).await,
            Err(SpaceAccessError::Denied)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn scoped_session_lists_only_its_bound_space(pool: PgPool) {
        let first_owner = create_identity(&pool).await;
        let second_owner = create_identity(&pool).await;
        let guest = create_identity(&pool).await;
        let first_owner_session = identity_session(&pool, first_owner).await;
        let second_owner_session = identity_session(&pool, second_owner).await;

        let first = create_space(&pool, &first_owner_session, "First", Uuid::new_v4())
            .await
            .unwrap();
        let second = create_space(&pool, &second_owner_session, "Second", Uuid::new_v4())
            .await
            .unwrap();

        let first_membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'guest')",
        )
        .bind(first_membership)
        .bind(first.space_id())
        .bind(guest)
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'member')",
        )
        .bind(Uuid::new_v4())
        .bind(second.space_id())
        .bind(guest)
        .execute(&pool)
        .await
        .unwrap();

        let session = scoped_session(&pool, guest, first.space_id(), first_membership).await;
        let spaces = list_spaces(&pool, &session).await.unwrap();

        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].space_id(), first.space_id());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn identity_session_lists_only_active_memberships(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let other_owner = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let other_session = identity_session(&pool, other_owner).await;

        let owned = create_space(&pool, &session, "Owned", Uuid::new_v4())
            .await
            .unwrap();
        let shared = create_space(&pool, &other_session, "Shared", Uuid::new_v4())
            .await
            .unwrap();

        let shared_membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'member')",
        )
        .bind(shared_membership)
        .bind(shared.space_id())
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let spaces = list_spaces(&pool, &session).await.unwrap();
        assert_eq!(spaces.len(), 2);
        assert!(
            spaces
                .iter()
                .any(|space| space.space_id() == owned.space_id())
        );
        assert!(
            spaces
                .iter()
                .any(|space| space.space_id() == shared.space_id())
        );

        sqlx::query(
            "UPDATE memberships
             SET state = 'left', ended_at = now()
             WHERE id = $1",
        )
        .bind(shared_membership)
        .execute(&pool)
        .await
        .unwrap();

        let spaces = list_spaces(&pool, &session).await.unwrap();
        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].space_id(), owned.space_id());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn owner_can_archive_once_and_audit_transition(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let correlation_id = Uuid::new_v4();
        let created = create_space(&pool, &session, "Archive Me", Uuid::new_v4())
            .await
            .unwrap();

        let archived = archive_space(&pool, &session, created.space_id(), correlation_id)
            .await
            .expect("owner should archive active Space");

        assert_eq!(archived.state(), SpaceState::Archived);

        let has_archived_at = sqlx::query_scalar::<_, bool>(
            "SELECT archived_at IS NOT NULL FROM spaces WHERE id = $1",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(has_archived_at);

        let audit = sqlx::query_as::<_, (String, Uuid)>(
            "SELECT event_type, correlation_id
             FROM audit_events
             WHERE space_id = $1 AND event_type = 'space.archived'",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(audit.0, SPACE_ARCHIVED_EVENT);
        assert_eq!(audit.1, correlation_id);

        assert!(matches!(
            archive_space(&pool, &session, created.space_id(), Uuid::new_v4()).await,
            Err(ArchiveSpaceError::NotActive)
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn failed_archive_audit_rolls_back_lifecycle_transition(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let created = create_space(&pool, &session, "Atomic Archive", Uuid::new_v4())
            .await
            .unwrap();

        let existing_audit_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id
             FROM audit_events
             WHERE space_id = $1 AND event_type = 'space.created'",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();

        let result = archive_space_with_audit_id(
            &pool,
            &session,
            created.space_id(),
            Uuid::new_v4(),
            existing_audit_id,
        )
        .await;

        assert!(matches!(result, Err(ArchiveSpaceError::Storage(_))));

        let state = sqlx::query_as::<_, (String, bool)>(
            "SELECT state::text, archived_at IS NULL
             FROM spaces
             WHERE id = $1",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(state.0, "active");
        assert!(
            state.1,
            "rolled-back archive must not leave archived_at set"
        );

        let archived_events = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
             FROM audit_events
             WHERE space_id = $1 AND event_type = 'space.archived'",
        )
        .bind(created.space_id())
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(archived_events, 0);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn member_without_manage_space_cannot_archive(pool: PgPool) {
        let owner = create_identity(&pool).await;
        let member = create_identity(&pool).await;
        let owner_session = identity_session(&pool, owner).await;
        let created = create_space(&pool, &owner_session, "Protected Archive", Uuid::new_v4())
            .await
            .unwrap();

        let membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'member')",
        )
        .bind(membership)
        .bind(created.space_id())
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();

        let member_session = identity_session(&pool, member).await;

        assert!(matches!(
            archive_space(&pool, &member_session, created.space_id(), Uuid::new_v4()).await,
            Err(ArchiveSpaceError::Denied)
        ));

        let state = sqlx::query_scalar::<_, String>("SELECT state::text FROM spaces WHERE id = $1")
            .bind(created.space_id())
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(state, "active");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn archived_space_is_rejected_by_active_write_guard(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let created = create_space(&pool, &session, "Read Only", Uuid::new_v4())
            .await
            .unwrap();
        archive_space(&pool, &session, created.space_id(), Uuid::new_v4())
            .await
            .unwrap();

        let mut transaction = pool.begin().await.unwrap();
        assert!(matches!(
            lock_active_space_for_write(&mut transaction, created.space_id()).await,
            Err(SpaceAccessError::Denied)
        ));
        transaction.rollback().await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn active_write_lock_serializes_competing_space_state_change(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;
        let created = create_space(&pool, &session, "Lock Me", Uuid::new_v4())
            .await
            .unwrap();

        let mut first = pool.begin().await.unwrap();
        authorize_space_capability(
            &mut first,
            &session,
            created.space_id(),
            Capability::ManageSpace,
        )
        .await
        .unwrap();
        lock_active_space_for_write(&mut first, created.space_id())
            .await
            .unwrap();

        let mut competing = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL lock_timeout = '100ms'")
            .execute(&mut *competing)
            .await
            .unwrap();

        let update = sqlx::query(
            "UPDATE spaces
             SET state = 'archived', archived_at = now()
             WHERE id = $1",
        )
        .bind(created.space_id())
        .execute(&mut *competing)
        .await;

        assert!(
            update.is_err(),
            "Space state must not change while an active-write transaction relies on it"
        );

        first.rollback().await.unwrap();
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn failed_owner_membership_insert_rolls_back_space(pool: PgPool) {
        let identity = create_identity(&pool).await;
        let session = identity_session(&pool, identity).await;

        let existing_space = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO spaces (id, name, created_by_identity_id)
             VALUES ($1, 'Existing', $2)",
        )
        .bind(existing_space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let existing_membership = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO memberships (id, space_id, identity_id, role)
             VALUES ($1, $2, $3, 'owner')",
        )
        .bind(existing_membership)
        .bind(existing_space)
        .bind(identity)
        .execute(&pool)
        .await
        .unwrap();

        let attempted_space = Uuid::new_v4();
        let result = create_space_with_ids(
            &pool,
            &session,
            "Must Roll Back",
            Uuid::new_v4(),
            NewSpaceIds {
                space_id: attempted_space,
                owner_membership_id: existing_membership,
                audit_event_id: Uuid::new_v4(),
            },
        )
        .await;

        assert!(matches!(result, Err(CreateSpaceError::Storage(_))));

        let count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*)
             FROM spaces
             WHERE id = $1",
        )
        .bind(attempted_space)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(count, 0, "failed owner creation must roll back the Space");
    }
}
