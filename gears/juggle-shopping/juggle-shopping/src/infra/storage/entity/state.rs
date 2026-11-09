//! `js_state`: one row per crew with the shopping state as JSON.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;
/// A crew's shopping state.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "js_state")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
pub struct Model {
    /// The crew id, also the row id.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The crew.
    pub tenant_id: Uuid,
    /// Incremented on every accepted change.
    pub revision: i64,
    /// The state in its stored JSON shape (see `codec`).
    pub payload: Json,
}
/// No relations are navigated through the ORM.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
