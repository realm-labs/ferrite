//! Protocol-neutral lifecycle state and ordered semantic effects.

use ferrite_foundation::identity::StableEntityId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    All,
    Moderators,
    GameMasters,
    Admins,
    Owners,
}

impl PermissionLevel {
    #[must_use]
    pub const fn entity_event(self) -> u8 {
        match self {
            Self::All => 24,
            Self::Moderators => 25,
            Self::GameMasters => 26,
            Self::Admins => 27,
            Self::Owners => 28,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinProjection {
    Login,
    Difficulty,
    Abilities,
    HeldSlot,
    Recipes,
    PermissionAndCommands,
    Statistics,
    RecipeBook,
    Scoreboard,
    Teleport,
    NonTransferStatus,
    LevelInfo,
    Effects,
    Inventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RespawnProjection {
    Respawn { keep_mask: u8 },
    Teleport,
    LevelSpawn,
    Difficulty,
    Experience,
    Effects,
    LevelInfo,
    PermissionAndCommands,
    Inventory,
    Health,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEffect {
    ProfileCached,
    PlayListenerInstalled,
    FlushSuspended,
    JoinProjection(JoinProjection),
    StatusInvalidated,
    JoinBroadcast,
    OldPlayerListQueued,
    LiveListAdded,
    UuidIndexAdded,
    LevelMembershipAdded,
    BossEventsJoined,
    IntegrationHookJoined,
    FlushResumed,
    LastDeathStored,
    WaitingForRespawnSet,
    WonGameCleared,
    OldLiveListRemoved,
    OldLevelMembershipRemoved,
    ReplacementConstructed,
    ConnectionTransferred,
    StateRestored {
        keep_all: bool,
        inventory_retained: bool,
    },
    RespawnProjection(RespawnProjection),
    SpectatorModeForced,
    SleepStopped,
    VehicleRemoved,
    DimensionChangeMarked,
    ItemUseStopped,
    SentMirrorsInvalidated,
    GameModeEvent(GameMode),
    ShoulderEntitiesRemoved,
    RidingStopped,
    LocationEffectsRemoved,
    CameraReset,
    LocationEffectsRefreshed,
    AbilitiesProjected,
    InvisibilityRecomputed,
    PermissionEvent(u8),
    CommandTreeRebuilt,
    ChatChainClosed,
    LeaveBroadcast,
    DisconnectedMarked,
    PassengersEjected,
    LeaveCriterionAwarded,
    PlayerSaved,
    StatisticsSaved,
    AdvancementsSaved,
    RootVehicleRemoved,
    OwnedPearlsRemoved,
    AdvancementTriggersRemoved,
    BossEventsDisconnected,
    UuidIndexRemoved,
    PlayerInfoRemoved,
    TextFilterLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerLifecycleState {
    pub player: StableEntityId,
    pub session_epoch: u64,
    pub incarnation: u64,
    pub mode: GameMode,
    pub previous_mode: Option<GameMode>,
    pub health: u16,
    pub waiting_for_respawn: bool,
    pub won_game: bool,
    pub sleeping: bool,
}

impl PlayerLifecycleState {
    #[must_use]
    pub const fn initial(player: StableEntityId, session_epoch: u64) -> Self {
        Self {
            player,
            session_epoch,
            incarnation: 1,
            mode: GameMode::Survival,
            previous_mode: None,
            health: 20,
            waiting_for_respawn: false,
            won_game: false,
            sleeping: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnRequest {
    pub keep_inventory: bool,
    pub hardcore: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RespawnOutcome {
    IgnoredAlive,
    Replaced {
        keep_all: bool,
        effects: Vec<LifecycleEffect>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleSnapshot {
    pub players: Vec<PlayerLifecycleState>,
}
