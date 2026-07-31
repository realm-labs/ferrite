use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::provider::IntProvider;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::tree_core::{
    FoliageAttachment, TreePlacementContext, TreeWorld,
};
use ferrite_world::generation::feature::tree_foliage::{
    FoliageConfig, FoliageError, FoliageKind, FoliageWorld,
};
use ferrite_world::id::BlockStateId;

#[test]
fn blob_corner_draw_precedes_persistent_admission_and_provider() {
    let position = BlockPos::new(4, 9, 2);
    let mut world = Fixture::default();
    world.states.insert(position, PERSISTENT);
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::with_integers([1]);

    blob()
        .place(&mut context, attachment(position, false), 0, 0, &mut random)
        .unwrap();

    assert_eq!(random.bounds, [2]);
    assert_eq!(context.world().reads, [position]);
    assert!(context.world().samples.is_empty());
    assert!(context.world().offers.is_empty());
}

#[test]
fn admitted_waterloggable_leaf_uses_source_fluid_and_is_attempted_on_rejected_write() {
    let position = BlockPos::new(1, 2, 3);
    let mut world = Fixture::default();
    world.water.insert(position, true);
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::with_integers([1]);

    blob()
        .place(&mut context, attachment(position, false), 0, 0, &mut random)
        .unwrap();

    assert!(context.foliage_attempted(position));
    assert_eq!(context.world().samples, [position]);
    assert_eq!(context.world().offers, [(position, WATERLOGGED, 19)]);
}

#[test]
fn dark_oak_double_row_zero_applies_the_signed_corner_rule() {
    let origin = BlockPos::new(0, 20, 0);
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::with_integers([0]);
    let config = FoliageConfig {
        radius: IntProvider::Constant(0),
        offset: IntProvider::Constant(0),
        kind: FoliageKind::DarkOak,
    };

    config
        .place(&mut context, attachment(origin, true), 4, 0, &mut random)
        .unwrap();

    let row_zero = context
        .world()
        .offers
        .iter()
        .filter(|(position, _, _)| position.y == origin.y)
        .count();
    assert_eq!(row_zero, 55);
    assert_eq!(random.bounds, [2]);
}

#[test]
fn random_spread_preserves_the_next_int_zero_exception_boundary() {
    let config = FoliageConfig {
        radius: IntProvider::Constant(0),
        offset: IntProvider::Constant(0),
        kind: FoliageKind::RandomSpread {
            foliage_height: IntProvider::Constant(1),
            leaf_placement_attempts: 1,
        },
    };
    let mut world = Fixture::default();
    let mut context = TreePlacementContext::new(&mut world);
    let mut random = ScriptedRandom::default();

    assert_eq!(
        config.place(
            &mut context,
            attachment(BlockPos::new(0, 0, 0), false),
            1,
            0,
            &mut random
        ),
        Err(FoliageError::ZeroRandomSpreadRadius)
    );
    assert!(random.bounds.is_empty());
}

#[test]
fn cherry_codec_encoder_reuses_wide_bottom_chance_for_corner_field() {
    let kind = FoliageKind::Cherry {
        height: IntProvider::Constant(4),
        wide_bottom_layer_hole_chance: 0.2,
        corner_hole_chance: 0.8,
        hanging_leaves_chance: 0.3,
        hanging_leaves_extension_chance: 0.4,
    };

    let encoded = kind.encoded_cherry_chances().unwrap();
    assert_eq!(encoded.wide_bottom_layer_hole_chance, 0.2);
    assert_eq!(encoded.corner_hole_chance, 0.2);
}

fn blob() -> FoliageConfig {
    FoliageConfig {
        radius: IntProvider::Constant(0),
        offset: IntProvider::Constant(0),
        kind: FoliageKind::Blob { height: 0 },
    }
}

fn attachment(position: BlockPos, double_trunk: bool) -> FoliageAttachment {
    FoliageAttachment {
        position,
        radius_offset: 0,
        double_trunk,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const LEAF: BlockStateId = BlockStateId::new(1);
const PERSISTENT: BlockStateId = BlockStateId::new(2);
const WATERLOGGED: BlockStateId = BlockStateId::new(3);

#[derive(Debug, Default)]
struct Fixture {
    states: BTreeMap<BlockPos, BlockStateId>,
    water: BTreeMap<BlockPos, bool>,
    reads: Vec<BlockPos>,
    samples: Vec<BlockPos>,
    offers: Vec<(BlockPos, BlockStateId, u32)>,
}

impl TreeWorld for Fixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn maximum_y(&self) -> i32 {
        319
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.reads.push(position);
        self.states.get(&position).copied().unwrap_or(AIR)
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == AIR
    }

    fn is_replaceable_by_trees(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_log(&self, _state: BlockStateId) -> bool {
        false
    }

    fn is_vine(&self, _state: BlockStateId) -> bool {
        false
    }

    fn optional_leaf_distance(&self, _state: BlockStateId) -> Option<u8> {
        None
    }

    fn with_leaf_distance(&self, state: BlockStateId, _distance: u8) -> BlockStateId {
        state
    }

    fn offer_tree_block(&mut self, position: BlockPos, state: BlockStateId, flags: u32) -> bool {
        self.offers.push((position, state, flags));
        false
    }

    fn update_tree_shape_at_edge(
        &mut self,
        _radius: u32,
        _minimum: BlockPos,
        _maximum: BlockPos,
        _filled: &[BlockPos],
    ) {
    }
}

impl FoliageWorld for Fixture {
    fn has_persistent_property_set(&self, state: BlockStateId) -> bool {
        state == PERSISTENT
    }

    fn sample_foliage(
        &mut self,
        position: BlockPos,
        _random: &mut impl GenerationRandom,
    ) -> BlockStateId {
        self.samples.push(position);
        LEAF
    }

    fn supports_waterlogged(&self, state: BlockStateId) -> bool {
        state == LEAF
    }

    fn is_source_water(&mut self, position: BlockPos) -> bool {
        self.water.get(&position).copied().unwrap_or(false)
    }

    fn with_waterlogged(&self, state: BlockStateId, waterlogged: bool) -> BlockStateId {
        if state == LEAF && waterlogged {
            WATERLOGGED
        } else {
            state
        }
    }
}

#[derive(Debug, Default)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn with_integers(values: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: values.into_iter().collect(),
            bounds: Vec::new(),
        }
    }
}

impl GenerationRandom for ScriptedRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        self.bounds.push(bound.get());
        let value = self.integers.pop_front().expect("scripted integer");
        assert!(value < bound.get());
        value
    }

    fn next_f32(&mut self) -> f32 {
        panic!("fixture does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fixture does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fixture does not draw Gaussian values")
    }
}
