use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::piece::{FluidState, HorizontalDirection, PieceWorld};
use ferrite_world::generation::structure::processor::StructureState;
use ferrite_world::generation::structure::swamp_hut::{
    SwampHutOccupant, SwampHutPiece, SwampHutSpawn, SwampHutWorld,
};

#[test]
fn full_hut_aligns_writes_exact_geometry_supports_and_latches_occupants() {
    let mut hut = SwampHutPiece::new(pos(0, 0, 0), HorizontalDirection::South);
    let clip = BlockBox::new(pos(-20, -20, -20), pos(20, 100, 20)).unwrap();
    let mut world = World {
        height: 70,
        minimum_y: -64,
        ..World::default()
    };
    for z in [2, 7] {
        for x in [1, 5] {
            world
                .states
                .insert(pos(x, 66, z), StructureState::new("minecraft:stone"));
        }
    }
    world.fluids.insert(pos(3, 71, 6), FluidState::Water);

    assert!(hut.place(&mut world, &clip));
    assert_eq!(world.height_probes, 63);
    assert_eq!(hut.average_ground_height, 70);
    assert_eq!(hut.piece.bounds.minimum.y, 70);
    assert_eq!(world.writes.len(), 162);
    assert!(
        world
            .writes
            .iter()
            .take(150)
            .all(|(_, _, flags)| *flags == 2)
    );
    assert_eq!(
        world.fluid_ticks,
        vec![(pos(3, 71, 6), FluidState::Water, 0)]
    );
    assert_eq!(world.shape_positions.len(), 4);
    assert_eq!(
        world.spawns,
        [
            SwampHutSpawn {
                occupant: SwampHutOccupant::Witch,
                position: pos(2, 72, 5),
                persistent: true,
                finalize_structure_spawn: true,
                force_black_cat: false,
            },
            SwampHutSpawn {
                occupant: SwampHutOccupant::Cat,
                position: pos(2, 72, 5),
                persistent: true,
                finalize_structure_spawn: true,
                force_black_cat: true,
            },
        ]
    );
    assert!(hut.witch_spawned && hut.cat_spawned);

    let probes = world.height_probes;
    assert!(hut.place(&mut world, &clip));
    assert_eq!(world.height_probes, probes);
    assert_eq!(world.spawns.len(), 2);
    assert_eq!(world.writes.len(), 312);
}

#[test]
fn empty_probe_clip_aborts_before_caching_or_writing() {
    let mut hut = SwampHutPiece::new(pos(0, 0, 0), HorizontalDirection::East);
    assert_eq!(hut.piece.bounds.size(), [9, 7, 7]);
    let clip = BlockBox::point(pos(0, 63, 0));
    let mut world = World {
        height: 70,
        minimum_y: -64,
        ..World::default()
    };
    assert!(!hut.place(&mut world, &clip));
    assert_eq!(hut.average_ground_height, -1);
    assert_eq!(world.height_probes, 0);
    assert!(world.writes.is_empty());
}

#[test]
fn negative_cached_mean_remains_a_sentinel_and_moves_again() {
    let mut hut = SwampHutPiece::new(pos(0, 0, 0), HorizontalDirection::South);
    let clip = BlockBox::new(pos(-20, -100, -20), pos(20, 100, 20)).unwrap();
    let mut world = World {
        height: -5,
        minimum_y: -64,
        ..World::default()
    };
    assert!(hut.place(&mut world, &clip));
    assert_eq!(hut.piece.bounds.minimum.y, -5);
    let first_probes = world.height_probes;
    world.height = -3;
    assert!(hut.place(&mut world, &clip));
    assert_eq!(world.height_probes, first_probes + 63);
    assert_eq!(hut.piece.bounds.minimum.y, -3);
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct World {
    height: i32,
    minimum_y: i32,
    height_probes: usize,
    states: BTreeMap<BlockPos, StructureState>,
    fluids: BTreeMap<BlockPos, FluidState>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    fluid_ticks: Vec<(BlockPos, FluidState, u32)>,
    shape_positions: Vec<BlockPos>,
    spawns: Vec<SwampHutSpawn>,
}

impl PieceWorld for World {
    fn state_at(&mut self, position: BlockPos) -> StructureState {
        self.states
            .get(&position)
            .cloned()
            .unwrap_or_else(|| StructureState::new("minecraft:air"))
    }

    fn fluid_at(&mut self, position: BlockPos) -> FluidState {
        self.fluids
            .get(&position)
            .copied()
            .unwrap_or(FluidState::Empty)
    }

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool {
        self.writes.push((position, state.clone(), flags));
        self.states.insert(position, state);
        true
    }

    fn schedule_fluid_tick(&mut self, position: BlockPos, fluid: FluidState, delay: u32) {
        self.fluid_ticks.push((position, fluid, delay));
    }

    fn mark_shape_postprocessing(&mut self, position: BlockPos) {
        self.shape_positions.push(position);
    }

    fn solid_render(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn is_loot_container(&mut self, _position: BlockPos) -> bool {
        false
    }

    fn install_loot(&mut self, _position: BlockPos, _table: &str, _seed: i64) {}
}

impl SwampHutWorld for World {
    fn motion_blocking_no_leaves_height(&mut self, _x: i32, _z: i32) -> i32 {
        self.height_probes += 1;
        self.height
    }

    fn minimum_y(&self) -> i32 {
        self.minimum_y
    }

    fn spawn_swamp_hut_occupant(&mut self, request: SwampHutSpawn) {
        self.spawns.push(request);
    }
}
