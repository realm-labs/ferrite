use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::biome::{
    BiomeSource, ClimateInterval, ClimatePoint, ClimateSampler, ClosestBiomeQuery, EndBiomes,
    HorizontalBiomeQuery, MultiNoiseCache,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BiomeId;

#[test]
fn checkerboard_masks_large_shift_counts_like_java() {
    let source = BiomeSource::Checkerboard {
        biomes: vec![BIOME_A, BIOME_B],
        scale: 30,
    };
    let mut sampler = Sampler::default();
    let mut cache = MultiNoiseCache::default();

    assert_eq!(
        source.sample(1, 99, 0, &mut sampler, &mut cache).unwrap(),
        BIOME_B
    );
}

#[test]
fn multi_noise_strict_tie_retains_the_cached_leaf() {
    let point = |value, biome| ClimatePoint {
        parameters: std::array::from_fn(|_| ClimateInterval::quantized(value, value)),
        offset: 0,
        biome,
    };
    let source = BiomeSource::MultiNoise {
        points: vec![point(0.0, BIOME_A), point(1.0, BIOME_B)],
    };
    let mut sampler = Sampler {
        climate: [1.0; 6],
        ..Sampler::default()
    };
    let mut cache = MultiNoiseCache::default();
    assert_eq!(
        source.sample(0, 0, 0, &mut sampler, &mut cache).unwrap(),
        BIOME_B
    );

    sampler.climate = [0.5; 6];
    assert_eq!(
        source.sample(0, 0, 0, &mut sampler, &mut cache).unwrap(),
        BIOME_B
    );
}

#[test]
fn end_thresholds_preserve_strict_and_inclusive_boundaries() {
    let source = BiomeSource::TheEnd(EndBiomes {
        center: BIOME_A,
        highlands: BIOME_B,
        midlands: BIOME_C,
        small_islands: BIOME_D,
        barrens: BIOME_E,
    });
    let mut cache = MultiNoiseCache::default();
    let mut sampler = Sampler {
        erosion: -0.0625,
        ..Sampler::default()
    };

    assert_eq!(
        source.sample(400, 0, 0, &mut sampler, &mut cache).unwrap(),
        BIOME_C
    );
    sampler.erosion = -0.21875;
    assert_eq!(
        source.sample(400, 0, 0, &mut sampler, &mut cache).unwrap(),
        BIOME_E
    );
}

#[test]
fn horizontal_reservoir_skips_the_first_draw_then_draws_for_every_later_match() {
    let source = BiomeSource::Checkerboard {
        biomes: vec![BIOME_A],
        scale: 2,
    };
    let mut sampler = Sampler::default();
    let mut cache = MultiNoiseCache::default();
    let mut random = ScriptedRandom::new(1..=8);

    let result = source
        .find_horizontal(
            HorizontalBiomeQuery {
                center: BlockPos::new(0, 20, 0),
                radius: 4,
                quart_step: 1,
                closest: false,
            },
            &mut sampler,
            &mut cache,
            &mut random,
            |_| true,
        )
        .unwrap();

    assert_eq!(result, Some((BlockPos::new(-4, 20, -4), BIOME_A)));
    assert_eq!(random.bounds, [2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn fixed_closest_3d_clamps_y_without_sampling_climate() {
    let source = BiomeSource::Fixed(BIOME_A);
    let mut sampler = Sampler::default();
    let mut cache = MultiNoiseCache::default();

    let result = source
        .find_closest_3d(
            ClosestBiomeQuery {
                origin: BlockPos::new(4, -100, 6),
                radius: 64,
                horizontal_step: 4,
                vertical_step: 8,
                minimum_y: -64,
                maximum_y: 319,
            },
            &mut sampler,
            &mut cache,
            |_| true,
        )
        .unwrap();

    assert_eq!(result, Some((BlockPos::new(4, -63, 6), BIOME_A)));
    assert_eq!(sampler.climate_calls, 0);
}

const BIOME_A: BiomeId = BiomeId::new(1);
const BIOME_B: BiomeId = BiomeId::new(2);
const BIOME_C: BiomeId = BiomeId::new(3);
const BIOME_D: BiomeId = BiomeId::new(4);
const BIOME_E: BiomeId = BiomeId::new(5);

#[derive(Debug)]
struct Sampler {
    climate: [f32; 6],
    erosion: f64,
    climate_calls: usize,
}

impl Default for Sampler {
    fn default() -> Self {
        Self {
            climate: [0.0; 6],
            erosion: 0.0,
            climate_calls: 0,
        }
    }
}

impl ClimateSampler for Sampler {
    fn sample_climate(&mut self, _quart_x: i32, _quart_y: i32, _quart_z: i32) -> [f32; 6] {
        self.climate_calls += 1;
        self.climate
    }

    fn sample_end_erosion(&mut self, _block_x: i32, _block_y: i32, _block_z: i32) -> f64 {
        self.erosion
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: values.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("fixture does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fixture does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fixture does not draw Gaussian values")
    }
}
