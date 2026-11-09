//! Typed configuration of the shopping gear (`gears.juggle-shopping.config`).
use serde::Deserialize;

/// Settings of the shopping gear.
#[derive(Debug, Clone, Deserialize, toolkit::ExpandVars)]
#[serde(default, deny_unknown_fields)]
pub struct ShoppingConfig {
    /// Give the demo crews a fresh shopping list on every start.
    pub demo: bool,
    /// Vision model with structured outputs (`${OPENAI_MODEL:-}`); empty
    /// uses the demo scanner. The key is not here: see `juggle-secrets`.
    #[expand_vars]
    pub openai_model: String,
    /// Base URL of the OpenAI API. Its host is also the upstream's alias in oagw.
    pub openai_base_url: String,
    /// Credstore reference of the OpenAI key.
    pub openai_secret_ref: String,
}

impl Default for ShoppingConfig {
    fn default() -> Self {
        Self {
            demo: false,
            openai_model: String::new(),
            openai_base_url: "https://api.openai.com".into(),
            openai_secret_ref: "openai-api-key".into(),
        }
    }
}

impl ShoppingConfig {
    /// Receipts go to OpenAI when a model is configured.
    pub fn scanner_configured(&self) -> bool {
        !self.openai_model.trim().is_empty()
    }
}
