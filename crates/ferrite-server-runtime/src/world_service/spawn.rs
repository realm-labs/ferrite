//! Deterministic generated-spawn selection and safe respawn placement.

use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_world::chunk::{ChunkColumn, ChunkLayout, VerticalSectionRange};
use ferrite_world::generation::border::state::WorldBorder;
use ferrite_world::generation::dimension::spawn::{
    SpawnCandidate, SpawnSurfaceChecks, initial_spawn_plan,
};
use ferrite_world::generation::overworld::{OverworldGenerationError, OverworldGeneratorV1};
use ferrite_world::generation::status::ChunkStatus;
use ferrite_world::id::{BiomeId, BlockStateId};
use ferrite_world::projection::ChunkSnapshot;
use thiserror::Error;

const DEFAULT_RESPAWN_RADIUS: i32 = 10;
const PLAYER_HALF_WIDTH: f64 = 0.3;

pub(crate) fn overworld_layout() -> ChunkLayout {
    ChunkLayout::new(
        VerticalSectionRange::new(-4, 24).expect("locked overworld vertical range is valid"),
        BlockStateId::new(0),
        BiomeId::new(0),
    )
}

pub(crate) fn resolve_generated_spawn(
    seed: i64,
    border: &WorldBorder,
) -> Result<BlockPos, SpawnResolutionError> {
    let generator = OverworldGeneratorV1::new(
        seed,
        BlockStateId::new(1),
        BlockStateId::new(2),
        [BiomeId::new(0), BiomeId::new(1), BiomeId::new(2)],
    );
    let suggestion = SpawnCandidate { x: 0, z: 0 };
    let plan = initial_spawn_plan(
        suggestion,
        DEFAULT_RESPAWN_RADIUS,
        border.distance_to_border(0.5, 0.5),
        false,
        |bound| deterministic_offset(seed, bound),
    );
    let mut chunks = BTreeMap::new();
    for candidate in plan.candidates.iter().chain([&plan.fallback]) {
        if !border.contains_point_with_radius(
            f64::from(candidate.x) + 0.5,
            f64::from(candidate.z) + 0.5,
            PLAYER_HALF_WIDTH,
        ) {
            continue;
        }
        let chunk_position = BlockPos::new(candidate.x, 0, candidate.z).chunk();
        if let std::collections::btree_map::Entry::Vacant(entry) = chunks.entry(chunk_position) {
            entry.insert(generate_column(&generator, chunk_position)?);
        }
        let chunk = chunks
            .get(&chunk_position)
            .expect("spawn candidate column was generated");
        if let Some(position) =
            safe_column_position(candidate.x, candidate.z, chunk.layout(), |position| {
                chunk.block_state(position).ok()
            })
        {
            return Ok(position);
        }
    }
    Err(SpawnResolutionError::NoSafeSpawn)
}

pub(crate) fn resolve_respawn(
    center: BlockPos,
    border: &WorldBorder,
    snapshots: &BTreeMap<ChunkPos, ChunkSnapshot>,
) -> Option<BlockPos> {
    let plan = initial_spawn_plan(
        SpawnCandidate {
            x: center.x,
            z: center.z,
        },
        DEFAULT_RESPAWN_RADIUS,
        border.distance_to_border(f64::from(center.x) + 0.5, f64::from(center.z) + 0.5),
        false,
        |_| 0,
    );
    [&plan.fallback]
        .into_iter()
        .chain(plan.candidates.iter())
        .find_map(|candidate| {
            if !border.contains_point_with_radius(
                f64::from(candidate.x) + 0.5,
                f64::from(candidate.z) + 0.5,
                PLAYER_HALF_WIDTH,
            ) {
                return None;
            }
            let snapshot = snapshots.get(&BlockPos::new(candidate.x, 0, candidate.z).chunk())?;
            safe_column_position(candidate.x, candidate.z, snapshot.layout(), |position| {
                snapshot_state(snapshot, position)
            })
        })
}

fn generate_column(
    generator: &OverworldGeneratorV1,
    position: ChunkPos,
) -> Result<ChunkColumn, OverworldGenerationError> {
    let mut chunk = ChunkColumn::new(position, overworld_layout());
    for status in ChunkStatus::ALL.into_iter().skip(1) {
        generator.apply_stage(&mut chunk, status)?;
    }
    Ok(chunk)
}

