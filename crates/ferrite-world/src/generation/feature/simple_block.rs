//! Simple-block feature branches, including double plants and pale moss carpets.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleBlockKind {
    Ordinary,
    DoublePlant,
    MossyCarpet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoublePlantHalf {
    Lower,
    Upper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MossFace {
    None,
    Low,
    Tall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MossFaces {
    pub north: MossFace,
    pub east: MossFace,
    pub south: MossFace,
    pub west: MossFace,
}

impl MossFaces {
    const NONE: Self = Self {
        north: MossFace::None,
        east: MossFace::None,
        south: MossFace::None,
        west: MossFace::None,
    };

    fn get(self, direction: Direction) -> MossFace {
        match direction {
            Direction::North => self.north,
            Direction::East => self.east,
            Direction::South => self.south,
            Direction::West => self.west,
            Direction::Down | Direction::Up => MossFace::None,
        }
    }

    fn set(&mut self, direction: Direction, face: MossFace) {
        match direction {
            Direction::North => self.north = face,
            Direction::East => self.east = face,
            Direction::South => self.south = face,
            Direction::West => self.west = face,
            Direction::Down | Direction::Up => {}
        }
    }

    fn has_any(self) -> bool {
        self != Self::NONE
    }
}

pub trait SimpleBlockWorld<R: GenerationRandom> {
    fn provide_simple_state(&mut self, origin: BlockPos, random: &mut R) -> Option<BlockStateId>;

    fn simple_block_kind(&self, state: BlockStateId) -> SimpleBlockKind;

    fn state_can_survive(&mut self, state: BlockStateId, position: BlockPos) -> bool;

    fn block_state(&mut self, position: BlockPos) -> BlockStateId;

    fn is_empty_block(&mut self, position: BlockPos) -> bool;

    fn state_has_waterlogged(&self, state: BlockStateId) -> bool;

    fn is_water_at(&mut self, position: BlockPos) -> bool;

    fn configure_double_plant_half(
        &mut self,
        state: BlockStateId,
        half: DoublePlantHalf,
        waterlogged: Option<bool>,
    ) -> Option<BlockStateId>;

    fn default_pale_moss_carpet(&self) -> BlockStateId;

    fn is_base_pale_moss(&self, state: BlockStateId) -> bool;

    fn is_nonbase_pale_moss(&self, state: BlockStateId) -> bool;

    fn is_replaceable_for_pale_moss(&self, state: BlockStateId) -> bool;

    fn pale_moss_face(&self, state: BlockStateId, direction: Direction) -> MossFace;

    fn pale_moss_face_supported(&mut self, position: BlockPos, direction: Direction) -> bool;

    fn configure_pale_moss(
        &mut self,
        default_state: BlockStateId,
        base: bool,
        faces: MossFaces,
    ) -> BlockStateId;

    fn next_level_bool(&mut self) -> bool;

    fn offer_simple_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool;

    fn schedule_block_tick(&mut self, position: BlockPos, block_state: BlockStateId, delay: u32);
}

pub fn place_simple_block<R, W>(
    world: &mut W,
    origin: BlockPos,
    schedule_tick: bool,
    random: &mut R,
    ensure_can_write: impl FnOnce(BlockPos) -> bool,
) -> Result<bool, SimpleBlockError>
where
    R: GenerationRandom,
    W: SimpleBlockWorld<R>,
{
    if !ensure_can_write(origin) {
        return Ok(false);
    }
    let Some(selected) = world.provide_simple_state(origin, random) else {
        return Ok(false);
    };
    if !world.state_can_survive(selected, origin) {
        return Ok(false);
    }
    match world.simple_block_kind(selected) {
        SimpleBlockKind::Ordinary => {
            let _ = world.offer_simple_block(origin, selected, 2);
        }
        SimpleBlockKind::DoublePlant => {
            if !place_double_plant(world, origin, selected)? {
                return Ok(false);
            }
        }
        SimpleBlockKind::MossyCarpet => {
            place_pale_moss(world, origin)?;
        }
    }
    if schedule_tick {
        let actual = world.block_state(origin);
        world.schedule_block_tick(origin, actual, 1);
    }
    Ok(true)
}

fn place_double_plant<R, W>(
    world: &mut W,
    origin: BlockPos,
    selected: BlockStateId,
) -> Result<bool, SimpleBlockError>
where
    R: GenerationRandom,
    W: SimpleBlockWorld<R>,
{
    let above = offset(origin, Direction::Up)?;
    if !world.is_empty_block(above) {
        return Ok(false);
    }
    let has_waterlogged = world.state_has_waterlogged(selected);
    let lower_water = has_waterlogged.then(|| world.is_water_at(origin));
    let lower = world
        .configure_double_plant_half(selected, DoublePlantHalf::Lower, lower_water)
        .ok_or(SimpleBlockError::MissingDoublePlantHalf)?;
    let _ = world.offer_simple_block(origin, lower, 2);
    let upper_water = has_waterlogged.then(|| world.is_water_at(above));
    let upper = world
        .configure_double_plant_half(selected, DoublePlantHalf::Upper, upper_water)
        .ok_or(SimpleBlockError::MissingDoublePlantHalf)?;
    let _ = world.offer_simple_block(above, upper, 2);
    Ok(true)
}

fn place_pale_moss<R, W>(world: &mut W, origin: BlockPos) -> Result<(), SimpleBlockError>
where
    R: GenerationRandom,
    W: SimpleBlockWorld<R>,
{
    let default = world.default_pale_moss_carpet();
    let (base_faces, existing_upper) = derive_moss_faces(world, origin)?;
    let base = world.configure_pale_moss(default, true, base_faces);
    let _ = world.offer_simple_block(origin, base, 2);
    if world.is_base_pale_moss(existing_upper)
        || !world.is_nonbase_pale_moss(existing_upper)
            && !world.is_replaceable_for_pale_moss(existing_upper)
    {
        return Ok(());
    }
    let above = offset(origin, Direction::Up)?;
    let (mut topper_faces, _) = derive_moss_faces(world, above)?;
    for direction in horizontal_directions() {
        if topper_faces.get(direction) == MossFace::None {
            continue;
        }
        let retained = base_faces.get(direction) != MossFace::None && world.next_level_bool();
        if !retained {
            topper_faces.set(direction, MossFace::None);
        }
    }
    if !topper_faces.has_any() {
        return Ok(());
    }
    let topper = world.configure_pale_moss(default, false, topper_faces);
    if topper == existing_upper {
        return Ok(());
    }
    let _ = world.offer_simple_block(above, topper, 2);
    let (recomputed_faces, _) = derive_moss_faces(world, origin)?;
    let recomputed_base = world.configure_pale_moss(default, true, recomputed_faces);
    let _ = world.offer_simple_block(origin, recomputed_base, 2);
    Ok(())
}

fn derive_moss_faces<R, W>(
    world: &mut W,
    position: BlockPos,
) -> Result<(MossFaces, BlockStateId), SimpleBlockError>
where
    R: GenerationRandom,
    W: SimpleBlockWorld<R>,
{
    let above = offset(position, Direction::Up)?;
    let above_state = world.block_state(above);
    let mut faces = MossFaces::NONE;
    for direction in horizontal_directions() {
        if !world.pale_moss_face_supported(position, direction) {
            continue;
        }
        let face = if world.is_nonbase_pale_moss(above_state)
            && world.pale_moss_face(above_state, direction) != MossFace::None
        {
            MossFace::Tall
        } else {
            MossFace::Low
        };
        faces.set(direction, face);
    }
    Ok((faces, above_state))
}

const fn horizontal_directions() -> [Direction; 4] {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
}

fn offset(origin: BlockPos, direction: Direction) -> Result<BlockPos, SimpleBlockError> {
    let [x, y, z] = direction.step();
    Ok(BlockPos::new(
        origin
            .x
            .checked_add(x)
            .ok_or(SimpleBlockError::PositionOverflow)?,
        origin
            .y
            .checked_add(y)
            .ok_or(SimpleBlockError::PositionOverflow)?,
        origin
            .z
            .checked_add(z)
            .ok_or(SimpleBlockError::PositionOverflow)?,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SimpleBlockError {
    #[error("configured double plant lacks the required half property")]
    MissingDoublePlantHalf,
    #[error("simple-block position arithmetic overflowed")]
    PositionOverflow,
}
