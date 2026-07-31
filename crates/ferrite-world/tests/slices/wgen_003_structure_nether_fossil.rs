use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::nether_fossil::{
    FOSSIL_TEMPLATES, FossilRotation, NetherFossilWorld, choose_piece, find_anchor,
    place_dried_ghast,
};
use ferrite_world::generation::structure::piece::{FluidState, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn anchor_draws_x_z_height_then_returns_the_lower_support_cell() {
    let mut world = World {
        top: 40,
        sea: 32,
        ..World::default()
    };
    world
        .column
        .insert(34, StructureState::new("minecraft:air"));
    world
        .column
        .insert(33, StructureState::new("minecraft:soul_sand"));
    let mut random = ScriptRandom::new([15, 0, 3]);
    assert_eq!(
        find_anchor(&mut world, -16, 32, &mut random),
        Some(pos(-1, 33, 32))
    );
    assert_eq!(random.bounds, [16, 16, 7]);
    assert_eq!(world.column_reads, [35, 34, 34, 33]);
}

#[test]
fn inverted_height_range_returns_minimum_without_a_height_draw() {
    let mut world = World {
        top: 20,
        sea: 40,
        ..World::default()
    };
    let mut random = ScriptRandom::new([1, 2]);
    assert_eq!(find_anchor(&mut world, 0, 0, &mut random), None);
    assert_eq!(random.bounds, [16, 16]);
    assert!(world.column_reads.is_empty());
}

#[test]
fn piece_choice_and_locked_template_census_are_exact() {
    let choice = choose_piece(pos(1, 2, 3), &mut ScriptRandom::new([3, 13]));
    assert_eq!(choice.rotation, FossilRotation::Counterclockwise90);
    assert_eq!(choice.template, 14);
    assert_eq!(
        FOSSIL_TEMPLATES
            .iter()
            .map(|template| {
                usize::from(template.y_axis_bones)
                    + usize::from(template.x_axis_bones)
                    + usize::from(template.z_axis_bones)
            })
            .sum::<usize>(),
        183
    );
    assert_eq!(
        FOSSIL_TEMPLATES
            .iter()
            .map(|template| template
                .size
                .map(usize::from)
                .into_iter()
                .product::<usize>())
            .sum::<usize>(),
        1_194
    );
}

#[test]
fn dried_ghast_stream_replays_from_box_center_and_ignores_write_result() {
    let bounds = BlockBox::new(pos(10, 4, 20), pos(13, 7, 24)).unwrap();
    let mut selected = None;
    for seed in 0..10_000_i64 {
        let mut random = LegacyRandom::new(seed);
        if random.next_f32() < 0.5 {
            selected = Some(seed);
            break;
        }
    }
    let seed = selected.unwrap();
    let mut first = World {
        positional_seed: seed,
        reject_writes: true,
        ..World::default()
    };
    let mut second = World {
        positional_seed: seed,
        reject_writes: true,
        ..World::default()
    };
    assert!(place_dried_ghast(&mut first, bounds, &bounds));
    assert!(place_dried_ghast(&mut second, bounds, &bounds));
    assert_eq!(first.writes, second.writes);
    let (_, state, flags) = &first.writes[0];
    assert_eq!(state.block, "minecraft:dried_ghast");
    assert_eq!(state.properties["hydration"], "0");
    assert_eq!(state.properties["waterlogged"], "false");
    assert_eq!(*flags, 2);
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    column: BTreeMap<i32, StructureState>,
    column_reads: Vec<i32>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    top: i32,
    sea: i32,
    positional_seed: i64,
    reject_writes: bool,
}

impl PieceWorld for World {
    fn state_at(&mut self, _position: BlockPos) -> StructureState {
        StructureState::new("minecraft:air")
    }

    fn fluid_at(&mut self, _position: BlockPos) -> FluidState {
        FluidState::Empty
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        self.writes.push((position, state, flags));
        !self.reject_writes
    }

    fn schedule_fluid_tick(&mut self, _position: BlockPos, _fluid: FluidState, _delay: u32) {}

    fn mark_shape_postprocessing(&mut self, _position: BlockPos) {}

    fn solid_render(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn is_loot_container(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

impl NetherFossilWorld for World {
    fn sea_level(&self) -> i32 {
        self.sea
    }

    fn generation_top(&self) -> i32 {
        self.top
    }

    fn base_column_state(&mut self, _x: i32, y: i32, _z: i32) -> StructureState {
        self.column_reads.push(y);
        self.column
            .get(&y)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:netherrack"))
    }

    fn sturdy_upper_face(&mut self, state: &StructureState) -> bool {
        state.block == "minecraft:netherrack"
    }

    fn positional_seed(&self, _position: BlockPos) -> i64 {
        self.positional_seed
    }
}

struct ScriptRandom {
    values: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            values: values.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        self.values.pop_front().unwrap_or_default() % bound.get()
    }

    fn next_f32(&mut self) -> f32 {
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
