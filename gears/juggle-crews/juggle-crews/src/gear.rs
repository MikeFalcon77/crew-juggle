//! The crews gear: crews, members, invite codes and the crews contract.
use crate::api::rest::routes;
use crate::config::CrewsConfig;
use crate::domain::CrewsService;
use crate::domain::local_client::LocalClient;
use crate::infra::storage::{SeaOrmCrewRepo, migrations::Migrator};
use async_trait::async_trait;
use juggle_crews_sdk::{CrewDirectoryV1, CrewsClientV1};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::{Arc, OnceLock};
use toolkit::api::OpenApiRegistry;
use toolkit::{DatabaseCapability, Gear, GearCtx, RestApiCapability};

/// Owns `cj_crews` and `cj_members` and answers `CrewsClientV1` and
/// `CrewDirectoryV1` for the other gears. Sign-in and the access policy live
/// in the two plugins next to it (`plugins/`).
#[toolkit::gear(name = "juggle-crews", capabilities = [db, rest])]
#[derive(Default)]
pub struct CrewsGear {
    service: OnceLock<Arc<CrewsService>>,
}

#[async_trait]
impl Gear for CrewsGear {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let config: CrewsConfig = ctx.config_expanded_or_default()?;
        config.validate()?;
        let repo = Arc::new(SeaOrmCrewRepo::new(Arc::new(ctx.db_required()?)));
        let service = Arc::new(CrewsService::new(repo));
        if config.demo {
            service.seed_demo(&config.demo_timezone).await?;
            tracing::warn!("demo mode: demo crews are rebuilt on every start");
        }
        let client = Arc::new(LocalClient::new(service.clone()));
        ctx.client_hub()
            .register::<dyn CrewsClientV1>(client.clone());
        ctx.client_hub().register::<dyn CrewDirectoryV1>(client);
        self.service
            .set(service)
            .map_err(|_| anyhow::anyhow!("crews already initialized"))?;
        Ok(())
    }
}

impl DatabaseCapability for CrewsGear {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        Migrator::migrations()
    }
}

impl RestApiCapability for CrewsGear {
    fn register_rest(
        &self,
        _: &GearCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let service = self
            .service
            .get()
            .ok_or_else(|| anyhow::anyhow!("crews not initialized"))?
            .clone();
        Ok(routes::register_routes(router, openapi, service))
    }
}
