//! Locked Overworld and Nether multi-noise preset construction.

use crate::generation::biome::{ClimateInterval, ClimatePoint};
use crate::id::BiomeId;

type Range = [f32; 2];

const FULL: Range = [-1.0, 1.0];
const TEMPERATURES: [Range; 5] = [
    [-1.0, -0.45],
    [-0.45, -0.15],
    [-0.15, 0.2],
    [0.2, 0.55],
    [0.55, 1.0],
];
const HUMIDITIES: [Range; 5] = [
    [-1.0, -0.35],
    [-0.35, -0.1],
    [-0.1, 0.1],
    [0.1, 0.3],
    [0.3, 1.0],
];
const EROSIONS: [Range; 7] = [
    [-1.0, -0.78],
    [-0.78, -0.375],
    [-0.375, -0.2225],
    [-0.2225, 0.05],
    [0.05, 0.45],
    [0.45, 0.55],
    [0.55, 1.0],
];
const MUSHROOM: Range = [-1.2, -1.05];
const DEEP_OCEAN: Range = [-1.05, -0.455];
const OCEAN: Range = [-0.455, -0.19];
const COAST: Range = [-0.19, -0.11];
const INLAND: Range = [-0.11, 0.55];
const NEAR: Range = [-0.11, 0.03];
const MID: Range = [0.03, 0.3];
const FAR: Range = [0.3, 1.0];

const OCEANS: [[&str; 5]; 2] = [
    [
        "deep_frozen_ocean",
        "deep_cold_ocean",
        "deep_ocean",
        "deep_lukewarm_ocean",
        "warm_ocean",
    ],
    [
        "frozen_ocean",
        "cold_ocean",
        "ocean",
        "lukewarm_ocean",
        "warm_ocean",
    ],
];
const MIDDLE: [[&str; 5]; 5] = [
    [
        "snowy_plains",
        "snowy_plains",
        "snowy_plains",
        "snowy_taiga",
        "taiga",
    ],
    [
        "plains",
        "plains",
        "forest",
        "taiga",
        "old_growth_spruce_taiga",
    ],
    [
        "flower_forest",
        "plains",
        "forest",
        "birch_forest",
        "dark_forest",
    ],
    ["savanna", "savanna", "forest", "jungle", "jungle"],
    ["desert", "desert", "desert", "desert", "desert"],
];
const MIDDLE_VARIANT: [[Option<&str>; 5]; 5] = [
    [Some("ice_spikes"), None, Some("snowy_taiga"), None, None],
    [None, None, None, None, Some("old_growth_pine_taiga")],
    [
        Some("sunflower_plains"),
        None,
        None,
        Some("old_growth_birch_forest"),
        None,
    ],
    [
        None,
        None,
        Some("plains"),
        Some("sparse_jungle"),
        Some("bamboo_jungle"),
    ],
    [None; 5],
];
const PLATEAU: [[&str; 5]; 5] = [
    [
        "snowy_plains",
        "snowy_plains",
        "snowy_plains",
        "snowy_taiga",
        "snowy_taiga",
    ],
    [
        "meadow",
        "meadow",
        "forest",
        "taiga",
        "old_growth_spruce_taiga",
    ],
    ["meadow", "meadow", "meadow", "meadow", "pale_garden"],
    [
        "savanna_plateau",
        "savanna_plateau",
        "forest",
        "forest",
        "jungle",
    ],
    [
        "badlands",
        "badlands",
        "badlands",
        "wooded_badlands",
        "wooded_badlands",
    ],
];
const PLATEAU_VARIANT: [[Option<&str>; 5]; 5] = [
    [Some("ice_spikes"), None, None, None, None],
    [
        Some("cherry_grove"),
        None,
        Some("meadow"),
        Some("meadow"),
        Some("old_growth_pine_taiga"),
    ],
    [
        Some("cherry_grove"),
        Some("cherry_grove"),
        Some("forest"),
        Some("birch_forest"),
        None,
    ],
    [None; 5],
    [
        Some("eroded_badlands"),
        Some("eroded_badlands"),
        None,
        None,
        None,
    ],
];
const SHATTERED: [[Option<&str>; 5]; 5] = [
    [
        Some("windswept_gravelly_hills"),
        Some("windswept_gravelly_hills"),
        Some("windswept_hills"),
        Some("windswept_forest"),
        Some("windswept_forest"),
    ],
    [
        Some("windswept_gravelly_hills"),
        Some("windswept_gravelly_hills"),
        Some("windswept_hills"),
        Some("windswept_forest"),
        Some("windswept_forest"),
    ],
    [
        Some("windswept_hills"),
        Some("windswept_hills"),
        Some("windswept_hills"),
        Some("windswept_forest"),
        Some("windswept_forest"),
    ],
    [None; 5],
    [None; 5],
];

