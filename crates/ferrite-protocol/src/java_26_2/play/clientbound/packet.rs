use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::{BlockPos, SectionPos};

use crate::java_26_2::play::clientbound::combat_look::packet::{
    PlayerCombatEnd, PlayerCombatKill, PlayerLookAt,
};
use crate::java_26_2::play::clientbound::command::CommandTree;
use crate::java_26_2::play::clientbound::container::packet::{
    ContainerClose, ContainerSetContent, ContainerSetData, ContainerSetSlot, OpenScreen,
    SetCursorItem, SetPlayerInventory,
};
use crate::java_26_2::play::clientbound::entity_effects::packet::{
    Explosion, RemoveMobEffect, UpdateMobEffect,
};
use crate::java_26_2::play::clientbound::entity_motion::packet::{
    EntityPositionSync, MoveMinecartAlongTrack, ProjectilePower, RelativePosition,
    RelativePositionRotation, RelativeRotation, RotateHead, SetEntityMotion, TeleportEntity,
};
use crate::java_26_2::play::clientbound::inventory_progression::packet::{
    MapItemData, TagQuery, UpdateAdvancements,
};
use crate::java_26_2::play::clientbound::merchant::packet::MerchantOffers;
use crate::java_26_2::play::clientbound::player_info::PlayerInfoUpdate;
use crate::java_26_2::play::clientbound::recipe::book::{PlaceGhostRecipe, RecipeBookRemove};
use crate::java_26_2::play::clientbound::recipe::{
    RecipeBookAdd, RecipeBookSettings, RecipeProjection,
};
use crate::java_26_2::play::clientbound::session::Respawn;
use crate::java_26_2::play::clientbound::special_screen::packet::{
    InteractionHand, MountScreenOpen, OpenSignEditor,
};
use crate::java_26_2::play::clientbound::terrain::packet::TerrainPacket;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

