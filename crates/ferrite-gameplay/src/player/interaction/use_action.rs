//! Main/offhand use dispatch, callback algebra, server admission, and stack convergence.

use ferrite_foundation::coordinate::BlockPos;

use crate::block::targeting::{valid_reconstructed_hit, within_block_reach};
use crate::player::interaction::{
    Hand, HitTarget, InteractionResult, ItemContext, StackMutation, StackState, SwingSource,
};
use crate::player::state::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandUseInput {
    pub stack: StackState,
    pub entity_stack_after: StackState,
    pub entity_result: InteractionResult,
    pub living_item_result: InteractionResult,
    pub entity_has_menu: bool,
    pub target_living: bool,
    pub block_result: InteractionResult,
    pub empty_hand_result: InteractionResult,
    pub use_on_result: InteractionResult,
    pub air_result: InteractionResult,
}

impl Default for HandUseInput {
    fn default() -> Self {
        Self {
            stack: StackState::EMPTY,
            entity_stack_after: StackState::EMPTY,
            entity_result: InteractionResult::Pass,
            living_item_result: InteractionResult::Pass,
            entity_has_menu: false,
            target_living: false,
            block_result: InteractionResult::Pass,
            empty_hand_result: InteractionResult::Pass,
            use_on_result: InteractionResult::Pass,
            air_result: InteractionResult::Pass,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientUseContext {
    pub destroying: bool,
    pub hands_busy: bool,
    pub spectator: bool,
    pub infinite_materials: bool,
    pub secondary_use: bool,
    pub target_inside_border: bool,
    pub entity_in_strict_range: bool,
    pub target: HitTarget,
    pub hands: [HandUseInput; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseEffect {
    SetRightClickDelay(u8),
    SendEntity { entity_id: u64, hand: Hand },
    InstallSecondaryAction,
    OpenEntityMenu(u64),
    EntityCallback { entity_id: u64, hand: Hand },
    LivingItemCallback { entity_id: u64, hand: Hand },
    EmitEntityInteractEvent(u64),
    BeginBlockPrediction,
    SendUseOn { position: BlockPos, hand: Hand },
    BlockItemCallback { position: BlockPos, hand: Hand },
    EmptyHandCallback { position: BlockPos },
    UseOnCallback { position: BlockPos, hand: Hand },
    BeginAirPrediction,
    SendUseInAir(Hand),
    AirItemCallback(Hand),
    TriggerEntityCriterion,
    TriggerBlockCriterion,
    TriggerItemCriterion,
    Swing(Hand),
    MutateStack { hand: Hand, mutation: StackMutation },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientUseStop {
    Suppressed,
    FeatureDisabled,
    Border,
    Success,
    BlockFail,
    Exhausted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientUsePlan {
    pub stop: ClientUseStop,
    pub effects: Vec<UseEffect>,
}

#[must_use]
pub fn plan_client_use(context: ClientUseContext) -> ClientUsePlan {
    if context.destroying || context.hands_busy {
        return ClientUsePlan {
            stop: ClientUseStop::Suppressed,
            effects: Vec::new(),
        };
    }
    let mut effects = vec![UseEffect::SetRightClickDelay(4)];
    for (index, hand) in Hand::ORDERED.into_iter().enumerate() {
        let input = context.hands[index];
        if !input.stack.feature_enabled {
            return plan(ClientUseStop::FeatureDisabled, effects);
        }
        match context.target {
            HitTarget::Entity(entity) => {
                if !context.target_inside_border {
                    return plan(ClientUseStop::Border, effects);
                }
                if context.entity_in_strict_range {
                    effects.push(UseEffect::SendEntity {
                        entity_id: entity.entity_id,
                        hand,
                    });
                    effects.push(UseEffect::InstallSecondaryAction);
                    let result = plan_entity_callback(
                        &mut effects,
                        entity.entity_id,
                        hand,
                        input,
                        context.spectator,
                        context.infinite_materials,
                    );
                    if result.consumes() {
                        finish_success(
                            &mut effects,
                            hand,
                            result,
                            Some(UseEffect::TriggerEntityCriterion),
                        );
                        return plan(ClientUseStop::Success, effects);
                    }
                }
            }
            HitTarget::Block(block) => {
                if !context.target_inside_border {
                    return plan(ClientUseStop::Border, effects);
                }
                effects.push(UseEffect::BeginBlockPrediction);
                effects.push(UseEffect::SendUseOn {
                    position: block.position,
                    hand,
                });
                if context.spectator {
                    return plan(ClientUseStop::Success, effects);
                }
                let block_result = plan_block_callback(
                    &mut effects,
                    block.position,
                    hand,
                    input,
                    context.secondary_use,
                    context.hands.iter().any(|hand| !hand.stack.is_empty()),
                    context.infinite_materials,
                );
                if block_result.consumes() {
                    finish_success(
                        &mut effects,
                        hand,
                        block_result,
                        Some(UseEffect::TriggerBlockCriterion),
                    );
                    return plan(ClientUseStop::Success, effects);
                }
                if block_result == InteractionResult::Fail {
                    return plan(ClientUseStop::BlockFail, effects);
                }
            }
            HitTarget::Miss { .. } => {}
        }
        let air_result = plan_air_callback(&mut effects, hand, input, context.spectator);
        if air_result.consumes() {
            finish_success(
                &mut effects,
                hand,
                air_result,
                Some(UseEffect::TriggerItemCriterion),
            );
            return plan(ClientUseStop::Success, effects);
        }
    }
    plan(ClientUseStop::Exhausted, effects)
}

fn plan_entity_callback(
    effects: &mut Vec<UseEffect>,
    entity_id: u64,
    hand: Hand,
    input: HandUseInput,
    spectator: bool,
    infinite_materials: bool,
) -> InteractionResult {
    if spectator {
        if input.entity_has_menu {
            effects.push(UseEffect::OpenEntityMenu(entity_id));
        }
        return InteractionResult::Pass;
    }
    effects.push(UseEffect::EntityCallback { entity_id, hand });
    if input.entity_result.consumes() {
        if infinite_materials
            && input.entity_stack_after.object_id == input.stack.object_id
            && input.entity_stack_after.count < input.stack.count
        {
            effects.push(UseEffect::MutateStack {
                hand,
                mutation: StackMutation::RestoreCount(input.stack.count),
            });
        }
        return input.entity_result;
    }
    if input.stack.is_empty() || !input.target_living {
        return InteractionResult::Pass;
    }
    effects.push(UseEffect::LivingItemCallback { entity_id, hand });
    if input.living_item_result.consumes() {
        effects.push(UseEffect::EmitEntityInteractEvent(entity_id));
        if interaction_stack(input.living_item_result).is_some_and(StackState::is_empty)
            && !infinite_materials
        {
            effects.push(UseEffect::MutateStack {
                hand,
                mutation: StackMutation::Clear,
            });
        }
    }
    input.living_item_result
}

fn plan_block_callback(
    effects: &mut Vec<UseEffect>,
    position: BlockPos,
    hand: Hand,
    input: HandUseInput,
    secondary_use: bool,
    either_hand_nonempty: bool,
    infinite_materials: bool,
) -> InteractionResult {
    let mut result = InteractionResult::Pass;
    if !(secondary_use && either_hand_nonempty) {
        effects.push(UseEffect::BlockItemCallback { position, hand });
        result = input.block_result;
        if result.consumes() {
            return result;
        }
        if result == InteractionResult::TryEmptyHandInteraction && hand == Hand::Main {
            effects.push(UseEffect::EmptyHandCallback { position });
            result = input.empty_hand_result;
            if result.consumes() {
                return result;
            }
        }
    }
    if !input.stack.is_empty() && !input.stack.on_cooldown {
        effects.push(UseEffect::UseOnCallback { position, hand });
        result = input.use_on_result;
        if infinite_materials {
            effects.push(UseEffect::MutateStack {
                hand,
                mutation: StackMutation::RestoreCount(input.stack.count),
            });
        }
    }
    result
}

fn plan_air_callback(
    effects: &mut Vec<UseEffect>,
    hand: Hand,
    input: HandUseInput,
    spectator: bool,
) -> InteractionResult {
    if input.stack.is_empty() {
        return InteractionResult::Pass;
    }
    effects.push(UseEffect::BeginAirPrediction);
    effects.push(UseEffect::SendUseInAir(hand));
    if spectator || input.stack.on_cooldown {
        return InteractionResult::Pass;
    }
    effects.push(UseEffect::AirItemCallback(hand));
    input.air_result
}

fn finish_success(
    effects: &mut Vec<UseEffect>,
    hand: Hand,
    result: InteractionResult,
    criterion: Option<UseEffect>,
) {
    if let ItemContext::ItemUsed { transformed } = result.item_context() {
        if let Some(criterion) = criterion {
            effects.push(criterion);
        }
        if let Some(stack) = transformed {
            effects.push(UseEffect::MutateStack {
                hand,
                mutation: if stack.is_empty() {
                    StackMutation::Clear
                } else {
                    StackMutation::Replace(stack)
                },
            });
        }
    }
    if result.swing_source() == SwingSource::Client {
        effects.push(UseEffect::Swing(hand));
    }
}

const fn interaction_stack(result: InteractionResult) -> Option<StackState> {
    match result.item_context() {
        ItemContext::ItemUsed { transformed } => transformed,
        ItemContext::None => None,
    }
}

fn plan(stop: ClientUseStop, effects: Vec<UseEffect>) -> ClientUsePlan {
    ClientUsePlan { stop, effects }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PredictionSequence {
    highest_received: i32,
}

impl Default for PredictionSequence {
    fn default() -> Self {
        Self {
            highest_received: -1,
        }
    }
}

impl PredictionSequence {
    pub fn register(&mut self, sequence: i32) -> bool {
        if sequence < 0 {
            return false;
        }
        self.highest_received = self.highest_received.max(sequence);
        true
    }

    #[must_use]
    pub const fn acknowledgement(self) -> i32 {
        self.highest_received
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerBlockAdmission {
    pub sequence: i32,
    pub client_loaded: bool,
    pub hand_feature_enabled: bool,
    pub eye: Vec3,
    pub target: BlockPos,
    pub interaction_range: f64,
    pub offset_x: f32,
    pub offset_y: f32,
    pub offset_z: f32,
    pub within_build_height: bool,
    pub spawn_protected: bool,
    pub teleport_pending: bool,
    pub may_interact: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerBlockAdmissionResult {
    RejectBeforeCallback,
    BuildLimit,
    Protected,
    Invoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerUseEffect {
    AcknowledgeSequence(i32),
    RefuseBuildLimit,
    RefuseProtected,
    InvokeBlockTransaction,
    InvokeEntityTransaction { entity_id: u64, hand: Hand },
    InvokeAirTransaction(Hand),
    InstallSecondaryAction,
    ApplyPacketRotation,
    TriggerEntityCriterion,
    TriggerBlockCriterion,
    ServerSwing(Hand),
    UpdateTargetBlock(BlockPos),
    UpdateHitFaceNeighbor,
    MutateStack(StackMutation),
    ResyncInventory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerUsePlan {
    pub admission: ServerBlockAdmissionResult,
    pub effects: Vec<ServerUseEffect>,
}

#[must_use]
pub fn admit_server_block_use(context: ServerBlockAdmission) -> ServerBlockAdmissionResult {
    if !context.client_loaded
        || !context.hand_feature_enabled
        || !within_block_reach(context.eye, context.target, context.interaction_range, 1.0)
        || !valid_reconstructed_hit(
            context.target,
            context.offset_x,
            context.offset_y,
            context.offset_z,
        )
    {
        return ServerBlockAdmissionResult::RejectBeforeCallback;
    }
    if !context.within_build_height {
        return ServerBlockAdmissionResult::BuildLimit;
    }
    if context.spawn_protected || context.teleport_pending || !context.may_interact {
        return ServerBlockAdmissionResult::Protected;
    }
    ServerBlockAdmissionResult::Invoke
}

#[must_use]
pub fn plan_server_block_use(
    context: ServerBlockAdmission,
    hand: Hand,
    result: InteractionResult,
) -> ServerUsePlan {
    let admission = admit_server_block_use(context);
    let mut effects = vec![ServerUseEffect::AcknowledgeSequence(context.sequence)];
    match admission {
        ServerBlockAdmissionResult::RejectBeforeCallback => {}
        ServerBlockAdmissionResult::BuildLimit => {
            effects.push(ServerUseEffect::RefuseBuildLimit);
            push_block_updates(&mut effects, context.target);
        }
        ServerBlockAdmissionResult::Protected => {
            effects.push(ServerUseEffect::RefuseProtected);
            push_block_updates(&mut effects, context.target);
        }
        ServerBlockAdmissionResult::Invoke => {
            effects.push(ServerUseEffect::InvokeBlockTransaction);
            if matches!(result.item_context(), ItemContext::ItemUsed { .. }) {
                effects.push(ServerUseEffect::TriggerBlockCriterion);
            }
            if result.swing_source() == SwingSource::Server {
                effects.push(ServerUseEffect::ServerSwing(hand));
            }
            push_block_updates(&mut effects, context.target);
        }
    }
    ServerUsePlan { admission, effects }
}

fn push_block_updates(effects: &mut Vec<ServerUseEffect>, target: BlockPos) {
    effects.push(ServerUseEffect::UpdateTargetBlock(target));
    effects.push(ServerUseEffect::UpdateHitFaceNeighbor);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ServerEntityAdmission {
    pub client_loaded: bool,
    pub target_current: bool,
    pub inside_world_border: bool,
    pub distance_to_bounds_squared: f64,
    pub interaction_range: f64,
    pub hand_feature_enabled: bool,
}

#[must_use]
pub fn admit_server_entity_use(context: ServerEntityAdmission) -> bool {
    let range = context.interaction_range + 3.0;
    context.client_loaded
        && context.target_current
        && context.inside_world_border
        && context.distance_to_bounds_squared < range * range
        && context.hand_feature_enabled
}

#[must_use]
pub fn plan_server_entity_use(
    context: ServerEntityAdmission,
    entity_id: u64,
    hand: Hand,
    result: InteractionResult,
) -> Vec<ServerUseEffect> {
    if !admit_server_entity_use(context) {
        return Vec::new();
    }
    let mut effects = vec![
        ServerUseEffect::InstallSecondaryAction,
        ServerUseEffect::InvokeEntityTransaction { entity_id, hand },
    ];
    if matches!(result.item_context(), ItemContext::ItemUsed { .. }) {
        effects.push(ServerUseEffect::TriggerEntityCriterion);
    }
    if result.swing_source() == SwingSource::Server {
        effects.push(ServerUseEffect::ServerSwing(hand));
    }
    effects
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerAirUseResult {
    pub mutation: StackMutation,
    pub resync_inventory: bool,
}

#[must_use]
pub fn converge_server_air_use(
    before: StackState,
    current: StackState,
    result: InteractionResult,
    began_using: bool,
    actively_using: bool,
) -> ServerAirUseResult {
    let same = before.object_id == current.object_id
        && before.count == current.count
        && before.damage == current.damage;
    if same && current.use_duration <= 0 {
        return ServerAirUseResult {
            mutation: StackMutation::Retain,
            resync_inventory: false,
        };
    }
    if result == InteractionResult::Fail && current.use_duration > 0 && !began_using {
        return ServerAirUseResult {
            mutation: StackMutation::Retain,
            resync_inventory: false,
        };
    }
    let mutation = match result.item_context() {
        ItemContext::ItemUsed {
            transformed: Some(stack),
        } if stack.is_empty() => StackMutation::Clear,
        ItemContext::ItemUsed {
            transformed: Some(stack),
        } => StackMutation::Replace(stack),
        ItemContext::None | ItemContext::ItemUsed { transformed: None } => StackMutation::Retain,
    };
    ServerAirUseResult {
        mutation,
        resync_inventory: !actively_using,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ServerAirUseContext {
    pub sequence: i32,
    pub client_loaded: bool,
    pub hand: Hand,
    pub hand_feature_enabled: bool,
    pub before: StackState,
    pub current: StackState,
    pub result: InteractionResult,
    pub began_using: bool,
    pub actively_using: bool,
}

#[must_use]
pub fn plan_server_air_use(context: ServerAirUseContext) -> Vec<ServerUseEffect> {
    let mut effects = vec![ServerUseEffect::AcknowledgeSequence(context.sequence)];
    if !context.client_loaded || !context.hand_feature_enabled {
        return effects;
    }
    effects.extend([
        ServerUseEffect::ApplyPacketRotation,
        ServerUseEffect::InvokeAirTransaction(context.hand),
    ]);
    let convergence = converge_server_air_use(
        context.before,
        context.current,
        context.result,
        context.began_using,
        context.actively_using,
    );
    if convergence.mutation != StackMutation::Retain {
        effects.push(ServerUseEffect::MutateStack(convergence.mutation));
    }
    if convergence.resync_inventory {
        effects.push(ServerUseEffect::ResyncInventory);
    }
    effects
}

#[cfg(test)]
mod tests {
    use crate::player::interaction::SwingSource;

    use super::*;

    fn success() -> InteractionResult {
        InteractionResult::Success {
            swing: SwingSource::Client,
            item: ItemContext::None,
        }
    }

    #[test]
    fn entity_pass_falls_through_to_same_hand_air_use() {
        let stack = StackState {
            object_id: 1,
            item_id: 2,
            count: 1,
            damage: 0,
            use_duration: 0,
            feature_enabled: true,
            on_cooldown: false,
        };
        let plan = plan_client_use(ClientUseContext {
            destroying: false,
            hands_busy: false,
            spectator: false,
            infinite_materials: false,
            secondary_use: false,
            target_inside_border: true,
            entity_in_strict_range: true,
            target: HitTarget::Entity(crate::player::interaction::EntityHit {
                entity_id: 4,
                location: Vec3::ZERO,
                relative_location: Vec3::ZERO,
            }),
            hands: [
                HandUseInput {
                    stack,
                    entity_result: InteractionResult::Pass,
                    air_result: success(),
                    ..HandUseInput::default()
                },
                HandUseInput::default(),
            ],
        });
        assert_eq!(plan.stop, ClientUseStop::Success);
        assert!(plan.effects.contains(&UseEffect::SendUseInAir(Hand::Main)));
    }

    #[test]
    fn block_admission_uses_strict_padding_and_component_geometry() {
        let target = BlockPos::default();
        assert_eq!(
            admit_server_block_use(ServerBlockAdmission {
                sequence: 1,
                client_loaded: true,
                hand_feature_enabled: true,
                eye: Vec3::new(6.5, 0.5, 0.5),
                target,
                interaction_range: 4.5,
                offset_x: 0.5,
                offset_y: 0.5,
                offset_z: 0.5,
                within_build_height: true,
                spawn_protected: false,
                teleport_pending: false,
                may_interact: true,
            }),
            ServerBlockAdmissionResult::RejectBeforeCallback
        );
    }
}
