//! The shopping storage on a real PostgreSQL (`make test-integration`):
//! idempotent commands, compare-and-swap, the append-only ledger trigger and
//! photo expiry, in a disposable schema.
use super::storage::{PgStore, migrations::Migrator};
use crate::domain::{Action, Command, NewItem, Service, Store};
use chrono::Utc;
use sea_orm::{ConnectionTrait, Database};
use sea_orm_migration::MigratorTrait;
use std::sync::Arc;
use toolkit_db::{ConnectOpts, DBProvider};
use uuid::Uuid;

// A DB contract cannot call the network, even when real credentials are present
// in the caller environment. Any accidental model use fails the contract.
struct NoExternalModel;
#[async_trait::async_trait]
impl crate::domain::Intelligence for NoExternalModel {
    fn configured(&self) -> bool {
        false
    }
    async fn receipt(
        &self,
        _: &[Vec<u8>],
        _: &[crate::domain::Product],
        _: &[crate::domain::CorrectionRule],
    ) -> crate::domain::Result<crate::domain::Extraction> {
        Err(crate::domain::Error::Model(
            "Database contract cannot invoke a model".into(),
        ))
    }
    async fn list(
        &self,
        _: &str,
        _: &[crate::domain::Product],
    ) -> crate::domain::Result<Vec<crate::domain::ExtractedProduct>> {
        Err(crate::domain::Error::Model(
            "Database contract cannot invoke a model".into(),
        ))
    }
}

