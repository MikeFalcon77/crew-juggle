//! The crews contract that other gears consume through ClientHub.
//!
//! A crew is a tenant: a flat, a team, a club. This crate carries no ORM, HTTP
//! or token types, so consumers depend on the contract, not on the gear.
#![forbid(unsafe_code)]

pub mod api;
pub mod demo;
pub mod errors;
pub mod identity;
pub mod models;
pub mod policy;

pub use api::{CrewDirectoryV1, CrewsClientV1};
pub use errors::CrewsError;
pub use identity::{
    PLATFORM_TENANT, integration_context, integration_subject, is_worker, lobby_tenant,
    secrets_provisioner_context, secrets_provisioner_subject, subject_for, worker_context,
    worker_subject,
};
pub use models::{AccountIdentity, AccountMembership, CrewInfo, CrewMember, MembershipKind, Role};
pub use policy::{CREW_DATA, CrewPolicy};
