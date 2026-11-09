//! Typed idempotency journal. Its legacy JSON representation belongs to infra.
use super::{
    Appearance, AppearanceEdit, AvatarMeta, Edit, Occurrence, Person, PersonEdit, Preferences,
    PreferencesEdit, Version,
};

/// Original command identity used to reject an altered replay.
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
pub enum Command {
    /// Check-off fingerprint, including date and observed author.
    Mark(String),
    /// Photo replacement fingerprint, including its normalized content digest.
    Avatar(String),
    /// Character change for one routine.
    Appearance(String, AppearanceEdit),
    /// Per-account display preference change.
    Preferences(PreferencesEdit),
    /// Person profile change.
    Person(PersonEdit),
    /// Routine version change.
    Routine(Edit),
}

/// Accepted result returned unchanged when the original command is replayed.
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
pub enum CommandResponse {
    /// Accepted dated task.
    Occurrence(Occurrence),
    /// Accepted routine version.
    Version(Version),
    /// Accepted person profile.
    Person(Person),
    /// Accepted per-account preferences.
    Preferences(Preferences),
    /// Accepted character appearance.
    Appearance(Appearance),
    /// Accepted photo revision and presence.
    Avatar(AvatarMeta),
}
