use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginDisconnectReason {
    json: String,
}

impl LoginDisconnectReason {
    pub fn from_json(json: impl Into<String>) -> Result<Self, LoginDisconnectReasonError> {
        let json = json.into();
        let value: Value =
            serde_json::from_str(&json).map_err(|_| LoginDisconnectReasonError::MalformedJson)?;
        if component_shape_is_valid(&value) {
            Ok(Self { json })
        } else {
            Err(LoginDisconnectReasonError::InvalidComponentShape)
        }
    }

    pub fn literal(text: &str) -> Result<Self, LoginDisconnectReasonError> {
        let json =
            serde_json::to_string(text).map_err(|_| LoginDisconnectReasonError::MalformedJson)?;
        Ok(Self { json })
    }

    #[must_use]
    pub fn as_json(&self) -> &str {
        &self.json
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LoginDisconnectReasonError {
    #[error("login disconnect component is malformed JSON")]
    MalformedJson,
    #[error("login disconnect JSON has no valid component root shape")]
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
