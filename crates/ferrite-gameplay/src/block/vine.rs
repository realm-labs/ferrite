//! Five-face vine state, support repair, and branch-exact growth candidates.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

pub const VINE_UPDATE_FLAGS: u16 = 2;
pub const VINE_STATE_COUNT: usize = 32;
pub const DENSITY_CELL_COUNT: usize = 243;
pub const HORIZONTAL_CEILING_CHANCE: f32 = 0.05;
pub const HORIZONTAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VineState(u8);

impl VineState {
    const UP: u8 = 1;
    const NORTH: u8 = 1 << 1;
    const EAST: u8 = 1 << 2;
    const SOUTH: u8 = 1 << 3;
    const WEST: u8 = 1 << 4;

    pub const fn from_bits(bits: u8) -> Option<Self> {
        if bits < VINE_STATE_COUNT as u8 {
            Some(Self(bits))
        } else {
            None
        }
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn has(self, face: Direction) -> bool {
        let bit = face_bit(face);
        bit != 0 && self.0 & bit != 0
    }

    pub const fn with(self, face: Direction, present: bool) -> Self {
        let bit = face_bit(face);
        if present {
            Self(self.0 | bit)
        } else {
            Self(self.0 & !bit)
        }
    }

    pub const fn face_count(self) -> u32 {
        self.0.count_ones()
    }

    pub const fn has_horizontal(self) -> bool {
        self.0 & !Self::UP != 0
    }

    pub const fn can_survive(self) -> bool {
        self.0 != 0
    }

    pub const fn uses_full_fallback_outline(self) -> bool {
        self.0 == 0
    }

    pub const fn can_be_replaced_by_vine(self) -> bool {
        self.face_count() < 5
    }
}

const fn face_bit(face: Direction) -> u8 {
    match face {
        Direction::Up => VineState::UP,
        Direction::North => VineState::NORTH,
        Direction::East => VineState::EAST,
        Direction::South => VineState::SOUTH,
        Direction::West => VineState::WEST,
        Direction::Down => 0,
    }
}

pub fn placement_state<F>(
    existing: Option<VineState>,
    nearest_directions: &[Direction],
    mut directly_supported: F,
) -> Option<VineState>
where
    F: FnMut(Direction) -> bool,
{
    let state = existing.unwrap_or_default();
    for &direction in nearest_directions {
        if direction != Direction::Down && !state.has(direction) && directly_supported(direction) {
            return Some(state.with(direction, true));
        }
    }
    existing
}

pub fn repair_state<F>(
    state: VineState,
    changed_direction: Direction,
    above: Option<VineState>,
    mut directly_supported: F,
) -> Option<VineState>
where
    F: FnMut(Direction) -> bool,
{
    if changed_direction == Direction::Down {
        return Some(state);
    }
    let mut repaired = state.with(Direction::Up, directly_supported(Direction::Up));
    for direction in HORIZONTAL_DIRECTIONS {
        if repaired.has(direction)
            && !directly_supported(direction)
            && !above.is_some_and(|above| above.has(direction))
        {
            repaired = repaired.with(direction, false);
        }
    }
    repaired.can_survive().then_some(repaired)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VineTransform {
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
    MirrorLeftRight,
    MirrorFrontBack,
}

pub fn transform(state: VineState, transform: VineTransform) -> VineState {
    let mut transformed = VineState::default().with(Direction::Up, state.has(Direction::Up));
    for direction in HORIZONTAL_DIRECTIONS {
        if state.has(direction) {
            transformed = transformed.with(transform_direction(direction, transform), true);
        }
    }
    transformed
}

const fn transform_direction(direction: Direction, transform: VineTransform) -> Direction {
    match transform {
        VineTransform::Clockwise90 => clockwise(direction),
        VineTransform::Clockwise180 => clockwise(clockwise(direction)),
        VineTransform::CounterClockwise90 => counterclockwise(direction),
        VineTransform::MirrorLeftRight => match direction {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            other => other,
        },
        VineTransform::MirrorFrontBack => match direction {
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            other => other,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VineGrowthAdmission {
    pub direction: Option<Direction>,
    pub draws_consumed: u8,
}

pub const fn growth_admission(
    spread_vines: bool,
    one_in_four_draw: u32,
    chosen_direction: Direction,
) -> VineGrowthAdmission {
    if !spread_vines {
        VineGrowthAdmission {
            direction: None,
            draws_consumed: 0,
        }
    } else if one_in_four_draw != 0 {
        VineGrowthAdmission {
            direction: None,
            draws_consumed: 1,
        }
    } else {
        VineGrowthAdmission {
            direction: Some(chosen_direction),
            draws_consumed: 2,
        }
    }
}

pub const fn density_allows_growth(vine_count: usize) -> bool {
    vine_count < 5
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizontalGrowthInputs {
    pub target_is_air: bool,
    pub target_accepts_selected_face: bool,
    pub target_clockwise_neighbor_acceptable: bool,
    pub target_counterclockwise_neighbor_acceptable: bool,
    pub clockwise_diagonal_empty: bool,
    pub counterclockwise_diagonal_empty: bool,
    pub clockwise_source_neighbor_accepts_opposite: bool,
    pub counterclockwise_source_neighbor_accepts_opposite: bool,
    pub fallback_draw: f32,
    pub target_ceiling_acceptable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VineWrite {
    pub position: BlockPos,
    pub state: VineState,
    pub flags: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VineGrowthResult {
    pub write: Option<VineWrite>,
    pub draws_consumed: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpwardGrowthInputs {
    pub maximum_y: i32,
    pub direct_ceiling_support: bool,
    pub above_is_air: bool,
    pub density_allows: bool,
    pub above_support: [bool; 4],
    pub coins: [bool; 4],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BelowVineTarget {
    Air,
    Vine(VineState),
    Other,
}

pub fn horizontal_growth(
    position: BlockPos,
    source: VineState,
    direction: Direction,
    density_allows: bool,
    inputs: HorizontalGrowthInputs,
) -> VineGrowthResult {
    if !direction.is_horizontal() || source.has(direction) || !density_allows {
        return growth_result(None, 0);
    }
    if !inputs.target_is_air {
        return growth_result(
            inputs.target_accepts_selected_face.then_some(VineWrite {
                position,
                state: source.with(direction, true),
                flags: VINE_UPDATE_FLAGS,
            }),
            0,
        );
    }

    let Some(target) = offset(position, direction) else {
        return growth_result(None, 0);
    };
    let clockwise = clockwise(direction);
    let counterclockwise = counterclockwise(direction);
    if source.has(clockwise) && inputs.target_clockwise_neighbor_acceptable {
        return growth_result(
            Some(write(target, VineState::default().with(clockwise, true))),
            0,
        );
    }
    if source.has(counterclockwise) && inputs.target_counterclockwise_neighbor_acceptable {
        return growth_result(
            Some(write(
                target,
                VineState::default().with(counterclockwise, true),
            )),
            0,
        );
    }
    if source.has(clockwise)
        && inputs.clockwise_diagonal_empty
        && inputs.clockwise_source_neighbor_accepts_opposite
    {
        return growth_result(
            offset(target, clockwise).map(|diagonal| {
                write(
                    diagonal,
                    VineState::default().with(direction.opposite(), true),
                )
            }),
            0,
        );
    }
    if source.has(counterclockwise)
        && inputs.counterclockwise_diagonal_empty
        && inputs.counterclockwise_source_neighbor_accepts_opposite
    {
        return growth_result(
            offset(target, counterclockwise).map(|diagonal| {
                write(
                    diagonal,
                    VineState::default().with(direction.opposite(), true),
                )
            }),
            0,
        );
    }
    growth_result(
        (inputs.fallback_draw < HORIZONTAL_CEILING_CHANCE && inputs.target_ceiling_acceptable)
            .then_some(write(
                target,
                VineState::default().with(Direction::Up, true),
            )),
        1,
    )
}

pub fn upward_growth(
    position: BlockPos,
    source: VineState,
    inputs: UpwardGrowthInputs,
) -> VineGrowthResult {
    if position.y >= inputs.maximum_y {
        return growth_result(None, 0);
    }
    if inputs.direct_ceiling_support {
        return growth_result(Some(write(position, source.with(Direction::Up, true))), 0);
    }
    if !inputs.above_is_air || !inputs.density_allows {
        return growth_result(None, 0);
    }
    let mut candidate = source;
    for (index, direction) in HORIZONTAL_DIRECTIONS.into_iter().enumerate() {
        candidate = candidate.with(
            direction,
            !inputs.coins[index] && inputs.above_support[index],
        );
    }
    growth_result(
        candidate.has_horizontal().then(|| {
            write(
                offset(position, Direction::Up).expect("position is below maximum Y"),
                candidate,
            )
        }),
        4,
    )
}

pub fn downward_growth(
    position: BlockPos,
    source: VineState,
    minimum_y: i32,
    below: BelowVineTarget,
    coins: [bool; 4],
) -> VineGrowthResult {
    if position.y <= minimum_y {
        return growth_result(None, 0);
    }
    let starting = match below {
        BelowVineTarget::Air => VineState::default(),
        BelowVineTarget::Vine(state) => state,
        BelowVineTarget::Other => return growth_result(None, 0),
    };
    let mut candidate = starting;
    for (index, direction) in HORIZONTAL_DIRECTIONS.into_iter().enumerate() {
        if coins[index] && source.has(direction) {
            candidate = candidate.with(direction, true);
        }
    }
    growth_result(
        (candidate != starting && candidate.has_horizontal()).then(|| {
            write(
                offset(position, Direction::Down).expect("position is above minimum Y"),
                candidate,
            )
        }),
        4,
    )
}

const fn clockwise(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::East,
        Direction::East => Direction::South,
        Direction::South => Direction::West,
        Direction::West => Direction::North,
        vertical => vertical,
    }
}

const fn counterclockwise(direction: Direction) -> Direction {
    clockwise(clockwise(clockwise(direction)))
}

fn offset(position: BlockPos, direction: Direction) -> Option<BlockPos> {
    position.checked_offset(direction, 1).ok()
}

const fn write(position: BlockPos, state: VineState) -> VineWrite {
    VineWrite {
        position,
        state,
        flags: VINE_UPDATE_FLAGS,
    }
}

const fn growth_result(write: Option<VineWrite>, draws_consumed: u8) -> VineGrowthResult {
    VineGrowthResult {
        write,
        draws_consumed,
    }
}
