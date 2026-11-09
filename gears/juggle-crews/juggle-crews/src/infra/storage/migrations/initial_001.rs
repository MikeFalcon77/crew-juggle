//! The first schema: crews and their members.
use sea_orm_migration::prelude::*;

/// Creates `cj_crews` and `cj_members`.
pub struct Migration;

impl MigrationName for Migration {
    /// Recorded as applied under this name since v0.1, when the migration
    /// lived in `store.rs`. Renaming it would run it again on existing data.
    fn name(&self) -> &str {
        "store"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() != sea_orm::DbBackend::Postgres {
            return Err(DbErr::Custom("PostgreSQL required".into()));
        }
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE cj_crews (
                    tenant_id UUID PRIMARY KEY,
                    name VARCHAR(80) NOT NULL,
                    timezone VARCHAR(64) NOT NULL,
                    invite_code VARCHAR(16) NOT NULL UNIQUE,
                    created_at TIMESTAMPTZ NOT NULL
                );
                CREATE TABLE cj_members (
                    uid VARCHAR(128) PRIMARY KEY,
                    subject_id UUID NOT NULL UNIQUE,
                    tenant_id UUID NOT NULL REFERENCES cj_crews(tenant_id) ON DELETE CASCADE,
                    display_name VARCHAR(80) NOT NULL,
                    email VARCHAR(320) NOT NULL,
                    role VARCHAR(16) NOT NULL CHECK (role IN ('owner', 'member')),
                    joined_at TIMESTAMPTZ NOT NULL
                );
                CREATE INDEX cj_members_tenant ON cj_members(tenant_id);",
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE cj_members; DROP TABLE cj_crews;")
            .await?;
        Ok(())
    }
}
