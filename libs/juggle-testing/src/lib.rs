//! Test support shared by crew-juggle's gears. Only tests link it.
#![forbid(unsafe_code)]

pub mod crew;
pub mod postgres;

pub use crew::FixedCrew;
pub use postgres::{Postgres, postgres};
