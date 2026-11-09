use crate::{api, domain, infra};
use async_trait::async_trait;
use authz_resolver as _;
use authz_resolver_sdk::PolicyEnforcer;
use juggle_crews as _;
use juggle_crews_sdk::{CrewsClientV1, demo, policy::CrewPolicy, worker_context};
use sea_orm_migration::{MigrationTrait, MigratorTrait};
use std::sync::{Arc, OnceLock};
use tokio_util::sync::CancellationToken;
use toolkit::api::OpenApiRegistry;
use toolkit::{DatabaseCapability, Gear, GearCtx, RestApiCapability};
use toolkit_security::AccessScope;

#[toolkit::gear(name = "juggle-routines", deps = [juggle_crews, authz_resolver], capabilities = [db, rest, stateful], lifecycle(entry = "serve", stop_timeout = "30s"))]
#[derive(Default)]
pub struct RoutinesGear {
    app: OnceLock<Arc<api::rest::handlers::App>>,
}
#[async_trait]
impl Gear for RoutinesGear {
    async fn init(&self, ctx: &GearCtx) -> anyhow::Result<()> {
        // Crew name, zone and members come from the crews gear, never from here.
        let config: crate::config::RoutinesConfig = ctx.config_expanded_or_default()?;
        config.validate()?;
        let crews = ctx.client_hub().get::<dyn CrewsClientV1>()?;
        // Every request asks authz-resolver; the crews policy plugin answers.
        let policy = CrewPolicy::new(PolicyEnforcer::from_hub(ctx.client_hub()));
        let store = Arc::new(infra::storage::PgStore::new(Arc::new(ctx.db_required()?)));
        if config.demo {
            seed_demo(&store, crews.as_ref()).await?;
        }
        let push = infra::push::Push::new(&config)?;
        let vapid_public_key = push.public_key.clone();
        let application = Arc::new(domain::application::Application {
            service: domain::Service {
                store: store.clone(),
                policy,
                crews,
            },
            avatars: store,
            normalizer: Arc::new(infra::avatars::PngNormalizer),
            push: Arc::new(push),
            vapid_public_key,
        });
        ctx.client_hub()
            .register::<dyn juggle_routines_sdk::RoutinesClientV1>(application.clone());
        let app = Arc::new(api::rest::handlers::App { application });
        self.app
            .set(app)
            .map_err(|_| anyhow::anyhow!("already initialized"))?;
        Ok(())
    }
}

/// Demo mode: every start rebuilds the demo crews' routines from scratch.
///
/// Runs during start-up, before the policy plugins are reachable, and touches
/// only the two demo crews, each through a scope bounded to that crew.
async fn seed_demo(
    store: &infra::storage::PgStore,
    crews: &dyn CrewsClientV1,
) -> anyhow::Result<()> {
    for crew in demo::CREWS {
        let ctx = worker_context(demo::crew_id(crew.key));
        let scope = AccessScope::for_tenant(demo::crew_id(crew.key));
        let info = crews.crew(&ctx).await?;
        let members = crews.members(&ctx).await?;
        let zone = domain::parse_zone(&info.timezone)?;
        if let Some(state) = domain::demo::state(crew.key, &members, zone, chrono::Utc::now())? {
            store.replace(&scope, &state).await?;
            seed_avatars(store, &scope, &state).await?;
        }
    }
    Ok(())
}

/// Demo faces, drawn for the cast and rendered by `web/scripts/icons.mjs`.
const DEMO_AVATARS: [(&str, &[u8]); 5] = [
    ("pip", include_bytes!("../demo-avatars/pip.png")),
    ("zed", include_bytes!("../demo-avatars/zed.png")),
    ("mochi", include_bytes!("../demo-avatars/mochi.png")),
    ("nova", include_bytes!("../demo-avatars/nova.png")),
    ("ike", include_bytes!("../demo-avatars/ike.png")),
];

async fn seed_avatars(
    store: &infra::storage::PgStore,
    scope: &toolkit_db::secure::AccessScope,
    state: &domain::State,
) -> anyhow::Result<()> {
    use domain::AvatarStore;
    for (key, png) in DEMO_AVATARS {
        let person = if key == "mochi" {
            state.people.iter().find(|p| p.id == "mochi")
        } else {
            let uid = demo::uid(key);
            state
                .people
                .iter()
                .find(|p| p.account.as_deref() == Some(uid.as_str()))
        };
        let Some(person) = person else { continue };
        store
            .change(
                scope,
                &person.id,
                "demo-seed",
                &domain::AvatarEdit {
                    operation_id: uuid::Uuid::new_v4(),
                    expected_revision: 0,
                },
                Some(infra::avatars::normalize(png)?),
            )
            .await?;
    }
    Ok(())
}
impl RoutinesGear {
    /// Runs reminder processing until the managed Gear cancellation signal.
    pub(crate) async fn serve(self: Arc<Self>, cancel: CancellationToken) -> anyhow::Result<()> {
        let app = self
            .app
            .get()
            .ok_or_else(|| anyhow::anyhow!("not initialized"))?;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                _ = cancel.cancelled() => break,
                _ = interval.tick() => if let Err(error) = domain::reminders::tick(&app.service, app.push.as_ref(), &cancel).await { tracing::error!(%error, "reminder tick failed"); }
            }
        }
        Ok(())
    }
}
impl DatabaseCapability for RoutinesGear {
    fn migrations(&self) -> Vec<Box<dyn MigrationTrait>> {
        infra::storage::migrations::Migrator::migrations()
    }
}
impl RestApiCapability for RoutinesGear {
    fn register_rest(
        &self,
        _: &GearCtx,
        router: axum::Router,
        openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        let app = self
            .app
            .get()
            .ok_or_else(|| anyhow::anyhow!("not initialized"))?
            .clone();
        Ok(api::rest::routes::register_routes(router, openapi, app))
    }
}