#[derive(Debug, Clone, PartialEq)]
pub struct NamedClimatePoint {
    pub parameters: [Range; 6],
    pub offset: f32,
    pub biome: &'static str,
}

impl NamedClimatePoint {
    pub fn resolve(&self, biome: BiomeId) -> ClimatePoint {
        ClimatePoint {
            parameters: self
                .parameters
                .map(|range| ClimateInterval::quantized(range[0], range[1])),
            offset: i64::from((self.offset * 10_000.0) as i32),
            biome,
        }
    }
}

pub fn overworld_points() -> Vec<NamedClimatePoint> {
    let mut builder = Builder::default();
    builder.add_offshore();
    for (kind, weirdness) in [
        (Slice::Mid, [-1.0, -0.93333334]),
        (Slice::High, [-0.93333334, -0.7666667]),
        (Slice::Peaks, [-0.7666667, -0.56666666]),
        (Slice::High, [-0.56666666, -0.4]),
        (Slice::Mid, [-0.4, -0.26666668]),
        (Slice::Low, [-0.26666668, -0.05]),
        (Slice::Valley, [-0.05, 0.05]),
        (Slice::Low, [0.05, 0.26666668]),
        (Slice::Mid, [0.26666668, 0.4]),
        (Slice::High, [0.4, 0.56666666]),
        (Slice::Peaks, [0.56666666, 0.7666667]),
        (Slice::High, [0.7666667, 0.93333334]),
        (Slice::Mid, [0.93333334, 1.0]),
    ] {
        match kind {
            Slice::Peaks => builder.add_peaks(weirdness),
            Slice::High => builder.add_high(weirdness),
            Slice::Mid => builder.add_mid(weirdness),
            Slice::Low => builder.add_low(weirdness),
            Slice::Valley => builder.add_valley(weirdness),
        }
    }
    builder.add_underground();
    builder.points
}

pub fn overworld_spawn_targets() -> [[Range; 6]; 2] {
    [
        [FULL, FULL, [-0.11, 1.0], FULL, [0.0, 0.0], [-1.0, -0.16]],
        [FULL, FULL, [-0.11, 1.0], FULL, [0.0, 0.0], [0.16, 1.0]],
    ]
}

pub fn nether_points() -> Vec<NamedClimatePoint> {
    [
        ([0.0, 0.0], [0.0, 0.0], "nether_wastes", 0.0),
        ([0.0, 0.0], [-0.5, -0.5], "soul_sand_valley", 0.0),
        ([0.4, 0.4], [0.0, 0.0], "crimson_forest", 0.0),
        ([0.0, 0.0], [0.5, 0.5], "warped_forest", 0.375),
        ([-0.5, -0.5], [0.0, 0.0], "basalt_deltas", 0.175),
    ]
    .into_iter()
    .map(|(temperature, humidity, biome, offset)| NamedClimatePoint {
        parameters: [
            temperature,
            humidity,
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
            [0.0, 0.0],
        ],
        offset,
        biome,
    })
    .collect()
}

pub fn is_deep_dark_region(erosion: f64, depth: f64) -> bool {
    erosion < f64::from(-0.225_f32) && depth > f64::from(0.9_f32)
}

#[derive(Debug, Clone, Copy)]
enum Slice {
    Peaks,
    High,
    Mid,
    Low,
    Valley,
}

#[derive(Default)]
struct Builder {
    points: Vec<NamedClimatePoint>,
}

impl Builder {
    fn surface(&mut self, t: Range, h: Range, c: Range, e: Range, w: Range, biome: &'static str) {
        for depth in [[0.0, 0.0], [1.0, 1.0]] {
            self.points.push(NamedClimatePoint {
                parameters: [t, h, c, e, depth, w],
                offset: 0.0,
                biome,
            });
        }
    }

    fn add_offshore(&mut self) {
        self.surface(FULL, FULL, MUSHROOM, FULL, FULL, "mushroom_fields");
        for (temperature, (deep, ordinary)) in TEMPERATURES
            .into_iter()
            .zip(OCEANS[0].into_iter().zip(OCEANS[1]))
        {
            self.surface(temperature, FULL, DEEP_OCEAN, FULL, FULL, deep);
            self.surface(temperature, FULL, OCEAN, FULL, FULL, ordinary);
        }
    }

