//! End-portal contact routing, platform transaction, and exact block/render surface.

use ferrite_foundation::coordinate::BlockPos;

use super::{ChunkTicket, Rotation, Vec3};

pub const END_SPAWN: BlockPos = BlockPos::new(100, 50, 0);
pub const END_PORTAL_EVENT: u16 = 1032;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub minimum: [f64; 3],
    pub maximum: [f64; 3],
}

pub const END_PORTAL_CONTACT_SHAPE: Aabb = Aabb {
    minimum: [0.0, 6.0 / 16.0, 0.0],
    maximum: [1.0, 12.0 / 16.0, 1.0],
};

pub const END_PORTAL_COLLISION_SHAPE: Option<Aabb> = None;

pub const fn contact_shape_contains(local: [f64; 3]) -> bool {
    local[0] >= 0.0
        && local[0] <= 1.0
        && local[1] >= 6.0 / 16.0
        && local[1] <= 12.0 / 16.0
        && local[2] >= 0.0
        && local[2] <= 1.0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndPortalBlockProperties {
    pub light: u8,
    pub hardness: f32,
    pub explosion_resistance: f32,
    pub has_loot_table: bool,
    pub piston_blocks: bool,
    pub replaceable_by_fluid: bool,
    pub ordinary_block_model: bool,
}

pub const END_PORTAL_PROPERTIES: EndPortalBlockProperties = EndPortalBlockProperties {
    light: 15,
    hardness: -1.0,
    explosion_resistance: 3_600_000.0,
    has_loot_table: false,
    piston_blocks: true,
    replaceable_by_fluid: false,
    ordinary_block_model: false,
};

pub const fn clone_stack(_include_data: bool) -> Option<&'static str> {
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmokeParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub random_draws: u8,
}

