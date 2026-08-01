#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionPacketKind {
    BlockValue,
    ChunkValue,
    EntityValue,
    Event,
    Sample,
}

impl DebugProjectionPacketKind {
    pub const ALL: [Self; 5] = [
        Self::BlockValue,
        Self::ChunkValue,
        Self::EntityValue,
        Self::Event,
        Self::Sample,
    ];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::BlockValue => 26,
            Self::ChunkValue => 27,
            Self::EntityValue => 28,
            Self::Event => 29,
            Self::Sample => 30,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::BlockValue => "minecraft:debug/block_value",
            Self::ChunkValue => "minecraft:debug/chunk_value",
            Self::EntityValue => "minecraft:debug/entity_value",
            Self::Event => "minecraft:debug/event",
            Self::Sample => "minecraft:debug_sample",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscription {
    DedicatedServerTickTime,
    Bees,
    Brains,
    Breezes,
    GoalSelectors,
    EntityPaths,
    EntityBlockIntersections,
    BeeHives,
    PointsOfInterest,
    RedstoneWireOrientations,
    VillageSections,
    Raids,
    Structures,
    GameEventListeners,
    NeighborUpdates,
    GameEvents,
}

impl DebugSubscription {
    pub const ALL: [Self; 16] = [
        Self::DedicatedServerTickTime,
        Self::Bees,
        Self::Brains,
        Self::Breezes,
        Self::GoalSelectors,
        Self::EntityPaths,
        Self::EntityBlockIntersections,
        Self::BeeHives,
        Self::PointsOfInterest,
        Self::RedstoneWireOrientations,
        Self::VillageSections,
        Self::Raids,
        Self::Structures,
        Self::GameEventListeners,
        Self::NeighborUpdates,
        Self::GameEvents,
    ];

    #[must_use]
    pub const fn from_raw_id(raw_id: i32) -> Option<Self> {
        match raw_id {
            0 => Some(Self::DedicatedServerTickTime),
            1 => Some(Self::Bees),
            2 => Some(Self::Brains),
            3 => Some(Self::Breezes),
            4 => Some(Self::GoalSelectors),
            5 => Some(Self::EntityPaths),
            6 => Some(Self::EntityBlockIntersections),
            7 => Some(Self::BeeHives),
            8 => Some(Self::PointsOfInterest),
            9 => Some(Self::RedstoneWireOrientations),
            10 => Some(Self::VillageSections),
            11 => Some(Self::Raids),
            12 => Some(Self::Structures),
            13 => Some(Self::GameEventListeners),
            14 => Some(Self::NeighborUpdates),
            15 => Some(Self::GameEvents),
            _ => None,
        }
    }

    #[must_use]
    pub const fn raw_id(self) -> i32 {
        match self {
            Self::DedicatedServerTickTime => 0,
            Self::Bees => 1,
            Self::Brains => 2,
            Self::Breezes => 3,
            Self::GoalSelectors => 4,
            Self::EntityPaths => 5,
            Self::EntityBlockIntersections => 6,
            Self::BeeHives => 7,
            Self::PointsOfInterest => 8,
            Self::RedstoneWireOrientations => 9,
            Self::VillageSections => 10,
            Self::Raids => 11,
            Self::Structures => 12,
            Self::GameEventListeners => 13,
            Self::NeighborUpdates => 14,
            Self::GameEvents => 15,
        }
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        match self {
            Self::DedicatedServerTickTime => "minecraft:dedicated_server_tick_time",
            Self::Bees => "minecraft:bees",
            Self::Brains => "minecraft:brains",
            Self::Breezes => "minecraft:breezes",
            Self::GoalSelectors => "minecraft:goal_selectors",
            Self::EntityPaths => "minecraft:entity_paths",
            Self::EntityBlockIntersections => "minecraft:entity_block_intersections",
            Self::BeeHives => "minecraft:bee_hives",
            Self::PointsOfInterest => "minecraft:pois",
            Self::RedstoneWireOrientations => "minecraft:redstone_wire_orientations",
            Self::VillageSections => "minecraft:village_sections",
            Self::Raids => "minecraft:raids",
            Self::Structures => "minecraft:structures",
            Self::GameEventListeners => "minecraft:game_event_listeners",
            Self::NeighborUpdates => "minecraft:neighbor_updates",
            Self::GameEvents => "minecraft:game_events",
        }
    }

    #[must_use]
    pub const fn retention(self) -> DebugRetention {
        match self {
            Self::DedicatedServerTickTime => DebugRetention::SampleOnly,
            Self::EntityBlockIntersections => DebugRetention::Expiring { ticks: 100 },
            Self::RedstoneWireOrientations | Self::NeighborUpdates => {
                DebugRetention::Expiring { ticks: 200 }
            }
            Self::GameEvents => DebugRetention::Expiring { ticks: 60 },
            Self::Bees
            | Self::Brains
            | Self::Breezes
            | Self::GoalSelectors
            | Self::EntityPaths
            | Self::BeeHives
            | Self::PointsOfInterest
            | Self::VillageSections
            | Self::Raids
            | Self::Structures
            | Self::GameEventListeners => DebugRetention::Persistent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugRetention {
    SampleOnly,
    Persistent,
    Expiring { ticks: u16 },
}

impl DebugRetention {
    #[must_use]
    pub fn deadline(self, published_at: i64) -> Option<i64> {
        match self {
            Self::Expiring { ticks } => Some(published_at.saturating_add(i64::from(ticks))),
            Self::SampleOnly | Self::Persistent => None,
        }
    }

    #[must_use]
    pub fn is_expired(self, published_at: i64, game_time: i64) -> bool {
        self.deadline(published_at)
            .is_some_and(|deadline| game_time >= deadline)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugValueState {
    Replace,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugProjectionPacket {
    BlockValue {
        subscription: DebugSubscription,
        state: DebugValueState,
    },
    ChunkValue {
        subscription: DebugSubscription,
        state: DebugValueState,
    },
    EntityValue {
        subscription: DebugSubscription,
        state: DebugValueState,
    },
    Event {
        subscription: DebugSubscription,
    },
    Sample,
}

impl DebugProjectionPacket {
    #[must_use]
    pub const fn kind(self) -> DebugProjectionPacketKind {
        match self {
            Self::BlockValue { .. } => DebugProjectionPacketKind::BlockValue,
            Self::ChunkValue { .. } => DebugProjectionPacketKind::ChunkValue,
            Self::EntityValue { .. } => DebugProjectionPacketKind::EntityValue,
            Self::Event { .. } => DebugProjectionPacketKind::Event,
            Self::Sample => DebugProjectionPacketKind::Sample,
        }
    }

    #[must_use]
    pub const fn subscription(self) -> DebugSubscription {
        match self {
            Self::BlockValue { subscription, .. }
            | Self::ChunkValue { subscription, .. }
            | Self::EntityValue { subscription, .. }
            | Self::Event { subscription } => subscription,
            Self::Sample => DebugSubscription::DedicatedServerTickTime,
        }
    }
}
