use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::command::CommandTree;
use crate::java_26_2::play::clientbound::packet::{
    BorderInitialization, ClockState, CommonSpawnInfo, DefaultSpawnPosition, GameMode,
    PlayClientboundPacket, PlayLogin, PlayerAbilities, PlayerPosition, PlayerRotation, ServerData,
    TickingState, Vector3, VehiclePosition,
};
use crate::java_26_2::play::clientbound::player_info::{
    AddedProfile, ChatSession, PlayerInfoEntry, PlayerInfoUpdate,
};
use crate::java_26_2::play::clientbound::recipe::{
    RecipeBookAdd, RecipeBookEntry, RecipeBookSettings, RecipeProjection,
};
use crate::java_26_2::play::serverbound::packet::{
    KeepAlive as ServerboundKeepAlive, MovePlayerRotation, MoveVehicle as ServerboundMoveVehicle,
    MovementFlags, PlayServerboundEntryPacket, PlayerPosition as ServerboundPosition,
    PlayerRotation as ServerboundRotation, Pong,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

const RELATIVE_X: u32 = 1 << 0;
const RELATIVE_Y: u32 = 1 << 1;
const RELATIVE_Z: u32 = 1 << 2;
const RELATIVE_YAW: u32 = 1 << 3;
const RELATIVE_PITCH: u32 = 1 << 4;
const RELATIVE_DELTA_X: u32 = 1 << 5;
const RELATIVE_DELTA_Y: u32 = 1 << 6;
const RELATIVE_DELTA_Z: u32 = 1 << 7;
const ROTATE_DELTA: u32 = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayEntryStage {
    AwaitingLogin,
    AwaitingDifficulty,
    AwaitingAbilities,
    AwaitingHeldSlot,
    AwaitingRecipeProjection,
    AwaitingPermission,
    AwaitingCommands,
    AwaitingRecipeBookSettings,
    AwaitingRecipeBookAdd,
    AwaitingPosition,
    PlayerInfoAndLevelInfo,
    AwaitingTime,
    AwaitingSpawn,
    AwaitingLoadStart,
    AwaitingTickingState,
    AwaitingTickingStep,
    ReadyForTerrain,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayClientAction {
    None,
    Disconnect(TextComponentNbt),
    EchoKeepAlive(i64),
    EchoPing(i32),
    EchoRotation {
        yaw: f32,
        pitch: f32,
    },
    EchoVehicle(VehicleMovementState),
    AcknowledgeTeleportThenEchoMovement {
        teleport_id: i32,
        state: LocalPlayerState,
        reset_block_prediction: bool,
    },
}

impl PlayClientAction {
    #[must_use]
    pub fn response_packet(&self) -> Option<PlayServerboundEntryPacket> {
        match self {
            Self::EchoKeepAlive(challenge) => Some(PlayServerboundEntryPacket::KeepAlive(
                ServerboundKeepAlive {
                    challenge: *challenge,
                },
            )),
            Self::EchoPing(payload) => {
                Some(PlayServerboundEntryPacket::Pong(Pong { payload: *payload }))
            }
            Self::EchoRotation { yaw, pitch } => Some(
                PlayServerboundEntryPacket::MovePlayerRotation(MovePlayerRotation {
                    rotation: ServerboundRotation {
                        yaw: *yaw,
                        pitch: *pitch,
                    },
                    flags: MovementFlags {
                        on_ground: false,
                        horizontal_collision: false,
                    },
                }),
            ),
            Self::EchoVehicle(state) => Some(PlayServerboundEntryPacket::MoveVehicle(
                ServerboundMoveVehicle {
                    position: ServerboundPosition {
                        x: state.position.x,
                        y: state.position.y,
                        z: state.position.z,
                    },
                    rotation: ServerboundRotation {
                        yaw: state.yaw,
                        pitch: state.pitch,
                    },
                    on_ground: state.on_ground,
                },
            )),
            Self::None | Self::Disconnect(_) | Self::AcknowledgeTeleportThenEchoMovement { .. } => {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LocalPlayerState {
    pub position: Vector3,
    pub motion: Vector3,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayerRenderRotation {
    pub old_yaw: f32,
    pub old_pitch: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VehicleMovementState {
    pub position: Vector3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RootVehicleProjection {
    pub movement: VehicleMovementState,
    pub locally_authoritative: bool,
    pub interpolation_target: Option<Vector3>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Peaceful,
    Easy,
    Normal,
    Hard,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AbilityProjection {
    pub invulnerable: bool,
    pub flying: bool,
    pub can_fly: bool,
    pub instant_build: bool,
    pub flying_speed: f32,
    pub walking_speed: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClientLevelProjection {
    pub entity_id: i32,
    pub hardcore: bool,
    pub spawn: CommonSpawnInfo,
    pub levels: BTreeSet<Identifier>,
    pub chunk_radius: i32,
    pub simulation_distance: i32,
    pub reduced_debug_info: bool,
    pub show_death_screen: bool,
    pub limited_crafting: bool,
    pub online_mode: bool,
    pub enforces_secure_chat: bool,
    pub terrain_load_started: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderProjection {
    pub center_x: f64,
    pub center_z: f64,
    pub size: BorderSize,
    pub absolute_maximum: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BorderSize {
    Immediate(f64),
    Lerp {
        old_size: f64,
        new_size: f64,
        duration_millis: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerListEntry {
    pub profile_id: u128,
    pub profile: AddedProfile,
    pub chat_session: Option<ChatSession>,
    pub game_mode: GameMode,
    pub listed: bool,
    pub latency_millis: i32,
    pub display_name: Option<TextComponentNbt>,
    pub list_order: i32,
    pub show_hat: bool,
}

#[derive(Debug, Clone)]
pub struct PlayEntryProjection {
    stage: PlayEntryStage,
    level: Option<ClientLevelProjection>,
    local_player: LocalPlayerState,
    render_rotation: PlayerRenderRotation,
    riding: bool,
    root_vehicle: Option<RootVehicleProjection>,
    difficulty: Difficulty,
    difficulty_locked: bool,
    abilities: AbilityProjection,
    held_slot: usize,
    permission_tier: Option<u8>,
    commands: Option<CommandTree>,
    recipe_settings: RecipeBookSettings,
    recipe_book: Vec<RecipeBookEntry>,
    recipes: Option<RecipeProjection>,
    players: BTreeMap<u128, PlayerListEntry>,
    border: Option<BorderProjection>,
    clocks: BTreeMap<Identifier, ClockState>,
    game_time: i64,
    default_spawn: Option<DefaultSpawnPosition>,
    ticking: Option<TickingState>,
    ticking_steps: i32,
    server_data: Option<ServerData>,
    has_server_list_record: bool,
}

impl PlayEntryProjection {
    #[must_use]
    pub fn new(
        initial_player_state: LocalPlayerState,
        riding: bool,
        has_server_list_record: bool,
    ) -> Self {
        Self {
            stage: PlayEntryStage::AwaitingLogin,
            level: None,
            local_player: initial_player_state,
            render_rotation: PlayerRenderRotation {
                old_yaw: initial_player_state.yaw,
                old_pitch: initial_player_state.pitch,
            },
            riding,
            root_vehicle: riding.then_some(RootVehicleProjection {
                movement: VehicleMovementState {
                    position: initial_player_state.position,
                    yaw: initial_player_state.yaw,
                    pitch: initial_player_state.pitch,
                    on_ground: false,
                },
                locally_authoritative: true,
                interpolation_target: None,
            }),
            difficulty: Difficulty::Normal,
            difficulty_locked: false,
            abilities: AbilityProjection::default(),
            held_slot: 0,
            permission_tier: None,
            commands: None,
            recipe_settings: RecipeBookSettings::default(),
            recipe_book: Vec::new(),
            recipes: None,
            players: BTreeMap::new(),
            border: None,
            clocks: BTreeMap::new(),
            game_time: 0,
            default_spawn: None,
            ticking: None,
            ticking_steps: 0,
            server_data: None,
            has_server_list_record,
        }
    }

    #[must_use]
    pub const fn stage(&self) -> PlayEntryStage {
        self.stage
    }

    #[must_use]
    pub fn level(&self) -> Option<&ClientLevelProjection> {
        self.level.as_ref()
    }

    #[must_use]
    pub const fn local_player(&self) -> LocalPlayerState {
        self.local_player
    }

    #[must_use]
    pub const fn render_rotation(&self) -> PlayerRenderRotation {
        self.render_rotation
    }

    #[must_use]
    pub const fn root_vehicle(&self) -> Option<RootVehicleProjection> {
        self.root_vehicle
    }

    pub const fn set_root_vehicle(&mut self, vehicle: Option<RootVehicleProjection>) {
        self.root_vehicle = vehicle;
    }

    #[must_use]
    pub const fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    #[must_use]
    pub const fn difficulty_locked(&self) -> bool {
        self.difficulty_locked
    }

    #[must_use]
    pub const fn abilities(&self) -> AbilityProjection {
        self.abilities
    }

    #[must_use]
    pub const fn held_slot(&self) -> usize {
        self.held_slot
    }

    #[must_use]
    pub const fn permission_tier(&self) -> Option<u8> {
        self.permission_tier
    }

    #[must_use]
    pub fn players(&self) -> &BTreeMap<u128, PlayerListEntry> {
        &self.players
    }

    #[must_use]
    pub fn commands(&self) -> Option<&CommandTree> {
        self.commands.as_ref()
    }

    #[must_use]
    pub fn recipe_settings(&self) -> RecipeBookSettings {
        self.recipe_settings
    }

    #[must_use]
    pub fn recipe_book(&self) -> &[RecipeBookEntry] {
        &self.recipe_book
    }

    #[must_use]
    pub fn recipes(&self) -> Option<&RecipeProjection> {
        self.recipes.as_ref()
    }

    #[must_use]
    pub fn border(&self) -> Option<&BorderProjection> {
        self.border.as_ref()
    }

    #[must_use]
    pub fn clocks(&self) -> &BTreeMap<Identifier, ClockState> {
        &self.clocks
    }

    #[must_use]
    pub const fn game_time(&self) -> i64 {
        self.game_time
    }

    #[must_use]
    pub fn default_spawn(&self) -> Option<&DefaultSpawnPosition> {
        self.default_spawn.as_ref()
    }

    #[must_use]
    pub fn ticking(&self) -> Option<TickingState> {
        self.ticking
    }

    #[must_use]
    pub const fn ticking_steps(&self) -> i32 {
        self.ticking_steps
    }

    #[must_use]
    pub fn server_data(&self) -> Option<&ServerData> {
        self.server_data.as_ref()
    }

    pub fn apply(
        &mut self,
        packet: PlayClientboundPacket,
    ) -> Result<PlayClientAction, PlayProjectionError> {
        if self.stage == PlayEntryStage::AwaitingLogin {
            return match packet {
                PlayClientboundPacket::Login(login) => {
                    self.install_level(login);
                    self.stage = PlayEntryStage::AwaitingDifficulty;
                    Ok(PlayClientAction::None)
                }
                _ => Err(PlayProjectionError::LevelNotInstalled),
            };
        }
        match packet {
            PlayClientboundPacket::Login(_) => Err(PlayProjectionError::DuplicateLogin),
            PlayClientboundPacket::Disconnect(reason) => Ok(PlayClientAction::Disconnect(reason)),
            PlayClientboundPacket::KeepAlive(packet) => {
                Ok(PlayClientAction::EchoKeepAlive(packet.challenge))
            }
            PlayClientboundPacket::MoveVehicle(packet) => Ok(self
                .apply_vehicle_correction(packet)
                .map_or(PlayClientAction::None, PlayClientAction::EchoVehicle)),
            PlayClientboundPacket::Ping(packet) => Ok(PlayClientAction::EchoPing(packet.payload)),
            PlayClientboundPacket::PlayerRotation(packet) => {
                self.apply_player_rotation(packet);
                Ok(PlayClientAction::EchoRotation {
                    yaw: self.local_player.yaw,
                    pitch: self.local_player.pitch,
                })
            }
            PlayClientboundPacket::ChangeDifficulty(packet) => {
                self.require_stage(PlayEntryStage::AwaitingDifficulty, "difficulty")?;
                self.difficulty = difficulty_from_raw(packet.raw_difficulty);
                self.difficulty_locked = packet.locked;
                self.stage = PlayEntryStage::AwaitingAbilities;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::PlayerAbilities(packet) => {
                self.require_stage(PlayEntryStage::AwaitingAbilities, "abilities")?;
                self.apply_abilities(packet);
                self.stage = PlayEntryStage::AwaitingHeldSlot;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::SetHeldSlot(slot) => {
                self.require_stage(PlayEntryStage::AwaitingHeldSlot, "held slot")?;
                if let Ok(slot) = usize::try_from(slot)
                    && slot < 9
                {
                    self.held_slot = slot;
                }
                self.stage = PlayEntryStage::AwaitingRecipeProjection;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::UpdateRecipes(recipes) => {
                self.require_stage(
                    PlayEntryStage::AwaitingRecipeProjection,
                    "recipe projection",
                )?;
                self.recipes = Some(recipes);
                self.stage = PlayEntryStage::AwaitingPermission;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::EntityEvent(event) => {
                self.require_stage(PlayEntryStage::AwaitingPermission, "permission event")?;
                if self
                    .level
                    .as_ref()
                    .is_some_and(|level| event.entity_id == level.entity_id)
                    && (24..=28).contains(&event.event)
                {
                    self.permission_tier = Some((event.event - 24) as u8);
                }
                self.stage = PlayEntryStage::AwaitingCommands;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::Commands(commands) => {
                self.require_stage(PlayEntryStage::AwaitingCommands, "commands")?;
                self.commands = Some(commands);
                self.stage = PlayEntryStage::AwaitingRecipeBookSettings;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::RecipeBookSettings(settings) => {
                self.require_stage(
                    PlayEntryStage::AwaitingRecipeBookSettings,
                    "recipe-book settings",
                )?;
                self.recipe_settings = settings;
                self.stage = PlayEntryStage::AwaitingRecipeBookAdd;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::RecipeBookAdd(packet) => {
                self.require_stage(PlayEntryStage::AwaitingRecipeBookAdd, "recipe-book add")?;
                self.apply_recipe_book(packet);
                self.stage = PlayEntryStage::AwaitingPosition;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::Respawn(packet) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "respawn")?;
                let retention = packet.retention();
                let level = self
                    .level
                    .as_mut()
                    .ok_or(PlayProjectionError::LevelNotInstalled)?;
                level.spawn = packet.spawn;
                level.terrain_load_started = false;
                if !retention.entity_data {
                    self.local_player = LocalPlayerState {
                        yaw: -180.0,
                        ..LocalPlayerState::default()
                    };
                }
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::PlayerPosition(position) => {
                self.require_stage(PlayEntryStage::AwaitingPosition, "player position")?;
                let action = self.apply_position(position);
                self.stage = PlayEntryStage::PlayerInfoAndLevelInfo;
                Ok(action)
            }
            PlayClientboundPacket::ServerData(data) => {
                self.require_stage(PlayEntryStage::PlayerInfoAndLevelInfo, "server data")?;
                if self.has_server_list_record {
                    self.server_data = Some(filter_server_icon(data));
                }
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::PlayerInfoUpdate(update) => {
                self.require_stage(PlayEntryStage::PlayerInfoAndLevelInfo, "player info")?;
                self.apply_player_info(update);
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::InitializeBorder(border) => {
                self.require_stage(
                    PlayEntryStage::PlayerInfoAndLevelInfo,
                    "border initialization",
                )?;
                self.border = Some(project_border(border));
                self.stage = PlayEntryStage::AwaitingTime;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::SetTime(time) => {
                self.require_stage(PlayEntryStage::AwaitingTime, "time")?;
                self.game_time = time.game_time;
                self.clocks = time.clocks;
                self.stage = PlayEntryStage::AwaitingSpawn;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::SetDefaultSpawnPosition(spawn) => {
                self.require_stage(PlayEntryStage::AwaitingSpawn, "default spawn")?;
                self.default_spawn = Some(spawn);
                self.stage = PlayEntryStage::AwaitingLoadStart;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::GameEvent(event) => {
                self.apply_game_event(event.event, event.parameter)?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::TickingState(state) => {
                self.require_stage(PlayEntryStage::AwaitingTickingState, "ticking state")?;
                self.ticking = Some(state);
                self.stage = PlayEntryStage::AwaitingTickingStep;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::TickingStep(steps) => {
                self.require_stage(PlayEntryStage::AwaitingTickingStep, "ticking step")?;
                self.ticking_steps = steps;
                self.stage = PlayEntryStage::ReadyForTerrain;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::Terrain(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "terrain")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::BlockChangedAck(_)
            | PlayClientboundPacket::BlockDestruction(_)
            | PlayClientboundPacket::BlockEntityData(_)
            | PlayClientboundPacket::BlockEvent(_)
            | PlayClientboundPacket::BlockUpdate(_)
            | PlayClientboundPacket::SectionBlocksUpdate(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "block convergence")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::ContainerClose(_)
            | PlayClientboundPacket::ContainerSetContent(_)
            | PlayClientboundPacket::ContainerSetData(_)
            | PlayClientboundPacket::ContainerSetSlot(_)
            | PlayClientboundPacket::OpenScreen(_)
            | PlayClientboundPacket::SetCursorItem(_)
            | PlayClientboundPacket::SetPlayerInventory(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "container convergence")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::MapItemData(_)
            | PlayClientboundPacket::MerchantOffers(_)
            | PlayClientboundPacket::MountScreenOpen(_)
            | PlayClientboundPacket::OpenBook(_)
            | PlayClientboundPacket::OpenSignEditor(_)
            | PlayClientboundPacket::PlaceGhostRecipe(_)
            | PlayClientboundPacket::RecipeBookRemove(_)
            | PlayClientboundPacket::TagQuery(_)
            | PlayClientboundPacket::UpdateAdvancements(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "inventory progression")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::PlayerCombatEnd(_)
            | PlayClientboundPacket::PlayerCombatEnter
            | PlayClientboundPacket::PlayerCombatKill(_)
            | PlayClientboundPacket::PlayerLookAt(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "combat and look")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::Explosion(_)
            | PlayClientboundPacket::RemoveMobEffect(_)
            | PlayClientboundPacket::UpdateMobEffect(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "entity effects")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::LevelParticles(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "particle projection")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::EntityPositionSync(_)
            | PlayClientboundPacket::MoveEntityPosition(_)
            | PlayClientboundPacket::MoveEntityPositionRotation(_)
            | PlayClientboundPacket::MoveEntityRotation(_)
            | PlayClientboundPacket::MoveMinecartAlongTrack(_)
            | PlayClientboundPacket::ProjectilePower(_)
            | PlayClientboundPacket::RotateHead(_)
            | PlayClientboundPacket::SetEntityMotion(_)
            | PlayClientboundPacket::TeleportEntity(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "entity motion")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::BossEvent(_) | PlayClientboundPacket::Waypoint(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "boss or waypoint")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::DeleteChat(_)
            | PlayClientboundPacket::DisguisedChat(_)
            | PlayClientboundPacket::PlayerChat(_)
            | PlayClientboundPacket::SystemChat(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "chat presentation")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::CommandSuggestions(_)
            | PlayClientboundPacket::CustomChatCompletions(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "completion projection")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::Animate(_)
            | PlayClientboundPacket::DamageEvent(_)
            | PlayClientboundPacket::HurtAnimation(_)
            | PlayClientboundPacket::SetCamera(_)
            | PlayClientboundPacket::TakeItemEntity(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "entity session")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::AddEntity(_) | PlayClientboundPacket::RemoveEntities(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "entity spawn")?;
                Ok(PlayClientAction::None)
            }
            PlayClientboundPacket::SetEntityData(_)
            | PlayClientboundPacket::SetEntityLink(_)
            | PlayClientboundPacket::SetEquipment(_)
            | PlayClientboundPacket::SetPassengers(_)
            | PlayClientboundPacket::UpdateAttributes(_) => {
                self.require_stage(PlayEntryStage::ReadyForTerrain, "entity state")?;
                Ok(PlayClientAction::None)
            }
        }
    }

    fn apply_player_rotation(&mut self, packet: PlayerRotation) {
        self.local_player.yaw = if packet.relative_yaw {
            self.local_player.yaw + packet.yaw
        } else {
            packet.yaw
        };
        let pitch = if packet.relative_pitch {
            self.local_player.pitch + packet.pitch
        } else {
            packet.pitch
        };
        self.local_player.pitch = pitch.clamp(-90.0, 90.0);
        self.render_rotation = PlayerRenderRotation {
            old_yaw: self.local_player.yaw,
            old_pitch: self.local_player.pitch,
        };
    }

    fn apply_vehicle_correction(
        &mut self,
        packet: VehiclePosition,
    ) -> Option<VehicleMovementState> {
        let vehicle = self.root_vehicle.as_mut()?;
        if !vehicle.locally_authoritative {
            return None;
        }
        let comparison_position = vehicle
            .interpolation_target
            .unwrap_or(vehicle.movement.position);
        if euclidean_distance(packet.position, comparison_position) > f64::from(1.0e-5_f32) {
            vehicle.interpolation_target = None;
            vehicle.movement.position = packet.position;
            vehicle.movement.yaw = packet.yaw;
            vehicle.movement.pitch = packet.pitch;
        }
        Some(vehicle.movement)
    }

    fn install_level(&mut self, login: PlayLogin) {
        self.level = Some(ClientLevelProjection {
            entity_id: login.player_entity_id,
            hardcore: login.hardcore,
            spawn: login.spawn,
            levels: login.levels,
            chunk_radius: login.chunk_radius,
            simulation_distance: login.simulation_distance,
            reduced_debug_info: login.reduced_debug_info,
            show_death_screen: login.show_death_screen,
            limited_crafting: login.limited_crafting,
            online_mode: login.online_mode,
            enforces_secure_chat: login.enforces_secure_chat,
            terrain_load_started: false,
        });
    }

    fn apply_abilities(&mut self, packet: PlayerAbilities) {
        self.abilities = AbilityProjection {
            invulnerable: packet.flags & 0x01 != 0,
            flying: packet.flags & 0x02 != 0,
            can_fly: packet.flags & 0x04 != 0,
            instant_build: packet.flags & 0x08 != 0,
            flying_speed: packet.flying_speed,
            walking_speed: packet.walking_speed,
        };
    }

    fn apply_recipe_book(&mut self, packet: RecipeBookAdd) {
        if packet.replace {
            self.recipe_book.clear();
        }
        self.recipe_book.extend(packet.entries);
    }

    fn apply_player_info(&mut self, update: PlayerInfoUpdate) {
        for update_entry in update.entries {
            self.apply_player_entry(update_entry);
        }
    }

    fn apply_player_entry(&mut self, update: PlayerInfoEntry) {
        if let Some(profile) = update.added_profile {
            self.players.insert(
                update.profile_id,
                PlayerListEntry {
                    profile_id: update.profile_id,
                    profile,
                    chat_session: None,
                    game_mode: GameMode::Survival,
                    listed: false,
                    latency_millis: 0,
                    display_name: None,
                    list_order: 0,
                    show_hat: false,
                },
            );
        }
        let Some(entry) = self.players.get_mut(&update.profile_id) else {
            return;
        };
        if let Some(chat) = update.chat_session {
            entry.chat_session = if self
                .level
                .as_ref()
                .is_some_and(|level| level.enforces_secure_chat)
            {
                None
            } else {
                chat
            };
        }
        if let Some(game_mode) = update.game_mode {
            entry.game_mode = game_mode;
        }
        if let Some(listed) = update.listed {
            entry.listed = listed;
        }
        if let Some(latency) = update.latency_millis {
            entry.latency_millis = latency;
        }
        if let Some(display_name) = update.display_name {
            entry.display_name = display_name;
        }
        if let Some(list_order) = update.list_order {
            entry.list_order = list_order;
        }
        if let Some(show_hat) = update.show_hat {
            entry.show_hat = show_hat;
        }
    }

    fn apply_position(&mut self, packet: PlayerPosition) -> PlayClientAction {
        if !self.riding {
            self.local_player = calculate_absolute(self.local_player, packet);
        }
        PlayClientAction::AcknowledgeTeleportThenEchoMovement {
            teleport_id: packet.teleport_id,
            state: self.local_player,
            reset_block_prediction: true,
        }
    }

    fn apply_game_event(&mut self, event: u8, parameter: f32) -> Result<(), PlayProjectionError> {
        if self.stage == PlayEntryStage::AwaitingLoadStart && event == 13 {
            if let Some(level) = &mut self.level {
                level.terrain_load_started = true;
            }
            self.stage = PlayEntryStage::AwaitingTickingState;
            return Ok(());
        }
        self.require_stage(PlayEntryStage::AwaitingLoadStart, "game event")?;
        if let Some(level) = &mut self.level {
            match event {
                3 => level.spawn.game_mode = GameMode::from_i32_or_survival(parameter as i32),
                11 => level.show_death_screen = parameter == 0.0,
                12 => level.limited_crafting = parameter != 0.0,
                _ => {}
            }
        }
        Ok(())
    }

    fn require_stage(
        &self,
        expected: PlayEntryStage,
        packet: &'static str,
    ) -> Result<(), PlayProjectionError> {
        if self.stage == expected {
            Ok(())
        } else {
            Err(PlayProjectionError::UnexpectedOrder {
                packet,
                expected,
                actual: self.stage,
            })
        }
    }
}

impl Default for PlayEntryProjection {
    fn default() -> Self {
        Self::new(LocalPlayerState::default(), false, true)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PlayProjectionError {
    #[error("play login must install the client level before level-dependent packets")]
    LevelNotInstalled,
    #[error("a second play login cannot recreate the active client level")]
    DuplicateLogin,
    #[error("{packet} requires entry stage {expected:?}, but projection is {actual:?}")]
    UnexpectedOrder {
        packet: &'static str,
        expected: PlayEntryStage,
        actual: PlayEntryStage,
    },
}

fn difficulty_from_raw(raw: i32) -> Difficulty {
    match raw.rem_euclid(4) {
        0 => Difficulty::Peaceful,
        1 => Difficulty::Easy,
        2 => Difficulty::Normal,
        _ => Difficulty::Hard,
    }
}

fn project_border(border: BorderInitialization) -> BorderProjection {
    BorderProjection {
        center_x: border.center_x,
        center_z: border.center_z,
        size: if border.lerp_millis <= 0 {
            BorderSize::Immediate(border.new_size)
        } else {
            BorderSize::Lerp {
                old_size: border.old_size,
                new_size: border.new_size,
                duration_millis: border.lerp_millis,
            }
        },
        absolute_maximum: border.absolute_maximum,
        warning_blocks: border.warning_blocks,
        warning_time: border.warning_time,
    }
}

fn filter_server_icon(mut data: ServerData) -> ServerData {
    if data.icon.as_deref().is_some_and(|icon| !valid_png(icon)) {
        data.icon = None;
    }
    data
}

fn valid_png(bytes: &[u8]) -> bool {
    bytes.len() >= 24
        && bytes[..8] == [137, 80, 78, 71, 13, 10, 26, 10]
        && bytes[12..16] == *b"IHDR"
        && u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) > 0
        && u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) > 0
}

fn calculate_absolute(current: LocalPlayerState, change: PlayerPosition) -> LocalPlayerState {
    let yaw = if change.relative_flags & RELATIVE_YAW != 0 {
        current.yaw + change.yaw
    } else {
        change.yaw
    };
    let pitch = if change.relative_flags & RELATIVE_PITCH != 0 {
        current.pitch + change.pitch
    } else {
        change.pitch
    }
    .clamp(-90.0, 90.0);
    let mut prior_motion = current.motion;
    if change.relative_flags & ROTATE_DELTA != 0 {
        prior_motion = rotate_x(prior_motion, (current.pitch - pitch).to_radians());
        prior_motion = rotate_y(prior_motion, (current.yaw - yaw).to_radians());
    }
    LocalPlayerState {
        position: Vector3 {
            x: absolute_component(
                current.position.x,
                change.position.x,
                change.relative_flags & RELATIVE_X != 0,
            ),
            y: absolute_component(
                current.position.y,
                change.position.y,
                change.relative_flags & RELATIVE_Y != 0,
            ),
            z: absolute_component(
                current.position.z,
                change.position.z,
                change.relative_flags & RELATIVE_Z != 0,
            ),
        },
        motion: Vector3 {
            x: absolute_component(
                prior_motion.x,
                change.motion.x,
                change.relative_flags & RELATIVE_DELTA_X != 0,
            ),
            y: absolute_component(
                prior_motion.y,
                change.motion.y,
                change.relative_flags & RELATIVE_DELTA_Y != 0,
            ),
            z: absolute_component(
                prior_motion.z,
                change.motion.z,
                change.relative_flags & RELATIVE_DELTA_Z != 0,
            ),
        },
        yaw,
        pitch,
    }
}

fn euclidean_distance(left: Vector3, right: Vector3) -> f64 {
    let x = left.x - right.x;
    let y = left.y - right.y;
    let z = left.z - right.z;
    (x * x + y * y + z * z).sqrt()
}

fn absolute_component(current: f64, change: f64, relative: bool) -> f64 {
    if relative { current + change } else { change }
}

fn rotate_x(vector: Vector3, radians: f32) -> Vector3 {
    let sin = f64::from(radians.sin());
    let cos = f64::from(radians.cos());
    Vector3 {
        x: vector.x,
        y: vector.y * cos + vector.z * sin,
        z: vector.z * cos - vector.y * sin,
    }
}

fn rotate_y(vector: Vector3, radians: f32) -> Vector3 {
    let sin = f64::from(radians.sin());
    let cos = f64::from(radians.cos());
    Vector3 {
        x: vector.x * cos + vector.z * sin,
        y: vector.y,
        z: vector.z * cos - vector.x * sin,
    }
}
