//! Errors of the shopping domain. Their messages reach the user as they are.
use toolkit::domain_model;

/// Result of a shopping operation.
pub type Result<T> = std::result::Result<T, Error>;
/// Why a shopping operation failed.
#[domain_model]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("You are not a member of this crew")]
    Forbidden,
    #[error("{0}")]
    Invalid(String),
    #[error("Someone changed this on another device. Refresh and try again.")]
    Conflict,
    #[error("Purchase not found")]
    NotFound,
    #[error("Shopping is temporarily unavailable")]
    Storage,
    #[error("{0}")]
    Model(String),
}