/// Uses only a newly generated disposable schema, never existing crew records.
async fn run_contract(base: &str) -> anyhow::Result<()> {
    let schema = format!("js_test_{}", Uuid::new_v4().simple());
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
    tracing::info!("Shopping PostgreSQL contract passed in disposable schema");
    Ok(())
}
async fn contract(base: &str, schema: &str) -> anyhow::Result<()> {
    let mut url = url::Url::parse(base)?;
    url.query_pairs_mut()
        .append_pair("options", &format!("-csearch_path={schema}"));
    let db = toolkit_db::connect_db(url.as_str(), ConnectOpts::default()).await?;
    let first = toolkit_db::migration_runner::run_migrations_for_gear(
        &db,
        "juggle-shopping",
        Migrator::migrations(),
    )
    .await?;
    anyhow::ensure!(first.applied == 1);
    let repeated = toolkit_db::migration_runner::run_migrations_for_gear(
        &db,
        "juggle-shopping",
        Migrator::migrations(),
    )
    .await?;
    anyhow::ensure!(repeated.applied == 0);
    let crew = Uuid::new_v4();
    let crews = Arc::new(juggle_testing::FixedCrew::new(
        crew,
        &[("shopping-test", "Test")],
    ));
    let policy = crews.policy();
    let ctx = juggle_crews_sdk::worker_context(crew);
    let scope = policy.scope(&ctx).await?;
    let store = Arc::new(PgStore {
        db: Arc::new(DBProvider::new(db)),
    });
    store.ensure(&scope).await?;
    store.ensure(&scope).await?;
    let service = Service {
        store: store.clone(),
        model: Arc::new(NoExternalModel),
        fingerprint: Arc::new(super::storage::codec::JsonFingerprint),
        photos: Arc::new(super::photos::JpegNormalizer),
        policy,
        crews: crews.clone(),
        routines: None,
    };
    let member = crews.context("shopping-test");
    let command = Command {
        operation_id: Uuid::new_v4(),
        action: Action::Add {
            items: vec![NewItem {
                id: Uuid::new_v4(),
                name: "Milk".into(),
                quantity: "2 l".into(),
            }],
        },
    };
    service.command(&member, &command).await?;
    service.command(&member, &command).await?;
    let (revision, state) = store.read(&scope).await?;
    anyhow::ensure!(state.list.len() == 1);
    let (a, b) = tokio::join!(
        store.commit(&scope, revision, &state, &[]),
        store.commit(&scope, revision, &state, &[])
    );
    anyhow::ensure!(a? != b?, "concurrent stale writer must lose");
    let denied = toolkit_security::SecurityContext::builder()
        .subject_id(Uuid::new_v4())
        .subject_tenant_id(crew)
        .build()?;
    anyhow::ensure!(service.read(&denied).await.is_err());
    let id = Uuid::new_v4();
    let photo = crate::domain::Photo {
        receipt_id: id.to_string(),
        ordinal: 0,
        bytes: vec![1, 2, 3],
        expires_at: Utc::now() + chrono::Duration::days(90),
    };
    service
        .upload(&member, id, "same-photo-digest".into(), vec![photo.clone()])
        .await?;
    service
        .upload(&member, id, "same-photo-digest".into(), vec![photo.clone()])
        .await?;
    let duplicate_id = Uuid::new_v4();
    let returned = service
        .upload(
            &member,
            duplicate_id,
            "same-photo-digest".into(),
            vec![crate::domain::Photo {
                receipt_id: duplicate_id.to_string(),
                ..photo
            }],
        )
        .await?;
    anyhow::ensure!(returned == id.to_string());
    anyhow::ensure!(
        store
            .photos(&scope, &duplicate_id.to_string(), Utc::now())
            .await?
            .is_empty(),
        "duplicate uploads must not orphan photo blobs"
    );
    let extraction = super::openai::extraction_from_json(
        serde_json::json!({"merchant":"Fixture store","date":"2026-10-31","time":"18:42","currency":"EUR","total_minor":338,"adjustments_minor":0,"restaurant":false,"needs_review":false,"issues":[],"items":[{"name":"Milk","translated_name":"Milk Meadow 3%, 1 l","raw_name":"FRESH MILK MEADOW 3% 1L","product_id":null,"variant":"Meadow 1 l","category":"food","subcategory":"Dairy & eggs","quantity":2,"unit":"pc","amount_minor":338,"uncertain":false}]}),
    )?;
    service
        .mutate(&member, &[], |state| {
            let index = state
                .receipts
                .iter()
                .position(|r| r.id == id.to_string())
                .unwrap();
            state.finish_receipt(index, extraction.clone(), chrono_tz::UTC)
        })
        .await?;
    let (_, booked) = store.read(&scope).await?;
    anyhow::ensure!(
        booked.events.len() == 1
            && crate::domain::ledger::purchases(&booked.events)[0].total == 338
    );
    let mut corrected = booked.receipts[0].clone();
    corrected.total = 400;
    corrected.items[0].amount = 400;
    let correction = Command {
        operation_id: Uuid::new_v4(),
        action: Action::EditReceipt {
            receipt: corrected,
            confirm_duplicate: false,
            allow_incomplete: false,
        },
    };
    service.command(&member, &correction).await?;
    service.command(&member, &correction).await?;
    let (revision, mut corrected) = store.read(&scope).await?;
    anyhow::ensure!(
        corrected.events.len() == 3
            && crate::domain::ledger::purchases(&corrected.events)[0].total == 400,
        "correction posts reversal+replacement exactly once"
    );
    let conn = Database::connect(url.as_str()).await?;
    anyhow::ensure!(
        conn.execute_unprepared("UPDATE js_ledger SET currency='USD'")
            .await
            .is_err(),
        "database must reject mutation of existing ledger events"
    );
    anyhow::ensure!(
        conn.execute_unprepared("DELETE FROM js_ledger")
            .await
            .is_err(),
        "database must reject ledger deletion"
    );
    let bad=serde_json::json!({"postings":[{"currency":"EUR","amount":1},{"currency":"EUR","amount":0}]}).to_string();
    anyhow::ensure!(conn.execute_unprepared(&format!("INSERT INTO js_ledger(id,tenant_id,receipt_id,purchase_date,recorded_at,currency,payload) VALUES ('{}','{}','bad','2026-10-31',NOW(),'EUR','{}'::jsonb)",Uuid::new_v4(),crew,bad)).await.is_err(),"database must reject unbalanced postings");
    let mut invalid = corrected.events[0].clone();
    invalid.id = Uuid::new_v4().to_string();
    invalid.postings[0].amount += 1;
    corrected.events.push(invalid);
    anyhow::ensure!(
        store
            .commit(&scope, revision, &corrected, &[])
            .await
            .is_err(),
        "failed event append must roll back the state revision"
    );
    anyhow::ensure!(store.read(&scope).await?.0 == revision);
    store
        .expire(&scope, Utc::now() + chrono::Duration::days(91))
        .await?;
    anyhow::ensure!(
        store
            .photos(&scope, &id.to_string(), Utc::now())
            .await?
            .is_empty()
    );
    let (_, state) = store.read(&scope).await?;
    anyhow::ensure!(
        state.events.len() == 3 && !state.receipts[0].items.is_empty(),
        "photo expiry must retain financial history"
    );
    service
        .command(
            &member,
            &Command {
                operation_id: Uuid::new_v4(),
                action: Action::DeleteReceipt {
                    id: id.to_string(),
                    revision: state.receipts[0].revision,
                },
            },
        )
        .await?;
    let (_, state) = store.read(&scope).await?;
    anyhow::ensure!(
        state.events.len() == 4 && crate::domain::ledger::purchases(&state.events).is_empty()
    );
    store.expire(&scope, Utc::now()).await?;
    // Reproduce a review receipt with a missing date and an unresolved amount.
    // Correcting its text must survive an actual database read, without a posting.
    let mut pending = state.receipts[0].clone();
    pending.id = Uuid::new_v4().to_string();
    pending.status = "review".into();
    pending.revision = 1;
    pending.date_missing = true;
    pending.total = 400;
    pending.items[0].amount = 338;
    pending.items[0].uncertain = true;
    pending.matched_ids.clear();
    pending.eligible.clear();
    service
        .mutate(&member, &[], |s| {
            s.receipts.push(pending.clone());
            Ok(())
        })
        .await?;
    let mut edit = pending.clone();
    edit.date = "".into();
    edit.items[0].name = "Oat flakes".into();
    edit.items[0].product_name = Some("Oat flakes".into());
    edit.items[0].uncertain = false;
    let partial = Command {
        operation_id: Uuid::new_v4(),
        action: Action::EditReceipt {
            receipt: edit,
            confirm_duplicate: false,
            allow_incomplete: true,
        },
    };
    service.command(&member, &partial).await?;
    service.command(&member, &partial).await?;
    let (_, read_back) = store.read(&scope).await?;
    let read_receipt = read_back
        .receipts
        .iter()
        .find(|r| r.id == pending.id)
        .unwrap();
    anyhow::ensure!(
        read_receipt.status == "review"
            && read_receipt.revision == 2
            && read_receipt.items[0].name == "Oat flakes"
    );
    anyhow::ensure!(
        read_back.catalog.iter().any(|p| p.name == "Oat flakes")
            && read_back.rules.iter().any(|r| r.name == "Oat flakes")
    );
    anyhow::ensure!(
        read_back.events.len() == 4
            && crate::domain::ledger::purchases(&read_back.events).is_empty(),
        "partial correction must not book an expense"
    );
    let mut ready = read_receipt.clone();
    ready.date_missing = false;
    ready.date = "2026-10-31".into();
    ready.total = 338;
    service
        .command(
            &member,
            &Command {
                operation_id: Uuid::new_v4(),
                action: Action::EditReceipt {
                    receipt: ready,
                    confirm_duplicate: false,
                    allow_incomplete: true,
                },
            },
        )
        .await?;
    let (_, complete) = store.read(&scope).await?;
    anyhow::ensure!(
        complete.events.len() == 5
            && crate::domain::ledger::purchases(&complete.events).len() == 1
            && crate::domain::ledger::purchases(&complete.events)[0].total == 338
    );
    Ok(())
}

#[tokio::test]
async fn the_postgres_contract_holds() {
    let Some(pg) = juggle_testing::postgres().await else {
        return;
    };
    run_contract(&pg.url).await.unwrap();
}
