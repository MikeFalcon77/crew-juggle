//! Wire shapes of the crews API. They match what the web app reads.
use crate::domain::model::{CrewSummary, Me, MemberSummary};
use uuid::Uuid;

/// You and, once you have one, your crew.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct MeDto {
    /// Identity-provider account id.
    pub uid: String,
    /// Verified email.
    pub email: String,
    /// Display name.
    pub name: String,
    /// The caller's crew, or null before they create or join one.
    pub crew: Option<CrewDto>,
}

/// The caller's crew.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct CrewDto {
    /// Crew id.
    #[schema(value_type = String)]
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// IANA zone of the crew's calendar.
    pub timezone: String,
    /// Invite code, `XXXX-XXXX`.
    pub invite_code: String,
    /// The caller's role: `owner` or `member`.
    pub role: String,
    /// Everyone in the crew, in the order they joined.
    pub members: Vec<MemberDto>,
}

/// A crewmate.
#[derive(Debug)]
#[toolkit_macros::api_dto(response)]
pub struct MemberDto {
    /// Identity-provider account id.
    pub uid: String,
    /// The name they chose for this crew.
    pub display_name: String,
    /// `owner` or `member`.
    pub role: String,
}

/// Create a crew with yourself as its owner.
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct CreateCrewRequest {
    /// Crew name, 1–80 characters.
    pub name: String,
    /// Your name in this crew, 1–80 characters.
    pub display_name: String,
    /// IANA time zone, e.g. `Europe/Berlin`.
    pub timezone: String,
}

/// Join a crew with its invite code.
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct JoinCrewRequest {
    /// Invite code; case, spaces and dashes are ignored.
    pub invite_code: String,
    /// Your name in this crew, 1–80 characters.
    pub display_name: String,
}

/// Rename the crew or move it to another zone (owner only).
#[derive(Debug)]
#[toolkit_macros::api_dto(request)]
pub struct UpdateCrewRequest {
    /// Crew name, 1–80 characters.
    pub name: String,
    /// IANA time zone.
    pub timezone: String,
}

impl From<MemberSummary> for MemberDto {
    fn from(member: MemberSummary) -> Self {
        Self {
            uid: member.uid,
            display_name: member.display_name,
            role: member.role.as_str().into(),
        }
    }
}

impl From<CrewSummary> for CrewDto {
    fn from(crew: CrewSummary) -> Self {
        Self {
            id: crew.id,
            name: crew.name,
            timezone: crew.timezone,
            invite_code: crew.invite_code,
            role: crew.role.as_str().into(),
            members: crew.members.into_iter().map(MemberDto::from).collect(),
        }
    }
}

impl From<Me> for MeDto {
    fn from(me: Me) -> Self {
        Self {
            uid: me.uid,
            email: me.email,
            name: me.name,
            crew: me.crew.map(CrewDto::from),
        }
    }
}
