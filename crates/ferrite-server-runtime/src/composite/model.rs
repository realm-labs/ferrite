use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::tick::GameTick;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CompositeOwner {
    Ingress = 0,
    PlayerService = 1,
    Simulation = 2,
    EntityService = 3,
    WorldService = 4,
}

impl CompositeOwner {
    pub const ALL: [Self; 5] = [
        Self::Ingress,
        Self::PlayerService,
        Self::Simulation,
        Self::EntityService,
        Self::WorldService,
    ];

    pub const fn stable_tag(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum CompositeStage {
    Ingress = 0,
    PlayerService = 1,
    Simulation = 2,
    EntityService = 3,
    WorldService = 4,
    Reconciliation = 5,
    Continuity = 6,
    Commit = 7,
    Projection = 8,
}

impl CompositeStage {
    pub const ALL: [Self; 9] = [
        Self::Ingress,
        Self::PlayerService,
        Self::Simulation,
        Self::EntityService,
        Self::WorldService,
        Self::Reconciliation,
        Self::Continuity,
        Self::Commit,
        Self::Projection,
    ];

    pub const fn stable_tag(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeCommand {
    tick: GameTick,
    owner: CompositeOwner,
    sequence: u64,
    kind: ResourceId,
    payload: Box<[u8]>,
}

impl CompositeCommand {
    #[must_use]
    pub fn new(
        tick: GameTick,
        owner: CompositeOwner,
        sequence: u64,
        kind: ResourceId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            tick,
            owner,
            sequence,
            kind,
            payload: payload.into_boxed_slice(),
        }
    }

    pub const fn tick(&self) -> GameTick {
        self.tick
    }

    pub const fn owner(&self) -> CompositeOwner {
        self.owner
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeProjection {
    owner: CompositeOwner,
    sequence: u64,
    kind: ResourceId,
    payload: Box<[u8]>,
}

impl CompositeProjection {
    #[must_use]
    pub fn new(owner: CompositeOwner, sequence: u64, kind: ResourceId, payload: Vec<u8>) -> Self {
        Self {
            owner,
            sequence,
            kind,
            payload: payload.into_boxed_slice(),
        }
    }

    pub const fn owner(&self) -> CompositeOwner {
        self.owner
    }

    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    pub const fn kind(&self) -> &ResourceId {
        &self.kind
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositeCommitReceipt {
    pub tick: GameTick,
    pub replay_identity: [u8; 32],
    pub continuity_hash: [u8; 32],
    pub projection_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompositeEvent {
    pub sequence: u64,
    pub tick: GameTick,
    pub stage: CompositeStage,
    pub replay_identity: Option<[u8; 32]>,
}
