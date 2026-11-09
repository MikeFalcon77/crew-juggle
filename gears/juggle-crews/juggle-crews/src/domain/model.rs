//! What the crews domain works with.
use chrono::{DateTime, Utc};
use juggle_crews_sdk::Role;
use toolkit::domain_model;
use uuid::Uuid;

/// A crew. Its id is also its tenant id.
#[domain_model]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Crew {
    /// Crew id and tenant id.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// IANA zone of the crew's calendar.
    pub timezone: String,
    /// Normalized invite code: 8 characters, no dash.
    pub invite_code: String,
    /// When the crew was created.
    pub created_at: DateTime<Utc>,
}

/// An account's membership in a crew. One crew per account in v0.1.
#[domain_model]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    /// Identity-provider account id.
    pub uid: String,
    /// Subject id of the account.
    pub subject_id: Uuid,
    /// The crew the account belongs to.
    pub crew_id: Uuid,
    /// The name the member chose for this crew.
    pub display_name: String,
    /// Verified email, shown to crewmates.
    pub email: String,
    /// Owner or member.
    pub role: Role,
    /// When the account joined.
    pub joined_at: DateTime<Utc>,
}

/// What `/me` shows: who you are and, once you have one, your crew.
#[domain_model]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Me {
    /// Identity-provider account id.
    pub uid: String,
    /// Verified email.
    pub email: String,
    /// Name from the identity provider, or the part of the email before `@`.
    pub name: String,
    /// The caller's crew, if they have one.
    pub crew: Option<CrewSummary>,
}

/// The caller's crew as `/me` shows it.
#[domain_model]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrewSummary {
    /// Crew id.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// IANA zone.
    pub timezone: String,
    /// Invite code as people type it: `K7QX-M2PA`.
    pub invite_code: String,
    /// The caller's role in the crew.
    pub role: Role,
    /// Everyone in the crew, in the order they joined.
    pub members: Vec<MemberSummary>,
}

/// A crewmate as `/me` lists them.
#[domain_model]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberSummary {
    /// Identity-provider account id.
    pub uid: String,
    /// The name the member chose for this crew.
    pub display_name: String,
    /// Owner or member.
    pub role: Role,
}
