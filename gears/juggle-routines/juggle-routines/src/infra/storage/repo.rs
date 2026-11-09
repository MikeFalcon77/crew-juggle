//! One row of routines state per crew, committed by a PostgreSQL compare-and-swap.
//! A single atomic UPDATE persists snapshots, journal and idempotency together.
use super::entity::crew_state::{ActiveModel, Column, Entity, Model};
use crate::domain::{Error, Result as DomainResult, State, Store};
use async_trait::async_trait;
use sea_orm::{Set, entity::prelude::*, sea_query::Expr};
use std::sync::Arc;
use toolkit_db::secure::{SecureEntityExt, SecureUpdateExt, secure_insert};
use toolkit_db::{DBProvider, DbError};
use toolkit_security::{AccessScope, pep_properties};

/// Secure ORM implementation of the crew state and avatar ports.
pub struct PgStore {
    /// The gear's database.
    pub db: Arc<DBProvider<DbError>>,
}
/// Wraps any storage failure as the domain's storage error.
pub(crate) fn failure(e: impl Into<anyhow::Error>) -> Error {
    Error::Storage(e.into())
}

/// The single crew a scope admits. Writes need to know which row they own,
/// and a scope that admits several crews (or all) is refused for writes.
pub(crate) fn crew_of(scope: &AccessScope) -> DomainResult<uuid::Uuid> {
    match scope.all_uuid_values_for(pep_properties::OWNER_TENANT_ID)[..] {
        [crew] => Ok(crew),
        _ => Err(Error::Forbidden),
    }
}

/// The revision and state stored in a row.
pub(crate) fn decode(row: Model) -> DomainResult<(i64, State)> {
    let state: State = serde_json::from_value::<super::codec::State>(row.payload)
        .map_err(failure)?
        .into();
    Ok((row.revision, state))
}

impl PgStore {
    /// A store over the gear's database.
    pub fn new(db: Arc<DBProvider<DbError>>) -> Self {
        Self { db }
    }

    /// Demo mode only: replaces one crew's state with a fresh demo state.
    pub async fn replace(&self, scope: &AccessScope, state: &State) -> DomainResult<()> {
        let crew = crew_of(scope)?;
        let conn = self.db.conn().map_err(failure)?;
        let payload =
            serde_json::to_value(super::codec::State::from(state.clone())).map_err(failure)?;
        let updated = Entity::update_many()
            .secure()
            .scope_with(scope)
            .col_expr(Column::Payload, Expr::value(payload))
            .col_expr(Column::Revision, Expr::value(1i64))
            .exec(&conn)
            .await
            .map_err(failure)?;
        if updated.rows_affected == 0 {
            secure_insert::<Entity>(
                ActiveModel {
                    tenant_id: Set(crew),
                    revision: Set(1),
                    payload: Set(
                        serde_json::to_value(super::codec::State::from(state.clone()))
                            .map_err(failure)?,
                    ),
                },
                scope,
                &conn,
            )
            .await
            .map_err(failure)?;
        }
        super::avatar_repo::clear(&self.db, scope).await
    }
}

#[async_trait]
impl Store for PgStore {
    async fn read(&self, scope: &AccessScope) -> DomainResult<(i64, State)> {
        crew_of(scope)?;
        let conn = self.db.conn().map_err(failure)?;
        let row = Entity::find()
            .secure()
            .scope_with(scope)
            .one(&conn)
            .await
            .map_err(failure)?
            .ok_or(Error::NotFound)?;
        decode(row)
    }

    async fn create(&self, scope: &AccessScope, state: &State) -> DomainResult<bool> {
        let crew = crew_of(scope)?;
        let conn = self.db.conn().map_err(failure)?;
        let payload =
            serde_json::to_value(super::codec::State::from(state.clone())).map_err(failure)?;
        match secure_insert::<Entity>(
            ActiveModel {
                tenant_id: Set(crew),
                revision: Set(1),
                payload: Set(payload),
            },
            scope,
            &conn,
        )
        .await
        {
            Ok(_) => Ok(true),
            // Another request created it first; that is fine.
            Err(error) if error.is_unique_violation() => Ok(false),
            Err(error) => Err(failure(error)),
        }
    }

    async fn compare_and_swap(
        &self,
        scope: &AccessScope,
        revision: i64,
        state: &State,
    ) -> DomainResult<bool> {
        crew_of(scope)?;
        let payload =
            serde_json::to_value(super::codec::State::from(state.clone())).map_err(failure)?;
        let conn = self.db.conn().map_err(failure)?;
        let result = Entity::update_many()
            .secure()
            .scope_with(scope)
            .filter(Column::Revision.eq(revision).into())
            .col_expr(Column::Payload, Expr::value(payload))
            .col_expr(Column::Revision, Expr::value(revision + 1))
            .exec(&conn)
            .await
            .map_err(failure)?;
        Ok(result.rows_affected == 1)
    }
}
