use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Owner,
    Admin,
    Contributor,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn role_allows(role: Role, capability: Capability) -> bool {
    use Capability::*;

    match role {
        Role::Owner => true,
        Role::Admin => matches!(
            capability,
            View | Upload
                | Download
                | DownloadOriginal
                | DeleteOwn
                | DeleteAny
                | Invite
                | ManageMembers
                | ManageSpace
                | EnableAi
                | ViewLocation
        ),
        Role::Contributor => matches!(
            capability,
            View | Upload | Download | DownloadOriginal | DeleteOwn
        ),
        Role::Viewer => matches!(capability, View | Download),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewer_is_denied_mutating_capabilities() {
        assert!(!role_allows(Role::Viewer, Capability::Upload));
        assert!(!role_allows(Role::Viewer, Capability::Invite));
        assert!(!role_allows(Role::Viewer, Capability::ManageSpace));
    }

    #[test]
    fn contributor_cannot_delete_other_members_media_or_invite() {
        assert!(role_allows(Role::Contributor, Capability::DeleteOwn));
        assert!(!role_allows(Role::Contributor, Capability::DeleteAny));
        assert!(!role_allows(Role::Contributor, Capability::Invite));
    }

    #[test]
    fn owner_has_every_declared_capability() {
        let capabilities = [
            Capability::View,
            Capability::Upload,
            Capability::Download,
            Capability::DownloadOriginal,
            Capability::DeleteOwn,
            Capability::DeleteAny,
            Capability::Invite,
            Capability::ManageMembers,
            Capability::ManageSpace,
            Capability::EnableAi,
            Capability::ViewLocation,
        ];

        assert!(
            capabilities
                .into_iter()
                .all(|capability| role_allows(Role::Owner, capability))
        );
    }
}
