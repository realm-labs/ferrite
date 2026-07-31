use std::cell::RefCell;
use std::num::NonZeroU32;

use ferrite_world::generation::feature::random::{GenerationRandom, LegacyRandom};
use ferrite_world::generation::noise::{
    ImprovedNoise, NoiseError, NoiseParameters, NormalNoise, PerlinNoise, wrap,
};

#[test]
fn improved_noise_draws_offsets_then_forward_shuffle_bounds() {
    let mut random = ZeroRandom::default();
    let noise = ImprovedNoise::new(&mut random);

    assert_eq!([noise.x_offset, noise.y_offset, noise.z_offset], [0.0; 3]);
    assert_eq!(random.double_draws, 3);
    assert_eq!(random.bounds.len(), 256);
    assert_eq!(random.bounds[0], 256);
    assert_eq!(random.bounds[255], 1);
    assert_eq!(noise.permutation()[0], 0);
    assert_eq!(noise.permutation()[255], 255);
    assert_eq!(noise.sample(0.0, 0.0, 0.0), 0.0);
    assert_eq!(noise.sample(256.0, 0.0, 0.0), 0.0);
}

#[test]
fn keyed_perlin_allocates_only_nonzero_octaves_in_index_order() {
    let octaves = RefCell::new(Vec::new());
    let noise = PerlinNoise::keyed(
        NoiseParameters {
            first_octave: -2,
            amplitudes: vec![1.0, 0.0, 2.0],
        },
        |octave| {
            octaves.borrow_mut().push(octave);
            LegacyRandom::new(i64::from(octave))
        },
    );

    assert_eq!(*octaves.borrow(), [-2, 0]);
    assert!(noise.levels()[0].is_some());
    assert!(noise.levels()[1].is_none());
    assert!(noise.levels()[2].is_some());
}

#[test]
fn legacy_zero_amplitude_gap_consumes_exactly_262_unbounded_ints() {
    let mut actual = LegacyRandom::new(7);
    PerlinNoise::legacy(
        &mut actual,
        NoiseParameters {
            first_octave: -1,
            amplitudes: vec![0.0, 1.0],
        },
    )
    .unwrap();

    let mut expected = LegacyRandom::new(7);
    let _ = ImprovedNoise::new(&mut expected);
    for _ in 0..262 {
        let _ = expected.next_i32();
    }
    assert_eq!(actual.next_i32(), expected.next_i32());
}

#[test]
fn legacy_positive_octaves_fail_after_the_zero_octave_allocation() {
    let mut random = LegacyRandom::new(3);
    let error = PerlinNoise::legacy(
        &mut random,
        NoiseParameters {
            first_octave: 0,
            amplitudes: vec![1.0, 0.0],
        },
    )
    .unwrap_err();

    assert_eq!(error, NoiseError::PositiveLegacyOctave);
}

#[test]
fn all_zero_normal_noise_has_zero_value_and_maximum() {
    let normal = NormalNoise::keyed(
        NoiseParameters {
            first_octave: -3,
            amplitudes: vec![0.0, -0.0],
        },
        |_, _| panic!("zero amplitude must not allocate an octave"),
    );

    assert_eq!(normal.sample(12.0, -4.0, 7.0), 0.0);
    assert_eq!(normal.maximum(), 0.0);
}

#[test]
fn wrap_uses_nearest_period_with_positive_half_tie() {
    let period = 33_554_432.0;
    assert_eq!(wrap(period / 2.0), -period / 2.0);
    assert_eq!(wrap(-period / 2.0), -period / 2.0);
    assert_eq!(wrap(period + 3.0), 3.0);
}

#[derive(Debug, Default)]
struct ZeroRandom {
    bounds: Vec<u32>,
    double_draws: usize,
}

impl GenerationRandom for ZeroRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        0
    }

    fn next_f32(&mut self) -> f32 {
        0.0
    }

    fn next_f64(&mut self) -> f64 {
        self.double_draws += 1;
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
