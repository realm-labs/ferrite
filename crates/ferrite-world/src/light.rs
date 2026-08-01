//! Deterministic light reconstruction for authoritative chunk columns.

use std::collections::VecDeque;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::chunk::{ChunkAccessError, ChunkColumn};
use crate::id::{light_emission, light_opacity};
use crate::projection::{ChunkLightState, ChunkProjectionError, LIGHT_LAYER_BYTES, LightLayer};

pub fn recompute_chunk_light(chunk: &mut ChunkColumn) -> Result<(), ChunkLightError> {
    let sections = chunk.layout().sections();
    let layer_count = usize::from(sections.count()) + 2;
    let block_count = usize::from(sections.count()) * 16 * 16 * 16;
    let mut sky = vec![0_u8; block_count];
    let mut block = vec![0_u8; block_count];
    let minimum_y = sections.minimum() * 16;
    let maximum_y = sections.maximum_exclusive() * 16;
    let origin_x = chunk.position().checked_min_block_x()?;
    let origin_z = chunk.position().checked_min_block_z()?;
    let mut queue = VecDeque::new();

    for local_z in 0..16 {
        for local_x in 0..16 {
            let mut exposed = true;
            for y in (minimum_y..maximum_y).rev() {
                let position = BlockPos::new(origin_x + local_x, y, origin_z + local_z);
                let state = chunk.block_state(position)?;
                if light_opacity(state) == 15 {
                    exposed = false;
                }
                let index = voxel_index(local_x as usize, y, local_z as usize, minimum_y);
                if exposed {
                    sky[index] = 15;
                }
                let emission = light_emission(state);
                if emission != 0 {
                    block[index] = emission;
                    queue.push_back((local_x, y, local_z));
                }
            }
        }
    }

    while let Some((x, y, z)) = queue.pop_front() {
        let source = block[voxel_index(x as usize, y, z as usize, minimum_y)];
        if source <= 1 {
            continue;
        }
        for (next_x, next_y, next_z) in [
            (x - 1, y, z),
            (x + 1, y, z),
            (x, y - 1, z),
            (x, y + 1, z),
            (x, y, z - 1),
            (x, y, z + 1),
        ] {
            if !(0..16).contains(&next_x)
                || !(minimum_y..maximum_y).contains(&next_y)
                || !(0..16).contains(&next_z)
            {
                continue;
            }
            let position = BlockPos::new(origin_x + next_x, next_y, origin_z + next_z);
            if light_opacity(chunk.block_state(position)?) == 15 {
                continue;
            }
            let index = voxel_index(next_x as usize, next_y, next_z as usize, minimum_y);
            let propagated = source - 1;
            if propagated > block[index] {
                block[index] = propagated;
                queue.push_back((next_x, next_y, next_z));
            }
        }
    }

    let mut sky_layers = vec![LightLayer::Empty; layer_count];
    sky_layers[0] = LightLayer::full_brightness();
    sky_layers[layer_count - 1] = LightLayer::full_brightness();
    let mut block_layers = vec![LightLayer::Empty; layer_count];
    for section in 0..usize::from(sections.count()) {
        sky_layers[section + 1] = encode_layer(&sky, section);
        block_layers[section + 1] = encode_layer(&block, section);
    }
    chunk.replace_light(ChunkLightState::new(
        sky_layers,
        block_layers,
        sections.count(),
    )?)?;
    Ok(())
}

fn voxel_index(x: usize, y: i32, z: usize, minimum_y: i32) -> usize {
    (((y - minimum_y) as usize * 16 + z) * 16) + x
}

fn encode_layer(values: &[u8], section: usize) -> LightLayer {
    let start = section * 16 * 16 * 16;
    let values = &values[start..start + 16 * 16 * 16];
    if values.iter().all(|value| *value == 0) {
        return LightLayer::Empty;
    }
    let mut bytes = Box::new([0_u8; LIGHT_LAYER_BYTES]);
    for (index, value) in values.iter().copied().enumerate() {
        bytes[index / 2] |= if index & 1 == 0 {
            value & 0x0f
        } else {
            (value & 0x0f) << 4
        };
    }
    LightLayer::Data(bytes)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChunkLightError {
    #[error(transparent)]
    Numeric(#[from] ferrite_foundation::numeric::NumericError),
    #[error(transparent)]
    Chunk(#[from] ChunkAccessError),
    #[error(transparent)]
    Projection(#[from] ChunkProjectionError),
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::coordinate::ChunkPos;

    use crate::chunk::{ChunkLayout, VerticalSectionRange};
    use crate::id::{BiomeId, FIRE, STONE};

    use super::*;

    #[test]
    fn opaque_roof_blocks_sky_and_fire_propagates_block_light() {
        let layout = ChunkLayout::new(
            VerticalSectionRange::new(0, 2).unwrap(),
            crate::id::AIR,
            BiomeId::new(0),
        );
        let mut chunk = ChunkColumn::new(ChunkPos::new(0, 0), layout);
        chunk.set_block(BlockPos::new(1, 8, 1), STONE).unwrap();
        chunk.set_block(BlockPos::new(4, 8, 4), FIRE).unwrap();
        recompute_chunk_light(&mut chunk).unwrap();
        let light = chunk.light().unwrap();
        assert!(matches!(light.sky()[1], LightLayer::Data(_)));
        assert!(matches!(light.block()[1], LightLayer::Data(_)));
    }
}
