//! The first schema: crew state and photos.
use sea_orm_migration::prelude::*;

/// Creates `jr_crew_state` and `jr_avatars`.
pub struct Migration;

impl MigrationName for Migration {
    /// Recorded as applied under this name since v0.1, when the migration
    /// lived in `storage.rs`. Renaming it would run it again on existing data.
    fn name(&self) -> &str {
        "storage"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> std::result::Result<(), DbErr> {
        if manager.get_database_backend() != sea_orm::DbBackend::Postgres {
            return Err(DbErr::Custom("PostgreSQL required".into()));
        }
        manager
            .get_connection()
            .execute_unprepared(
                "CREATE TABLE jr_crew_state (
                    tenant_id UUID PRIMARY KEY,
                    revision BIGINT NOT NULL CHECK (revision > 0),
                    payload JSONB NOT NULL
                );
                CREATE TABLE jr_avatars (
                    tenant_id UUID NOT NULL,
                    person_id VARCHAR(100) NOT NULL,
                    revision BIGINT NOT NULL CHECK (revision > 0),
                    image BYTEA,
                    PRIMARY KEY (tenant_id, person_id)
                );",
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> std::result::Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared("DROP TABLE jr_avatars; DROP TABLE jr_crew_state;")
            .await?;
        Ok(())
    }
}
