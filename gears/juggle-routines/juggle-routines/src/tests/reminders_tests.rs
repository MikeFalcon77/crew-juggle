use super::*;
use crate::domain::Subscription;
use crate::test_support::*;

fn device(id: &str, uid: &str) -> Subscription {
    Subscription {
        id: id.into(),
        uid: uid.into(),
        endpoint: format!("https://push.example/{id}"),
        p256dh: "key".into(),
        auth: "auth".into(),
    }
}

#[test]
fn members_hear_about_their_own_tasks_and_the_cats() {
    let now = at("2026-11-10T19:15:00Z"); // 20:15 in Berlin
    let mut state = flat(now);
    let (pip, zed) = (person_of(&state, PIP), person_of(&state, ZED));
    add(
        &mut state,
        routine("trash", &pip, "Take out the trash", "19:30", &[2]),
        now,
    );
    add(
        &mut state,
        routine("dishes", &zed, "Do the dishes", "09:00", &[]),
        now,
    );
    add(
        &mut state,
        routine("dinner", "mochi", "Dinner for Mochi", "20:00", &[]),
        now,
    );
    add(
        &mut state,
        routine("later", &pip, "Read a book", "22:00", &[]),
        now,
    );
    state.subscriptions = vec![device("pip-phone", PIP), device("zed-phone", ZED)];

    prepare_reminders(&mut state, now);
    let for_device = |id: &str| -> Vec<String> {
        let mut people: Vec<_> = state
            .deliveries
            .iter()
            .filter(|d| d.subscription_id == id)
            .map(|d| d.person_id.clone())
            .collect();
        people.sort();
        people
    };
    let mut pip_expected = vec![pip.clone(), "mochi".to_string()];
    pip_expected.sort();
    let mut zed_expected = vec![zed.clone(), "mochi".to_string()];
    zed_expected.sort();
    assert_eq!(for_device("pip-phone"), pip_expected);
    assert_eq!(for_device("zed-phone"), zed_expected);

    // Preparing again creates nothing new.
    let count = state.deliveries.len();
    prepare_reminders(&mut state, now);
    assert_eq!(state.deliveries.len(), count);

    let text = summary(&state, now, ZED, "Europe/Berlin").unwrap().unwrap();
    assert!(text.contains("Zed · 09:00: Do the dishes"), "{text}");
    assert!(text.contains("Mochi · 20:00: Dinner for Mochi"), "{text}");
    assert!(
        !text.contains("trash"),
        "Pip's task is not Zed's business: {text}"
    );
    assert!(!text.contains("Read a book"), "not due yet: {text}");
}

#[test]
fn a_claimed_batch_is_leased_and_cancelled_once_the_work_is_done() {
    let now = at("2026-11-10T19:15:00Z");
    let mut state = flat(now);
    add(
        &mut state,
        routine("dinner", "mochi", "Dinner for Mochi", "20:00", &[]),
        now,
    );
    state.subscriptions = vec![device("pip-phone", PIP)];
    prepare_reminders(&mut state, now);

    let batch = claim_batch(&mut state, "pip-phone", now);
    assert_eq!(batch.len(), 1);
    assert!(batch.iter().all(|d| d.tag == "routines/2026-11-10"));
    assert!(
        claim_batch(&mut state, "pip-phone", now).is_empty(),
        "leased"
    );

    check(&mut state, "dinner", PIP, true, None, now).unwrap();
    assert!(
        summary(&state, now, PIP, "Europe/Berlin")
            .unwrap()
            .is_none()
    );
    let later = now + chrono::Duration::minutes(3);
    assert!(claim_batch(&mut state, "pip-phone", later).is_empty());
    assert!(state.deliveries.iter().all(|d| d.status == "cancelled"));
}
