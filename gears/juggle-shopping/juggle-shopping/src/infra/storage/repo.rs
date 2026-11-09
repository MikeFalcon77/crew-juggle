use super::codec;
use super::entity::{ledger, photo, state};
use crate::domain::{Error, Photo, Result, State, Store};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{QuerySelect, Set, entity::prelude::*, sea_query::Expr};
use std::sync::Arc;
use toolkit_db::{
    DBProvider, DbError,
    secure::{SecureDeleteExt, SecureEntityExt, SecureUpdateExt, secure_insert},
};
use toolkit_security::AccessScope;

/// Shopping state, photos and ledger in PostgreSQL.
pub struct PgStore {
    /// The gear's database.
    pub db: Arc<DBProvider<DbError>>,
}
/// The single crew a scope admits; scopes over several crews are refused.
fn crew_of(scope: &AccessScope) -> Result<uuid::Uuid> {
    match scope.all_uuid_values_for(toolkit_security::pep_properties::OWNER_TENANT_ID)[..] {
        [crew] => Ok(crew),
        _ => Err(Error::Forbidden),
    }
}
impl PgStore {
    /// Demo mode only: wipes one crew's list, receipts and photos before reseeding.
    /// The ledger stays: it is append-only, so demo purchases are reversed instead.
    pub async fn reset(&self, scope: &AccessScope) -> Result<()> {
        crew_of(scope)?;
        let conn = self.db.conn().map_err(|_| Error::Storage)?;
        photo::Entity::delete_many()
            .secure()
            .scope_with(scope)
            .exec(&conn)
            .await
            .map_err(|_| Error::Storage)?;
        state::Entity::delete_many()
            .secure()
            .scope_with(scope)
            .exec(&conn)
            .await
            .map_err(|_| Error::Storage)?;
        Ok(())
    }
}
#[async_trait]
impl Store for PgStore {
    /// Creates the crew's empty shopping state once, tolerating concurrent requests.
    async fn ensure(&self, scope: &AccessScope) -> Result<()> {
        let crew = crew_of(scope)?;
        let conn = self.db.conn().map_err(|_| Error::Storage)?;
        if state::Entity::find_by_id(crew)
            .secure()
            .scope_with(scope)
            .one(&conn)
            .await
            .map_err(|_| Error::Storage)?
            .is_some()
        {
            return Ok(());
        }
        let result = secure_insert::<state::Entity>(
            state::ActiveModel {
                id: Set(crew),
                tenant_id: Set(crew),
                revision: Set(1),
                payload: Set(codec::encode_state(&State::default()).map_err(|_| Error::Storage)?),
            },
            scope,
            &conn,
        )
        .await;
        if result.is_err()
            && state::Entity::find_by_id(crew)
                .secure()
                .scope_with(scope)
                .one(&conn)
                .await
                .map_err(|_| Error::Storage)?
                .is_none()
        {
            return Err(Error::Storage);
        }
        Ok(())
    }
    async fn read(&self, scope: &AccessScope) -> Result<(i64, State)> {
        let crew = crew_of(scope)?;
        let scope = scope.clone();
        self.db
            .transaction(move |tx| {
                Box::pin(async move {
                    let row = state::Entity::find_by_id(crew)
                        .lock(sea_orm::sea_query::LockType::Share)
                        .secure()
                        .scope_with(&scope)
                        .one(tx)
                        .await?
                        .ok_or_else(|| sea_orm::DbErr::Custom("Missing shopping state".into()))?;
                    let mut value: State = codec::decode_state(row.payload)
                        .map_err(|_| sea_orm::DbErr::Custom("Invalid shopping state".into()))?;
                    value.events = ledger::Entity::find()
                        .secure()
                        .scope_with(&scope)
                        .order_by(ledger::Column::Sequence, sea_orm::Order::Asc)
                        .all(tx)
                        .await?
                        .into_iter()
                        .map(|e| {
                            codec::decode_event(e.payload).map_err(|_| {
                                sea_orm::DbErr::Custom("Invalid financial event".into())
                            })
                        })
                        .collect::<std::result::Result<Vec<_>, _>>()?;
                    value.persisted_event_count = value.events.len();
                    Ok((row.revision, value))
                })
            })
            .await
            .map_err(|_| Error::Storage)
    }
    async fn commit(
        &self,
        scope: &AccessScope,
        revision: i64,
        value: &State,
        photos: &[Photo],
    ) -> Result<bool> {
        let crew = crew_of(scope)?;
        let payload = codec::encode_state(value).map_err(|_| Error::Storage)?;
        let scope = scope.clone();
        let photos: Vec<_> = photos
            .iter()
            .filter(|p| value.receipts.iter().any(|r| r.id == p.receipt_id))
            .cloned()
            .collect();
        let deleted: Vec<_> = value
            .receipts
            .iter()
            .filter(|r| r.status == "deleted")
            .map(|r| r.id.clone())
            .collect();
        let events = value
            .events
            .iter()
            .skip(value.persisted_event_count)
            .cloned()
            .collect::<Vec<_>>();
        self.db
            .transaction(move |tx| {
                Box::pin(async move {
                    let changed = state::Entity::update_many()
                        .secure()
                        .scope_with(&scope)
                        .filter(state::Column::Id.eq(crew).into())
                        .filter(state::Column::Revision.eq(revision).into())
                        .col_expr(state::Column::Revision, Expr::value(revision + 1))
                        .col_expr(state::Column::Payload, Expr::value(payload))
                        .exec(tx)
                        .await?;
                    if changed.rows_affected != 1 {
                        return Ok(false);
                    }
                    for event in events {
                        let id = uuid::Uuid::parse_str(&event.id)
                            .map_err(|_| sea_orm::DbErr::Custom("Invalid ledger ID".into()))?;
                        if ledger::Entity::find_by_id(id)
                            .secure()
                            .scope_with(&scope)
                            .one(tx)
                            .await?
                            .is_none()
                        {
                            event.validate().map_err(|_| {
                                sea_orm::DbErr::Custom("Unbalanced ledger event".into())
                            })?;
                            secure_insert::<ledger::Entity>(
                                ledger::ActiveModel {
                                    id: Set(id),
                                    tenant_id: Set(crew),
                                    receipt_id: Set(event.receipt_id.clone()),
                                    sequence: sea_orm::NotSet,
                                    purchase_date: Set(chrono::NaiveDate::parse_from_str(
                                        &event.purchase_date,
                                        "%Y-%m-%d",
                                    )
                                    .map_err(|_| {
                                        sea_orm::DbErr::Custom("Invalid purchase date".into())
                                    })?),
                                    recorded_at: Set(event.recorded_at.fixed_offset()),
                                    currency: Set(event.currency.clone()),
                                    payload: Set(codec::encode_event(&event).map_err(|_| {
                                        sea_orm::DbErr::Custom("Invalid ledger event".into())
                                    })?),
                                },
                                &scope,
                                tx,
                            )
                            .await?;
                        }
                    }
                    for p in photos {
                        let id = uuid::Uuid::new_v5(
                            &uuid::Uuid::NAMESPACE_URL,
                            format!("shopping/photo/{}/{}", p.receipt_id, p.ordinal).as_bytes(),
                        );
                        if photo::Entity::find_by_id(id)
                            .secure()
                            .scope_with(&scope)
                            .one(tx)
                            .await?
                            .is_none()
                        {
                            secure_insert::<photo::Entity>(
                                photo::ActiveModel {
                                    id: Set(id),
                                    tenant_id: Set(crew),
                                    receipt_id: Set(p.receipt_id),
                                    ordinal: Set(p.ordinal),
                                    image: Set(p.bytes),
                                    expires_at: Set(p.expires_at.fixed_offset()),
                                },
                                &scope,
                                tx,
                            )
                            .await?;
                        }
                    }
                    if !deleted.is_empty() {
                        photo::Entity::delete_many()
                            .secure()
                            .scope_with(&scope)
                            .filter(photo::Column::ReceiptId.is_in(deleted).into())
                            .exec(tx)
                            .await?;
                    }
                    Ok(true)
                })
            })
            .await
            .map_err(|_| Error::Storage)
    }
    async fn photos(
        &self,
        scope: &AccessScope,
        receipt_id: &str,
        now: DateTime<Utc>,
    ) -> Result<Vec<Vec<u8>>> {
        let conn = self.db.conn().map_err(|_| Error::Storage)?;
        Ok(photo::Entity::find()
            .secure()
            .scope_with(scope)
            .filter(photo::Column::ReceiptId.eq(receipt_id).into())
            .filter(photo::Column::ExpiresAt.gt(now).into())
            .order_by(photo::Column::Ordinal, sea_orm::Order::Asc)
            .all(&conn)
            .await
            .map_err(|_| Error::Storage)?
            .into_iter()
            .map(|r| r.image)
            .collect())
    }
    async fn expire(&self, scope: &AccessScope, now: DateTime<Utc>) -> Result<()> {
        let conn = self.db.conn().map_err(|_| Error::Storage)?;
        photo::Entity::delete_many()
            .secure()
            .scope_with(scope)
            .filter(photo::Column::ExpiresAt.lte(now).into())
            .exec(&conn)
            .await
            .map_err(|_| Error::Storage)?;
        Ok(())
    }
}
