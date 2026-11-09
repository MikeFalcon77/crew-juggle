//! Adapters of the shopping gear.
pub mod demo_scanner;
pub mod no_scanner;
pub mod openai;
#[cfg(all(test, feature = "integration"))]
#[path = "pg_tests.rs"]
mod pg_tests;
pub mod photos;
pub mod storage;
