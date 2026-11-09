//! Policy plugin for the platform `authz-resolver`.
//!
//! Gears with crew data ask `authz-resolver` "may this caller access its
//! crew's data?"; the resolver routes the question here. The answer is yes for
//! members of that crew and for the internal worker, and it comes with a
//! constraint that limits every query to that one crew.
#![forbid(unsafe_code)]

pub mod gear;
pub use gear::CrewsAuthzPlugin;

pub mod in_process;

#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
