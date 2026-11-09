//! `CrewRepo` on PostgreSQL. Every query goes through the secure ORM with the
//! scope the domain passed in; this file never decides a scope itself.
use super::entity::{crew, member};
use crate::domain::error::{DomainError, Result};
use crate::domain::model::{Crew, Member};
use crate::domain::repo::CrewRepo;
use async_trait::async_trait;
use juggle_crews_sdk::Role;
use sea_orm::{ActiveValue::Set, ColumnTrait, Order, entity::prelude::*};
use std::sync::Arc;
use toolkit_db::secure::{
    DBRunner, SecureDeleteExt, SecureEntityExt, SecureUpdateExt, secure_insert,
};
use toolkit_db::{DBProvider, DbError};
use toolkit_security::AccessScope;
use uuid::Uuid;

/// Crews and members in PostgreSQL.
pub struct SeaOrmCrewRepo {
    db: Arc<DBProvider<DbError>>,
}

impl SeaOrmCrewRepo {
    /// A repository over the gear's database.
    pub fn new(db: Arc<DBProvider<DbError>>) -> Self {
        Self { db }
    }
}

fn unavailable(error: impl std::fmt::Display) -> DomainError {
    DomainError::Unavailable(format!("crew storage: {error}"))
}

/// Lets a failed transaction surface as the domain's storage error.
impl From<DbError> for DomainError {
    fn from(error: DbError) -> Self {
        unavailable(error)
    }
}

fn role_from(text: &str) -> Role {
    if text == "owner" {
        Role::Owner
    } else {
        Role::Member
    }
}

impl From<crew::Model> for Crew {
    fn from(row: crew::Model) -> Self {
        Self {
            id: row.tenant_id,
            name: row.name,
            timezone: row.timezone,
            invite_code: row.invite_code,
            created_at: row.created_at,
        }
    }
}

impl From<member::Model> for Member {
    fn from(row: member::Model) -> Self {
        Self {
            role: role_from(&row.role),
            uid: row.uid,
            subject_id: row.subject_id,
            crew_id: row.tenant_id,
            display_name: row.display_name,
            email: row.email,
            joined_at: row.joined_at,
        }
    }
}

async fn insert_member(member: Member, scope: &AccessScope, conn: &impl DBRunner) -> Result<()> {
    secure_insert::<member::Entity>(
        member::ActiveModel {
            uid: Set(member.uid),
            subject_id: Set(member.subject_id),
            tenant_id: Set(member.crew_id),
            display_name: Set(member.display_name),
            email: Set(member.email),
            role: Set(member.role.as_str().into()),
            joined_at: Set(member.joined_at),
        },
        scope,
        conn,
    )
    .await
    .map_err(unavailable)?;
    Ok(())
}

#[async_trait]
impl CrewRepo for SeaOrmCrewRepo {
    async fn member_by_uid(&self, scope: &AccessScope, uid: &str) -> Result<Option<Member>> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(member::Entity::find()
            .secure()
            .scope_with(scope)
            .filter(member::Column::Uid.eq(uid).into())
            .one(&conn)
            .await
            .map_err(unavailable)?
            .map(Member::from))
    }

    async fn crew_by_code(&self, scope: &AccessScope, code: &str) -> Result<Option<Crew>> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(crew::Entity::find()
            .secure()
            .scope_with(scope)
            .filter(crew::Column::InviteCode.eq(code).into())
            .one(&conn)
            .await
            .map_err(unavailable)?
            .map(Crew::from))
    }

    async fn crew_ids(&self, scope: &AccessScope) -> Result<Vec<Uuid>> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(crew::Entity::find()
            .secure()
            .scope_with(scope)
            .order_by(crew::Column::CreatedAt, Order::Asc)
            .all(&conn)
            .await
            .map_err(unavailable)?
            .into_iter()
            .map(|c| c.tenant_id)
            .collect())
    }

    async fn crew(&self, scope: &AccessScope) -> Result<Option<Crew>> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(crew::Entity::find()
            .secure()
            .scope_with(scope)
            .one(&conn)
            .await
            .map_err(unavailable)?
            .map(Crew::from))
    }

    async fn members(&self, scope: &AccessScope) -> Result<Vec<Member>> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(member::Entity::find()
            .secure()
            .scope_with(scope)
            .order_by(member::Column::JoinedAt, Order::Asc)
            .all(&conn)
            .await
            .map_err(unavailable)?
            .into_iter()
            .map(Member::from)
            .collect())
    }

    async fn is_member(&self, scope: &AccessScope, subject: Uuid) -> Result<bool> {
        let conn = self.db.conn().map_err(unavailable)?;
        Ok(member::Entity::find()
            .secure()
            .scope_with(scope)
            .filter(member::Column::SubjectId.eq(subject).into())
            .one(&conn)
            .await
            .map_err(unavailable)?
            .is_some())
    }

    async fn create_crew(&self, scope: &AccessScope, crew: Crew, owner: Member) -> Result<()> {
        let scope = scope.clone();
        self.db
            .db()
            .transaction_ref_mapped(move |txn| {
                Box::pin(async move {
                    secure_insert::<crew::Entity>(
                        crew::ActiveModel {
                            tenant_id: Set(crew.id),
                            name: Set(crew.name),
                            timezone: Set(crew.timezone),
                            invite_code: Set(crew.invite_code),
                            created_at: Set(crew.created_at),
                        },
                        &scope,
                        txn,
                    )
                    .await
                    .map_err(unavailable)?;
                    insert_member(owner, &scope, txn).await
                })
            })
            .await
    }

    async fn add_member(&self, scope: &AccessScope, member: Member) -> Result<()> {
        let conn = self.db.conn().map_err(unavailable)?;
        insert_member(member, scope, &conn).await
    }

    async fn update_crew(&self, scope: &AccessScope, crew: &Crew) -> Result<()> {
        let conn = self.db.conn().map_err(unavailable)?;
        crew::Entity::update_many()
            .secure()
            .scope_with(scope)
            .filter(crew::Column::TenantId.eq(crew.id).into())
            .col_expr(
                crew::Column::Name,
                sea_orm::sea_query::Expr::value(crew.name.clone()),
            )
            .col_expr(
                crew::Column::Timezone,
                sea_orm::sea_query::Expr::value(crew.timezone.clone()),
            )
            .col_expr(
                crew::Column::InviteCode,
                sea_orm::sea_query::Expr::value(crew.invite_code.clone()),
            )
            .exec(&conn)
            .await
            .map_err(unavailable)?;
        Ok(())
    }

    async fn delete_crew(&self, scope: &AccessScope) -> Result<()> {
        let conn = self.db.conn().map_err(unavailable)?;
        member::Entity::delete_many()
            .secure()
            .scope_with(scope)
            .exec(&conn)
            .await
            .map_err(unavailable)?;
        crew::Entity::delete_many()
            .secure()
            .scope_with(scope)
            .exec(&conn)
            .await
            .map_err(unavailable)?;
        Ok(())
    }
}
