//! Timeline validation and Java-compatible track sampling.

use std::collections::BTreeMap;

use super::environment::{
    AttributeEntry, AttributeMap, AttributeValue, Modifier, declaration_by_id,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    Constant,
    CubicBezier { x1: f32, y1: f32, x2: f32, y2: f32 },
}

impl Easing {
    pub fn apply(self, fraction: f32) -> f32 {
        let fraction = fraction.clamp(0.0, 1.0);
        match self {
            Self::Linear => fraction,
            Self::Constant => 0.0,
            Self::CubicBezier { x1, y1, x2, y2 } => cubic_bezier_y_for_x(fraction, x1, y1, x2, y2),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keyframe {
    pub ticks: i64,
    pub value: AttributeValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttributeTrack {
    pub attribute: String,
    pub modifier: Modifier,
    pub easing: Easing,
    pub keyframes: Vec<Keyframe>,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TimelineError {
    #[error("period must be positive")]
    InvalidPeriod,
    #[error("marker {marker} tick {ticks} is outside the period")]
    MarkerOutsidePeriod { marker: String, ticks: i64 },
    #[error("track {attribute} has no keyframes")]
    EmptyTrack { attribute: String },
    #[error("track {attribute} has a negative or unordered keyframe")]
    InvalidKeyframeOrder { attribute: String },
    #[error("track {attribute} has too many repeated ticks")]
    TooManyRepeatedTicks { attribute: String },
    #[error("periodic track {attribute} exceeds the period")]
    TrackOutsidePeriod { attribute: String },
    #[error("track {attribute} uses an unknown or mismatched attribute")]
    InvalidAttribute { attribute: String },
}

impl AttributeTrack {
    pub fn new(
        attribute: impl Into<String>,
        modifier: Modifier,
        easing: Easing,
        keyframes: Vec<Keyframe>,
        period: Option<i64>,
    ) -> Result<Self, TimelineError> {
        let attribute = attribute.into();
        if keyframes.is_empty() {
            return Err(TimelineError::EmptyTrack { attribute });
        }
        if declaration_by_id(&attribute).is_none()
            || keyframes.iter().any(|frame| {
                declaration_by_id(&attribute).is_none_or(|declaration| {
                    !declaration.default.same_type_for_track(&frame.value)
                })
            })
        {
            return Err(TimelineError::InvalidAttribute { attribute });
        }
        if keyframes.iter().any(|frame| frame.ticks < 0)
            || keyframes
                .windows(2)
                .any(|pair| pair[0].ticks > pair[1].ticks)
        {
            return Err(TimelineError::InvalidKeyframeOrder { attribute });
        }
        if period.is_some_and(|period| keyframes.iter().any(|frame| frame.ticks > period)) {
            return Err(TimelineError::TrackOutsidePeriod { attribute });
        }
        let all_same = keyframes
            .iter()
            .all(|frame| frame.ticks == keyframes.last().expect("nonempty").ticks);
        let maximum_run = if all_same { 2 } else { 3 };
        let mut run = 1;
        for pair in keyframes.windows(2) {
            if pair[0].ticks == pair[1].ticks {
                run += 1;
                if run > maximum_run {
                    return Err(TimelineError::TooManyRepeatedTicks { attribute });
                }
            } else {
                run = 1;
            }
        }
        Ok(Self {
            attribute,
            modifier,
            easing,
            keyframes,
        })
    }

    pub fn sample(&self, total_ticks: i64, period: Option<i64>) -> AttributeValue {
        if self.keyframes.len() == 1 {
            return self.keyframes[0].value.clone();
        }
        let declaration = declaration_by_id(&self.attribute)
            .expect("validated timeline track has a registered declaration");
        let sample_tick = period.map_or(total_ticks, |period| total_ticks.rem_euclid(period));
        if let Some(period) = period {
            return self.sample_periodic(sample_tick, period, &declaration);
        }
        sample_frames(&self.keyframes, sample_tick, self.easing, &declaration)
    }

    fn sample_periodic(
        &self,
        sample_tick: i64,
        period: i64,
        declaration: &super::environment::AttributeDeclaration,
    ) -> AttributeValue {
        let first = &self.keyframes[0];
        let last = self.keyframes.last().expect("validated nonempty track");
        if sample_tick < first.ticks {
            return sample_segment(
                &Keyframe {
                    ticks: last.ticks - period,
                    value: last.value.clone(),
                },
                first,
                sample_tick,
                self.easing,
                declaration,
            );
        }
        if sample_tick >= last.ticks {
            return sample_segment(
                last,
                &Keyframe {
                    ticks: first.ticks + period,
                    value: first.value.clone(),
                },
                sample_tick,
                self.easing,
                declaration,
            );
        }
        sample_frames(&self.keyframes, sample_tick, self.easing, declaration)
    }
}

trait TrackValueType {
    fn same_type_for_track(&self, other: &Self) -> bool;
}

impl TrackValueType for AttributeValue {
    fn same_type_for_track(&self, other: &Self) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimeMarker {
    pub ticks: i64,
    pub show_in_commands: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    pub clock: String,
    pub period_ticks: Option<i64>,
    pub markers: BTreeMap<String, TimeMarker>,
    pub tracks: Vec<AttributeTrack>,
}

impl Timeline {
    pub fn new(
        clock: impl Into<String>,
        period_ticks: Option<i64>,
        markers: BTreeMap<String, TimeMarker>,
        tracks: Vec<AttributeTrack>,
    ) -> Result<Self, TimelineError> {
        if period_ticks.is_some_and(|period| period <= 0) {
            return Err(TimelineError::InvalidPeriod);
        }
        if let Some(period) = period_ticks
            && let Some((marker, value)) = markers
                .iter()
                .find(|(_, value)| !(0..period).contains(&value.ticks))
        {
            return Err(TimelineError::MarkerOutsidePeriod {
                marker: marker.clone(),
                ticks: value.ticks,
            });
        }
        Ok(Self {
            clock: clock.into(),
            period_ticks,
            markers,
            tracks,
        })
    }

    pub fn sample(&self, total_ticks: i64) -> AttributeMap {
        self.tracks
            .iter()
            .map(|track| {
                (
                    track.attribute.clone(),
                    AttributeEntry {
                        value: track.sample(total_ticks, self.period_ticks),
                        modifier: track.modifier,
                    },
                )
            })
            .collect()
    }

    pub fn network_tracks(&self) -> Vec<&AttributeTrack> {
        self.tracks
            .iter()
            .filter(|track| declaration_by_id(&track.attribute).is_some_and(|value| value.syncable))
            .collect()
    }

    pub fn marker(&self, marker: &str) -> Option<i64> {
        self.markers.get(marker).map(|value| value.ticks)
    }
}

fn sample_frames(
    frames: &[Keyframe],
    tick: i64,
    easing: Easing,
    declaration: &super::environment::AttributeDeclaration,
) -> AttributeValue {
    if tick < frames[0].ticks {
        return frames[0].value.clone();
    }
    for pair in frames.windows(2) {
        if tick < pair[1].ticks {
            return sample_segment(&pair[0], &pair[1], tick, easing, declaration);
        }
    }
    frames
        .last()
        .expect("validated nonempty track")
        .value
        .clone()
}

fn sample_segment(
    start: &Keyframe,
    end: &Keyframe,
    tick: i64,
    easing: Easing,
    declaration: &super::environment::AttributeDeclaration,
) -> AttributeValue {
    if end.ticks <= start.ticks {
        return end.value.clone();
    }
    let fraction = ((tick - start.ticks) as f32 / (end.ticks - start.ticks) as f32).clamp(0.0, 1.0);
    declaration
        .interpolate(&start.value, &end.value, easing.apply(fraction))
        .unwrap_or_else(|| {
            if easing.apply(fraction) < 0.5 {
                start.value.clone()
            } else {
                end.value.clone()
            }
        })
}

fn cubic_bezier_y_for_x(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let mut low = 0.0_f32;
    let mut high = 1.0_f32;
    for _ in 0..16 {
        let time = (low + high) * 0.5;
        if cubic(time, x1, x2) < x {
            low = time;
        } else {
            high = time;
        }
    }
    cubic((low + high) * 0.5, y1, y2)
}

fn cubic(time: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - time;
    3.0 * inverse * inverse * time * first
        + 3.0 * inverse * time * time * second
        + time * time * time
}

fn frame(ticks: i64, value: AttributeValue) -> Keyframe {
    Keyframe { ticks, value }
}

fn f(value: f32) -> AttributeValue {
    AttributeValue::Float(value)
}

fn b(value: bool) -> AttributeValue {
    AttributeValue::Boolean(value)
}

fn id(value: &str) -> AttributeValue {
    AttributeValue::Identifier(value.to_owned())
}

fn rgb(value: u32) -> AttributeValue {
    AttributeValue::Color((0xff00_0000 | value) as i32)
}

fn track(
    id: &str,
    modifier: Modifier,
    easing: Easing,
    frames: Vec<Keyframe>,
    period: Option<i64>,
) -> AttributeTrack {
    AttributeTrack::new(format!("minecraft:{id}"), modifier, easing, frames, period)
        .expect("locked timeline track is valid")
}

pub fn locked_day_timeline() -> Timeline {
    let period = Some(24_000);
    let angle_easing = Easing::CubicBezier {
        x1: 0.362,
        y1: 0.241,
        x2: 0.638,
        y2: 0.759,
    };
    let mut markers = BTreeMap::new();
    for (name, ticks, show) in [
        ("minecraft:day", 1_000, true),
        ("minecraft:noon", 6_000, true),
        ("minecraft:night", 13_000, true),
        ("minecraft:midnight", 18_000, true),
        ("minecraft:roll_village_siege", 18_000, false),
        ("minecraft:wake_up_from_sleep", 0, false),
    ] {
        markers.insert(
            name.to_owned(),
            TimeMarker {
                ticks,
                show_in_commands: show,
            },
        );
    }
    let tracks = vec![
        track(
            "audio/firefly_bush_sounds",
            Modifier::Or,
            Easing::Linear,
            vec![frame(12_600, b(true)), frame(23_401, b(false))],
            period,
        ),
        track(
            "gameplay/bees_stay_in_hive",
            Modifier::Or,
            Easing::Linear,
            vec![frame(12_542, b(true)), frame(23_460, b(false))],
            period,
        ),
        track(
            "gameplay/cat_waking_up_gift_chance",
            Modifier::Maximum,
            Easing::Constant,
            vec![frame(362, f(0.0)), frame(23_667, f(0.7))],
            period,
        ),
        track(
            "gameplay/creaking_active",
            Modifier::Or,
            Easing::Linear,
            vec![frame(12_600, b(true)), frame(23_401, b(false))],
            period,
        ),
        track(
            "gameplay/eyeblossom_open",
            Modifier::Override,
            Easing::Linear,
            vec![frame(12_600, id("true")), frame(23_401, id("false"))],
            period,
        ),
        track(
            "gameplay/monsters_burn",
            Modifier::Or,
            Easing::Linear,
            vec![frame(12_542, b(false)), frame(23_460, b(true))],
            period,
        ),
        track(
            "gameplay/sky_light_level",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(133, f(1.0)),
                frame(11_867, f(1.0)),
                frame(13_670, f(0.266_666_68)),
                frame(22_330, f(0.266_666_68)),
            ],
            period,
        ),
        track(
            "gameplay/turtle_egg_hatch_chance",
            Modifier::Maximum,
            Easing::Constant,
            vec![frame(21_062, f(1.0)), frame(21_905, f(0.002))],
            period,
        ),
        track(
            "visual/cloud_color",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(133, rgb(0xff_ffff)),
                frame(11_867, rgb(0xff_ffff)),
                frame(13_670, AttributeValue::Color(-15_132_378)),
                frame(22_330, AttributeValue::Color(-15_132_378)),
            ],
            period,
        ),
        track(
            "visual/fog_color",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(133, rgb(0xff_ffff)),
                frame(11_867, rgb(0xff_ffff)),
                frame(13_670, rgb(0x0c_0c16)),
                frame(22_330, rgb(0x16_1616)),
            ],
            period,
        ),
        track(
            "visual/moon_angle",
            Modifier::Override,
            angle_easing,
            vec![frame(6_000, f(540.0)), frame(6_000, f(180.0))],
            period,
        ),
        track(
            "visual/sky_color",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(133, rgb(0xff_ffff)),
                frame(11_867, rgb(0xff_ffff)),
                frame(13_670, rgb(0)),
                frame(22_330, rgb(0)),
            ],
            period,
        ),
        track(
            "visual/sky_light_color",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(730, rgb(0xff_ffff)),
                frame(11_270, rgb(0xff_ffff)),
                frame(13_140, rgb(0x7a_7aff)),
                frame(22_860, rgb(0x7a_7aff)),
            ],
            period,
        ),
        track(
            "visual/sky_light_factor",
            Modifier::Multiply,
            Easing::Linear,
            vec![
                frame(730, f(1.0)),
                frame(11_270, f(1.0)),
                frame(13_140, f(0.24)),
                frame(22_860, f(0.24)),
            ],
            period,
        ),
        track(
            "visual/star_angle",
            Modifier::Override,
            angle_easing,
            vec![frame(6_000, f(360.0)), frame(6_000, f(0.0))],
            period,
        ),
        track(
            "visual/star_brightness",
            Modifier::Maximum,
            Easing::Linear,
            vec![
                frame(92, f(0.037)),
                frame(627, f(0.0)),
                frame(11_373, f(0.0)),
                frame(11_732, f(0.016)),
                frame(11_959, f(0.044)),
                frame(12_399, f(0.143)),
                frame(12_729, f(0.258)),
                frame(13_228, f(0.5)),
                frame(22_772, f(0.5)),
                frame(23_032, f(0.364)),
                frame(23_356, f(0.225)),
                frame(23_758, f(0.101)),
            ],
            period,
        ),
        track(
            "visual/sun_angle",
            Modifier::Override,
            angle_easing,
            vec![frame(6_000, f(360.0)), frame(6_000, f(0.0))],
            period,
        ),
        track(
            "visual/sunrise_sunset_color",
            Modifier::Override,
            Easing::Linear,
            sunrise_frames(),
            period,
        ),
    ];
    Timeline::new("minecraft:overworld", period, markers, tracks).expect("locked day timeline")
}

fn sunrise_frames() -> Vec<Keyframe> {
    let values: &[(i64, u32)] = &[
        (71, 0x5fef_a333),
        (310, 0x29f5_ba33),
        (565, 0x06fb_d433),
        (730, 0x00ff_e533),
        (11_270, 0x00ff_e533),
        (11_397, 0x04fc_d833),
        (11_522, 0x0ff9_cb33),
        (11_690, 0x29f5_ba33),
        (11_929, 0x5fef_a333),
        (12_243, 0xb1e7_8787),
        (12_358, 0xcce4_7e33),
        (12_512, 0xe9e0_7233),
        (12_613, 0xf6dd_6b33),
        (12_732, 0xfeda_6333),
        (12_841, 0xfed7_5c33),
        (13_035, 0xecd2_5133),
        (13_252, 0xc1cc_4733),
        (13_775, 0x36be_3733),
        (13_888, 0x1fbb_3533),
        (14_039, 0x09b7_3333),
        (14_192, 0x00b3_3333),
        (21_807, 0x00b2_3333),
        (21_961, 0x09b7_3333),
        (22_112, 0x1fbb_3533),
        (22_225, 0x36be_3733),
        (22_748, 0xc1cc_4733),
        (22_965, 0xecd2_5133),
        (23_159, 0xfed7_5c33),
        (23_272, 0xfeda_6333),
        (23_488, 0xe9e0_7233),
        (23_642, 0xcce4_7e33),
        (23_757, 0xb1e7_8787),
    ];
    values
        .iter()
        .map(|(ticks, value)| frame(*ticks, AttributeValue::Color(*value as i32)))
        .collect()
}

pub fn locked_moon_timeline() -> Timeline {
    let period = Some(192_000);
    let brightness = [0.5, 0.375, 0.25, 0.125, 0.0, 0.125, 0.25, 0.375];
    let phases = [
        "full_moon",
        "waning_gibbous",
        "third_quarter",
        "waning_crescent",
        "new_moon",
        "waxing_crescent",
        "first_quarter",
        "waxing_gibbous",
    ];
    let slime = brightness
        .into_iter()
        .enumerate()
        .map(|(index, value)| frame(index as i64 * 24_000, f(value)))
        .collect();
    let moon = phases
        .into_iter()
        .enumerate()
        .map(|(index, value)| frame(index as i64 * 24_000, id(value)))
        .collect();
    Timeline::new(
        "minecraft:overworld",
        period,
        BTreeMap::new(),
        vec![
            track(
                "gameplay/surface_slime_spawn_chance",
                Modifier::Maximum,
                Easing::Constant,
                slime,
                period,
            ),
            track(
                "visual/moon_phase",
                Modifier::Override,
                Easing::Linear,
                moon,
                period,
            ),
        ],
    )
    .expect("locked moon timeline")
}

pub fn locked_early_game_timeline() -> Timeline {
    Timeline::new(
        "minecraft:overworld",
        None,
        BTreeMap::new(),
        vec![track(
            "gameplay/can_pillager_patrol_spawn",
            Modifier::And,
            Easing::Linear,
            vec![frame(0, b(false)), frame(120_000, b(true))],
            None,
        )],
    )
    .expect("locked early-game timeline")
}

pub fn locked_villager_schedule_timeline() -> Timeline {
    let period = Some(24_000);
    Timeline::new(
        "minecraft:overworld",
        period,
        BTreeMap::new(),
        vec![
            track(
                "gameplay/villager_activity",
                Modifier::Override,
                Easing::Linear,
                vec![
                    frame(10, id("minecraft:idle")),
                    frame(2_000, id("minecraft:work")),
                    frame(9_000, id("minecraft:meet")),
                    frame(11_000, id("minecraft:idle")),
                    frame(12_000, id("minecraft:rest")),
                ],
                period,
            ),
            track(
                "gameplay/baby_villager_activity",
                Modifier::Override,
                Easing::Linear,
                vec![
                    frame(10, id("minecraft:idle")),
                    frame(3_000, id("minecraft:play")),
                    frame(6_000, id("minecraft:idle")),
                    frame(10_000, id("minecraft:play")),
                    frame(12_000, id("minecraft:rest")),
                ],
                period,
            ),
        ],
    )
    .expect("locked villager timeline")
}

pub fn locked_timelines() -> BTreeMap<String, Timeline> {
    [
        ("minecraft:day".to_owned(), locked_day_timeline()),
        (
            "minecraft:early_game".to_owned(),
            locked_early_game_timeline(),
        ),
        ("minecraft:moon".to_owned(), locked_moon_timeline()),
        (
            "minecraft:villager_schedule".to_owned(),
            locked_villager_schedule_timeline(),
        ),
    ]
    .into_iter()
    .collect()
}
