//! The shopping domain: rules and use cases, free of storage, HTTP and models.
pub mod demo;
pub mod error;
pub mod ledger;
pub mod model;
pub mod ports;
pub mod rules;
pub mod service;
mod state;
mod worker;

pub use error::{Error, Result};
pub use model::*;
pub use ports::*;
pub use rules::*;
pub use service::Service;
