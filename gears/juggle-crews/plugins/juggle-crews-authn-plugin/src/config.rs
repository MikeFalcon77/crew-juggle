//! Typed configuration (`gears.juggle-crews-authn-plugin.config`).
use serde::Deserialize;

/// How people sign in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignInMode {
    /// Google accounts through Firebase Authentication.
    #[default]
    Firebase,
    /// A character picker; tokens die with the process. Never in production.
    Demo,
}

/// Settings of the sign-in plugin.
#[derive(Debug, Clone, Deserialize, toolkit::ExpandVars)]
#[serde(default, deny_unknown_fields)]
pub struct SignInConfig {
    /// `firebase` or `demo`.
    pub mode: SignInMode,
    /// Firebase project whose ID tokens are accepted, e.g. `${FIREBASE_PROJECT_ID}`.
    #[expand_vars]
    pub firebase_project_id: String,
    /// Comma-separated addresses allowed to sign in; empty means open sign-up.
    #[expand_vars]
    pub allowed_emails: String,
    /// Vendor this plugin registers under; `authn-resolver` picks by vendor.
    pub vendor: String,
    /// Lower wins when several plugins of the vendor are registered.
    pub priority: i16,
}

impl Default for SignInConfig {
    fn default() -> Self {
        Self {
            mode: SignInMode::default(),
            firebase_project_id: String::new(),
            allowed_emails: String::new(),
            vendor: "constructorfabric".into(),
            priority: 10,
        }
    }
}

impl SignInConfig {
    /// Firebase mode needs a project.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.mode == SignInMode::Firebase && self.firebase_project_id.trim().is_empty() {
            anyhow::bail!(
                "juggle-crews-authn-plugin: set firebase_project_id (FIREBASE_PROJECT_ID), \
                 or start with config/demo.yaml for a local demo"
            );
        }
        Ok(())
    }

    /// The allowlist as a list, or `None` for open sign-up.
    pub fn allowlist(&self) -> Option<Vec<String>> {
        let list: Vec<String> = self
            .allowed_emails
            .split(',')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(str::to_owned)
            .collect();
        (!list.is_empty()).then_some(list)
    }
}
