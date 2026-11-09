//! Shared fixtures: Flat 4B with Pip and Zed (accounts) and Mochi (no account).
use crate::domain::*;
use chrono::{DateTime, NaiveDate, Utc};
use juggle_crews_sdk::{CrewMember, Role};
use uuid::Uuid;

pub const ZONE: chrono_tz::Tz = chrono_tz::Europe::Berlin;
pub const PIP: &str = "demo-pip";
pub const ZED: &str = "demo-zed";

pub fn at(text: &str) -> DateTime<Utc> {
    text.parse().unwrap()
}

/// Tuesday 2026-11-10, 10:30 in Berlin.
pub fn tuesday() -> DateTime<Utc> {
    at("2026-11-10T09:30:00Z")
}

pub fn date(text: &str) -> NaiveDate {
    text.parse().unwrap()
}

pub fn member(uid: &str, name: &str) -> CrewMember {
    CrewMember {
        uid: uid.into(),
        subject_id: juggle_crews_sdk::subject_for(uid),
        display_name: name.into(),
        role: Role::Member,
    }
}

pub fn person_of(state: &State, uid: &str) -> String {
    state
        .people
        .iter()
        .find(|p| p.account.as_deref() == Some(uid))
        .unwrap()
        .id
        .clone()
}

/// A crew created on `now`'s date with Pip, Zed and Mochi.
pub fn flat(now: DateTime<Utc>) -> State {
    let mut state = State::empty(today(now, ZONE), ZONE);
    state.sync_accounts(
        &[member(PIP, "Pip"), member(ZED, "Zed")],
        state.created_date,
    );
    state
        .edit_person(
            PIP,
            &PersonEdit {
                operation_id: Uuid::new_v4(),
                expected_revision: 0,
                person: Person {
                    id: "mochi".into(),
                    name: "Mochi".into(),
                    tone: "peach".into(),
                    account: None,
                    revision: 0,
                    created_date: state.created_date,
                },
            },
            now,
        )
        .unwrap();
    state
}

pub fn routine(id: &str, person_id: &str, name: &str, time: &str, days: &[u8]) -> Routine {
    Routine {
        id: id.into(),
        revision: 0,
        assignee: String::new(),
        person_id: person_id.into(),
        name: name.into(),
        note: String::new(),
        group: String::new(),
        time: time.into(),
        days: days.to_vec(),
        start_date: date("2026-01-01"),
        end_date: None,
        enabled: true,
        prerequisite: None,
    }
}

pub fn add(state: &mut State, mut r: Routine, now: DateTime<Utc>) -> Version {
    r.start_date = state.today(now);
    state
        .edit(
            PIP,
            &Edit {
                operation_id: Uuid::new_v4(),
                expected_revision: 0,
                effective_from: state.today(now),
                routine: r,
            },
            now,
        )
        .unwrap()
}

pub fn check(
    state: &mut State,
    id: &str,
    actor: &str,
    completed: bool,
    confirmed: Option<&str>,
    now: DateTime<Utc>,
) -> Result<Occurrence> {
    let date = state.today(now);
    state.ensure_day(date, date);
    let revision = state.days[&date]
        .iter()
        .find(|i| i.routine.id == id)
        .map_or(0, |i| i.revision);
    state.mark(
        date,
        id,
        actor,
        &Mark {
            operation_id: Uuid::new_v4(),
            expected_revision: revision,
            completed,
            confirmed_completed_by: confirmed.map(Into::into),
        },
        now,
    )
}
