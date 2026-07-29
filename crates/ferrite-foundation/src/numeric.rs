//! Checked integer operations used by spatial and persistent identity types.

use std::num::NonZeroU16;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum NumericError {
    #[error("i32 addition overflow: {left} + {right}")]
    AddI32 { left: i32, right: i32 },
    #[error("i32 multiplication overflow: {left} * {right}")]
    MultiplyI32 { left: i32, right: i32 },
    #[error("u64 multiplication overflow: {left} * {right}")]
    MultiplyU64 { left: u64, right: u64 },
    #[error("range is inverted: minimum {minimum} exceeds maximum {maximum}")]
    InvertedRange { minimum: i32, maximum: i32 },
    #[error("value {value} is outside i32")]
    I32Range { value: i64 },
}

pub const fn add_i32(left: i32, right: i32) -> Result<i32, NumericError> {
    match left.checked_add(right) {
        Some(value) => Ok(value),
        None => Err(NumericError::AddI32 { left, right }),
    }
}

pub const fn multiply_i32(left: i32, right: i32) -> Result<i32, NumericError> {
    match left.checked_mul(right) {
        Some(value) => Ok(value),
        None => Err(NumericError::MultiplyI32 { left, right }),
    }
}

pub const fn multiply_u64(left: u64, right: u64) -> Result<u64, NumericError> {
    match left.checked_mul(right) {
        Some(value) => Ok(value),
        None => Err(NumericError::MultiplyU64 { left, right }),
    }
}

pub const fn i32_from_i64(value: i64) -> Result<i32, NumericError> {
    if value < i32::MIN as i64 || value > i32::MAX as i64 {
        Err(NumericError::I32Range { value })
    } else {
        Ok(value as i32)
    }
}

pub const fn inclusive_span(minimum: i32, maximum: i32) -> Result<u64, NumericError> {
    if minimum > maximum {
        return Err(NumericError::InvertedRange { minimum, maximum });
    }
    Ok((maximum as i64 - minimum as i64 + 1) as u64)
}

pub const fn floor_div_i32(value: i32, divisor: NonZeroU16) -> i32 {
    value.div_euclid(divisor.get() as i32)
}

pub const fn floor_rem_u16(value: i32, divisor: NonZeroU16) -> u16 {
    value.rem_euclid(divisor.get() as i32) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_operations_are_euclidean_for_negative_coordinates() {
        let sixteen = NonZeroU16::new(16).unwrap();
        assert_eq!(floor_div_i32(-1, sixteen), -1);
        assert_eq!(floor_rem_u16(-1, sixteen), 15);
        assert_eq!(floor_div_i32(-16, sixteen), -1);
        assert_eq!(floor_rem_u16(-16, sixteen), 0);
        assert_eq!(floor_div_i32(-17, sixteen), -2);
        assert_eq!(floor_rem_u16(-17, sixteen), 15);
    }

    #[test]
    fn checked_operations_reject_overflow() {
        assert!(add_i32(i32::MAX, 1).is_err());
        assert!(multiply_i32(i32::MIN, -1).is_err());
        assert!(multiply_u64(u64::MAX, 2).is_err());
        assert!(i32_from_i64(i64::from(i32::MAX) + 1).is_err());
    }

    #[test]
    fn inclusive_span_handles_the_full_i32_domain() {
        assert_eq!(inclusive_span(i32::MIN, i32::MAX).unwrap(), 1_u64 << 32);
        assert!(inclusive_span(1, 0).is_err());
    }
}
