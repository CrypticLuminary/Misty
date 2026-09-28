use chrono::Duration;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    access_use_cases::{
        AccessError, authenticate_session, issue_invitation, join_with_invitation, revoke_invitation,
        revoke_session,
    },
    authorization::{Capability, Role},
    space_authorization::{AuthorizationError, require_capability},
    space_use_cases::create_space,
};

#[sqlx::test(migrations = "./migrations")]
async fn owner_membership_is_created_and_authorizes_management(pool: PgPool) {
    let owner = Uuid::new_v4();
    let space = create_space(&pool, owner, "Owner", "Mustang Trip", Uuid::new_v4())
        .await
        .expect("space should be created");

    let result = require_capability(&pool, owner, space, Capability::ManageSpace).await;
    assert!(result.is_ok());
}

#[sqlx::test(migrations = "./migrations")]
async fn subject_from_another_space_is_denied(pool: PgPool) {
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();
    let alice_space = create_space(&pool, alice, "Alice", "Alice Trip", Uuid::new_v4())
        .await
        .expect("alice space");
    let _bob_space = create_space(&pool, bob, "Bob", "Bob Trip", Uuid::new_v4())
        .await
        .expect("bob space");

    let result = require_capability(&pool, bob, alice_space, Capability::View).await;
    assert_eq!(result, Err(AuthorizationError::NotMember));
}

#[sqlx::test(migrations = "./migrations")]
async fn invitation_join_issues_a_valid_scoped_guest_session(pool: PgPool) {
    let owner = Uuid::new_v4();
    let space = create_space(&pool, owner, "Owner", "Trip", Uuid::new_v4())
        .await
        .expect("space");

    let invitation = issue_invitation(
        &pool,
        owner,
        space,
        Role::Contributor,
        Duration::hours(24),
        Uuid::new_v4(),
    )
    .await
    .expect("invitation");

    let session = join_with_invitation(&pool, &invitation.secret, "Guest", Uuid::new_v4())
        .await
        .expect("join");

    assert_eq!(
        authenticate_session(&pool, &session.secret).await.expect("session"),
        session.subject_id
    );
    assert!(
        require_capability(&pool, session.subject_id, space, Capability::Upload)
            .await
            .is_ok()
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn revoked_invitation_cannot_be_used(pool: PgPool) {
    let owner = Uuid::new_v4();
    let space = create_space(&pool, owner, "Owner", "Trip", Uuid::new_v4())
        .await
        .expect("space");
    let invitation = issue_invitation(
        &pool,
        owner,
        space,
        Role::Viewer,
        Duration::hours(1),
        Uuid::new_v4(),
    )
    .await
    .expect("invitation");

    revoke_invitation(&pool, owner, space, invitation.invitation_id, Uuid::new_v4())
        .await
        .expect("revoke");

    assert!(matches!(
        join_with_invitation(&pool, &invitation.secret, "Late Guest", Uuid::new_v4()).await,
        Err(AccessError::InviteUnavailable)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn revoked_session_stops_authenticating(pool: PgPool) {
    let owner = Uuid::new_v4();
    let space = create_space(&pool, owner, "Owner", "Trip", Uuid::new_v4())
        .await
        .expect("space");
    let invitation = issue_invitation(
        &pool,
        owner,
        space,
        Role::Viewer,
        Duration::hours(1),
        Uuid::new_v4(),
    )
    .await
    .expect("invitation");
    let session = join_with_invitation(&pool, &invitation.secret, "Guest", Uuid::new_v4())
        .await
        .expect("join");

    revoke_session(&pool, session.subject_id, session.session_id)
        .await
        .expect("revoke session");

    assert!(matches!(
        authenticate_session(&pool, &session.secret).await,
        Err(AccessError::Unauthorized)
    ));
}

#[sqlx::test(migrations = "./migrations")]
async fn malformed_invitation_is_indistinguishable_from_unavailable_invite(pool: PgPool) {
    assert!(matches!(
        join_with_invitation(&pool, "not-a-valid-secret***", "Guest", Uuid::new_v4()).await,
        Err(AccessError::InviteUnavailable)
    ));
}
