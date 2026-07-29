use bevy_ecs::prelude::Component;
use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct BlockBreakSession {
    pub position: BlockPos,
    pub expected_state: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakAction {
    Start,
    Abort,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakDecision {
    Track(BlockBreakSession),
    Clear,
    Remove(BlockPos),
    Correct(BlockPos),
}

#[must_use]
pub fn decide_break(
    action: BreakAction,
    active: Option<BlockBreakSession>,
    position: BlockPos,
    current: BlockStateId,
    air: BlockStateId,
) -> BreakDecision {
    match action {
        BreakAction::Start if current == air => BreakDecision::Correct(position),
        BreakAction::Start => BreakDecision::Track(BlockBreakSession {
            position,
            expected_state: current,
        }),
        BreakAction::Abort => BreakDecision::Clear,
        BreakAction::Stop => match active {
            Some(session) if session.position == position && session.expected_state == current => {
                BreakDecision::Remove(position)
            }
            _ => BreakDecision::Correct(position),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_requires_the_same_position_and_state_seen_at_start() {
        let position = BlockPos::new(1, 2, 3);
        let stone = BlockStateId::new(1);
        let session = BlockBreakSession {
            position,
            expected_state: stone,
        };
        assert_eq!(
            decide_break(
                BreakAction::Stop,
                Some(session),
                position,
                stone,
                BlockStateId::new(0)
            ),
            BreakDecision::Remove(position)
        );
        assert_eq!(
            decide_break(
                BreakAction::Stop,
                Some(session),
                position,
                BlockStateId::new(2),
                BlockStateId::new(0)
            ),
            BreakDecision::Correct(position)
        );
    }
}
