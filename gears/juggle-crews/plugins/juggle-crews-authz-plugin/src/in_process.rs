//! The crew policy without the platform resolver in between.
//!
//! For tests and tools that run a gear's domain without a server: the
//! enforcer asks this plugin's policy directly instead of `authz-resolver`.
use crate::domain::CrewAccessPolicy;
use crate::domain::client::PolicyPlugin;
use async_trait::async_trait;
use authz_resolver_sdk::{
    AuthZResolverApi, AuthZResolverPluginClient, EvaluationRequest, EvaluationResponse,
    PolicyEnforcer,
};
use juggle_crews_sdk::{CrewDirectoryV1, CrewPolicy};
use std::sync::Arc;
use toolkit_canonical_errors::CanonicalError;
use toolkit_security::PlatformSecurityContext;

struct Direct(PolicyPlugin);

#[async_trait]
impl AuthZResolverApi for Direct {
    async fn evaluate(
        &self,
        _: PlatformSecurityContext,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, CanonicalError> {
        AuthZResolverPluginClient::evaluate(&self.0, request)
            .await
            .map_err(|e| CanonicalError::internal(e.to_string()).create())
    }
}

/// An enforcer that decides with this plugin's rules over `directory`.
pub fn enforcer(directory: Arc<dyn CrewDirectoryV1>) -> PolicyEnforcer {
    let plugin = PolicyPlugin::new(Arc::new(CrewAccessPolicy::new(directory)));
    PolicyEnforcer::new(Arc::new(Direct(plugin)))
}

/// A `CrewPolicy` that decides with this plugin's rules over `directory`.
pub fn crew_policy(directory: Arc<dyn CrewDirectoryV1>) -> CrewPolicy {
    CrewPolicy::new(enforcer(directory))
}
