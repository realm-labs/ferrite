use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberFormat {
    Blank,
    Styled(NetworkNbt),
    Fixed(TextComponentNbt),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectiveRenderType {
    Integer,
    Hearts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectiveParameters {
    pub display_name: TextComponentNbt,
    pub render_type: ObjectiveRenderType,
    pub number_format: Option<NumberFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetScore {
    pub owner: String,
    pub objective_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetDisplayObjective {
    pub slot: DisplaySlot,
    pub objective_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetObjective {
    pub objective_name: String,
    pub method: i8,
    pub parameters: Option<ObjectiveParameters>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetScore {
    pub owner: String,
    pub objective_name: String,
    pub score: i32,
    pub display: Option<TextComponentNbt>,
    pub number_format: Option<NumberFormat>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayerTeam {
    pub team_name: String,
    pub method: i8,
    pub parameters: Option<TeamParameters>,
    pub players: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TeamColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

impl TeamColor {
    #[must_use]
    pub const fn from_fallback_id(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::DarkBlue,
            2 => Self::DarkGreen,
            3 => Self::DarkAqua,
            4 => Self::DarkRed,
            5 => Self::DarkPurple,
            6 => Self::Gold,
            7 => Self::Gray,
            8 => Self::DarkGray,
            9 => Self::Blue,
            10 => Self::Green,
            11 => Self::Aqua,
            12 => Self::Red,
            13 => Self::LightPurple,
            14 => Self::Yellow,
            15 => Self::White,
            _ => Self::Black,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DisplaySlot {
    List,
    Sidebar,
    BelowName,
    SidebarTeam(TeamColor),
}

impl DisplaySlot {
    #[must_use]
    pub const fn from_fallback_id(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Sidebar,
            2 => Self::BelowName,
            3..=18 => Self::SidebarTeam(TeamColor::from_fallback_id(raw_id - 3)),
            _ => Self::List,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::List => 0,
            Self::Sidebar => 1,
            Self::BelowName => 2,
            Self::SidebarTeam(color) => color.id() + 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameTagVisibility {
    Always,
    Never,
    HideForOtherTeams,
    HideForOwnTeam,
}

impl NameTagVisibility {
    #[must_use]
    pub const fn from_fallback_id(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Never,
            2 => Self::HideForOtherTeams,
            3 => Self::HideForOwnTeam,
            _ => Self::Always,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionRule {
    Always,
    Never,
    PushOtherTeams,
    PushOwnTeam,
}

impl CollisionRule {
    #[must_use]
    pub const fn from_fallback_id(raw_id: i32) -> Self {
        match raw_id {
            1 => Self::Never,
            2 => Self::PushOtherTeams,
            3 => Self::PushOwnTeam,
            _ => Self::Always,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamParameters {
    pub display_name: TextComponentNbt,
    pub member_prefix: TextComponentNbt,
    pub member_suffix: TextComponentNbt,
    pub visibility: NameTagVisibility,
    pub collision_rule: CollisionRule,
    pub color: Option<TeamColor>,
    pub allow_friendly_fire: bool,
    pub see_friendly_invisibles: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoreboardPacket {
    ResetScore(ResetScore),
    SetDisplayObjective(SetDisplayObjective),
    SetObjective(SetObjective),
    SetPlayerTeam(SetPlayerTeam),
    SetScore(SetScore),
}
