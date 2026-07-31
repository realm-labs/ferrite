use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::aquifer::AquiferResolver;
use ferrite_world::generation::noise_fill::{
    NoiseFillWorld, NoiseMaterial, NoiseSettings, fill_noise_chunk, resolve_material,
};
use ferrite_world::id::BlockStateId;

#[test]
fn nonpositive_clamped_cell_count_completes_without_locking_or_lifecycle() {
    let mut world = Fixture {
        minimum_y: 100,
        maximum_y: 110,
        ..Fixture::default()
    };

    assert!(!fill_noise_chunk(&mut world, settings()).unwrap());
    assert!(world.events.is_empty());
}

#[test]
fn fill_locks_top_down_and_traverses_y_then_x_then_z_inside_the_cell() {
    let mut world = Fixture::default();

    assert!(fill_noise_chunk(&mut world, settings()).unwrap());

    assert_eq!(world.acquired, [0]);
    assert_eq!(world.released, [0]);
    assert_eq!(
        &world.materials[..4],
        [
            BlockPos::new(0, 15, 0),
            BlockPos::new(0, 15, 1),
            BlockPos::new(0, 15, 2),
            BlockPos::new(0, 15, 3),
        ]
    );
    assert_eq!(world.materials[16], BlockPos::new(1, 15, 0));
    assert_eq!(world.materials[256], BlockPos::new(0, 14, 0));
    assert_eq!(world.materials.len(), 16 * 16 * 16);
    assert_eq!(world.events.first().map(String::as_str), Some("acquire"));
    assert_eq!(world.events.last().map(String::as_str), Some("release"));
}

#[test]
fn fill_error_still_stops_interpolation_and_releases_every_section() {
    let mut world = Fixture {
        fail_material: true,
        ..Fixture::default()
    };

    assert_eq!(fill_noise_chunk(&mut world, settings()), Err("material"));

    assert!(world.events.contains(&"stop".to_owned()));
    assert_eq!(world.released, [0]);
    assert_eq!(world.events.last().map(String::as_str), Some("release"));
}

#[test]
fn material_resolution_is_aquifer_then_ore_then_default() {
    let position = BlockPos::new(0, 0, 0);
    let mut aquifer = MockAquifer {
        result: Some(WATER),
        schedule: true,
    };
    let mut ore_calls = 0;
    assert_eq!(
        resolve_material(
            &mut aquifer,
            position,
            -1.0,
            || {
                ore_calls += 1;
                Some(ORE)
            },
            STONE,
        ),
        NoiseMaterial {
            state: WATER,
            schedule_fluid_update: true,
        }
    );
    assert_eq!(ore_calls, 0);

    aquifer.result = None;
    assert_eq!(
        resolve_material(&mut aquifer, position, 1.0, || Some(ORE), STONE).state,
        ORE
    );
    assert_eq!(
        resolve_material(&mut aquifer, position, 1.0, || None, STONE).state,
        STONE
    );
}

fn settings() -> NoiseSettings {
    NoiseSettings {
        minimum_y: 0,
        height: 16,
        horizontal_size: 4,
        vertical_size: 4,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const WATER: BlockStateId = BlockStateId::new(2);
const ORE: BlockStateId = BlockStateId::new(3);

#[derive(Debug)]
struct Fixture {
    minimum_y: i32,
    maximum_y: i32,
    acquired: Vec<i32>,
    released: Vec<i32>,
    events: Vec<String>,
    materials: Vec<BlockPos>,
    fail_material: bool,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            minimum_y: 0,
            maximum_y: 15,
            acquired: Vec::new(),
            released: Vec::new(),
            events: Vec::new(),
            materials: Vec::new(),
            fail_material: false,
        }
    }
}

impl NoiseFillWorld for Fixture {
    type Error = &'static str;

    fn accessor_minimum_y(&self) -> i32 {
        self.minimum_y
    }

    fn accessor_maximum_y(&self) -> i32 {
        self.maximum_y
    }

    fn chunk_minimum_x(&self) -> i32 {
        0
    }

    fn chunk_minimum_z(&self) -> i32 {
        0
    }

    fn acquire_section(&mut self, section_y: i32) -> Result<(), Self::Error> {
        self.acquired.push(section_y);
        self.events.push("acquire".to_owned());
        Ok(())
    }

    fn release_section(&mut self, section_y: i32) {
        self.released.push(section_y);
        self.events.push("release".to_owned());
    }

    fn start_interpolation(&mut self) -> Result<(), Self::Error> {
        self.events.push("start".to_owned());
        Ok(())
    }

    fn advance_cell_x(&mut self, _cell_x: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn select_cell(&mut self, _cell_y: i32, _cell_z: i32) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_for_y(&mut self, _y_fraction: f64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_for_x(&mut self, _x_fraction: f64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_for_z(&mut self, _z_fraction: f64) -> Result<(), Self::Error> {
        Ok(())
    }

    fn material(&mut self, position: BlockPos) -> Result<NoiseMaterial, Self::Error> {
        self.materials.push(position);
        if self.fail_material {
            Err("material")
        } else {
            Ok(NoiseMaterial {
                state: STONE,
                schedule_fluid_update: false,
            })
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool {
        state == WATER
    }

    fn write_block(
        &mut self,
        _position: BlockPos,
        _state: BlockStateId,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn update_ocean_floor_heightmap(&mut self, _position: BlockPos, _state: BlockStateId) {}

    fn update_world_surface_heightmap(&mut self, _position: BlockPos, _state: BlockStateId) {}

    fn mark_for_postprocessing(&mut self, _position: BlockPos) {}

    fn swap_slices(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn stop_interpolation(&mut self) -> Result<(), Self::Error> {
        self.events.push("stop".to_owned());
        Ok(())
    }
}

#[derive(Debug)]
struct MockAquifer {
    result: Option<BlockStateId>,
    schedule: bool,
}

impl AquiferResolver for MockAquifer {
    fn compute_substance(&mut self, _position: BlockPos, _density: f64) -> Option<BlockStateId> {
        self.result
    }

    fn should_schedule_fluid_update(&self) -> bool {
        self.schedule
    }
}
