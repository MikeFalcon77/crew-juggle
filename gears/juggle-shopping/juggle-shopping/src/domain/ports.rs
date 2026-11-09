//! What the shopping domain needs from the outside world.
use super::error::Result;
use super::model::{Action, CorrectionRule, ExtractedProduct, Extraction, Photo, Product, State};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use toolkit_security::AccessScope;

#[async_trait]
pub trait Store: Send + Sync {
    /// Creates the crew's empty shopping state if it does not exist yet.
    async fn ensure(&self, scope: &AccessScope) -> Result<()>;
    async fn read(&self, scope: &AccessScope) -> Result<(i64, State)>;
    async fn commit(
        &self,
        scope: &AccessScope,
        revision: i64,
        state: &State,
        photos: &[Photo],
    ) -> Result<bool>;
    async fn photos(
        &self,
        scope: &AccessScope,
        receipt_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Vec<Vec<u8>>>;
    async fn expire(&self, scope: &AccessScope, now: DateTime<Utc>) -> Result<()>;
}
#[async_trait]
pub trait Intelligence: Send + Sync {
    async fn receipt(
        &self,
        photos: &[Vec<u8>],
        catalog: &[Product],
        rules: &[CorrectionRule],
    ) -> Result<Extraction>;
    async fn list(&self, text: &str, catalog: &[Product]) -> Result<Vec<ExtractedProduct>>;
    fn configured(&self) -> bool;
}

/// The stored identity of a command, for telling a replay from a new request
/// that reuses an operation id.
pub trait ActionFingerprint: Send + Sync {
    /// A stable text form of `action`.
    fn of(&self, action: &Action) -> String;
}

/// Turns an uploaded photo into the stored JPEG.
#[async_trait]
pub trait PhotoNormalizer: Send + Sync {
    /// The photo, re-encoded, or why it was refused.
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;
}
