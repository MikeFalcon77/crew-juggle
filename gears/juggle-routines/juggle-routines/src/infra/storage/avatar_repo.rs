//! `AvatarStore` on PostgreSQL: one photo per person inside one crew.
use super::entity::{avatar as entity, crew_state};
use super::repo::{self as storage, PgStore};
use crate::domain::{AvatarEdit, AvatarMeta, AvatarStore, Error, Result, State};
use async_trait::async_trait;
use aws_lc_rs::digest::{SHA256, digest};
use sea_orm::{QuerySelect, Set, entity::prelude::*};
use std::collections::HashMap;
use toolkit_db::secure::{SecureDeleteExt, SecureEntityExt, SecureUpdateExt, secure_insert};
use toolkit_db::{DBProvider, DbError};
use toolkit_security::AccessScope;

impl From<toolkit_db::DbError> for Error {
    fn from(error: toolkit_db::DbError) -> Self {
        Error::Storage(error.into())
    }
}
fn failure(e: impl Into<anyhow::Error>) -> Error {
    Error::Storage(e.into())
}
#[async_trait]
impl AvatarStore for PgStore {
    async fn metadata(&self, scope: &AccessScope) -> Result<HashMap<String, AvatarMeta>> {
        let conn = self.db.conn().map_err(failure)?;
        let rows = entity::Entity::find()
            .secure()
            .scope_with(scope)
            .all(&conn)
            .await
            .map_err(failure)?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.person_id,
                    AvatarMeta {
                        revision: r.revision as u64,
                        has_photo: r.image.is_some(),
                    },
                )
            })
            .collect())
    }
    async fn image(&self, scope: &AccessScope, id: &str) -> Result<Vec<u8>> {
        let conn = self.db.conn().map_err(failure)?;
        entity::Entity::find()
            .secure()
            .scope_with(scope)
            .filter(entity::Column::PersonId.eq(id).into())
            .one(&conn)
            .await
            .map_err(failure)?
            .and_then(|r| r.image)
            .ok_or(Error::NotFound)
    }
    async fn change(
        &self,
        scope: &AccessScope,
        id: &str,
        actor: &str,
        request: &AvatarEdit,
        image: Option<Vec<u8>>,
    ) -> Result<AvatarMeta> {
        let digest = image
            .as_ref()
            .map(|bytes| {
                digest(&SHA256, bytes)
                    .as_ref()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
            })
            .unwrap_or_else(|| "remove".into());
        let id = id.to_owned();
        let actor = actor.to_owned();
        let request = request.clone();
        let scope = scope.clone();
        let crew = storage::crew_of(&scope)?;
        self.db
            .db()
            .transaction_ref_mapped(move |txn| {
                Box::pin(async move {
                    let row = crew_state::Entity::find()
                        .lock(sea_orm::sea_query::LockType::Update)
                        .secure()
                        .scope_with(&scope)
                        .one(txn)
                        .await
                        .map_err(failure)?
                        .ok_or(Error::NotFound)?;
                    let (revision_before, mut state): (i64, State) = storage::decode(row)?;
                    let previous = entity::Entity::find()
                        .secure()
                        .scope_with(&scope)
                        .filter(entity::Column::PersonId.eq(id.clone()).into())
                        .one(txn)
                        .await
                        .map_err(failure)?;
                    let decision = state.apply_avatar_change(
                        &id,
                        &actor,
                        &request,
                        &digest,
                        previous.as_ref().map_or(0, |r| r.revision as u64),
                        image.is_some(),
                    )?;
                    if decision.replayed {
                        return Ok(decision.metadata);
                    }
                    let response = decision.metadata;
                    let revision = i64::try_from(response.revision).map_err(|_| Error::Conflict)?;
                    if previous.is_some() {
                        entity::Entity::update_many()
                            .secure()
                            .scope_with(&scope)
                            .filter(entity::Column::PersonId.eq(id.clone()).into())
                            .col_expr(
                                entity::Column::Revision,
                                sea_orm::sea_query::Expr::value(revision),
                            )
                            .col_expr(
                                entity::Column::Image,
                                sea_orm::sea_query::Expr::value(image),
                            )
                            .exec(txn)
                            .await
                            .map_err(failure)?;
                    } else {
                        secure_insert::<entity::Entity>(
                            entity::ActiveModel {
                                person_id: Set(id),
                                tenant_id: Set(crew),
                                revision: Set(revision),
                                image: Set(image),
                            },
                            &scope,
                            txn,
                        )
                        .await
                        .map_err(failure)?;
                    }
                    crew_state::Entity::update_many()
                        .secure()
                        .scope_with(&scope)
                        .col_expr(
                            crew_state::Column::Revision,
                            sea_orm::sea_query::Expr::value(revision_before + 1),
                        )
                        .col_expr(
                            crew_state::Column::Payload,
                            sea_orm::sea_query::Expr::value(
                                serde_json::to_value(super::codec::State::from(state.clone()))
                                    .map_err(failure)?,
                            ),
                        )
                        .exec(txn)
                        .await
                        .map_err(failure)?;
                    Ok(response)
                })
            })
            .await
    }
}

/// Demo mode only: removes one crew's photos.
pub(crate) async fn clear(db: &DBProvider<DbError>, scope: &AccessScope) -> Result<()> {
    storage::crew_of(scope)?;
    let conn = db.conn().map_err(failure)?;
    entity::Entity::delete_many()
        .secure()
        .scope_with(scope)
        .exec(&conn)
        .await
        .map_err(failure)?;
    Ok(())
}
