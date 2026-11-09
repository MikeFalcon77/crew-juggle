//! Typed configuration (`gears.juggle-crews-authz-plugin.config`).
use serde::Deserialize;

/// Settings of the policy plugin.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PolicyConfig {
    /// Vendor this plugin registers under; `authz-resolver` picks by vendor.
    pub vendor: String,
    /// Lower wins when several plugins of the vendor are registered.
    pub priority: i16,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            vendor: "constructorfabric".into(),
            priority: 10,
        }
    }
}
