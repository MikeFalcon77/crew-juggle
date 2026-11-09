//! Routines state and photos in PostgreSQL through the secure ORM.
//!
//! The crew's state is one JSON document with a revision; `codec` is its
//! stored shape, separate from the domain types.
mod avatar_repo;
mod codec;
pub mod entity;
mod journal;
pub mod migrations;
mod repo;

pub use repo::PgStore;
