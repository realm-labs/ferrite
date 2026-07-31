use std::collections::BTreeSet;

use ferrite_world::generation::blending::{
    BlendFlatCache, Blender, BlendingData, Direction8, OldBlock, OldChunkColumnSource,
    height_to_offset,
};
use ferrite_world::generation::density::DensityContext;
use ferrite_world::id::BiomeId;

const OLD_BIOME: BiomeId = BiomeId::new(7);
const NEW_BIOME: BiomeId = BiomeId::new(8);

struct LayeredColumn {
    primed: Option<i32>,
    surface_y: i32,
}

impl OldChunkColumnSource for LayeredColumn {
    fn primed_surface_height(&self, _block_x: i32, _block_z: i32) -> Option<i32> {
        self.primed
    }

    fn block(&self, _block_x: i32, block_y: i32, _block_z: i32) -> OldBlock {
        if block_y == self.surface_y {
            OldBlock::Surface
        } else if block_y < self.surface_y {
            OldBlock::Solid
        } else {
            OldBlock::Air
        }
    }
}

fn column_data(cell_x: i32, cell_z: i32, height: f64, density: f64) -> BlendingData {
    let mut data = BlendingData::new(0, 2);
    assert!(data.set_boundary_column(cell_x, cell_z, height, vec![density; 4]));
    data
}

#[test]
fn boundary_calculation_obeys_exact_direction_gates_and_is_idempotent() {
    let source = LayeredColumn {
        primed: Some(20),
        surface_y: 13,
    };
    let mut data = BlendingData::new(0, 2);
    data.calculate_boundary_columns(
        &source,
        &BTreeSet::from([
            Direction8::East,
            Direction8::NorthEast,
            Direction8::South,
            Direction8::SouthEast,
        ]),
    );

    assert_eq!(data.get_height(4, 0), Some(13.0));
    assert_eq!(data.get_height(4, 1), Some(13.0));
    assert_eq!(data.get_height(4, 4), Some(13.0));
    assert_eq!(data.get_height(0, 4), Some(13.0));
    assert_eq!(data.get_height(0, 0), None);

    let changed = LayeredColumn {
        primed: Some(20),
        surface_y: 7,
    };
    data.calculate_boundary_columns(&changed, &BTreeSet::from([Direction8::North]));
    assert_eq!(data.get_height(1, 0), None);
}

#[test]
fn density_column_has_locked_minimum_cell_and_surface_correction() {
    let source = LayeredColumn {
        primed: None,
        surface_y: 13,
    };
    let mut data = BlendingData::new(0, 2);
    data.calculate_boundary_columns(&source, &BTreeSet::from([Direction8::North]));

    assert_eq!(data.get_density(0, 0, 0), Some(0.1));
    assert_eq!(data.get_density(1, 1, 0), Some(0.4));
    assert_eq!(data.get_density(1, 2, 0), Some(-2.0 / 11.0));
    assert_eq!(data.get_density(1, 5, 0), None);
}

#[test]
fn packed_height_round_trip_uses_max_value_for_absent_columns() {
    let mut data = BlendingData::new(-4, 20);
    assert!(data.set_boundary_column(0, 0, 64.0, vec![0.0; 48]));
    let packed = data.packed_heights().expect("one height is present");
    assert_eq!(packed[3], 64.0);
    assert_eq!(packed[0], f64::MAX);

    let unpacked = BlendingData::from_packed_heights(-4, 20, Some(packed));
    assert_eq!(unpacked.get_height(0, 0), Some(64.0));
    assert_eq!(unpacked.get_height(3, 0), None);
}

