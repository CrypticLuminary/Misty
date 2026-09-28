use chrono::{DateTime, Duration, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    authorization::{Capability, Role},
    invitations,
    sessions,
    space_authorization::{AuthorizationError, require_capability},
};

#[derive(Debug)]
pub enum AccessError {
    InvalidInput,
    Unauthorized,
    Forbidden,
    InviteUnavailable,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for AccessError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

impl From<AuthorizationError> for AccessError {
    fn from(value: AuthorizationError) -> Self {
        match value {
            AuthorizationError::NotMember => Self::Unauthorized,
            AuthorizationError::Forbidden => Self::Forbidden,
            AuthorizationError::Database => Self::Database(sqlx::Error::RowNotFound),
        }
    }
}

pub struct IssuedInvitation {
    pub invitation_id: Uuid,
    pub secret: String,
    pub expires_at: DateTime<Utc>,
}

pub struct IssuedSession {
    pub subject_id: Uuid,
    pub session_id: Uuid,
    pub secret: String,
    pub expires_at: DateTime<Utc>,
}


pub async fn create_account_session_for_bootstrap(
    pool: &PgPool,
) -> Result<IssuedSession, AccessError> {
    let subject_id = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    sqlx::query("INSERT INTO subjects (id, kind) VALUES ($1, 'account')")
        .bind(subject_id)
        .execute(&mut *tx)
        .await?;
    let session = create_session(&mut tx, subject_id, Duration::hours(8)).await?;
    tx.commit().await?;
    Ok(session)
}

pub async fn issue_invitation(
    pool: &PgPool,
    actor_subject_id: Uuid,
    space_id: Uuid,
    role: Role,
    lifetime: Duration,
    correlation_id: Uuid,
) -> Result<IssuedInvitation, AccessError> {
    require_capability(pool, actor_subject_id, space_id, Capability::Invite).await?;
    if matches!(role, Role::Owner) || lifetime <= Duration::zero() || lifetime > Duration::days(30) {
        return Err(AccessError::InvalidInput);
    }

    let creator_membership_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM memberships
         WHERE subject_id = $1 AND space_id = $2 AND status = 'active'",
    )
    .bind(actor_subject_id)
    .bind(space_id)
    .fetch_one(pool)
    .await?;

    let raw_secret = invitations::generate_secret();
    let secret_hash = invitations::hash_secret(&raw_secret);
    let invitation_id = Uuid::new_v4();
    let expires_at = Utc::now() + lifetime;

    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO invitations
         (id, space_id, token_hash, role, created_by_membership_id, expires_at)
         VALUES ($1, $2, $3, $4::membership_role, $5, $6)",
    )
    .bind(invitation_id)
    .bind(space_id)
    .bind(secret_hash.as_slice())
    .bind(role_db(role))
    .bind(creator_membership_id)
    .bind(expires_at)
    .execute(&mut *tx)
    .await?;

    audit(&mut tx, space_id, actor_subject_id, "invitation.created", correlation_id).await?;
    tx.commit().await?;

    Ok(IssuedInvitation {
        invitation_id,
        secret: invitations::encode_secret(&raw_secret),
        expires_at,
    })
}

