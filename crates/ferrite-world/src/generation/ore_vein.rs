//! Ordered ore-vein material selection.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OreVeinStates {
    pub copper_ore: BlockStateId,
    pub raw_copper: BlockStateId,
    pub granite: BlockStateId,
    pub deepslate_iron_ore: BlockStateId,
    pub raw_iron: BlockStateId,
    pub tuff: BlockStateId,
    pub debug_air: BlockStateId,
    pub debug_filler: BlockStateId,
}

pub trait OreVeinNoise {
    fn toggle(&mut self, position: BlockPos) -> f64;

    fn ridged(&mut self, position: BlockPos) -> f64;

    fn gap(&mut self, position: BlockPos) -> f64;
}

pub fn resolve_ore_vein<N, R>(
    noise: &mut N,
    position: BlockPos,
    states: OreVeinStates,
    debug: bool,
    mut random_at: impl FnMut(BlockPos) -> R,
) -> Option<BlockStateId>
where
    N: OreVeinNoise,
    R: GenerationRandom,
{
    let toggle = noise.toggle(position);
    let vein = if toggle > 0.0 {
        Vein {
            minimum_y: 0,
            maximum_y: 50,
            ore: states.copper_ore,
            raw: states.raw_copper,
            filler: states.granite,
        }
    } else {
        Vein {
            minimum_y: -60,
            maximum_y: -8,
            ore: states.deepslate_iron_ore,
            raw: states.raw_iron,
            filler: states.tuff,
        }
    };
    let early_default = debug.then_some(states.debug_air);
    let distance_from_top = vein.maximum_y - position.y;
    let distance_from_bottom = position.y - vein.minimum_y;
    if distance_from_bottom < 0 || distance_from_top < 0 {
        return early_default;
    }
    let distance_from_edge = distance_from_top.min(distance_from_bottom);
    let roundoff = clamped_map(f64::from(distance_from_edge), 0.0, 20.0, -0.2, 0.0);
    let veininess = toggle.abs();
    if veininess + roundoff < f64::from(0.4_f32) {
        return early_default;
    }
    let mut random = random_at(position);
    if random.next_f32() > 0.7 {
        return early_default;
    }
    if noise.ridged(position) >= 0.0 {
        return early_default;
    }
    let richness = clamped_map(
        veininess,
        f64::from(0.4_f32),
        f64::from(0.6_f32),
        f64::from(0.1_f32),
        f64::from(0.3_f32),
    );
    if f64::from(random.next_f32()) < richness && noise.gap(position) > f64::from(-0.3_f32) {
        return Some(if random.next_f32() < 0.02 {
            vein.raw
        } else {
            vein.ore
        });
    }
    Some(if debug {
        states.debug_filler
    } else {
        vein.filler
    })
}

#[derive(Debug, Clone, Copy)]
struct Vein {
    minimum_y: i32,
    maximum_y: i32,
    ore: BlockStateId,
    raw: BlockStateId,
    filler: BlockStateId,
}

fn clamped_map(
    value: f64,
    from_minimum: f64,
    from_maximum: f64,
    to_minimum: f64,
    to_maximum: f64,
) -> f64 {
    if value <= from_minimum {
        to_minimum
    } else if value >= from_maximum {
        to_maximum
    } else {
        to_minimum
            + (value - from_minimum) / (from_maximum - from_minimum) * (to_maximum - to_minimum)
    }
}
