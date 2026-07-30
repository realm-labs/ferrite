use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameItem {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBeacon {
    pub primary: Option<Identifier>,
    pub secondary: Option<Identifier>,
}
