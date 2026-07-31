use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::aquifer::{
    AquiferEnvironment, AquiferResolver, AquiferStates, DisabledAquifer, EnabledAquifer,
    FluidStatus, GlobalFluidPicker, pressure_value,
};
use ferrite_world::id::BlockStateId;

#[test]
fn disabled_aquifer_uses_strict_positive_density_and_never_schedules() {
    let mut aquifer = DisabledAquifer::new(global_picker());
    let position = BlockPos::new(0, 20, 0);

    assert_eq!(aquifer.compute_substance(position, 0.0), Some(WATER));
    assert_eq!(aquifer.compute_substance(position, f64::EPSILON), None);
    assert!(!aquifer.should_schedule_fluid_update());
}

#[test]
fn global_picker_switches_status_below_minimum_of_lava_and_sea_levels() {
    let picker = global_picker();

    assert_eq!(
        picker.status(BlockPos::new(0, -55, 0)),
        FluidStatus {
            level: -54,
            state: LAVA,
        }
    );
    assert_eq!(
        picker.status(BlockPos::new(0, -54, 0)),
        FluidStatus {
            level: 63,
            state: WATER,
        }
    );
}

#[test]
fn pressure_handles_water_lava_equal_levels_and_closed_noise_band() {
    let water = FluidStatus {
        level: 20,
        state: WATER,
    };
    let lava = FluidStatus {
        level: 20,
        state: LAVA,
    };
    assert_eq!(pressure_value(0, water, lava, states(), 99.0), 2.0);
    assert_eq!(pressure_value(0, water, water, states(), 99.0), 0.0);

    let high = FluidStatus {
        level: 10,
        state: WATER,
    };
    let low = FluidStatus {
        level: 0,
        state: WATER,
    };
    assert_eq!(pressure_value(5, high, low, states(), 99.0), 6.0);
    assert_eq!(
        pressure_value(10, high, low, states(), 0.25),
        2.0 * (0.25 + -0.5 / 2.5)
    );
}

#[test]
fn enabled_aquifer_density_and_ceiling_bypasses_precede_center_sampling() {
    let mut aquifer = EnabledAquifer::new(Environment::default(), states(), 0, -32_512, false);
    assert_eq!(aquifer.skip_sampling_above_y(), 34);

    assert_eq!(aquifer.compute_substance(BlockPos::new(0, 0, 0), 0.1), None);
    assert_eq!(
        aquifer.compute_substance(BlockPos::new(0, 35, 0), 0.0),
        Some(WATER)
    );
    assert!(aquifer.environment().center_cells.is_empty());
}

#[test]
fn local_center_and_status_caches_survive_repeated_material_queries() {
    let mut aquifer = EnabledAquifer::new(Environment::default(), states(), 100, -32_512, false);
    let position = BlockPos::new(0, 0, 0);

    assert_eq!(aquifer.compute_substance(position, -100.0), Some(WATER));
    assert_eq!(aquifer.environment().center_cells.len(), 12);
    let surface_calls = aquifer.environment().surface_calls;

    assert_eq!(aquifer.compute_substance(position, -100.0), Some(WATER));
    assert_eq!(aquifer.environment().center_cells.len(), 12);
    assert_eq!(aquifer.environment().surface_calls, surface_calls);
}

const AIR: BlockStateId = BlockStateId::new(0);
const WATER: BlockStateId = BlockStateId::new(1);
const LAVA: BlockStateId = BlockStateId::new(2);

fn states() -> AquiferStates {
    AquiferStates {
        air: AIR,
        water: WATER,
        lava: LAVA,
    }
}

fn global_picker() -> GlobalFluidPicker {
    GlobalFluidPicker {
        sea_level: 63,
        default_fluid: WATER,
        states: states(),
    }
}

#[derive(Debug, Default)]
struct Environment {
    center_cells: Vec<[i32; 3]>,
    surface_calls: usize,
}

impl AquiferEnvironment for Environment {
    fn global_fluid(&mut self, position: BlockPos) -> FluidStatus {
        global_picker().status(position)
    }

    fn preliminary_surface(&mut self, _x: i32, _z: i32) -> i32 {
        self.surface_calls += 1;
        100
    }

    fn center_offsets(&mut self, cell: [i32; 3]) -> [i32; 3] {
        self.center_cells.push(cell);
        [0, 0, 0]
    }

    fn erosion(&mut self, _position: BlockPos) -> f64 {
        0.0
    }

    fn depth(&mut self, _position: BlockPos) -> f64 {
        0.0
    }

    fn floodedness(&mut self, _position: BlockPos) -> f64 {
        1.0
    }

    fn spread(&mut self, _cell: [i32; 3]) -> f64 {
        0.0
    }

    fn lava(&mut self, _cell: [i32; 3]) -> f64 {
        0.0
    }

    fn barrier(&mut self, _position: BlockPos) -> f64 {
        0.0
    }
}
