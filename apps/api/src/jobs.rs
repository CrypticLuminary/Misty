use serde::{Deserialize, Serialize};
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
}
