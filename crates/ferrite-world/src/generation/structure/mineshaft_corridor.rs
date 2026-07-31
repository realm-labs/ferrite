//! Mineshaft-corridor carving, supports, decoration, carts, spiders, and rails.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::mineshaft_graph::{MineshaftCorridor, MineshaftType};
use crate::generation::structure::mineshaft_place::{
    MineshaftChestCartSpawn, MineshaftFace, MineshaftWorld, interior, invalid_location, is_air,
    place_replacing,
};
use crate::generation::structure::piece::{OrientedPiece, PieceWorld};
use crate::generation::structure::processor::StructureState;

pub fn place_mineshaft_corridor<R, F>(
    world: &mut impl MineshaftWorld,
    corridor: &mut MineshaftCorridor,
    kind: MineshaftType,
    clip: &BlockBox,
    random: &mut R,
    loot_seed: &mut F,
) -> bool
where
    R: GenerationRandom,
    F: FnMut() -> i64,
{
    if invalid_location(world, corridor.bounding_box, clip) {
        return false;
    }
    let piece = OrientedPiece {
        bounds: corridor.bounding_box,
        orientation: corridor.orientation,
    };
    let length = corridor.sections * 5 - 1;
    for y in 0..=1 {
        for x in 0..=2 {
            for z in 0..=length {
                place_local(world, kind, piece, clip, BlockPos::new(x, y, z), cave_air());
            }
        }
    }
    for x in 0..=2 {
        for z in 0..=length {
            if random.next_f32() <= 0.8 {
                place_local(world, kind, piece, clip, BlockPos::new(x, 2, z), cave_air());
            }
        }
    }
    if corridor.spider_corridor {
        for y in 0..=1 {
            for x in 0..=2 {
                for z in 0..=length {
                    if random.next_f32() <= 0.6
                        && interior(world, piece, clip, BlockPos::new(x, y, z))
                    {
                        place_local(
                            world,
                            kind,
                            piece,
                            clip,
                            BlockPos::new(x, y, z),
                            StructureState::new("minecraft:cobweb"),
                        );
                    }
                }
            }
        }
    }
    for section in 0..corridor.sections {
        let bay = 2 + section * 5;
        place_support(world, kind, piece, clip, bay, random);
        for (x, z, probability) in [
            (0, bay - 1, 0.1),
            (2, bay - 1, 0.1),
            (0, bay + 1, 0.1),
            (2, bay + 1, 0.1),
            (0, bay - 2, 0.05),
            (2, bay - 2, 0.05),
            (0, bay + 2, 0.05),
            (2, bay + 2, 0.05),
        ] {
            maybe_cobweb(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(x, 2, z),
                probability,
                random,
            );
        }
        if bounded(random, 100) == 0 {
            create_chest_cart(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(2, 0, bay - 1),
                random,
                loot_seed,
            );
        }
        if bounded(random, 100) == 0 {
            create_chest_cart(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(0, 0, bay + 1),
                random,
                loot_seed,
            );
        }
        if corridor.spider_corridor && !corridor.has_placed_spider {
            let spider_z = bay - 1 + bounded(random, 3) as i32;
            let local = BlockPos::new(1, 0, spider_z);
            let position = piece.world_position(local);
            if clip.contains(position) && interior(world, piece, clip, local) {
                corridor.has_placed_spider = true;
                PieceWorld::set_state(world, position, StructureState::new("minecraft:spawner"), 2);
                if world.is_spawner_block_entity(position) {
                    world.configure_cave_spider_spawner(position);
                }
            }
        }
    }
    for x in 0..=2 {
        for z in 0..=length {
            set_floor_planks(world, kind, piece, clip, BlockPos::new(x, -1, z));
        }
    }
    for z in [2, length - 2]
        .into_iter()
        .take(if corridor.sections > 1 { 2 } else { 1 })
    {
        for x in [0, 2] {
            let local = BlockPos::new(x, -1, z);
            let position = piece.world_position(local);
            if clip.contains(position)
                && PieceWorld::state_at(world, position).block == kind.planks()
            {
                fill_pillar_or_chain(world, kind, position);
            }
        }
    }
    if corridor.has_rails {
        for z in 0..=length {
            let floor = piece.world_position(BlockPos::new(1, -1, z));
            if !clip.contains(floor) {
                continue;
            }
            let floor_state = PieceWorld::state_at(world, floor);
            if is_air(&floor_state) || !PieceWorld::solid_render(world, floor) {
                continue;
            }
            let local = BlockPos::new(1, 0, z);
            let probability = if interior(world, piece, clip, local) {
                0.7
            } else {
                0.9
            };
            if random.next_f32() < probability {
                place_local(world, kind, piece, clip, local, rail(true));
            }
        }
    }
    true
}

fn place_support(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    piece: OrientedPiece,
    clip: &BlockBox,
    z: i32,
    random: &mut impl GenerationRandom,
) {
    if (0..=2).any(|x| {
        let position = piece.world_position(BlockPos::new(x, 3, z));
        !clip.contains(position) || is_air(&PieceWorld::state_at(world, position))
    }) {
        return;
    }
    let mut west_fence = StructureState::new(kind.fence());
    west_fence.properties.insert("west".into(), "true".into());
    let mut east_fence = StructureState::new(kind.fence());
    east_fence.properties.insert("east".into(), "true".into());
    for y in 0..=1 {
        place_local(
            world,
            kind,
            piece,
            clip,
            BlockPos::new(0, y, z),
            west_fence.clone(),
        );
        place_local(
            world,
            kind,
            piece,
            clip,
            BlockPos::new(2, y, z),
            east_fence.clone(),
        );
    }
    if bounded(random, 4) == 0 {
        for x in [0, 2] {
            place_local(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(x, 2, z),
                StructureState::new(kind.planks()),
            );
        }
    } else {
        for x in 0..=2 {
            place_local(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(x, 2, z),
                StructureState::new(kind.planks()),
            );
        }
        if random.next_f32() < 0.05 {
            place_local(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(1, 2, z - 1),
                wall_torch("south"),
            );
        }
        if random.next_f32() < 0.05 {
            place_local(
                world,
                kind,
                piece,
                clip,
                BlockPos::new(1, 2, z + 1),
                wall_torch("north"),
            );
        }
    }
}

