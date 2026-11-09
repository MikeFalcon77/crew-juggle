//! Verifying a Firebase ID token and turning it into an identity.
//!
//! # The thing to get right
//!
//! Firebase social login issues a perfectly valid ID token to **anybody** with
//! a Google account. A token that verifies proves who somebody is and says
//! nothing about what they may see. In crew-juggle that decision belongs to
//! crew membership, which the policy decision point checks on every request.
//!
//! A private deployment can additionally restrict who may sign in at all with
//! `allowed_emails` (`JUGGLE_ALLOWED_EMAILS`). Without it, sign-up is open, which is the point of
//! an app that many crews share.
//!
//! An unverified address is always treated as absent: the email is shown to
//! crewmates, so it must belong to the person holding the token.
//!
//! # Which keys
//!
//! Firebase ID tokens are signed by `securetoken@system.gserviceaccount.com`,
//! and the keys live at the JWK endpoint named below. The nearby
//! `oauth2/v3/certs` is a different key set for Google's *OAuth* ID tokens, and
//! using it here would refuse every genuine token with a signature error,
//! which reads as "Firebase is broken" rather than "wrong endpoint".

use std::collections::HashMap;
use std::time::{Duration, Instant};

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;
use toolkit_http::HttpClient;
use tracing::{debug, warn};

use crate::domain::{IdentityVerifier, Refusal};
use async_trait::async_trait;
use juggle_crews_sdk::AccountIdentity;

/// Google's public keys for Firebase ID tokens, in JWK form.
const JWK_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";

/// Used only when the response carries no usable `Cache-Control: max-age`.
/// Google rotates daily and always sends one; this is the floor under a
/// malformed header, not the normal path.
const FALLBACK_KEY_TTL: Duration = Duration::from_hours(1);

/// The claims worth reading. Everything else Firebase sends is either checked
/// by the library or of no interest here.
#[derive(Debug, Deserialize)]
struct Claims {
    /// The Firebase user id. Stable across email changes and sign-in methods,
    /// which is why memberships are keyed on it rather than on an address.
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    /// Google can put an address on the token before the owner has confirmed
    /// it. An unverified claim is treated as absent.
    #[serde(default)]
    email_verified: bool,
}

struct CachedKeys {
    keys: JwkSet,
    fetched_at: Instant,
    ttl: Duration,
}

impl CachedKeys {
    fn is_fresh(&self) -> bool {
        self.fetched_at.elapsed() < self.ttl
    }
}

/// Verifies Firebase ID tokens against Google's published keys.
pub struct FirebaseVerifier {
    project_id: String,
    /// Lower-case addresses allowed to sign in; `None` means open sign-up.
    allowlist: Option<Vec<String>>,
    /// `None` only in tests, which verify against a key of their own and never
    /// reach Google. A missing client at runtime would be a construction bug,
    /// and it refuses rather than panicking.
    http: Option<HttpClient>,
    keys: RwLock<Option<CachedKeys>>,
}

#[derive(Debug, thiserror::Error)]
/// Why the verifier could not be built.
pub enum FirebaseInitError {
    /// The HTTP client for Google's keys failed to build.
    #[error("building the Firebase key client")]
    Http(#[source] Box<dyn std::error::Error + Send + Sync>),
}

impl FirebaseVerifier {
    /// Must be built inside a Tokio runtime: the HTTP client spawns a tower
    /// worker. `Gear::init` is async, which is where this happens.
    ///
    /// # Errors
    ///
    /// When the HTTP client cannot be built.
    pub fn new(
        project_id: String,
        allowlist: Option<Vec<String>>,
    ) -> Result<Self, FirebaseInitError> {
        let http = HttpClient::builder()
            .with_otel()
            .build()
            .map_err(|e| FirebaseInitError::Http(Box::new(e)))?;
        Ok(Self {
            project_id,
            allowlist: allowlist.map(normalize),
            http: Some(http),
            keys: RwLock::new(None),
        })
    }

    /// A verifier that cannot fetch keys. Used to pin the 503 mapping for a
    /// key-fetch outage, and by unit tests that inject a signing key of their
    /// own.
    #[must_use]
    pub fn without_http(project_id: String, allowlist: Option<Vec<String>>) -> Self {
        Self {
            project_id,
            allowlist: allowlist.map(normalize),
            http: None,
            keys: RwLock::new(None),
        }
    }

    /// The identity behind a Firebase ID token.
    ///
    /// # Errors
    ///
    /// [`Refusal`] for every reason the token is not accepted, including the
    /// one that is not the caller's fault.
    pub async fn verify_token(&self, token: &str) -> Result<AccountIdentity, Refusal> {
        let header =
            decode_header(token).map_err(|e| Refusal::NotAValidToken(format!("header: {e}")))?;

        // Pinned rather than taken from the token. `alg` is attacker-supplied,
        // and honouring it is how a token signed with `none`, or with the
        // public key as an HMAC secret, gets accepted.
        if header.alg != Algorithm::RS256 {
            return Err(Refusal::NotAValidToken(format!(
                "algorithm {:?}, expected RS256",
                header.alg
            )));
        }
        let kid = header
            .kid
            .ok_or_else(|| Refusal::NotAValidToken("no key id".to_owned()))?;

        let key = self.key_for(&kid).await?;
        self.check(token, &key)
    }

