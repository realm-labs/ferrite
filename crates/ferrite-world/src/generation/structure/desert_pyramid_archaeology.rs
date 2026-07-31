//! Global deterministic archaeology selection for completed desert pyramids.

use std::collections::BTreeSet;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::desert_pyramid::{DesertPyramidPiece, DesertPyramidWorld};
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::StructureState;

pub trait DesertArchaeologyWorld: DesertPyramidWorld {
    fn is_brushable_block_entity(&mut self, position: BlockPos) -> bool;

    fn install_archaeology_loot(&mut self, position: BlockPos, table: &str, seed: i64);
}

pub fn place_desert_archaeology(
    world: &mut impl DesertArchaeologyWorld,
    pieces: &[DesertPyramidPiece],
    clip: &BlockBox,
) {
    for piece in pieces {
        place_suspicious_sand(world, piece.collapsed_roof_position, clip);
    }
    let mut unique = BTreeSet::new();
    for piece in pieces {
        unique.extend(piece.archaeology_candidates.iter().copied());
    }
    let mut candidates = unique.into_iter().collect::<Vec<_>>();
    candidates.sort_by_key(|position| (position.y, position.z, position.x));
    let Some(center) = pieces.first().map(|piece| piece.piece.bounds.center()) else {
        return;
    };
    let mut random = LegacyRandom::new(world.positional_seed(center));
    shuffle(&mut candidates, &mut random);
    let mut suspicious =
        5 + random.next_u32(NonZeroU32::new(3).expect("three archaeology counts")) as usize;
    suspicious = suspicious.min(candidates.len());
    for position in candidates {
        if suspicious > 0 {
            suspicious -= 1;
            place_suspicious_sand(world, position, clip);
        } else if clip.contains(position) {
            PieceWorld::set_state(world, position, StructureState::new("minecraft:sand"), 2);
        }
    }
}

fn place_suspicious_sand(
    world: &mut impl DesertArchaeologyWorld,
    position: BlockPos,
    clip: &BlockBox,
) {
    if !clip.contains(position) {
        return;
    }
    PieceWorld::set_state(
        world,
        position,
        StructureState::new("minecraft:suspicious_sand"),
        2,
    );
    if world.is_brushable_block_entity(position) {
        world.install_archaeology_loot(
            position,
            "minecraft:archaeology/desert_pyramid",
            block_position_seed(position),
        );
    }
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for length in (2..=values.len()).rev() {
        let index = random.next_u32(
            NonZeroU32::new(u32::try_from(length).expect("candidate length fits u32"))
                .expect("shuffle length is at least two"),
        ) as usize;
        values.swap(length - 1, index);
    }
}

fn block_position_seed(position: BlockPos) -> i64 {
    ((i64::from(position.x) & 0x3ff_ffff) << 38)
        | ((i64::from(position.z) & 0x3ff_ffff) << 12)
        | (i64::from(position.y) & 0xfff)
}
