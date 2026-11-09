//! `jr_crew_state`: one row of routines state per crew.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "jr_crew_state")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
/// One crew's routines aggregate.
pub struct Model {
    /// The crew; the PDP-derived scope restricts every query to it.
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: uuid::Uuid,
    /// Optimistic revision incremented after an accepted change.
    pub revision: i64,
    /// Serialized aggregate at the persistence boundary.
    pub payload: Json,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
/// No relations are navigated through the ORM.
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
