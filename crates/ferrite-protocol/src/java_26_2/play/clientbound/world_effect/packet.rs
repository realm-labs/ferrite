use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelEvent {
    pub event_type: i32,
    pub position: BlockPos,
    pub data: i32,
    pub global: bool,
}
