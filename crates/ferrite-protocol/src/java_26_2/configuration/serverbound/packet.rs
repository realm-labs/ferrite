use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::known_pack::KnownPack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationServerboundPacket {
    ClientInformation(ClientInformation),
    CustomPayload(CustomPayload),
    FinishConfiguration,
    KeepAlive(i64),
    Pong(i32),
    SelectKnownPacks(Vec<KnownPack>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomPayload {
    Brand(String),
    /// The base server consumes and ignores an unknown channel's bounded remainder.
    Discarded {
        channel: Identifier,
        length: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientInformation {
    pub language: String,
    pub view_distance: i8,
    pub chat_visibility: ChatVisibility,
    pub chat_colors: bool,
    pub model_customization: u8,
    pub main_hand: MainHand,
    pub text_filtering: bool,
    pub allows_listing: bool,
    pub particle_status: ParticleStatus,
}

impl Default for ClientInformation {
    fn default() -> Self {
        Self {
            language: "en_us".to_owned(),
            view_distance: 2,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customization: 0,
            main_hand: MainHand::Right,
            text_filtering: false,
            allows_listing: false,
            particle_status: ParticleStatus::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatVisibility {
    Full,
    System,
    Hidden,
}

impl ChatVisibility {
    pub(crate) const fn ordinal(self) -> i32 {
        match self {
            Self::Full => 0,
            Self::System => 1,
            Self::Hidden => 2,
        }
    }

    pub(crate) const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Full),
            1 => Some(Self::System),
            2 => Some(Self::Hidden),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainHand {
    Left,
    Right,
}

impl MainHand {
    pub(crate) const fn ordinal(self) -> i32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }

    pub(crate) const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::Left),
            1 => Some(Self::Right),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleStatus {
    All,
    Decreased,
    Minimal,
}

impl ParticleStatus {
    pub(crate) const fn ordinal(self) -> i32 {
        match self {
            Self::All => 0,
            Self::Decreased => 1,
            Self::Minimal => 2,
        }
    }

    pub(crate) const fn from_ordinal(ordinal: i32) -> Option<Self> {
        match ordinal {
            0 => Some(Self::All),
            1 => Some(Self::Decreased),
            2 => Some(Self::Minimal),
            _ => None,
        }
    }
}
