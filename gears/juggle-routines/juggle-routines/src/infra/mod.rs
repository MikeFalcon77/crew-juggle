/// Avatar revision, replay and image replacement contracts.
pub mod avatars;
#[cfg(all(test, feature = "integration"))]
#[path = "pg_tests.rs"]
mod pg_tests;
/// Web Push encryption and delivery adapter.
pub mod push;
pub mod storage;
