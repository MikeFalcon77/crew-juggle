//! Crew routines: rules and ports. No HTTP, ORM or browser types enter here.
/// Authorized use cases and the cross-gear SDK implementation.
pub(crate) mod application;
/// Avatar revision, replay and image replacement contracts.
pub(crate) mod avatars;
/// Demo crews: Flat 4B (Pip, Zed and Mochi the cat) and the Robotics Club.
pub mod demo;
/// Typed replay identities and accepted results.
mod journal;
/// Reminder grouping, delivery claims and retry policy.
pub(crate) mod reminders;
/// Registered devices eligible for crew notifications.
pub(crate) mod subscriptions;
use async_trait::async_trait;
use chrono::{DateTime, Datelike, LocalResult, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
pub(crate) use journal::{Command, CommandResponse};
use juggle_crews_sdk::{CrewMember, CrewsClientV1, policy::CrewPolicy};
use std::collections::{BTreeMap, HashMap};
use toolkit_security::AccessScope;
use toolkit_security::SecurityContext;
use uuid::Uuid;

/// Parses an IANA zone name such as `Europe/Berlin`.
pub fn parse_zone(name: &str) -> Result<Tz> {
    name.parse()
        .map_err(|_| Error::Invalid("Pick a time zone from the list".into()))
}

/// The crew's calendar date at `now`.
pub fn today(now: DateTime<Utc>, zone: Tz) -> NaiveDate {
    now.with_timezone(&zone).date_naive()
}

/// One instant per clock slot: the earlier repeated hour, or the first valid
/// minute after a daylight-saving gap.
pub fn scheduled_at(date: NaiveDate, time: &str, zone: Tz) -> Result<DateTime<Utc>> {
    let clock = NaiveTime::parse_from_str(time, "%H:%M")
        .map_err(|_| Error::Invalid("Time must look like 08:30".into()))?;
    let wall = date.and_time(clock);
    for minutes in 0..=180 {
        match zone.from_local_datetime(&(wall + chrono::Duration::minutes(minutes))) {
            LocalResult::Single(at) => return Ok(at.with_timezone(&Utc)),
            LocalResult::Ambiguous(a, b) => return Ok(a.min(b).with_timezone(&Utc)),
            LocalResult::None => {}
        }
    }
    Err(Error::Invalid(
        "That time does not exist on this date".into(),
    ))
}

/// A crew clock slot, plus the viewer's local time when their zone differs.
pub fn time_label(date: NaiveDate, time: &str, crew: Tz, viewer: &str) -> Result<String> {
    let viewer = parse_zone(viewer)?;
    let mut label = time.to_string();
    if viewer != crew {
        let local = scheduled_at(date, time, crew)?.with_timezone(&viewer);
        let city = viewer
            .name()
            .rsplit('/')
            .next()
            .unwrap_or("UTC")
            .replace('_', " ");
        label.push_str(&format!(" ({} {city})", local.format("%H:%M")));
        if local.date_naive() != date {
            label.push_str(&format!(" {}", local.format("%b %-d")));
        }
    }
    Ok(label)
}

/// ISO weekday number, Monday = 1 … Sunday = 7.
pub fn weekday(date: NaiveDate) -> u8 {
    date.weekday().number_from_monday() as u8
}

#[derive(Debug, thiserror::Error)]
#[toolkit_macros::domain_model]
/// Domain failures mapped to HTTP or SDK errors at the boundary.
pub enum Error {
    /// The PDP denied the caller access to this crew's data.
    #[error("You are not a member of this crew")]
    Forbidden,
    #[error("{0}")]
    /// The requested change breaks a crew rule.
    Invalid(String),
    #[error("Someone changed this on another device. Refresh and try again.")]
    /// The expected revision or replay fingerprint no longer matches.
    Conflict,
    #[error("Confirm that you want to undo someone else's check-off")]
    /// Undoing another member's completion requires explicit confirmation.
    ConfirmationRequired,
    #[error("Not found")]
    /// The requested record does not exist.
    NotFound,
    #[error("Storage is temporarily unavailable")]
    /// Persistence failed; the cause stays available for diagnostics.
    Storage(#[source] anyhow::Error),
}
/// Result of a crew domain operation.
pub type Result<T> = std::result::Result<T, Error>;

#[async_trait]
/// Port for encrypted notification delivery with typed outcomes.
pub trait PushTransport: Send + Sync {
    async fn send(
        &self,
        subscription: &Subscription,
        message: &juggle_routines_sdk::Notification,
    ) -> Result<juggle_routines_sdk::DeliveryOutcome>;
}

#[async_trait]
/// Port for validating and normalizing uploaded avatar bytes.
pub trait AvatarNormalizer: Send + Sync {
    async fn normalize(&self, bytes: Vec<u8>) -> Result<Vec<u8>>;
}

#[toolkit_macros::domain_model]
/// Current routine versions and crew display data.
pub struct Schedule {
    /// Every version of every routine, oldest first.
    pub versions: Vec<Version>,
    /// Everyone routines can be assigned to.
    pub people: Vec<Person>,
    /// Today in the crew's zone.
    pub today: NaiveDate,
    /// The crew's calendar zone.
    pub timezone: String,
    /// Resolved UTC instants keyed by `date/time`.
    pub scheduled_times: BTreeMap<String, DateTime<Utc>>,
    /// Character moods keyed by routine id.
    pub appearances: HashMap<String, Appearance>,
}

/// Card colors a person can pick.
pub const TONES: [&str; 4] = ["blue", "lilac", "green", "peach"];

#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Someone routines can be assigned to. People with an `account` can sign in;
/// people without one (Mochi the cat) still have routines.
pub struct Person {
    /// Stable id; `m-…` for accounts, a chosen slug otherwise.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Card color, one of [`TONES`].
    pub tone: String,
    /// Identity-provider account linked to this person, set only by the server.
    pub account: Option<String>,
    /// Incremented on every accepted change.
    pub revision: u64,
    /// First crew date on which this person exists.
    pub created_date: NaiveDate,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Revision-guarded, replayable profile change.
pub struct PersonEdit {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
    /// The person as it should be after the change.
    pub person: Person,
}

#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// A repeating task: who, what, when and on which days.
pub struct Routine {
    /// Stable routine id.
    pub id: String,
    /// Incremented on every accepted change.
    pub revision: u64,
    /// The assignee's name on the day a task was created; history keeps it.
    pub assignee: String,
    /// The person the routine is assigned to.
    pub person_id: String,
    /// What to do, e.g. "Take out the trash".
    pub name: String,
    /// Free-form note shown under the name.
    pub note: String,
    /// Optional free label such as "Kitchen" or "Laundry".
    pub group: String,
    /// Crew clock slot in HH:MM format.
    pub time: String,
    /// ISO weekdays (Monday = 1). Empty means every day.
    pub days: Vec<u8>,
    /// First day the routine applies.
    pub start_date: NaiveDate,
    /// Last day, inclusive, when the routine is bounded.
    pub end_date: Option<NaiveDate>,
    /// Disabled routines create no new tasks.
    pub enabled: bool,
    /// Routine that must be done first on the same day ("wash" before "hang").
    pub prerequisite: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// A routine version effective from a particular crew date.
pub struct Version {
    /// First crew date this version applies to.
    pub effective_from: NaiveDate,
    /// The routine as of that date.
    pub routine: Routine,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// One dated task with its completion author and revision.
pub struct Occurrence {
    /// `<date>/<routine id>`.
    pub id: String,
    /// Routine values frozen for this day.
    pub routine: Routine,
    /// Incremented on every check-off or undo.
    pub revision: u64,
    /// `pending`, `done` or `cancelled`.
    pub status: String,
    /// Account that checked this task off.
    pub completed_by: Option<String>,
    /// When the task was checked off.
    pub completed_at: Option<DateTime<Utc>>,
}
#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// Immutable audit event for an accepted check-off or undo.
pub struct Event {
    /// The request that caused the event.
    pub operation_id: Uuid,
    /// The task it changed.
    pub occurrence_id: String,
    /// `complete` or `undo`.
    pub action: String,
    /// Account that made the change.
    pub actor: String,
    /// When the change was accepted.
    pub at: DateTime<Utc>,
}
#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// Private device delivery credentials owned by one account.
pub struct Subscription {
    /// Stable id derived from the endpoint.
    pub id: String,
    /// Owner's account id.
    pub uid: String,
    /// Validated HTTPS Web Push endpoint.
    pub endpoint: String,
    /// Device public encryption key; never log it.
    pub p256dh: String,
    /// Device authentication secret; never log it.
    pub auth: String,
}
#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// Durable reminder attempt and its retry schedule.
pub struct Delivery {
    /// Idempotency key of the reminder: subscription, day and slot.
    pub key: String,
    /// Device the reminder goes to.
    pub subscription_id: String,
    /// Crew date of the reminded tasks.
    pub day: NaiveDate,
    /// Name of the person reminded.
    pub assignee: String,
    /// Id of the person reminded.
    pub person_id: String,
    /// Clock slot of the reminded tasks.
    pub time: String,
    /// Delivery attempts so far.
    pub attempts: u32,
    /// `pending`, `sending`, `sent`, `failed` or `cancelled`.
    pub status: String,
    /// Earliest time of the next attempt.
    pub retry_at: DateTime<Utc>,
    /// Notification tag, so a device shows one reminder per slot.
    pub tag: String,
}
#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// Accepted command fingerprint and cached response for idempotent replay.
pub struct Operation {
    /// Account that sent the command.
    pub actor: String,
    /// The command as accepted.
    pub command: Command,
    /// The answer it got, returned again on replay.
    pub response: CommandResponse,
}

#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// One crew's routines, committed atomically with its operation journal.
pub struct State {
    /// First crew date with data; days before it are empty.
    pub created_date: NaiveDate,
    /// Everyone routines can be assigned to.
    pub people: Vec<Person>,
    /// Per-account display preferences.
    pub preferences: HashMap<String, Preferences>,
    /// Character moods keyed by routine id.
    pub appearances: HashMap<String, Appearance>,
    /// Every version of every routine, oldest first.
    pub versions: Vec<Version>,
    /// Materialized dated tasks; finished and past days are immutable.
    pub days: BTreeMap<NaiveDate, Vec<Occurrence>>,
    /// Audit trail of check-offs and undos.
    pub events: Vec<Event>,
    /// Accepted operations by id, for idempotent replay.
    pub operations: HashMap<Uuid, Operation>,
    /// Devices subscribed to reminders.
    pub subscriptions: Vec<Subscription>,
    /// Reminder attempts and their retry schedule.
    pub deliveries: Vec<Delivery>,
    /// The crew's calendar zone, supplied by the crews gear on every load.
    /// Not persisted: the crew owns it.
    pub zone: Tz,
}
#[derive(Clone, Debug)]
#[toolkit_macros::domain_model]
/// Read model for one crew date.
pub struct Day {
    /// The crew date shown.
    pub date: NaiveDate,
    /// Today in the crew's zone.
    pub today: NaiveDate,
    /// The crew's calendar zone.
    pub timezone: String,
    /// Tasks of the day in clock order.
    pub items: Vec<Occurrence>,
    /// Everyone routines can be assigned to.
    pub people: Vec<Person>,
    /// Server clock, so the client can show lateness correctly.
    pub server_now: DateTime<Utc>,
    /// First crew date with data.
    pub first_date: NaiveDate,
    /// Resolved UTC instants for the day's clock slots.
    pub scheduled_times: BTreeMap<String, DateTime<Utc>>,
    /// Character moods keyed by routine id.
    pub appearances: HashMap<String, Appearance>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[toolkit_macros::domain_model]
/// Expression of a routine's character.
pub enum Mood {
    #[default]
    /// Cheerful card face.
    Happy,
    /// Calm card face.
    Neutral,
    /// Grumpy card face, for chores nobody likes.
    Grumpy,
}
#[derive(Clone, Debug, Default, PartialEq)]
#[toolkit_macros::domain_model]
/// Revisioned character appearance, separate from task history.
pub struct Appearance {
    /// Current mood.
    pub mood: Mood,
    /// Incremented on every accepted change.
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Replayable change to a routine's character.
pub struct AppearanceEdit {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
    /// The new mood.
    pub mood: Mood,
}
#[derive(Clone, Debug, Default, PartialEq)]
#[toolkit_macros::domain_model]
/// Current photo revision and whether image bytes exist.
pub struct AvatarMeta {
    /// Incremented on every accepted change.
    pub revision: u64,
    /// Whether a photo is stored.
    pub has_photo: bool,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Replayable photo replacement or deletion.
pub struct AvatarEdit {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
}
#[async_trait]
/// Port for atomic avatar metadata, image and journal changes.
pub trait AvatarStore: Send + Sync {
    async fn metadata(&self, scope: &AccessScope) -> Result<HashMap<String, AvatarMeta>>;
    async fn image(&self, scope: &AccessScope, person_id: &str) -> Result<Vec<u8>>;
    async fn change(
        &self,
        scope: &AccessScope,
        person_id: &str,
        actor: &str,
        request: &AvatarEdit,
        image: Option<Vec<u8>>,
    ) -> Result<AvatarMeta>;
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// One account's display preferences.
pub struct Preferences {
    /// Zone this person wants times shown in; the crew calendar stays the crew's.
    pub timezone: String,
    /// Zero until the person saves a choice.
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Replayable change to one account's display zone.
pub struct PreferencesEdit {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
    /// IANA zone to show times in.
    pub timezone: String,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Check-off or undo with revision and optional author confirmation.
pub struct Mark {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
    /// Requested state; false asks for an undo.
    pub completed: bool,
    /// The other member whose check-off the caller confirmed undoing.
    pub confirmed_completed_by: Option<String>,
}
#[derive(Clone, Debug, PartialEq)]
#[toolkit_macros::domain_model]
/// Replayable routine change with its effective crew date.
pub struct Edit {
    /// Client-generated id; a retried request with the same id is applied once.
    pub operation_id: Uuid,
    /// Revision the client last saw; a different current revision is a conflict.
    pub expected_revision: u64,
    /// First crew date the change applies to; history before it stays.
    pub effective_from: NaiveDate,
    /// The routine as it should be from that date.
    pub routine: Routine,
}
#[async_trait]
/// Port for revision-guarded crew state, one row per crew.
pub trait Store: Send + Sync {
    /// The crew's state; `Error::NotFound` when the crew has none yet.
    async fn read(&self, scope: &AccessScope) -> Result<(i64, State)>;
    /// Creates the crew's first state. Returns false if it already exists.
    async fn create(&self, scope: &AccessScope, state: &State) -> Result<bool>;
    async fn compare_and_swap(
        &self,
        scope: &AccessScope,
        revision: i64,
        state: &State,
    ) -> Result<bool>;
}

fn short_id(seed: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_URL, seed.as_bytes())
        .simple()
        .to_string()[..10]
        .to_string()
}

impl State {
    /// A crew with nobody and nothing scheduled yet.
    pub fn empty(created_date: NaiveDate, zone: Tz) -> Self {
        Self {
            created_date,
            people: vec![],
            preferences: HashMap::new(),
            appearances: HashMap::new(),
            versions: vec![],
            days: BTreeMap::new(),
            events: vec![],
            operations: HashMap::new(),
            subscriptions: vec![],
            deliveries: vec![],
            zone,
        }
    }

    /// Today in the crew's zone at `now`.
    pub fn today(&self, now: DateTime<Utc>) -> NaiveDate {
        today(now, self.zone)
    }

    fn record(&mut self, id: Uuid, actor: &str, command: Command, response: CommandResponse) {
        self.operations.insert(
            id,
            Operation {
                actor: actor.into(),
                command,
                response,
            },
        );
    }

    /// Gives every member account a person to assign routines to.
    /// Returns true when someone was added.
    pub fn sync_accounts(&mut self, members: &[CrewMember], date: NaiveDate) -> bool {
        let mut changed = false;
        for member in members {
            if self
                .people
                .iter()
                .any(|p| p.account.as_deref() == Some(member.uid.as_str()))
            {
                continue;
            }
            let tone = TONES[self.people.len() % TONES.len()];
            self.people.push(Person {
                id: format!("m-{}", short_id(&member.uid)),
                name: member.display_name.clone(),
                tone: tone.into(),
                account: Some(member.uid.clone()),
                revision: 1,
                created_date: date.max(self.created_date),
            });
            changed = true;
        }
        changed
    }

    /// Applies a revision-guarded character change without touching history.
    pub fn edit_appearance(
        &mut self,
        id: &str,
        actor: &str,
        request: &AppearanceEdit,
    ) -> Result<Appearance> {
        let command = Command::Appearance(id.into(), request.clone());
        if let Some(response) = self.replay(request.operation_id, actor, &command)? {
            return match response {
                CommandResponse::Appearance(value) => Ok(value),
                _ => Err(Error::Conflict),
            };
        }
        if !self.versions.iter().any(|v| v.routine.id == id) {
            return Err(Error::NotFound);
        }
        if self.appearances.get(id).map_or(0, |a| a.revision) != request.expected_revision {
            return Err(Error::Conflict);
        }
        let appearance = Appearance {
            mood: request.mood,
            revision: request.expected_revision + 1,
        };
        self.appearances.insert(id.into(), appearance.clone());
        self.record(
            request.operation_id,
            actor,
            command,
            CommandResponse::Appearance(appearance.clone()),
        );
        Ok(appearance)
    }

    /// One account's preferences, defaulting to the crew zone.
    pub fn preferences_for(&self, actor: &str) -> Preferences {
        self.preferences
            .get(actor)
            .cloned()
            .unwrap_or_else(|| Preferences {
                timezone: self.zone.name().into(),
                revision: 0,
            })
    }

    /// Applies or replays a revision-guarded display-zone change.
    pub fn edit_preferences(
        &mut self,
        actor: &str,
        request: &PreferencesEdit,
    ) -> Result<Preferences> {
        let command = Command::Preferences(request.clone());
        if let Some(response) = self.replay(request.operation_id, actor, &command)? {
            return match response {
                CommandResponse::Preferences(value) => Ok(value),
                _ => Err(Error::Conflict),
            };
        }
        let zone = parse_zone(&request.timezone)?;
        if self.preferences_for(actor).revision != request.expected_revision {
            return Err(Error::Conflict);
        }
        let preferences = Preferences {
            timezone: zone.name().into(),
            revision: request.expected_revision + 1,
        };
        self.preferences.insert(actor.into(), preferences.clone());
        self.record(
            request.operation_id,
            actor,
            command,
            CommandResponse::Preferences(preferences.clone()),
        );
        Ok(preferences)
    }

    /// Applies or replays a revision-guarded profile edit. The account link
    /// is never taken from the request.
    pub fn edit_person(
        &mut self,
        actor: &str,
        request: &PersonEdit,
        now: DateTime<Utc>,
    ) -> Result<Person> {
        let command = Command::Person(request.clone());
        if let Some(response) = self.replay(request.operation_id, actor, &command)? {
            return match response {
                CommandResponse::Person(value) => Ok(value),
                _ => Err(Error::Conflict),
            };
        }
        let today = self.today(now);
        let mut p = request.person.clone();
        p.name = p.name.trim().into();
        if p.name.is_empty()
            || p.name.chars().count() > 80
            || !TONES.contains(&p.tone.as_str())
            || p.id.is_empty()
            || p.id.len() > 100
            || !p.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        {
            return Err(Error::Invalid("Check the name and card color".into()));
        }
        if let Some(previous) = self.people.iter().find(|old| old.id == p.id) {
            if previous.revision != request.expected_revision {
                return Err(Error::Conflict);
            }
            p.created_date = previous.created_date;
            p.account = previous.account.clone();
        } else {
            if request.expected_revision != 0 {
                return Err(Error::Conflict);
            }
            if self.people.len() >= 20 {
                return Err(Error::Invalid("A crew can have up to 20 people".into()));
            }
            p.created_date = today;
            p.account = None;
        }
        p.revision = request.expected_revision + 1;
        if let Some(index) = self.people.iter().position(|old| old.id == p.id) {
            self.people[index] = p.clone();
        } else {
            self.people.push(p.clone());
        }
        self.ensure_day(today, today);
        self.record(
            request.operation_id,
            actor,
            command,
            CommandResponse::Person(p.clone()),
        );
        Ok(p)
    }

    /// Routine versions in effect on `day`.
    pub fn routines_on(&self, day: NaiveDate) -> Vec<Routine> {
        let mut found: BTreeMap<&str, &Version> = BTreeMap::new();
        for version in &self.versions {
            if version.effective_from > day {
                continue;
            }
            let id = version.routine.id.as_str();
            if found.get(id).is_none_or(|v| {
                (version.effective_from, version.routine.revision)
                    > (v.effective_from, v.routine.revision)
            }) {
                found.insert(id, version);
            }
        }
        found.values().map(|v| v.routine.clone()).collect()
    }

    /// The latest known version of each routine.
    pub fn latest_routines(&self) -> Vec<Version> {
        let mut latest: BTreeMap<&str, &Version> = BTreeMap::new();
        for version in &self.versions {
            if latest
                .get(version.routine.id.as_str())
                .is_none_or(|v| version.routine.revision > v.routine.revision)
            {
                latest.insert(&version.routine.id, version);
            }
        }
        latest.values().map(|v| (*v).clone()).collect()
    }

    /// Reconciles only today's unfinished items; finished and past days are immutable.
    pub fn ensure_day(&mut self, date: NaiveDate, current: NaiveDate) {
        if date < self.created_date || date > current {
            return;
        }
        if date < current && self.days.contains_key(&date) {
            return;
        }
        let day_of_week = weekday(date);
        let active: Vec<_> = self
            .routines_on(date)
            .into_iter()
            .filter_map(|mut r| {
                let person = self
                    .people
                    .iter()
                    .find(|person| person.id == r.person_id && person.created_date <= date)?;
                r.assignee = person.name.clone();
                Some(r)
            })
            .filter(|r| {
                r.enabled
                    && r.start_date <= date
                    && r.end_date.is_none_or(|end| date <= end)
                    && (r.days.is_empty() || r.days.contains(&day_of_week))
            })
            .collect();
        let items = self.days.entry(date).or_default();
        for item in items.iter_mut() {
            if item.status == "done" {
                continue;
            }
            match active.iter().find(|r| r.id == item.routine.id) {
                Some(r) if r != &item.routine || item.status == "cancelled" => {
                    item.routine = r.clone();
                    item.status = "pending".into();
                    item.revision += 1;
                }
                None if item.status != "cancelled" => {
                    item.status = "cancelled".into();
                    item.revision += 1;
                }
                _ => {}
            }
        }
        for r in active {
            if !items.iter().any(|item| item.routine.id == r.id) {
                items.push(Occurrence {
                    id: format!("{date}/{}", r.id),
                    routine: r,
                    revision: 1,
                    status: "pending".into(),
                    completed_by: None,
                    completed_at: None,
                });
            }
        }
        items.sort_by(|a, b| {
            (
                &a.routine.time,
                &a.routine.group,
                a.routine.prerequisite.is_some(),
                &a.routine.id,
            )
                .cmp(&(
                    &b.routine.time,
                    &b.routine.group,
                    b.routine.prerequisite.is_some(),
                    &b.routine.id,
                ))
        });
    }

    fn replay(&self, id: Uuid, actor: &str, command: &Command) -> Result<Option<CommandResponse>> {
        if let Some(op) = self.operations.get(&id) {
            if op.actor != actor || op.command != *command {
                return Err(Error::Conflict);
            }
            return Ok(Some(op.response.clone()));
        }
        Ok(None)
    }

    fn routine_name(&self, id: &str) -> String {
        self.latest_routines()
            .into_iter()
            .find(|v| v.routine.id == id)
            .map(|v| v.routine.name)
            .unwrap_or_else(|| "the first step".into())
    }

    /// Checks a task off, or undoes it with confirmation, enforcing step order and replay.
    pub fn mark(
        &mut self,
        date: NaiveDate,
        routine_id: &str,
        actor: &str,
        request: &Mark,
        now: DateTime<Utc>,
    ) -> Result<Occurrence> {
        let fingerprint = format!(
            "mark/{date}/{routine_id}/{}/{}/{:?}",
            request.completed, request.expected_revision, request.confirmed_completed_by
        );
        let command = Command::Mark(fingerprint);
        if let Some(response) = self.replay(request.operation_id, actor, &command)? {
            return match response {
                CommandResponse::Occurrence(value) => Ok(value),
                _ => Err(Error::Conflict),
            };
        }
        let today = self.today(now);
        if date != today {
            return Err(Error::Invalid(
                "Only today's tasks can be checked off".into(),
            ));
        }
        self.ensure_day(date, today);
        let prerequisite_name = self
            .days
            .get(&date)
            .and_then(|items| items.iter().find(|i| i.routine.id == routine_id))
            .and_then(|i| i.routine.prerequisite.clone())
            .map(|id| self.routine_name(&id));
        let dependent_name = self
            .days
            .get(&date)
            .and_then(|items| {
                items.iter().find(|i| {
                    i.routine.prerequisite.as_deref() == Some(routine_id) && i.status == "done"
                })
            })
            .map(|i| i.routine.name.clone());
        let items = self.days.get_mut(&date).ok_or(Error::NotFound)?;
        let index = items
            .iter()
            .position(|i| i.routine.id == routine_id)
            .ok_or(Error::NotFound)?;
        let item = &items[index];
        if item.revision != request.expected_revision {
            return Err(Error::Conflict);
        }
        if !request.completed
            && item.status == "done"
            && item.completed_by.as_deref() != Some(actor)
            && (item.completed_by.is_none() || request.confirmed_completed_by != item.completed_by)
        {
            return Err(Error::ConfirmationRequired);
        }
        if item.status == "cancelled" {
            return Err(Error::Invalid(
                "This task was removed from today's plan".into(),
            ));
        }
        if request.completed
            && let Some(prerequisite) = &item.routine.prerequisite
            && !items
                .iter()
                .any(|i| &i.routine.id == prerequisite && i.status == "done")
        {
            return Err(Error::Invalid(format!(
                "Check off “{}” first",
                prerequisite_name.unwrap_or_default()
            )));
        }
        if !request.completed
            && let Some(name) = dependent_name
        {
            return Err(Error::Invalid(format!("Undo “{name}” first")));
        }
        let item = &mut items[index];
        if (item.status == "done") != request.completed {
            item.status = if request.completed { "done" } else { "pending" }.into();
            item.revision += 1;
            item.completed_by = request.completed.then(|| actor.into());
            item.completed_at = request.completed.then_some(now);
            self.events.push(Event {
                operation_id: request.operation_id,
                occurrence_id: item.id.clone(),
                action: if request.completed {
                    "complete"
                } else {
                    "undo"
                }
                .into(),
                actor: actor.into(),
                at: now,
            });
        }
        let response = item.clone();
        self.record(
            request.operation_id,
            actor,
            command,
            CommandResponse::Occurrence(response.clone()),
        );
        Ok(response)
    }

    /// Applies or replays a validated routine change.
    pub fn edit(&mut self, actor: &str, request: &Edit, now: DateTime<Utc>) -> Result<Version> {
        let command = Command::Routine(request.clone());
        if let Some(response) = self.replay(request.operation_id, actor, &command)? {
            return match response {
                CommandResponse::Version(value) => Ok(value),
                _ => Err(Error::Conflict),
            };
        }
        let today = self.today(now);
        let mut r = request.routine.clone();
        let person = self
            .people
            .iter()
            .find(|person| person.id == r.person_id)
            .ok_or_else(|| Error::Invalid("Pick someone from your crew".into()))?;
        r.assignee = person.name.clone();
        r.days.sort_unstable();
        r.days.dedup();
        validate_routine(&r)?;
        if request.effective_from < today {
            return Err(Error::Invalid("Changes can start today or later".into()));
        }
        let latest = self
            .latest_routines()
            .into_iter()
            .find(|v| v.routine.id == r.id);
        if latest.as_ref().map_or(0, |v| v.routine.revision) != request.expected_revision {
            return Err(Error::Conflict);
        }
        if latest
            .as_ref()
            .is_some_and(|v| v.effective_from > request.effective_from)
        {
            return Err(Error::Invalid(
                "A later change is already scheduled for this routine".into(),
            ));
        }
        match (&latest, &r.prerequisite) {
            (Some(v), _) if v.routine.prerequisite != r.prerequisite => {
                return Err(Error::Invalid(
                    "The order of steps cannot change after a routine exists".into(),
                ));
            }
            (None, Some(first)) => {
                // A new second step: the first step must exist and stand alone.
                let first = self
                    .latest_routines()
                    .into_iter()
                    .find(|v| &v.routine.id == first)
                    .ok_or_else(|| Error::Invalid("Pick an existing first step".into()))?;
                let taken = self
                    .latest_routines()
                    .iter()
                    .any(|v| v.routine.prerequisite.as_deref() == Some(first.routine.id.as_str()));
                if first.routine.prerequisite.is_some() || taken {
                    return Err(Error::Invalid(
                        "Each routine can have one second step, two steps at most".into(),
                    ));
                }
                // A pair shares one slot: who, when and on which days.
                r.person_id = first.routine.person_id.clone();
                r.assignee = first.routine.assignee.clone();
                r.time = first.routine.time.clone();
                r.days = first.routine.days.clone();
                r.group = first.routine.group.clone();
                r.start_date = first.routine.start_date;
                r.end_date = first.routine.end_date;
                r.enabled = first.routine.enabled;
            }
            _ => {}
        }
        r.revision = request.expected_revision + 1;
        // Keep a two-step pair on the same slot; names and notes stay separate.
        let pair_id = r.prerequisite.clone().or_else(|| {
            self.latest_routines()
                .into_iter()
                .find(|v| v.routine.prerequisite.as_deref() == Some(&r.id))
                .map(|v| v.routine.id)
        });
        if latest
            .as_ref()
            .is_some_and(|version| version.routine.person_id != r.person_id)
            && request.effective_from == today
            && self.days.get(&today).into_iter().flatten().any(|item| {
                item.status == "done"
                    && (item.routine.id == r.id
                        || pair_id.as_deref() == Some(item.routine.id.as_str()))
            })
        {
            return Err(Error::Invalid(
                "Part of this routine is already done today. Change who does it from tomorrow."
                    .into(),
            ));
        }
        if let Some(pair_id) = pair_id.filter(|_| latest.is_some()) {
            let v = self
                .latest_routines()
                .into_iter()
                .find(|v| v.routine.id == pair_id)
                .ok_or(Error::NotFound)?;
            if v.effective_from > request.effective_from {
                return Err(Error::Conflict);
            }
            let mut pair = v.routine;
            pair.person_id = r.person_id.clone();
            pair.assignee = r.assignee.clone();
            pair.time = r.time.clone();
            pair.days = r.days.clone();
            pair.group = r.group.clone();
            pair.start_date = r.start_date;
            pair.end_date = r.end_date;
            pair.enabled = r.enabled;
            pair.revision += 1;
            self.versions.push(Version {
                effective_from: request.effective_from,
                routine: pair,
            });
        }
        let version = Version {
            effective_from: request.effective_from,
            routine: r,
        };
        self.versions.push(version.clone());
        self.ensure_day(today, today);
        self.record(
            request.operation_id,
            actor,
            command,
            CommandResponse::Version(version.clone()),
        );
        Ok(version)
    }
}

/// Validates identity, clock slot, days and dates before accepting an edit.
pub fn validate_routine(r: &Routine) -> Result<()> {
    if r.assignee.trim().is_empty()
        || r.name.trim().is_empty()
        || r.name.chars().count() > 120
        || r.note.chars().count() > 1000
        || r.group.chars().count() > 60
    {
        return Err(Error::Invalid("Check who, the name and the note".into()));
    }
    if r.id.is_empty()
        || r.id.len() > 100
        || !r.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    {
        return Err(Error::Invalid("Invalid routine id".into()));
    }
    if NaiveTime::parse_from_str(&r.time, "%H:%M").is_err() || r.time.len() != 5 {
        return Err(Error::Invalid("Time must look like 08:30".into()));
    }
    if r.days.iter().any(|d| !(1..=7).contains(d)) {
        return Err(Error::Invalid(
            "Days run from Monday (1) to Sunday (7)".into(),
        ));
    }
    if r.end_date.is_some_and(|end| end < r.start_date) {
        return Err(Error::Invalid(
            "The last day is before the first day".into(),
        ));
    }
    Ok(())
}

/// A crew's state, loaded with everything needed to act on it.
#[toolkit_macros::domain_model]
pub(crate) struct Loaded {
    /// Scope the PDP granted for this crew.
    pub scope: AccessScope,
    /// Revision the state was read at.
    pub revision: i64,
    /// The crew's state.
    pub state: State,
}

#[toolkit_macros::domain_model]
/// Application operations over revision-guarded crew state.
pub struct Service {
    /// Where the crew state is stored.
    pub store: std::sync::Arc<dyn Store>,
    /// Evaluates the actual caller before every operation (this gear is the PEP).
    pub policy: CrewPolicy,
    /// Crew name, zone and members, from the crews gear via ClientHub.
    pub crews: std::sync::Arc<dyn CrewsClientV1>,
}
impl Service {
    /// Evaluates the caller through the PDP and compiles the crew scope.
    pub(crate) async fn scope(&self, ctx: &SecurityContext) -> Result<AccessScope> {
        self.policy.scope(ctx).await.map_err(|_| Error::Forbidden)
    }

    /// Reads the crew's state, creating it on first use and making sure every
    /// member account has a person.
    pub(crate) async fn load(&self, ctx: &SecurityContext, now: DateTime<Utc>) -> Result<Loaded> {
        let scope = self.scope(ctx).await?;
        let crew = self.crews.crew(ctx).await.map_err(|_| Error::Forbidden)?;
        let members = self
            .crews
            .members(ctx)
            .await
            .map_err(|_| Error::Forbidden)?;
        let zone = parse_zone(&crew.timezone)?;
        for _ in 0..8 {
            let (revision, mut state) = match self.store.read(&scope).await {
                Ok(found) => found,
                Err(Error::NotFound) => {
                    let fresh = State::empty(today(now, zone), zone);
                    self.store.create(&scope, &fresh).await?;
                    continue;
                }
                Err(error) => return Err(error),
            };
            state.zone = zone;
            if state.sync_accounts(&members, state.today(now)) {
                self.store
                    .compare_and_swap(&scope, revision, &state)
                    .await?;
                continue;
            }
            return Ok(Loaded {
                scope,
                revision,
                state,
            });
        }
        Err(Error::Conflict)
    }

    /// The crew's state under the caller's PDP-derived scope.
    pub async fn read(&self, ctx: &SecurityContext) -> Result<(i64, State)> {
        let loaded = self.load(ctx, Utc::now()).await?;
        Ok((loaded.revision, loaded.state))
    }

    /// The account behind `ctx`, if it belongs to the crew.
    pub(crate) async fn member(&self, ctx: &SecurityContext) -> Result<CrewMember> {
        self.crews
            .members(ctx)
            .await
            .map_err(|_| Error::Forbidden)?
            .into_iter()
            .find(|m| m.subject_id == ctx.subject_id())
            .ok_or(Error::Forbidden)
    }

    /// Current routine versions and clock slots.
    pub async fn schedule(&self, ctx: &SecurityContext, now: DateTime<Utc>) -> Result<Schedule> {
        let state = self.load(ctx, now).await?.state;
        let today = state.today(now);
        let versions = state.latest_routines();
        let scheduled_times = versions
            .iter()
            .map(|v| {
                let date = v.effective_from.max(today).max(v.routine.start_date);
                Ok((
                    format!("{date}/{}", v.routine.time),
                    scheduled_at(date, &v.routine.time, state.zone)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Schedule {
            versions,
            timezone: state.zone.name().into(),
            people: state.people,
            today,
            scheduled_times,
            appearances: state.appearances,
        })
    }

    /// Retries a state change under optimistic revision control.
    pub async fn mutate<T: Send>(
        &self,
        ctx: &SecurityContext,
        change: impl Fn(&mut State) -> Result<T> + Send + Sync,
    ) -> Result<T> {
        for attempt in 0..8 {
            let Loaded {
                scope,
                revision,
                mut state,
            } = self.load(ctx, Utc::now()).await?;
            let result = change(&mut state)?;
            if self
                .store
                .compare_and_swap(&scope, revision, &state)
                .await?
            {
                return Ok(result);
            }
            tokio::time::sleep(std::time::Duration::from_millis(5 * (attempt + 1))).await;
        }
        Err(Error::Conflict)
    }

    /// One crew date: refuses future days and keeps past snapshots intact.
    pub async fn day(
        &self,
        ctx: &SecurityContext,
        date: Option<NaiveDate>,
        now: DateTime<Utc>,
    ) -> Result<Day> {
        let state = self.load(ctx, now).await?.state;
        let today = state.today(now);
        let date = date.unwrap_or(today);
        if date > today {
            return Err(Error::Invalid(
                "Future days live in the routine editor".into(),
            ));
        }
        let mut reconciled = state.clone();
        reconciled.ensure_day(date, today);
        let state = if state.days.get(&date) != reconciled.days.get(&date) {
            self.mutate(ctx, |s| {
                s.ensure_day(date, today);
                Ok(())
            })
            .await?;
            self.load(ctx, now).await?.state
        } else {
            state
        };
        Ok(Day {
            date,
            today,
            timezone: state.zone.name().into(),
            items: state.days.get(&date).cloned().unwrap_or_default(),
            people: state
                .people
                .iter()
                .filter(|p| p.created_date <= date)
                .map(|p| {
                    let mut person = p.clone();
                    if date < today
                        && let Some(snapshot) = state
                            .days
                            .get(&date)
                            .into_iter()
                            .flatten()
                            .find(|i| i.routine.person_id == p.id)
                    {
                        person.name = snapshot.routine.assignee.clone();
                    }
                    person
                })
                .collect(),
            server_now: now,
            first_date: state.created_date,
            scheduled_times: state
                .days
                .get(&date)
                .into_iter()
                .flatten()
                .map(|item| {
                    Ok((
                        item.routine.time.clone(),
                        scheduled_at(date, &item.routine.time, state.zone)?,
                    ))
                })
                .collect::<Result<_>>()?,
            appearances: state.appearances,
        })
    }
}

#[cfg(test)]
#[path = "../tests/domain.rs"]
mod regression_tests;
