use super::*;
use crate::test_support::*;
use chrono::Duration;

#[test]
fn weekly_routines_show_up_only_on_their_days() {
    let now = tuesday();
    let mut state = flat(now);
    let pip = person_of(&state, PIP);
    add(
        &mut state,
        routine("trash", &pip, "Take out the trash", "19:30", &[2]),
        now,
    );
    add(
        &mut state,
        routine("plants", &pip, "Water the plants", "10:00", &[6]),
        now,
    );
    add(
        &mut state,
        routine("kitchen", &pip, "Wipe the kitchen", "21:00", &[]),
        now,
    );

    let tuesday_ids: Vec<_> = state.days[&date("2026-11-10")]
        .iter()
        .map(|i| i.routine.id.as_str())
        .collect();
    assert_eq!(tuesday_ids, ["trash", "kitchen"]);

    let wednesday = now + Duration::days(1);
    state.ensure_day(date("2026-11-11"), today(wednesday, ZONE));
    let wednesday_ids: Vec<_> = state.days[&date("2026-11-11")]
        .iter()
        .map(|i| i.routine.id.as_str())
        .collect();
    assert_eq!(wednesday_ids, ["kitchen"], "trash is a Tuesday thing");
}

#[test]
fn bad_days_and_times_are_refused_and_days_are_normalized() {
    let now = tuesday();
    let mut state = flat(now);
    let pip = person_of(&state, PIP);
    let mut r = routine("x", &pip, "Thing", "08:00", &[8]);
    r.start_date = state.today(now);
    let edit = |r: Routine| Edit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        effective_from: date("2026-11-10"),
        routine: r,
    };
    assert!(state.edit(PIP, &edit(r.clone()), now).is_err());
    r.days = vec![5, 1, 1];
    r.time = "8:00".into();
    assert!(state.edit(PIP, &edit(r.clone()), now).is_err());
    r.time = "08:00".into();
    let version = state.edit(PIP, &edit(r), now).unwrap();
    assert_eq!(version.routine.days, [1, 5]);
}

#[test]
fn every_account_gets_one_person_and_the_cat_needs_no_account() {
    let now = tuesday();
    let mut state = flat(now);
    assert_eq!(state.people.len(), 3);
    assert!(!state.sync_accounts(&[member(PIP, "Pip")], state.today(now)));
    assert!(state.sync_accounts(&[member("demo-new", "Newbie")], state.today(now)));
    assert_eq!(state.people.len(), 4);

    let mochi = state.people.iter().find(|p| p.id == "mochi").unwrap();
    assert_eq!(mochi.account, None);

    // Nobody can claim an account by editing a profile.
    let mut pip = state
        .people
        .iter()
        .find(|p| p.account.as_deref() == Some(PIP))
        .unwrap()
        .clone();
    let mut hijack = mochi.clone();
    hijack.account = Some(ZED.into());
    let saved = state
        .edit_person(
            PIP,
            &PersonEdit {
                operation_id: Uuid::new_v4(),
                expected_revision: hijack.revision,
                person: hijack,
            },
            now,
        )
        .unwrap();
    assert_eq!(saved.account, None);
    pip.name = "Pip the Planner".into();
    let renamed = state
        .edit_person(
            PIP,
            &PersonEdit {
                operation_id: Uuid::new_v4(),
                expected_revision: pip.revision,
                person: Person {
                    account: None,
                    ..pip
                },
            },
            now,
        )
        .unwrap();
    assert_eq!(
        renamed.account.as_deref(),
        Some(PIP),
        "the link survives a rename"
    );
}

