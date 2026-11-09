//! `cj_members`: one row per account; one crew per account in v0.1.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;

/// A membership row.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "cj_members")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
pub struct Model {
    /// One crew per account in v0.1, so the account id is the key.
    #[sea_orm(primary_key, auto_increment = false)]
    pub uid: String,
    /// Subject id of the account.
    pub subject_id: Uuid,
    /// The crew.
    pub tenant_id: Uuid,
    /// The name the member chose for this crew.
    pub display_name: String,
    /// Verified email.
    pub email: String,
    /// `owner` or `member`.
    pub role: String,
    /// Join time.
    pub joined_at: DateTimeUtc,
}

/// No relations are navigated through the ORM.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
