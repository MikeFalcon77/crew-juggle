//! The routines capability other gears consume through ClientHub.
//! Plain types only: no ORM, REST or push-transport details cross this line.
use async_trait::async_trait;
use toolkit_security::SecurityContext;

/// Someone routines can be assigned to: a member with an account, or a crew
/// member without one (Mochi the cat has chores but no login).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrewPerson {
    pub id: String,
    pub name: String,
    /// Identity-provider account id when this person can sign in.
    pub account: Option<String>,
}

/// A push message for crew devices. Only in-app URLs are accepted.
#[derive(Clone, Debug)]
pub struct Notification {
    pub title: String,
    pub body: String,
    /// OS replacement tag; a newer message with the same tag replaces the old one.
    pub tag: String,
    pub url: String,
    pub kind: NotificationKind,
    pub ttl_seconds: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationKind {
    Routines,
    Shopping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryOutcome {
    Sent,
    SubscriptionGone,
    Retry,
}

#[derive(Debug, thiserror::Error)]
pub enum RoutinesError {
    #[error("Access to this crew is denied")]
    Forbidden,
    #[error("Routines are temporarily unavailable")]
    Unavailable,
    #[error("Not found")]
    NotFound,
    #[error("Invalid notification")]
    Invalid,
}

#[async_trait]
pub trait RoutinesClientV1: Send + Sync {
    /// Everyone in the caller's crew who can have routines.
    async fn people(&self, ctx: &SecurityContext) -> Result<Vec<CrewPerson>, RoutinesError>;
    /// Pushes `message` to every registered device in the crew, except the
    /// devices of `skip_account`. Only the internal worker identity may call it.
    /// Returns how many devices accepted the message.
    async fn notify_crew(
        &self,
        ctx: &SecurityContext,
        message: Notification,
        skip_account: Option<String>,
    ) -> Result<usize, RoutinesError>;
}
