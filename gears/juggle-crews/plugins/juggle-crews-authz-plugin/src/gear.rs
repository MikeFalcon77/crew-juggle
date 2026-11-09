//! The policy plugin gear.
use crate::config::PolicyConfig;
use crate::domain::CrewAccessPolicy;
use crate::domain::client::PolicyPlugin;
use async_trait::async_trait;
use authz_resolver_sdk::{AuthZResolverPluginClient, AuthZResolverPluginSpecV1};
use juggle_crews as _;
use juggle_crews_sdk::CrewDirectoryV1;
use std::sync::Arc;
use toolkit::client_hub::ClientScope;
use toolkit::{Gear, GearCtx};
use toolkit_gts::PluginV1;
use types_registry_sdk::{RegisterResult, TypesRegistryClient};

/// Registers the crew access policy with `authz-resolver`.
#[toolkit::gear(name = "juggle-crews-authz-plugin", deps = [types_registry, juggle_crews])]
#[derive(Default)]
pub struct CrewsAuthzPlugin;

#[async_trait]
impl Gear for CrewsAuthzPlugin {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let config: PolicyConfig = ctx.config_or_default()?;
        let directory = ctx.client_hub().get::<dyn CrewDirectoryV1>()?;
        let plugin = Arc::new(PolicyPlugin::new(Arc::new(CrewAccessPolicy::new(
            directory,
        ))));
        let (id, registration) = PluginV1::<AuthZResolverPluginSpecV1>::build_registration(
            "cf.juggle.crews_authz.plugin.v1",
            config.vendor,
            config.priority,
        )?;
        let registry = ctx.client_hub().get::<dyn TypesRegistryClient>()?;
        RegisterResult::ensure_all_ok(&registry.register(vec![registration]).await?)?;
        ctx.client_hub()
            .register_scoped::<dyn AuthZResolverPluginClient>(ClientScope::gts_id(&id), plugin);
        Ok(())
    }
}
