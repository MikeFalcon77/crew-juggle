//! Sign-in: verify a token, then ask the crews gear where the account lands.
pub mod client;
pub mod ports;
pub mod service;

pub use ports::{IdentityVerifier, Refusal};
pub use service::SignIn;
