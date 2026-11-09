//! Crews and memberships in PostgreSQL through the secure ORM.
pub mod entity;
pub mod migrations;
pub mod repo;

pub use repo::SeaOrmCrewRepo;

#[cfg(all(test, feature = "integration"))]
#[path = "pg_tests.rs"]
mod pg_tests;
