//! Puts app-wide secrets into credstore when the server starts.
//!
//! The only code in crew-juggle that sees the OpenAI key. It hands the key to
//! credstore; oagw takes it from there and adds it to outgoing requests, so the
//! shopping gear never holds it.
#![forbid(unsafe_code)]

pub mod gear;
pub use gear::SecretsGear;

#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
