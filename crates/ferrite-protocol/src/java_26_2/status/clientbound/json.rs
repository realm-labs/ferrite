use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::java_26_2::status::clientbound::packet::{
    ServerStatus, StatusDescription, StatusPlayers, StatusSample, StatusVersion,
};

const FAVICON_PREFIX: &str = "data:image/png;base64,";

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StatusJsonError {
    #[error("status response contains malformed JSON")]
    MalformedJson,
    #[error("status response JSON root is not an object")]
    InvalidRoot,
    #[error("status response JSON could not be serialized")]
    Serialization,
}

pub fn decode(json: &str) -> Result<ServerStatus, StatusJsonError> {
    let value: Value = serde_json::from_str(json).map_err(|_| StatusJsonError::MalformedJson)?;
    let object = value.as_object().ok_or(StatusJsonError::InvalidRoot)?;
    Ok(ServerStatus {
        description: decode_description(object),
        players: object.get("players").and_then(decode_players),
        version: object.get("version").and_then(decode_version),
        favicon: object.get("favicon").and_then(decode_favicon),
        enforces_secure_chat: object
            .get("enforcesSecureChat")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

pub fn encode(status: &ServerStatus) -> Result<String, StatusJsonError> {
    let mut fields = Vec::new();
    if !status.description.is_empty() {
        fields.push(("description", status.description.value().clone()));
    }
    if let Some(players) = &status.players {
        fields.push(("players", encode_players(players)));
    }
    if let Some(version) = &status.version {
        fields.push(("version", encode_version(version)));
    }
    if let Some(favicon) = &status.favicon {
        fields.push((
            "favicon",
            Value::String(format!("{FAVICON_PREFIX}{}", STANDARD.encode(favicon))),
        ));
    }
    if status.enforces_secure_chat {
        fields.push(("enforcesSecureChat", Value::Bool(true)));
    }

    let mut json = String::from("{");
    for (index, (name, value)) in fields.into_iter().enumerate() {
        if index != 0 {
            json.push(',');
        }
        json.push('"');
        json.push_str(name);
        json.push_str("\":");
        json.push_str(&serde_json::to_string(&value).map_err(|_| StatusJsonError::Serialization)?);
    }
    json.push('}');
    Ok(json)
}

fn decode_description(object: &Map<String, Value>) -> StatusDescription {
    object
        .get("description")
        .cloned()
        .and_then(|value| StatusDescription::from_value(value).ok())
        .unwrap_or_default()
}

fn decode_players(value: &Value) -> Option<StatusPlayers> {
    let object = value.as_object()?;
    let max = json_i32(object.get("max")?)?;
    let online = json_i32(object.get("online")?)?;
    let sample = object
        .get("sample")
        .and_then(decode_sample)
        .unwrap_or_default();
    Some(StatusPlayers {
        max,
        online,
        sample,
    })
}

fn decode_sample(value: &Value) -> Option<Vec<StatusSample>> {
    let entries = value.as_array()?;
    let mut sample = Vec::with_capacity(entries.len());
    for entry in entries {
        let object = entry.as_object()?;
        sample.push(StatusSample {
            id: parse_uuid(object.get("id")?.as_str()?)?,
            name: object.get("name")?.as_str()?.to_owned(),
        });
    }
    Some(sample)
}

fn decode_version(value: &Value) -> Option<StatusVersion> {
    let object = value.as_object()?;
    Some(StatusVersion {
        name: object.get("name")?.as_str()?.to_owned(),
        protocol: json_i32(object.get("protocol")?)?,
    })
}

fn decode_favicon(value: &Value) -> Option<Vec<u8>> {
    let encoded = value.as_str()?.strip_prefix(FAVICON_PREFIX)?;
    let compact = encoded.replace('\n', "");
    STANDARD.decode(compact.as_bytes()).ok()
}

fn encode_players(players: &StatusPlayers) -> Value {
    let mut object = Map::new();
    object.insert("max".to_owned(), Value::from(players.max));
    object.insert("online".to_owned(), Value::from(players.online));
    if !players.sample.is_empty() {
        object.insert(
            "sample".to_owned(),
            Value::Array(
                players
                    .sample
                    .iter()
                    .map(|sample| {
                        let mut entry = Map::new();
                        entry.insert("id".to_owned(), Value::String(format_uuid(sample.id)));
                        entry.insert("name".to_owned(), Value::String(sample.name.clone()));
                        Value::Object(entry)
                    })
                    .collect(),
            ),
        );
    }
    Value::Object(object)
}

fn encode_version(version: &StatusVersion) -> Value {
    let mut object = Map::new();
    object.insert("name".to_owned(), Value::String(version.name.clone()));
    object.insert("protocol".to_owned(), Value::from(version.protocol));
    Value::Object(object)
}

fn json_i32(value: &Value) -> Option<i32> {
    i32::try_from(value.as_i64()?).ok()
}

fn parse_uuid(value: &str) -> Option<u128> {
    if value.len() != 36
        || ![8, 13, 18, 23]
            .into_iter()
            .all(|index| value.as_bytes()[index] == b'-')
    {
        return None;
    }
    let mut compact = String::with_capacity(32);
    for (index, character) in value.chars().enumerate() {
        if ![8, 13, 18, 23].contains(&index) {
            compact.push(character);
        }
    }
    u128::from_str_radix(&compact, 16).ok()
}

fn format_uuid(value: u128) -> String {
    let compact = format!("{value:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &compact[0..8],
        &compact[8..12],
        &compact[12..16],
        &compact[16..20],
        &compact[20..32]
    )
}
