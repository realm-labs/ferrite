use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::sync::Arc;

use ferrite_world::generation::carver_path::{
    CanyonPathConfig, CanyonShapeConfig, CarverFloatProvider, CarverHeightProvider, CarverRandom,
    CarverSkip, CaveFamily, CavePathConfig, carve_canyon_source, carve_cave_source,
};
use ferrite_world::generation::feature::provider::{HeightAnchor, HeightContext};
use ferrite_world::generation::feature::random::GenerationRandom;

#[test]
fn cave_start_equality_is_admitted_and_nested_count_bounds_are_ordered() {
    let mut random = ScriptedRandom::new([4, 2, 0], [0.15], []);
    let mut volumes = Vec::new();

    assert!(
        carve_cave_source(
            &mut random,
            height(),
            [0, 0],
            [0, 0],
            &cave_config(0.15),
            |volume| volumes.push(volume),
        )
        .unwrap()
    );

    assert_eq!(random.bounds, [15, 5, 3]);
    assert!(volumes.is_empty());
}

#[test]
fn cave_room_precedes_tunnel_and_outer_stream_keeps_exact_draw_order() {
    let integers = [1, 1, 1, 0, 0, 0, 0, 1, 0];
    let floats = [0.0, 0.0, 0.0, 0.5, 0.0, 0.0];
    let mut random = ScriptedRandom::new(integers, floats, [7]);
    let mut volumes = Vec::new();

    assert!(
        carve_cave_source(
            &mut random,
            height(),
            [0, 0],
            [0, 0],
            &cave_config(1.0),
            |volume| volumes.push(volume),
        )
        .unwrap()
    );

    assert_eq!(random.bounds, [15, 2, 2, 16, 16, 4, 4, 10, 28]);
    assert_eq!(random.long_draws, 1);
    let room = &volumes[0].ellipsoid;
    assert_eq!(
        [room.center_x, room.center_y, room.center_z],
        [1.0, 20.0, 0.0]
    );
    assert_eq!(room.horizontal_radius, 2.5);
    assert_eq!(room.vertical_radius, 1.25);
}

#[test]
fn canyon_zero_length_still_consumes_all_outer_parameters_and_seed() {
    let mut random = ScriptedRandom::new([3, 4], [0.01, 0.25], [11]);
    let mut volumes = Vec::new();
    let config = CanyonPathConfig {
        probability: 0.01,
        y: CarverHeightProvider::Constant(HeightAnchor::Absolute(30)),
        vertical_rotation: CarverFloatProvider::Constant(0.0),
        y_scale: CarverFloatProvider::Constant(3.0),
        shape: CanyonShapeConfig {
            distance_factor: CarverFloatProvider::Constant(0.0),
            thickness: CarverFloatProvider::Constant(2.0),
            horizontal_radius_factor: CarverFloatProvider::Constant(1.0),
            vertical_radius_default_factor: 1.0,
            vertical_radius_center_factor: 0.0,
            width_smoothness: 3,
        },
    };

    assert!(
        carve_canyon_source(&mut random, height(), [2, -1], [0, 0], &config, |volume| {
            volumes.push(volume)
        },)
        .unwrap()
    );

    assert_eq!(random.bounds, [16, 16]);
    assert_eq!(random.float_draws, 2);
    assert_eq!(random.long_draws, 1);
    assert!(volumes.is_empty());
}

#[test]
fn cave_and_canyon_skip_equalities_are_excluded() {
    let cave = CarverSkip::Cave { floor_level: -0.4 };
    assert!(cave.should_skip(0.0, -0.4, 0.0, 0));
    assert!(cave.should_skip(1.0, 0.0, 0.0, 0));
    assert!(!cave.should_skip(0.5, 0.0, 0.0, 0));

    let canyon = CarverSkip::Canyon {
        minimum_y: 0,
        width_factors: Arc::from([1.0]),
    };
    assert!(canyon.should_skip(1.0, 0.0, 0.0, 1));
    assert!(!canyon.should_skip(0.0, 0.0, 0.0, 1));
    assert!(canyon.should_skip(0.0, 0.0, 0.0, 0));
}

fn height() -> HeightContext {
    HeightContext {
        minimum_y: 0,
        depth: 64,
    }
}

fn cave_config(probability: f32) -> CavePathConfig {
    CavePathConfig {
        probability,
        y: CarverHeightProvider::Constant(HeightAnchor::Absolute(20)),
        horizontal_radius_multiplier: CarverFloatProvider::Constant(1.0),
        vertical_radius_multiplier: CarverFloatProvider::Constant(1.0),
        floor_level: CarverFloatProvider::Constant(-0.7),
        y_scale: CarverFloatProvider::Constant(0.5),
        family: CaveFamily::Ordinary,
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    floats: VecDeque<f32>,
    longs: VecDeque<i64>,
    bounds: Vec<u32>,
    float_draws: usize,
    long_draws: usize,
}

impl ScriptedRandom {
    fn new(
        integers: impl IntoIterator<Item = u32>,
        floats: impl IntoIterator<Item = f32>,
        longs: impl IntoIterator<Item = i64>,
    ) -> Self {
        Self {
            integers: integers.into_iter().collect(),
            floats: floats.into_iter().collect(),
            longs: longs.into_iter().collect(),
            bounds: Vec::new(),
            float_draws: 0,
            long_draws: 0,
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("unexpected integer draw");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        self.float_draws += 1;
        self.floats.pop_front().expect("unexpected float draw")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("unexpected double draw")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("unexpected Gaussian draw")
    }
}

impl CarverRandom for ScriptedRandom {
    fn next_i64(&mut self) -> i64 {
        self.long_draws += 1;
        self.longs.pop_front().expect("unexpected long draw")
    }
}
