//! The two traits the crews gear registers in ClientHub.
use crate::{AccountIdentity, AccountMembership, CrewInfo, CrewMember, CrewsError};
use async_trait::async_trait;
use toolkit_security::SecurityContext;
use uuid::Uuid;

/// What gears that store crew data ask about the caller's crew.
///
/// Every call is made on behalf of a signed-in member or of the internal
/// worker, and answers only about the crew in that context.
#[async_trait]
pub trait CrewsClientV1: Send + Sync {
    /// The crew of the caller (or of the worker context).
    async fn crew(&self, ctx: &SecurityContext) -> Result<CrewInfo, CrewsError>;
    /// Accounts that belong to the caller's crew.
    async fn members(&self, ctx: &SecurityContext) -> Result<Vec<CrewMember>, CrewsError>;
    /// Every crew id. Only the internal worker identity may call this.
    async fn crew_ids(&self, ctx: &SecurityContext) -> Result<Vec<Uuid>, CrewsError>;
}

/// What the sign-in and policy plugins ask before any crew is known.
///
/// In-process only: it answers across crews, so it is never exposed over REST.
#[async_trait]
pub trait CrewDirectoryV1: Send + Sync {
    /// Whether `subject` is a member of `crew`. The policy plugin asks this.
    async fn is_member(&self, subject: Uuid, crew: Uuid) -> Result<bool, CrewsError>;
    /// Records a verified account and returns the crew it belongs to, or a
    /// lobby tenant if it has none yet. The sign-in plugin calls this.
    async fn sign_in(&self, identity: AccountIdentity) -> Result<AccountMembership, CrewsError>;
}