#[test]
fn direct_lookup_prefers_owner_then_northwest_west_and_north() {
    let owner = column_data(0, 0, 10.0, 1.0);
    let northwest = column_data(4, 4, 20.0, 2.0);
    let west = column_data(4, 0, 30.0, 3.0);
    let north = column_data(0, 4, 40.0, 4.0);

    let blender = Blender::new(
        vec![
            ((0, 0), owner),
            ((-1, -1), northwest),
            ((-1, 0), west),
            ((0, -1), north),
        ],
        Vec::new(),
    );
    assert_eq!(
        blender.blend_offset_and_factor(0, 0).offset,
        height_to_offset(10.0)
    );
}

#[test]
fn exact_old_samples_replace_new_height_and_density_values() {
    let height = column_data(0, 0, 72.0, 2.5);
    let density = height.clone();
    let blender = Blender::new(vec![((0, 0), height)], vec![((0, 0), density)]);

    let output = blender.blend_offset_and_factor(0, 0);
    assert_eq!(output.alpha, 0.0);
    assert_eq!(output.offset, height_to_offset(72.0));
    assert_eq!(
        blender.blend_density(DensityContext { x: 0, y: 8, z: 0 }, -9.0),
        0.25
    );
}

#[test]
fn weighted_blending_uses_locked_ranges_and_smoothstep() {
    let data = column_data(0, 0, 64.0, 2.0);
    let blender = Blender::new(vec![((0, 0), data.clone())], vec![((0, 0), data)]);

    let output = blender.blend_offset_and_factor(8, 0);
    let t = 2.0_f64 / 28.0;
    assert_eq!(output.alpha, 3.0 * t * t - 2.0 * t * t * t);
    assert_eq!(output.offset, height_to_offset(64.0));

    let blended = blender.blend_density(DensityContext { x: 8, y: 8, z: 0 }, 1.0);
    assert_eq!(blended, 0.2 + (2.0 / 3.0) * (1.0 - 0.2));
}

#[test]
fn empty_blender_preserves_fallback_markers_and_density_arrays() {
    let blender = Blender::empty();
    assert!(blender.is_empty());
    assert_eq!(
        blender.blend_offset_and_factor(-100, 100),
        ferrite_world::generation::blending::BlendingOutput {
            alpha: 1.0,
            offset: 0.0,
        }
    );
    let contexts = [
        DensityContext { x: 0, y: -1, z: 0 },
        DensityContext { x: 4, y: 8, z: 4 },
    ];
    let mut values = [0.25, -0.75];
    blender.blend_density_array(&contexts, &mut values);
    assert_eq!(values, [0.25, -0.75]);
}

#[test]
fn biome_blending_uses_nearest_strict_tie_and_shifted_half_gate() {
    let mut first = column_data(0, 0, 64.0, 0.0);
    assert!(first.set_boundary_biomes(0, 0, vec![Some(OLD_BIOME); 8]));
    let mut tied = column_data(4, 0, 64.0, 0.0);
    assert!(tied.set_boundary_biomes(4, 0, vec![Some(NEW_BIOME); 8]));
    let blender = Blender::new(vec![((0, 0), first), ((-1, 0), tied)], Vec::new());

    assert_eq!(
        blender.resolve_biome(0, 0, 0, |_, _| 0.0, || NEW_BIOME),
        OLD_BIOME
    );
    assert_eq!(
        blender.resolve_biome(0, 0, 0, |_, _| 2.0, || NEW_BIOME),
        NEW_BIOME
    );
    assert_eq!(
        blender.resolve_biome(0, 99, 0, |_, _| 0.0, || NEW_BIOME),
        NEW_BIOME
    );
}

#[test]
fn quart_flat_cache_handles_negative_coordinates_and_dynamic_fallback() {
    let data = column_data(0, 0, 64.0, 0.0);
    let blender = Blender::new(vec![((-1, 0), data)], Vec::new());
    let cache = BlendFlatCache::new(&blender, -4, 0, 1);

    assert_eq!(
        cache.sample(&blender, -16, 0),
        blender.blend_offset_and_factor(-16, 0)
    );
    assert_eq!(
        cache.sample(&blender, 400, 400),
        blender.blend_offset_and_factor(400, 400)
    );
}
