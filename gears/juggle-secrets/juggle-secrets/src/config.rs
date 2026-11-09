//! Typed configuration (`gears.juggle-secrets.config`).
use serde::Deserialize;

/// One secret to provision.
#[derive(Debug, Clone, Deserialize, toolkit::ExpandVars)]
#[serde(deny_unknown_fields)]
pub struct SecretEntry {
    /// Credstore reference, e.g. `openai-api-key`.
    pub key: String,
    /// The value, usually `${OPENAI_API_KEY:-}`. Empty values are skipped.
    #[expand_vars]
    pub value: String,
}

/// Settings of the secrets gear.
#[derive(Debug, Clone, Default, Deserialize, toolkit::ExpandVars)]
#[serde(default, deny_unknown_fields)]
pub struct SecretsConfig {
    /// Secrets to write on every start.
    #[expand_vars]
    pub secrets: Vec<SecretEntry>,
}
