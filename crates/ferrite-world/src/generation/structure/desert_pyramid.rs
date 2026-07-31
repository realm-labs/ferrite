//! Desert-pyramid terrain alignment, fixed geometry, chests, and archaeology candidates.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{
    HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use crate::generation::structure::processor::StructureState;

pub trait DesertPyramidWorld: PieceWorld {
    fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32;

    fn minimum_y(&self) -> i32;

    fn positional_seed(&self, position: BlockPos) -> i64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesertPyramidPiece {
    pub piece: OrientedPiece,
    pub height_position: i32,
    pub placed_chests: [bool; 4],
    pub archaeology_candidates: Vec<BlockPos>,
    pub collapsed_roof_position: BlockPos,
}

pub fn generate_desert_pyramid_piece(
    chunk_minimum: BlockPos,
    random: &mut impl GenerationRandom,
) -> DesertPyramidPiece {
    let orientation = HorizontalDirection::ALL
        [random.next_u32(NonZeroU32::new(4).expect("four pyramid orientations")) as usize];
    DesertPyramidPiece::new(chunk_minimum, orientation)
}

impl DesertPyramidPiece {
    pub fn new(chunk_minimum: BlockPos, orientation: HorizontalDirection) -> Self {
        Self {
            piece: OrientedPiece {
                bounds: BlockBox::new(
                    BlockPos::new(chunk_minimum.x, 64, chunk_minimum.z),
                    BlockPos::new(chunk_minimum.x + 20, 78, chunk_minimum.z + 20),
                )
                .expect("positive desert-pyramid dimensions"),
                orientation,
            },
            height_position: -1,
            placed_chests: [false; 4],
            archaeology_candidates: Vec::new(),
            collapsed_roof_position: BlockPos::new(0, 0, 0),
        }
    }

    pub fn place<R, L, F>(
        &mut self,
        world: &mut impl DesertPyramidWorld,
        clip: &BlockBox,
        caller_random: &mut R,
        level_random: &mut L,
        loot_seed: &mut F,
    ) -> bool
    where
        R: GenerationRandom,
        L: GenerationRandom,
        F: FnMut() -> i64,
    {
        let offset =
            -(caller_random.next_u32(NonZeroU32::new(3).expect("three pyramid offsets")) as i32);
        if self.height_position < 0 {
            self.align_to_lowest_ground(world, offset);
        }
        let placement = PiecePlacement {
            piece: self.piece,
            clip,
        };
        place_pyramid(world, placement);
        place_trap(world, placement);
        for (index, local) in [
            BlockPos::new(10, -11, 8),
            BlockPos::new(12, -11, 10),
            BlockPos::new(10, -11, 12),
            BlockPos::new(8, -11, 10),
        ]
        .into_iter()
        .enumerate()
        {
            if !self.placed_chests[index] {
                self.placed_chests[index] = placement.create_chest(
                    world,
                    local,
                    "minecraft:chests/desert_pyramid",
                    &mut *loot_seed,
                );
            }
        }
        self.place_cellar(world, placement, level_random);
        true
    }

    fn align_to_lowest_ground(&mut self, world: &mut impl DesertPyramidWorld, offset: i32) {
        let mut minimum = i32::MAX;
        for z in self.piece.bounds.minimum.z..=self.piece.bounds.maximum.z {
            for x in self.piece.bounds.minimum.x..=self.piece.bounds.maximum.x {
                minimum = minimum.min(world.motion_blocking_no_leaves_height(x, z));
            }
        }
        self.height_position = minimum;
        self.piece.bounds = self.piece.bounds.moved([
            0,
            minimum
                .wrapping_add(offset)
                .wrapping_sub(self.piece.bounds.minimum.y),
            0,
        ]);
    }

