//! Serialization belongs to the storage boundary, not domain definitions.
use crate::domain;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Person {
    pub id: String,
    pub name: String,
    pub tone: String,
    #[serde(default)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Routine {
    pub id: String,
    pub revision: u64,
    pub assignee: String,
    pub person_id: String,
    pub name: String,
    pub note: String,
    pub group: String,
    pub time: String,
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub operation_id: Uuid,
    pub occurrence_id: String,
    pub action: String,
    pub actor: String,
    pub at: DateTime<Utc>,
}
impl From<domain::Event> for Event {
    fn from(value: domain::Event) -> Self {
        Self {
            operation_id: value.operation_id,
            occurrence_id: value.occurrence_id,
            action: value.action,
            actor: value.actor,
            at: value.at,
        }
    }
}
impl From<Event> for domain::Event {
    fn from(value: Event) -> Self {
        Self {
            operation_id: value.operation_id,
            occurrence_id: value.occurrence_id,
            action: value.action,
            actor: value.actor,
            at: value.at,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Subscription {
    pub id: String,
    pub uid: String,
    pub endpoint: String,
    pub p256dh: String,
    pub auth: String,
}
impl From<domain::Subscription> for Subscription {
    fn from(value: domain::Subscription) -> Self {
        Self {
            id: value.id,
            uid: value.uid,
            endpoint: value.endpoint,
            p256dh: value.p256dh,
            auth: value.auth,
        }
    }
}
impl From<Subscription> for domain::Subscription {
    fn from(value: Subscription) -> Self {
        Self {
            id: value.id,
            uid: value.uid,
            endpoint: value.endpoint,
            p256dh: value.p256dh,
            auth: value.auth,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Delivery {
    pub key: String,
    pub subscription_id: String,
    pub day: NaiveDate,
    pub assignee: String,
    pub person_id: String,
    pub time: String,
    pub attempts: u32,
    pub status: String,
    pub retry_at: DateTime<Utc>,
    pub tag: String,
}
impl From<domain::Delivery> for Delivery {
    fn from(value: domain::Delivery) -> Self {
        Self {
            key: value.key,
            subscription_id: value.subscription_id,
            day: value.day,
            assignee: value.assignee,
            person_id: value.person_id,
            time: value.time,
            attempts: value.attempts,
            status: value.status,
            retry_at: value.retry_at,
            tag: value.tag,
        }
    }
}
impl From<Delivery> for domain::Delivery {
    fn from(value: Delivery) -> Self {
        Self {
            key: value.key,
            subscription_id: value.subscription_id,
            day: value.day,
            assignee: value.assignee,
            person_id: value.person_id,
            time: value.time,
            attempts: value.attempts,
            status: value.status,
            retry_at: value.retry_at,
            tag: value.tag,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Operation {
    pub actor: String,
    pub command: domain::Command,
    pub response: domain::CommandResponse,
}
impl From<domain::Operation> for Operation {
    fn from(value: domain::Operation) -> Self {
        Self {
            actor: value.actor,
            command: value.command,
            response: value.response,
        }
    }
}
impl From<Operation> for domain::Operation {
    fn from(value: Operation) -> Self {
        Self {
            actor: value.actor,
            command: value.command,
            response: value.response,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct State {
    pub created_date: NaiveDate,
    #[serde(default)]
    pub people: Vec<Person>,
    #[serde(default)]
    pub preferences: HashMap<String, Preferences>,
    #[serde(default)]
    pub appearances: HashMap<String, Appearance>,
    pub versions: Vec<Version>,
    pub days: BTreeMap<NaiveDate, Vec<Occurrence>>,
    pub events: Vec<Event>,
    pub operations: HashMap<Uuid, Operation>,
    pub subscriptions: Vec<Subscription>,
    pub deliveries: Vec<Delivery>,
}
impl From<domain::State> for State {
    fn from(value: domain::State) -> Self {
        Self {
            created_date: value.created_date,
            people: value.people.into_iter().map(|value| value.into()).collect(),
            preferences: value
                .preferences
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            appearances: value
                .appearances
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            versions: value
                .versions
                .into_iter()
                .map(|value| value.into())
                .collect(),
            days: value
                .days
                .into_iter()
                .map(|(key, value)| (key, value.into_iter().map(|value| value.into()).collect()))
                .collect(),
            events: value.events.into_iter().map(|value| value.into()).collect(),
            operations: value
                .operations
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            subscriptions: value
                .subscriptions
                .into_iter()
                .map(|value| value.into())
                .collect(),
            deliveries: value
                .deliveries
                .into_iter()
                .map(|value| value.into())
                .collect(),
        }
    }
}
impl From<State> for domain::State {
    fn from(value: State) -> Self {
        Self {
            created_date: value.created_date,
            people: value.people.into_iter().map(|value| value.into()).collect(),
            preferences: value
                .preferences
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            appearances: value
                .appearances
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            versions: value
                .versions
                .into_iter()
                .map(|value| value.into())
                .collect(),
            days: value
                .days
                .into_iter()
                .map(|(key, value)| (key, value.into_iter().map(|value| value.into()).collect()))
                .collect(),
            events: value.events.into_iter().map(|value| value.into()).collect(),
            operations: value
                .operations
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
            subscriptions: value
                .subscriptions
                .into_iter()
                .map(|value| value.into())
                .collect(),
            deliveries: value
                .deliveries
                .into_iter()
                .map(|value| value.into())
                .collect(),
            // The crews gear owns the zone; the service sets it after loading.
            zone: chrono_tz::UTC,
        }
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "snake_case")]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
