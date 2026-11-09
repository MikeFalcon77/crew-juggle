//! The crew's shopping state, receipt photos and purchase ledger in
//! PostgreSQL through the secure ORM.
pub mod codec;
pub mod entity;
pub mod migrations;
mod repo;

pub use repo::PgStore;
