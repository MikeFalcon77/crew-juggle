//! The shopping gear: the list, receipts and the purchase ledger.
use crate::{
    api, domain,
    domain::{Intelligence, Service, Store},
    infra,
};
use async_trait::async_trait;
use authz_resolver as _;
use authz_resolver_sdk::PolicyEnforcer;
use juggle_crews as _;
use juggle_crews_sdk::{CrewsClientV1, demo, policy::CrewPolicy};
use juggle_routines as _;
use juggle_routines_sdk::RoutinesClientV1;
use oagw as _;
use oagw_sdk::ServiceGatewayClientV1;
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::{Arc, OnceLock};
use tokio_util::sync::CancellationToken;
use toolkit::{DatabaseCapability, Gear, GearCtx, RestApiCapability, api::OpenApiRegistry};
use toolkit_security::AccessScope;

#[toolkit::gear(name="juggle-shopping", deps=[juggle_crews, juggle_routines, authz_resolver, oagw], capabilities=[db,rest,stateful], lifecycle(entry="serve",stop_timeout="30s"))]
/// Owns `js_state`, `js_photos` and `js_ledger` and runs the receipt worker.
#[derive(Default)]
pub struct ShoppingGear {
    service: OnceLock<Arc<Service>>,
    /// Set when receipts are scanned by OpenAI through oagw.
    openai: OnceLock<(
        Arc<dyn ServiceGatewayClientV1>,
        Arc<infra::openai::OpenAiUpstream>,
    )>,
}
#[async_trait]
impl Gear for ShoppingGear {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        let config: crate::config::ShoppingConfig = ctx.config_expanded_or_default()?;
        let crews = ctx.client_hub().get::<dyn CrewsClientV1>()?;
        let routines = ctx.client_hub().get::<dyn RoutinesClientV1>()?;
        // Every request asks authz-resolver; the crews policy plugin answers.
        let policy = CrewPolicy::new(PolicyEnforcer::from_hub(ctx.client_hub()));
        let store = Arc::new(infra::storage::PgStore {
            db: Arc::new(ctx.db_required()?),
        });
        let model: Arc<dyn Intelligence> = if config.scanner_configured() {
            let gateway = ctx.client_hub().get::<dyn ServiceGatewayClientV1>()?;
            let upstream = Arc::new(infra::openai::OpenAiUpstream::new(
                &config.openai_base_url,
                &config.openai_secret_ref,
            )?);
            let scanner = infra::openai::OpenAiScanner::new(
                gateway.clone(),
                upstream.clone(),
                &config.openai_model,
            );
            self.openai
                .set((gateway, upstream))
                .map_err(|_| anyhow::anyhow!("shopping already initialized"))?;
            Arc::new(scanner)
        } else if config.demo {
            tracing::warn!("no OpenAI model configured: receipts use the demo scanner");
            Arc::new(infra::demo_scanner::DemoScanner)
        } else {
            tracing::warn!("no OpenAI model configured: receipt scanning is off");
            Arc::new(infra::no_scanner::NoScanner)
        };
        if config.demo {
            seed_demo(&store).await?;
        }
        self.service
            .set(Arc::new(Service {
                store,
                model,
                fingerprint: Arc::new(infra::storage::codec::JsonFingerprint),
                photos: Arc::new(infra::photos::JpegNormalizer),
                policy,
                crews,
                routines: Some(routines),
            }))
            .map_err(|_| anyhow::anyhow!("shopping already initialized"))?;
        Ok(())
    }
}

/// Demo mode: every start gives the demo crews a fresh shopping list.
///
/// Runs during start-up, before the policy plugins are reachable, and touches
/// only the two demo crews, each through a scope bounded to that crew.
async fn seed_demo(store: &infra::storage::PgStore) -> anyhow::Result<()> {
    for list in domain::demo::LISTS {
        let crew = list.crew;
        let scope = AccessScope::for_tenant(demo::crew_id(crew));
        store.ensure(&scope).await?;
        let (revision, mut state) = store.read(&scope).await?;
        if state.reverse_all("Demo reset")? {
            anyhow::ensure!(
                store.commit(&scope, revision, &state, &[]).await?,
                "demo reset raced"
            );
        }
        store.reset(&scope).await?;
        store.ensure(&scope).await?;
        let (revision, mut state) = store.read(&scope).await?;
        domain::demo::fill(&mut state, &list)?;
        anyhow::ensure!(
            store.commit(&scope, revision, &state, &[]).await?,
            "demo seed raced"
        );
    }
    Ok(())
}
impl ShoppingGear {
    async fn serve(self: Arc<Self>, cancel: CancellationToken) -> anyhow::Result<()> {
        let service = self
            .service
            .get()
            .ok_or_else(|| anyhow::anyhow!("shopping not initialized"))?;
        if let Some((gateway, upstream)) = self.openai.get() {
            let (gateway, upstream, cancel) = (gateway.clone(), upstream.clone(), cancel.clone());
            tokio::spawn(async move { provision_openai(gateway, upstream, cancel).await });
        }
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! { _=cancel.cancelled()=>break, _=interval.tick()=>{} }
            tokio::select! { _=cancel.cancelled()=>break, result=service.tick()=>if result.is_err() { tracing::warn!("shopping worker failed; durable job retained"); } }
        }
        Ok(())
    }
}
/// Creates the OpenAI upstream in oagw, retrying until the key is in credstore.
async fn provision_openai(
    gateway: Arc<dyn ServiceGatewayClientV1>,
    upstream: Arc<infra::openai::OpenAiUpstream>,
    cancel: CancellationToken,
) {
    let mut delay = std::time::Duration::from_millis(500);
    loop {
        match upstream.provision(&gateway).await {
            Ok(()) => {
                tracing::info!("OpenAI upstream ready in oagw");
                return;
            }
            Err(error) => tracing::warn!(%error, "OpenAI upstream not ready yet; retrying"),
        }
        tokio::select! {
            () = cancel.cancelled() => return,
            () = tokio::time::sleep(delay) => {}
        }
        delay = (delay * 2).min(std::time::Duration::from_secs(60));
    }
}

impl DatabaseCapability for ShoppingGear {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        infra::storage::migrations::Migrator::migrations()
    }
}
impl RestApiCapability for ShoppingGear {
    fn register_rest(
        &self,
        _: &GearCtx,
        router: axum::Router,
        registry: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let service = self
            .service
            .get()
            .ok_or_else(|| anyhow::anyhow!("shopping not initialized"))?
            .clone();
        Ok(api::rest::routes::register_routes(
            router, registry, service,
        ))
    }
}
