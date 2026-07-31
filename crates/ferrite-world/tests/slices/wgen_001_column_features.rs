use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::column::{
    BlockColumnConfig, BlockColumnWorld, place_block_column,
};
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn block_column_scans_lookahead_before_tip_preserving_truncation_and_writes() {
    let origin = BlockPos::new(3, 10, -5);
    let mut world = ColumnFixture {
        predicate_reads: Vec::new(),
        provider_calls: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = NoRandom;
    assert!(
        place_block_column(
            &mut world,
            origin,
            &BlockColumnConfig {
                layer_heights: vec![IntProvider::Constant(2), IntProvider::Constant(1)],
                direction: Direction::Up,
                prioritize_tip: true,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(
        world.predicate_reads,
        [BlockPos::new(3, 11, -5), BlockPos::new(3, 12, -5)]
    );
    assert_eq!(world.provider_calls, [(1, origin)]);
    assert_eq!(world.offers, [(origin, BlockStateId::new(11), 2)]);
}

#[test]
fn block_column_wrapped_zero_returns_false_without_predicate_or_state_work() {
    let mut world = ColumnFixture {
        predicate_reads: Vec::new(),
        provider_calls: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = NoRandom;
    assert!(
        !place_block_column(
            &mut world,
            BlockPos::new(0, 0, 0),
            &BlockColumnConfig {
                layer_heights: vec![
                    IntProvider::Constant(i32::MAX),
                    IntProvider::Constant(i32::MAX),
                    IntProvider::Constant(2),
                ],
                direction: Direction::North,
                prioritize_tip: false,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert!(world.predicate_reads.is_empty());
    assert!(world.provider_calls.is_empty());
    assert!(world.offers.is_empty());
}

#[test]
fn block_column_base_priority_truncates_layers_from_the_tip_backwards() {
    let origin = BlockPos::new(3, 10, -5);
    let mut world = ColumnFixture {
        predicate_reads: Vec::new(),
        provider_calls: Vec::new(),
        offers: Vec::new(),
    };
    let mut random = NoRandom;
    assert!(
        place_block_column(
            &mut world,
            origin,
            &BlockColumnConfig {
                layer_heights: vec![IntProvider::Constant(2), IntProvider::Constant(1)],
                direction: Direction::Up,
                prioritize_tip: false,
            },
            &mut random,
            |_| true,
        )
        .unwrap()
    );
    assert_eq!(world.provider_calls, [(0, origin)]);
    assert_eq!(world.offers, [(origin, BlockStateId::new(10), 2)]);
}

#[derive(Debug)]
struct ColumnFixture {
    predicate_reads: Vec<BlockPos>,
    provider_calls: Vec<(usize, BlockPos)>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl BlockColumnWorld<NoRandom> for ColumnFixture {
    fn allowed_placement(&mut self, position: BlockPos) -> bool {
        self.predicate_reads.push(position);
        position.y < 12
    }

    fn provide_layer_state(
        &mut self,
        layer_index: usize,
        position: BlockPos,
        _random: &mut NoRandom,
    ) -> BlockStateId {
        self.provider_calls.push((layer_index, position));
        BlockStateId::new(10 + layer_index as u32)
    }

    fn offer_column_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

#[derive(Debug)]
struct NoRandom;

impl GenerationRandom for NoRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        panic!("constant block-column heights do not draw integers")
    }

    fn next_f32(&mut self) -> f32 {
        panic!("block column does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("block column does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("block column does not draw Gaussian values")
    }
}
