use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    authorization::Capability,
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
