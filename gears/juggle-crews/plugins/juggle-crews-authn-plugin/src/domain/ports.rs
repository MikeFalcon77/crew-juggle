//! What sign-in needs from the outside world: a way to verify a token.
use async_trait::async_trait;
use juggle_crews_sdk::AccountIdentity;
use toolkit::domain_model;

/// Why a token was not accepted.
///
/// Separate variants for the log. The caller gets one `Unauthorized` with one
/// message for the first two, because telling a refused person *why* only
/// tells them how close they got.
#[domain_model]
#[derive(Debug)]
pub enum Refusal {
    /// Malformed, wrong algorithm, wrong issuer, expired, or badly signed.
    NotAValidToken(String),
    /// Genuine, but without a verified email or outside the allowlist.
    NotAllowed {
        /// The account that was refused.
        uid: String,
        /// Its email, if the token had a verified one.
        email: Option<String>,
    },
    /// The identity provider's keys could not be fetched. Not the caller's
    /// fault, and not something to answer 401 to.
    KeysUnavailable(String),
}

/// Turns a bearer token into a verified account.
#[async_trait]
pub trait IdentityVerifier: Send + Sync {
    /// The account behind `token`, or why it is refused.
    async fn verify(&self, token: &str) -> Result<AccountIdentity, Refusal>;
}