pub fn animate_tick(block: BlockPos, mut next_double: impl FnMut() -> f64) -> SmokeParticle {
    let x = next_double();
    let z = next_double();
    SmokeParticle {
        position: Vec3 {
            x: f64::from(block.x) + x,
            y: f64::from(block.y) + 0.8,
            z: f64::from(block.z) + z,
        },
        velocity: Vec3::ZERO,
        random_draws: 2,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

pub const fn should_render_block_entity_face(direction: Direction) -> bool {
    matches!(direction, Direction::Down | Direction::Up)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndPortalBlockEntityContract {
    pub has_subtype_state: bool,
    pub persists_custom_data: bool,
    pub has_update_packet: bool,
    pub has_ticker: bool,
    pub clears_reusable_face_set: bool,
    pub neighbor_culling: bool,
}

pub const END_PORTAL_BLOCK_ENTITY_CONTRACT: EndPortalBlockEntityContract =
    EndPortalBlockEntityContract {
        has_subtype_state: false,
        persists_custom_data: false,
        has_update_packet: false,
        has_ticker: false,
        clears_reusable_face_set: true,
        neighbor_culling: false,
    };

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub direction: Direction,
    pub vertices: [[f32; 3]; 4],
}

pub fn world_block_entity_quads() -> [Quad; 2] {
    [
        horizontal_quad(Direction::Down, 0.375),
        horizontal_quad(Direction::Up, 0.75),
    ]
}

pub fn special_model_quads() -> [Quad; 6] {
    [
        horizontal_quad(Direction::Down, 0.375),
        horizontal_quad(Direction::Up, 0.75),
        vertical_z_quad(Direction::North, 0.0),
        vertical_z_quad(Direction::South, 1.0),
        vertical_x_quad(Direction::West, 0.0),
        vertical_x_quad(Direction::East, 1.0),
    ]
}

fn horizontal_quad(direction: Direction, y: f32) -> Quad {
    Quad {
        direction,
        vertices: [[0.0, y, 0.0], [1.0, y, 0.0], [1.0, y, 1.0], [0.0, y, 1.0]],
    }
}

fn vertical_z_quad(direction: Direction, z: f32) -> Quad {
    Quad {
        direction,
        vertices: [
            [0.0, 0.375, z],
            [1.0, 0.375, z],
            [1.0, 0.75, z],
            [0.0, 0.75, z],
        ],
    }
}

fn vertical_x_quad(direction: Direction, x: f32) -> Quad {
    Quad {
        direction,
        vertices: [
            [x, 0.375, 0.0],
            [x, 0.375, 1.0],
            [x, 0.75, 1.0],
            [x, 0.75, 0.0],
        ],
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndPortalRenderPipeline {
    pub sampler_zero: &'static str,
    pub sampler_one: &'static str,
    pub position_only: bool,
    pub default_depth_state: bool,
    pub portal_layers: u8,
    pub applies_environment_fog: bool,
    pub ignores_light_overlay_foil_outline: bool,
}

pub const END_PORTAL_RENDER_PIPELINE: EndPortalRenderPipeline = EndPortalRenderPipeline {
    sampler_zero: "minecraft:textures/environment/end_sky.png",
    sampler_one: "minecraft:textures/entity/end_portal.png",
    position_only: true,
    default_depth_state: true,
    portal_layers: 15,
    applies_environment_fog: true,
    ignores_light_overlay_foil_outline: true,
};

pub const END_PORTAL_COLORS: [[f32; 3]; 16] = [
    [0.022_087, 0.098_399, 0.110_818],
    [0.011_892, 0.095_924, 0.089_485],
    [0.027_636, 0.101_689, 0.100_326],
    [0.046_564, 0.109_883, 0.114_838],
    [0.064_901, 0.117_696, 0.097_189],
    [0.063_761, 0.086_895, 0.123_646],
    [0.084_817, 0.111_994, 0.166_380],
    [0.097_489, 0.154_120, 0.091_064],
    [0.106_152, 0.131_144, 0.195_191],
    [0.097_721, 0.110_188, 0.187_229],
    [0.133_516, 0.138_278, 0.148_582],
    [0.070_006, 0.243_332, 0.235_792],
    [0.196_766, 0.142_899, 0.214_696],
    [0.047_281, 0.315_338, 0.321_970],
    [0.204_675, 0.390_010, 0.302_066],
    [0.080_955, 0.314_821, 0.661_491],
];

pub const END_PORTAL_SCALE_TRANSLATE: [[f32; 4]; 4] = [
    [0.5, 0.0, 0.0, 0.25],
    [0.0, 0.5, 0.0, 0.25],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndPortalShaderLayer {
    pub layer: u8,
    pub color: [f32; 3],
    pub translation: [f32; 2],
    pub rotation_radians: f32,
    pub scale: f32,
}

pub fn end_portal_shader_layer(layer: u8, game_time: f32) -> Option<EndPortalShaderLayer> {
    if !(1..=END_PORTAL_RENDER_PIPELINE.portal_layers).contains(&layer) {
        return None;
    }
    let value = f32::from(layer);
    Some(EndPortalShaderLayer {
        layer,
        color: END_PORTAL_COLORS[usize::from(layer - 1)],
        translation: [17.0 / value, (2.0 + value / 1.5) * (game_time * 1.5)],
        rotation_radians: ((value * value * 4_321.0 + value * 9.0) * 2.0).to_radians(),
        scale: (4.5 - value / 4.0) * 2.0,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndPortalDesiredBlock {
    Obsidian,
    Air,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndPlatformWrite {
    pub position: BlockPos,
    pub desired: EndPortalDesiredBlock,
    pub destroy_mismatch_with_drops: bool,
}

/// The platform origin is the bottom-center block `(100,48,0)`.
pub fn entering_end_platform() -> Vec<EndPlatformWrite> {
    let mut writes = Vec::with_capacity(100);
    for y in 0..=3 {
        for z in -2..=2 {
            for x in -2..=2 {
                writes.push(EndPlatformWrite {
                    position: BlockPos::new(100 + x, 48 + y, z),
                    desired: if y == 0 {
                        EndPortalDesiredBlock::Obsidian
                    } else {
                        EndPortalDesiredBlock::Air
                    },
                    destroy_mismatch_with_drops: true,
                });
            }
        }
    }
    writes
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CreditsContact {
    pub source_is_literal_end: bool,
    pub is_server_player: bool,
    pub seen_credits: bool,
    pub won_game: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CreditsContactResult {
    pub bypass_processor: bool,
    pub dismount_and_remove: bool,
    pub set_won_game: bool,
    pub set_seen_credits: bool,
    pub send_win_game_event: bool,
}

pub const fn end_portal_credits_contact(input: CreditsContact) -> CreditsContactResult {
    if input.source_is_literal_end && input.is_server_player && !input.seen_credits {
        CreditsContactResult {
            bypass_processor: true,
            dismount_and_remove: true,
            set_won_game: !input.won_game,
            set_seen_credits: !input.won_game,
            send_win_game_event: !input.won_game,
        }
    } else {
        CreditsContactResult {
            bypass_processor: false,
            dismount_and_remove: false,
            set_won_game: false,
            set_seen_credits: false,
            send_win_game_event: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedRespawn {
    pub position: BlockPos,
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EndPortalTransition {
    pub destination_key: &'static str,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Rotation,
    pub yaw_is_absolute: bool,
    pub pitch_is_relative: bool,
    pub player_level_event: Option<u16>,
    pub ticket: Option<ChunkTicket>,
    pub build_platform: bool,
    pub use_player_respawn_resolver: bool,
}

pub fn enter_end(
    destination_available: bool,
    is_server_player: bool,
    velocity: Vec3,
    old_pitch: f32,
) -> Option<EndPortalTransition> {
    destination_available.then(|| {
        let position = Vec3 {
            x: 100.5,
            y: if is_server_player { 49.0 } else { 50.0 },
            z: 0.5,
        };
        EndPortalTransition {
            destination_key: "minecraft:the_end",
            position,
            velocity,
            rotation: Rotation {
                yaw: 90.0,
                pitch: old_pitch,
            },
            yaw_is_absolute: true,
            pitch_is_relative: true,
            player_level_event: is_server_player.then_some(END_PORTAL_EVENT),
            ticket: Some(ChunkTicket::portal(position.containing())),
            build_platform: true,
            use_player_respawn_resolver: false,
        }
    })
}

pub fn leave_end(
    destination_available: bool,
    respawn_dimension_key: &'static str,
    respawn: SavedRespawn,
    is_server_player: bool,
    velocity: Vec3,
) -> Option<EndPortalTransition> {
    destination_available.then(|| {
        let position = Vec3 {
            x: f64::from(respawn.position.x) + 0.5,
            y: f64::from(respawn.position.y),
            z: f64::from(respawn.position.z) + 0.5,
        };
        EndPortalTransition {
            destination_key: respawn_dimension_key,
            position,
            velocity,
            rotation: Rotation {
                yaw: respawn.yaw,
                pitch: respawn.pitch,
            },
            yaw_is_absolute: false,
            pitch_is_relative: true,
            player_level_event: None,
            ticket: (!is_server_player).then_some(ChunkTicket::portal(position.containing())),
            build_platform: false,
            use_player_respawn_resolver: is_server_player,
        }
    })
}
