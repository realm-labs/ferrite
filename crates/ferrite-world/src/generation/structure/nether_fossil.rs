//! Nether-fossil anchor search, piece choice, and dried-ghast postpass.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::{GenerationRandom, LegacyRandom};
use crate::generation::structure::BlockBox;
use crate::generation::structure::piece::PieceWorld;
use crate::generation::structure::processor::StructureState;

pub trait NetherFossilWorld: PieceWorld {
    fn sea_level(&self) -> i32;

    fn generation_top(&self) -> i32;

    fn base_column_state(&mut self, x: i32, y: i32, z: i32) -> StructureState;

    fn sturdy_upper_face(&mut self, state: &StructureState) -> bool;

    fn positional_seed(&self, position: BlockPos) -> i64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FossilRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetherFossilChoice {
    pub anchor: BlockPos,
    pub rotation: FossilRotation,
    pub template: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FossilTemplateAudit {
    pub size: [u8; 3],
    pub y_axis_bones: u8,
    pub x_axis_bones: u8,
    pub z_axis_bones: u8,
}

pub const FOSSIL_TEMPLATES: [FossilTemplateAudit; 14] = [
    audit([4, 4, 5], 9, 1, 0),
    audit([5, 1, 5], 0, 5, 5),
    audit([3, 4, 2], 4, 2, 0),
    audit([3, 4, 1], 5, 1, 0),
    audit([2, 5, 1], 4, 1, 0),
    audit([7, 5, 5], 18, 3, 0),
    audit([4, 6, 5], 13, 5, 0),
    audit([3, 5, 1], 4, 2, 0),
    audit([3, 5, 5], 9, 6, 0),
    audit([3, 7, 1], 6, 2, 0),
    audit([5, 5, 7], 18, 6, 0),
    audit([4, 4, 3], 7, 4, 0),
    audit([4, 5, 6], 11, 6, 0),
    audit([7, 7, 6], 21, 5, 0),
];

pub fn find_anchor(
    world: &mut impl NetherFossilWorld,
    chunk_min_x: i32,
    chunk_min_z: i32,
    random: &mut impl GenerationRandom,
) -> Option<BlockPos> {
    let x = chunk_min_x
        + i32::try_from(random.next_u32(nonzero(16))).expect("local coordinate fits i32");
    let z = chunk_min_z
        + i32::try_from(random.next_u32(nonzero(16))).expect("local coordinate fits i32");
    let minimum = 32;
    let maximum = world.generation_top().wrapping_sub(2);
    let mut y = if maximum < minimum {
        minimum
    } else {
        let width = u32::try_from(maximum - minimum + 1).expect("ordered height span fits u32");
        minimum + i32::try_from(random.next_u32(nonzero(width))).expect("height draw fits i32")
    };
    while y > world.sea_level() {
        let upper = world.base_column_state(x, y, z);
        y -= 1;
        let lower = world.base_column_state(x, y, z);
        if upper.block == "minecraft:air"
            && (lower.block == "minecraft:soul_sand" || world.sturdy_upper_face(&lower))
        {
            return Some(BlockPos { x, y, z });
        }
    }
    None
}

pub fn choose_piece(anchor: BlockPos, random: &mut impl GenerationRandom) -> NetherFossilChoice {
    let rotation = match random.next_u32(nonzero(4)) {
        0 => FossilRotation::None,
        1 => FossilRotation::Clockwise90,
        2 => FossilRotation::Clockwise180,
        _ => FossilRotation::Counterclockwise90,
    };
    NetherFossilChoice {
        anchor,
        rotation,
        template: u8::try_from(random.next_u32(nonzero(14)) + 1).expect("template index fits u8"),
    }
}

pub fn place_dried_ghast(
    world: &mut impl NetherFossilWorld,
    template_box: BlockBox,
    processing_box: &BlockBox,
) -> bool {
    let mut random = LegacyRandom::new(world.positional_seed(template_box.center()));
    if random.next_f32() >= 0.5 {
        return false;
    }
    let x = template_box.minimum.x
        + i32::try_from(random.next_u32(nonzero(template_box.size()[0] as u32)))
            .expect("box X draw fits i32");
    let z = template_box.minimum.z
        + i32::try_from(random.next_u32(nonzero(template_box.size()[2] as u32)))
            .expect("box Z draw fits i32");
    let position = BlockPos {
        x,
        y: template_box.minimum.y,
        z,
    };
    if world.state_at(position).block != "minecraft:air" || !processing_box.contains(position) {
        return false;
    }
    let facing = ["north", "east", "south", "west"][random.next_u32(nonzero(4)) as usize];
    let mut state = StructureState::new("minecraft:dried_ghast");
    state.properties.insert("facing".into(), facing.into());
    state.properties.insert("hydration".into(), "0".into());
    state
        .properties
        .insert("waterlogged".into(), "false".into());
    world.set_state(position, state, 2);
    true
}

const fn audit(
    size: [u8; 3],
    y_axis_bones: u8,
    x_axis_bones: u8,
    z_axis_bones: u8,
) -> FossilTemplateAudit {
    FossilTemplateAudit {
        size,
        y_axis_bones,
        x_axis_bones,
        z_axis_bones,
    }
}

fn nonzero(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("random bound must be positive")
}
