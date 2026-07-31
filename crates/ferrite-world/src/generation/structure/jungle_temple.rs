//! Terrain-aligned jungle-temple geometry, masonry selection, traps, and loot latches.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{
    HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use crate::generation::structure::processor::StructureState;

pub trait JungleTempleWorld: PieceWorld {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32;
}

pub fn generate_jungle_temple_piece(
    chunk_minimum: BlockPos,
    random: &mut impl GenerationRandom,
) -> JungleTemplePiece {
    let orientation = HorizontalDirection::ALL
        [random.next_u32(NonZeroU32::new(4).expect("four temple orientations")) as usize];
    JungleTemplePiece::new(chunk_minimum, orientation)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JungleTemplePiece {
    pub piece: OrientedPiece,
    pub average_ground_height: i32,
    pub placed_trap_one: bool,
    pub placed_trap_two: bool,
    pub placed_main_chest: bool,
    pub placed_hidden_chest: bool,
}

impl JungleTemplePiece {
    pub fn new(chunk_minimum: BlockPos, orientation: HorizontalDirection) -> Self {
        let horizontal = match orientation {
            HorizontalDirection::North | HorizontalDirection::South => [12, 15],
            HorizontalDirection::East | HorizontalDirection::West => [15, 12],
        };
        Self {
            piece: OrientedPiece {
                bounds: BlockBox::new(
                    BlockPos::new(chunk_minimum.x, 64, chunk_minimum.z),
                    BlockPos::new(
                        chunk_minimum.x + horizontal[0] - 1,
                        73,
                        chunk_minimum.z + horizontal[1] - 1,
                    ),
                )
                .expect("positive jungle-temple dimensions"),
                orientation,
            },
            average_ground_height: -1,
            placed_trap_one: false,
            placed_trap_two: false,
            placed_main_chest: false,
            placed_hidden_chest: false,
        }
    }

    pub fn place<R, F>(
        &mut self,
        world: &mut impl JungleTempleWorld,
        clip: &BlockBox,
        random: &mut R,
        loot_seed: &mut F,
    ) -> bool
    where
        R: GenerationRandom,
        F: FnMut() -> i64,
    {
        if self.average_ground_height < 0 && !self.align_to_terrain(world, clip) {
            return false;
        }
        let placement = PiecePlacement {
            piece: self.piece,
            clip,
        };
        place_masonry(world, placement, random);
        place_air(world, placement);
        place_stairs(world, placement);
        place_trap_one(world, placement);
        if !self.placed_trap_one {
            self.placed_trap_one = create_dispenser(
                world,
                placement,
                BlockPos::new(3, -2, 1),
                "north",
                "minecraft:chests/jungle_temple_dispenser",
                loot_seed,
            );
        }
        placement.place_block(world, BlockPos::new(3, -2, 2), vine("south"));
        place_trap_two(world, placement);
        if !self.placed_trap_two {
            self.placed_trap_two = create_dispenser(
                world,
                placement,
                BlockPos::new(9, -2, 3),
                "west",
                "minecraft:chests/jungle_temple_dispenser",
                loot_seed,
            );
        }
        placement.place_block(world, BlockPos::new(8, -1, 3), vine("east"));
        placement.place_block(world, BlockPos::new(8, -2, 3), vine("east"));
        if !self.placed_main_chest {
            self.placed_main_chest = placement.create_chest(
                world,
                BlockPos::new(8, -3, 3),
                "minecraft:chests/jungle_temple",
                &mut *loot_seed,
            );
        }
        place_hidden_mechanism(world, placement);
        if !self.placed_hidden_chest {
            self.placed_hidden_chest = placement.create_chest(
                world,
                BlockPos::new(9, -3, 10),
                "minecraft:chests/jungle_temple",
                loot_seed,
            );
        }
        true
    }

    fn align_to_terrain(&mut self, world: &mut impl JungleTempleWorld, clip: &BlockBox) -> bool {
        let mut sum = 0_i64;
        let mut count = 0_i64;
        for z in self.piece.bounds.minimum.z..=self.piece.bounds.maximum.z {
            for x in self.piece.bounds.minimum.x..=self.piece.bounds.maximum.x {
                if clip.contains(BlockPos::new(x, 64, z)) {
                    sum += i64::from(world.motion_blocking_no_leaves_height(x, z));
                    count += 1;
                }
            }
        }
        if count == 0 {
            return false;
        }
        let average = i32::try_from(sum / count).expect("mean of i32 heights fits i32");
        self.average_ground_height = average;
        self.piece.bounds =
            self.piece
                .bounds
                .moved([0, average.wrapping_sub(self.piece.bounds.minimum.y), 0]);
        true
    }
}

pub fn jungle_temple_start_allowed(
    world_surface_height: &mut impl FnMut(i32, i32) -> i32,
    chunk_minimum: BlockPos,
    sea_level: i32,
) -> bool {
    [
        (chunk_minimum.x, chunk_minimum.z),
        (chunk_minimum.x, chunk_minimum.z + 15),
        (chunk_minimum.x + 12, chunk_minimum.z),
        (chunk_minimum.x + 12, chunk_minimum.z + 15),
    ]
    .map(|(x, z)| world_surface_height(x, z))
    .into_iter()
    .min()
    .is_some_and(|height| height >= sea_level)
}

fn place_masonry(
    world: &mut impl PieceWorld,
    placement: PiecePlacement<'_>,
    random: &mut impl GenerationRandom,
) {
    for (minimum, maximum) in [
        ((0, -4, 0), (11, 0, 14)),
        ((2, 1, 2), (9, 2, 2)),
        ((2, 1, 12), (9, 2, 12)),
        ((2, 1, 3), (2, 2, 11)),
        ((9, 1, 3), (9, 2, 11)),
        ((1, 3, 1), (10, 6, 1)),
        ((1, 3, 13), (10, 6, 13)),
        ((1, 3, 2), (1, 6, 12)),
        ((10, 3, 2), (10, 6, 12)),
        ((2, 3, 2), (9, 3, 12)),
        ((2, 6, 2), (9, 6, 12)),
        ((3, 7, 3), (8, 7, 11)),
        ((4, 8, 4), (7, 8, 10)),
    ] {
        selector_box(world, placement, minimum, maximum, random);
    }
    for z in [0, 14] {
        for x in [2, 4, 7, 9] {
            selector_box(world, placement, (x, 4, z), (x, 5, z), random);
        }
    }
    selector_box(world, placement, (5, 6, 0), (6, 6, 0), random);
    for x in [0, 11] {
        for z in (2..=12).step_by(2) {
            selector_box(world, placement, (x, 4, z), (x, 5, z), random);
        }
        for z in [5, 9] {
            selector_box(world, placement, (x, 6, z), (x, 6, z), random);
        }
    }
    for x in [2, 9] {
        for z in [2, 12] {
            selector_box(world, placement, (x, 7, z), (x, 9, z), random);
        }
    }
    for x in [4, 7] {
        for z in [4, 10] {
            selector_box(world, placement, (x, 9, z), (x, 9, z), random);
        }
    }
    selector_box(world, placement, (5, 9, 7), (6, 9, 7), random);
    for position in [(4, 1, 9), (7, 1, 9)] {
        selector_box(world, placement, position, position, random);
    }
    selector_box(world, placement, (4, 1, 10), (7, 2, 10), random);
    selector_box(world, placement, (5, 4, 5), (6, 4, 5), random);
    for z in (1..=13).step_by(2) {
        selector_box(world, placement, (1, -3, z), (1, -2, z), random);
    }
    for z in (2..=12).step_by(2) {
        selector_box(world, placement, (1, -1, z), (3, -1, z), random);
    }
    for (minimum, maximum) in [
        ((2, -2, 1), (5, -2, 1)),
        ((7, -2, 1), (9, -2, 1)),
        ((6, -3, 1), (6, -3, 1)),
        ((6, -1, 1), (6, -1, 1)),
        ((9, -1, 1), (9, -1, 5)),
        ((8, -3, 8), (8, -3, 10)),
        ((10, -3, 8), (10, -3, 10)),
    ] {
        selector_box(world, placement, minimum, maximum, random);
    }
}

fn selector_box(
    world: &mut impl PieceWorld,
    placement: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    random: &mut impl GenerationRandom,
) {
    placement.fill_box(
        world,
        tuple_position(minimum),
        tuple_position(maximum),
        false,
        |_, _| {
            let block = if random.next_f32() < 0.4 {
                "minecraft:cobblestone"
            } else {
                "minecraft:mossy_cobblestone"
            };
            StructureState::new(block)
        },
    );
}

fn place_air(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    for (minimum, maximum) in [
        ((3, 1, 3), (8, 2, 11)),
        ((4, 3, 6), (7, 3, 9)),
        ((2, 4, 2), (9, 5, 12)),
        ((4, 6, 5), (7, 6, 9)),
        ((5, 7, 6), (6, 7, 8)),
        ((5, 1, 2), (6, 2, 2)),
        ((5, 2, 12), (6, 2, 12)),
        ((5, 5, 1), (6, 5, 1)),
        ((5, 5, 13), (6, 5, 13)),
        ((1, -3, 12), (10, -1, 13)),
        ((1, -3, 1), (3, -1, 13)),
        ((1, -3, 1), (9, -1, 5)),
        ((8, -3, 8), (10, -1, 10)),
    ] {
        direct_box(world, placement, minimum, maximum, "minecraft:air");
    }
    for position in [(1, 5, 5), (10, 5, 5), (1, 5, 9), (10, 5, 9)] {
        placement.place_block(
            world,
            tuple_position(position),
            StructureState::new("minecraft:air"),
        );
    }
    for offset in 0..=3 {
        direct_box(
            world,
            placement,
            (5, -offset, 7 + offset),
            (6, -offset, 9 + offset),
            "minecraft:air",
        );
    }
}

fn place_stairs(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    for (position, facing) in [
        ((5, 9, 6), "north"),
        ((6, 9, 6), "north"),
        ((5, 9, 8), "south"),
        ((6, 9, 8), "south"),
        ((4, 4, 5), "east"),
        ((7, 4, 5), "west"),
    ] {
        placement.place_block(world, tuple_position(position), stair(facing));
    }
    for x in 4..=7 {
        placement.place_block(world, BlockPos::new(x, 0, 0), stair("north"));
    }
    for x in [4, 7] {
        for (y, z) in [(1, 8), (2, 9), (3, 10)] {
            placement.place_block(world, BlockPos::new(x, y, z), stair("north"));
        }
    }
    for offset in 0..=3 {
        for x in [5, 6] {
            placement.place_block(world, BlockPos::new(x, -offset, 6 + offset), stair("south"));
        }
    }
}

fn place_trap_one(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    placement.place_block(world, BlockPos::new(1, -3, 8), tripwire_hook("east"));
    placement.place_block(world, BlockPos::new(4, -3, 8), tripwire_hook("west"));
    for x in 2..=3 {
        placement.place_block(world, BlockPos::new(x, -3, 8), tripwire(true, false));
    }
    for z in (2..=7).rev() {
        placement.place_block(world, BlockPos::new(5, -3, z), redstone("north", "south"));
    }
    placement.place_block(world, BlockPos::new(5, -3, 1), redstone("north", "west"));
    placement.place_block(world, BlockPos::new(4, -3, 1), redstone("east", "west"));
    placement.place_block(
        world,
        BlockPos::new(3, -3, 1),
        StructureState::new("minecraft:mossy_cobblestone"),
    );
}

fn place_trap_two(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    placement.place_block(world, BlockPos::new(7, -3, 1), tripwire_hook("south"));
    placement.place_block(world, BlockPos::new(7, -3, 5), tripwire_hook("north"));
    for z in 2..=4 {
        placement.place_block(world, BlockPos::new(7, -3, z), tripwire(false, true));
    }
    placement.place_block(world, BlockPos::new(8, -3, 6), redstone("east", "west"));
    placement.place_block(world, BlockPos::new(9, -3, 6), redstone("west", "south"));
    let mut rising_wire = redstone("north", "south");
    rising_wire.properties.insert("south".into(), "up".into());
    placement.place_block(world, BlockPos::new(9, -3, 5), rising_wire);
    placement.place_block(
        world,
        BlockPos::new(9, -3, 4),
        StructureState::new("minecraft:mossy_cobblestone"),
    );
    placement.place_block(world, BlockPos::new(9, -2, 4), redstone("north", "south"));
}

fn place_hidden_mechanism(world: &mut impl PieceWorld, placement: PiecePlacement<'_>) {
    for position in [
        (9, -3, 2),
        (8, -3, 1),
        (4, -3, 5),
        (5, -2, 5),
        (5, -1, 5),
        (6, -3, 5),
        (7, -2, 5),
        (7, -1, 5),
        (8, -3, 5),
    ] {
        placement.place_block(
            world,
            tuple_position(position),
            StructureState::new("minecraft:mossy_cobblestone"),
        );
    }
    for x in 8..=10 {
        placement.place_block(
            world,
            BlockPos::new(x, -2, 11),
            StructureState::new("minecraft:chiseled_stone_bricks"),
        );
        placement.place_block(world, BlockPos::new(x, -2, 12), lever("north"));
    }
    placement.place_block(
        world,
        BlockPos::new(10, -2, 9),
        StructureState::new("minecraft:mossy_cobblestone"),
    );
    placement.place_block(world, BlockPos::new(8, -2, 9), redstone("north", "south"));
    placement.place_block(world, BlockPos::new(8, -2, 10), redstone("north", "south"));
    let mut all_wire = StructureState::new("minecraft:redstone_wire");
    for direction in ["north", "east", "south", "west"] {
        all_wire.properties.insert(direction.into(), "side".into());
    }
    placement.place_block(world, BlockPos::new(10, -1, 9), all_wire);
    placement.place_block(world, BlockPos::new(9, -2, 8), piston("up"));
    placement.place_block(world, BlockPos::new(10, -2, 8), piston("west"));
    placement.place_block(world, BlockPos::new(10, -1, 8), piston("west"));
    let mut repeater = StructureState::new("minecraft:repeater");
    repeater.properties.insert("facing".into(), "north".into());
    placement.place_block(world, BlockPos::new(10, -2, 10), repeater);
}

fn create_dispenser(
    world: &mut impl PieceWorld,
    placement: PiecePlacement<'_>,
    local: BlockPos,
    facing: &str,
    table: &str,
    loot_seed: &mut impl FnMut() -> i64,
) -> bool {
    let position = placement.piece.world_position(local);
    if !placement.clip.contains(position)
        || PieceWorld::state_at(world, position).block == "minecraft:dispenser"
    {
        return false;
    }
    let mut dispenser = StructureState::new("minecraft:dispenser");
    dispenser.properties.insert("facing".into(), facing.into());
    placement.place_block(world, local, dispenser);
    if PieceWorld::is_loot_container(world, position) {
        PieceWorld::install_loot(world, position, table, loot_seed());
    }
    true
}

fn direct_box(
    world: &mut impl PieceWorld,
    placement: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    block: &str,
) {
    placement.fill_box(
        world,
        tuple_position(minimum),
        tuple_position(maximum),
        false,
        |_, _| StructureState::new(block),
    );
}

fn stair(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:cobblestone_stairs");
    state.properties.insert("facing".into(), facing.into());
    state
}

fn tripwire_hook(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:tripwire_hook");
    state.properties.insert("facing".into(), facing.into());
    state.properties.insert("attached".into(), "true".into());
    state
}

fn tripwire(east_west: bool, north_south: bool) -> StructureState {
    let mut state = StructureState::new("minecraft:tripwire");
    state.properties.insert("attached".into(), "true".into());
    if east_west {
        state.properties.insert("east".into(), "true".into());
        state.properties.insert("west".into(), "true".into());
    }
    if north_south {
        state.properties.insert("north".into(), "true".into());
        state.properties.insert("south".into(), "true".into());
    }
    state
}

fn redstone(first: &str, second: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:redstone_wire");
    state.properties.insert(first.into(), "side".into());
    state.properties.insert(second.into(), "side".into());
    state
}

fn vine(face: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:vine");
    state.properties.insert(face.into(), "true".into());
    state
}

fn lever(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:lever");
    state.properties.insert("face".into(), "wall".into());
    state.properties.insert("facing".into(), facing.into());
    state
}

fn piston(facing: &str) -> StructureState {
    let mut state = StructureState::new("minecraft:sticky_piston");
    state.properties.insert("facing".into(), facing.into());
    state
}

fn tuple_position((x, y, z): (i32, i32, i32)) -> BlockPos {
    BlockPos::new(x, y, z)
}
