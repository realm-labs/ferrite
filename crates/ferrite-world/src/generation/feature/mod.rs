//! Configured and placed feature dispatch.

pub mod basalt_columns;
pub mod basic;
pub mod chorus;
pub mod column;
pub mod coral;
pub mod direct_write;
pub mod end_spike;
pub mod fallen_tree;
pub mod fossil;
pub mod geode;
pub mod huge_fungus;
pub mod iceberg;
mod java_hash_set;
pub mod lake;
pub mod large_dripstone;
pub mod modifier;
pub mod monster_room;
pub mod multiface;
pub mod mushroom;
pub mod nether_vines;
pub mod ore;
pub mod placement;
pub mod platform;
pub mod predicate;
pub mod provider;
pub mod random;
pub mod root_system;
pub mod sculk;
pub mod selector;
pub mod simple_block;
pub mod speleothem;
pub mod speleothem_cluster;
pub mod structure;
pub mod template;
pub mod terrain;
pub mod tree_core;
pub mod tree_decorator_attachments;
pub mod tree_decorator_ground;
pub mod tree_decorator_logs;
pub mod tree_decorator_vines;
pub mod tree_foliage;
pub mod tree_roots;
pub mod tree_trunk;
pub mod tree_trunk_complex;
pub mod vegetation;
pub mod vegetation_patch;

use ferrite_foundation::coordinate::BlockPos;

#[must_use]
pub fn place_configured(
    origin: BlockPos,
    mut ensure_can_write: impl FnMut(BlockPos) -> bool,
    algorithm: impl FnOnce() -> bool,
) -> bool {
    if !ensure_can_write(origin) {
        return false;
    }
    algorithm()
}

#[must_use]
pub const fn place_no_op() -> bool {
    true
}
