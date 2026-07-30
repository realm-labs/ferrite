use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDestroyContext {
    pub position: BlockPos,
    pub action_restricted: bool,
    pub item_allows_destroy: bool,
    pub game_master_allows_destroy: bool,
    pub is_air: bool,
    pub fluid_legacy_state: u32,
    pub write_succeeds: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalDestroyEffect {
    AdventureCheck,
    ItemDestroyCheck,
    GameMasterCheck,
    PlayerWillDestroy(BlockPos),
    ReadFluidAfterCallback(BlockPos),
    WriteFluid {
        position: BlockPos,
        state: u32,
        flags: u32,
    },
    BlockDestroyHook(BlockPos),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDestroyPlan {
    pub destroyed: bool,
    pub effects: Vec<LocalDestroyEffect>,
}

#[must_use]
pub fn plan_local_destroy(context: LocalDestroyContext) -> LocalDestroyPlan {
    let mut effects = vec![LocalDestroyEffect::AdventureCheck];
    if context.action_restricted {
        return rejected(effects);
    }
    effects.push(LocalDestroyEffect::ItemDestroyCheck);
    if !context.item_allows_destroy {
        return rejected(effects);
    }
    effects.push(LocalDestroyEffect::GameMasterCheck);
    if !context.game_master_allows_destroy || context.is_air {
        return rejected(effects);
    }
    effects.extend([
        LocalDestroyEffect::PlayerWillDestroy(context.position),
        LocalDestroyEffect::ReadFluidAfterCallback(context.position),
        LocalDestroyEffect::WriteFluid {
            position: context.position,
            state: context.fluid_legacy_state,
            flags: 11,
        },
    ]);
    if context.write_succeeds {
        effects.push(LocalDestroyEffect::BlockDestroyHook(context.position));
    }
    LocalDestroyPlan {
        destroyed: context.write_succeeds,
        effects,
    }
}

fn rejected(effects: Vec<LocalDestroyEffect>) -> LocalDestroyPlan {
    LocalDestroyPlan {
        destroyed: false,
        effects,
    }
}
