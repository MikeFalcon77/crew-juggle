//! The `authn-resolver` plugin contract, served by the sign-in use case.
use super::service::{SignIn, SignInError};
use async_trait::async_trait;
use authn_resolver_sdk::{
    AuthNResolverError, AuthNResolverPluginClient, AuthenticationResult, ClientCredentialsRequest,
};
use std::sync::Arc;
use toolkit::domain_model;
use toolkit_security::SecurityContext;

/// Registered in ClientHub under this plugin's GTS instance id.
#[domain_model]
pub struct SignInPlugin {
    sign_in: Arc<SignIn>,
}

impl SignInPlugin {
    /// The plugin over a sign-in use case.
    pub fn new(sign_in: Arc<SignIn>) -> Self {
        Self { sign_in }
    }
}

#[async_trait]
impl AuthNResolverPluginClient for SignInPlugin {
    async fn authenticate(&self, token: &str) -> Result<AuthenticationResult, AuthNResolverError> {
        let membership = self.sign_in.sign_in(token).await.map_err(|e| match e {
            SignInError::Refused => AuthNResolverError::Unauthorized(e.to_string()),
            SignInError::Unavailable => AuthNResolverError::ServiceUnavailable(e.to_string()),
        })?;
        // Accounts are always members or guests. Only the internal worker is a
        // "service", and it never signs in.
        let security_context = SecurityContext::builder()
            .subject_id(membership.subject_id)
            .subject_tenant_id(membership.tenant_id)
            .subject_type(membership.kind.subject_type())
            .build()
            .map_err(|e| AuthNResolverError::Internal(e.to_string()))?;
        Ok(AuthenticationResult { security_context })
    }

    async fn exchange_client_credentials(
        &self,
        _: &ClientCredentialsRequest,
    ) -> Result<AuthenticationResult, AuthNResolverError> {
        Err(AuthNResolverError::TokenAcquisitionFailed(
            "sign in with Google or pick a demo character".into(),
        ))
    }
}
