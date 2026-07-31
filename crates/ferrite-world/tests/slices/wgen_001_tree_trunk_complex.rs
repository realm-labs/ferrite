use std::cell::RefCell;
use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Axis;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{TreePlacementContext, TreeWorld};
use ferrite_world::generation::feature::tree_trunk::TrunkWorld;
use ferrite_world::generation::feature::tree_trunk_complex::{
    CherryTrunkConfig, place_cherry_trunk, place_dark_oak_trunk, place_fancy_trunk,
};
use ferrite_world::id::BlockStateId;

#[test]
fn dark_oak_scans_all_twelve_peripheral_cells_after_the_main_attachment() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut integers = vec![0, 0, 0];
    integers.extend([1; 12]);
    let mut random = ScriptedRandom::new(integers, []);

    let attachments = place_dark_oak_trunk(&mut context, &mut random, 1, origin).unwrap();

    assert_eq!(random.bounds, [4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3]);
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].position, origin);
    assert!(attachments[0].double_trunk);
}

#[test]
fn short_fancy_tree_keeps_its_unvalidated_initial_attachment() {
    let origin = BlockPos::new(2, 20, 3);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::default();

    let attachments = place_fancy_trunk(&mut context, &mut random, 3, origin).unwrap();

    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].position, origin);
    assert!(random.floats.is_empty());
    assert_eq!(context.world().trunk_samples.len(), 8);
}

#[test]
fn cherry_samples_both_starts_even_for_one_branch_and_uses_shared_horizontal_axis() {
    let origin = BlockPos::new(0, 30, 0);
    let config = CherryTrunkConfig {
        branch_count: IntProvider::Constant(1),
        branch_horizontal_length: IntProvider::Constant(2),
        branch_start_minimum: -2,
        branch_start_maximum: 0,
        branch_end_offset_from_top: IntProvider::Constant(0),
    };
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::new([0, 0, 0], [0.0, 0.0, 1.0]);

    let attachments = place_cherry_trunk(&mut context, &mut random, 5, origin, &config).unwrap();

    assert_eq!(random.bounds, [3, 2, 4]);
    assert_eq!(
        attachments,
        [
            ferrite_world::generation::feature::tree_core::FoliageAttachment {
                position: BlockPos::new(0, 35, -2),
                radius_offset: 0,
                double_trunk: false,
            }
        ]
    );
    assert!(context.world().axes.borrow().contains(&Axis::Z));
}

const AIR: BlockStateId = BlockStateId::new(0);
const DIRT: BlockStateId = BlockStateId::new(1);
const LOG: BlockStateId = BlockStateId::new(2);

#[derive(Debug, Default)]
struct Fixture {
    trunk_samples: Vec<BlockPos>,
    axes: RefCell<Vec<Axis>>,
}

impl TreeWorld for Fixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn maximum_y(&self) -> i32 {
        255
    }

    fn block_state(&mut self, _position: BlockPos) -> BlockStateId {
        AIR
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_replaceable_by_trees(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_log(&self, state: BlockStateId) -> bool {
        state == LOG
    }

    fn is_vine(&self, _state: BlockStateId) -> bool {
        false
    }

    fn optional_leaf_distance(&self, _state: BlockStateId) -> Option<u8> {
        None
    }

    fn with_leaf_distance(&self, state: BlockStateId, _distance: u8) -> BlockStateId {
        state
    }

    fn offer_tree_block(&mut self, _position: BlockPos, _state: BlockStateId, _flags: u32) -> bool {
        false
    }

    fn update_tree_shape_at_edge(
        &mut self,
        _radius: u32,
        _minimum: BlockPos,
        _maximum: BlockPos,
        _filled: &[BlockPos],
    ) {
    }
}

impl TrunkWorld for Fixture {
    fn sample_below_trunk<R: GenerationRandom>(
        &mut self,
        _position: BlockPos,
        _random: &mut R,
    ) -> Option<BlockStateId> {
        Some(DIRT)
    }

    fn sample_trunk<R: GenerationRandom>(
        &mut self,
        position: BlockPos,
        _random: &mut R,
    ) -> BlockStateId {
        self.trunk_samples.push(position);
        LOG
    }

    fn with_trunk_axis(&self, state: BlockStateId, axis: Axis) -> BlockStateId {
        self.axes.borrow_mut().push(axis);
        state
    }

    fn can_upward_branch_grow_through(&self, _state: BlockStateId) -> bool {
        false
    }
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>, floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            bounds: Vec::new(),
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
        self.floats.pop_front().expect("scripted float")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fixture does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fixture does not draw Gaussian values")
    }
}
