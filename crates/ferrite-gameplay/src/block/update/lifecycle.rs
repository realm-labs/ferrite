use super::flags::BlockUpdateFlags;

pub const DEFAULT_SHAPE_BUDGET: i32 = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityTransition {
    None,
    RetainAndRebind,
    Remove,
    RemoveAndCreate,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetBlockInputs {
    pub valid_bounds: bool,
    pub debug_level: bool,
    pub canonical_state_changed: bool,
    pub block_type_changed: bool,
    pub requested_is_rail: bool,
    pub callback_retained_requested_type: bool,
    pub reread_is_requested_state: bool,
    pub server_side: bool,
    pub chunk_block_ticking: bool,
    pub requested_has_analog_output: bool,
    pub block_entity_transition: BlockEntityTransition,
    pub flags: BlockUpdateFlags,
    pub shape_budget: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetBlockEffect {
    InstallState,
    UpdateSectionCounters,
    UpdateHeightmaps,
    UpdateLighting,
    BlockEntityPreRemoval,
    RemoveBlockEntity,
    OldRemovalHook { moved: bool },
    OnPlace { moved: bool },
    RetainAndRebindBlockEntity,
    CreateBlockEntity,
    MarkChunkUnsaved,
    MarkBlocksDirty,
    PublishServerChange,
    PublishClientChange { immediate: bool },
    InvalidatePathCache,
    RecomputeNavigationPaths,
    NotifyOrdinaryNeighbors,
    NotifyComparatorNeighbors,
    OldIndirectShapes { remaining_budget: i32 },
    RequestedDirectShapes { remaining_budget: i32 },
    RequestedIndirectShapes { remaining_budget: i32 },
    RemoveOldPoi,
    AddNewPoi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetBlockPlan {
    pub accepted: bool,
    pub effects: Vec<SetBlockEffect>,
}

pub fn plan_set_block(inputs: SetBlockInputs) -> SetBlockPlan {
    if !inputs.valid_bounds || inputs.debug_level || !inputs.canonical_state_changed {
        return SetBlockPlan {
            accepted: false,
            effects: Vec::new(),
        };
    }

    let mut effects = vec![
        SetBlockEffect::InstallState,
        SetBlockEffect::UpdateSectionCounters,
        SetBlockEffect::UpdateHeightmaps,
        SetBlockEffect::UpdateLighting,
    ];
    append_block_entity_removal(
        &mut effects,
        inputs.block_entity_transition,
        !inputs
            .flags
            .contains(BlockUpdateFlags::UPDATE_SKIP_BLOCK_ENTITY_SIDE_EFFECTS),
    );

    let moved = inputs
        .flags
        .contains(BlockUpdateFlags::UPDATE_MOVE_BY_PISTON);
    if inputs.server_side
        && (inputs.block_type_changed || inputs.requested_is_rail)
        && (inputs.flags.contains(BlockUpdateFlags::UPDATE_NEIGHBORS) || moved)
    {
        effects.push(SetBlockEffect::OldRemovalHook { moved });
        if !inputs.callback_retained_requested_type {
            return SetBlockPlan {
                accepted: false,
                effects,
            };
        }
    }
    if inputs.server_side
        && !inputs
            .flags
            .contains(BlockUpdateFlags::UPDATE_SKIP_ON_PLACE)
    {
        effects.push(SetBlockEffect::OnPlace { moved });
    }
    append_block_entity_install(&mut effects, inputs.block_entity_transition);
    effects.push(SetBlockEffect::MarkChunkUnsaved);

    if !inputs.reread_is_requested_state {
        return SetBlockPlan {
            accepted: true,
            effects,
        };
    }
    effects.push(SetBlockEffect::MarkBlocksDirty);
    append_publication(&mut effects, inputs);
    append_neighbor_work(&mut effects, inputs);
    SetBlockPlan {
        accepted: true,
        effects,
    }
}

fn append_block_entity_removal(
    effects: &mut Vec<SetBlockEffect>,
    transition: BlockEntityTransition,
    run_pre_removal: bool,
) {
    let removal = match transition {
        BlockEntityTransition::Remove | BlockEntityTransition::RemoveAndCreate => true,
        BlockEntityTransition::None
        | BlockEntityTransition::RetainAndRebind
        | BlockEntityTransition::Create => false,
    };
    if removal {
        if run_pre_removal {
            effects.push(SetBlockEffect::BlockEntityPreRemoval);
        }
        effects.push(SetBlockEffect::RemoveBlockEntity);
    }
}

fn append_block_entity_install(
    effects: &mut Vec<SetBlockEffect>,
    transition: BlockEntityTransition,
) {
    match transition {
        BlockEntityTransition::RetainAndRebind => {
            effects.push(SetBlockEffect::RetainAndRebindBlockEntity);
        }
        BlockEntityTransition::Create | BlockEntityTransition::RemoveAndCreate => {
            effects.push(SetBlockEffect::CreateBlockEntity);
        }
        BlockEntityTransition::None | BlockEntityTransition::Remove => {}
    }
}

fn append_publication(effects: &mut Vec<SetBlockEffect>, inputs: SetBlockInputs) {
    if !inputs.flags.contains(BlockUpdateFlags::UPDATE_CLIENTS) {
        return;
    }
    if inputs.server_side {
        if inputs.chunk_block_ticking {
            effects.extend([
                SetBlockEffect::PublishServerChange,
                SetBlockEffect::InvalidatePathCache,
                SetBlockEffect::RecomputeNavigationPaths,
            ]);
        }
    } else if !inputs.flags.contains(BlockUpdateFlags::UPDATE_INVISIBLE) {
        effects.push(SetBlockEffect::PublishClientChange {
            immediate: inputs.flags.contains(BlockUpdateFlags::UPDATE_IMMEDIATE),
        });
    }
}

fn append_neighbor_work(effects: &mut Vec<SetBlockEffect>, inputs: SetBlockInputs) {
    if inputs.flags.contains(BlockUpdateFlags::UPDATE_NEIGHBORS) {
        effects.push(SetBlockEffect::NotifyOrdinaryNeighbors);
        if inputs.server_side && inputs.requested_has_analog_output {
            effects.push(SetBlockEffect::NotifyComparatorNeighbors);
        }
        if !inputs.flags.contains(BlockUpdateFlags::UPDATE_KNOWN_SHAPE) && inputs.shape_budget > 0 {
            let remaining_budget = inputs.shape_budget - 1;
            effects.extend([
                SetBlockEffect::OldIndirectShapes { remaining_budget },
                SetBlockEffect::RequestedDirectShapes { remaining_budget },
                SetBlockEffect::RequestedIndirectShapes { remaining_budget },
            ]);
        }
    }
    effects.extend([SetBlockEffect::RemoveOldPoi, SetBlockEffect::AddNewPoi]);
}
