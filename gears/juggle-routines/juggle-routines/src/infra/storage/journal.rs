//! Legacy operation JSON is decoded here, never inside domain policy.
use super::codec;
use crate::domain::{self, Command, CommandResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LegacyOperation {
    actor: String,
    fingerprint: String,
    response: serde_json::Value,
}

fn encode(operation: &domain::Operation) -> Result<LegacyOperation, serde_json::Error> {
    let fingerprint = match &operation.command {
        Command::Mark(value) | Command::Avatar(value) => value.clone(),
        Command::Appearance(id, value) => format!(
            "appearance/{id}/{}",
            serde_json::to_string(&codec::AppearanceEdit::from(value.clone()))?
        ),
        Command::Preferences(value) => format!(
            "preferences/{}",
            serde_json::to_string(&codec::PreferencesEdit::from(value.clone()))?
        ),
        Command::Person(value) => format!(
            "person/{}",
            serde_json::to_string(&codec::PersonEdit::from(value.clone()))?
        ),
        Command::Routine(value) => serde_json::to_string(&codec::Edit::from(value.clone()))?,
    };
    let response = match &operation.response {
        CommandResponse::Occurrence(value) => {
            serde_json::to_value(codec::Occurrence::from(value.clone()))?
        }
        CommandResponse::Version(value) => {
            serde_json::to_value(codec::Version::from(value.clone()))?
        }
        CommandResponse::Person(value) => serde_json::to_value(codec::Person::from(value.clone()))?,
        CommandResponse::Preferences(value) => {
            serde_json::to_value(codec::Preferences::from(value.clone()))?
        }
        CommandResponse::Appearance(value) => {
            serde_json::to_value(codec::Appearance::from(value.clone()))?
        }
        CommandResponse::Avatar(value) => {
            serde_json::to_value(codec::AvatarMeta::from(value.clone()))?
        }
    };
    Ok(LegacyOperation {
        actor: operation.actor.clone(),
        fingerprint,
        response,
    })
}

fn decode(value: LegacyOperation) -> Result<domain::Operation, serde_json::Error> {
    let LegacyOperation {
        actor,
        mut fingerprint,
        response,
    } = value;
    let (command, response) = if fingerprint.starts_with("mark/") {
        if fingerprint.matches('/').count() == 4 {
            fingerprint.push_str("/None");
        }
        (
            Command::Mark(fingerprint),
            CommandResponse::Occurrence(
                serde_json::from_value::<codec::Occurrence>(response)?.into(),
            ),
        )
    } else if fingerprint.starts_with("avatar/") {
        (
            Command::Avatar(fingerprint),
            CommandResponse::Avatar(serde_json::from_value::<codec::AvatarMeta>(response)?.into()),
        )
    } else if let Some(value) = fingerprint.strip_prefix("appearance/") {
        let (id, json) = value.split_once('/').ok_or_else(|| {
            <serde_json::Error as serde::de::Error>::custom("invalid stored appearance identity")
        })?;
        (
            Command::Appearance(
                id.into(),
                serde_json::from_str::<codec::AppearanceEdit>(json)?.into(),
            ),
            CommandResponse::Appearance(
                serde_json::from_value::<codec::Appearance>(response)?.into(),
            ),
        )
    } else if let Some(json) = fingerprint.strip_prefix("preferences/") {
        (
            Command::Preferences(serde_json::from_str::<codec::PreferencesEdit>(json)?.into()),
            CommandResponse::Preferences(
                serde_json::from_value::<codec::Preferences>(response)?.into(),
            ),
        )
    } else if let Some(json) = fingerprint.strip_prefix("person/") {
        (
            Command::Person(serde_json::from_str::<codec::PersonEdit>(json)?.into()),
            CommandResponse::Person(serde_json::from_value::<codec::Person>(response)?.into()),
        )
    } else {
        (
            Command::Routine(serde_json::from_str::<codec::Edit>(&fingerprint)?.into()),
            CommandResponse::Version(serde_json::from_value::<codec::Version>(response)?.into()),
        )
    };
    Ok(domain::Operation {
        actor,
        command,
        response,
    })
}

impl Serialize for codec::Operation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        encode(&self.clone().into())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for codec::Operation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        decode(LegacyOperation::deserialize(deserializer)?)
            .map(Into::into)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
#[path = "../../tests/journal_tests.rs"]
mod journal_tests;
