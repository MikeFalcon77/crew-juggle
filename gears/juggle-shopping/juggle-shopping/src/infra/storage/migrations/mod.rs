//! Schema of the shopping gear. The migration has carried this name since v0.1.
use sea_orm_migration::prelude::*;
/// Every migration of the shopping gear, in order.
pub struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(Initial)]
    }
}
/// Creates the three tables and the ledger guard.
pub struct Initial;
impl MigrationName for Initial {
    fn name(&self) -> &str {
        "m20261031_shopping"
    }
}
#[async_trait::async_trait]
impl MigrationTrait for Initial {
    async fn up(&self, m: &SchemaManager) -> std::result::Result<(), DbErr> {
        m.get_connection().execute_unprepared("CREATE TABLE js_state (id UUID PRIMARY KEY,tenant_id UUID NOT NULL,revision BIGINT NOT NULL CHECK(revision>0),payload JSONB NOT NULL); CREATE TABLE js_photos (id UUID PRIMARY KEY,tenant_id UUID NOT NULL,receipt_id TEXT NOT NULL,ordinal INTEGER NOT NULL,image BYTEA NOT NULL,expires_at TIMESTAMPTZ NOT NULL,UNIQUE(tenant_id,receipt_id,ordinal)); CREATE INDEX js_photos_expiry ON js_photos(expires_at); CREATE INDEX js_photos_receipt ON js_photos(tenant_id,receipt_id)").await?;
        m.get_connection().execute_unprepared("CREATE TABLE js_ledger (id UUID PRIMARY KEY,tenant_id UUID NOT NULL,receipt_id TEXT NOT NULL,sequence BIGSERIAL UNIQUE NOT NULL,purchase_date DATE NOT NULL,recorded_at TIMESTAMPTZ NOT NULL,currency TEXT NOT NULL,payload JSONB NOT NULL); CREATE INDEX js_ledger_period ON js_ledger(tenant_id,purchase_date,currency,sequence); CREATE INDEX js_ledger_receipt ON js_ledger(tenant_id,receipt_id,sequence); CREATE FUNCTION js_ledger_guard() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN IF TG_OP <> 'INSERT' THEN RAISE EXCEPTION 'Financial ledger is append-only'; END IF; IF jsonb_typeof(NEW.payload->'postings') IS DISTINCT FROM 'array' OR jsonb_array_length(NEW.payload->'postings') < 2 OR (SELECT COALESCE(SUM((p->>'amount')::bigint),0) FROM jsonb_array_elements(NEW.payload->'postings') p) <> 0 OR EXISTS(SELECT 1 FROM jsonb_array_elements(NEW.payload->'postings') p WHERE p->>'currency' IS DISTINCT FROM NEW.currency) THEN RAISE EXCEPTION 'Financial postings must balance in one currency'; END IF; RETURN NEW; END $$; CREATE TRIGGER js_ledger_guard BEFORE INSERT OR UPDATE OR DELETE ON js_ledger FOR EACH ROW EXECUTE FUNCTION js_ledger_guard()").await?;
        Ok(())
    }
    async fn down(&self, m: &SchemaManager) -> std::result::Result<(), DbErr> {
        m.get_connection().execute_unprepared("DROP TABLE js_ledger; DROP FUNCTION js_ledger_guard(); DROP TABLE js_photos; DROP TABLE js_state").await?;
        Ok(())
    }
}
