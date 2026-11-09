//! `js_photos`: receipt photos, deleted when they expire.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;
/// One photo of one receipt.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "js_photos")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
pub struct Model {
    /// Derived from the receipt id and ordinal.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The crew.
    pub tenant_id: Uuid,
    /// The receipt.
    pub receipt_id: String,
    /// Position among the receipt's photos.
    pub ordinal: i32,
    /// JPEG bytes.
    pub image: Vec<u8>,
    /// When the photo is deleted.
    pub expires_at: DateTimeWithTimeZone,
}
/// No relations are navigated through the ORM.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
