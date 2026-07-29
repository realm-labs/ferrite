//! Validated namespaced resource identifiers.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ResourceId {
    namespace: String,
    path: String,
}

impl ResourceId {
    pub fn new(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, ResourceIdError> {
        let namespace = namespace.into();
        let path = path.into();
        validate_namespace(&namespace)?;
        validate_path(&path)?;
        Ok(Self { namespace, path })
    }

    pub fn minecraft(path: impl Into<String>) -> Result<Self, ResourceIdError> {
        Self::new(DEFAULT_NAMESPACE, path)
    }

    pub fn parse_with_default_namespace(value: &str) -> Result<Self, ResourceIdError> {
        match value.split_once(':') {
            Some((namespace, path)) => Self::new(namespace, path),
            None => Self::minecraft(value),
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

impl Display for ResourceId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
}

impl FromStr for ResourceId {
    type Err = ResourceIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse_with_default_namespace(value)
    }
}

impl TryFrom<String> for ResourceId {
    type Error = ResourceIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse_with_default_namespace(&value)
    }
}

impl From<ResourceId> for String {
    fn from(value: ResourceId) -> Self {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourceIdError {
    #[error("resource namespace cannot be empty")]
    EmptyNamespace,
    #[error("resource path cannot be empty")]
    EmptyPath,
    #[error("invalid resource namespace character {character:?} at byte {index}")]
    InvalidNamespaceCharacter { character: char, index: usize },
    #[error("invalid resource path character {character:?} at byte {index}")]
    InvalidPathCharacter { character: char, index: usize },
    #[error("resource path contains ambiguous segment {segment:?}")]
    AmbiguousPathSegment { segment: String },
}

fn validate_namespace(namespace: &str) -> Result<(), ResourceIdError> {
    if namespace.is_empty() {
        return Err(ResourceIdError::EmptyNamespace);
    }
    for (index, character) in namespace.char_indices() {
        if !matches!(character, 'a'..='z' | '0'..='9' | '_' | '.' | '-') {
            return Err(ResourceIdError::InvalidNamespaceCharacter { character, index });
        }
    }
    Ok(())
}

fn validate_path(path: &str) -> Result<(), ResourceIdError> {
    if path.is_empty() {
        return Err(ResourceIdError::EmptyPath);
    }
    for (index, character) in path.char_indices() {
        if !matches!(
            character,
            'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'
        ) {
            return Err(ResourceIdError::InvalidPathCharacter { character, index });
        }
    }
    for segment in path.split('/') {
        if segment.is_empty() || matches!(segment, "." | "..") {
            return Err(ResourceIdError::AmbiguousPathSegment {
                segment: segment.to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_namespace_uses_the_minecraft_default() {
        let identifier = ResourceId::parse_with_default_namespace("stone").unwrap();
        assert_eq!(identifier.namespace(), "minecraft");
        assert_eq!(identifier.path(), "stone");
        assert_eq!(identifier.to_string(), "minecraft:stone");
    }

    #[test]
    fn invalid_and_ambiguous_components_are_rejected() {
        assert!(ResourceId::new("Ferrite", "stone").is_err());
        assert!(ResourceId::new("ferrite", "blocks//stone").is_err());
        assert!(ResourceId::new("ferrite", "../stone").is_err());
        assert!(ResourceId::new("", "stone").is_err());
    }

    #[test]
    fn serde_uses_and_validates_the_canonical_string() {
        let identifier = ResourceId::new("ferrite", "world/spawn").unwrap();
        let encoded = serde_json::to_string(&identifier).unwrap();
        assert_eq!(encoded, "\"ferrite:world/spawn\"");
        assert_eq!(
            serde_json::from_str::<ResourceId>(&encoded).unwrap(),
            identifier
        );
        assert!(serde_json::from_str::<ResourceId>("\"Ferrite:spawn\"").is_err());
    }
}
