use std::cell::RefCell;
use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::terrain::{
    BLOCK_PILE_WRITE_FLAGS, BasaltPillarWorld, BlockFace, BlockPileWorld, DeltaFeatureConfig,
    DeltaFeatureWorld, DesertWellBlocks, DesertWellWorld, DiskWorld, ReplacementBlobWorld,
    SpikeWorld, UnderwaterMagmaConfig, UnderwaterMagmaWorld, place_basalt_pillar, place_block_pile,
    place_delta_feature, place_desert_well, place_disk, place_replacement_blob, place_spike,
    place_underwater_magma,
};
use ferrite_world::id::BlockStateId;

#[test]
fn block_pile_uses_x_fastest_then_y_then_z_and_provider_after_support() {
    let origin = BlockPos::new(10, 20, -10);
    let mut world = PileFixture::default();
    let floats = (0..50).flat_map(|_| [1.0, 0.0]);
    let mut random = ScriptedRandom::new([0, 0], floats);
    assert!(place_block_pile(&mut world, origin, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [2, 2]);
    assert_eq!(random.float_draws, 100);
    assert_eq!(world.offers.len(), 50);
    assert_eq!(
        &world.candidates[..7],
        [
            BlockPos::new(8, 20, -12),
            BlockPos::new(9, 20, -12),
            BlockPos::new(10, 20, -12),
            BlockPos::new(11, 20, -12),
            BlockPos::new(12, 20, -12),
            BlockPos::new(8, 21, -12),
            BlockPos::new(9, 21, -12),
        ]
    );
    assert!(
        world
            .trace
            .chunks_exact(4)
            .all(|events| { events == ["empty", "support", "provider", "offer"] })
    );
}

#[test]
fn disk_preserves_active_runs_across_null_provider_values() {
    let origin = BlockPos::new(4, 10, -2);
    let mut world = DiskFixture::default();
    let mut random = ScriptedRandom::new([], []);
    assert!(
        place_disk(
            &mut world,
            origin,
            &IntProvider::Constant(0),
            2,
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(
        world.offers,
        [
            BlockPos::new(4, 12, -2),
            BlockPos::new(4, 10, -2),
            BlockPos::new(4, 8, -2),
        ]
    );
    assert_eq!(
        world.postprocessed,
        [
            BlockPos::new(4, 13, -2),
            BlockPos::new(4, 14, -2),
            BlockPos::new(4, 9, -2),
            BlockPos::new(4, 10, -2),
        ]
    );
    assert!(random.bounds.is_empty());
    assert_eq!(random.float_draws, 0);
}

#[test]
fn basalt_pillar_disables_sides_then_uses_four_base_and_forty_nine_root_draws() {
    let origin = BlockPos::new(0, 10, 0);
    let basalt = BlockStateId::new(5);
    let integers = [vec![0; 8], vec![9; 49]].concat();
    let mut random = ScriptedRandom::new(integers, []);
    let mut world = BasaltFixture {
        origin,
        offers: Vec::new(),
    };
    assert!(place_basalt_pillar(&mut world, origin, basalt, &mut random, |_| true).unwrap());
    assert_eq!(&random.bounds[..4], [10; 4]);
    assert_eq!(&random.bounds[4..8], [2; 4]);
    assert_eq!(&random.bounds[8..], [10; 49]);
    assert_eq!(world.offers.len(), 14);
    assert_eq!(world.offers[0], origin);
    assert_eq!(
        world
            .offers
            .iter()
            .skip(1)
            .filter(|position| position.x == 0)
            .count(),
        7
    );
    assert_eq!(
        world
            .offers
            .iter()
            .skip(1)
            .filter(|position| position.z == 0)
            .count(),
        7
    );
}

#[test]
fn replacement_blob_searches_before_three_radius_samples() {
    let origin = BlockPos::new(2, 5, -3);
    let target = BlockStateId::new(7);
    let replacement = BlockStateId::new(8);
    let mut world = ReplacementFixture {
        target,
        reads: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom::new([0, 0, 0], []);
    assert!(
        place_replacement_blob(
            &mut world,
            origin,
            target,
            replacement,
            &IntProvider::Uniform {
                minimum: 0,
                maximum: 0,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(
        world.reads,
        [origin, BlockPos::new(2, 4, -3), BlockPos::new(2, 4, -3)]
    );
    assert_eq!(random.bounds, [1, 1, 1]);
    assert_eq!(world.offers, [(BlockPos::new(2, 4, -3), replacement, 3)]);
}

#[test]
fn underwater_magma_scans_both_edges_then_draws_once_for_the_floor_cube() {
    let origin = BlockPos::new(0, 10, 0);
    let magma = BlockStateId::new(3);
    let mut world = MagmaFixture {
        origin,
        reads: Vec::new(),
        faces: RefCell::new(Vec::new()),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom::new([], [0.4]);
    assert!(
        place_underwater_magma(
            &mut world,
            origin,
            UnderwaterMagmaConfig {
                floor_search_range: 2,
                placement_radius: 0,
                placement_probability: 0.5,
                magma,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(random.float_draws, 1);
    assert_eq!(
        world.reads,
        [
            origin,
            origin,
            BlockPos::new(0, 11, 0),
            origin,
            BlockPos::new(0, 9, 0),
            BlockPos::new(0, 9, 0),
            BlockPos::new(0, 8, 0),
            BlockPos::new(0, 9, -1),
            BlockPos::new(1, 9, 0),
            BlockPos::new(0, 9, 1),
            BlockPos::new(-1, 9, 0),
        ]
    );
    assert_eq!(
        *world.faces.borrow(),
        [
            (BlockPos::new(0, 8, 0), BlockFace::Up),
            (BlockPos::new(0, 9, -1), BlockFace::South),
            (BlockPos::new(1, 9, 0), BlockFace::West),
            (BlockPos::new(0, 9, 1), BlockFace::North),
            (BlockPos::new(-1, 9, 0), BlockFace::East),
        ]
    );
    assert_eq!(world.offers, [(BlockPos::new(0, 9, 0), magma, 2)]);
}

#[test]
fn delta_no_rim_branch_skips_provider_and_rechecks_the_same_contents_position() {
    let origin = BlockPos::new(3, 7, -4);
    let contents = BlockStateId::new(3);
    let mut world = DeltaFixture {
        reads: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = DeltaRandom {
        integers: [0, 0].into_iter().collect(),
        bounds: Vec::new(),
        double_draws: 0,
    };
    assert!(
        place_delta_feature(
            &mut world,
            origin,
            &DeltaFeatureConfig {
                contents,
                rim: BlockStateId::new(4),
                size: IntProvider::Uniform {
                    minimum: 0,
                    maximum: 0,
                },
                rim_size: IntProvider::Uniform {
                    minimum: 1,
                    maximum: 1,
                },
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    let clarity_reads = [
        origin,
        BlockPos::new(3, 6, -4),
        BlockPos::new(3, 8, -4),
        BlockPos::new(3, 7, -5),
        BlockPos::new(3, 7, -3),
        BlockPos::new(2, 7, -4),
        BlockPos::new(4, 7, -4),
    ];
    assert_eq!(world.reads, [clarity_reads, clarity_reads].concat());
    assert_eq!(random.double_draws, 1);
    assert_eq!(random.bounds, [1, 1]);
    assert_eq!(world.offers, [(origin, contents, 3)]);
}

#[test]
fn desert_well_writes_fixed_matrix_then_selects_two_archaeology_positions() {
    let origin = BlockPos::new(0, 10, 0);
    let center = BlockPos::new(0, 11, 0);
    let blocks = DesertWellBlocks {
        sandstone: BlockStateId::new(1),
        water: BlockStateId::new(2),
        sand: BlockStateId::new(3),
        sandstone_slab: BlockStateId::new(4),
        suspicious_sand: BlockStateId::new(5),
    };
    let mut world = DesertWellFixture {
        sand: blocks.sand,
        empty_reads: Vec::new(),
        state_reads: Vec::new(),
        offers: Vec::new(),
        loot: Vec::new(),
    };
    let mut random = ScriptedRandom::new([0, 4], []);
    assert!(place_desert_well(&mut world, origin, blocks, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [5, 5]);
    assert_eq!(world.empty_reads.len(), 26);
    assert_eq!(world.empty_reads[0], center);
    assert_eq!(world.state_reads, [center]);
    assert_eq!(world.offers.len(), 128);
    assert_eq!(
        &world.offers[..3],
        [
            (BlockPos::new(-2, 9, -2), blocks.sandstone, 2),
            (BlockPos::new(-2, 9, -1), blocks.sandstone, 2),
            (BlockPos::new(-2, 9, 0), blocks.sandstone, 2),
        ]
    );
    assert_eq!(
        &world.offers[75..80],
        [
            (center, blocks.water, 2),
            (BlockPos::new(0, 11, -1), blocks.water, 2),
            (BlockPos::new(1, 11, 0), blocks.water, 2),
            (BlockPos::new(0, 11, 1), blocks.water, 2),
            (BlockPos::new(-1, 11, 0), blocks.water, 2),
        ]
    );
    let first_archaeology = BlockPos::new(0, 10, 0);
    let second_archaeology = BlockPos::new(0, 9, -1);
    assert_eq!(
        &world.offers[126..],
        [
            (first_archaeology, blocks.suspicious_sand, 3),
            (second_archaeology, blocks.suspicious_sand, 3),
        ]
    );
    assert_eq!(
        world.loot,
        [
            (first_archaeology, packed_block_position(first_archaeology)),
            (
                second_archaeology,
                packed_block_position(second_archaeology)
            ),
        ]
    );
}

#[test]
fn spike_radius_one_draws_only_admitted_perimeter_cells_then_roots_to_y_fifty() {
    let origin = BlockPos::new(0, 60, 0);
    let spike = BlockStateId::new(7);
    let mut world = SpikeFixture {
        empty_reads: Vec::new(),
        state_reads: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom::new([0, 0, 0], [0.0; 8]);
    assert!(place_spike(&mut world, origin, spike, &mut random, |_| true).unwrap());
    assert_eq!(random.bounds, [4, 4, 2]);
    assert_eq!(random.float_draws, 8);
    assert_eq!(
        &world.offers[..5],
        [
            (BlockPos::new(-1, 60, 0), spike, 3),
            (BlockPos::new(0, 60, -1), spike, 3),
            (origin, spike, 3),
            (BlockPos::new(0, 60, 1), spike, 3),
            (BlockPos::new(1, 60, 0), spike, 3),
        ]
    );
    assert_eq!(world.offers.len(), 24);
    assert_eq!(
        &world.offers[15..],
        [
            (BlockPos::new(0, 59, 0), spike, 3),
            (BlockPos::new(0, 58, 0), spike, 3),
            (BlockPos::new(0, 57, 0), spike, 3),
            (BlockPos::new(0, 56, 0), spike, 3),
            (BlockPos::new(0, 55, 0), spike, 3),
            (BlockPos::new(0, 54, 0), spike, 3),
            (BlockPos::new(0, 53, 0), spike, 3),
            (BlockPos::new(0, 52, 0), spike, 3),
            (BlockPos::new(0, 51, 0), spike, 3),
        ]
    );
}

#[derive(Debug, Default)]
struct PileFixture {
    candidates: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    trace: Vec<&'static str>,
}

impl BlockPileWorld<ScriptedRandom> for PileFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.candidates.push(position);
        self.trace.push("empty");
        true
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        self.trace.push("support");
        BlockStateId::new(1)
    }

    fn is_dirt_path(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_sturdy_up(&self, _state: BlockStateId) -> bool {
        true
    }

    fn provide_pile_state(
        &mut self,
        _position: BlockPos,
        _random: &mut ScriptedRandom,
    ) -> BlockStateId {
        self.trace.push("provider");
        BlockStateId::new(9)
    }

    fn offer_pile_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        assert_eq!(flags, BLOCK_PILE_WRITE_FLAGS);
        self.trace.push("offer");
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug, Default)]
struct DiskFixture {
    offers: Vec<BlockPos>,
    postprocessed: Vec<BlockPos>,
}

impl DiskWorld<ScriptedRandom> for DiskFixture {
    fn test_disk_target(&mut self, position: BlockPos, _random: &mut ScriptedRandom) -> bool {
        position.y != 9
    }

    fn provide_disk_state(
        &mut self,
        position: BlockPos,
        _random: &mut ScriptedRandom,
    ) -> Option<BlockStateId> {
        (position.y != 11).then_some(BlockStateId::new(3))
    }

    fn offer_disk_block(&mut self, position: BlockPos, _state: BlockStateId, flags: u32) -> bool {
        assert_eq!(flags, 2);
        self.offers.push(position);
        false
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        BlockStateId::new(1)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn mark_for_postprocessing(&mut self, position: BlockPos) {
        self.postprocessed.push(position);
    }
}

#[derive(Debug)]
struct BasaltFixture {
    origin: BlockPos,
    offers: Vec<BlockPos>,
}

impl BasaltPillarWorld for BasaltFixture {
    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        position == self.origin
    }

    fn is_outside_build_height(&self, _position: BlockPos) -> bool {
        false
    }

    fn offer_basalt(&mut self, position: BlockPos, _state: BlockStateId, flags: u32) -> bool {
        assert_eq!(flags, 2);
        self.offers.push(position);
        false
    }
}

#[derive(Debug)]
struct ReplacementFixture {
    target: BlockStateId,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl ReplacementBlobWorld for ReplacementFixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn maximum_y(&self) -> i32 {
        10
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position.y == 4 {
            self.target
        } else {
            BlockStateId::new(0)
        }
    }

    fn same_block_identity(&self, state: BlockStateId, target: BlockStateId) -> bool {
        state == target
    }

    fn offer_replacement(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct MagmaFixture {
    origin: BlockPos,
    reads: Vec<BlockPos>,
    faces: RefCell<Vec<(BlockPos, BlockFace)>>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl UnderwaterMagmaWorld for MagmaFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position == self.origin {
            BlockStateId::new(1)
        } else {
            BlockStateId::new(2)
        }
    }

    fn is_exact_water(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(1)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn has_full_face(&self, _state: BlockStateId, position: BlockPos, face: BlockFace) -> bool {
        self.faces.borrow_mut().push((position, face));
        true
    }

    fn offer_magma(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct DeltaFixture {
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl DeltaFeatureWorld for DeltaFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        if position.y == 8 {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(2)
        }
    }

    fn same_block_identity(&self, state: BlockStateId, target: BlockStateId) -> bool {
        state == target
    }

    fn is_protected_delta_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn offer_delta_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct DesertWellFixture {
    sand: BlockStateId,
    empty_reads: Vec<BlockPos>,
    state_reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
    loot: Vec<(BlockPos, i64)>,
}

impl DesertWellWorld for DesertWellFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_reads.push(position);
        false
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.state_reads.push(position);
        self.sand
    }

    fn is_sand_block(&self, state: BlockStateId) -> bool {
        state == self.sand
    }

    fn offer_well_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn assign_desert_well_loot(&mut self, position: BlockPos, seed: i64) -> bool {
        self.loot.push((position, seed));
        true
    }
}

const fn packed_block_position(position: BlockPos) -> i64 {
    ((position.x as i64 & 0x3ff_ffff) << 38)
        | ((position.z as i64 & 0x3ff_ffff) << 12)
        | (position.y as i64 & 0xfff)
}

#[derive(Debug)]
struct SpikeFixture {
    empty_reads: Vec<BlockPos>,
    state_reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl SpikeWorld for SpikeFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn is_empty_block(&mut self, position: BlockPos) -> bool {
        self.empty_reads.push(position);
        false
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.state_reads.push(position);
        BlockStateId::new(1)
    }

    fn is_air(&self, _state: BlockStateId) -> bool {
        false
    }

    fn can_place_spike_on(&self, _state: BlockStateId) -> bool {
        true
    }

    fn can_replace_with_spike(&self, _state: BlockStateId) -> bool {
        true
    }

    fn same_block_identity(&self, left: BlockStateId, right: BlockStateId) -> bool {
        left == right
    }

    fn offer_spike(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct DeltaRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
    double_draws: usize,
}

impl GenerationRandom for DeltaRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("delta feature does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        self.double_draws += 1;
        0.95
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("delta feature does not draw Gaussian values")
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
    float_draws: usize,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
            float_draws: 0,
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("block-pile does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("block-pile does not draw Gaussian values")
    }
}
