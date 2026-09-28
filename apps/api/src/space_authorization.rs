use sqlx::PgPool;
use uuid::Uuid;

use crate::authorization::{Capability, Role, role_allows};

#[derive(Debug, PartialEq, Eq)]
pub enum AuthorizationError {
    NotMember,
    Forbidden,
    Database,
}

pub async fn require_capability(
    pool: &PgPool,
    subject_id: Uuid,
    space_id: Uuid,
    capability: Capability,
) -> Result<Role, AuthorizationError> {
    let role: Option<String> = sqlx::query_scalar(
        "SELECT role::text
         FROM memberships
         WHERE subject_id = $1 AND space_id = $2 AND status = 'active'",
    )
    .bind(subject_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| AuthorizationError::Database)?;

    let role = role.ok_or(AuthorizationError::NotMember)?;
    let role = match role.as_str() {
        "owner" => Role::Owner,
        "admin" => Role::Admin,
        "contributor" => Role::Contributor,
        "viewer" => Role::Viewer,
        _ => return Err(AuthorizationError::Database),
    };

    role_allows(role, capability)
        .then_some(role)
        .ok_or(AuthorizationError::Forbidden)
}
