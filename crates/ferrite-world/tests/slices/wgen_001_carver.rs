use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::carver::{
    CarverEllipsoid, CarverMaterialConfig, CarverShiftNoise, CarverWorld, CarvingMask,
    OldGenerationCuboid, carve_ellipsoid, old_generation_excludes,
};
use ferrite_world::id::BlockStateId;

#[test]
fn mask_is_set_before_aquifer_rejection_and_suppresses_the_second_visit() {
    let mut world = Fixture {
        aquifer: None,
        ..Fixture::default()
    };
    let mut mask = Mask::default();

    assert!(!carve(&mut world, &mut mask, config(-100)).unwrap());
    assert_eq!(mask.sets, [(0, 11, 0)]);
    assert_eq!(world.reads, [BlockPos::new(0, 11, 0)]);

    world.reads.clear();
    assert!(!carve(&mut world, &mut mask, config(-100)).unwrap());
    assert!(world.reads.is_empty());
}

#[test]
fn forced_lava_still_queries_aquifer_update_flag_and_marks_fluid() {
    let mut world = Fixture {
        schedule_fluid: true,
        ..Fixture::default()
    };
    let mut mask = Mask::default();

    assert!(carve(&mut world, &mut mask, config(11)).unwrap());

    assert_eq!(world.aquifer_calls, 0);
    assert_eq!(world.update_queries, 1);
    assert_eq!(world.offers, [(BlockPos::new(0, 11, 0), LAVA)]);
    assert_eq!(world.postprocess, [BlockPos::new(0, 11, 0)]);
}

#[test]
fn remembered_surface_restores_dirt_below_after_the_carve() {
    let carved = BlockPos::new(0, 11, 0);
    let below = BlockPos::new(0, 10, 0);
    let mut world = Fixture::default();
    world.states.insert(carved, GRASS);
    world.states.insert(below, DIRT);
    world.aquifer = Some(AIR);
    let mut mask = Mask::default();

    assert!(carve(&mut world, &mut mask, config(-100)).unwrap());

    assert_eq!(world.offers, [(carved, AIR), (below, TOP)]);
    assert_eq!(world.surface_calls, [(below, 1, 1, i32::MIN)]);
}

#[test]
fn old_generation_filter_uses_three_permuted_noise_calls_and_strict_distance() {
    let cuboid = OldGenerationCuboid::from_blending_data(0, 16, 0, 0);
    let mut noise = ShiftNoise::new([-0.125, 0.0, 0.0]);

    assert!(!old_generation_excludes(&[cuboid], &mut noise, 20, 8, 8));
    assert_eq!(noise.calls, [(20, 8, 8), (8, 8, 20), (8, 20, 8)]);

    let mut noise = ShiftNoise::new([-0.125, 0.0, 0.0]);
    assert!(old_generation_excludes(&[cuboid], &mut noise, 19, 8, 8));
}

fn carve(
    world: &mut Fixture,
    mask: &mut Mask,
    config: CarverMaterialConfig,
) -> Result<bool, ferrite_world::generation::carver::CarverError> {
    carve_ellipsoid(
        world,
        mask,
        0,
        0,
        CarverEllipsoid {
            center_x: 0.5,
            center_y: 10.5,
            center_z: 0.5,
            horizontal_radius: 0.6,
            vertical_radius: 0.6,
        },
        config,
        |x, y, z, _| x * x + y * y + z * z >= 1.0,
    )
}

fn config(lava_level: i32) -> CarverMaterialConfig {
    CarverMaterialConfig {
        lava_level,
        lava: LAVA,
        debug_mode: false,
        debug_barrier: BARRIER,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const LAVA: BlockStateId = BlockStateId::new(2);
const GRASS: BlockStateId = BlockStateId::new(3);
const DIRT: BlockStateId = BlockStateId::new(4);
const TOP: BlockStateId = BlockStateId::new(5);
const BARRIER: BlockStateId = BlockStateId::new(6);

#[derive(Debug, Default)]
struct Mask {
    values: BTreeSet<(u8, i32, u8)>,
    sets: Vec<(u8, i32, u8)>,
}

impl CarvingMask for Mask {
    fn contains(&self, local_x: u8, y: i32, local_z: u8) -> bool {
        self.values.contains(&(local_x, y, local_z))
    }

    fn set(&mut self, local_x: u8, y: i32, local_z: u8) {
        self.values.insert((local_x, y, local_z));
        self.sets.push((local_x, y, local_z));
    }
}

#[derive(Debug)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    aquifer: Option<BlockStateId>,
    schedule_fluid: bool,
    reads: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId)>,
    postprocess: Vec<BlockPos>,
    aquifer_calls: usize,
    update_queries: usize,
    surface_calls: Vec<(BlockPos, i32, i32, i32)>,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            states: BTreeMap::new(),
            aquifer: Some(AIR),
            schedule_fluid: false,
            reads: Vec::new(),
            offers: Vec::new(),
            postprocess: Vec::new(),
            aquifer_calls: 0,
            update_queries: 0,
            surface_calls: Vec::new(),
        }
    }
}

impl CarverWorld for Fixture {
    fn minimum_y(&self) -> i32 {
        0
    }

    fn generation_depth(&self) -> i32 {
        64
    }

    fn upgrading_chunk(&self) -> bool {
        false
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        self.states.get(&position).copied().unwrap_or(STONE)
    }

    fn is_grass_or_mycelium(&self, state: BlockStateId) -> bool {
        state == GRASS
    }

    fn is_carver_replaceable(&self, state: BlockStateId) -> bool {
        matches!(state, STONE | GRASS | DIRT)
    }

    fn aquifer_substance(&mut self, _position: BlockPos, _density: f64) -> Option<BlockStateId> {
        self.aquifer_calls += 1;
        self.aquifer
    }

    fn debug_marker_for(&self, state: BlockStateId) -> BlockStateId {
        state
    }

    fn aquifer_should_schedule_fluid_update(&mut self) -> bool {
        self.update_queries += 1;
        self.schedule_fluid
    }

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool {
        state == LAVA
    }

    fn offer_carved_block(&mut self, position: BlockPos, state: BlockStateId) {
        self.offers.push((position, state));
    }

    fn mark_for_postprocessing(&mut self, position: BlockPos) {
        self.postprocess.push(position);
    }

    fn is_dirt(&self, state: BlockStateId) -> bool {
        state == DIRT
    }

    fn surface_top_material(
        &mut self,
        position: BlockPos,
        above: i32,
        below: i32,
        water_height: i32,
    ) -> Option<BlockStateId> {
        self.surface_calls
            .push((position, above, below, water_height));
        Some(TOP)
    }
}

#[derive(Debug)]
struct ShiftNoise {
    values: std::collections::VecDeque<f64>,
    calls: Vec<(i32, i32, i32)>,
}

impl ShiftNoise {
    fn new(values: impl IntoIterator<Item = f64>) -> Self {
        Self {
            values: values.into_iter().collect(),
            calls: Vec::new(),
        }
    }
}

impl CarverShiftNoise for ShiftNoise {
    fn sample(&mut self, x: i32, y: i32, z: i32) -> f64 {
        self.calls.push((x, y, z));
        self.values.pop_front().expect("scripted shift noise")
    }
}
