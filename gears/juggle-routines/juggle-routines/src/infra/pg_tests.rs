//! The routines storage on a real PostgreSQL (`make test-integration`).
//!
//! Everything runs in a fresh schema that is dropped afterwards, so the check
//! never reads or writes a real crew.
use crate::{
    domain::{self, AvatarEdit, AvatarStore, Error, PersonEdit, State, Store},
    infra::storage::{PgStore, migrations::Migrator},
};
use chrono::Utc;
use sea_orm::{ConnectionTrait, Database};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use toolkit_db::{ConnectOpts, DBProvider, secure::AccessScope};
use uuid::Uuid;

async fn run_contract(base: &str) -> anyhow::Result<()> {
    let schema = format!("jr_check_{}", Uuid::new_v4().simple());
    let admin = Database::connect(base).await?;
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await?;
    let result = contract(base, &schema).await;
    let cleanup = admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await;
    result?;
    cleanup?;
    tracing::info!(
        "routines PostgreSQL contract passed: migrations, lazy crew state, crew isolation, compare-and-swap, multi-crew write refusal, avatar isolation"
    );
    Ok(())
}

fn png() -> anyhow::Result<Vec<u8>> {
    let mut bytes = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image::RgbaImage::new(256, 256))
        .write_to(&mut bytes, image::ImageFormat::Png)?;
    Ok(bytes.into_inner())
}

async fn contract(base: &str, schema: &str) -> anyhow::Result<()> {
    let mut url = url::Url::parse(base)?;
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let db = toolkit_db::connect_db(url.as_str(), ConnectOpts::default()).await?;
    let first = toolkit_db::migration_runner::run_migrations_for_gear(
        &db,
        "juggle-routines",
        Migrator::migrations(),
    )
    .await?;
    anyhow::ensure!(first.applied == 1, "the Gears runner applies the migration");
    let again = toolkit_db::migration_runner::run_migrations_for_gear(
        &db,
        "juggle-routines",
        Migrator::migrations(),
    )
    .await?;
    anyhow::ensure!(
        again.applied == 0 && again.skipped == 1,
        "migration history survives a restart"
    );
    let store = PgStore::new(Arc::new(DBProvider::new(db)));
    let zone = chrono_tz::Europe::Berlin;
    let now = Utc::now();
    let today = domain::today(now, zone);
    let (flat, club) = (Uuid::new_v4(), Uuid::new_v4());
    let (flat_scope, club_scope) = (AccessScope::for_tenant(flat), AccessScope::for_tenant(club));

    // A crew gets its state on first use, exactly once.
    anyhow::ensure!(matches!(
        store.read(&flat_scope).await,
        Err(Error::NotFound)
    ));
    anyhow::ensure!(
        store
            .create(&flat_scope, &State::empty(today, zone))
            .await?
    );
    anyhow::ensure!(
        !store
            .create(&flat_scope, &State::empty(today, zone))
            .await?
    );
    anyhow::ensure!(
        store
            .create(&club_scope, &State::empty(today, zone))
            .await?
    );

    // Flat 4B adopts a cat. The Robotics Club does not see it.
    let (revision, mut state) = store.read(&flat_scope).await?;
    state.zone = zone;
    state.edit_person(
        "demo-pip",
        &PersonEdit {
            operation_id: Uuid::new_v4(),
            expected_revision: 0,
            person: domain::Person {
                id: "mochi".into(),
                name: "Mochi".into(),
                tone: "peach".into(),
                account: None,
                revision: 0,
                created_date: today,
            },
        },
        now,
    )?;
    anyhow::ensure!(
        store
            .compare_and_swap(&flat_scope, revision, &state)
            .await?
    );
    anyhow::ensure!(store.read(&flat_scope).await?.1.people.len() == 1);
    anyhow::ensure!(
        store.read(&club_scope).await?.1.people.is_empty(),
        "one crew's people never appear in another crew"
    );

    // Two writers start from the same revision: exactly one wins.
    let (revision, state) = store.read(&flat_scope).await?;
    anyhow::ensure!(
        store
            .compare_and_swap(&flat_scope, revision, &state)
            .await?
    );
    anyhow::ensure!(
        !store
            .compare_and_swap(&flat_scope, revision, &state)
            .await?,
        "a stale revision is rejected"
    );

    // A scope that admits several crews, or all of them, cannot be used here.
    let both = AccessScope::for_tenants(vec![flat, club]);
    anyhow::ensure!(matches!(store.read(&both).await, Err(Error::Forbidden)));
    anyhow::ensure!(matches!(
        store.read(&AccessScope::allow_all()).await,
        Err(Error::Forbidden)
    ));

    // Photos follow the same boundary.
    store
        .change(
            &flat_scope,
            "mochi",
            "demo-pip",
            &AvatarEdit {
                operation_id: Uuid::new_v4(),
                expected_revision: 0,
            },
            Some(super::avatars::normalize(&png()?)?),
        )
        .await?;
    anyhow::ensure!(store.metadata(&flat_scope).await?["mochi"].has_photo);
    anyhow::ensure!(store.metadata(&club_scope).await?.is_empty());
    anyhow::ensure!(matches!(
        store.image(&club_scope, "mochi").await,
        Err(Error::NotFound)
    ));
    Ok(())
}

#[tokio::test]
async fn the_postgres_contract_holds() {
    let Some(pg) = juggle_testing::postgres().await else {
        return;
    };
    run_contract(&pg.url).await.unwrap();
}
