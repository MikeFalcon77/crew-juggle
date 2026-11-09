//! Demo crews, built through the same rules a real crew goes through.
//!
//! Flat 4B: Pip plans everything (today is already checked off), Zed builds
//! robots in the kitchen (the dishes are overdue), and Mochi the cat has no
//! account but does have a job. The Robotics Club keeps a workshop running.
use super::{Appearance, Edit, Mark, Mood, Person, Result, Routine, State, scheduled_at, today};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use chrono_tz::Tz;
use juggle_crews_sdk::{CrewMember, demo as cast};
use uuid::Uuid;

struct Plan {
    id: &'static str,
    who: &'static str,
    name: &'static str,
    note: &'static str,
    group: &'static str,
    time: &'static str,
    days: &'static [u8],
    after: Option<&'static str>,
    mood: Mood,
}

const EVERY_DAY: &[u8] = &[];

const FLAT_4B: &[Plan] = &[
    Plan {
        id: "mochi-breakfast",
        who: "pip",
        name: "Breakfast for Mochi",
        note: "Half a pouch. She will tell you it was nothing.",
        group: "Mochi",
        time: "08:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Happy,
    },
    Plan {
        id: "dishes",
        who: "zed",
        name: "Do the dishes",
        note: "Robot parts are not dishes. Take them off the drying rack.",
        group: "Kitchen",
        time: "09:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Grumpy,
    },
    Plan {
        id: "plants",
        who: "pip",
        name: "Water the plants",
        note: "The cactus too, a little.",
        group: "Flat",
        time: "10:00",
        days: &[6],
        after: None,
        mood: Mood::Happy,
    },
    Plan {
        id: "laundry-start",
        who: "zed",
        name: "Start the laundry",
        note: "40°, no lab coats with the towels.",
        group: "Laundry",
        time: "11:00",
        days: &[7],
        after: None,
        mood: Mood::Neutral,
    },
    Plan {
        id: "laundry-hang",
        who: "zed",
        name: "Hang the laundry",
        note: "",
        group: "Laundry",
        time: "11:00",
        days: &[7],
        after: Some("laundry-start"),
        mood: Mood::Neutral,
    },
    Plan {
        id: "litter",
        who: "zed",
        name: "Clean the litter box",
        note: "",
        group: "Mochi",
        time: "19:00",
        days: &[1, 3, 5],
        after: None,
        mood: Mood::Grumpy,
    },
    Plan {
        id: "trash",
        who: "pip",
        name: "Take out the trash",
        note: "Paper on the left, plastic on the right.",
        group: "Flat",
        time: "19:30",
        days: &[2],
        after: None,
        mood: Mood::Neutral,
    },
    Plan {
        id: "mochi-dinner",
        who: "zed",
        name: "Dinner for Mochi",
        note: "",
        group: "Mochi",
        time: "20:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Happy,
    },
    Plan {
        id: "kitchen",
        who: "pip",
        name: "Wipe the kitchen",
        note: "",
        group: "Kitchen",
        time: "21:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Neutral,
    },
    Plan {
        id: "supervise",
        who: "mochi",
        name: "Supervise kitchen cleaning 😼",
        note: "From the top of the fridge, as usual.",
        group: "Mochi",
        time: "21:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Happy,
    },
];

const ROBOTICS_CLUB: &[Plan] = &[
    Plan {
        id: "chargers",
        who: "ike",
        name: "Check the battery chargers",
        note: "Anything warm to the touch goes in the metal box.",
        group: "Workshop",
        time: "17:00",
        days: &[1, 4],
        after: None,
        mood: Mood::Neutral,
    },
    Plan {
        id: "snacks",
        who: "ike",
        name: "Order snacks and spare parts",
        note: "Scan the receipt so the club fund stays honest.",
        group: "Supplies",
        time: "16:00",
        days: &[5],
        after: None,
        mood: Mood::Happy,
    },
    Plan {
        id: "backup",
        who: "nova",
        name: "Back up the robot code",
        note: "",
        group: "Workshop",
        time: "18:00",
        days: &[3],
        after: None,
        mood: Mood::Happy,
    },
    Plan {
        id: "lock",
        who: "nova",
        name: "Lock the workshop",
        note: "",
        group: "Workshop",
        time: "21:00",
        days: EVERY_DAY,
        after: None,
        mood: Mood::Neutral,
    },
];

