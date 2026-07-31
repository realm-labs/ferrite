use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::surface_extension::{
    FrozenOceanInput, FrozenOceanStates, SurfaceExtensionColumn, SurfaceExtensionNoise,
    eroded_badlands_extension, frozen_ocean_extension,
};
use ferrite_world::id::BlockStateId;

#[test]
fn badlands_reads_support_pass_then_restarts_at_top_for_air_fill() {
    let mut column = Column::default();
    column.states.insert(BlockPos::new(0, 86, 0), STONE);
    let mut noise = Noise {
        badlands_surface: 1.0,
        badlands_pillar: 1.0,
        ..Noise::default()
    };

    eroded_badlands_extension(&mut column, &mut noise, 0, 0, 80, 0, STONE);

    assert_eq!(
        &column.reads[..6],
        [
            BlockPos::new(0, 88, 0),
            BlockPos::new(0, 87, 0),
            BlockPos::new(0, 86, 0),
            BlockPos::new(0, 88, 0),
            BlockPos::new(0, 87, 0),
            BlockPos::new(0, 86, 0),
        ]
    );
    assert_eq!(
        column.writes,
        [
            (BlockPos::new(0, 88, 0), STONE),
            (BlockPos::new(0, 87, 0), STONE),
        ]
    );
}

#[test]
fn frozen_threshold_equality_stops_before_roof_and_random() {
    let mut column = Column::default();
    let mut noise = Noise {
        iceberg_surface: 1.8 / 8.25,
        iceberg_pillar: 1.0,
        ..Noise::default()
    };
    let mut random = ScriptedRandom::new([], []);

    frozen_ocean_extension(&mut column, &mut noise, &mut random, frozen_input());

    assert_eq!(noise.iceberg_roof_calls, 0);
    assert!(random.bounds.is_empty());
    assert!(column.reads.is_empty());
}

#[test]
fn frozen_air_gate_rereads_failed_cells_and_uses_strict_double_threshold() {
    let mut column = Column::default();
    let mut noise = Noise {
        iceberg_surface: 2.0 / 8.25,
        iceberg_pillar: 1.0,
        ..Noise::default()
    };
    let mut random = ScriptedRandom::new([0, 0], [0.02, 0.02]);

    frozen_ocean_extension(&mut column, &mut noise, &mut random, frozen_input());

    assert_eq!(random.bounds, [4, 10]);
    assert_eq!(
        column.reads,
        [
            BlockPos::new(0, 68, 0),
            BlockPos::new(0, 68, 0),
            BlockPos::new(0, 67, 0),
            BlockPos::new(0, 67, 0),
            BlockPos::new(0, 66, 0),
            BlockPos::new(0, 65, 0),
        ]
    );
    assert_eq!(
        column.writes,
        [
            (BlockPos::new(0, 66, 0), PACKED_ICE),
            (BlockPos::new(0, 65, 0), PACKED_ICE),
        ]
    );
}

fn frozen_input() -> FrozenOceanInput {
    FrozenOceanInput {
        x: 0,
        z: 0,
        original_height: 64,
        minimum_surface_y: 65,
        sea_level: 63,
        melts_slightly: false,
        states: FrozenOceanStates {
            snow: SNOW,
            packed_ice: PACKED_ICE,
        },
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const WATER: BlockStateId = BlockStateId::new(2);
const SNOW: BlockStateId = BlockStateId::new(3);
const PACKED_ICE: BlockStateId = BlockStateId::new(4);

#[derive(Debug, Default)]
struct Column {
    states: BTreeMap<BlockPos, BlockStateId>,
    reads: Vec<BlockPos>,
    writes: Vec<(BlockPos, BlockStateId)>,
}

impl SurfaceExtensionColumn for Column {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        self.states.get(&position).copied().unwrap_or(AIR)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_water(&self, state: BlockStateId) -> bool {
        state == WATER
    }

    fn is_same_block_as_default(&self, state: BlockStateId, default: BlockStateId) -> bool {
        state == default
    }

    fn set_extension_block(&mut self, position: BlockPos, state: BlockStateId) {
        self.writes.push((position, state));
    }
}

#[derive(Debug, Default)]
struct Noise {
    badlands_surface: f64,
    badlands_pillar: f64,
    iceberg_surface: f64,
    iceberg_pillar: f64,
    iceberg_roof_calls: usize,
}

impl SurfaceExtensionNoise for Noise {
    fn badlands_surface(&mut self, _x: i32, _z: i32) -> f64 {
        self.badlands_surface
    }

    fn badlands_pillar(&mut self, _x: f64, _z: f64) -> f64 {
        self.badlands_pillar
    }

    fn badlands_roof(&mut self, _x: f64, _z: f64) -> f64 {
        0.0
    }

    fn iceberg_surface(&mut self, _x: i32, _z: i32) -> f64 {
        self.iceberg_surface
    }

    fn iceberg_pillar(&mut self, _x: f64, _z: f64) -> f64 {
        self.iceberg_pillar
    }

    fn iceberg_roof(&mut self, _x: f64, _z: f64) -> f64 {
        self.iceberg_roof_calls += 1;
        0.0
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    doubles: VecDeque<f64>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        doubles: impl IntoIterator<Item = f64>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            doubles: doubles.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        self.integers.pop_front().expect("unexpected integer draw")
    }

    fn next_f32(&mut self) -> f32 {
        panic!("unexpected float draw")
    }

    fn next_f64(&mut self) -> f64 {
        self.doubles.pop_front().expect("unexpected double draw")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("unexpected Gaussian draw")
    }
}
