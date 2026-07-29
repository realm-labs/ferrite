//! Cobweb movement residue and low-profile collision joins.

pub const COBWEB_STATE_ID: u32 = 2_247;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StuckMultiplier {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

pub fn cobweb_contact(living_with_weaving: bool, spider: bool) -> Option<StuckMultiplier> {
    if spider {
        return None;
    }
    if living_with_weaving {
        Some(StuckMultiplier {
            x: 0.5,
            y: 0.25,
            z: 0.5,
        })
    } else {
        Some(StuckMultiplier {
            x: 0.25,
            y: 0.05000000074505806,
            z: 0.25,
        })
    }
}

pub fn apply_deferred_stuck(
    displacement: [f64; 3],
    multiplier: StuckMultiplier,
    piston_move: bool,
) -> [f64; 3] {
    if piston_move {
        displacement
    } else {
        [
            displacement[0] * multiplier.x,
            displacement[1] * multiplier.y,
            displacement[2] * multiplier.z,
        ]
    }
}