#[derive(Debug, Clone, PartialEq)]
pub enum PlayClientboundPacket {
    BlockChangedAck(BlockChangedAck),
    BlockDestruction(BlockDestruction),
    BlockEntityData(BlockEntityData),
    BlockEvent(BlockEvent),
    BlockUpdate(BlockUpdate),
    ChangeDifficulty(ChangeDifficulty),
    Commands(CommandTree),
    ContainerClose(ContainerClose),
    ContainerSetContent(ContainerSetContent),
    ContainerSetData(ContainerSetData),
    ContainerSetSlot(ContainerSetSlot),
    Disconnect(TextComponentNbt),
    EntityEvent(EntityEvent),
    EntityPositionSync(EntityPositionSync),
    Explosion(Box<Explosion>),
    GameEvent(GameEvent),
    InitializeBorder(BorderInitialization),
    KeepAlive(KeepAlive),
    Login(PlayLogin),
    MapItemData(MapItemData),
    MerchantOffers(MerchantOffers),
    MountScreenOpen(MountScreenOpen),
    MoveEntityPosition(RelativePosition),
    MoveEntityPositionRotation(RelativePositionRotation),
    MoveEntityRotation(RelativeRotation),
    MoveMinecartAlongTrack(MoveMinecartAlongTrack),
    MoveVehicle(VehiclePosition),
    OpenBook(InteractionHand),
    OpenScreen(OpenScreen),
    OpenSignEditor(OpenSignEditor),
    Ping(Ping),
    PlayerAbilities(PlayerAbilities),
    PlayerCombatEnd(PlayerCombatEnd),
    PlayerCombatEnter,
    PlayerCombatKill(PlayerCombatKill),
    PlayerInfoUpdate(PlayerInfoUpdate),
    PlayerLookAt(PlayerLookAt),
    PlayerPosition(PlayerPosition),
    PlayerRotation(PlayerRotation),
    ProjectilePower(ProjectilePower),
    PlaceGhostRecipe(Box<PlaceGhostRecipe>),
    RecipeBookAdd(RecipeBookAdd),
    RecipeBookRemove(RecipeBookRemove),
    RecipeBookSettings(RecipeBookSettings),
    Respawn(Respawn),
    RemoveMobEffect(RemoveMobEffect),
    RotateHead(RotateHead),
    ServerData(ServerData),
    SectionBlocksUpdate(SectionBlocksUpdate),
    SetDefaultSpawnPosition(DefaultSpawnPosition),
    SetCursorItem(SetCursorItem),
    SetEntityMotion(SetEntityMotion),
    SetHeldSlot(i32),
    SetPlayerInventory(SetPlayerInventory),
    SetTime(SetTime),
    TagQuery(TagQuery),
    TeleportEntity(TeleportEntity),
    Terrain(TerrainPacket),
    TickingState(TickingState),
    TickingStep(i32),
    UpdateAdvancements(UpdateAdvancements),
    UpdateMobEffect(UpdateMobEffect),
    UpdateRecipes(RecipeProjection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChangedAck {
    pub sequence: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockDestruction {
    pub breaker_entity_id: i32,
    pub position: BlockPos,
    pub progress: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntityData {
    pub position: BlockPos,
    pub type_raw_id: i32,
    pub update_tag: NetworkNbt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEvent {
    pub position: BlockPos,
    pub action: u8,
    pub parameter: u8,
    pub block_raw_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockUpdate {
    pub position: BlockPos,
    pub state: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionBlockChange {
    pub relative_position: u16,
    /// `None` preserves the locked client's nullable registry lookup on decode.
    pub state: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionBlocksUpdate {
    pub section: SectionPos,
    pub changes: Vec<SectionBlockChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeepAlive {
    pub challenge: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ping {
    pub payload: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehiclePosition {
    pub position: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRotation {
    pub yaw: f32,
    pub relative_yaw: bool,
    pub pitch: f32,
    pub relative_pitch: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeDifficulty {
    pub raw_difficulty: i32,
    pub locked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityEvent {
    pub entity_id: i32,
    pub event: i8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameEvent {
    pub event: u8,
    pub parameter: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderInitialization {
    pub center_x: f64,
    pub center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub lerp_millis: i64,
    pub absolute_maximum: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayLogin {
    pub player_entity_id: i32,
    pub hardcore: bool,
    pub levels: BTreeSet<Identifier>,
    pub max_players: i32,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub limited_crafting: bool,
    pub spawn: CommonSpawnInfo,
    pub online_mode: bool,
    pub enforces_secure_chat: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommonSpawnInfo {
    pub dimension_type: Identifier,
    pub dimension: Identifier,
    pub obfuscated_seed: i64,
    pub game_mode: GameMode,
    pub previous_game_mode: Option<GameMode>,
    pub is_debug: bool,
    pub is_flat: bool,
    pub last_death: Option<GlobalBlockPosition>,
    pub portal_cooldown: i32,
    pub sea_level: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    #[must_use]
    pub const fn from_i8_or_survival(value: i8) -> Self {
        match value {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    #[must_use]
    pub const fn from_i32_or_survival(value: i32) -> Self {
        match value {
            1 => Self::Creative,
            2 => Self::Adventure,
            3 => Self::Spectator,
            _ => Self::Survival,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1,
            Self::Adventure => 2,
            Self::Spectator => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalBlockPosition {
    pub dimension: Identifier,
    pub packed_position: i64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerAbilities {
    pub flags: u8,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerPosition {
    pub teleport_id: i32,
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub relative_flags: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerData {
    pub motd: TextComponentNbt,
    pub icon: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSpawnPosition {
    pub position: GlobalBlockPosition,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetTime {
    pub game_time: i64,
    pub clocks: BTreeMap<Identifier, ClockState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockState {
    pub total_ticks: i64,
    pub partial_tick: f32,
    pub rate: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickingState {
    pub tick_rate: f32,
    pub frozen: bool,
}
