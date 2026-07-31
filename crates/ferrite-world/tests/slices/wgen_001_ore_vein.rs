use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::ore_vein::{OreVeinNoise, OreVeinStates, resolve_ore_vein};
use ferrite_world::id::BlockStateId;

#[test]
fn out_of_band_returns_before_random_factory_or_secondary_noise() {
    let position = BlockPos::new(1, 51, 2);
    let mut noise = Noise::new(1.0, -1.0, 1.0);
    let mut factories = 0;

    assert_eq!(
        resolve_ore_vein(&mut noise, position, states(), false, |_| {
            factories += 1;
            ScriptedRandom::new([])
        }),
        None
    );

    assert_eq!(noise.calls, ["toggle"]);
    assert_eq!(factories, 0);
}

#[test]
fn solidness_equality_survives_and_first_float_equality_reaches_ridged() {
    let position = BlockPos::new(0, 20, 0);
    let mut noise = Noise::new(f64::from(0.4_f32), -1.0, -1.0);

    let result = resolve_ore_vein(&mut noise, position, states(), false, |_| {
        ScriptedRandom::new([0.7, 1.0])
    });

    assert_eq!(result, Some(GRANITE));
    assert_eq!(noise.calls, ["toggle", "ridged"]);
}

#[test]
fn richness_draw_is_strict_and_failed_equality_skips_gap() {
    let position = BlockPos::new(0, 25, 0);
    let mut noise = Noise::new(f64::from(0.5_f32), -1.0, 1.0);

    let result = resolve_ore_vein(&mut noise, position, states(), false, |_| {
        ScriptedRandom::new([0.0, 0.2])
    });

    assert_eq!(result, Some(GRANITE));
    assert_eq!(noise.calls, ["toggle", "ridged"]);
}

#[test]
fn gap_and_raw_thresholds_are_strict_and_success_is_not_debug_mapped() {
    let position = BlockPos::new(0, 25, 0);
    let mut noise = Noise::new(1.0, -1.0, f64::from(-0.3_f32) + f64::EPSILON);

    let result = resolve_ore_vein(&mut noise, position, states(), true, |_| {
        ScriptedRandom::new([0.0, 0.0, 0.019])
    });

    assert_eq!(result, Some(RAW_COPPER));
    assert_eq!(noise.calls, ["toggle", "ridged", "gap"]);
}

const COPPER: BlockStateId = BlockStateId::new(1);
const RAW_COPPER: BlockStateId = BlockStateId::new(2);
const GRANITE: BlockStateId = BlockStateId::new(3);
const IRON: BlockStateId = BlockStateId::new(4);
const RAW_IRON: BlockStateId = BlockStateId::new(5);
const TUFF: BlockStateId = BlockStateId::new(6);
const AIR: BlockStateId = BlockStateId::new(7);
const BUTTON: BlockStateId = BlockStateId::new(8);

fn states() -> OreVeinStates {
    OreVeinStates {
        copper_ore: COPPER,
        raw_copper: RAW_COPPER,
        granite: GRANITE,
        deepslate_iron_ore: IRON,
        raw_iron: RAW_IRON,
        tuff: TUFF,
        debug_air: AIR,
        debug_filler: BUTTON,
    }
}

#[derive(Debug)]
struct Noise {
    toggle: f64,
    ridged: f64,
    gap: f64,
    calls: Vec<&'static str>,
}

impl Noise {
    fn new(toggle: f64, ridged: f64, gap: f64) -> Self {
        Self {
            toggle,
            ridged,
            gap,
            calls: Vec::new(),
        }
    }
}

impl OreVeinNoise for Noise {
    fn toggle(&mut self, _position: BlockPos) -> f64 {
        self.calls.push("toggle");
        self.toggle
    }

    fn ridged(&mut self, _position: BlockPos) -> f64 {
        self.calls.push("ridged");
        self.ridged
    }

    fn gap(&mut self, _position: BlockPos) -> f64 {
        self.calls.push("gap");
        self.gap
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    floats: VecDeque<f32>,
}

impl ScriptedRandom {
    fn new(floats: impl IntoIterator<Item = f32>) -> Self {
        Self {
            floats: floats.into_iter().collect(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, _bound: NonZeroU32) -> u32 {
        panic!("unexpected integer draw")
    }

    fn next_f32(&mut self) -> f32 {
        self.floats.pop_front().expect("unexpected float draw")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("unexpected double draw")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("unexpected Gaussian draw")
    }
}
