//! Lodestone POI constants and exact Compass binding/validation decisions.

pub const BLOCK_ID: u16 = 923;
pub const STATE_ID: u32 = 21_830;
pub const ITEM_ID: u16 = 1_414;
pub const POI_ID: u16 = 18;
pub const POI_MAX_TICKETS: u8 = 0;
pub const POI_VALID_RANGE: u8 = 1;
pub const TRACKER_COMPONENT_ID: u16 = 67;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompassBinding {
    pub mutate_held_in_place: bool,
    pub consume_source: u8,
    pub create_bound_copy: bool,
    pub drop_if_inventory_full: bool,
}

pub fn bind_compass(count: u8, infinite_materials: bool) -> CompassBinding {
    if !infinite_materials && count == 1 {
        CompassBinding {
            mutate_held_in_place: true,
            consume_source: 0,
            create_bound_copy: false,
            drop_if_inventory_full: false,
        }
    } else {
        CompassBinding {
            mutate_held_in_place: false,
            consume_source: u8::from(!infinite_materials),
            create_bound_copy: true,
            drop_if_inventory_full: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tracker {
    pub has_target: bool,
    pub tracked: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerTick {
    SameRecord,
    ClearTarget,
}

pub fn tracker_tick(
    tracker: Tracker,
    target_in_current_dimension: bool,
    target_in_world_bounds: bool,
    lodestone_poi_exists: bool,
) -> TrackerTick {
    if !tracker.tracked || !tracker.has_target || !target_in_current_dimension {
        TrackerTick::SameRecord
    } else if !target_in_world_bounds || !lodestone_poi_exists {
        TrackerTick::ClearTarget
    } else {
        TrackerTick::SameRecord
    }
}
