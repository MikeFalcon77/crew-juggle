//! The `authz-resolver` plugin contract, served by the crew access policy.
use super::service::CrewAccessPolicy;
use async_trait::async_trait;
use authz_resolver_sdk::{
    AuthZResolverError, AuthZResolverPluginClient, EvaluationRequest, EvaluationResponse,
};
use std::sync::Arc;
use toolkit::domain_model;

/// Registered in ClientHub under this plugin's GTS instance id.
#[domain_model]
pub struct PolicyPlugin {
    policy: Arc<CrewAccessPolicy>,
}

impl PolicyPlugin {
    /// The plugin over a policy.
    pub fn new(policy: Arc<CrewAccessPolicy>) -> Self {
        Self { policy }
    }
}

#[async_trait]
impl AuthZResolverPluginClient for PolicyPlugin {
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, AuthZResolverError> {
        Ok(self.policy.evaluate(&request).await)
    }
}
