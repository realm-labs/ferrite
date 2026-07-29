use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusClientboundPacket {
    Response(ServerStatus),
    Pong(i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerStatus {
    pub description: StatusDescription,
    pub players: Option<StatusPlayers>,
    pub version: Option<StatusVersion>,
    pub favicon: Option<Vec<u8>>,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusDescription {
    value: Value,
}

impl StatusDescription {
    pub fn from_json(json: &str) -> Result<Self, StatusDescriptionError> {
        let value =
            serde_json::from_str(json).map_err(|_| StatusDescriptionError::MalformedJson)?;
        Self::from_value(value)
    }

    pub fn literal(text: impl Into<String>) -> Self {
        Self {
            value: Value::String(text.into()),
        }
    }

    pub(crate) fn from_value(value: Value) -> Result<Self, StatusDescriptionError> {
        if component_shape_is_valid(&value) {
            Ok(Self { value })
        } else {
            Err(StatusDescriptionError::InvalidComponentShape)
        }
    }

    #[must_use]
    pub fn as_json(&self) -> String {
        serde_json::to_string(&self.value)
            .unwrap_or_else(|_| unreachable!("a JSON value always serializes"))
    }

    #[must_use]
    pub(crate) fn value(&self) -> &Value {
        &self.value
    }

    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.value == Value::String(String::new())
    }
}

impl Default for StatusDescription {
    fn default() -> Self {
        Self::literal("")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPlayers {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<StatusSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusSample {
    pub id: u128,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusVersion {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StatusDescriptionError {
    #[error("status description is malformed JSON")]
    MalformedJson,
    #[error("status description JSON has no valid component root shape")]
    InvalidComponentShape,
}

fn component_shape_is_valid(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Array(values) => !values.is_empty() && values.iter().all(component_shape_is_valid),
        Value::Object(values) => !values.is_empty(),
        _ => false,
    }
}
