//! Errors of the crews domain. Their messages reach the user as they are.
use toolkit::domain_model;

/// Why a crews operation failed.
#[domain_model]
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// The caller's identity is not known to this server (sign in again).
    #[error("Sign in again")]
    Unauthorized,
    /// The caller may not do this, with the reason shown to them.
    #[error("{0}")]
    Forbidden(&'static str),
    /// The request is not valid, with the reason shown to them.
    #[error("{0}")]
    Invalid(String),
    /// The request conflicts with the current state.
    #[error("{0}")]
    Conflict(&'static str),
    /// The crew does not exist.
    #[error("Not found")]
    NotFound,
    /// Storage cannot answer; the cause goes to the log, not to the user.
    #[error("Crews are temporarily unavailable")]
    Unavailable(String),
}

/// Result of a crews operation.
pub type Result<T> = std::result::Result<T, DomainError>;
