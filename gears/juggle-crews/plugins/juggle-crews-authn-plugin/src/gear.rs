//! The sign-in plugin gear.
use crate::api::rest::{handlers::SignInState, routes};
use crate::config::{SignInConfig, SignInMode};
use crate::domain::client::SignInPlugin;
use crate::domain::{IdentityVerifier, SignIn};
use crate::infra::{demo_tokens::DemoTokens, firebase::FirebaseVerifier};
use async_trait::async_trait;
use authn_resolver_sdk::{AuthNResolverPluginClient, AuthNResolverPluginSpecV1};
use juggle_crews as _;
use juggle_crews_sdk::CrewDirectoryV1;
use std::sync::{Arc, OnceLock};
use toolkit::api::OpenApiRegistry;
use toolkit::client_hub::ClientScope;
use toolkit::{Gear, GearCtx, RestApiCapability};
use toolkit_gts::PluginV1;
use types_registry_sdk::{RegisterResult, TypesRegistryClient};

/// Registers the sign-in plugin with `authn-resolver` and serves the two
/// anonymous routes the sign-in screen needs.
#[toolkit::gear(name = "juggle-crews-authn-plugin", deps = [types_registry, juggle_crews], capabilities = [rest])]
#[derive(Default)]
pub struct CrewsAuthnPlugin {
    state: OnceLock<SignInState>,
}

#[async_trait]
impl Gear for CrewsAuthnPlugin {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let config: SignInConfig = ctx.config_expanded_or_default()?;
        config.validate()?;
        let (verifier, demo): (Arc<dyn IdentityVerifier>, _) = match config.mode {
            SignInMode::Demo => {
                tracing::warn!("demo sign-in: pick a character, tokens die with the process");
                let tokens = Arc::new(DemoTokens::new()?);
                (tokens.clone(), Some(tokens))
            }
            SignInMode::Firebase => {
                let firebase =
                    FirebaseVerifier::new(config.firebase_project_id.clone(), config.allowlist())?;
                (Arc::new(firebase), None)
            }
        };
        let directory = ctx.client_hub().get::<dyn CrewDirectoryV1>()?;
        let plugin = Arc::new(SignInPlugin::new(Arc::new(SignIn::new(
            verifier, directory,
        ))));

        let (id, registration) = PluginV1::<AuthNResolverPluginSpecV1>::build_registration(
            "cf.juggle.crews_authn.plugin.v1",
            config.vendor.clone(),
            config.priority,
        )?;
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        RegisterResult::ensure_all_ok(&registry.register(vec![registration]).await?)?;
        ctx.client_hub()
            .register_scoped::<dyn AuthNResolverPluginClient>(ClientScope::gts_id(&id), plugin);

        self.state
            .set(SignInState { demo })
            .map_err(|_| anyhow::anyhow!("sign-in plugin already initialized"))?;
        Ok(())
    }
}

impl RestApiCapability for CrewsAuthnPlugin {
    fn register_rest(
        &self,
        _: &GearCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let state = self
            .state
            .get()
            .ok_or_else(|| anyhow::anyhow!("sign-in plugin not initialized"))?
            .clone();
        Ok(routes::register_routes(router, openapi, state))
    }
}
