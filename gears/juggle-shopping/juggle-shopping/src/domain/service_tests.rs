//! Use cases through the policy and an in-memory store.
use crate::domain::*;
use crate::test_support::*;
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

#[test]
fn a_stranger_can_neither_read_nor_change_the_crews_list() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let store = Arc::new(Memory::new());
        let (service, crew) = crew_service(store.clone());
        let pip = crew.context("demo-pip");
        let add = Command {
            operation_id: Uuid::new_v4(),
            action: Action::Add {
                items: vec![NewItem {
                    id: Uuid::new_v4(),
                    name: "Cat food".into(),
                    quantity: "6".into(),
                }],
            },
        };
        service.command(&pip, &add).await.unwrap();
        assert_eq!(service.read(&pip).await.unwrap().1.list[0].author, "Pip");

        let stranger = toolkit_security::SecurityContext::builder()
            .subject_id(Uuid::new_v4())
            .subject_tenant_id(crew.id)
            .subject_type("member")
            .build()
            .unwrap();
        assert!(matches!(
            service.read(&stranger).await,
            Err(Error::Forbidden)
        ));
        let sneaky = Command {
            operation_id: Uuid::new_v4(),
            ..add
        };
        assert!(service.command(&stranger, &sneaky).await.is_err());
        assert_eq!(store.state.lock().unwrap().1.list.len(), 1);
    });
}

#[test]
fn the_demo_scanner_saves_a_receipt_and_checks_off_the_list() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let store = Arc::new(Memory::new());
        let (service, crew) = crew_service(store.clone());
        let pip = crew.context("demo-pip");
        let zed = crew.context("demo-zed");
        service
            .command(
                &pip,
                &Command {
                    operation_id: Uuid::new_v4(),
                    action: Action::Import {
                        id: Uuid::new_v4(),
                        text: "Cat food, Dish soap".into(),
                    },
                },
            )
            .await
            .unwrap();
        service.tick().await.unwrap();
        assert_eq!(service.read(&pip).await.unwrap().1.list.len(), 2);

        let receipt = Uuid::new_v4();
        let photo = Photo {
            receipt_id: receipt.to_string(),
            ordinal: 0,
            bytes: vec![1, 2, 3],
            expires_at: Utc::now() + chrono::Duration::days(1),
        };
        service
            .upload(&zed, receipt, "digest".into(), vec![photo])
            .await
            .unwrap();
        service.tick().await.unwrap();
        let (_, state) = service.read(&pip).await.unwrap();
        let saved = state
            .receipts
            .iter()
            .find(|r| r.id == receipt.to_string())
            .unwrap();
        assert_eq!(saved.status, "saved");
        assert_eq!(saved.registered_by, "Zed");
        assert_eq!(saved.registered_account.as_deref(), Some("demo-zed"));
        assert!(saved.issues[0].starts_with("Demo scan"));
        assert!(state.list.iter().all(|i| i.done), "{:?}", state.list);
    });
}
