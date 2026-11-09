//! Schema of the routines gear.
use sea_orm_migration::prelude::*;

mod initial_001;

/// Every migration of the routines gear, in order.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(initial_001::Migration)]
    }
}
