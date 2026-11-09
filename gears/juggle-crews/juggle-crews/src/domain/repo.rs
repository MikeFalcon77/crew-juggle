//! The storage port of the crews domain. `infra::storage` implements it.
use super::error::Result;
use super::model::{Crew, Member};
use async_trait::async_trait;
use toolkit_security::AccessScope;
use uuid::Uuid;

/// Crew and membership storage. Every call takes the scope it may read or
/// write; the service decides the scope, the repository only applies it.
#[async_trait]
pub trait CrewRepo: Send + Sync {
    /// The membership of an account, searched within `scope`.
    async fn member_by_uid(&self, scope: &AccessScope, uid: &str) -> Result<Option<Member>>;
    /// The crew that owns a normalized invite code, searched within `scope`.
    async fn crew_by_code(&self, scope: &AccessScope, code: &str) -> Result<Option<Crew>>;
    /// Ids of all crews within `scope`, oldest first.
    async fn crew_ids(&self, scope: &AccessScope) -> Result<Vec<Uuid>>;
    /// The crew within `scope`.
    async fn crew(&self, scope: &AccessScope) -> Result<Option<Crew>>;
    /// Members within `scope`, in the order they joined.
    async fn members(&self, scope: &AccessScope) -> Result<Vec<Member>>;
    /// Whether `subject` is a member within `scope`.
    async fn is_member(&self, scope: &AccessScope, subject: Uuid) -> Result<bool>;
    /// Creates a crew and its owner in one transaction.
    async fn create_crew(&self, scope: &AccessScope, crew: Crew, owner: Member) -> Result<()>;
    /// Adds an account to an existing crew.
    async fn add_member(&self, scope: &AccessScope, member: Member) -> Result<()>;
    /// Saves a crew's name, zone and invite code.
    async fn update_crew(&self, scope: &AccessScope, crew: &Crew) -> Result<()>;
    /// Deletes a crew and its members (demo mode only).
    async fn delete_crew(&self, scope: &AccessScope) -> Result<()>;
}
