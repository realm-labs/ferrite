use ferrite_foundation::coordinate::BlockPos;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignUpdate {
    pub position: BlockPos,
    pub front_text: bool,
    pub lines: [String; 4],
}
