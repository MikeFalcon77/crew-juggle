//! Transport-agnostic models of the crews contract.
use uuid::Uuid;

/// What a member may do inside their crew.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Created the crew; may rotate the invite code and rename the crew.
    Owner,
    /// Joined with an invite code.
    Member,
}

impl Role {
    /// The name used on the wire and in storage: `owner` or `member`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }
}

/// The caller's crew as other gears see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrewInfo {
    /// The crew id, which is also its tenant id.
    pub id: Uuid,
    /// Display name, e.g. "Flat 4B".
    pub name: String,
    /// IANA zone that defines the crew's calendar day and clock slots.
    pub timezone: String,
}

/// One signed-in account that belongs to a crew.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrewMember {
    /// Identity-provider account id (Firebase UID or `demo-*`).
    pub uid: String,
    /// Subject id carried in every `SecurityContext` of this account.
    pub subject_id: Uuid,
    /// The name the member chose for this crew.
    pub display_name: String,
    /// Owner or member.
    pub role: Role,
}

/// An account whose token the sign-in plugin has verified.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccountIdentity {
    /// Identity-provider account id.
    pub uid: String,
    /// Verified email address.
    pub email: String,
    /// Display name from the identity provider, if it has one.
    pub name: Option<String>,
}

/// Whether a signed-in account already belongs to a crew.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MembershipKind {
    /// Belongs to the crew in [`AccountMembership::tenant_id`].
    Member,
    /// Has no crew yet; the tenant is a lobby that no policy grants access to.
    Guest,
}

impl MembershipKind {
    /// The `subject_type` the sign-in plugin puts on the security context.
    pub fn subject_type(self) -> &'static str {
        match self {
            Self::Member => "member",
            Self::Guest => "guest",
        }
    }
}

/// Where a signed-in account lands: its subject id and tenant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccountMembership {
    /// Stable subject id of the account.
    pub subject_id: Uuid,
    /// The crew id, or the account's lobby tenant for a guest.
    pub tenant_id: Uuid,
    /// Member of a crew, or guest without one.
    pub kind: MembershipKind,
}
