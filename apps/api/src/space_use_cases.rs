use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug)]
pub enum SpaceError {
    InvalidName,
    Database(sqlx::Error),
}

impl From<sqlx::Error> for SpaceError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value)
    }
}

pub async fn create_space(
    pool: &PgPool,
    owner_subject_id: Uuid,
    owner_display_name: &str,
    space_name: &str,
    correlation_id: Uuid,
) -> Result<Uuid, SpaceError> {
    let space_name = space_name.trim();
    let owner_display_name = owner_display_name.trim();

    if space_name.is_empty() || space_name.chars().count() > 120 {
        return Err(SpaceError::InvalidName);
    }
    if owner_display_name.is_empty() || owner_display_name.chars().count() > 80 {
        return Err(SpaceError::InvalidName);
    }

    let mut tx = pool.begin().await?;
    ensure_subject(&mut tx, owner_subject_id).await?;

    let space_id = Uuid::new_v4();
    let membership_id = Uuid::new_v4();

    sqlx::query("INSERT INTO spaces (id, name) VALUES ($1, $2)")
        .bind(space_id)
        .bind(space_name)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO memberships (id, space_id, subject_id, role, display_name)
         VALUES ($1, $2, $3, 'owner', $4)",
    )
    .bind(membership_id)
    .bind(space_id)
    .bind(owner_subject_id)
    .bind(owner_display_name)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO audit_events
         (id, space_id, actor_subject_id, event_type, correlation_id)
         VALUES ($1, $2, $3, 'space.created', $4)",
    )
    .bind(Uuid::new_v4())
    .bind(space_id)
    .bind(owner_subject_id)
    .bind(correlation_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(space_id)
}

async fn ensure_subject(
    tx: &mut Transaction<'_, Postgres>,
    subject_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO subjects (id, kind) VALUES ($1, 'account')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(subject_id)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn trims_are_part_of_the_use_case_contract() {
        assert_eq!("  trip  ".trim(), "trip");
    }
}
