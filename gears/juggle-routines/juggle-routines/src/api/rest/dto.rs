//! HTTP-only DTOs, mapped explicitly to and from domain types.
use crate::domain;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub tone: String,
    /// Set by the server for people who can sign in; ignored in requests.
    pub account: Option<String>,
    pub revision: u64,
    pub created_date: NaiveDate,
}
impl From<domain::Person> for Person {
    fn from(value: domain::Person) -> Self {
        Self {
            id: value.id,
            name: value.name,
            tone: value.tone,
            account: value.account,
            revision: value.revision,
            created_date: value.created_date,
        }
    }
}
impl From<Person> for domain::Person {
    fn from(value: Person) -> Self {
        Self {
            id: value.id,
            name: value.name,
            tone: value.tone,
            account: value.account,
            revision: value.revision,
            created_date: value.created_date,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct PersonEdit {
    pub operation_id: Uuid,
    pub expected_revision: u64,
    pub person: Person,
}
impl From<domain::PersonEdit> for PersonEdit {
    fn from(value: domain::PersonEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            person: value.person.into(),
        }
    }
}
impl From<PersonEdit> for domain::PersonEdit {
    fn from(value: PersonEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            person: value.person.into(),
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Routine {
    pub id: String,
    pub revision: u64,
    pub assignee: String,
    #[serde(default)]
    pub person_id: String,
    pub name: String,
    pub note: String,
    pub group: String,
    pub time: String,
    /// ISO weekdays, Monday = 1. Empty means every day.
    #[serde(default)]
    pub days: Vec<u8>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub enabled: bool,
    pub prerequisite: Option<String>,
}
impl From<domain::Routine> for Routine {
    fn from(value: domain::Routine) -> Self {
        Self {
            id: value.id,
            revision: value.revision,
            assignee: value.assignee,
            person_id: value.person_id,
            name: value.name,
            note: value.note,
            group: value.group,
            time: value.time,
            days: value.days,
            start_date: value.start_date,
            end_date: value.end_date,
            enabled: value.enabled,
            prerequisite: value.prerequisite,
        }
    }
}
impl From<Routine> for domain::Routine {
    fn from(value: Routine) -> Self {
        Self {
            id: value.id,
            revision: value.revision,
            assignee: value.assignee,
            person_id: value.person_id,
            name: value.name,
            note: value.note,
            group: value.group,
            time: value.time,
            days: value.days,
            start_date: value.start_date,
            end_date: value.end_date,
            enabled: value.enabled,
            prerequisite: value.prerequisite,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Version {
    pub effective_from: NaiveDate,
    pub routine: Routine,
}
impl From<domain::Version> for Version {
    fn from(value: domain::Version) -> Self {
        Self {
            effective_from: value.effective_from,
            routine: value.routine.into(),
        }
    }
}
impl From<Version> for domain::Version {
    fn from(value: Version) -> Self {
        Self {
            effective_from: value.effective_from,
            routine: value.routine.into(),
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Occurrence {
    pub id: String,
    pub routine: Routine,
    pub revision: u64,
    pub status: String,
    pub completed_by: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
}
impl From<domain::Occurrence> for Occurrence {
    fn from(value: domain::Occurrence) -> Self {
        Self {
            id: value.id,
            routine: value.routine.into(),
            revision: value.revision,
            status: value.status,
            completed_by: value.completed_by,
            completed_at: value.completed_at,
        }
    }
}
impl From<Occurrence> for domain::Occurrence {
    fn from(value: Occurrence) -> Self {
        Self {
            id: value.id,
            routine: value.routine.into(),
            revision: value.revision,
            status: value.status,
            completed_by: value.completed_by,
            completed_at: value.completed_at,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Day {
    pub date: NaiveDate,
    pub today: NaiveDate,
    pub timezone: String,
    pub items: Vec<Occurrence>,
    pub people: Vec<Person>,
    pub server_now: DateTime<Utc>,
    pub first_date: NaiveDate,
    pub scheduled_times: BTreeMap<String, DateTime<Utc>>,
    pub appearances: HashMap<String, Appearance>,
}
impl From<domain::Day> for Day {
    fn from(value: domain::Day) -> Self {
        Self {
            date: value.date,
            today: value.today,
            timezone: value.timezone,
            items: value.items.into_iter().map(|value| value.into()).collect(),
            people: value.people.into_iter().map(|value| value.into()).collect(),
            server_now: value.server_now,
            first_date: value.first_date,
            scheduled_times: value.scheduled_times,
            appearances: value
                .appearances
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}
impl From<Day> for domain::Day {
    fn from(value: Day) -> Self {
        Self {
            date: value.date,
            today: value.today,
            timezone: value.timezone,
            items: value.items.into_iter().map(|value| value.into()).collect(),
            people: value.people.into_iter().map(|value| value.into()).collect(),
            server_now: value.server_now,
            first_date: value.first_date,
            scheduled_times: value.scheduled_times,
            appearances: value
                .appearances
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug, Default)]
pub enum Mood {
    #[default]
    Happy,
    Neutral,
    Grumpy,
}
impl From<domain::Mood> for Mood {
    fn from(value: domain::Mood) -> Self {
        match value {
            domain::Mood::Happy => Self::Happy,
            domain::Mood::Neutral => Self::Neutral,
            domain::Mood::Grumpy => Self::Grumpy,
        }
    }
}
impl From<Mood> for domain::Mood {
    fn from(value: Mood) -> Self {
        match value {
            Mood::Happy => Self::Happy,
            Mood::Neutral => Self::Neutral,
            Mood::Grumpy => Self::Grumpy,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug, Default)]
pub struct Appearance {
    pub mood: Mood,
    pub revision: u64,
}
impl From<domain::Appearance> for Appearance {
    fn from(value: domain::Appearance) -> Self {
        Self {
            mood: value.mood.into(),
            revision: value.revision,
        }
    }
}
impl From<Appearance> for domain::Appearance {
    fn from(value: Appearance) -> Self {
        Self {
            mood: value.mood.into(),
            revision: value.revision,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct AppearanceEdit {
    pub operation_id: Uuid,
    pub expected_revision: u64,
    pub mood: Mood,
}
impl From<domain::AppearanceEdit> for AppearanceEdit {
    fn from(value: domain::AppearanceEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            mood: value.mood.into(),
        }
    }
}
impl From<AppearanceEdit> for domain::AppearanceEdit {
    fn from(value: AppearanceEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            mood: value.mood.into(),
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug, Default)]
pub struct AvatarMeta {
    pub revision: u64,
    pub has_photo: bool,
}
impl From<domain::AvatarMeta> for AvatarMeta {
    fn from(value: domain::AvatarMeta) -> Self {
        Self {
            revision: value.revision,
            has_photo: value.has_photo,
        }
    }
}
impl From<AvatarMeta> for domain::AvatarMeta {
    fn from(value: AvatarMeta) -> Self {
        Self {
            revision: value.revision,
            has_photo: value.has_photo,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct AvatarEdit {
    pub operation_id: Uuid,
    pub expected_revision: u64,
}
impl From<domain::AvatarEdit> for AvatarEdit {
    fn from(value: domain::AvatarEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
        }
    }
}
impl From<AvatarEdit> for domain::AvatarEdit {
    fn from(value: AvatarEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Preferences {
    pub timezone: String,
    pub revision: u64,
}
impl From<domain::Preferences> for Preferences {
    fn from(value: domain::Preferences) -> Self {
        Self {
            timezone: value.timezone,
            revision: value.revision,
        }
    }
}
impl From<Preferences> for domain::Preferences {
    fn from(value: Preferences) -> Self {
        Self {
            timezone: value.timezone,
            revision: value.revision,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct PreferencesEdit {
    pub operation_id: Uuid,
    pub expected_revision: u64,
    pub timezone: String,
}
impl From<domain::PreferencesEdit> for PreferencesEdit {
    fn from(value: domain::PreferencesEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            timezone: value.timezone,
        }
    }
}
impl From<PreferencesEdit> for domain::PreferencesEdit {
    fn from(value: PreferencesEdit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            timezone: value.timezone,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Mark {
    pub operation_id: Uuid,
    pub expected_revision: u64,
    pub completed: bool,
    #[serde(default)]
    pub confirmed_completed_by: Option<String>,
}
impl From<domain::Mark> for Mark {
    fn from(value: domain::Mark) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            completed: value.completed,
            confirmed_completed_by: value.confirmed_completed_by,
        }
    }
}
impl From<Mark> for domain::Mark {
    fn from(value: Mark) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            completed: value.completed,
            confirmed_completed_by: value.confirmed_completed_by,
        }
    }
}
#[toolkit_macros::api_dto(request, response)]
#[derive(Clone, Debug)]
pub struct Edit {
    pub operation_id: Uuid,
    pub expected_revision: u64,
    pub effective_from: NaiveDate,
    pub routine: Routine,
}
impl From<domain::Edit> for Edit {
    fn from(value: domain::Edit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            effective_from: value.effective_from,
            routine: value.routine.into(),
        }
    }
}
impl From<Edit> for domain::Edit {
    fn from(value: Edit) -> Self {
        Self {
            operation_id: value.operation_id,
            expected_revision: value.expected_revision,
            effective_from: value.effective_from,
            routine: value.routine.into(),
        }
    }
}
#[toolkit_macros::api_dto(response)]
pub struct Schedule {
    pub versions: Vec<Version>,
    pub people: Vec<Person>,
    pub today: NaiveDate,
    pub timezone: String,
    pub scheduled_times: BTreeMap<String, DateTime<Utc>>,
    pub appearances: HashMap<String, Appearance>,
}
impl From<domain::Schedule> for Schedule {
    fn from(value: domain::Schedule) -> Self {
        Self {
            versions: value.versions.into_iter().map(Into::into).collect(),
            people: value.people.into_iter().map(Into::into).collect(),
            today: value.today,
            timezone: value.timezone,
            scheduled_times: value.scheduled_times,
            appearances: value
                .appearances
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
        }
    }
}

/// Your account, the crew's zone and today in it.
#[toolkit_macros::api_dto(response)]
pub struct MeView {
    /// Identity-provider account id.
    pub uid: String,
    /// The crew's IANA zone.
    pub timezone: String,
    /// Today in the crew's zone.
    pub today: NaiveDate,
    /// Public VAPID key for subscribing this browser to reminders.
    pub vapid_public_key: String,
}

/// A crewmate's display zone.
#[toolkit_macros::api_dto(response)]
pub struct MemberPreferences {
    /// Identity-provider account id.
    pub uid: String,
    /// Display name.
    pub name: String,
    /// Their preferences.
    pub preferences: Preferences,
}

/// Your preferences and your crewmates'.
#[toolkit_macros::api_dto(response)]
pub struct PreferencesView {
    /// Yours.
    pub preferences: Preferences,
    /// Everyone's, including yours.
    pub members: Vec<MemberPreferences>,
}

/// Which day to show.
#[toolkit_macros::api_dto(request)]
pub struct DayQuery {
    /// A date in the crew's calendar; today when absent.
    pub date: Option<NaiveDate>,
}

/// One day with the photos of the people in it.
#[toolkit_macros::api_dto(response)]
pub struct DayView {
    /// The day.
    #[serde(flatten)]
    pub day: Day,
    /// Photo metadata by person id.
    pub avatars: HashMap<String, AvatarMeta>,
}

/// The schedule with the photos of the people in it.
#[toolkit_macros::api_dto(response)]
pub struct ScheduleView {
    /// The schedule.
    #[serde(flatten)]
    pub schedule: Schedule,
    /// Photo metadata by person id.
    pub avatars: HashMap<String, AvatarMeta>,
}

/// The crew's people.
#[toolkit_macros::api_dto(response)]
pub struct PeopleView {
    /// Everyone with tasks, with or without an account.
    pub people: Vec<Person>,
    /// Today in the crew's zone.
    pub today: NaiveDate,
    /// Photo metadata by person id.
    pub avatars: HashMap<String, AvatarMeta>,
}

/// A browser's push subscription.
#[toolkit_macros::api_dto(request)]
pub struct Subscribe {
    /// Push service endpoint.
    pub endpoint: String,
    /// The browser's keys.
    pub keys: SubscriptionKeys,
}

/// Keys of a push subscription.
#[toolkit_macros::api_dto(request)]
pub struct SubscriptionKeys {
    /// The browser's public key.
    pub p256dh: String,
    /// The shared authentication secret.
    pub auth: String,
}

/// The id of a stored subscription.
#[toolkit_macros::api_dto(response)]
pub struct SubscriptionCreated {
    /// Subscription id.
    pub id: String,
}

/// Send a test reminder to one subscription.
#[toolkit_macros::api_dto(request)]
pub struct TestPush {
    /// Subscription id.
    pub id: String,
}

/// The operation succeeded.
#[toolkit_macros::api_dto(response)]
pub struct Done {
    /// Always true.
    pub ok: bool,
}
