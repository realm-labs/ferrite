//! Vanilla's independent wrapping signed-32-bit block-position stream.

use ferrite_foundation::coordinate::BlockPos;

const POSITION_MULTIPLIER: i32 = 3;
const POSITION_ADDEND: i32 = 1_013_904_223;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomPositionStream {
    value: i32,
}

impl RandomPositionStream {
    pub const fn new(value: i32) -> Self {
        Self { value }
    }

    pub const fn value(self) -> i32 {
        self.value
    }

    pub fn next(&mut self, base: BlockPos, y_mask: i32) -> BlockPos {
        self.value = self
            .value
            .wrapping_mul(POSITION_MULTIPLIER)
            .wrapping_add(POSITION_ADDEND);
        let shifted = self.value >> 2;
        BlockPos::new(
            base.x.wrapping_add(shifted & 15),
            base.y.wrapping_add((shifted >> 16) & y_mask),
            base.z.wrapping_add((shifted >> 8) & 15),
        )
    }
}
