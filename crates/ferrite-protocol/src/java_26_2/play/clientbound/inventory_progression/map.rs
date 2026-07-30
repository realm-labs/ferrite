use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::java_26_2::play::clientbound::inventory_progression::packet::{
    MapDecoration, MapItemData, MapPatch,
};
use crate::java_26_2::value::identifier::Identifier;

const MAP_SIDE: usize = 128;
const MAP_PIXELS: usize = MAP_SIDE * MAP_SIDE;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMapData {
    pub scale: i8,
    pub locked: bool,
    pub dimension: Identifier,
    pub colors: Vec<u8>,
    pub decorations: Vec<MapDecoration>,
    pub tracked_decoration_count: usize,
    pub texture_refreshes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapClientProjection {
    dimension: Identifier,
    capacity: usize,
    tracking_types: BTreeSet<Identifier>,
    maps: BTreeMap<i32, ClientMapData>,
}

impl MapClientProjection {
    #[must_use]
    pub fn new(
        dimension: Identifier,
        capacity: usize,
        tracking_types: BTreeSet<Identifier>,
    ) -> Self {
        Self {
            dimension,
            capacity,
            tracking_types,
            maps: BTreeMap::new(),
        }
    }

    pub fn apply(&mut self, packet: &MapItemData) -> Result<(), MapProjectionError> {
        if !self.maps.contains_key(&packet.map_id) {
            if self.maps.len() == self.capacity {
                return Err(MapProjectionError::Capacity {
                    capacity: self.capacity,
                });
            }
            self.maps.insert(
                packet.map_id,
                ClientMapData {
                    scale: packet.scale,
                    locked: packet.locked,
                    dimension: self.dimension.clone(),
                    colors: vec![0; MAP_PIXELS],
                    decorations: Vec::new(),
                    tracked_decoration_count: 0,
                    texture_refreshes: 0,
                },
            );
        }
        let map = self
            .maps
            .get_mut(&packet.map_id)
            .expect("map was present or inserted");
        if let Some(decorations) = &packet.decorations {
            map.decorations.clone_from(decorations);
            map.tracked_decoration_count = decorations
                .iter()
                .filter(|decoration| self.tracking_types.contains(&decoration.decoration_type))
                .count();
        }
        if let Some(patch) = &packet.patch {
            apply_patch(&mut map.colors, patch)?;
        }
        map.texture_refreshes = map.texture_refreshes.saturating_add(1);
        Ok(())
    }

    #[must_use]
    pub fn get(&self, map_id: i32) -> Option<&ClientMapData> {
        self.maps.get(&map_id)
    }
}

fn apply_patch(colors: &mut [u8], patch: &MapPatch) -> Result<(), MapProjectionError> {
    for x in 0..usize::from(patch.width) {
        for y in 0..usize::from(patch.height) {
            let source = x + y * usize::from(patch.width);
            let Some(color) = patch.colors.get(source).copied() else {
                return Err(MapProjectionError::PatchSource {
                    index: source,
                    colors: patch.colors.len(),
                });
            };
            let destination =
                usize::from(patch.start_x) + x + (usize::from(patch.start_y) + y) * MAP_SIDE;
            let Some(pixel) = colors.get_mut(destination) else {
                return Err(MapProjectionError::PatchDestination { index: destination });
            };
            *pixel = color;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DirtyBounds {
    minimum_x: u8,
    minimum_y: u8,
    maximum_x: u8,
    maximum_y: u8,
}

impl DirtyBounds {
    const fn one(x: u8, y: u8) -> Self {
        Self {
            minimum_x: x,
            minimum_y: y,
            maximum_x: x,
            maximum_y: y,
        }
    }

    fn include(&mut self, x: u8, y: u8) {
        self.minimum_x = self.minimum_x.min(x);
        self.minimum_y = self.minimum_y.min(y);
        self.maximum_x = self.maximum_x.max(x);
        self.maximum_y = self.maximum_y.max(y);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapHoldingPublisher {
    scale: i8,
    locked: bool,
    colors: Vec<u8>,
    decorations: Vec<MapDecoration>,
    maximum_decorations: usize,
    dirty_pixels: Option<DirtyBounds>,
    decorations_dirty: bool,
    decoration_tick: i32,
}

impl MapHoldingPublisher {
    #[must_use]
    pub fn new(scale: i8, locked: bool, maximum_decorations: usize) -> Self {
        Self {
            scale,
            locked,
            colors: vec![0; MAP_PIXELS],
            decorations: Vec::new(),
            maximum_decorations,
            dirty_pixels: None,
            decorations_dirty: false,
            decoration_tick: 0,
        }
    }

    pub fn set_color(&mut self, x: u8, y: u8, color: u8) -> Result<(), MapPublicationError> {
        if usize::from(x) >= MAP_SIDE || usize::from(y) >= MAP_SIDE {
            return Err(MapPublicationError::PixelOutsideMap { x, y });
        }
        self.colors[usize::from(x) + usize::from(y) * MAP_SIDE] = color;
        if let Some(bounds) = &mut self.dirty_pixels {
            bounds.include(x, y);
        } else {
            self.dirty_pixels = Some(DirtyBounds::one(x, y));
        }
        Ok(())
    }

    pub fn replace_decorations(
        &mut self,
        decorations: Vec<MapDecoration>,
    ) -> Result<(), MapPublicationError> {
        if decorations.len() > self.maximum_decorations {
            return Err(MapPublicationError::DecorationCapacity {
                requested: decorations.len(),
                maximum: self.maximum_decorations,
            });
        }
        self.decorations = decorations;
        self.decorations_dirty = true;
        Ok(())
    }

    pub fn next_packet(&mut self, map_id: i32) -> Option<MapItemData> {
        let patch = self.dirty_pixels.take().map(|bounds| {
            let width = bounds.maximum_x - bounds.minimum_x + 1;
            let height = bounds.maximum_y - bounds.minimum_y + 1;
            let mut colors = Vec::with_capacity(usize::from(width) * usize::from(height));
            for y in bounds.minimum_y..=bounds.maximum_y {
                for x in bounds.minimum_x..=bounds.maximum_x {
                    colors.push(self.colors[usize::from(x) + usize::from(y) * MAP_SIDE]);
                }
            }
            MapPatch {
                width,
                height,
                start_x: bounds.minimum_x,
                start_y: bounds.minimum_y,
                colors,
            }
        });
        let decorations = if self.decorations_dirty {
            let old_tick = self.decoration_tick;
            self.decoration_tick = self.decoration_tick.wrapping_add(1);
            if old_tick % 5 == 0 {
                self.decorations_dirty = false;
                Some(self.decorations.clone())
            } else {
                None
            }
        } else {
            None
        };
        if patch.is_none() && decorations.is_none() {
            None
        } else {
            Some(MapItemData {
                map_id,
                scale: self.scale,
                locked: self.locked,
                decorations,
                patch,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MapProjectionError {
    #[error("map projection reached its {capacity}-map bound")]
    Capacity { capacity: usize },
    #[error("map patch color index {index} is outside its {colors} bytes")]
    PatchSource { index: usize, colors: usize },
    #[error("map patch destination index {index} is outside the 128x128 map")]
    PatchDestination { index: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MapPublicationError {
    #[error("map pixel ({x}, {y}) is outside 0..=127")]
    PixelOutsideMap { x: u8, y: u8 },
    #[error("map has {requested} decorations, above publication bound {maximum}")]
    DecorationCapacity { requested: usize, maximum: usize },
}
