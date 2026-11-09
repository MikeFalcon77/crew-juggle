//! The crews storage on a real PostgreSQL (`make test-integration`): the
//! migration, two crews that never see each other, invite codes and the demo
//! seed, in a disposable schema.
use super::{SeaOrmCrewRepo, migrations::Migrator};
use crate::domain::CrewsService;
use crate::domain::model::Crew;
use crate::domain::repo::CrewRepo;
use juggle_crews_sdk::{AccountIdentity, Role, demo, subject_for};
use sea_orm::{ConnectionTrait, Database};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use toolkit_db::{ConnectOpts, DBProvider};
use toolkit_security::{AccessScope, SecurityContext};
use uuid::Uuid;

fn account(name: &str) -> AccountIdentity {
    AccountIdentity {
        uid: format!("uid-{name}"),
        email: format!("{name}@example.test"),
        name: Some(name.into()),
    }
}

async fn signed_in(service: &CrewsService, name: &str) -> SecurityContext {
    let membership = service.sign_in(account(name)).await.unwrap();
    SecurityContext::builder()
        .subject_id(membership.subject_id)
        .subject_tenant_id(membership.tenant_id)
        .subject_type(membership.kind.subject_type())
        .build()
        .unwrap()
}

async fn contract(base: &str, schema: &str) -> anyhow::Result<()> {
    let mut url = url::Url::parse(base)?;
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let db = toolkit_db::connect_db(url.as_str(), ConnectOpts::default()).await?;
    let runner = toolkit_db::migration_runner::run_migrations_for_gear;
    anyhow::ensure!(
        runner(&db, "juggle-crews", Migrator::migrations())
            .await?
            .applied
            == 1
    );
    anyhow::ensure!(
        runner(&db, "juggle-crews", Migrator::migrations())
            .await?
            .applied
            == 0
    );
    let repo = Arc::new(SeaOrmCrewRepo::new(Arc::new(DBProvider::new(db))));
    let crews = CrewsService::new(repo.clone());

    let pip = signed_in(&crews, "pip").await;
    let flat = crews
        .create(&pip, "Flat 4B", "Pip", "Europe/Berlin")
        .await?
        .crew
        .unwrap();
    let zed = signed_in(&crews, "zed").await;
    crews.join(&zed, &flat.invite_code, "Zed").await?;
    let nova = signed_in(&crews, "nova").await;
    let club = crews
        .create(&nova, "Robotics Club", "Nova", "Europe/Berlin")
        .await?
        .crew
        .unwrap();

    // Each crew's scope sees its own rows and nothing else.
    let flat_members = crews.members(flat.id).await?;
    anyhow::ensure!(
        flat_members
            .iter()
            .map(|m| m.display_name.as_str())
            .collect::<Vec<_>>()
            == ["Pip", "Zed"]
    );
    anyhow::ensure!(flat_members[0].role == Role::Owner);
    anyhow::ensure!(crews.members(club.id).await?.len() == 1);
    anyhow::ensure!(crews.is_member(subject_for("uid-zed"), flat.id).await?);
    anyhow::ensure!(!crews.is_member(subject_for("uid-zed"), club.id).await?);

    // A write scoped to the club cannot touch the flat, even by id.
    let stolen = Crew {
        id: flat.id,
        name: "Taken".into(),
        timezone: "UTC".into(),
        invite_code: "AAAAAAAA".into(),
        created_at: chrono::Utc::now(),
    };
    repo.update_crew(&AccessScope::for_tenant(club.id), &stolen)
        .await?;
    anyhow::ensure!(crews.crew(flat.id).await?.unwrap().name == "Flat 4B");
    anyhow::ensure!(
        repo.crew(&AccessScope::for_tenant(club.id))
            .await?
            .unwrap()
            .id
            == club.id
    );

    // Only the explicit cross-crew read sees both.
    anyhow::ensure!(crews.all_crew_ids().await?.len() == 2);

    // The demo seed can run on every start.
    crews.seed_demo("Europe/Berlin").await?;
    crews.seed_demo("Europe/Berlin").await?;
    anyhow::ensure!(crews.members(demo::crew_id("flat-4b")).await?.len() == 2);
    anyhow::ensure!(crews.all_crew_ids().await?.len() == 4);
    Ok(())
}

#[tokio::test]
async fn crews_stay_apart_in_postgres() {
    let Some(pg) = juggle_testing::postgres().await else {
        return;
    };
    let schema = format!("cj_check_{}", Uuid::new_v4().simple());
    let admin = Database::connect(&pg.url).await.unwrap();
    admin
        .execute_unprepared(&format!("CREATE SCHEMA {schema}"))
        .await
        .unwrap();
    let result = contract(&pg.url, &schema).await;
    admin
        .execute_unprepared(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    result.unwrap();
}
