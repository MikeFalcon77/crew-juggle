//! Crews, members and invite codes: the rules, free of storage and HTTP.
pub mod error;
pub mod invite;
pub mod local_client;
pub mod model;
pub mod repo;
pub mod service;

pub use error::DomainError;
pub use service::CrewsService;
