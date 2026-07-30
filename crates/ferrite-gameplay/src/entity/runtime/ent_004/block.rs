//! Projectile block-breaking admission and the three source-owned block callbacks.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileBlock {
    ChorusFlower,
    DecoratedPot,
    PointedDripstone,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlockImpactInput {
    pub server_side: bool,
    pub may_interact: bool,
    pub impact_projectile_tag: bool,
    pub projectiles_can_break_blocks: bool,
    pub block: ProjectileBlock,
    pub thrown_trident: bool,
    pub speed: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockMutation {
    DestroyWithDrops { projectile_is_breaker: bool },
    CrackDecoratedPotThenDestroy { write_flags: u16 },
}

#[must_use]
pub const fn may_break(impact_projectile_tag: bool, projectiles_can_break_blocks: bool) -> bool {
    impact_projectile_tag && projectiles_can_break_blocks
}

#[must_use]
pub fn block_impact(input: BlockImpactInput) -> Option<BlockMutation> {
    if !input.server_side
        || !input.may_interact
        || !may_break(
            input.impact_projectile_tag,
            input.projectiles_can_break_blocks,
        )
    {
        return None;
    }
    match input.block {
        ProjectileBlock::ChorusFlower => Some(BlockMutation::DestroyWithDrops {
            projectile_is_breaker: true,
        }),
        ProjectileBlock::DecoratedPot => {
            Some(BlockMutation::CrackDecoratedPotThenDestroy { write_flags: 260 })
        }
        ProjectileBlock::PointedDripstone if input.thrown_trident && input.speed > 0.6 => {
            Some(BlockMutation::DestroyWithDrops {
                projectile_is_breaker: true,
            })
        }
        ProjectileBlock::PointedDripstone | ProjectileBlock::Other => None,
    }
}
