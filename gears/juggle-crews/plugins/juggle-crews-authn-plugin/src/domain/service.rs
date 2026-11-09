//! Sign-in as a use case.
use super::ports::{IdentityVerifier, Refusal};
use juggle_crews_sdk::{AccountMembership, CrewDirectoryV1};
use std::sync::Arc;
use toolkit::domain_model;

/// Tokens longer than this are refused before any parsing.
const MAX_TOKEN_BYTES: usize = 8192;

/// Why sign-in did not produce a membership.
#[domain_model]
#[derive(Debug, thiserror::Error)]
pub enum SignInError {
    /// The token is not acceptable; the reason is in the log only.
    #[error("account not permitted")]
    Refused,
    /// The identity provider or the crews gear cannot answer right now.
    #[error("sign-in unavailable")]
    Unavailable,
}

/// Verifies a token and asks the crews directory where the account lands.
#[domain_model]
pub struct SignIn {
    verifier: Arc<dyn IdentityVerifier>,
    directory: Arc<dyn CrewDirectoryV1>,
}

impl SignIn {
    /// Sign-in with one way to verify tokens.
    pub fn new(verifier: Arc<dyn IdentityVerifier>, directory: Arc<dyn CrewDirectoryV1>) -> Self {
        Self {
            verifier,
            directory,
        }
    }

    /// The membership of the account behind `token`.
    pub async fn sign_in(&self, token: &str) -> Result<AccountMembership, SignInError> {
        if token.len() > MAX_TOKEN_BYTES {
            return Err(SignInError::Refused);
        }
        let identity = self.verifier.verify(token).await.map_err(|refusal| {
            match refusal {
                Refusal::KeysUnavailable(reason) => {
                    tracing::warn!(%reason, "identity provider keys unavailable");
                    return SignInError::Unavailable;
                }
                Refusal::NotAValidToken(reason) => tracing::debug!(%reason, "token refused"),
                Refusal::NotAllowed { uid, .. } => {
                    tracing::info!(%uid, "account outside the allowed list");
                }
            }
            SignInError::Refused
        })?;
        self.directory.sign_in(identity).await.map_err(|error| {
            tracing::warn!(%error, "crews directory unavailable during sign-in");
            SignInError::Unavailable
        })
    }
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod service_tests;
