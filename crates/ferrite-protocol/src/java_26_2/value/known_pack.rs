/// A connection-local descriptor for data the vanilla client may supply from an installed pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl KnownPack {
    #[must_use]
    pub fn vanilla_core() -> Self {
        Self {
            namespace: "minecraft".to_owned(),
            id: "core".to_owned(),
            version: "26.2".to_owned(),
        }
    }
}
