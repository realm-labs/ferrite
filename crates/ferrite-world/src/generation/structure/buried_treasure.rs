//! Buried-treasure support scan, enclosure, and chest transaction.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::{
    HorizontalDirection, OrientedPiece, PiecePlacement, PieceWorld,
};
use crate::generation::structure::processor::StructureState;

pub trait BuriedTreasureWorld: PieceWorld {
    fn ocean_floor_height(&mut self, x: i32, z: i32) -> i32;

    fn minimum_y(&self) -> i32;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuriedTreasureResult {
    pub final_box: Option<BlockBox>,
    pub chest_attempted: bool,
}

pub fn place_buried_treasure<F>(
    world: &mut impl BuriedTreasureWorld,
    x: i32,
    z: i32,
    processing_box: &BlockBox,
    loot_seed: F,
) -> BuriedTreasureResult
where
    F: FnOnce() -> i64,
{
    let mut candidate = BlockPos {
        x,
        y: world.ocean_floor_height(x, z),
        z,
    };
    while candidate.y > world.minimum_y() {
        let current = world.state_at(candidate);
        let below = offset(candidate, Direction::Down);
        let support = world.state_at(below);
        if is_support(&support.block) {
            let enclosure = if is_empty_or_liquid(&current.block) {
                StructureState::new("minecraft:sand")
            } else {
                current
            };
            enclose(world, candidate, &support, &enclosure);
            let piece = OrientedPiece {
                bounds: BlockBox::point(candidate),
                orientation: HorizontalDirection::South,
            };
            let chest_attempted = PiecePlacement {
                piece,
                clip: processing_box,
            }
            .create_chest(
                world,
                BlockPos { x: 0, y: 0, z: 0 },
                "minecraft:chests/buried_treasure",
                loot_seed,
            );
            return BuriedTreasureResult {
                final_box: Some(BlockBox::point(candidate)),
                chest_attempted,
            };
        }
        candidate.y -= 1;
    }
    BuriedTreasureResult {
        final_box: None,
        chest_attempted: false,
    }
}

fn enclose(
    world: &mut impl PieceWorld,
    candidate: BlockPos,
    support: &StructureState,
    enclosure: &StructureState,
) {
    for direction in Direction::ALL {
        let neighbor = offset(candidate, direction);
        if !is_empty_or_liquid(&world.state_at(neighbor).block) {
            continue;
        }
        let below_neighbor = offset(neighbor, Direction::Down);
        let below_is_empty = is_empty_or_liquid(&world.state_at(below_neighbor).block);
        let state = if below_is_empty && direction != Direction::Up {
            support
        } else {
            enclosure
        };
        world.set_state(neighbor, state.clone(), 3);
    }
}

fn is_support(block: &str) -> bool {
    matches!(
        block,
        "minecraft:sandstone"
            | "minecraft:stone"
            | "minecraft:andesite"
            | "minecraft:granite"
            | "minecraft:diorite"
    )
}

fn is_empty_or_liquid(block: &str) -> bool {
    matches!(
        block,
        "minecraft:air" | "minecraft:water" | "minecraft:lava"
    )
}

fn offset(position: BlockPos, direction: Direction) -> BlockPos {
    let [x, y, z] = direction.step();
    BlockPos {
        x: position.x + x,
        y: position.y + y,
        z: position.z + z,
    }
}
