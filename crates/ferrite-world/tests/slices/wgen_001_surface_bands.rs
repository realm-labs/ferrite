use std::num::NonZeroU32;

use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::surface_bands::{ClayBandError, ClayBandStates, ClayBands};
use ferrite_world::id::BlockStateId;

#[test]
fn generation_uses_orange_then_three_color_runs_then_white_stream() {
    let mut random = ZeroRandom::default();
    let bands = ClayBands::generate(&mut random, states());

    assert_eq!(&random.bounds[..4], [5, 5, 5, 5]);
    assert!(random.bounds.contains(&10));
    assert!(random.bounds.contains(&192));
    assert!(random.bounds.contains(&7));
    assert_eq!(bands.values()[0], WHITE);
    assert_eq!(bands.values()[4], WHITE);
    assert_eq!(bands.values()[8], WHITE);
}

#[test]
fn lookup_uses_java_round_and_remainder_instead_of_euclidean_modulo() {
    let mut random = ZeroRandom::default();
    let bands = ClayBands::generate(&mut random, states());

    assert_eq!(bands.state(0, -0.125).unwrap(), bands.values()[0]);
    assert_eq!(bands.state(0, 0.125).unwrap(), bands.values()[1]);
    assert_eq!(bands.state(-192, 0.0), Ok(bands.values()[0]));
    assert_eq!(bands.state(-193, 0.0), Err(ClayBandError::NegativeIndex));
}

const TERRACOTTA: BlockStateId = BlockStateId::new(1);
const ORANGE: BlockStateId = BlockStateId::new(2);
const YELLOW: BlockStateId = BlockStateId::new(3);
const BROWN: BlockStateId = BlockStateId::new(4);
const RED: BlockStateId = BlockStateId::new(5);
const WHITE: BlockStateId = BlockStateId::new(6);
const LIGHT_GRAY: BlockStateId = BlockStateId::new(7);

fn states() -> ClayBandStates {
    ClayBandStates {
        terracotta: TERRACOTTA,
        orange: ORANGE,
        yellow: YELLOW,
        brown: BROWN,
        red: RED,
        white: WHITE,
        light_gray: LIGHT_GRAY,
    }
}

#[derive(Debug, Default)]
struct ZeroRandom {
    bounds: Vec<u32>,
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
        0.0
    }

    fn next_gaussian(&mut self) -> f64 {
        0.0
    }
}
