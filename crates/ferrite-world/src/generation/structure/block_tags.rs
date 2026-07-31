//! Recursive block-tag loading for structure processor records.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use serde_json::Value;
use thiserror::Error;

pub trait BlockTagResolver {
    fn resolve_block_tag(&mut self, name: &str) -> Result<BTreeSet<String>, BlockTagError>;
}

#[derive(Debug)]
pub struct FileBlockTagResolver {
    root: PathBuf,
    cache: BTreeMap<ResourceId, BTreeSet<String>>,
}

impl FileBlockTagResolver {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            cache: BTreeMap::new(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }

    pub fn path_for(&self, id: &ResourceId) -> PathBuf {
        self.root
            .join("data")
            .join(id.namespace())
            .join("tags")
            .join("block")
            .join(id.path())
            .with_extension("json")
    }

    fn resolve(
        &mut self,
        id: ResourceId,
        resolving: &mut BTreeSet<ResourceId>,
    ) -> Result<BTreeSet<String>, BlockTagError> {
        if let Some(blocks) = self.cache.get(&id) {
            return Ok(blocks.clone());
        }
        if !resolving.insert(id.clone()) {
            return Err(BlockTagError::Cycle(id));
        }
        let path = self.path_for(&id);
        let bytes = match fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                resolving.remove(&id);
                return Err(BlockTagError::Missing(id));
            }
            Err(error) => {
                resolving.remove(&id);
                return Err(BlockTagError::Read { path, error });
            }
        };
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| BlockTagError::Json {
            tag: id.clone(),
            error,
        })?;
        let values = value
            .get("values")
            .and_then(Value::as_array)
            .ok_or_else(|| BlockTagError::Values(id.clone()))?;
        let mut blocks = BTreeSet::new();
        for value in values {
            let (name, required) = tag_entry(value, &id)?;
            if let Some(nested) = name.strip_prefix('#') {
                let nested = ResourceId::parse_with_default_namespace(nested)?;
                match self.resolve(nested, resolving) {
                    Ok(values) => blocks.extend(values),
                    Err(BlockTagError::Missing(_)) if !required => {}
                    Err(error) => return Err(error),
                }
            } else {
                let block = ResourceId::parse_with_default_namespace(name)?;
                blocks.insert(block.to_string());
            }
        }
        resolving.remove(&id);
        self.cache.insert(id, blocks.clone());
        Ok(blocks)
    }
}

impl BlockTagResolver for FileBlockTagResolver {
    fn resolve_block_tag(&mut self, name: &str) -> Result<BTreeSet<String>, BlockTagError> {
        let name = name.strip_prefix('#').unwrap_or(name);
        let id = ResourceId::parse_with_default_namespace(name)?;
        self.resolve(id, &mut BTreeSet::new())
    }
}

fn tag_entry<'a>(value: &'a Value, tag: &ResourceId) -> Result<(&'a str, bool), BlockTagError> {
    if let Some(name) = value.as_str() {
        return Ok((name, true));
    }
    let object = value
        .as_object()
        .ok_or_else(|| BlockTagError::Entry(tag.clone()))?;
    let name = object
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| BlockTagError::Entry(tag.clone()))?;
    let required = match object.get("required") {
        None => true,
        Some(Value::Bool(required)) => *required,
        Some(_) => return Err(BlockTagError::Entry(tag.clone())),
    };
    Ok((name, required))
}

#[derive(Debug, Error)]
pub enum BlockTagError {
    #[error(transparent)]
    ResourceId(#[from] ResourceIdError),
    #[error("required block tag {0} is missing")]
    Missing(ResourceId),
    #[error("block tag reference cycle reaches {0}")]
    Cycle(ResourceId),
    #[error("read block tag {}: {error}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        error: std::io::Error,
    },
    #[error("decode block tag {tag}: {error}")]
    Json {
        tag: ResourceId,
        #[source]
        error: serde_json::Error,
    },
    #[error("block tag {0} must contain a values array")]
    Values(ResourceId),
    #[error("block tag {0} has an invalid value entry")]
    Entry(ResourceId),
}
