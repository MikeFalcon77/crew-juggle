//! Sign-in plugin for the platform `authn-resolver`.
//!
//! It proves who a caller is (a Firebase ID token, or a demo character in demo
//! mode) and asks the crews gear which crew the account belongs to. What the
//! caller may then do is the policy plugin's decision, not this one's.
#![forbid(unsafe_code)]

pub mod gear;
pub use gear::CrewsAuthnPlugin;

#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
