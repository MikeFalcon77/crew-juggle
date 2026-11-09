//! Crew shopping: a shared list, receipt scanning and an append-only purchase ledger.
#![forbid(unsafe_code)]

pub mod gear;
pub use gear::ShoppingGear;

#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;

#[cfg(test)]
mod test_support;
