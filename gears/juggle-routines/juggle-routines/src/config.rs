//! Typed configuration of the routines gear (`gears.juggle-routines.config`).
use serde::Deserialize;

/// Settings of the routines gear.
#[derive(Debug, Clone, Default, Deserialize, toolkit::ExpandVars)]
#[serde(default, deny_unknown_fields)]
pub struct RoutinesConfig {
    /// Rebuild the demo crews' routines on every start.
    pub demo: bool,
    /// A production deployment refuses throwaway push keys and demo mode.
    pub production: bool,
    /// VAPID private key, base64url (`${VAPID_PRIVATE_KEY:-}`).
    #[expand_vars]
    pub vapid_private_key: String,
    /// VAPID public key, base64url (`${VAPID_PUBLIC_KEY:-}`).
    #[expand_vars]
    pub vapid_public_key: String,
    /// `mailto:` or `https:` contact for push services (`${VAPID_SUBJECT:-}`).
    #[expand_vars]
    pub vapid_subject: String,
}

impl RoutinesConfig {
    /// Demo mode and production never go together.
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            !(self.demo && self.production),
            "juggle-routines: demo mode cannot run in a production deployment"
        );
        Ok(())
    }
}