fn person_of(state: &State, who: &str) -> Option<String> {
    if who == "mochi" {
        return Some("mochi".into());
    }
    let uid = cast::uid(who);
    state
        .people
        .iter()
        .find(|p| p.account.as_deref() == Some(uid.as_str()))
        .map(|p| p.id.clone())
}

fn noon(date: NaiveDate, zone: Tz) -> Result<DateTime<Utc>> {
    scheduled_at(date, "12:00", zone)
}

/// The demo state of one crew at `now`, or `None` for a key that is not a demo crew.
pub fn state(
    crew: &str,
    members: &[CrewMember],
    zone: Tz,
    now: DateTime<Utc>,
) -> Result<Option<State>> {
    let plans = match crew {
        "flat-4b" => FLAT_4B,
        "robotics-club" => ROBOTICS_CLUB,
        _ => return Ok(None),
    };
    let today = today(now, zone);
    let start = today - Duration::days(3);
    let mut state = State::empty(start, zone);
    state.sync_accounts(members, start);
    if crew == "flat-4b" {
        state.people.push(Person {
            id: "mochi".into(),
            name: "Mochi".into(),
            tone: "peach".into(),
            account: None,
            revision: 1,
            created_date: start,
        });
    }
    let owner = cast::USERS
        .iter()
        .find(|u| u.crew == crew && u.owner)
        .map(|u| cast::uid(u.key))
        .unwrap_or_default();
    let created_at = noon(start, zone)?;
    for plan in plans {
        let Some(person_id) = person_of(&state, plan.who) else {
            continue;
        };
        state.edit(
            &owner,
            &Edit {
                operation_id: Uuid::new_v4(),
                expected_revision: 0,
                effective_from: start,
                routine: Routine {
                    id: plan.id.into(),
                    revision: 0,
                    assignee: String::new(),
                    person_id,
                    name: plan.name.into(),
                    note: plan.note.into(),
                    group: plan.group.into(),
                    time: plan.time.into(),
                    days: plan.days.to_vec(),
                    start_date: start,
                    end_date: None,
                    enabled: true,
                    prerequisite: plan.after.map(Into::into),
                },
            },
            created_at,
        )?;
        state.appearances.insert(
            plan.id.into(),
            Appearance {
                mood: plan.mood,
                revision: 1,
            },
        );
    }
    // Three days of history: most things done, Zed's dishes sometimes not.
    for back in (1..=3).rev() {
        let date = today - Duration::days(back);
        state.ensure_day(date, date);
        let skip_dishes = back % 2 == 1;
        if let Some(items) = state.days.get_mut(&date) {
            for item in items.iter_mut() {
                if skip_dishes && item.routine.id == "dishes" {
                    continue;
                }
                let doer = plans
                    .iter()
                    .find(|p| p.id == item.routine.id)
                    .map(|p| if p.who == "mochi" { "pip" } else { p.who })
                    .unwrap_or("pip");
                let at = scheduled_at(date, &item.routine.time, zone)? + Duration::minutes(7);
                item.status = "done".into();
                item.revision += 1;
                item.completed_by = Some(cast::uid(doer));
                item.completed_at = Some(at);
            }
        }
    }
    // Today: Pip has already checked off every one of their tasks, in advance.
    state.ensure_day(today, today);
    if crew == "flat-4b" {
        let pip = cast::uid("pip");
        let pip_person = person_of(&state, "pip");
        let mine: Vec<_> = state
            .days
            .get(&today)
            .into_iter()
            .flatten()
            .filter(|i| Some(&i.routine.person_id) == pip_person.as_ref())
            .map(|i| (i.routine.id.clone(), i.revision))
            .collect();
        for (id, revision) in mine {
            state.mark(
                today,
                &id,
                &pip,
                &Mark {
                    operation_id: Uuid::new_v4(),
                    expected_revision: revision,
                    completed: true,
                    confirmed_completed_by: None,
                },
                now,
            )?;
        }
    }
    Ok(Some(state))
}
