//! `jr_avatars`: one photo per person inside one crew.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "jr_avatars")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
/// One person's photo inside one crew.
pub struct Model {
    /// The crew; the PDP-derived scope restricts every query to it.
    #[sea_orm(primary_key, auto_increment = false)]
    pub tenant_id: uuid::Uuid,
    /// The person inside that crew.
    #[sea_orm(primary_key, auto_increment = false)]
    pub person_id: String,
    /// Optimistic revision incremented after an accepted change.
    pub revision: i64,
    /// Normalized photo bytes, or None after deletion.
    pub image: Option<Vec<u8>>,
}
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
/// No ORM relationships are needed for this aggregate.
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
