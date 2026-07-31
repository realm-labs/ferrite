//! Weighted structure-template feature with rotation-derived origin offsets.

use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TemplateFeatureId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateRotation {
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedTemplateEntry {
    pub identifier: TemplateFeatureId,
    pub weight: NonZeroU32,
    pub rotations: Vec<TemplateRotation>,
}

pub trait TemplateFeatureWorld {
    fn resolve_template(&mut self, identifier: TemplateFeatureId) -> bool;

    fn unrotated_template_size(&mut self, identifier: TemplateFeatureId) -> [i32; 3];

    fn place_template_feature<R: GenerationRandom>(
        &mut self,
        identifier: TemplateFeatureId,
        position: BlockPos,
        pivot: BlockPos,
        rotation: TemplateRotation,
        random: &mut R,
        flags: u32,
    ) -> bool;
}

pub fn place_template_feature<R, W>(
    world: &mut W,
    origin: BlockPos,
    entries: &[WeightedTemplateEntry],
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, TemplateFeatureError>
where
    R: GenerationRandom,
    W: TemplateFeatureWorld,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let entry = select_entry(entries, random)?;
    let rotation_bound = u32::try_from(entry.rotations.len())
        .ok()
        .and_then(NonZeroU32::new)
        .ok_or(TemplateFeatureError::EmptyRotations(entry.identifier))?;
    let rotation = entry.rotations[random.next_u32(rotation_bound) as usize];
    if !world.resolve_template(entry.identifier) {
        return Err(TemplateFeatureError::MissingTemplate(entry.identifier));
    }
    let size = world.unrotated_template_size(entry.identifier);
    if size[0] < 0 || size[2] < 0 {
        return Err(TemplateFeatureError::InvalidTemplateSize);
    }
    let x_half = size[0] / 2;
    let z_half = size[2] / 2;
    let x_direction = rotate(Direction::West, rotation);
    let z_direction = rotate(Direction::North, rotation);
    let [x_step_x, _, x_step_z] = x_direction.step();
    let [z_step_x, _, z_step_z] = z_direction.step();
    let x_offset = x_step_x
        .checked_mul(x_half)
        .and_then(|value| {
            z_step_x
                .checked_mul(z_half)
                .and_then(|other| value.checked_add(other))
        })
        .ok_or(TemplateFeatureError::PositionOverflow)?;
    let z_offset = x_step_z
        .checked_mul(x_half)
        .and_then(|value| {
            z_step_z
                .checked_mul(z_half)
                .and_then(|other| value.checked_add(other))
        })
        .ok_or(TemplateFeatureError::PositionOverflow)?;
    let position = BlockPos::new(
        origin
            .x
            .checked_add(x_offset)
            .ok_or(TemplateFeatureError::PositionOverflow)?,
        origin.y,
        origin
            .z
            .checked_add(z_offset)
            .ok_or(TemplateFeatureError::PositionOverflow)?,
    );
    Ok(world.place_template_feature(entry.identifier, position, position, rotation, random, 3))
}

fn select_entry<'a>(
    entries: &'a [WeightedTemplateEntry],
    random: &mut impl GenerationRandom,
) -> Result<&'a WeightedTemplateEntry, TemplateFeatureError> {
    let total = entries.iter().try_fold(0_u32, |total, entry| {
        total
            .checked_add(entry.weight.get())
            .ok_or(TemplateFeatureError::WeightOverflow)
    })?;
    let bound = NonZeroU32::new(total).ok_or(TemplateFeatureError::EmptyEntries)?;
    let draw = random.next_u32(bound);
    let mut cumulative = 0_u32;
    for entry in entries {
        cumulative += entry.weight.get();
        if draw < cumulative {
            return Ok(entry);
        }
    }
    unreachable!("bounded template weight draw belongs to one entry")
}

const fn rotate(direction: Direction, rotation: TemplateRotation) -> Direction {
    match rotation {
        TemplateRotation::None => direction,
        TemplateRotation::Clockwise90 => clockwise(direction),
        TemplateRotation::Clockwise180 => clockwise(clockwise(direction)),
        TemplateRotation::Counterclockwise90 => clockwise(clockwise(clockwise(direction))),
    }
}

const fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        Direction::Down | Direction::Up => direction,
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TemplateFeatureError {
    #[error("template feature weighted list is empty")]
    EmptyEntries,
    #[error("template feature total weight overflowed")]
    WeightOverflow,
    #[error("template feature entry {0:?} has no rotations")]
    EmptyRotations(TemplateFeatureId),
    #[error("template feature could not resolve {0:?}")]
    MissingTemplate(TemplateFeatureId),
    #[error("template feature has a negative horizontal size")]
    InvalidTemplateSize,
    #[error("template feature position overflow")]
    PositionOverflow,
}
