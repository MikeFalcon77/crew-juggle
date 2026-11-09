//! The policy enforcement point (PEP) every gear with crew data uses.
//!
//! Each gear asks for a decision in every domain operation and hands the
//! returned scope to the secure ORM. The decision itself comes from the
//! platform `authz-resolver`, which routes it to the crews policy plugin.
use authz_resolver_sdk::{EnforcerError, PolicyEnforcer, ResourceType};
use toolkit_security::{AccessScope, SecurityContext, pep_properties};

/// The one resource type crew-juggle gears protect: data owned by a crew.
pub const CREW_DATA: ResourceType = ResourceType::from_static(
    toolkit_gts::gts_id!("cf.juggle.crew.data.v1~"),
    &[pep_properties::OWNER_TENANT_ID],
);

/// The action every crew data operation asks for.
pub const ACCESS: &str = "access";

/// PEP helper shared by the gears that store crew data.
#[derive(Clone)]
pub struct CrewPolicy {
    enforcer: PolicyEnforcer,
}

impl CrewPolicy {
    /// Wraps the platform enforcer, usually `PolicyEnforcer::from_hub(ctx.client_hub())`.
    pub fn new(enforcer: PolicyEnforcer) -> Self {
        Self { enforcer }
    }

    /// Asks whether the caller may access its crew's data and compiles the
    /// answer into a scope for the secure ORM.
    pub async fn scope(&self, ctx: &SecurityContext) -> Result<AccessScope, EnforcerError> {
        self.enforcer
            .access_scope(ctx, &CREW_DATA, ACCESS, None)
            .await
    }
}