#[test]
fn a_second_step_waits_for_the_first_and_shares_its_slot() {
    let now = tuesday();
    let mut state = flat(now);
    let zed = person_of(&state, ZED);
    let pip = person_of(&state, PIP);
    add(
        &mut state,
        routine("wash", &zed, "Start the laundry", "11:00", &[2, 7]),
        now,
    );
    let mut hang = routine("hang", &pip, "Hang the laundry", "18:00", &[]);
    hang.prerequisite = Some("wash".into());
    let second = add(&mut state, hang, now).routine;
    assert_eq!(
        (
            second.person_id.as_str(),
            second.time.as_str(),
            second.days.as_slice()
        ),
        (zed.as_str(), "11:00", [2u8, 7].as_slice()),
        "a pair shares who, when and which days"
    );

    let error = check(&mut state, "hang", ZED, true, None, now).unwrap_err();
    assert_eq!(error.to_string(), "Check off “Start the laundry” first");
    check(&mut state, "wash", ZED, true, None, now).unwrap();
    check(&mut state, "hang", ZED, true, None, now).unwrap();
    let error = check(&mut state, "wash", ZED, false, None, now).unwrap_err();
    assert_eq!(error.to_string(), "Undo “Hang the laundry” first");

    let mut third = routine("fold", &zed, "Fold it", "11:00", &[]);
    third.prerequisite = Some("wash".into());
    third.start_date = state.today(now);
    let refused = state.edit(
        PIP,
        &Edit {
            operation_id: Uuid::new_v4(),
            expected_revision: 0,
            effective_from: state.today(now),
            routine: third,
        },
        now,
    );
    assert!(refused.is_err(), "two steps at most");
}

#[test]
fn undoing_someone_elses_check_off_needs_their_name_confirmed() {
    let now = tuesday();
    let mut state = flat(now);
    let mochi = "mochi";
    add(
        &mut state,
        routine("feed", mochi, "Breakfast for Mochi", "08:00", &[]),
        now,
    );
    check(&mut state, "feed", PIP, true, None, now).unwrap();

    assert!(matches!(
        check(&mut state, "feed", ZED, false, None, now),
        Err(Error::ConfirmationRequired)
    ));
    assert!(matches!(
        check(&mut state, "feed", ZED, false, Some("someone-else"), now),
        Err(Error::ConfirmationRequired)
    ));
    let undone = check(&mut state, "feed", ZED, false, Some(PIP), now).unwrap();
    assert_eq!(undone.status, "pending");
    assert_eq!(undone.completed_by, None);
}

#[test]
fn history_is_frozen_while_tomorrow_can_change() {
    let now = tuesday();
    let mut state = flat(now);
    let zed = person_of(&state, ZED);
    let version = add(
        &mut state,
        routine("dishes", &zed, "Do the dishes", "09:00", &[]),
        now,
    );
    check(&mut state, "dishes", ZED, true, None, now).unwrap();

    // Renaming from tomorrow leaves today's done task as it was.
    let mut renamed = version.routine.clone();
    renamed.name = "Do ALL the dishes".into();
    state
        .edit(
            PIP,
            &Edit {
                operation_id: Uuid::new_v4(),
                expected_revision: version.routine.revision,
                effective_from: date("2026-11-11"),
                routine: renamed,
            },
            now,
        )
        .unwrap();
    assert_eq!(
        state.days[&date("2026-11-10")][0].routine.name,
        "Do the dishes"
    );

    let tomorrow = now + Duration::days(1);
    state.ensure_day(date("2026-11-11"), today(tomorrow, ZONE));
    assert_eq!(
        state.days[&date("2026-11-11")][0].routine.name,
        "Do ALL the dishes"
    );
    assert!(
        state
            .edit(
                PIP,
                &Edit {
                    operation_id: Uuid::new_v4(),
                    expected_revision: 2,
                    effective_from: date("2026-11-09"),
                    routine: version.routine,
                },
                now,
            )
            .is_err(),
        "changes cannot start in the past"
    );
}

#[test]
fn the_crew_day_follows_the_crew_zone_including_dst() {
    // 23:30 UTC is already the next day in Berlin and still the same day in New York.
    let late = at("2026-11-10T23:30:00Z");
    assert_eq!(today(late, ZONE), date("2026-11-11"));
    assert_eq!(
        today(late, chrono_tz::America::New_York),
        date("2026-11-10")
    );

    // Spring forward: 02:30 does not exist in Berlin, the first valid minute is 03:00.
    assert_eq!(
        scheduled_at(date("2026-03-29"), "02:30", ZONE).unwrap(),
        at("2026-03-29T01:00:00Z")
    );
    // Fall back: 02:30 happens twice; the earlier one counts.
    assert_eq!(
        scheduled_at(date("2026-10-25"), "02:30", ZONE).unwrap(),
        at("2026-10-25T00:30:00Z")
    );
}

