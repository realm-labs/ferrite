//! Namespace-aware loading and deterministic caching for structure templates.

use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use thiserror::Error;

use crate::generation::structure::template::{StructureTemplate, TemplateDecodeError};

pub trait TemplateSource {
    fn load_template(&self, id: &ResourceId) -> Result<Option<Vec<u8>>, TemplateSourceError>;
}

#[derive(Debug, Clone)]
pub struct FileTemplateSource {
    root: PathBuf,
}

impl FileTemplateSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path_for(&self, id: &ResourceId) -> PathBuf {
        self.root
            .join("data")
            .join(id.namespace())
            .join("structure")
            .join(id.path())
            .with_extension("nbt")
    }
}

impl TemplateSource for FileTemplateSource {
    fn load_template(&self, id: &ResourceId) -> Result<Option<Vec<u8>>, TemplateSourceError> {
        let path = self.path_for(id);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(TemplateSourceError::Read { path, error }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TemplateLookup {
    pub id: ResourceId,
    pub template: Arc<StructureTemplate>,
    pub missing: bool,
}

#[derive(Debug)]
struct CachedTemplate {
    template: Arc<StructureTemplate>,
    missing: bool,
}

#[derive(Debug)]
pub struct TemplateManager<S> {
    source: S,
    cache: BTreeMap<ResourceId, CachedTemplate>,
}

impl<S> TemplateManager<S>
where
    S: TemplateSource,
{
    pub fn new(source: S) -> Self {
        Self {
            source,
            cache: BTreeMap::new(),
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn cached_count(&self) -> usize {
        self.cache.len()
    }

    pub fn get_or_create(&mut self, name: &str) -> Result<TemplateLookup, TemplateManagerError> {
        let id = ResourceId::parse_with_default_namespace(name)?;
        if let Some(cached) = self.cache.get(&id) {
            return Ok(TemplateLookup {
                id,
                template: Arc::clone(&cached.template),
                missing: cached.missing,
            });
        }

        let bytes = self.source.load_template(&id)?;
        let (template, missing) = match bytes {
            Some(bytes) => (Arc::new(StructureTemplate::decode_gzip(&bytes)?), false),
            None => (Arc::new(StructureTemplate::empty()), true),
        };
        self.cache.insert(
            id.clone(),
            CachedTemplate {
                template: Arc::clone(&template),
                missing,
            },
        );
        Ok(TemplateLookup {
            id,
            template,
            missing,
        })
    }

    pub fn require(&mut self, name: &str) -> Result<TemplateLookup, TemplateManagerError> {
        let lookup = self.get_or_create(name)?;
        if lookup.missing {
            return Err(TemplateManagerError::Missing(lookup.id));
        }
        Ok(lookup)
    }
}

#[derive(Debug, Error)]
pub enum TemplateSourceError {
    #[error("read structure template {}: {error}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        error: std::io::Error,
    },
    #[error("structure template source failed: {0}")]
    Other(String),
}

#[derive(Debug, Error)]
pub enum TemplateManagerError {
    #[error(transparent)]
    ResourceId(#[from] ResourceIdError),
    #[error(transparent)]
    Source(#[from] TemplateSourceError),
    #[error(transparent)]
    Decode(#[from] TemplateDecodeError),
    #[error("required structure template {0} is missing")]
    Missing(ResourceId),
}