    fn place_cellar(
        &mut self,
        world: &mut impl DesertPyramidWorld,
        placement: PiecePlacement<'_>,
        level_random: &mut impl GenerationRandom,
    ) {
        for (x, y) in [(13, -1), (14, -2), (15, -3)] {
            placement.place_block(world, BlockPos::new(x, y, 17), stair("east"));
        }
        let variant = level_random.next_bool();
        for x in 12..=16 {
            placement.place_block(world, BlockPos::new(x, 0, 17), state("sand"));
        }
        placement.place_block(world, BlockPos::new(14, -1, 17), state("sand"));
        placement.place_block(
            world,
            BlockPos::new(15, -1, 17),
            state(if variant { "sand" } else { "sandstone" }),
        );
        placement.place_block(
            world,
            BlockPos::new(16, -1, 17),
            state(if variant { "sandstone" } else { "sand" }),
        );
        placement.place_block(world, BlockPos::new(15, -2, 17), state("sand"));
        placement.place_block(world, BlockPos::new(16, -2, 17), state("sandstone"));
        placement.place_block(world, BlockPos::new(16, -3, 17), state("sand"));

        for (y, block) in [
            (-3, "cut_sandstone"),
            (-2, "chiseled_sandstone"),
            (-1, "cut_sandstone"),
        ] {
            direct_box(world, placement, (13, y, 10), (13, y, 15), block, true);
            direct_box(world, placement, (19, y, 10), (19, y, 15), block, true);
            direct_box(world, placement, (13, y, 10), (19, y, 11), block, true);
            direct_box(world, placement, (13, y, 16), (19, y, 16), block, true);
        }
        for y in -3..=-1 {
            for x in 14..=18 {
                for z in 11..=15 {
                    self.archaeology_candidates
                        .push(placement.piece.world_position(BlockPos::new(x, y, z)));
                }
            }
        }
        for x in 14..=18 {
            for z in 11..=15 {
                let block = if level_random.next_f32() < 0.33 {
                    "sandstone"
                } else {
                    "sand"
                };
                placement.place_block(world, BlockPos::new(x, 0, z), state(block));
            }
        }
        let seed_position = placement.piece.world_position(BlockPos::new(14, 0, 11));
        let mut roof_random = LegacyRandom::new(world.positional_seed(seed_position));
        let roof_x =
            14 + roof_random.next_u32(NonZeroU32::new(5).expect("five roof X cells")) as i32;
        let roof_z =
            11 + roof_random.next_u32(NonZeroU32::new(5).expect("five roof Z cells")) as i32;
        self.collapsed_roof_position = placement
            .piece
            .world_position(BlockPos::new(roof_x, 0, roof_z));
        place_cellar_floor(world, placement, &mut self.archaeology_candidates);
    }
}

pub fn desert_pyramid_start_allowed(
    world_surface_height: &mut impl FnMut(i32, i32) -> i32,
    chunk_minimum: BlockPos,
    sea_level: i32,
) -> bool {
    [
        (chunk_minimum.x, chunk_minimum.z),
        (chunk_minimum.x, chunk_minimum.z + 21),
        (chunk_minimum.x + 21, chunk_minimum.z),
        (chunk_minimum.x + 21, chunk_minimum.z + 21),
    ]
    .map(|(x, z)| world_surface_height(x, z))
    .into_iter()
    .min()
    .is_some_and(|height| height >= sea_level)
}

fn place_pyramid(world: &mut impl DesertPyramidWorld, placement: PiecePlacement<'_>) {
    direct_box(
        world,
        placement,
        (0, -4, 0),
        (20, 0, 20),
        "sandstone",
        false,
    );
    for inset in 1..=9 {
        direct_box(
            world,
            placement,
            (inset, inset, inset),
            (20 - inset, inset, 20 - inset),
            "sandstone",
            false,
        );
        direct_box(
            world,
            placement,
            (inset + 1, inset, inset + 1),
            (19 - inset, inset, 19 - inset),
            "air",
            false,
        );
    }
    for x in 0..=20 {
        for z in 0..=20 {
            placement.fill_column_down(
                world,
                BlockPos::new(x, -5, z),
                state("sandstone"),
                world.minimum_y(),
                |live, fluid| {
                    !fluid.is_empty()
                        || matches!(
                            live.block.as_str(),
                            "minecraft:air"
                                | "minecraft:glow_lichen"
                                | "minecraft:seagrass"
                                | "minecraft:tall_seagrass"
                        )
                },
            );
        }
    }
    place_upper_structure(world, placement);
}

fn place_upper_structure(world: &mut impl PieceWorld, p: PiecePlacement<'_>) {
    shell_box(world, p, (0, 0, 0), (4, 9, 4), "sandstone", "air");
    direct_box(world, p, (1, 10, 1), (3, 10, 3), "sandstone", false);
    for (position, facing) in [
        ((2, 10, 0), "north"),
        ((2, 10, 4), "south"),
        ((0, 10, 2), "east"),
        ((4, 10, 2), "west"),
    ] {
        p.place_block(world, tuple(position), stair(facing));
    }
    shell_box(world, p, (16, 0, 0), (20, 9, 4), "sandstone", "air");
    direct_box(world, p, (17, 10, 1), (19, 10, 3), "sandstone", false);
    for (position, facing) in [
        ((18, 10, 0), "north"),
        ((18, 10, 4), "south"),
        ((16, 10, 2), "east"),
        ((20, 10, 2), "west"),
    ] {
        p.place_block(world, tuple(position), stair(facing));
    }
    shell_box(world, p, (8, 0, 0), (12, 4, 4), "sandstone", "air");
    direct_box(world, p, (9, 1, 0), (11, 3, 4), "air", false);
    for position in [
        (9, 1, 1),
        (9, 2, 1),
        (9, 3, 1),
        (10, 3, 1),
        (11, 3, 1),
        (11, 2, 1),
        (11, 1, 1),
    ] {
        p.place_block(world, tuple(position), state("cut_sandstone"));
    }
    shell_box(world, p, (4, 1, 1), (8, 3, 3), "sandstone", "air");
    direct_box(world, p, (4, 1, 2), (8, 2, 2), "air", false);
    shell_box(world, p, (12, 1, 1), (16, 3, 3), "sandstone", "air");
    direct_box(world, p, (12, 1, 2), (16, 2, 2), "air", false);
    direct_box(world, p, (5, 4, 5), (15, 4, 15), "sandstone", false);
    direct_box(world, p, (9, 4, 9), (11, 4, 11), "air", false);
    for (x, z) in [(8, 8), (12, 8), (8, 12), (12, 12)] {
        direct_box(world, p, (x, 1, z), (x, 3, z), "cut_sandstone", false);
    }
    direct_box(world, p, (1, 1, 5), (4, 4, 11), "sandstone", false);
    direct_box(world, p, (16, 1, 5), (19, 4, 11), "sandstone", false);
    direct_box(world, p, (6, 7, 9), (6, 7, 11), "sandstone", false);
    direct_box(world, p, (14, 7, 9), (14, 7, 11), "sandstone", false);
    direct_box(world, p, (5, 5, 9), (5, 7, 11), "cut_sandstone", false);
    direct_box(world, p, (15, 5, 9), (15, 7, 11), "cut_sandstone", false);
    for position in [
        (5, 5, 10),
        (5, 6, 10),
        (6, 6, 10),
        (15, 5, 10),
        (15, 6, 10),
        (14, 6, 10),
    ] {
        p.place_block(world, tuple(position), state("air"));
    }
    direct_box(world, p, (2, 4, 4), (2, 6, 4), "air", false);
    direct_box(world, p, (18, 4, 4), (18, 6, 4), "air", false);
    for position in [(2, 4, 5), (2, 3, 4), (18, 4, 5), (18, 3, 4)] {
        p.place_block(world, tuple(position), stair("north"));
    }
    direct_box(world, p, (1, 1, 3), (2, 2, 3), "sandstone", false);
    direct_box(world, p, (18, 1, 3), (19, 2, 3), "sandstone", false);
    for position in [(1, 1, 2), (19, 1, 2)] {
        p.place_block(world, tuple(position), state("sandstone"));
    }
    for position in [(1, 2, 2), (19, 2, 2)] {
        p.place_block(world, tuple(position), state("sandstone_slab"));
    }
    p.place_block(world, BlockPos::new(2, 1, 2), stair("west"));
    p.place_block(world, BlockPos::new(18, 1, 2), stair("east"));
    direct_box(world, p, (4, 3, 5), (4, 3, 17), "sandstone", false);
    direct_box(world, p, (16, 3, 5), (16, 3, 17), "sandstone", false);
    direct_box(world, p, (3, 1, 5), (4, 2, 16), "air", false);
    direct_box(world, p, (15, 1, 5), (16, 2, 16), "air", false);
    for z in (5..=17).step_by(2) {
        for x in [4, 16] {
            p.place_block(world, BlockPos::new(x, 1, z), state("cut_sandstone"));
            p.place_block(world, BlockPos::new(x, 2, z), state("chiseled_sandstone"));
        }
    }
    for (x, z) in [
        (10, 7),
        (10, 8),
        (9, 9),
        (11, 9),
        (8, 10),
        (12, 10),
        (7, 10),
        (13, 10),
        (9, 11),
        (11, 11),
        (10, 12),
        (10, 13),
    ] {
        p.place_block(world, BlockPos::new(x, 0, z), state("orange_terracotta"));
    }
    p.place_block(world, BlockPos::new(10, 0, 10), state("blue_terracotta"));
    place_facades(world, p);
}

fn place_facades(world: &mut impl PieceWorld, p: PiecePlacement<'_>) {
    let rows = [
        ["cut_sandstone", "orange_terracotta", "cut_sandstone"],
        ["cut_sandstone", "orange_terracotta", "cut_sandstone"],
        [
            "orange_terracotta",
            "chiseled_sandstone",
            "orange_terracotta",
        ],
        ["cut_sandstone", "orange_terracotta", "cut_sandstone"],
        [
            "orange_terracotta",
            "chiseled_sandstone",
            "orange_terracotta",
        ],
        [
            "orange_terracotta",
            "orange_terracotta",
            "orange_terracotta",
        ],
        ["cut_sandstone", "cut_sandstone", "cut_sandstone"],
    ];
    for x in [0, 20] {
        for (dy, row) in rows.iter().enumerate() {
            for (dz, block) in row.iter().enumerate() {
                p.place_block(
                    world,
                    BlockPos::new(x, dy as i32 + 2, dz as i32 + 1),
                    state(block),
                );
            }
        }
    }
    for center in [2, 18] {
        for (dy, row) in rows.iter().enumerate() {
            for (dx, block) in row.iter().enumerate() {
                p.place_block(
                    world,
                    BlockPos::new(center + dx as i32 - 1, dy as i32 + 2, 0),
                    state(block),
                );
            }
        }
    }
    direct_box(world, p, (8, 4, 0), (12, 6, 0), "cut_sandstone", false);
    for position in [(8, 6, 0), (12, 6, 0)] {
        p.place_block(world, tuple(position), state("air"));
    }
    for position in [(9, 5, 0), (11, 5, 0)] {
        p.place_block(world, tuple(position), state("orange_terracotta"));
    }
    p.place_block(world, BlockPos::new(10, 5, 0), state("chiseled_sandstone"));
}

fn place_trap(world: &mut impl PieceWorld, p: PiecePlacement<'_>) {
    direct_box(world, p, (8, -14, 8), (12, -11, 12), "cut_sandstone", false);
    direct_box(
        world,
        p,
        (8, -10, 8),
        (12, -10, 12),
        "chiseled_sandstone",
        false,
    );
    direct_box(world, p, (8, -9, 8), (12, -9, 12), "cut_sandstone", false);
    direct_box(world, p, (8, -8, 8), (12, -1, 12), "sandstone", false);
    direct_box(world, p, (9, -11, 9), (11, -1, 11), "air", false);
    p.place_block(
        world,
        BlockPos::new(10, -11, 10),
        state("stone_pressure_plate"),
    );
    direct_box(world, p, (9, -13, 9), (11, -13, 11), "tnt", false);
    for (opening, outward) in [
        ((8, -11, 10), (7, -11, 10)),
        ((12, -11, 10), (13, -11, 10)),
        ((10, -11, 8), (10, -11, 7)),
        ((10, -11, 12), (10, -11, 13)),
    ] {
        p.place_block(world, tuple(opening), state("air"));
        p.place_block(
            world,
            BlockPos::new(opening.0, opening.1 + 1, opening.2),
            state("air"),
        );
        p.place_block(
            world,
            BlockPos::new(outward.0, outward.1 + 1, outward.2),
            state("chiseled_sandstone"),
        );
        p.place_block(world, tuple(outward), state("cut_sandstone"));
    }
}

fn place_cellar_floor(
    world: &mut impl PieceWorld,
    p: PiecePlacement<'_>,
    candidates: &mut Vec<BlockPos>,
) {
    p.place_block(world, BlockPos::new(16, -4, 13), state("blue_terracotta"));
    for (x, z) in [
        (17, 12),
        (17, 14),
        (15, 12),
        (15, 14),
        (18, 13),
        (14, 13),
        (16, 15),
        (16, 11),
        (19, 13),
        (13, 13),
        (16, 16),
        (16, 10),
    ] {
        p.place_block(world, BlockPos::new(x, -4, z), state("orange_terracotta"));
    }
    for position in [
        (19, -3, 13),
        (19, -2, 13),
        (13, -3, 13),
        (13, -2, 13),
        (16, -3, 16),
        (16, -2, 16),
        (16, -3, 10),
        (16, -2, 10),
    ] {
        candidates.push(p.piece.world_position(tuple(position)));
    }
    for (position, block) in [
        ((20, -3, 13), "cut_sandstone"),
        ((20, -2, 13), "chiseled_sandstone"),
        ((12, -3, 13), "cut_sandstone"),
        ((12, -2, 13), "chiseled_sandstone"),
        ((16, -3, 9), "cut_sandstone"),
        ((16, -2, 9), "chiseled_sandstone"),
    ] {
        p.place_block(world, tuple(position), state(block));
    }
}

fn shell_box(
    world: &mut impl PieceWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    edge: &str,
    inside: &str,
) {
    p.fill_box(
        world,
        tuple(minimum),
        tuple(maximum),
        false,
        |_, is_edge| state(if is_edge { edge } else { inside }),
    );
}
fn direct_box(
    world: &mut impl PieceWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    block: &str,
    skip_air: bool,
) {
    p.fill_box(world, tuple(minimum), tuple(maximum), skip_air, |_, _| {
        state(block)
    });
}
fn state(block: &str) -> StructureState {
    StructureState::new(format!("minecraft:{block}"))
}
fn stair(facing: &str) -> StructureState {
    let mut s = state("sandstone_stairs");
    s.properties.insert("facing".into(), facing.into());
    s
}
fn tuple((x, y, z): (i32, i32, i32)) -> BlockPos {
    BlockPos::new(x, y, z)
}