    fn add_peaks(&mut self, weirdness: Range) {
        self.for_grid(weirdness, |this, t, h, ti, hi, w| {
            let middle = pick_middle(ti, hi, w);
            let badlands = pick_badlands_or_middle(ti, hi, w);
            let slope = pick_slope_or_badlands(ti, hi, w);
            let plateau = pick_plateau(ti, hi, w);
            let shattered = pick_shattered(ti, hi, w);
            let windswept = maybe_windswept(ti, hi, w, shattered);
            let peak = pick_peak(ti, hi, w);
            for (c, e, biome) in [
                (span(COAST, FAR), EROSIONS[0], peak),
                (span(COAST, NEAR), EROSIONS[1], slope),
                (span(MID, FAR), EROSIONS[1], peak),
                (span(COAST, NEAR), span(EROSIONS[2], EROSIONS[3]), middle),
                (span(MID, FAR), EROSIONS[2], plateau),
                (MID, EROSIONS[3], badlands),
                (FAR, EROSIONS[3], plateau),
                (span(COAST, FAR), EROSIONS[4], middle),
                (span(COAST, NEAR), EROSIONS[5], windswept),
                (span(MID, FAR), EROSIONS[5], shattered),
                (span(COAST, FAR), EROSIONS[6], middle),
            ] {
                this.surface(t, h, c, e, w, biome);
            }
        });
    }

    fn add_high(&mut self, weirdness: Range) {
        self.for_grid(weirdness, |this, t, h, ti, hi, w| {
            let middle = pick_middle(ti, hi, w);
            let badlands = pick_badlands_or_middle(ti, hi, w);
            let cold_slope = pick_slope_or_badlands(ti, hi, w);
            let plateau = pick_plateau(ti, hi, w);
            let shattered = pick_shattered(ti, hi, w);
            let windswept = maybe_windswept(ti, hi, w, middle);
            let slope = pick_slope(ti, hi, w);
            let peak = pick_peak(ti, hi, w);
            for (c, e, biome) in [
                (COAST, span(EROSIONS[0], EROSIONS[1]), middle),
                (NEAR, EROSIONS[0], slope),
                (span(MID, FAR), EROSIONS[0], peak),
                (NEAR, EROSIONS[1], cold_slope),
                (span(MID, FAR), EROSIONS[1], slope),
                (span(COAST, NEAR), span(EROSIONS[2], EROSIONS[3]), middle),
                (span(MID, FAR), EROSIONS[2], plateau),
                (MID, EROSIONS[3], badlands),
                (FAR, EROSIONS[3], plateau),
                (span(COAST, FAR), EROSIONS[4], middle),
                (span(COAST, NEAR), EROSIONS[5], windswept),
                (span(MID, FAR), EROSIONS[5], shattered),
                (span(COAST, FAR), EROSIONS[6], middle),
            ] {
                this.surface(t, h, c, e, w, biome);
            }
        });
    }

    fn add_mid(&mut self, weirdness: Range) {
        self.shared_rows(weirdness);
        self.for_grid(weirdness, |this, t, h, ti, hi, w| {
            let middle = pick_middle(ti, hi, w);
            let badlands = pick_badlands_or_middle(ti, hi, w);
            let cold_slope = pick_slope_or_badlands(ti, hi, w);
            let shattered = pick_shattered(ti, hi, w);
            let plateau = pick_plateau(ti, hi, w);
            let beach = pick_beach(ti);
            let windswept = maybe_windswept(ti, hi, w, middle);
            let coast = pick_shattered_coast(ti, hi, w);
            let slope = pick_slope(ti, hi, w);
            for (c, e, biome) in [
                (span(NEAR, FAR), EROSIONS[0], slope),
                (span(NEAR, MID), EROSIONS[1], cold_slope),
                (FAR, EROSIONS[1], if ti == 0 { slope } else { plateau }),
                (NEAR, EROSIONS[2], middle),
                (MID, EROSIONS[2], badlands),
                (FAR, EROSIONS[2], plateau),
                (span(COAST, NEAR), EROSIONS[3], middle),
                (span(MID, FAR), EROSIONS[3], badlands),
            ] {
                this.surface(t, h, c, e, w, biome);
            }
            if !positive(w) {
                this.surface(t, h, COAST, EROSIONS[4], w, beach);
                this.surface(t, h, span(NEAR, FAR), EROSIONS[4], w, middle);
            } else {
                this.surface(t, h, span(COAST, FAR), EROSIONS[4], w, middle);
            }
            for (c, e, biome) in [
                (COAST, EROSIONS[5], coast),
                (NEAR, EROSIONS[5], windswept),
                (span(MID, FAR), EROSIONS[5], shattered),
                (COAST, EROSIONS[6], if positive(w) { middle } else { beach }),
            ] {
                this.surface(t, h, c, e, w, biome);
            }
            if ti == 0 {
                this.surface(t, h, span(NEAR, FAR), EROSIONS[6], w, middle);
            }
        });
    }

