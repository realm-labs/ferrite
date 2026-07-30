//! Piston block-event revalidation, overwrite-safe world plan, and retraction decisions.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::redstone::delay::orientation::{InitialOrientation, initial_orientation};
use crate::redstone::piston::power::{MovingAhead, PistonEvent};
use crate::redstone::piston::resolver::{
    PistonBlock, PistonBlockKind, PushReaction, ResolvedStructure, ResolverWorld, is_pushable,
};

pub const REVALIDATION_WRITE_FLAGS: u16 = 2;
pub const EXTENDED_BASE_WRITE_FLAGS: u16 = 67;
pub const RETRACTING_BASE_WRITE_FLAGS: u16 = 276;
pub const MOVING_DESTINATION_WRITE_FLAGS: u16 = 324;
pub const CLEARED_SOURCE_WRITE_FLAGS: u16 = 82;
pub const DESTROY_AIR_WRITE_FLAGS: u16 = 18;
pub const SHAPE_UPDATE_FLAGS: u16 = 2;
pub const SOUND_VOLUME: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventRevalidation {
    Execute,
    CancelWithoutWrite,
    RestoreExtendedAndCancel { write_flags: u16 },
}

pub const fn revalidate_event(
    server: bool,
    powered: bool,
    event: PistonEvent,
) -> EventRevalidation {
    if !server {
        return EventRevalidation::Execute;
    }
    match (powered, event) {
        (true, PistonEvent::Contract | PistonEvent::Drop) => {
            EventRevalidation::RestoreExtendedAndCancel {
                write_flags: REVALIDATION_WRITE_FLAGS,
            }
        }
        (false, PistonEvent::Extend) => EventRevalidation::CancelWithoutWrite,
        _ => EventRevalidation::Execute,
    }
}

pub const fn extension_pitch(random_float: f32) -> f32 {
    random_float * 0.25 + 0.6
}

