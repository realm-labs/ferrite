//! Full-copper weathering and Copper-Golem-Statue semantic kernels.

use ferrite_foundation::direction::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CopperAge {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

impl CopperAge {
    pub fn next(self) -> Option<Self> {
        match self {
            Self::Unaffected => Some(Self::Exposed),
            Self::Exposed => Some(Self::Weathered),
            Self::Weathered => Some(Self::Oxidized),
            Self::Oxidized => None,
        }
    }

    const fn index(self) -> u32 {
        match self {
            Self::Unaffected => 0,
            Self::Exposed => 1,
            Self::Weathered => 2,
            Self::Oxidized => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperFullKind {
    Block,
    Cut,
    Chiseled,
}

impl CopperFullKind {
    pub fn state_id(self, age: CopperAge, waxed: bool) -> u32 {
        let base = match self {
            Self::Block => 27_782,
            Self::Cut => 27_792,
            Self::Chiseled => 27_800,
        };
        base + age.index() + if waxed { 4 } else { 0 }
    }

    pub fn block_id(self, age: CopperAge, waxed: bool) -> u16 {
        let base = match self {
            Self::Block => 1_034,
            Self::Cut => 1_044,
            Self::Chiseled => 1_052,
        };
        base + age.index() as u16 + if waxed { 4 } else { 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatheringDecision {
    FirstDrawRejected,
    YoungerNeighborAbort,
    SecondDrawRejected { threshold: f32 },
    Advance(CopperAge),
    Terminal,
}

pub fn weathering_decision(
    age: CopperAge,
    first_draw: f32,
    younger_neighbor: bool,
    same: u32,
    older: u32,
    second_draw: Option<f32>,
) -> WeatheringDecision {
    let next = match age.next() {
        Some(next) => next,
        None => return WeatheringDecision::Terminal,
    };
    if first_draw >= 0.05688889 {
        return WeatheringDecision::FirstDrawRejected;
    }
    if younger_neighbor {
        return WeatheringDecision::YoungerNeighborAbort;
    }
    let ratio = (older + 1) as f32 / (older + same + 1) as f32;
    let modifier = if age == CopperAge::Unaffected {
        0.75
    } else {
        1.0
    };
    let threshold = ratio * ratio * modifier;
    if second_draw.is_some_and(|draw| draw < threshold) {
        WeatheringDecision::Advance(next)
    } else {
        WeatheringDecision::SecondDrawRejected { threshold }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxeCopperTransform {
    Scrape(CopperAge),
    WaxOff(CopperAge),
    Pass,
}

pub fn axe_transform(age: CopperAge, waxed: bool) -> AxeCopperTransform {
    if waxed {
        AxeCopperTransform::WaxOff(age)
    } else {
        match age {
            CopperAge::Unaffected => AxeCopperTransform::Pass,
            CopperAge::Exposed => AxeCopperTransform::Scrape(CopperAge::Unaffected),
            CopperAge::Weathered => AxeCopperTransform::Scrape(CopperAge::Exposed),
            CopperAge::Oxidized => AxeCopperTransform::Scrape(CopperAge::Weathered),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatuePose {
    Standing,
    Sitting,
    Running,
    Star,
}

impl StatuePose {
    pub fn next(self) -> Self {
        match self {
            Self::Standing => Self::Sitting,
            Self::Sitting => Self::Running,
            Self::Running => Self::Star,
            Self::Star => Self::Standing,
        }
    }

    pub fn comparator_output(self) -> u8 {
        match self {
            Self::Standing => 1,
            Self::Sitting => 2,
            Self::Running => 3,
            Self::Star => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatueState {
    pub age: CopperAge,
    pub waxed: bool,
    pub pose: StatuePose,
    pub facing: Direction,
    pub waterlogged: bool,
}

impl StatueState {
    pub fn state_id(self) -> Option<u32> {
        let facing = match self.facing {
            Direction::North => 0,
            Direction::South => 1,
            Direction::West => 2,
            Direction::East => 3,
            Direction::Down | Direction::Up => return None,
        };
        let pose = match self.pose {
            StatuePose::Standing => 0,
            StatuePose::Sitting => 1,
            StatuePose::Running => 2,
            StatuePose::Star => 3,
        };
        Some(
            29_760
                + self.age.index() * 32
                + u32::from(self.waxed) * 128
                + pose * 8
                + facing * 2
                + u32::from(!self.waterlogged),
        )
    }
}

pub fn statue_family_state_count() -> usize {
    8 * 4 * 4 * 2
}

pub fn statue_use_non_axe(state: StatueState) -> StatueState {
    StatueState {
        pose: state.pose.next(),
        ..state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GolemWeatherClock {
    Waxed,
    Initialize { deadline: u64 },
    Waiting,
    Advanced { age: CopperAge, next_deadline: u64 },
    TestStatueConversion,
}

pub fn copper_golem_weather_tick(
    next_weather_age: i64,
    game_time: u64,
    age: CopperAge,
    duration_504000_to_552000: u64,
) -> GolemWeatherClock {
    if next_weather_age == -2 {
        return GolemWeatherClock::Waxed;
    }
    if next_weather_age == -1 {
        return GolemWeatherClock::Initialize {
            deadline: game_time + duration_504000_to_552000.clamp(504_000, 552_000),
        };
    }
    if age == CopperAge::Oxidized {
        return GolemWeatherClock::TestStatueConversion;
    }
    if game_time < next_weather_age as u64 {
        return GolemWeatherClock::Waiting;
    }
    let next = match age.next() {
        Some(next) => next,
        None => CopperAge::Oxidized,
    };
    GolemWeatherClock::Advanced {
        age: next,
        next_deadline: if next == CopperAge::Oxidized {
            0
        } else {
            next_weather_age as u64 + duration_504000_to_552000.clamp(504_000, 552_000)
        },
    }
}

pub fn golem_statue_conversion_admitted(position_air: bool, next_float: f32) -> bool {
    position_air && next_float <= 0.0058
}
