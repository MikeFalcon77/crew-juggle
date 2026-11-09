//! Invite codes: eight characters from an alphabet without look-alikes.
use super::error::{DomainError, Result};
use aws_lc_rs::rand::{SecureRandom, SystemRandom};

/// Characters of an invite code; no 0/O, 1/I/L.
pub(crate) const CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Invite codes look like `K7QX-M2PA`; spaces, dashes and case are ignored.
pub fn normalize_code(code: &str) -> String {
    code.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// The code as people type and share it: `K7QX-M2PA`.
pub fn display_code(code: &str) -> String {
    match code.get(..4).zip(code.get(4..)) {
        Some((head, tail)) => format!("{head}-{tail}"),
        None => code.to_owned(),
    }
}

/// A fresh random code.
pub fn new_code() -> Result<String> {
    let mut bytes = [0u8; 8];
    SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| DomainError::Unavailable("no system randomness".into()))?;
    Ok(bytes
        .iter()
        .map(|b| CODE_ALPHABET[usize::from(*b) % CODE_ALPHABET.len()] as char)
        .collect())
}

#[cfg(test)]
#[path = "invite_tests.rs"]
mod invite_tests;