fn safe_column_position(
    x: i32,
    z: i32,
    layout: ChunkLayout,
    mut state: impl FnMut(BlockPos) -> Option<BlockStateId>,
) -> Option<BlockPos> {
    let minimum_y = layout.sections().minimum().checked_mul(16)?;
    let maximum_y = layout
        .sections()
        .maximum_exclusive()
        .checked_mul(16)?
        .checked_sub(2)?;
    for feet_y in (minimum_y.saturating_add(1)..=maximum_y).rev() {
        let support = state(BlockPos::new(x, feet_y - 1, z))?;
        let feet = state(BlockPos::new(x, feet_y, z))?;
        let head = state(BlockPos::new(x, feet_y + 1, z))?;
        let checks = SpawnSurfaceChecks {
            at_or_above_min_y: feet_y >= minimum_y,
            valid_surface_stack: support != BlockStateId::new(0),
            full_support: !ferrite_world::id::has_empty_collision(support),
            liquid_free: !is_fluid(feet) && !is_fluid(head),
            collision_free: ferrite_world::id::has_empty_collision(feet)
                && ferrite_world::id::has_empty_collision(head),
        };
        if checks.accepted() {
            return Some(BlockPos::new(x, feet_y, z));
        }
    }
    None
}

fn snapshot_state(snapshot: &ChunkSnapshot, position: BlockPos) -> Option<BlockStateId> {
    if snapshot.position() != position.chunk() {
        return None;
    }
    let section_y = position.section().y;
    let sections = snapshot.layout().sections();
    if !sections.contains(section_y) {
        return None;
    }
    let index = usize::try_from(section_y - sections.minimum()).ok()?;
    Some(snapshot.sections().get(index)?.block(position.local()))
}

const fn is_fluid(state: BlockStateId) -> bool {
    matches!(state.get(), 3 | 4)
}

fn deterministic_offset(seed: i64, bound: u32) -> u32 {
    if bound == 0 {
        return 0;
    }
    let mut value = seed as u64 ^ 0x5350_4157_4e5f_5631;
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((value ^ (value >> 31)) % u64::from(bound)) as u32
}

#[derive(Debug, Error)]
pub(crate) enum SpawnResolutionError {
    #[error("generated world has no safe spawn candidate inside the authoritative border")]
    NoSafeSpawn,
    #[error(transparent)]
    Generation(#[from] OverworldGenerationError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_world::light::recompute_chunk_light;

    #[test]
    fn generated_spawn_is_seeded_safe_and_inside_the_border() {
        let mut border = WorldBorder::default();
        border.set_size(32.0);
        let first = resolve_generated_spawn(7, &border).unwrap();
        let repeated = resolve_generated_spawn(7, &border).unwrap();
        let other = resolve_generated_spawn(8, &border).unwrap();
        assert_eq!(first, repeated);
        assert_ne!(first, other);
        assert!(border.contains_point_with_radius(
            f64::from(first.x) + 0.5,
            f64::from(first.z) + 0.5,
            PLAYER_HALF_WIDTH,
        ));
        assert!(first.y > 48);
    }

    #[test]
    fn respawn_search_places_above_an_obstructed_origin() {
        let center = BlockPos::new(0, 70, 0);
        let mut chunk = ChunkColumn::new(center.chunk(), overworld_layout());
        for x in 0..16 {
            for z in 0..16 {
                chunk
                    .set_block(BlockPos::new(x, 69, z), BlockStateId::new(1))
                    .unwrap();
            }
        }
        chunk
            .set_block(BlockPos::new(0, 70, 0), BlockStateId::new(1))
            .unwrap();
        recompute_chunk_light(&mut chunk).unwrap();
        let light = chunk
            .light()
            .unwrap()
            .snapshot(chunk.layout().sections().count())
            .unwrap();
        let snapshot = chunk
            .snapshot(light, |_, state| state != BlockStateId::new(0))
            .unwrap();
        let snapshots = BTreeMap::from([(chunk.position(), snapshot)]);
        let resolved = resolve_respawn(center, &WorldBorder::default(), &snapshots).unwrap();
        assert_ne!(resolved, center);
        assert_eq!(resolved, BlockPos::new(0, 71, 0));
    }
}