pub async fn revoke_invitation(
    pool: &PgPool,
    actor_subject_id: Uuid,
    space_id: Uuid,
    invitation_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), AccessError> {
    require_capability(pool, actor_subject_id, space_id, Capability::Invite).await?;
    let mut tx = pool.begin().await?;
    let affected = sqlx::query(
        "UPDATE invitations SET revoked_at = COALESCE(revoked_at, now())
         WHERE id = $1 AND space_id = $2",
    )
    .bind(invitation_id)
    .bind(space_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(AccessError::InviteUnavailable);
    }
    audit(&mut tx, space_id, actor_subject_id, "invitation.revoked", correlation_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn join_with_invitation(
    pool: &PgPool,
    encoded_secret: &str,
    display_name: &str,
    correlation_id: Uuid,
) -> Result<IssuedSession, AccessError> {
    let display_name = display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > 80 {
        return Err(AccessError::InvalidInput);
    }
    let raw_secret = invitations::decode_secret(encoded_secret).ok_or(AccessError::InviteUnavailable)?;
    let token_hash = invitations::hash_secret(&raw_secret);

    let mut tx = pool.begin().await?;
    let row: Option<(Uuid, Uuid, String)> = sqlx::query_as(
        "SELECT id, space_id, role::text
         FROM invitations
         WHERE token_hash = $1
           AND revoked_at IS NULL
           AND expires_at > now()
           AND (max_uses IS NULL OR use_count < max_uses)
         FOR UPDATE",
    )
    .bind(token_hash.as_slice())
    .fetch_optional(&mut *tx)
    .await?;

    let (invitation_id, space_id, role) = row.ok_or(AccessError::InviteUnavailable)?;
    let subject_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();

    sqlx::query("INSERT INTO subjects (id, kind) VALUES ($1, 'guest')")
        .bind(subject_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO memberships (id, space_id, subject_id, role, display_name)
         VALUES ($1, $2, $3, $4::membership_role, $5)",
    )
    .bind(membership_id)
    .bind(space_id)
    .bind(subject_id)
    .bind(role)
    .bind(display_name)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE invitations SET use_count = use_count + 1 WHERE id = $1")
        .bind(invitation_id)
        .execute(&mut *tx)
        .await?;

    let issued = create_session(&mut tx, subject_id, Duration::days(7)).await?;
    audit(&mut tx, space_id, subject_id, "member.joined", correlation_id).await?;
    tx.commit().await?;
    Ok(issued)
}

pub async fn authenticate_session(pool: &PgPool, encoded_secret: &str) -> Result<Uuid, AccessError> {
    let raw_secret = sessions::decode_secret(encoded_secret).ok_or(AccessError::Unauthorized)?;
    let secret_hash = sessions::hash_secret(&raw_secret);
    let subject_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT subject_id FROM sessions
         WHERE secret_hash = $1 AND revoked_at IS NULL AND expires_at > now()",
    )
    .bind(secret_hash.as_slice())
    .fetch_optional(pool)
    .await?;
    subject_id.ok_or(AccessError::Unauthorized)
}

pub async fn revoke_session(
    pool: &PgPool,
    subject_id: Uuid,
    session_id: Uuid,
) -> Result<(), AccessError> {
    let affected = sqlx::query(
        "UPDATE sessions SET revoked_at = COALESCE(revoked_at, now())
         WHERE id = $1 AND subject_id = $2",
    )
    .bind(session_id)
    .bind(subject_id)
    .execute(pool)
    .await?
    .rows_affected();
    (affected == 1).then_some(()).ok_or(AccessError::Unauthorized)
}

async fn create_session(
    tx: &mut Transaction<'_, Postgres>,
    subject_id: Uuid,
    lifetime: Duration,
) -> Result<IssuedSession, AccessError> {
    let raw_secret = sessions::generate_secret();
    let secret_hash = sessions::hash_secret(&raw_secret);
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + lifetime;

    sqlx::query(
        "INSERT INTO sessions (id, subject_id, secret_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(subject_id)
    .bind(secret_hash.as_slice())
    .bind(expires_at)
    .execute(&mut **tx)
    .await?;

    Ok(IssuedSession {
        subject_id,
        session_id,
        secret: sessions::encode_secret(&raw_secret),
        expires_at,
    })
}

async fn audit(
    tx: &mut Transaction<'_, Postgres>,
    space_id: Uuid,
    actor_subject_id: Uuid,
    event_type: &str,
    correlation_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO audit_events
         (id, space_id, actor_subject_id, event_type, correlation_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(space_id)
    .bind(actor_subject_id)
    .bind(event_type)
    .bind(correlation_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn role_db(role: Role) -> &'static str {
    match role {
        Role::Owner => "owner",
        Role::Admin => "admin",
        Role::Contributor => "contributor",
        Role::Viewer => "viewer",
    }
}