    fn add_low(&mut self, weirdness: Range) {
        self.shared_rows(weirdness);
        self.for_grid(weirdness, |this, t, h, ti, hi, w| {
            let middle = pick_middle(ti, hi, w);
            let badlands = pick_badlands_or_middle(ti, hi, w);
            let cold_slope = pick_slope_or_badlands(ti, hi, w);
            let beach = pick_beach(ti);
            let windswept = maybe_windswept(ti, hi, w, middle);
            let coast = pick_shattered_coast(ti, hi, w);
            for (c, e, biome) in [
                (NEAR, span(EROSIONS[0], EROSIONS[1]), badlands),
                (span(MID, FAR), span(EROSIONS[0], EROSIONS[1]), cold_slope),
                (NEAR, span(EROSIONS[2], EROSIONS[3]), middle),
                (span(MID, FAR), span(EROSIONS[2], EROSIONS[3]), badlands),
                (COAST, span(EROSIONS[3], EROSIONS[4]), beach),
                (span(NEAR, FAR), EROSIONS[4], middle),
                (COAST, EROSIONS[5], coast),
                (NEAR, EROSIONS[5], windswept),
                (span(MID, FAR), EROSIONS[5], middle),
                (COAST, EROSIONS[6], beach),
            ] {
                this.surface(t, h, c, e, w, biome);
            }
            if ti == 0 {
                this.surface(t, h, span(NEAR, FAR), EROSIONS[6], w, middle);
            }
        });
    }

    fn add_valley(&mut self, weirdness: Range) {
        let frozen = TEMPERATURES[0];
        let unfrozen = span(TEMPERATURES[1], TEMPERATURES[4]);
        let river_coast = if positive(weirdness) {
            ("frozen_river", "river")
        } else {
            ("stony_shore", "stony_shore")
        };
        for (t, c, e, biome) in [
            (frozen, COAST, span(EROSIONS[0], EROSIONS[1]), river_coast.0),
            (
                unfrozen,
                COAST,
                span(EROSIONS[0], EROSIONS[1]),
                river_coast.1,
            ),
            (frozen, NEAR, span(EROSIONS[0], EROSIONS[1]), "frozen_river"),
            (unfrozen, NEAR, span(EROSIONS[0], EROSIONS[1]), "river"),
            (
                frozen,
                span(COAST, FAR),
                span(EROSIONS[2], EROSIONS[5]),
                "frozen_river",
            ),
            (
                unfrozen,
                span(COAST, FAR),
                span(EROSIONS[2], EROSIONS[5]),
                "river",
            ),
            (frozen, COAST, EROSIONS[6], "frozen_river"),
            (unfrozen, COAST, EROSIONS[6], "river"),
            (
                span(TEMPERATURES[1], TEMPERATURES[2]),
                span(INLAND, FAR),
                EROSIONS[6],
                "swamp",
            ),
            (
                span(TEMPERATURES[3], TEMPERATURES[4]),
                span(INLAND, FAR),
                EROSIONS[6],
                "mangrove_swamp",
            ),
            (frozen, span(INLAND, FAR), EROSIONS[6], "frozen_river"),
        ] {
            self.surface(t, FULL, c, e, weirdness, biome);
        }
        self.for_grid(weirdness, |this, t, h, ti, hi, w| {
            this.surface(
                t,
                h,
                span(MID, FAR),
                span(EROSIONS[0], EROSIONS[1]),
                w,
                pick_badlands_or_middle(ti, hi, w),
            );
        });
    }

    fn shared_rows(&mut self, weirdness: Range) {
        self.surface(
            FULL,
            FULL,
            COAST,
            span(EROSIONS[0], EROSIONS[2]),
            weirdness,
            "stony_shore",
        );
        self.surface(
            span(TEMPERATURES[1], TEMPERATURES[2]),
            FULL,
            span(NEAR, FAR),
            EROSIONS[6],
            weirdness,
            "swamp",
        );
        self.surface(
            span(TEMPERATURES[3], TEMPERATURES[4]),
            FULL,
            span(NEAR, FAR),
            EROSIONS[6],
            weirdness,
            "mangrove_swamp",
        );
    }

