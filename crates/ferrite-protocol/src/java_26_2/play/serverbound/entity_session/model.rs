use std::collections::BTreeMap;

use crate::java_26_2::play::serverbound::entity_session::packet::LowPrecisionVector;
use crate::java_26_2::play::serverbound::packet::Hand;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SessionPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for SessionPosition {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMode {
    Survival,
    Creative,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionEntityKind {
    Ordinary,
    Living,
    Avatar,
    Item,
    ExperienceOrb,
    AbstractArrow { attackable: bool },
}

impl SessionEntityKind {
    #[must_use]
    pub const fn is_living(self) -> bool {
        matches!(self, Self::Living | Self::Avatar)
    }

    #[must_use]
    pub const fn is_avatar(self) -> bool {
        matches!(self, Self::Avatar)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackRangeProjection {
    pub minimum: f64,
    pub maximum: f64,
    pub creative_minimum: f64,
    pub creative_maximum: f64,
    pub hitbox_margin: f64,
    pub mob_factor: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SessionItemStack {
    pub item: Option<String>,
    pub count: i32,
    pub components: BTreeMap<String, String>,
    pub feature_enabled: bool,
    pub piercing_weapon: bool,
    pub attack_range: Option<AttackRangeProjection>,
}

impl SessionItemStack {
    #[must_use]
    pub fn item(item: impl Into<String>, count: i32) -> Self {
        Self {
            item: Some(item.into()),
            count,
            feature_enabled: true,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.item.is_none() || self.count <= 0
    }

    #[must_use]
    pub fn same_item_and_components(&self, other: &Self) -> bool {
        self.item == other.item && self.components == other.components
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwingSource {
    None,
    Client,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionResultProjection {
    Pass,
    Consume,
    Success { swing: SwingSource },
}

impl InteractionResultProjection {
    #[must_use]
    pub const fn consumes_action(self) -> bool {
        !matches!(self, Self::Pass)
    }

    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Success { .. })
    }

    #[must_use]
    pub const fn swing_source(self) -> SwingSource {
        match self {
            Self::Success { swing } => swing,
            Self::Pass | Self::Consume => SwingSource::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemInteractionProjection {
    pub result: InteractionResultProjection,
    pub resulting_stack: SessionItemStack,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionEntityProjection {
    pub entity_id: i32,
    pub uuid: u128,
    pub kind: SessionEntityKind,
    pub position: SessionPosition,
    pub eye_to_aabb_distance_squared: f64,
    pub inside_world_border: bool,
    pub removed: bool,
    pub pickable: bool,
    pub menu_provider: bool,
    pub pick_result: Option<SessionItemStack>,
    pub target_interaction: InteractionResultProjection,
    pub item_interaction: Option<ItemInteractionProjection>,
}

impl SessionEntityProjection {
    #[must_use]
    pub fn new(entity_id: i32, uuid: u128, kind: SessionEntityKind) -> Self {
        Self {
            entity_id,
            uuid,
            kind,
            position: SessionPosition::default(),
            eye_to_aabb_distance_squared: 0.0,
            inside_world_border: true,
            removed: false,
            pickable: true,
            menu_provider: false,
            pick_result: None,
            target_interaction: InteractionResultProjection::Pass,
            item_interaction: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionLevelProjection {
    pub key: String,
    pub entities: Vec<SessionEntityProjection>,
}

impl SessionLevelProjection {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            entities: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntitySessionPlayer {
    pub entity_id: i32,
    pub current_level: usize,
    pub position: SessionPosition,
    pub client_loaded: bool,
    pub mode: PlayerMode,
    pub health: f32,
    pub won_game: bool,
    pub hardcore: bool,
    pub idle_resets: u64,
    pub shift_key_down: bool,
    pub interaction_range: f64,
    pub minimum_attack_charge_met: bool,
    pub infinite_materials: bool,
    pub can_use_game_master_blocks: bool,
    pub gamerule_permission: bool,
    pub main_hand: SessionItemStack,
    pub off_hand: SessionItemStack,
    pub selected_hotbar: usize,
    pub hotbar: Vec<SessionItemStack>,
    pub inventory: Vec<SessionItemStack>,
    pub camera_entity_id: i32,
    pub stats_dirty: BTreeMap<String, i32>,
    pub gamerules: BTreeMap<String, String>,
    pub load_grace_ticks: u32,
    pub generation: u64,
}

impl EntitySessionPlayer {
    #[must_use]
    pub fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            current_level: 0,
            position: SessionPosition::default(),
            client_loaded: true,
            mode: PlayerMode::Survival,
            health: 20.0,
            won_game: false,
            hardcore: false,
            idle_resets: 0,
            shift_key_down: false,
            interaction_range: 3.0,
            minimum_attack_charge_met: true,
            infinite_materials: false,
            can_use_game_master_blocks: false,
            gamerule_permission: false,
            main_hand: SessionItemStack::default(),
            off_hand: SessionItemStack::default(),
            selected_hotbar: 0,
            hotbar: vec![SessionItemStack::default(); 9],
            inventory: Vec::new(),
            camera_entity_id: entity_id,
            stats_dirty: BTreeMap::new(),
            gamerules: BTreeMap::new(),
            load_grace_ticks: 0,
            generation: 0,
        }
    }

    #[must_use]
    pub fn hand(&self, hand: Hand) -> &SessionItemStack {
        match hand {
            Hand::Main => &self.main_hand,
            Hand::Off => &self.off_hand,
        }
    }

    pub fn hand_mut(&mut self, hand: Hand) -> &mut SessionItemStack {
        match hand {
            Hand::Main => &mut self.main_hand,
            Hand::Off => &mut self.off_hand,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntitySessionAction {
    AttackExecuted {
        target_entity_id: i32,
    },
    DisconnectInvalidAttack {
        target_entity_id: i32,
    },
    TargetInteraction {
        target_entity_id: i32,
        hand: Hand,
        location: LowPrecisionVector,
    },
    ItemInteraction {
        target_entity_id: i32,
        hand: Hand,
    },
    SpectatorMenuOpened {
        target_entity_id: i32,
    },
    EntityInteractGameEvent {
        target_entity_id: i32,
    },
    InteractionCriterion {
        target_entity_id: i32,
        stack: SessionItemStack,
    },
    SwingPublished {
        hand: Hand,
        include_self: bool,
    },
    HeldSlotConvergence {
        slot: usize,
    },
    InventoryMenuConvergence,
    AvatarProfilePrinted {
        target_entity_id: i32,
    },
    CameraTargetRelocated {
        target_entity_id: i32,
    },
    CameraPublished {
        target_entity_id: i32,
    },
    KnownPositionReset,
    CameraResetToSelf,
    SameDimensionTeleport {
        target_entity_id: i32,
    },
    CrossDimensionRespawn {
        keep_mask: u8,
    },
    PositionChallenge,
    LevelReprojection,
    PlayerRespawned {
        retain_player_data: bool,
    },
    RespawnPublished,
    ClientLoadGraceRestarted {
        ticks: u32,
    },
    EndToOverworldCriterion,
    StatsPublished {
        values: BTreeMap<String, i32>,
    },
    GamerulesPublished {
        values: BTreeMap<String, String>,
    },
    GameruleRequestDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntitySessionDisposition {
    Handled,
    Ignored,
    DisconnectInvalidAttack,
}
