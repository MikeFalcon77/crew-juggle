//! Stored shapes: old rows load, fingerprints stay, a round-trip loses nothing.
use crate::domain::{ledger, *};
use crate::infra::storage::codec;
use crate::test_support::*;
use uuid::Uuid;

#[test]
fn partial_review_corrections_survive_reopen_without_posting_or_closing_list() {
    let mut state = State::default();
    state
        .apply(
            &Action::Add {
                items: vec![NewItem {
                    id: Uuid::new_v4(),
                    name: "Milk".into(),
                    quantity: "2 pc".into(),
                }],
            },
            "Pip",
        )
        .unwrap();
    let mut receipt = draft();
    receipt.eligible = state.list.clone();
    state.receipts.push(receipt);
    let mut data = extracted(478);
    data.date = None;
    data.items[0].uncertain = true;
    let mut second = data.items[0].clone();
    second.name = "Another unknown item".into();
    second.raw_name = "UNREADABLE 2".into();
    second.amount_minor = 100;
    data.items.push(second);
    state.finish_receipt(0, data, ZONE).unwrap();
    let mut edit = state.receipts[0].clone();
    edit.date = "".into();
    edit.items[0].name = "Oat flakes".into();
    edit.items[0].product_name = Some("Oat flakes".into());
    edit.items[0].uncertain = false;
    state
        .apply(
            &Action::EditReceipt {
                receipt: edit,
                confirm_duplicate: false,
                allow_incomplete: true,
            },
            "Pip",
        )
        .unwrap();
    // A cold round-trip through the stored form, as another session would load it.
    let mut reopened: State = codec::decode_state(codec::encode_state(&state).unwrap()).unwrap();
    assert_eq!(reopened.receipts[0].status, "review");
    assert!(reopened.receipts[0].date_missing);
    assert_eq!(reopened.receipts[0].items[0].name, "Oat flakes");
    assert!(
        reopened
            .catalog
            .iter()
            .any(|p| p.name == "Oat flakes" && p.aliases.contains(&"Oat flakes".into()))
    );
    assert_eq!(reopened.rules.len(), 1);
    assert!(reopened.receipts[0].items[1].uncertain);
    assert!(reopened.events.is_empty());
    assert!(ledger::purchases(&reopened.events).is_empty());
    assert!(!reopened.list[0].done);
    assert!(
        reopened.receipts[0]
            .issues
            .iter()
            .any(|s| s.contains("add up"))
    );
    assert!(reopened.receipts[0].issues.iter().any(|s| s.contains("2")));
    let mut ready = reopened.receipts[0].clone();
    ready.date = "2026-10-31".into();
    ready.date_missing = false;
    ready.total = 438;
    ready.items[1].uncertain = false;
    reopened
        .apply(
            &Action::EditReceipt {
                receipt: ready,
                confirm_duplicate: false,
                allow_incomplete: true,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(reopened.receipts[0].status, "saved");
    assert_eq!(ledger::purchases(&reopened.events)[0].total, 438);
    let before = reopened.events.clone();
    let mut invalid = reopened.receipts[0].clone();
    invalid.total += 1;
    assert!(
        reopened
            .apply(
                &Action::EditReceipt {
                    receipt: invalid,
                    confirm_duplicate: false,
                    allow_incomplete: true
                },
                "Pip"
            )
            .is_err()
    );
    assert_eq!(reopened.events.clone(), before);
}
#[test]
fn incomplete_mode_is_opt_in_and_does_not_change_legacy_operation_fingerprints() {
    let action = Action::EditReceipt {
        receipt: draft(),
        confirm_duplicate: false,
        allow_incomplete: false,
    };
    let json = serde_json::to_value(codec::Action::from(action)).unwrap();
    assert!(json.get("allow_incomplete").is_none());
    let parsed: Action = serde_json::from_value::<codec::Action>(json)
        .unwrap()
        .into();
    assert!(matches!(
        parsed,
        Action::EditReceipt {
            allow_incomplete: false,
            ..
        }
    ));
}
#[test]
fn old_states_load_without_rules_and_missing_dates_are_explicit() {
    let state: State =
        codec::decode_state(serde_json::json!({"list":[],"catalog":[],"receipts":[]})).unwrap();
    assert!(state.rules.is_empty());
    let mut state = state;
    state.receipts.push(draft());
    let mut extraction = extracted(338);
    extraction.date = None;
    state.finish_receipt(0, extraction, ZONE).unwrap();
    assert!(state.receipts[0].date_missing);
    assert_eq!(state.receipts[0].status, "review");
    assert!(state.events.is_empty());
    assert!(
        state.receipts[0]
            .issues
            .iter()
            .any(|s| s.contains("date is not visible"))
    );
}
