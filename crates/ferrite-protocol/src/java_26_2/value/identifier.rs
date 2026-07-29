use std::fmt::{self, Display, Formatter};

use thiserror::Error;

use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const DEFAULT_NAMESPACE: &str = "minecraft";
const MAX_IDENTIFIER_CODE_UNITS: usize = 32_767;

/// A 26.2 `Identifier`, preserving Minecraft's exact, deliberately permissive grammar.
///
/// This is not `ferrite_foundation::ResourceId`: Minecraft accepts empty and ambiguous path
/// segments, while Ferrite's storage-facing identifier intentionally rejects them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Identifier {
    namespace: String,
    path: String,
}

impl Identifier {
    pub fn new(
        namespace: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Self, IdentifierError> {
        let namespace = namespace.into();
        let path = path.into();
        validate_namespace(&namespace)?;
        validate_path(&path)?;
        Ok(Self { namespace, path })
    }

    pub fn minecraft(path: impl Into<String>) -> Result<Self, IdentifierError> {
        Self::new(DEFAULT_NAMESPACE, path)
    }

    pub fn parse(value: &str) -> Result<Self, IdentifierError> {
        match value.split_once(':') {
            Some(("", path)) => Self::minecraft(path),
            Some((namespace, path)) => Self::new(namespace, path),
            None => Self::minecraft(value),
        }
    }

    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn read(reader: &mut WireReader<'_>) -> Result<Self, IdentifierReadError> {
        let value = reader.read_utf(MAX_IDENTIFIER_CODE_UNITS)?;
        Self::parse(&value).map_err(IdentifierReadError::Invalid)
    }

    pub(crate) fn write(&self, writer: &mut WireWriter) -> Result<(), WireError> {
        writer.write_utf(&self.to_string(), MAX_IDENTIFIER_CODE_UNITS)
    }
}

impl Display for Identifier {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.namespace, self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum IdentifierError {
    #[error("identifier namespace {namespace:?} is invalid")]
    InvalidNamespace { namespace: String },
    #[error("identifier path {path:?} is invalid")]
    InvalidPath { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum IdentifierReadError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error(transparent)]
    Invalid(#[from] IdentifierError),
}

fn validate_namespace(namespace: &str) -> Result<(), IdentifierError> {
    if namespace == ".."
        || !namespace
            .chars()
            .all(|character| matches!(character, 'a'..='z' | '0'..='9' | '_' | '.' | '-'))
    {
        Err(IdentifierError::InvalidNamespace {
            namespace: namespace.to_owned(),
        })
    } else {
        Ok(())
    }
}

fn validate_path(path: &str) -> Result<(), IdentifierError> {
    if path
        .chars()
        .all(|character| matches!(character, 'a'..='z' | '0'..='9' | '_' | '.' | '-' | '/'))
    {
        Ok(())
    } else {
        Err(IdentifierError::InvalidPath {
            path: path.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::value::identifier::Identifier;

    #[test]
    fn mirrors_the_locked_identifier_grammar() {
        assert_eq!(
            Identifier::parse("stone").unwrap().to_string(),
            "minecraft:stone"
        );
        assert_eq!(
            Identifier::parse(":stone").unwrap().to_string(),
            "minecraft:stone"
        );
        assert!(Identifier::parse("minecraft:").is_ok());
        assert!(Identifier::parse("minecraft:a//../b").is_ok());
        assert!(Identifier::parse("..:stone").is_err());
        assert!(Identifier::parse("minecraft:Stone").is_err());
    }
}
