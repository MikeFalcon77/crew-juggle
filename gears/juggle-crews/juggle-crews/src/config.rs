//! Typed configuration of the crews gear (`gears.juggle-crews.config`).
use serde::Deserialize;

/// Settings of the crews gear.
#[derive(Debug, Clone, Deserialize, toolkit::ExpandVars)]
#[serde(default, deny_unknown_fields)]
pub struct CrewsConfig {
    /// Rebuild the demo crews (Flat 4B, Robotics Club) on every start.
    pub demo: bool,
    /// Calendar zone of the demo crews, e.g. `${JUGGLE_DEMO_TIMEZONE:-Europe/Berlin}`.
    #[expand_vars]
    pub demo_timezone: String,
}

impl Default for CrewsConfig {
    fn default() -> Self {
        Self {
            demo: false,
            demo_timezone: "Europe/Berlin".into(),
        }
    }
}

impl CrewsConfig {
    /// Refuses a demo zone that is not an IANA time zone.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.demo && self.demo_timezone.trim().parse::<chrono_tz::Tz>().is_err() {
            anyhow::bail!("juggle-crews: demo_timezone is not a valid time zone");
        }
        Ok(())
    }
}
