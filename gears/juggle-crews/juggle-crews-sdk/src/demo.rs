//! The demo cast, shared by every gear that seeds demo data.
//!
//! Demo mode is a local sandbox: sign-in is a character picker and data resets
//! on restart. Each gear turns it on in its own config (`config/demo.yaml`).
use uuid::Uuid;

/// One demo crew.
pub struct DemoCrew {
    /// Stable key, used to derive the crew id.
    pub key: &'static str,
    /// Display name.
    pub name: &'static str,
}

/// One demo account. People without accounts (Mochi the cat) are seeded by
/// the routines gear, because they exist only there.
pub struct DemoUser {
    /// Character key, also the demo sign-in name (`pip`, `zed`, ...).
    pub key: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Key of the character's crew.
    pub crew: &'static str,
    /// Whether the character created the crew.
    pub owner: bool,
    /// One line for the sign-in picker.
    pub tagline: &'static str,
}

/// The demo crews.
pub const CREWS: [DemoCrew; 2] = [
    DemoCrew {
        key: "flat-4b",
        name: "Flat 4B",
    },
    DemoCrew {
        key: "robotics-club",
        name: "Robotics Club",
    },
];

/// The demo characters that can sign in.
pub const USERS: [DemoUser; 4] = [
    DemoUser {
        key: "pip",
        name: "Pip",
        crew: "flat-4b",
        owner: true,
        tagline: "Plans everything. Started this app.",
    },
    DemoUser {
        key: "zed",
        name: "Zed",
        crew: "flat-4b",
        owner: false,
        tagline: "Builds robots in the kitchen. Allergic to dishes.",
    },
    DemoUser {
        key: "nova",
        name: "Nova",
        crew: "robotics-club",
        owner: true,
        tagline: "Captain of the Robotics Club.",
    },
    DemoUser {
        key: "ike",
        name: "Ike",
        crew: "robotics-club",
        owner: false,
        tagline: "Solders things. Snack enthusiast.",
    },
];

/// Stable crew id of a demo crew.
pub fn crew_id(key: &str) -> Uuid {
    Uuid::new_v5(
        &Uuid::NAMESPACE_URL,
        format!("crew-juggle/demo/crew/{key}").as_bytes(),
    )
}

/// Account id of a demo character.
pub fn uid(key: &str) -> String {
    format!("demo-{key}")
}

#[cfg(test)]
#[path = "demo_tests.rs"]
mod demo_tests;
