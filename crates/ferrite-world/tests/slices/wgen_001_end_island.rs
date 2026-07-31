use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_world::generation::end_island::{EndIslandDensity, SimplexNoise};
use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};

#[test]
fn central_height_and_declared_bounds_are_exact() {
    let density = EndIslandDensity::new(0);

    assert_eq!(density.sample(0, 0), 0.5625);
    assert_eq!(density.sample(-7, 0), density.sample(0, 0));
    assert_eq!(density.bounds(), (-0.84375, 0.5625));
}

#[test]
fn end_constructor_consumes_17292_unbounded_ints_before_simplex() {
    let density = EndIslandDensity::new(42);
    let mut expected = LegacyRandom::new(42);
    for _ in 0..17_292 {
        let _ = expected.next_i32();
    }
    let expected = SimplexNoise::new(&mut expected);

    assert_eq!(
        [
            density.simplex().x_offset,
            density.simplex().y_offset,
            density.simplex().z_offset,
        ],
        [expected.x_offset, expected.y_offset, expected.z_offset]
    );
    assert_eq!(density.simplex().permutation(), expected.permutation());
}

#[test]
fn simplex_2d_ignores_all_three_constructor_offsets() {
    let mut first = ScriptedRandom::new([0.0, 0.0, 0.0]);
    let mut second = ScriptedRandom::new([0.25, 0.5, 0.75]);
    let first = SimplexNoise::new(&mut first);
    let second = SimplexNoise::new(&mut second);

    assert_eq!(first.permutation(), second.permutation());
    assert_eq!(first.sample_2d(1.25, 1.25), second.sample_2d(1.25, 1.25));
}

#[derive(Debug)]
struct ScriptedRandom {
    doubles: VecDeque<f64>,
}

impl ScriptedRandom {
    fn new(doubles: impl IntoIterator<Item = f64>) -> Self {
        Self {
            doubles: doubles.into_iter().collect(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        0
    }

    fn next_f32(&mut self) -> f32 {
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        self.doubles.pop_front().expect("unexpected double draw")
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
