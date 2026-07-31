use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClearTitles {
    pub reset_times: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectAdvancementsTab {
    pub tab: Option<Identifier>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetActionBarText {
    pub text: TextComponentNbt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSubtitleText {
    pub text: TextComponentNbt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetTitleText {
    pub text: TextComponentNbt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetTitlesAnimation {
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabList {
    pub header: TextComponentNbt,
    pub footer: TextComponentNbt,
}
