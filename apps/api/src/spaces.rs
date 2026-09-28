#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceStatus {
    Active,
    Archived,
    Deleting,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    InvalidTransition,
}

impl SpaceStatus {
    pub fn transition(self, next: Self) -> Result<Self, TransitionError> {
        let valid = matches!(
            (self, next),
            (Self::Active, Self::Archived)
                | (Self::Archived, Self::Active)
                | (Self::Active, Self::Deleting)
                | (Self::Archived, Self::Deleting)
                | (Self::Deleting, Self::Deleted)
        );

        valid
            .then_some(next)
            .ok_or(TransitionError::InvalidTransition)
    }

    pub fn accepts_writes(self) -> bool {
        self == Self::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deleted_space_cannot_transition_back_to_active() {
        assert_eq!(
            SpaceStatus::Deleted.transition(SpaceStatus::Active),
            Err(TransitionError::InvalidTransition)
        );
    }

    #[test]
    fn archived_space_rejects_normal_writes() {
        assert!(!SpaceStatus::Archived.accepts_writes());
        assert!(SpaceStatus::Active.accepts_writes());
    }
}
