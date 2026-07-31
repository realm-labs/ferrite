use std::collections::VecDeque;
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::feature::fossil::{
    FossilBoundingBox, FossilClip, FossilConfig, FossilPlacementSettings, FossilProcessorId,
    FossilRotation, FossilTemplateId, FossilWorld, place_fossil,
};
use ferrite_world::generation::feature::random::GenerationRandom;
use ferrite_world::id::BlockStateId;

#[test]
fn fossil_uses_primary_footprint_eight_ordered_corners_and_paired_processors() {
    let origin = BlockPos::new(0, 100, 0);
    let config = FossilConfig {
        primary_templates: vec![FossilTemplateId(10)],
        overlay_templates: vec![FossilTemplateId(20)],
        primary_processors: vec![FossilProcessorId(1)],
        overlay_processors: vec![FossilProcessorId(2), FossilProcessorId(3)],
        maximum_empty_corners: 4,
    };
    let mut world = FossilFixture::new();
    let mut random = ScriptedRandom::new([2, 0, 9]);

    assert!(place_fossil(&mut world, origin, &config, &mut random, |_| true).unwrap());

    assert_eq!(random.bounds, [4, 1, 10]);
    assert_eq!(
        world.resolutions,
        [FossilTemplateId(10), FossilTemplateId(20)]
    );
    assert_eq!(
        world.height_queries,
        [
            (-1, -2),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -2),
            (0, -1),
            (0, 0),
            (0, 1),
        ]
    );
    assert_eq!(
        world.corner_reads,
        [
            BlockPos::new(0, 68, 1),
            BlockPos::new(-1, 68, 1),
            BlockPos::new(0, 66, 1),
            BlockPos::new(-1, 66, 1),
            BlockPos::new(0, 68, -2),
            BlockPos::new(-1, 68, -2),
            BlockPos::new(0, 66, -2),
            BlockPos::new(-1, 66, -2),
        ]
    );
    assert_eq!(
        world.placements,
        [
            PlacementRecord {
                template: FossilTemplateId(10),
                position: BlockPos::new(-1, 66, -2),
                processors: vec![FossilProcessorId(1)],
                rotation: FossilRotation::Clockwise180,
                clip: expected_clip(),
                flags: 260,
            },
            PlacementRecord {
                template: FossilTemplateId(20),
                position: BlockPos::new(-1, 66, -2),
                processors: vec![FossilProcessorId(2), FossilProcessorId(3)],
                rotation: FossilRotation::Clockwise180,
                clip: expected_clip(),
                flags: 260,
            },
        ]
    );
}

fn expected_clip() -> FossilClip {
    FossilClip {
        minimum: BlockPos::new(-16, -64, -16),
        maximum: BlockPos::new(31, 319, 31),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlacementRecord {
    template: FossilTemplateId,
    position: BlockPos,
    processors: Vec<FossilProcessorId>,
    rotation: FossilRotation,
    clip: FossilClip,
    flags: u32,
}

#[derive(Debug)]
struct FossilFixture {
    resolutions: Vec<FossilTemplateId>,
    height_queries: Vec<(i32, i32)>,
    corner_reads: Vec<BlockPos>,
    placements: Vec<PlacementRecord>,
}

impl FossilFixture {
    fn new() -> Self {
        Self {
            resolutions: Vec::new(),
            height_queries: Vec::new(),
            corner_reads: Vec::new(),
            placements: Vec::new(),
        }
    }
}

impl FossilWorld for FossilFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn maximum_y(&self) -> i32 {
        319
    }

    fn resolve_template(&mut self, identifier: FossilTemplateId) -> bool {
        self.resolutions.push(identifier);
        true
    }

    fn rotated_template_size(
        &mut self,
        _template: FossilTemplateId,
        _rotation: FossilRotation,
    ) -> [i32; 3] {
        [2, 3, 4]
    }

    fn ocean_floor_wg(&mut self, x: i32, z: i32) -> i32 {
        self.height_queries.push((x, z));
        90
    }

    fn transformed_zero_position(
        &mut self,
        _template: FossilTemplateId,
        position: BlockPos,
        _rotation: FossilRotation,
    ) -> BlockPos {
        position
    }

    fn template_bounding_box(
        &mut self,
        _template: FossilTemplateId,
        zero_position: BlockPos,
        _rotation: FossilRotation,
        clip: FossilClip,
    ) -> FossilBoundingBox {
        assert_eq!(clip, expected_clip());
        FossilBoundingBox {
            minimum: zero_position,
            maximum: BlockPos::new(
                zero_position.x + 1,
                zero_position.y + 2,
                zero_position.z + 3,
            ),
        }
    }

    fn block_state(&mut self, position: BlockPos) -> BlockStateId {
        self.corner_reads.push(position);
        if self.corner_reads.len() <= 4 {
            BlockStateId::new(0)
        } else {
            BlockStateId::new(9)
        }
    }

    fn is_air(&self, state: BlockStateId) -> bool {
        state == BlockStateId::new(0)
    }

    fn is_water_or_lava_block_identity(&self, _state: BlockStateId) -> bool {
        false
    }

    fn place_fossil_template<R: GenerationRandom>(
        &mut self,
        template: FossilTemplateId,
        position: BlockPos,
        pivot: BlockPos,
        settings: FossilPlacementSettings<'_>,
        _random: &mut R,
        flags: u32,
    ) -> bool {
        assert_eq!(position, pivot);
        self.placements.push(PlacementRecord {
            template,
            position,
            processors: settings.processors.to_vec(),
            rotation: settings.rotation,
            clip: settings.clip,
            flags,
        });
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
        panic!("fossil feature does not draw floats")
    }

    fn next_f64(&mut self) -> f64 {
        panic!("fossil feature does not draw doubles")
    }

    fn next_gaussian(&mut self) -> f64 {
        panic!("fossil feature does not draw Gaussian values")
    }
}
