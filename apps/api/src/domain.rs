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
