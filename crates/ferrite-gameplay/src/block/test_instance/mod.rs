//! Test-instance block data, structure operations, and client projection.

pub mod client;
pub mod data;
pub mod geometry;
pub mod operations;

pub const BLOCK_STATE_ID: u32 = 21_742;
pub const BLOCK_ENTITY_PROTOCOL_ID: u32 = 46;
pub const BLOCK_UPDATE_FLAGS: u16 = 3;
pub const TEMPLATE_WRITE_FLAGS: u16 = 818;
pub const POI_TICKETS: u8 = 0;
pub const POI_VALID_RANGE: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestInstanceBlockProperties {
    pub destroy_time: f32,
    pub explosion_resistance: f32,
    pub has_loot_table: bool,
    pub occluding: bool,
    pub view_blocking: bool,
    pub full_collision_cube: bool,
    pub item_stack_limit: u8,
    pub item_epic: bool,
    pub item_has_special_data_components: bool,
    pub block_and_item_share_cube_all_texture: bool,
    pub dragon_immune: bool,
    pub wither_immune: bool,
}

pub const BLOCK_PROPERTIES: TestInstanceBlockProperties = TestInstanceBlockProperties {
    destroy_time: -1.0,
    explosion_resistance: 3_600_000.0,
    has_loot_table: false,
    occluding: false,
    view_blocking: false,
    full_collision_cube: true,
    item_stack_limit: 64,
    item_epic: true,
    item_has_special_data_components: false,
    block_and_item_share_cube_all_texture: true,
    dragon_immune: true,
    wither_immune: true,
};
