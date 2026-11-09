//! Sign-in tokens for the demo character picker.
//!
//! A token is `demo.<user>.<hmac>` signed with a random per-process key, so it
//! dies with the process and is never valid on another server. The AuthN
//! plugin accepts these tokens only while demo mode is on, and demo mode
//! is switched on only by `config/demo.yaml`.
use crate::domain::{IdentityVerifier, Refusal};
use async_trait::async_trait;
use aws_lc_rs::{hmac, rand::SecureRandom, rand::SystemRandom};
use juggle_crews_sdk::AccountIdentity;
use juggle_crews_sdk::demo;

/// Mints and checks demo tokens with a key that lives only in this process.
pub struct DemoTokens {
    key: hmac::Key,
}

impl DemoTokens {
    /// A fresh random key.
    pub fn new() -> anyhow::Result<Self> {
        let mut secret = [0u8; 32];
        SystemRandom::new()
            .fill(&mut secret)
            .map_err(|_| anyhow::anyhow!("no system randomness for demo tokens"))?;
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, &secret),
        })
    }

    /// Mints a token for one of the demo characters.
    pub fn mint(&self, user: &str) -> Option<String> {
        demo::USERS.iter().find(|u| u.key == user)?;
        let tag = hmac::sign(&self.key, user.as_bytes());
        Some(format!("demo.{user}.{}", hex(tag.as_ref())))
    }

    /// The demo identity behind a token, or `None` for anything else.
    pub fn identity_of(&self, token: &str) -> Option<AccountIdentity> {
        let mut parts = token.splitn(3, '.');
        if parts.next()? != "demo" {
            return None;
        }
        let user = parts.next()?;
        let tag = unhex(parts.next()?)?;
        hmac::verify(&self.key, user.as_bytes(), &tag).ok()?;
        let cast = demo::USERS.iter().find(|u| u.key == user)?;
        Some(AccountIdentity {
            uid: demo::uid(cast.key),
            email: format!("{}@demo.crew-juggle.test", cast.key),
            name: Some(cast.name.into()),
        })
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() != 64 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

#[async_trait]
impl IdentityVerifier for DemoTokens {
    async fn verify(&self, token: &str) -> Result<AccountIdentity, Refusal> {
        self.identity_of(token)
            .ok_or_else(|| Refusal::NotAValidToken("not a demo token of this server".into()))
    }
}

#[cfg(test)]
#[path = "demo_tokens_tests.rs"]
mod demo_tokens_tests;
