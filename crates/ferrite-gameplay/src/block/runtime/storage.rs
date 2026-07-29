//! Block-owned banner, shelf, and decorated-pot state transitions.

use crate::block::runtime::geometry::DyeColor;
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_foundation::resource::ResourceId;
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stack {
    pub item: Option<ResourceId>,
    pub count: u16,
    pub maximum: u16,
    pub component_fingerprint: u64,
}

impl Stack {
    pub const fn empty() -> Self {
        Self {
            item: None,
            count: 0,
            maximum: 64,
            component_fingerprint: 0,
        }
    }

    pub fn normalized(mut self) -> Self {
        if self.item.is_none() || self.count == 0 {
            return Self::empty();
        }
        self.count = self.count.min(self.maximum);
        self
    }

    pub fn compatible_with(&self, other: &Self) -> bool {
        self.item.is_some()
            && self.item == other.item
            && self.component_fingerprint == other.component_fingerprint
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerLayer {
    pub pattern: ResourceId,
    pub color: DyeColor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BannerWash {
    TryWithEmptyHand,
    ClientSuccess,
    ServerCleaned(Vec<BannerLayer>),
}

pub fn wash_banner(layers: &[BannerLayer], server_side: bool) -> BannerWash {
    if layers.is_empty() {
        return BannerWash::TryWithEmptyHand;
    }
    if !server_side {
        return BannerWash::ClientSuccess;
    }
    let mut cleaned = layers.to_vec();
    cleaned.pop();
    BannerWash::ServerCleaned(cleaned)
}

pub fn banner_tooltip_layers(layers: &[BannerLayer]) -> &[BannerLayer] {
    &layers[..layers.len().min(6)]
}

pub fn banner_render_layers(layers: &[BannerLayer]) -> &[BannerLayer] {
    &layers[..layers.len().min(16)]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerMarker {
    pub position: BlockPos,
    pub color: DyeColor,
    pub custom_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BannerToggle {
    Removed,
    Added,
    OutOfBounds,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannerMap {
    center_x: i32,
    center_z: i32,
    scale: u8,
    tracked_decorations: usize,
    banners: BTreeMap<String, BannerMarker>,
}

impl BannerMap {
    pub fn new(center_x: i32, center_z: i32, scale: u8) -> Self {
        Self {
            center_x,
            center_z,
            scale,
            tracked_decorations: 0,
            banners: BTreeMap::new(),
        }
    }

    pub fn set_tracked_decorations(&mut self, count: usize) {
        self.tracked_decorations = count;
    }

    pub fn toggle(&mut self, marker: BannerMarker) -> BannerToggle {
        let scale = 1_i64 << self.scale.min(31);
        let normalized_x = f64::from(marker.position.x - self.center_x) / scale as f64;
        let normalized_z = f64::from(marker.position.z - self.center_z) / scale as f64;
        if !(-63.0..=63.0).contains(&normalized_x) || !(-63.0..=63.0).contains(&normalized_z) {
            return BannerToggle::OutOfBounds;
        }
        let key = marker_key(marker.position);
        if self.banners.get(&key) == Some(&marker) {
            self.banners.remove(&key);
            self.tracked_decorations = self.tracked_decorations.saturating_sub(1);
            return BannerToggle::Removed;
        }
        if self.tracked_decorations > 256 {
            return BannerToggle::Full;
        }
        if self.banners.insert(key, marker).is_none() {
            self.tracked_decorations += 1;
        }
        BannerToggle::Added
    }

    pub fn validate(&mut self, position: BlockPos, current: Option<&BannerMarker>) -> bool {
        let key = marker_key(position);
        let valid = self
            .banners
            .get(&key)
            .is_none_or(|stored| current == Some(stored));
        if !valid {
            self.banners.remove(&key);
            self.tracked_decorations = self.tracked_decorations.saturating_sub(1);
        }
        valid
    }
}

fn marker_key(position: BlockPos) -> String {
    format!("banner-{},{},{}", position.x, position.y, position.z)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideChainPart {
    Unconnected,
    Right,
    Center,
    Left,
}

pub fn canonical_shelf_parts(length: usize) -> Option<Vec<SideChainPart>> {
    match length {
        1 => Some(vec![SideChainPart::Unconnected]),
        2 => Some(vec![SideChainPart::Left, SideChainPart::Right]),
        3 => Some(vec![
            SideChainPart::Left,
            SideChainPart::Center,
            SideChainPart::Right,
        ]),
        _ => None,
    }
}

pub fn admitted_shelf_chain(left: usize, right: usize) -> (bool, bool) {
    let current = 1;
    let admit_left = left > 0 && left + current <= 3;
    let occupied = current + usize::from(admit_left) * left;
    let admit_right = right > 0 && right + occupied <= 3;
    (admit_left, admit_right)
}

pub fn shelf_hit_slot(
    facing: Direction,
    hit_face: Direction,
    relative_x: f64,
    relative_z: f64,
) -> Option<usize> {
    if hit_face != facing || !relative_x.is_finite() || !relative_z.is_finite() {
        return None;
    }
    let coordinate = match facing {
        Direction::North => 1.0 - relative_x,
        Direction::South => relative_x,
        Direction::West => relative_z,
        Direction::East => 1.0 - relative_z,
        Direction::Down | Direction::Up => return None,
    };
    Some((coordinate * 3.0).floor().clamp(0.0, 2.0) as usize)
}

pub fn shelf_comparator(facing: Direction, query: Direction, slots: &[Stack; 3]) -> u8 {
    if query != facing.opposite() {
        return 0;
    }
    slots.iter().enumerate().fold(0, |signal, (index, stack)| {
        signal | u8::from(stack.item.is_some()) << index
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShelfSwap {
    pub returned: Stack,
    pub stored: Stack,
    pub changed: bool,
}

pub fn swap_unpowered_shelf(input: Stack, stored: Stack, infinite_materials: bool) -> ShelfSwap {
    let mut accepted = input.clone().normalized();
    accepted.count = accepted.count.min(99).min(accepted.maximum);
    let returned = if infinite_materials && stored.item.is_none() {
        input
    } else {
        stored
    };
    let changed = accepted != returned;
    ShelfSwap {
        returned,
        stored: accepted,
        changed,
    }
}

pub fn swap_powered_shelves(
    shelves: &mut [Option<[Stack; 3]>],
    hotbar: &mut [Stack; 9],
) -> PoweredShelfSwap {
    let count = shelves.len().min(3);
    let first_hotbar = 9 - 3 * count;
    let mut pairs_changed = 0;
    let mut shelves_updated = 0;
    for (shelf_index, shelf) in shelves.iter_mut().take(count).enumerate() {
        let Some(slots) = shelf else {
            continue;
        };
        shelves_updated += 1;
        for (slot_index, slot) in slots.iter_mut().enumerate() {
            let hotbar_index = first_hotbar + shelf_index * 3 + slot_index;
            if slot.item.is_none() && hotbar[hotbar_index].item.is_none() {
                continue;
            }
            std::mem::swap(slot, &mut hotbar[hotbar_index]);
            pairs_changed += 1;
        }
    }
    PoweredShelfSwap {
        pairs_changed,
        shelves_updated,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoweredShelfSwap {
    pub pairs_changed: usize,
    pub shelves_updated: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PotDecorations {
    faces: [Option<ResourceId>; 4],
}

impl PotDecorations {
    pub fn decode(values: &[ResourceId]) -> Result<Self, PotDecorationError> {
        if values.len() > 4 {
            return Err(PotDecorationError::TooManyFaces {
                actual: values.len(),
            });
        }
        let mut faces = [const { None }; 4];
        for (index, value) in values.iter().enumerate() {
            if value.path() != "brick" || value.namespace() != "minecraft" {
                faces[index] = Some(value.clone());
            }
        }
        Ok(Self { faces })
    }

    pub fn empty() -> Self {
        Self {
            faces: [const { None }; 4],
        }
    }

    pub fn encoded(&self) -> [ResourceId; 4] {
        std::array::from_fn(|index| {
            self.faces[index]
                .clone()
                .unwrap_or_else(|| ResourceId::minecraft("brick").expect("locked identifier"))
        })
    }

    pub fn tooltip_order(&self) -> [Option<&ResourceId>; 4] {
        [
            self.faces[3].as_ref(),
            self.faces[1].as_ref(),
            self.faces[2].as_ref(),
            self.faces[0].as_ref(),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PotDecorationError {
    #[error("pot decorations contain {actual} faces, maximum is four")]
    TooManyFaces { actual: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WobbleStyle {
    Positive,
    Negative,
}

impl WobbleStyle {
    pub const fn duration(self) -> u64 {
        match self {
            Self::Positive => 7,
            Self::Negative => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecoratedPot {
    pub decorations: PotDecorations,
    pub stored: Stack,
    pub wobble_started: u64,
    pub wobble: Option<WobbleStyle>,
}

impl DecoratedPot {
    pub fn empty() -> Self {
        Self {
            decorations: PotDecorations::empty(),
            stored: Stack::empty(),
            wobble_started: 0,
            wobble: None,
        }
    }

    pub fn insert(
        &mut self,
        hand: &mut Stack,
        infinite_materials: bool,
        game_tick: u64,
    ) -> PotInsert {
        let admissible = hand.item.is_some()
            && (self.stored.item.is_none()
                || (self.stored.compatible_with(hand) && self.stored.count < self.stored.maximum));
        if !admissible {
            self.wobble_started = game_tick;
            self.wobble = Some(WobbleStyle::Negative);
            return PotInsert::Rejected;
        }
        if self.stored.item.is_none() {
            self.stored = hand.clone();
            self.stored.count = 1;
        } else {
            self.stored.count += 1;
        }
        if !infinite_materials {
            hand.count = hand.count.saturating_sub(1);
            *hand = hand.clone().normalized();
        }
        self.wobble_started = game_tick;
        self.wobble = Some(WobbleStyle::Positive);
        PotInsert::Inserted {
            comparator: self.comparator(),
        }
    }

    pub fn comparator(&self) -> u8 {
        if self.stored.item.is_none() {
            return 0;
        }
        ((14 * u32::from(self.stored.count)) / u32::from(self.stored.maximum) + 1) as u8
    }

    pub fn visible_wobble_yaw(&self, game_tick: u64, partial_tick: f32) -> f32 {
        let Some(style) = self.wobble else {
            return 0.0;
        };
        let elapsed = game_tick.saturating_sub(self.wobble_started) as f32 + partial_tick;
        let progress = elapsed / style.duration() as f32;
        if !(0.0..=1.0).contains(&progress) {
            return 0.0;
        }
        (-3.0 * std::f32::consts::PI * progress).sin() * 0.125 * (1.0 - progress)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotInsert {
    Inserted { comparator: u8 },
    Rejected,
}
