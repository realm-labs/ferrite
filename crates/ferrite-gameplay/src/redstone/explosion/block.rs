//! Block-interaction gates, source-order shuffle, and explosion drop collectors.

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

pub const DROP_COLLECTOR_MERGE_CAP: i32 = 16;
pub const LARGE_EXPLOSION_RADIUS: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockInteraction {
    Keep,
    Destroy,
    DestroyWithDecay,
    TriggerBlock,
}

impl BlockInteraction {
    pub const fn interacts_with_blocks(self) -> bool {
        !matches!(self, Self::Keep)
    }

    pub const fn mode_affects_blocklike_entities(self) -> bool {
        matches!(self, Self::Destroy | Self::DestroyWithDecay)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionSourceKind {
    None,
    BreezeWindCharge,
    WindCharge,
    Other,
}

pub const fn can_trigger_blocks(
    interaction: BlockInteraction,
    source: ExplosionSourceKind,
    mob_griefing: bool,
) -> bool {
    if !matches!(interaction, BlockInteraction::TriggerBlock) {
        return false;
    }
    !matches!(source, ExplosionSourceKind::BreezeWindCharge) || mob_griefing
}

pub const fn should_affect_blocklike_entities(
    interaction: BlockInteraction,
    source: ExplosionSourceKind,
    mob_griefing: bool,
) -> bool {
    let is_not_wind_charge = !matches!(
        source,
        ExplosionSourceKind::BreezeWindCharge | ExplosionSourceKind::WindCharge
    );
    is_not_wind_charge && (mob_griefing || interaction.mode_affects_blocklike_entities())
}

pub const fn is_small_explosion(radius: f32, interaction: BlockInteraction) -> bool {
    radius < LARGE_EXPLOSION_RADIUS || !interaction.interacts_with_blocks()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ShuffleError {
    #[error("explosion block shuffle needs one bounded draw for each list size from n through 2")]
    MissingBoundedDraw,
    #[error("shuffle draw {draw} is outside the exclusive bound {bound}")]
    DrawOutOfRange { draw: u32, bound: u32 },
}

pub fn shuffle_in_place<T>(values: &mut [T], bounded_draws: &[u32]) -> Result<usize, ShuffleError> {
    let required = values.len().saturating_sub(1);
    if bounded_draws.len() < required {
        return Err(ShuffleError::MissingBoundedDraw);
    }
    for (draw_index, bound) in (2..=values.len()).rev().enumerate() {
        let draw = bounded_draws[draw_index];
        let bound = u32::try_from(bound).expect("explosion target list fits in u32");
        if draw >= bound {
            return Err(ShuffleError::DrawOutOfRange { draw, bound });
        }
    }
    for (draw_index, index) in (1..values.len()).rev().enumerate() {
        values.swap(index, bounded_draws[draw_index] as usize);
    }
    Ok(required)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEffectStage {
    ShuffleTargets,
    ReReadCurrentStateAndCallback,
    PopCollectedDrops,
}

pub const BLOCK_EFFECT_ORDER: [BlockEffectStage; 3] = [
    BlockEffectStage::ShuffleTargets,
    BlockEffectStage::ReReadCurrentStateAndCallback,
    BlockEffectStage::PopCollectedDrops,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropStack {
    pub item_and_components: u64,
    pub count: i32,
    pub max_stack_size: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackCollector {
    pub position: BlockPos,
    pub stack: DropStack,
}

pub fn add_or_append_stack(
    collectors: &mut Vec<StackCollector>,
    mut incoming: DropStack,
    position: BlockPos,
) {
    for collector in collectors.iter_mut() {
        try_merge(&mut collector.stack, &mut incoming);
        if incoming.count <= 0 {
            return;
        }
    }
    collectors.push(StackCollector {
        position,
        stack: incoming,
    });
}

fn try_merge(collected: &mut DropStack, incoming: &mut DropStack) {
    if !are_mergeable(*collected, *incoming) {
        return;
    }
    let maximum = collected.max_stack_size.min(DROP_COLLECTOR_MERGE_CAP);
    let transferred = maximum.wrapping_sub(collected.count).min(incoming.count);
    collected.count = collected.count.wrapping_add(transferred);
    incoming.count = incoming.count.wrapping_sub(transferred);
}

fn are_mergeable(collected: DropStack, incoming: DropStack) -> bool {
    collected.item_and_components == incoming.item_and_components
        && incoming.count.wrapping_add(collected.count) <= incoming.max_stack_size
}
