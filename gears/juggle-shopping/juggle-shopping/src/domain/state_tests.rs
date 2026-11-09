//! The rules of one crew's shopping state.
use crate::domain::{ledger, *};
use crate::test_support::*;
use chrono::Utc;
use uuid::Uuid;

#[test]
fn an_unusual_ocr_year_requires_review_without_rewriting_a_real_historical_date() {
    let mut state = State::default();
    let mut receipt = draft();
    receipt.created_at = chrono::DateTime::parse_from_rfc3339("2026-11-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    state.receipts.push(receipt);
    let mut data = extracted(338);
    data.date = Some("2025-11-01".into());
    state.finish_receipt(0, data, ZONE).unwrap();
    assert_eq!(state.receipts[0].date, "2025-11-01");
    assert_eq!(state.receipts[0].status, "review");
    assert!(state.events.is_empty());
    assert!(
        state.receipts[0]
            .issues
            .iter()
            .any(|s| s.contains("Check the year"))
    );
    let verified = state.receipts[0].clone();
    state
        .apply(
            &Action::EditReceipt {
                receipt: verified,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(ledger::purchases(&state.events)[0].date, "2025-11-01");
    let original = state.events[0].clone();
    let mut corrected = state.receipts[0].clone();
    corrected.date = "2026-11-01".into();
    state
        .apply(
            &Action::EditReceipt {
                receipt: corrected,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(state.events.len(), 3);
    assert_eq!(state.events[0].clone(), original);
    assert_eq!(ledger::purchases(&state.events)[0].date, "2026-11-01");
    assert_eq!(ledger::purchases(&state.events)[0].total, 338);
}
#[test]
fn year_rollover_also_warns_and_explicit_date_corrections_are_canonical() {
    let mut state = State::default();
    let mut receipt = draft();
    receipt.created_at = chrono::DateTime::parse_from_rfc3339("2026-01-01T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    state.receipts.push(receipt);
    let mut data = extracted(338);
    data.date = Some("2025-12-31".into());
    state.finish_receipt(0, data, ZONE).unwrap();
    assert_eq!(state.receipts[0].status, "review");
    assert!(state.events.is_empty());
    assert!(
        state.receipts[0]
            .issues
            .iter()
            .any(|s| s.contains("Check the year"))
    );
    let mut corrected = state.receipts[0].clone();
    corrected.date = "2026-1-1".into();
    state
        .apply(
            &Action::EditReceipt {
                receipt: corrected,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(ledger::purchases(&state.events)[0].date, "2026-01-01");
}
#[test]
fn corrections_learn_exact_merchant_lines_without_changing_financial_values() {
    let mut state = State::default();
    state.receipts.push(draft());
    let mut extraction = extracted(338);
    extraction.items[0].raw_name = "MOOVALLY YOG 2%".into();
    extraction.items[0].name = "Milk".into();
    state.finish_receipt(0, extraction.clone(), ZONE).unwrap();
    let original_event = state.events[0].clone();
    let mut edit = state.receipts[0].clone();
    edit.items[0].raw_name = "MOOVALE YOG 2%".into();
    edit.items[0].name = "Moovale yogurt 2%".into();
    edit.items[0].product_name = Some("Yogurt".into());
    state
        .apply(
            &Action::EditReceipt {
                receipt: edit,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(state.rules.len(), 1);
    assert_eq!(state.rules[0].originals.len(), 2);
    assert_eq!(state.rules[0].product_name, "Yogurt");
    assert_eq!(state.events.len(), 3);
    assert_eq!(state.events[0].clone(), original_event);
    assert_eq!(ledger::purchases(&state.events)[0].total, 338);
    // A conflicting catalog default must not override the exact crew correction.
    let target = state
        .catalog
        .iter_mut()
        .find(|p| p.id == state.rules[0].product_id)
        .unwrap();
    target.category_override = true;
    target.category = "home".into();
    target.subcategory = "Household".into();
    let mut probe = State {
        catalog: state.catalog.clone(),
        rules: state.rules.clone(),
        ..State::default()
    };
    probe.receipts.push(draft());
    probe.finish_receipt(0, extraction.clone(), ZONE).unwrap();
    let item = &probe.receipts[0].items[0];
    assert_eq!(item.name, "Moovale yogurt 2%");
    assert_eq!(item.raw_name, "MOOVALE YOG 2%");
    assert_eq!(item.amount, 338);
    assert_eq!(item.quantity, 2.0);
    assert_eq!(item.category, "food");
    extraction.merchant = Some("Another store".into());
    probe.receipts.push(draft());
    probe.finish_receipt(1, extraction.clone(), ZONE).unwrap();
    assert_ne!(probe.receipts[1].items[0].name, "Moovale yogurt 2%");
    extraction.merchant = Some("Corner Market".into());
    extraction.items[0].raw_name = "MOOVALE MILK 3% 1L".into();
    probe.receipts.push(draft());
    probe.finish_receipt(2, extraction, ZONE).unwrap();
    assert_ne!(
        probe.receipts[2].items[0].product_id,
        Some(state.rules[0].product_id.clone())
    );
    let rule = state.rules[0].clone();
    let mut stale = rule.clone();
    stale.revision = 0;
    assert!(matches!(
        state.apply(&Action::Rule { rule: stale }, "Pip"),
        Err(Error::Conflict)
    ));
    let count = state.events.len();
    let mut changed = rule.clone();
    changed.name = "Corrected yogurt".into();
    state.apply(&Action::Rule { rule: changed }, "Pip").unwrap();
    assert_eq!(state.events.len(), count);
    assert!(matches!(
        state.apply(
            &Action::RuleDelete {
                id: rule.id.clone(),
                revision: rule.revision
            },
            "Pip"
        ),
        Err(Error::Conflict)
    ));
    state
        .apply(
            &Action::RuleDelete {
                id: rule.id,
                revision: rule.revision + 1,
            },
            "Pip",
        )
        .unwrap();
    assert!(state.rules.is_empty());
    assert_eq!(state.events.len(), count);
}
#[test]
fn receipt_without_list_becomes_balanced_event_with_original_and_translated_names() {
    let mut state = State::default();
    state.receipts.push(draft());
    state.finish_receipt(0, extracted(338), ZONE).unwrap();
    assert!(state.list.is_empty());
    assert_eq!(state.receipts[0].status, "saved");
    assert_eq!(state.events.len(), 1);
    let event = &state.events[0];
    event.validate().unwrap();
    assert_eq!(event.postings.iter().map(|p| p.amount).sum::<i64>(), 0);
    assert_eq!(event.purchase_time.as_deref(), Some("18:42"));
    let purchases = ledger::purchases(&state.events);
    assert_eq!(purchases[0].total, 338);
    assert_eq!(purchases[0].items[0].raw_name, "FRESH MILK MEADOW 3% 1L");
    assert_eq!(purchases[0].items[0].name, "Milk Meadow 3%, 1 l");
    assert_eq!(state.catalog[0].name, "Milk");
}
#[test]
fn correction_reverses_and_rebooks_instead_of_overwriting_old_events() {
    let mut state = State::default();
    state.receipts.push(draft());
    state.finish_receipt(0, extracted(338), ZONE).unwrap();
    let original = state.events[0].clone();
    let mut edited = state.receipts[0].clone();
    edited.total = 400;
    edited.items[0].amount = 400;
    edited.items[0].category = "home".into();
    edited.items[0].subcategory = "Household".into();
    state
        .apply(
            &Action::EditReceipt {
                receipt: edited,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Zed",
        )
        .unwrap();
    assert_eq!(state.events.len(), 3);
    assert_eq!(state.events[0].clone(), original);
    assert_eq!(
        state.events[1].reverses.as_deref(),
        Some(state.events[0].id.as_str())
    );
    assert_eq!(
        ledger::purchases(&state.events)
            .iter()
            .map(|p| p.total)
            .sum::<i64>(),
        400
    );
    assert_eq!(
        state
            .events
            .iter()
            .flat_map(|e| e.postings.iter())
            .filter(|p| p.account == "expense")
            .map(|p| p.amount)
            .sum::<i64>(),
        400
    );
    let receipt = state.receipts[0].clone();
    state
        .apply(
            &Action::DeleteReceipt {
                id: receipt.id,
                revision: receipt.revision,
            },
            "Pip",
        )
        .unwrap();
    assert!(ledger::purchases(&state.events).is_empty());
    assert_eq!(state.events.len(), 4);
    assert_eq!(
        state
            .events
            .iter()
            .flat_map(|e| &e.postings)
            .filter(|p| p.account == "expense")
            .map(|p| p.amount)
            .sum::<i64>(),
        0
    );
}
#[test]
fn incomplete_receipts_and_uncertain_matches_do_not_post_or_close_list() {
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
    state.finish_receipt(0, extracted(399), ZONE).unwrap();
    assert_eq!(state.receipts[0].status, "review");
    assert!(state.events.is_empty());
    assert!(!state.list[0].done);
    let mut corrected = state.receipts[0].clone();
    corrected.total = 338;
    state
        .apply(
            &Action::EditReceipt {
                receipt: corrected,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert!(state.list[0].done);
    assert_eq!(state.events.len(), 1);
}
#[test]
fn list_modified_since_upload_is_not_automatically_completed() {
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
    state.list[0].revision += 1;
    state.finish_receipt(0, extracted(338), ZONE).unwrap();
    assert!(!state.list[0].done);
}
#[test]
fn restaurant_context_and_discount_allocation_preserve_balancing() {
    let mut state = State::default();
    state.receipts.push(draft());
    let mut data = extracted(300);
    data.restaurant = true;
    data.adjustments_minor = -38;
    state.finish_receipt(0, data, ZONE).unwrap();
    assert_eq!(state.receipts[0].items[0].category, "restaurants");
    assert_eq!(state.receipts[0].total, 300);
    state.events[0].validate().unwrap();
}
#[test]
fn missing_date_or_currency_requires_review_and_absent_time_stays_absent() {
    for missing in ["date", "currency"] {
        let mut state = State::default();
        state.receipts.push(draft());
        let mut data = extracted(338);
        if missing == "date" {
            data.date = None;
        } else {
            data.currency = Some("bad".into());
        }
        state.finish_receipt(0, data, ZONE).unwrap();
        assert_eq!(state.receipts[0].status, "review");
        assert!(state.events.is_empty());
    }
    let mut state = State::default();
    state.receipts.push(draft());
    let mut data = extracted(338);
    data.time = None;
    state.finish_receipt(0, data, ZONE).unwrap();
    assert_eq!(state.events[0].purchase_time, None);
}
#[test]
fn semantic_duplicate_requires_an_explicit_decision() {
    let mut state = State::default();
    state.receipts.push(draft());
    state.finish_receipt(0, extracted(338), ZONE).unwrap();
    state.receipts.push(draft());
    state.finish_receipt(1, extracted(338), ZONE).unwrap();
    assert_eq!(state.receipts[1].status, "review");
    assert!(state.receipts[1].suspected_duplicate.is_some());
    let duplicate = state.receipts[1].clone();
    assert!(
        state
            .apply(
                &Action::EditReceipt {
                    receipt: duplicate.clone(),
                    confirm_duplicate: false,
                    allow_incomplete: false,
                },
                "Pip"
            )
            .is_err()
    );
    state
        .apply(
            &Action::EditReceipt {
                receipt: duplicate,
                confirm_duplicate: true,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(ledger::purchases(&state.events).len(), 2);
}
#[test]
fn model_refusals_incomplete_outputs_and_extreme_amounts_are_rejected() {
    use crate::infra::openai::parse_response;
    assert!(parse_response(&serde_json::json!({"status":"incomplete","output":[]})).is_err());
    assert!(parse_response(&serde_json::json!({"status":"completed","output":[{"content":[{"type":"refusal","refusal":"no"}]}]})).is_err());
    let mut state = State::default();
    state.receipts.push(draft());
    let mut data = extracted(338);
    data.items[0].amount_minor = i64::MIN;
    assert!(state.finish_receipt(0, data, ZONE).is_err());
}
#[test]
fn quantities_are_checked_before_completing_a_quantified_list_item() {
    for (sold, amount, completed) in [(1., 169, false), (2., 338, true)] {
        let mut state = State::default();
        state
            .apply(
                &Action::Add {
                    items: vec![NewItem {
                        id: Uuid::new_v4(),
                        name: "Milk".into(),
                        quantity: "2 l".into(),
                    }],
                },
                "Pip",
            )
            .unwrap();
        let mut receipt = draft();
        receipt.eligible = state.list.clone();
        state.receipts.push(receipt);
        let mut data = extracted(amount);
        data.items[0].quantity = sold;
        state.finish_receipt(0, data, ZONE).unwrap();
        assert_eq!(state.list[0].done, completed);
    }
}
#[test]
fn translated_manual_alias_reuses_the_existing_product_type() {
    let mut state = State::default();
    state
        .apply(
            &Action::Add {
                items: vec![
                    NewItem {
                        id: Uuid::new_v4(),
                        name: "Milk".into(),
                        quantity: "1 pc".into(),
                    },
                    NewItem {
                        id: Uuid::new_v4(),
                        name: "fresh milk".into(),
                        quantity: "1 pc".into(),
                    },
                ],
            },
            "Pip",
        )
        .unwrap();
    let canonical = state.catalog[0].clone();
    let source = state.catalog[1].clone();
    let translated = ExtractedProduct {
        name: "Milk".into(),
        original_name: "fresh milk".into(),
        product_id: Some(canonical.id.clone()),
        category: "food".into(),
        subcategory: "Dairy & eggs".into(),
        quantity: "1 pc".into(),
        variant: "".into(),
    };
    state.reconcile_classification(&[source], &[translated]);
    assert_eq!(state.catalog.len(), 1);
    assert!(state.catalog[0].aliases.iter().any(|a| a == "fresh milk"));
    assert!(
        state
            .list
            .iter()
            .all(|i| i.product_id.as_deref() == Some(&canonical.id))
    );
}
#[test]
fn correcting_currency_reverses_the_old_currency_and_projects_one_purchase() {
    let mut state = State::default();
    state.receipts.push(draft());
    let mut data = extracted(1220);
    data.currency = Some("USD".into());
    data.items[0].amount_minor = 1220;
    state.finish_receipt(0, data, ZONE).unwrap();
    let original = state.events[0].clone();
    let mut corrected = state.receipts[0].clone();
    corrected.currency = "EUR".into();
    state
        .apply(
            &Action::EditReceipt {
                receipt: corrected,
                confirm_duplicate: false,
                allow_incomplete: false,
            },
            "Pip",
        )
        .unwrap();
    assert_eq!(state.events.len(), 3);
    assert_eq!(state.events[0].clone(), original);
    assert_eq!(state.events[1].currency, "USD");
    assert_eq!(
        state.events[1].reverses.as_deref(),
        Some(state.events[0].id.as_str())
    );
    for event in &state.events {
        event.validate().unwrap();
    }
    let usd_expenses: i64 = state
        .events
        .iter()
        .flat_map(|event| &event.postings)
        .filter(|posting| posting.currency == "USD" && posting.account == "expense")
        .map(|posting| posting.amount)
        .sum();
    assert_eq!(usd_expenses, 0);
    let purchases = ledger::purchases(&state.events);
    assert_eq!(purchases.len(), 1);
    assert_eq!(purchases[0].currency, "EUR");
    assert_eq!(purchases[0].total, 1220);
    assert_eq!(purchases[0].items[0].amount, 1220);
}
#[test]
fn refunds_are_balanced_events_and_currencies_remain_separate() {
    let mut state = State::default();
    state.receipts.push(draft());
    let mut refund = extracted(-338);
    refund.items[0].amount_minor = -338;
    state.finish_receipt(0, refund, ZONE).unwrap();
    state.events[0].validate().unwrap();
    state.receipts.push(draft());
    let mut purchase = extracted(1000);
    purchase.currency = Some("GBP".into());
    purchase.items[0].amount_minor = 1000;
    state.finish_receipt(1, purchase, ZONE).unwrap();
    let purchases = ledger::purchases(&state.events);
    assert_eq!(
        purchases
            .iter()
            .filter(|p| p.currency == "EUR")
            .map(|p| p.total)
            .sum::<i64>(),
        -338
    );
    assert_eq!(
        purchases
            .iter()
            .filter(|p| p.currency == "GBP")
            .map(|p| p.total)
            .sum::<i64>(),
        1000
    );
}
