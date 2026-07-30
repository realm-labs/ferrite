//! Top-level explosion phase order and sampled-count result semantics.

use crate::redstone::explosion::block::BlockInteraction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionStage {
    EmitExplodeGameEvent,
    CalculateAffectedPositions,
    HurtEntities,
    PushBlockProfiler,
    ShuffleAndInvokeBlockCallbacks,
    PopCollectedDrops,
    PopBlockProfiler,
    CreateFire,
    ReturnSampledUniquePositionCount,
}

pub fn explosion_order(interaction: BlockInteraction, creates_fire: bool) -> Vec<ExplosionStage> {
    let mut order = vec![
        ExplosionStage::EmitExplodeGameEvent,
        ExplosionStage::CalculateAffectedPositions,
        ExplosionStage::HurtEntities,
    ];
    if interaction.interacts_with_blocks() {
        order.extend([
            ExplosionStage::PushBlockProfiler,
            ExplosionStage::ShuffleAndInvokeBlockCallbacks,
            ExplosionStage::PopCollectedDrops,
            ExplosionStage::PopBlockProfiler,
        ]);
    }
    if creates_fire {
        order.push(ExplosionStage::CreateFire);
    }
    order.push(ExplosionStage::ReturnSampledUniquePositionCount);
    order
}

pub const fn explosion_result(sampled_unique_positions: usize) -> usize {
    sampled_unique_positions
}
