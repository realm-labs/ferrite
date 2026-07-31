use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::structure::BlockBox;
use ferrite_world::generation::structure::piece::{
    FluidState, HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use ferrite_world::generation::structure::processor::StructureState;

#[test]
fn oriented_piece_maps_local_axes_and_rotates_state_properties() {
    let anchor = pos(10, 20, 30);
    let expected = [
        (HorizontalDirection::North, pos(11, 22, 29), "south"),
        (HorizontalDirection::East, pos(11, 22, 31), "west"),
        (HorizontalDirection::South, pos(11, 22, 31), "north"),
        (HorizontalDirection::West, pos(9, 22, 31), "east"),
    ];
    for (orientation, world, facing) in expected {
        let piece = OrientedPiece::from_anchor(anchor, pos(0, 0, 0), [3, 4, 5], orientation);
        assert_eq!(piece.world_position(pos(1, 2, 1)), world);
        let mut state = StructureState::new("minecraft:oak_stairs");
        state.properties.insert("facing".into(), "north".into());
        assert_eq!(piece.transform_state(state).properties["facing"], facing);
    }
}

#[test]
fn fill_box_is_y_x_z_ordered_and_selector_runs_before_clip() {
    let piece = OrientedPiece::from_anchor(
        pos(0, 0, 0),
        pos(0, 0, 0),
        [3, 2, 2],
        HorizontalDirection::South,
    );
    let clip = BlockBox::new(pos(1, 0, 0), pos(2, 1, 1)).unwrap();
    let placement = PiecePlacement { piece, clip: &clip };
    let mut world = TestWorld::default();
    let mut selected = Vec::new();
    placement.fill_box(
        &mut world,
        pos(0, 0, 0),
        pos(2, 1, 1),
        false,
        |local, edge| {
            selected.push((local, edge));
            StructureState::new("minecraft:stone")
        },
    );
    assert_eq!(selected.len(), 12);
    assert_eq!(selected[0].0, pos(0, 0, 0));
    assert_eq!(selected[1].0, pos(0, 0, 1));
    assert_eq!(selected[2].0, pos(1, 0, 0));
    assert_eq!(selected[6].0, pos(0, 1, 0));
    assert_eq!(world.writes.len(), 8);
    assert!(world.writes.iter().all(|write| clip.contains(write.0)));
}

#[test]
fn direct_placement_schedules_postwrite_fluid_and_shape_work_after_failure() {
    let piece = OrientedPiece::from_anchor(
        pos(0, 0, 0),
        pos(0, 0, 0),
        [1, 1, 1],
        HorizontalDirection::South,
    );
    let clip = BlockBox::point(pos(0, 0, 0));
    let placement = PiecePlacement { piece, clip: &clip };
    let mut world = TestWorld {
        reject_writes: true,
        ..TestWorld::default()
    };
    world.fluids.insert(pos(0, 0, 0), FluidState::Water);
    assert!(!placement.place_block(
        &mut world,
        pos(0, 0, 0),
        StructureState::new("minecraft:oak_fence"),
    ));
    assert_eq!(world.fluid_ticks, [(pos(0, 0, 0), FluidState::Water, 0)]);
    assert_eq!(world.shape_positions, [pos(0, 0, 0)]);
}

#[test]
fn downward_columns_clip_only_the_start_and_stop_above_the_floor_guard() {
    let piece = OrientedPiece::from_anchor(
        pos(0, 5, 0),
        pos(0, 0, 0),
        [1, 1, 1],
        HorizontalDirection::South,
    );
    let clip = BlockBox::point(pos(0, 5, 0));
    let placement = PiecePlacement { piece, clip: &clip };
    let mut world = TestWorld::default();
    world
        .states
        .insert(pos(0, 2, 0), StructureState::new("minecraft:stone"));
    placement.fill_column_down(
        &mut world,
        pos(0, 0, 0),
        StructureState::new("minecraft:oak_log"),
        -64,
        |state, fluid| state.block == "minecraft:air" && fluid.is_empty(),
    );
    assert_eq!(
        world
            .writes
            .iter()
            .map(|write| write.0.y)
            .collect::<Vec<_>>(),
        [5, 4, 3]
    );
}

#[test]
fn chest_helper_latches_after_admission_but_seed_install_is_type_gated() {
    let piece = OrientedPiece::from_anchor(
        pos(0, 0, 0),
        pos(0, 0, 0),
        [1, 1, 1],
        HorizontalDirection::South,
    );
    let clip = BlockBox::point(pos(0, 0, 0));
    let placement = PiecePlacement { piece, clip: &clip };
    let mut world = TestWorld {
        reject_writes: true,
        loot_targets: BTreeSet::from([pos(0, 0, 0)]),
        ..TestWorld::default()
    };
    world.solids.insert(pos(-1, 0, 0));
    assert!(placement.create_chest(&mut world, pos(0, 0, 0), "minecraft:chests/test", || 42,));
    assert_eq!(world.writes[0].1.properties["facing"], "east");
    assert_eq!(
        world.loot,
        [(pos(0, 0, 0), "minecraft:chests/test".into(), 42)]
    );

    world
        .states
        .insert(pos(0, 0, 0), StructureState::new("minecraft:chest"));
    assert!(!placement.create_chest(&mut world, pos(0, 0, 0), "minecraft:chests/other", || 7,));
}

fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos { x, y, z }
}

#[derive(Default)]
struct TestWorld {
    states: BTreeMap<BlockPos, StructureState>,
    fluids: BTreeMap<BlockPos, FluidState>,
    solids: BTreeSet<BlockPos>,
    loot_targets: BTreeSet<BlockPos>,
    writes: Vec<(BlockPos, StructureState, u32)>,
    fluid_ticks: Vec<(BlockPos, FluidState, u32)>,
    shape_positions: Vec<BlockPos>,
    loot: Vec<(BlockPos, String, i64)>,
    reject_writes: bool,
}

impl PieceWorld for TestWorld {
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
        if !self.reject_writes {
            self.states.insert(position, state);
        }
        !self.reject_writes
    }

    fn schedule_fluid_tick(&mut self, position: BlockPos, fluid: FluidState, delay: u32) {
        self.fluid_ticks.push((position, fluid, delay));
    }

    fn mark_shape_postprocessing(&mut self, position: BlockPos) {
        self.shape_positions.push(position);
    }

    fn solid_render(&mut self, position: BlockPos) -> bool {
        self.solids.contains(&position)
    }

    fn is_loot_container(&mut self, position: BlockPos) -> bool {
        self.loot_targets.contains(&position)
    }

    fn install_loot(&mut self, position: BlockPos, table: &str, seed: i64) {
        self.loot.push((position, table.into(), seed));
    }
}
