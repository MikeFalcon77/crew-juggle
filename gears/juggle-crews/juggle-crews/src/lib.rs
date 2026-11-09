//! The crews gear: crews, members and invite codes.
//!
//! The public contract is in `juggle-crews-sdk`. Sign-in and the access policy
//! are the plugins in `gears/juggle-crews/plugins/`.
#![forbid(unsafe_code)]

pub use juggle_crews_sdk::{CrewDirectoryV1, CrewsClientV1, CrewsError};

pub mod gear;
pub use gear::CrewsGear;

#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
