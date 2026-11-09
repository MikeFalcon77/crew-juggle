//! `cj_crews`: one row per crew.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;

/// A crew row.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "cj_crews")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
pub struct Model {
    /// The crew id is the tenant id.
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: Uuid,
    /// Display name.
    pub name: String,
    /// IANA zone.
    pub timezone: String,
    /// Normalized invite code.
    pub invite_code: String,
    /// Creation time.
    pub created_at: DateTimeUtc,
}

/// No relations are navigated through the ORM.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
