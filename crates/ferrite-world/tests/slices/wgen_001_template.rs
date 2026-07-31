use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::generation::feature::template::{
    TemplateFeatureId, TemplateFeatureWorld, TemplateRotation, WeightedTemplateEntry,
    place_template_feature,
};

#[test]
fn template_feature_uses_weight_then_rotation_and_unrotated_axis_halves() {
    let origin = BlockPos::new(10, 50, 20);
    let entries = [
        WeightedTemplateEntry {
            identifier: TemplateFeatureId(10),
            weight: NonZeroU32::new(2).unwrap(),
            rotations: vec![TemplateRotation::None],
        },
        WeightedTemplateEntry {
            identifier: TemplateFeatureId(20),
            weight: NonZeroU32::new(3).unwrap(),
            rotations: vec![TemplateRotation::Clockwise90],
        },
    ];
    let mut world = TemplateFixture::new();
    let mut random = ScriptedRandom::new([2, 0]);

    assert!(!place_template_feature(&mut world, origin, &entries, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [5, 1]);
    assert_eq!(world.resolved, [TemplateFeatureId(20)]);
    assert_eq!(
        world.placement,
        Some((
            TemplateFeatureId(20),
            BlockPos::new(14, 50, 16),
            BlockPos::new(14, 50, 16),
            TemplateRotation::Clockwise90,
            3,
        ))
    );
}

#[derive(Debug)]
struct TemplateFixture {
    resolved: Vec<TemplateFeatureId>,
    placement: Option<(TemplateFeatureId, BlockPos, BlockPos, TemplateRotation, u32)>,
}

impl TemplateFixture {
    fn new() -> Self {
        Self {
            resolved: Vec::new(),
            placement: None,
        }
    }
}

impl TemplateFeatureWorld for TemplateFixture {
    fn resolve_template(&mut self, identifier: TemplateFeatureId) -> bool {
        self.resolved.push(identifier);
        true
    }

    fn unrotated_template_size(&mut self, _identifier: TemplateFeatureId) -> [i32; 3] {
        [8, 11, 9]
    }

    fn place_template_feature<R: GenerationRandom>(
        &mut self,
        identifier: TemplateFeatureId,
        position: BlockPos,
        pivot: BlockPos,
        rotation: TemplateRotation,
        _random: &mut R,
        flags: u32,
    ) -> bool {
        self.placement = Some((identifier, position, pivot, rotation, flags));
        false
    }
}

#[derive(Debug)]
struct ScriptedRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl ScriptedRandom {
    fn new(integers: impl IntoIterator<Item = u32>) -> Self {
        Self {
            integers: integers.into_iter().collect(),
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
        panic!("template feature does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("template feature does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("template feature does not draw Gaussian values")
    }
}
