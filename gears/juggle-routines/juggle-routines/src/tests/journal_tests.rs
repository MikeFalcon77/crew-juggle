use crate::test_support::*;
use crate::{domain::*, infra::storage::codec};
use chrono::Duration;
use uuid::Uuid;

#[test]
fn every_command_replays_identically_after_a_save_and_load() {
    let now = tuesday();
    let mut state = flat(now);
    let zed = person_of(&state, ZED);
    let version = add(
        &mut state,
        routine("dishes", &zed, "Do the dishes", "09:00", &[1, 2]),
        now,
    );
    let date = state.today(now);
    let mark = Mark {
        operation_id: Uuid::new_v4(),
        expected_revision: 1,
        completed: true,
        confirmed_completed_by: None,
    };
    let marked = state.mark(date, "dishes", ZED, &mark, now).unwrap();
    let appearance = AppearanceEdit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        mood: Mood::Grumpy,
    };
    state.edit_appearance("dishes", ZED, &appearance).unwrap();
    let preferences = PreferencesEdit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        timezone: "Asia/Tokyo".into(),
    };
    state.edit_preferences(ZED, &preferences).unwrap();
    let edit = Edit {
        operation_id: Uuid::new_v4(),
        expected_revision: version.routine.revision,
        effective_from: date + Duration::days(1),
        routine: version.routine.clone(),
    };
    state.edit(PIP, &edit, now).unwrap();
    let avatar = AvatarEdit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
    };
    state
        .apply_avatar_change("mochi", PIP, &avatar, "aabbcc", 0, true)
        .unwrap();

    let json = serde_json::to_value(codec::State::from(state.clone())).unwrap();
    let mut restored: State = serde_json::from_value::<codec::State>(json.clone())
        .unwrap()
        .into();
    restored.zone = ZONE;
    assert_eq!(
        serde_json::to_value(codec::State::from(restored.clone())).unwrap(),
        json,
        "the stored shape round-trips"
    );
    assert_eq!(
        restored.mark(date, "dishes", ZED, &mark, now).unwrap(),
        marked
    );
    restored
        .edit_appearance("dishes", ZED, &appearance)
        .unwrap();
    restored.edit_preferences(ZED, &preferences).unwrap();
    restored.edit(PIP, &edit, now).unwrap();
    assert!(
        restored
            .apply_avatar_change("mochi", PIP, &avatar, "aabbcc", 1, true)
            .unwrap()
            .replayed
    );
    // The same operation id from someone else, or with other content, is a conflict.
    assert!(matches!(
        restored.mark(date, "dishes", PIP, &mark, now),
        Err(Error::Conflict)
    ));
    assert_eq!(
        restored.latest_routines()[0].routine.days,
        [1, 2],
        "weekdays survive storage"
    );
    assert!(
        restored
            .people
            .iter()
            .any(|p| p.account.as_deref() == Some(PIP))
    );
}
