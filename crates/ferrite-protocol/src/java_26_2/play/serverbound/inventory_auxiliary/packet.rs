use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleItemSelected {
    pub slot: i32,
    pub selected: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditBook {
    pub slot: i32,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeenAdvancements {
    OpenedTab(Identifier),
    ClosedScreen,
}
