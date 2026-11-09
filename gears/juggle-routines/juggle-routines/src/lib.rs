//! Crew routines: repeating tasks, check-offs, reminders and people.
//! Other gears use the `juggle-routines-sdk` contract through ClientHub.
/// HTTP adapters and transport contracts.
mod api;
/// Typed gear configuration.
mod config;
mod domain;
/// Gear registration, dependency wiring and managed worker lifecycle.
mod gear;
/// IO implementations and persistence representations.
mod infra;
pub use gear::RoutinesGear;
pub use juggle_routines_sdk::*;
#[cfg(test)]
#[path = "tests/support.rs"]
pub(crate) mod test_support;
