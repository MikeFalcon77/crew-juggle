//! Errors of the crews contract.

/// Why a crews call failed.
#[derive(Debug, thiserror::Error)]
pub enum CrewsError {
    /// The caller is not a member of the crew in its context.
    #[error("not a member of this crew")]
    Forbidden,
    /// The crew does not exist.
    #[error("crew not found")]
    NotFound,
    /// Crew storage cannot answer right now.
    #[error("crew directory unavailable")]
    Unavailable,
}
