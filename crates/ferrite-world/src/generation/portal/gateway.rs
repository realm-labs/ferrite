//! End-gateway persistence, cooldown, exit search, generation, and transition geometry.

use ferrite_foundation::coordinate::BlockPos;

use super::{ChunkTicket, Rotation, Vec3};

pub const GATEWAY_SPAWNING_AGE: i64 = 200;
pub const GATEWAY_COOLDOWN: i32 = 40;
pub const GATEWAY_ATTENTION_INTERVAL: i64 = 2_400;
pub const GATEWAY_RADIAL_DISTANCE: f64 = 1_024.0;
pub const GATEWAY_RADIAL_STEP: f64 = 16.0;
pub const GATEWAY_RADIAL_LIMIT: u8 = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SavedGateway {
    pub age: i64,
    pub exit_position: Option<BlockPos>,
    pub exact_teleport: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EndGateway {
    pub age: i64,
    pub exit_position: Option<BlockPos>,
    pub exact_teleport: bool,
    /// Transient and intentionally absent from `SavedGateway`.
    pub cooldown: i32,
}

impl EndGateway {
    pub const fn from_saved(saved: SavedGateway) -> Self {
        Self {
            age: saved.age,
            exit_position: saved.exit_position,
            exact_teleport: saved.exact_teleport,
            cooldown: 0,
        }
    }

    pub const fn saved(self) -> SavedGateway {
        SavedGateway {
            age: self.age,
            exit_position: self.exit_position,
            exact_teleport: self.exact_teleport,
        }
    }

    pub const fn spawning(self) -> bool {
        self.age < GATEWAY_SPAWNING_AGE
    }

    pub const fn cooling(self) -> bool {
        self.cooldown > 0
    }

    pub fn tick(&mut self) -> GatewayTick {
        self.age = self.age.wrapping_add(1);
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return GatewayTick {
                broadcast_cooldown: false,
                cooldown: self.cooldown,
                attention_trigger: false,
            };
        }
        if self.age.rem_euclid(GATEWAY_ATTENTION_INTERVAL) == 0 {
            self.cooldown = GATEWAY_COOLDOWN;
            GatewayTick {
                broadcast_cooldown: true,
                cooldown: self.cooldown,
                attention_trigger: true,
            }
        } else {
            GatewayTick {
                broadcast_cooldown: false,
                cooldown: 0,
                attention_trigger: false,
            }
        }
    }

    /// Marks contact and starts/broadcasts cooldown before destination work.
    pub fn contact<T>(&mut self, resolve: impl FnOnce() -> Option<T>) -> GatewayContact<T> {
        if self.cooling() {
            return GatewayContact {
                admitted: false,
                marked_entity: false,
                broadcast_cooldown: false,
                transition: None,
            };
        }
        self.cooldown = GATEWAY_COOLDOWN;
        GatewayContact {
            admitted: true,
            marked_entity: true,
            broadcast_cooldown: true,
            transition: resolve(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayTick {
    pub broadcast_cooldown: bool,
    pub cooldown: i32,
    pub attention_trigger: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GatewayContact<T> {
    pub admitted: bool,
    pub marked_entity: bool,
    pub broadcast_cooldown: bool,
    pub transition: Option<T>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewaySurfaceCandidate {
    pub position: BlockPos,
    pub full_collision: bool,
    pub bedrock: bool,
    pub encounter_order: u32,
}

pub fn configured_exit(
    source_is_literal_end: bool,
    exit: Option<BlockPos>,
    exact: bool,
    candidates: impl IntoIterator<Item = GatewaySurfaceCandidate>,
) -> Option<BlockPos> {
    if !source_is_literal_end {
        return None;
    }
    let exit = exit?;
    if exact {
        return Some(exit);
    }
    let center = BlockPos::new(exit.x, exit.y + 2, exit.z);
    candidates
        .into_iter()
        .filter(|candidate| candidate.position.x != center.x || candidate.position.z != center.z)
        .filter(|candidate| candidate.full_collision && !candidate.bedrock)
        .filter(|candidate| {
            (candidate.position.x - center.x).abs() <= 5
                && (candidate.position.z - center.z).abs() <= 5
        })
        .max_by(|left, right| {
            left.position
                .y
                .cmp(&right.position.y)
                .then_with(|| right.encounter_order.cmp(&left.encounter_order))
        })
        .map_or_else(
            || Some(BlockPos::new(exit.x, exit.y + 3, exit.z)),
            |candidate| {
                Some(BlockPos::new(
                    candidate.position.x,
                    candidate.position.y + 1,
                    candidate.position.z,
                ))
            },
        )
}

pub const fn may_generate_unconfigured(
    source_is_literal_end: bool,
    exit_position: Option<BlockPos>,
) -> bool {
    source_is_literal_end && exit_position.is_none()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayEndStoneCandidate {
    pub position: BlockPos,
    pub end_stone: bool,
    pub two_clear_above: bool,
    pub encounter_order: u32,
}

pub fn select_gateway_anchor(
    radial_position: [f64; 2],
    candidates: impl IntoIterator<Item = GatewayEndStoneCandidate>,
) -> (BlockPos, bool) {
    candidates
        .into_iter()
        .filter(|candidate| candidate.position.y >= 30)
        .filter(|candidate| candidate.end_stone && candidate.two_clear_above)
        .min_by(|left, right| {
            origin_distance(left.position)
                .cmp(&origin_distance(right.position))
                .then(left.encounter_order.cmp(&right.encounter_order))
        })
        .map_or_else(
            || {
                (
                    BlockPos::new(
                        (radial_position[0] + 0.5).floor() as i32,
                        75,
                        (radial_position[1] + 0.5).floor() as i32,
                    ),
                    true,
                )
            },
            |candidate| (candidate.position, false),
        )
}

fn origin_distance(position: BlockPos) -> i128 {
    let x = i128::from(position.x);
    let y = i128::from(position.y);
    let z = i128::from(position.z);
    x * x + y * y + z * z
}

/// Implements the 1024-outward, backward-through-land, forward-through-void walk.
pub fn radial_chunk_walk(
    source: BlockPos,
    mut chunk_is_empty: impl FnMut([i32; 2]) -> bool,
) -> [f64; 2] {
    let length = (f64::from(source.x).powi(2) + f64::from(source.z).powi(2)).sqrt();
    let direction = if length < 1.0e-12 {
        [0.0, 0.0]
    } else {
        [f64::from(source.x) / length, f64::from(source.z) / length]
    };
    let mut position = [
        direction[0] * GATEWAY_RADIAL_DISTANCE,
        direction[1] * GATEWAY_RADIAL_DISTANCE,
    ];
    for _ in 0..GATEWAY_RADIAL_LIMIT {
        if chunk_is_empty(chunk(position)) {
            break;
        }
        position[0] -= direction[0] * GATEWAY_RADIAL_STEP;
        position[1] -= direction[1] * GATEWAY_RADIAL_STEP;
    }
    for _ in 0..GATEWAY_RADIAL_LIMIT {
        if !chunk_is_empty(chunk(position)) {
            break;
        }
        position[0] += direction[0] * GATEWAY_RADIAL_STEP;
        position[1] += direction[1] * GATEWAY_RADIAL_STEP;
    }
    position
}

fn chunk(position: [f64; 2]) -> [i32; 2] {
    [
        (position[0].floor() as i32).div_euclid(16),
        (position[1].floor() as i32).div_euclid(16),
    ]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayFullBlock {
    pub position: BlockPos,
    pub full_collision: bool,
    pub scan_order: u32,
}

pub fn reciprocal_gateway_position(
    anchor: BlockPos,
    blocks: impl IntoIterator<Item = GatewayFullBlock>,
) -> BlockPos {
    let surface = blocks
        .into_iter()
        .filter(|block| block.full_collision)
        .filter(|block| {
            (block.position.x - anchor.x).abs() <= 16 && (block.position.z - anchor.z).abs() <= 16
        })
        .max_by(|left, right| {
            left.position
                .y
                .cmp(&right.position.y)
                .then_with(|| right.scan_order.cmp(&left.scan_order))
        })
        .map_or(anchor, |block| block.position);
    BlockPos::new(surface.x, surface.y + 10, surface.z)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GatewayGeneration {
    pub anchor: BlockPos,
    pub place_end_island: bool,
    pub reciprocal_gateway: BlockPos,
    pub reciprocal_exit: BlockPos,
    pub reciprocal_exact: bool,
    pub stored_exit: BlockPos,
    pub retained_source_exact: bool,
    pub fresh_feature_random_source: bool,
}

pub fn generation_plan(
    source_gateway: BlockPos,
    source_exact: bool,
    anchor: BlockPos,
    place_end_island: bool,
    reciprocal_gateway: BlockPos,
) -> GatewayGeneration {
    GatewayGeneration {
        anchor,
        place_end_island,
        reciprocal_gateway,
        reciprocal_exit: source_gateway,
        reciprocal_exact: false,
        stored_exit: reciprocal_gateway,
        retained_source_exact: source_exact,
        fresh_feature_random_source: place_end_island,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GatewayTransition {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub same_level: bool,
    pub relative_motion_and_rotation: bool,
    pub ticket: ChunkTicket,
    pub portal_sound: bool,
}

pub fn gateway_transition(
    exit: BlockPos,
    is_ender_pearl: bool,
    velocity: Vec3,
    rotation: Rotation,
) -> GatewayTransition {
    let position = Vec3 {
        x: f64::from(exit.x) + 0.5,
        y: f64::from(exit.y),
        z: f64::from(exit.z) + 0.5,
    };
    GatewayTransition {
        position,
        velocity: if is_ender_pearl { Vec3::ZERO } else { velocity },
        rotation: if is_ender_pearl {
            Rotation {
                yaw: 0.0,
                pitch: 0.0,
            }
        } else {
            rotation
        },
        same_level: true,
        relative_motion_and_rotation: !is_ender_pearl,
        ticket: ChunkTicket::portal(position.containing()),
        portal_sound: false,
    }
}