fn maybe_cobweb(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    piece: OrientedPiece,
    clip: &BlockBox,
    local: BlockPos,
    probability: f32,
    random: &mut impl GenerationRandom,
) {
    if !interior(world, piece, clip, local) || random.next_f32() >= probability {
        return;
    }
    let position = piece.world_position(local);
    let mut sturdy = 0;
    for (face, offset) in [
        (MineshaftFace::Up, (0, -1, 0)),
        (MineshaftFace::Down, (0, 1, 0)),
        (MineshaftFace::South, (0, 0, -1)),
        (MineshaftFace::North, (0, 0, 1)),
        (MineshaftFace::East, (-1, 0, 0)),
        (MineshaftFace::West, (1, 0, 0)),
    ] {
        let neighbor = BlockPos::new(
            position.x + offset.0,
            position.y + offset.1,
            position.z + offset.2,
        );
        if clip.contains(neighbor) && world.sturdy_face(neighbor, face) {
            sturdy += 1;
            if sturdy >= 2 {
                place_local(
                    world,
                    kind,
                    piece,
                    clip,
                    local,
                    StructureState::new("minecraft:cobweb"),
                );
                return;
            }
        }
    }
}

fn create_chest_cart(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    piece: OrientedPiece,
    clip: &BlockBox,
    local: BlockPos,
    random: &mut impl GenerationRandom,
    loot_seed: &mut impl FnMut() -> i64,
) -> bool {
    let position = piece.world_position(local);
    let below = BlockPos::new(position.x, position.y - 1, position.z);
    if !clip.contains(position)
        || !is_air(&PieceWorld::state_at(world, position))
        || is_air(&PieceWorld::state_at(world, below))
    {
        return false;
    }
    let north_south = random.next_bool();
    place_local(world, kind, piece, clip, local, rail(north_south));
    let entity_position = [
        f64::from(position.x) + 0.5,
        f64::from(position.y) + 0.5,
        f64::from(position.z) + 0.5,
    ];
    if !world.create_mineshaft_chest_cart(entity_position) {
        return true;
    }
    world.spawn_mineshaft_chest_cart(MineshaftChestCartSpawn {
        position: entity_position,
        rail_north_south: north_south,
        creation_reason_chunk_generation: true,
        loot_table: "minecraft:chests/abandoned_mineshaft",
        loot_seed: loot_seed(),
    });
    true
}

fn set_floor_planks(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    piece: OrientedPiece,
    clip: &BlockBox,
    local: BlockPos,
) {
    if !interior(world, piece, clip, local) {
        return;
    }
    let position = piece.world_position(local);
    if !world.sturdy_top(position) {
        PieceWorld::set_state(world, position, StructureState::new(kind.planks()), 2);
    }
}

fn fill_pillar_or_chain(world: &mut impl MineshaftWorld, kind: MineshaftType, origin: BlockPos) {
    let mut below_active = true;
    let mut above_active = true;
    let mut distance = 1;
    while below_active || above_active {
        if below_active {
            let position = BlockPos::new(origin.x, origin.y - distance, origin.z);
            let state = PieceWorld::state_at(world, position);
            let open =
                world.structure_replaceable(position, &state) && state.block != "minecraft:lava";
            if !open && world.sturdy_top(position) {
                for y in position.y + 1..origin.y {
                    PieceWorld::set_state(
                        world,
                        BlockPos::new(origin.x, y, origin.z),
                        StructureState::new(kind.log()),
                        2,
                    );
                }
                return;
            }
            below_active = distance <= 20 && open && position.y > world.minimum_y() + 1;
        }
        if above_active {
            let position = BlockPos::new(origin.x, origin.y + distance, origin.z);
            let state = PieceWorld::state_at(world, position);
            let open = world.structure_replaceable(position, &state);
            if !open && world.supports_center_down(position) && !world.falling_block(position) {
                PieceWorld::set_state(
                    world,
                    BlockPos::new(origin.x, origin.y + 1, origin.z),
                    StructureState::new(kind.fence()),
                    2,
                );
                for y in origin.y + 2..position.y {
                    PieceWorld::set_state(
                        world,
                        BlockPos::new(origin.x, y, origin.z),
                        StructureState::new("minecraft:chain"),
                        2,
                    );
                }
                return;
            }
            above_active = distance <= 50 && open && position.y < world.maximum_y();
        }
        distance += 1;
    }
}

fn place_local(
    world: &mut impl MineshaftWorld,
    kind: MineshaftType,
    piece: OrientedPiece,
    clip: &BlockBox,
    local: BlockPos,
    state: StructureState,
) -> bool {
    place_replacing(
        world,
        kind,
        piece.world_position(local),
        piece.transform_state(state),
        clip,
    )
}

fn cave_air() -> StructureState {
    StructureState::new("minecraft:cave_air")
}

fn rail(north_south: bool) -> StructureState {
    let mut state = StructureState::new("minecraft:rail");
    state.properties.insert(
        "shape".into(),
        if north_south {
            "north_south"
        } else {
            "east_west"
        }
        .into(),
    );
    state
}

fn wall_torch(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:wall_torch");
    state.properties.insert("facing".into(), facing.into());
    state
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive corridor bound"))
}