    fn add_underground(&mut self) {
        self.points.extend([
            named(
                [FULL, FULL, [0.8, 1.0], FULL, [0.2, 0.9], FULL],
                "dripstone_caves",
            ),
            named(
                [FULL, [0.7, 1.0], FULL, FULL, [0.2, 0.9], FULL],
                "lush_caves",
            ),
            named(
                [
                    FULL,
                    FULL,
                    span(COAST, INLAND),
                    span(EROSIONS[5], EROSIONS[6]),
                    [0.2, 0.9],
                    [-1.1, -0.85],
                ],
                "sulfur_caves",
            ),
            named(
                [
                    FULL,
                    FULL,
                    FULL,
                    span(EROSIONS[0], EROSIONS[1]),
                    [1.1, 1.1],
                    FULL,
                ],
                "deep_dark",
            ),
        ]);
    }

    fn for_grid(
        &mut self,
        weirdness: Range,
        mut emit: impl FnMut(&mut Self, Range, Range, usize, usize, Range),
    ) {
        for (temperature_index, temperature) in TEMPERATURES.into_iter().enumerate() {
            for (humidity_index, humidity) in HUMIDITIES.into_iter().enumerate() {
                emit(
                    self,
                    temperature,
                    humidity,
                    temperature_index,
                    humidity_index,
                    weirdness,
                );
            }
        }
    }
}

fn named(parameters: [Range; 6], biome: &'static str) -> NamedClimatePoint {
    NamedClimatePoint {
        parameters,
        offset: 0.0,
        biome,
    }
}

fn span(first: Range, last: Range) -> Range {
    [first[0], last[1]]
}

fn positive(weirdness: Range) -> bool {
    (weirdness[1] * 10_000.0) as i64 >= 0
}

fn pick_middle(t: usize, h: usize, w: Range) -> &'static str {
    if positive(w) {
        MIDDLE_VARIANT[t][h].unwrap_or(MIDDLE[t][h])
    } else {
        MIDDLE[t][h]
    }
}

fn pick_badlands(h: usize, w: Range) -> &'static str {
    match h {
        0 | 1 if positive(w) => "eroded_badlands",
        0..=2 => "badlands",
        _ => "wooded_badlands",
    }
}

fn pick_badlands_or_middle(t: usize, h: usize, w: Range) -> &'static str {
    if t == 4 {
        pick_badlands(h, w)
    } else {
        pick_middle(t, h, w)
    }
}

fn pick_plateau(t: usize, h: usize, w: Range) -> &'static str {
    if positive(w) {
        PLATEAU_VARIANT[t][h].unwrap_or(PLATEAU[t][h])
    } else {
        PLATEAU[t][h]
    }
}

fn pick_slope(t: usize, h: usize, w: Range) -> &'static str {
    if t >= 3 {
        pick_plateau(t, h, w)
    } else if h <= 1 {
        "snowy_slopes"
    } else {
        "grove"
    }
}

fn pick_slope_or_badlands(t: usize, h: usize, w: Range) -> &'static str {
    if t == 0 {
        pick_slope(t, h, w)
    } else {
        pick_badlands_or_middle(t, h, w)
    }
}

fn pick_peak(t: usize, h: usize, w: Range) -> &'static str {
    match t {
        0..=2 if positive(w) => "frozen_peaks",
        0..=2 => "jagged_peaks",
        3 => "stony_peaks",
        _ => pick_badlands(h, w),
    }
}

fn pick_shattered(t: usize, h: usize, w: Range) -> &'static str {
    SHATTERED[t][h].unwrap_or_else(|| pick_middle(t, h, w))
}

fn maybe_windswept(t: usize, h: usize, w: Range, fallback: &'static str) -> &'static str {
    if t > 1 && h < 4 && positive(w) {
        "windswept_savanna"
    } else {
        fallback
    }
}

fn pick_beach(t: usize) -> &'static str {
    match t {
        0 => "snowy_beach",
        4 => "desert",
        _ => "beach",
    }
}

fn pick_shattered_coast(t: usize, h: usize, w: Range) -> &'static str {
    let fallback = if positive(w) {
        pick_middle(t, h, w)
    } else {
        pick_beach(t)
    };
    maybe_windswept(t, h, w, fallback)
}
