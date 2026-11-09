//! `js_ledger`: append-only purchase events; a trigger refuses changes.
use sea_orm::entity::prelude::*;
use toolkit_db_macros::Scopable;
/// One purchase event.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
#[sea_orm(table_name = "js_ledger")]
#[secure(tenant_col = "tenant_id", no_resource, no_owner, no_type)]
pub struct Model {
    /// The event id.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// The crew.
    pub tenant_id: Uuid,
    /// The receipt it books.
    pub receipt_id: String,
    /// Global order of insertion.
    pub sequence: i64,
    /// Day of the purchase.
    pub purchase_date: Date,
    /// When it was booked.
    pub recorded_at: DateTimeWithTimeZone,
    /// Currency of all its postings.
    pub currency: String,
    /// The event in its stored JSON shape (see `codec`).
    pub payload: Json,
}
/// No relations are navigated through the ORM.
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
impl ActiveModelBehavior for ActiveModel {}