#[test]
fn time_labels_add_the_viewers_clock_only_when_it_differs() {
    let day = date("2026-11-10");
    assert_eq!(
        time_label(day, "08:00", ZONE, "Europe/Berlin").unwrap(),
        "08:00"
    );
    assert_eq!(
        time_label(day, "08:00", ZONE, "America/New_York").unwrap(),
        "08:00 (02:00 New York)"
    );
    assert_eq!(
        time_label(day, "00:30", ZONE, "America/Los_Angeles").unwrap(),
        "00:30 (15:30 Los Angeles) Nov 9"
    );
}

#[test]
fn preferences_default_to_the_crew_zone_and_replay_exactly() {
    let now = tuesday();
    let mut state = flat(now);
    assert_eq!(state.preferences_for(ZED).timezone, "Europe/Berlin");
    let edit = PreferencesEdit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        timezone: "Asia/Tokyo".into(),
    };
    let saved = state.edit_preferences(ZED, &edit).unwrap();
    assert_eq!(state.edit_preferences(ZED, &edit).unwrap(), saved);
    assert_eq!(state.preferences_for(PIP).revision, 0, "per account");
    let altered = PreferencesEdit {
        timezone: "Europe/Paris".into(),
        ..edit
    };
    assert!(matches!(
        state.edit_preferences(ZED, &altered),
        Err(Error::Conflict)
    ));
    assert!(
        state
            .edit_preferences(
                ZED,
                &PreferencesEdit {
                    operation_id: Uuid::new_v4(),
                    expected_revision: 1,
                    timezone: "Mars/Olympus".into(),
                },
            )
            .is_err()
    );
}

#[test]
fn moods_are_separate_from_task_history() {
    let now = tuesday();
    let mut state = flat(now);
    let zed = person_of(&state, ZED);
    add(
        &mut state,
        routine("litter", &zed, "Clean the litter box", "19:00", &[]),
        now,
    );
    let before = state.days.clone();
    let edit = AppearanceEdit {
        operation_id: Uuid::new_v4(),
        expected_revision: 0,
        mood: Mood::Grumpy,
    };
    let saved = state.edit_appearance("litter", ZED, &edit).unwrap();
    assert_eq!(saved.mood, Mood::Grumpy);
    assert_eq!(state.days, before);
    assert!(matches!(
        state.edit_appearance("missing", ZED, &edit),
        Err(Error::Conflict) | Err(Error::NotFound)
    ));
}

#[test]
fn the_demo_flat_has_pip_done_zed_behind_and_a_cat_with_a_job() {
    let now = tuesday();
    let members = [member(PIP, "Pip"), member(ZED, "Zed")];
    let state = demo::state("flat-4b", &members, ZONE, now)
        .unwrap()
        .unwrap();
    let today = state.days[&date("2026-11-10")].clone();
    let pip = person_of(&state, PIP);
    assert!(
        today
            .iter()
            .filter(|i| i.routine.person_id == pip)
            .all(|i| i.status == "done"),
        "Pip checked everything off in advance"
    );
    let dishes = today.iter().find(|i| i.routine.id == "dishes").unwrap();
    assert_eq!(dishes.status, "pending");
    assert!(
        today
            .iter()
            .any(|i| i.routine.person_id == "mochi" && i.routine.name.contains("Supervise"))
    );
    assert!(
        today.iter().any(|i| i.routine.id == "trash"),
        "Tuesday is trash day"
    );
    assert_eq!(state.days.len(), 4, "three days of history plus today");
    assert!(demo::state("nope", &members, ZONE, now).unwrap().is_none());
}
