use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::speleothem::{PointedThickness, SpeleothemWorld};
use ferrite_world::generation::feature::speleothem_cluster::{
    ClusterFloatProvider, SpeleothemClusterConfig, SpeleothemClusterWorld, place_speleothem_cluster,
};
use ferrite_world::id::BlockStateId;

#[test]
fn cluster_column_draws_wetness_then_two_double_gates_and_unconditional_merge_boolean() {
    let origin = BlockPos::new(0, 10, 0);
    let mut world = ClusterFixture {
        ceiling_y: 12,
        floor_y: 8,
        offers: Vec::new(),
    };
    let mut random = ScriptedRandom {
        integers: [0, 1].into_iter().collect(),
        floats: [0.5, 0.0].into_iter().collect(),
        doubles: [0.0, 0.0].into_iter().collect(),
        bounds: Vec::new(),
        gaussian_draws: 0,
    };
    assert!(
        place_speleothem_cluster(&mut world, origin, &config(), &mut random, |_| true,).unwrap()
    );
    assert_eq!(random.bounds, [1, 2]);
    assert_eq!(random.gaussian_draws, 1);
    assert_eq!(
        world.offers,
        [
            (BlockPos::new(0, 12, 0), BlockStateId::new(3), 2),
            (BlockPos::new(0, 8, 0), BlockStateId::new(3), 2),
        ]
    );
}

fn config() -> SpeleothemClusterConfig {
    SpeleothemClusterConfig {
        base_block: BlockStateId::new(3),
        pointed_block: BlockStateId::new(4),
        water: BlockStateId::new(5),
        floor_to_ceiling_search_range: 8,
        height: IntProvider::Constant(1),
        wetness: ClusterFloatProvider::Constant(0.0),
        density: ClusterFloatProvider::Constant(1.0),
        radius: IntProvider::Constant(0),
        maximum_stalagmite_stalactite_height_difference: 0,
        height_deviation: 0.0,
        base_layer_thickness: IntProvider::Constant(1),
        chance_at_edge: 1.0,
        maximum_edge_distance: 1,
        maximum_height_bias_distance: 1,
    }
}

#[derive(Debug)]
struct ClusterFixture {
    ceiling_y: i32,
    floor_y: i32,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl SpeleothemWorld for ClusterFixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        if position.y >= self.ceiling_y || position.y <= self.floor_y {
            BlockStateId::new(2)
        } else {
            BlockStateId::new(0)
        }
    }

    fn is_base_block_identity(&self, state: BlockStateId, base: BlockStateId) -> bool {
        state == base
    }

    fn is_replaceable_speleothem_block(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(2)
    }

    fn is_air_or_water_block(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_water_at(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn configure_pointed_state(
        &mut self,
        default_state: BlockStateId,
        _direction: Direction,
        _thickness: PointedThickness,
        _waterlogged: bool,
    ) -> Option<BlockStateId> {
        Some(default_state)
    }

    fn offer_speleothem_block(
        &mut self,
        position: BlockPos,
        state: BlockStateId,
        flags: u32,
    ) -> bool {
        self.offers.push((position, state, flags));
        false
    }
}

impl SpeleothemClusterWorld for ClusterFixture {
    fn is_lava_block(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_water_block_identity(&self, _state: BlockStateId) -> bool {
        false
    }

    fn has_water_tagged_fluid(&mut self, _position: BlockPos, _state: BlockStateId) -> bool {
        false
    }

    fn is_base_stone_overworld(&self, _state: BlockStateId) -> bool {
        true
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    doubles: VecDeque<f64>,
    bounds: Vec<u32>,
    gaussian_draws: usize,
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
        self.doubles.pop_front().expect("scripted double")
    }

    fn next_gaussian(&mut self) -> f64 {
        self.gaussian_draws += 1;
        0.0
    }
}
