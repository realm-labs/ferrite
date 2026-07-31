use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::provider::{HeightAnchor, HeightContext};
use ferrite_world::generation::surface::{SurfaceWorld, build_surface_column};
use ferrite_world::generation::surface_rule::{
    StoneSurface, SurfaceCondition, SurfaceContext, SurfaceEnvironment, SurfaceRule,
};
use ferrite_world::id::BlockStateId;

#[test]
fn noise_thresholds_are_closed_and_cache_by_2d_or_3d_epoch() {
    let mut world = Fixture {
        condition_noise: 0.5,
        ..Fixture::default()
    };
    let mut context = context(&mut world);
    context.update_xz(2, 3);
    context.update_y(4, 1, 1, None);
    let two_d = SurfaceCondition::<u8>::NoiseThreshold {
        key: "surface".to_owned(),
        minimum: 0.5,
        maximum: 0.5,
        is_3d: false,
    };
    let three_d = SurfaceCondition::<u8>::NoiseThreshold {
        key: "cave".to_owned(),
        minimum: 0.5,
        maximum: 0.5,
        is_3d: true,
    };

    assert!(two_d.test(&mut context));
    assert!(two_d.test(&mut context));
    assert!(three_d.test(&mut context));
    assert!(three_d.test(&mut context));
    context.update_y(3, 2, 1, None);
    assert!(two_d.test(&mut context));
    assert!(three_d.test(&mut context));

    assert_eq!(
        context.environment_mut().noise_calls,
        [
            ("surface".to_owned(), BlockPos::new(2, 0, 3), false),
            ("cave".to_owned(), BlockPos::new(2, 4, 3), true),
            ("cave".to_owned(), BlockPos::new(2, 3, 3), true),
        ]
    );
}

#[test]
fn vertical_gradient_uses_endpoint_shortcuts_and_strict_float_gate() {
    let mut world = Fixture {
        gradient_random: 0.5,
        ..Fixture::default()
    };
    let mut context = context(&mut world);
    context.update_xz(0, 0);
    let condition = SurfaceCondition::<u8>::VerticalGradient {
        random_name: "bedrock".to_owned(),
        true_at_and_below: HeightAnchor::Absolute(0),
        false_at_and_above: HeightAnchor::Absolute(10),
    };

    context.update_y(0, 1, 1, None);
    assert!(condition.test(&mut context));
    context.update_y(10, 1, 1, None);
    assert!(!condition.test(&mut context));
    context.update_y(5, 1, 1, None);
    assert!(!condition.test(&mut context));

    assert_eq!(context.environment_mut().gradient_calls, 1);
}

#[test]
fn rule_sequence_stops_at_first_non_null_and_skips_false_follow_up() {
    let mut world = Fixture::default();
    let mut context = context(&mut world);
    context.update_xz(0, 0);
    context.update_y(0, 1, 1, None);
    let rule = SurfaceRule::Sequence(vec![
        SurfaceRule::Condition {
            condition: SurfaceCondition::BiomeNever,
            follow_up: Box::new(SurfaceRule::Bandlands),
        },
        SurfaceRule::Block(REPLACEMENT),
        SurfaceRule::Bandlands,
    ]);

    assert_eq!(rule.evaluate(&mut context), Some(REPLACEMENT));
    assert_eq!(context.environment_mut().band_calls, 0);
}

#[test]
fn column_scan_fixes_depth_below_and_replaces_only_first_default_layer() {
    let mut world = Fixture::default();
    world.states.insert(BlockPos::new(0, 4, 0), AIR);
    world.states.insert(BlockPos::new(0, 3, 0), WATER);
    world.states.insert(BlockPos::new(0, 2, 0), STONE);
    world.states.insert(BlockPos::new(0, 1, 0), STONE);
    world.states.insert(BlockPos::new(0, 0, 0), AIR);
    let rule = SurfaceRule::Condition {
        condition: SurfaceCondition::StoneDepth {
            surface: StoneSurface::Floor,
            offset: 0,
            add_surface_depth: false,
            secondary_depth_range: 0,
        },
        follow_up: Box::new(SurfaceRule::Block(REPLACEMENT)),
    };
    let mut context = context(&mut world);

    build_surface_column(&mut context, &rule, 0, 0, 4, 0, STONE).unwrap();

    assert_eq!(
        context.environment_mut().offers,
        [(BlockPos::new(0, 2, 0), REPLACEMENT)]
    );
}

fn context(world: &mut Fixture) -> SurfaceContext<'_, Fixture> {
    SurfaceContext::new(
        world,
        HeightContext {
            minimum_y: 0,
            depth: 64,
        },
        32,
    )
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const WATER: BlockStateId = BlockStateId::new(2);
const REPLACEMENT: BlockStateId = BlockStateId::new(3);
const BAND: BlockStateId = BlockStateId::new(4);

#[derive(Debug)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    offers: Vec<(BlockPos, BlockStateId)>,
    condition_noise: f64,
    noise_calls: Vec<(String, BlockPos, bool)>,
    gradient_random: f32,
    gradient_calls: usize,
    band_calls: usize,
}

impl Default for Fixture {
    fn default() -> Self {
        Self {
            states: BTreeMap::new(),
            offers: Vec::new(),
            condition_noise: 0.0,
            noise_calls: Vec::new(),
            gradient_random: 0.0,
            gradient_calls: 0,
            band_calls: 0,
        }
    }
}

impl SurfaceEnvironment for Fixture {
    type Biome = u8;

    fn surface_noise(&mut self, _x: i32, _z: i32) -> f64 {
        0.0
    }

    fn surface_depth_random(&mut self, _x: i32, _z: i32) -> f64 {
        0.0
    }

    fn preliminary_surface(&mut self, _x: i32, _z: i32) -> i32 {
        20
    }

    fn secondary_noise(&mut self, _x: i32, _z: i32) -> f64 {
        0.0
    }

    fn condition_noise(&mut self, key: &str, position: BlockPos, is_3d: bool) -> f64 {
        self.noise_calls.push((key.to_owned(), position, is_3d));
        self.condition_noise
    }

    fn vertical_gradient_random(&mut self, _name: &str, _position: BlockPos) -> f32 {
        self.gradient_calls += 1;
        self.gradient_random
    }

    fn biome(&mut self, _position: BlockPos) -> Self::Biome {
        1
    }

    fn cold_enough_to_snow(
        &mut self,
        _biome: Self::Biome,
        _position: BlockPos,
        _sea_level: i32,
    ) -> bool {
        false
    }

    fn surface_height(&mut self, _local_x: u8, _local_z: u8) -> i32 {
        0
    }

    fn bandlands_state(&mut self, _position: BlockPos) -> BlockStateId {
        self.band_calls += 1;
        BAND
    }
}

impl SurfaceWorld for Fixture {
    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.states.get(&position).copied().unwrap_or(AIR)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn has_nonempty_fluid(&self, state: BlockStateId) -> bool {
        state == WATER
    }

    fn offer_surface(&mut self, position: BlockPos, state: BlockStateId) -> bool {
        self.offers.push((position, state));
        true
    }
}