pub const fn retraction_pitch(random_float: f32) -> f32 {
    random_float * 0.15 + 0.6
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionStage {
    MoveBlocks,
    WriteExtendedBase,
    PlaySound,
    EmitBlockActivate,
}

pub const EXTENSION_ORDER: [ExtensionStage; 4] = [
    ExtensionStage::MoveBlocks,
    ExtensionStage::WriteExtendedBase,
    ExtensionStage::PlaySound,
    ExtensionStage::EmitBlockActivate,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtensionCompletionPlan {
    pub base_write_flags: u16,
    pub sound_draw_consumed_after_writes: bool,
    pub order: [ExtensionStage; 4],
}

pub const fn extension_completion(movement_succeeded: bool) -> Option<ExtensionCompletionPlan> {
    if movement_succeeded {
        Some(ExtensionCompletionPlan {
            base_write_flags: EXTENDED_BASE_WRITE_FLAGS,
            sound_draw_consumed_after_writes: true,
            order: EXTENSION_ORDER,
        })
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DestroyStep {
    pub position: BlockPos,
    pub snapshot: PistonBlock,
    pub drop_resources: bool,
    pub write_flags: u16,
    pub emit_block_destroy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveStep {
    pub source: BlockPos,
    pub destination: BlockPos,
    pub snapshot: PistonBlock,
    pub write_flags: u16,
    pub installs_moving_entity: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearShapeStage {
    SourceIndirect,
    AirNeighbor,
    AirIndirect,
}

pub const CLEAR_SHAPE_ORDER: [ClearShapeStage; 3] = [
    ClearShapeStage::SourceIndirect,
    ClearShapeStage::AirNeighbor,
    ClearShapeStage::AirIndirect,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestroyNotificationStage {
    RemovalHook,
    SourceIndirectShape,
    OrientedNeighbors,
}

pub const DESTROY_NOTIFICATION_ORDER: [DestroyNotificationStage; 3] = [
    DestroyNotificationStage::RemovalHook,
    DestroyNotificationStage::SourceIndirectShape,
    DestroyNotificationStage::OrientedNeighbors,
];

#[derive(Debug, Clone, PartialEq)]
pub struct MovementExecutionPlan {
    pub preclear_retraction_head: Option<BlockPos>,
    pub preclear_flags: Option<u16>,
    pub destroy_reverse: Vec<DestroyStep>,
    pub move_reverse: Vec<MoveStep>,
    pub extension_head: Option<BlockPos>,
    pub extension_head_write_flags: Option<u16>,
    pub extension_head_entity: bool,
    pub clear_sources_unordered: BTreeSet<BlockPos>,
    pub clear_write_flags: u16,
    pub clear_shape_order: [ClearShapeStage; 3],
    pub destroy_updates_reverse: Vec<BlockPos>,
    pub destroy_notification_order: [DestroyNotificationStage; 3],
    pub push_notifications_reverse: Vec<BlockPos>,
    pub notify_extension_head: Option<BlockPos>,
    pub orientation: InitialOrientation,
}

pub fn movement_execution_plan(
    world: &ResolverWorld,
    piston_position: BlockPos,
    piston_direction: Direction,
    extending: bool,
    arm_is_piston_head: bool,
    resolved: &ResolvedStructure,
    redstone_experiments: bool,
) -> Option<MovementExecutionPlan> {
    let arm_position = piston_position.checked_offset(piston_direction, 1).ok()?;
    let push_direction = if extending {
        piston_direction
    } else {
        piston_direction.opposite()
    };
    let snapshots: Vec<_> = resolved
        .to_push
        .iter()
        .map(|position| world.block(*position))
        .collect();
    let mut delete_after_move: BTreeMap<_, _> = resolved
        .to_push
        .iter()
        .copied()
        .zip(snapshots.iter().copied())
        .collect();

    let destroy_reverse = resolved
        .to_destroy
        .iter()
        .rev()
        .map(|position| DestroyStep {
            position: *position,
            snapshot: world.block(*position),
            drop_resources: true,
            write_flags: DESTROY_AIR_WRITE_FLAGS,
            emit_block_destroy: true,
        })
        .collect();

    let mut move_reverse = Vec::with_capacity(resolved.to_push.len());
    for index in (0..resolved.to_push.len()).rev() {
        let source = resolved.to_push[index];
        let destination = source.checked_offset(push_direction, 1).ok()?;
        delete_after_move.remove(&destination);
        move_reverse.push(MoveStep {
            source,
            destination,
            snapshot: snapshots[index],
            write_flags: MOVING_DESTINATION_WRITE_FLAGS,
            installs_moving_entity: true,
        });
    }
    if extending {
        delete_after_move.remove(&arm_position);
    }
    Some(MovementExecutionPlan {
        preclear_retraction_head: if extending || !arm_is_piston_head {
            None
        } else {
            Some(arm_position)
        },
        preclear_flags: if extending || !arm_is_piston_head {
            None
        } else {
            Some(RETRACTING_BASE_WRITE_FLAGS)
        },
        destroy_reverse,
        move_reverse,
        extension_head: if extending { Some(arm_position) } else { None },
        extension_head_write_flags: if extending {
            Some(MOVING_DESTINATION_WRITE_FLAGS)
        } else {
            None
        },
        extension_head_entity: extending,
        clear_sources_unordered: delete_after_move.into_keys().collect(),
        clear_write_flags: CLEARED_SOURCE_WRITE_FLAGS,
        clear_shape_order: CLEAR_SHAPE_ORDER,
        destroy_updates_reverse: resolved.to_destroy.iter().rev().copied().collect(),
        destroy_notification_order: DESTROY_NOTIFICATION_ORDER,
        push_notifications_reverse: resolved.to_push.iter().rev().copied().collect(),
        notify_extension_head: if extending { Some(arm_position) } else { None },
        orientation: initial_orientation(redstone_experiments, Some(resolved.push_direction), None),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetractionStage {
    FinalizeHeadMovingEntity,
    WriteRetractingBase,
    InstallSourceMovingEntity,
    UpdateBaseNeighbors,
    UpdateBaseShapes,
    StickyOrDefaultHeadHandling,
    PlaySound,
    EmitBlockDeactivate,
}

pub const RETRACTION_ORDER: [RetractionStage; 8] = [
    RetractionStage::FinalizeHeadMovingEntity,
    RetractionStage::WriteRetractingBase,
    RetractionStage::InstallSourceMovingEntity,
    RetractionStage::UpdateBaseNeighbors,
    RetractionStage::UpdateBaseShapes,
    RetractionStage::StickyOrDefaultHeadHandling,
    RetractionStage::PlaySound,
    RetractionStage::EmitBlockDeactivate,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetractionPlan {
    pub finalize_head_entity: bool,
    pub base_write_flags: u16,
    pub install_source_entity: bool,
    pub finalize_compatible_two_ahead: bool,
    pub start_fresh_pull: bool,
    pub remove_head: bool,
    pub sound_draw_consumed_after_writes: bool,
    pub order: [RetractionStage; 8],
}

pub fn retraction_plan(
    world: &ResolverWorld,
    piston_position: BlockPos,
    piston_direction: Direction,
    sticky: bool,
    event: PistonEvent,
    head_has_moving_entity: bool,
    two_ahead_moving: Option<MovingAhead>,
) -> Option<RetractionPlan> {
    let two_ahead_position = piston_position.checked_offset(piston_direction, 2).ok()?;
    let two_ahead = world.block(two_ahead_position);
    let compatible_piece = sticky
        && two_ahead_moving.is_some_and(|moving| {
            moving.is_moving_piston && moving.facing == piston_direction && moving.extending
        });
    let pullable = !two_ahead.is_air()
        && is_pushable(
            two_ahead,
            world,
            two_ahead_position,
            piston_direction.opposite(),
            false,
            piston_direction,
        )
        && (matches!(two_ahead.reaction, PushReaction::Normal)
            || matches!(
                two_ahead.kind,
                PistonBlockKind::Piston { .. } | PistonBlockKind::StickyPiston { .. }
            ));
    let start_fresh_pull =
        sticky && !compatible_piece && matches!(event, PistonEvent::Contract) && pullable;
    Some(RetractionPlan {
        finalize_head_entity: head_has_moving_entity,
        base_write_flags: RETRACTING_BASE_WRITE_FLAGS,
        install_source_entity: true,
        finalize_compatible_two_ahead: compatible_piece,
        start_fresh_pull,
        remove_head: !compatible_piece && !start_fresh_pull,
        sound_draw_consumed_after_writes: true,
        order: RETRACTION_ORDER,
    })
}
