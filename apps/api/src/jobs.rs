use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub const JOB_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JobEnvelope {
    pub schema_version: u16,
    pub job_id: Uuid,
    pub job_type: String,
    pub entity_id: Uuid,
    pub correlation_id: Uuid,
    pub attempt: u16,
}

impl JobEnvelope {
    pub fn new(job_type: impl Into<String>, entity_id: Uuid, correlation_id: Uuid) -> Self {
        Self {
            schema_version: JOB_SCHEMA_VERSION,
            job_id: Uuid::new_v4(),
            job_type: job_type.into(),
            entity_id,
            correlation_id,
            attempt: 0,
        }
    }
}

pub async fn enqueue(pool: &PgPool, job: &JobEnvelope) -> Result<(), sqlx::Error> {
    let payload =
        serde_json::to_value(job).map_err(|error| sqlx::Error::Encode(Box::new(error)))?;

    sqlx::query(
        r#"
        INSERT INTO outbox_events (
            id, schema_version, event_type, aggregate_type, aggregate_id, correlation_id, payload
        )
        VALUES ($1, $2, $3, 'job', $4, $5, $6)
        ON CONFLICT (id) DO NOTHING
        "#,
    )
    .bind(job.job_id)
    .bind(i32::from(job.schema_version))
    .bind(&job.job_type)
    .bind(job.entity_id)
    .bind(job.correlation_id)
    .bind(payload)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_envelopes_are_versioned_and_start_at_attempt_zero() {
        let entity_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let job = JobEnvelope::new("asset.verify", entity_id, correlation_id);

        assert_eq!(job.schema_version, JOB_SCHEMA_VERSION);
        assert_eq!(job.entity_id, entity_id);
        assert_eq!(job.correlation_id, correlation_id);
        assert_eq!(job.attempt, 0);
    }

    #[test]
    fn envelopes_serialize_with_the_contract_fields() {
        let job = JobEnvelope::new("asset.verify", Uuid::new_v4(), Uuid::new_v4());
        let value = serde_json::to_value(&job).expect("job envelope should serialize");

        assert_eq!(value["schema_version"], JOB_SCHEMA_VERSION);
        assert_eq!(value["job_type"], "asset.verify");
        assert_eq!(value["attempt"], 0);
    }
}