    /// Everything after the signing key has been found.
    ///
    /// Separated so it can be tested against a key of our own. Fetching
    /// Google's would make every one of those tests a network call, and the
    /// rules that matter here -- audience, issuer, expiry, verified email and
    /// the optional allowlist -- have nothing to do with where the key came from.
    fn check(&self, token: &str, key: &DecodingKey) -> Result<AccountIdentity, Refusal> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.project_id]);
        validation.set_issuer(&[format!(
            "https://securetoken.google.com/{}",
            self.project_id
        )]);
        // `sub` keys every membership, so a token without one must never be
        // accepted as an empty identity.
        validation.required_spec_claims = ["exp", "iat", "aud", "iss", "sub"]
            .iter()
            .map(|c| (*c).to_owned())
            .collect();

        let data = decode::<Claims>(token, key, &validation)
            .map_err(|e| Refusal::NotAValidToken(e.to_string()))?;

        let uid = data.claims.sub;
        if uid.is_empty() {
            return Err(Refusal::NotAValidToken("empty subject".to_owned()));
        }

        let email = data
            .claims
            .email
            .filter(|value| !value.trim().is_empty() && data.claims.email_verified);
        let Some(email) = email else {
            return Err(Refusal::NotAllowed { uid, email: None });
        };
        if self
            .allowlist
            .as_ref()
            .is_some_and(|list| !list.contains(&email.to_ascii_lowercase()))
        {
            return Err(Refusal::NotAllowed {
                uid,
                email: Some(email),
            });
        }

        Ok(AccountIdentity {
            uid,
            email,
            name: data.claims.name.filter(|n| !n.trim().is_empty()),
        })
    }

    /// One signing key, from cache when the cache is still good.
    async fn key_for(&self, kid: &str) -> Result<DecodingKey, Refusal> {
        if let Some(key) = self.cached_key(kid).await? {
            return Ok(key);
        }

        self.refresh_keys().await?;

        // A `kid` that is still missing after a fresh fetch is a property of the
        // token, not of the cache -- so this is a refusal rather than a retry.
        self.cached_key(kid).await?.ok_or_else(|| {
            Refusal::NotAValidToken(format!("no published key matches key id {kid}"))
        })
    }

    async fn cached_key(&self, kid: &str) -> Result<Option<DecodingKey>, Refusal> {
        let guard = self.keys.read().await;
        let Some(cached) = guard.as_ref() else {
            return Ok(None);
        };
        if !cached.is_fresh() {
            return Ok(None);
        }
        let Some(jwk) = cached.keys.find(kid) else {
            return Ok(None);
        };
        DecodingKey::from_jwk(jwk).map(Some).map_err(|e| {
            warn!(error = %e, %kid, "firebase jwk could not be decoded");
            Refusal::KeysUnavailable(format!("jwk {kid}: {e}"))
        })
    }

    async fn refresh_keys(&self) -> Result<(), Refusal> {
        let http = self
            .http
            .as_ref()
            .ok_or_else(|| Refusal::KeysUnavailable("no http client".to_owned()))?;

        let response = http
            .get(JWK_URL)
            .send()
            .await
            .map_err(|e| Refusal::KeysUnavailable(e.to_string()))?;

        let ttl = max_age(response.headers()).unwrap_or(FALLBACK_KEY_TTL);

        let body = response
            .bytes()
            .await
            .map_err(|e| Refusal::KeysUnavailable(e.to_string()))?;

        let keys: JwkSet = serde_json::from_slice(&body)
            .map_err(|e| Refusal::KeysUnavailable(format!("key set: {e}")))?;

        debug!(
            count = keys.keys.len(),
            ttl_secs = ttl.as_secs(),
            "fetched Google signing keys"
        );

        *self.keys.write().await = Some(CachedKeys {
            keys,
            fetched_at: Instant::now(),
            ttl,
        });
        Ok(())
    }
}

/// How long Google says its keys are good for.
///
/// Honoured rather than assumed: the keys rotate daily, and caching past the
/// window means refusing genuine tokens for however long the cache outlives the
/// key -- a failure that looks like everybody being logged out at once.
fn max_age(headers: &http::HeaderMap) -> Option<Duration> {
    let value = headers.get(http::header::CACHE_CONTROL)?.to_str().ok()?;
    let directives: HashMap<&str, &str> = value
        .split(',')
        .filter_map(|part| part.trim().split_once('='))
        .collect();
    let seconds: u64 = directives.get("max-age")?.trim().parse().ok()?;
    if seconds == 0 {
        warn!("google returned max-age=0 for its signing keys");
        return None;
    }
    Some(Duration::from_secs(seconds))
}

#[async_trait]
impl IdentityVerifier for FirebaseVerifier {
    async fn verify(&self, token: &str) -> Result<AccountIdentity, Refusal> {
        self.verify_token(token).await
    }
}

#[cfg(test)]
#[path = "firebase_tests.rs"]
mod firebase_tests;

fn normalize(list: Vec<String>) -> Vec<String> {
    list.into_iter()
        .map(|e| e.trim().to_ascii_lowercase())
        .filter(|e| !e.is_empty())
        .collect()
}
