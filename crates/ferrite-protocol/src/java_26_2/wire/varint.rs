use crate::java_26_2::wire::error::WireError;

pub const MAX_VAR_INT_BYTES: usize = 5;
pub const MAX_VAR_LONG_BYTES: usize = 10;

/// A minimally encoded signed variable-width integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EncodedVariable<const N: usize> {
    bytes: [u8; N],
    length: u8,
}

impl<const N: usize> EncodedVariable<N> {
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..usize::from(self.length)]
    }
}

#[must_use]
pub fn encode_i32(value: i32) -> EncodedVariable<MAX_VAR_INT_BYTES> {
    encode_unsigned(u64::from(value as u32))
}

#[must_use]
pub fn encode_i64(value: i64) -> EncodedVariable<MAX_VAR_LONG_BYTES> {
    encode_unsigned(value as u64)
}

pub fn decode_i32(input: &[u8]) -> Result<(i32, usize), WireError> {
    match decode_i32_partial(input)? {
        Some(decoded) => Ok(decoded),
        None => Err(WireError::UnexpectedEnd {
            field: "VarInt",
            needed: input.len().saturating_add(1),
            remaining: input.len(),
        }),
    }
}

pub fn decode_i64(input: &[u8]) -> Result<(i64, usize), WireError> {
    match decode_i64_partial(input)? {
        Some(decoded) => Ok(decoded),
        None => Err(WireError::UnexpectedEnd {
            field: "VarLong",
            needed: input.len().saturating_add(1),
            remaining: input.len(),
        }),
    }
}

pub(crate) fn decode_i32_partial(input: &[u8]) -> Result<Option<(i32, usize)>, WireError> {
    decode_unsigned_partial::<u32, MAX_VAR_INT_BYTES>(input, "VarInt")
        .map(|value| value.map(|(decoded, length)| (decoded as i32, length)))
}

fn decode_i64_partial(input: &[u8]) -> Result<Option<(i64, usize)>, WireError> {
    decode_unsigned_partial::<u64, MAX_VAR_LONG_BYTES>(input, "VarLong")
        .map(|value| value.map(|(decoded, length)| (decoded as i64, length)))
}

fn encode_unsigned<const N: usize>(mut remaining: u64) -> EncodedVariable<N> {
    let mut bytes = [0; N];
    let mut length = 0;
    loop {
        let payload = (remaining & 0x7f) as u8;
        remaining >>= 7;
        bytes[length] = if remaining == 0 {
            payload
        } else {
            payload | 0x80
        };
        length += 1;
        if remaining == 0 {
            break;
        }
    }
    EncodedVariable {
        bytes,
        length: length as u8,
    }
}

trait VariableUnsigned:
    Copy + From<u8> + std::ops::BitOrAssign + std::ops::Shl<usize, Output = Self> + std::fmt::Debug
{
}

impl VariableUnsigned for u32 {}
impl VariableUnsigned for u64 {}

fn decode_unsigned_partial<T, const N: usize>(
    input: &[u8],
    kind: &'static str,
) -> Result<Option<(T, usize)>, WireError>
where
    T: VariableUnsigned,
{
    let mut value = T::from(0);
    for index in 0..N {
        let Some(byte) = input.get(index).copied() else {
            return Ok(None);
        };
        value |= T::from(byte & 0x7f) << (index * 7);
        if byte & 0x80 == 0 {
            return Ok(Some((value, index + 1)));
        }
    }
    Err(WireError::VariableIntegerTooLong {
        kind,
        maximum_bytes: N,
    })
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::wire::error::WireError;
    use crate::java_26_2::wire::varint::{decode_i32, decode_i64, encode_i32, encode_i64};

    #[test]
    fn matches_locked_varint_vectors() {
        let vectors: &[(i32, &[u8])] = &[
            (0, &[0x00]),
            (1, &[0x01]),
            (127, &[0x7f]),
            (128, &[0x80, 0x01]),
            (255, &[0xff, 0x01]),
            (i32::MAX, &[0xff, 0xff, 0xff, 0xff, 0x07]),
            (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
            (i32::MIN, &[0x80, 0x80, 0x80, 0x80, 0x08]),
        ];
        for (value, bytes) in vectors {
            assert_eq!(encode_i32(*value).as_slice(), *bytes);
            assert_eq!(decode_i32(bytes), Ok((*value, bytes.len())));
        }
    }

    #[test]
    fn accepts_non_minimal_encodings() {
        assert_eq!(decode_i32(&[0x80, 0x00]), Ok((0, 2)));
        assert_eq!(decode_i64(&[0x81, 0x00]), Ok((1, 2)));
    }

    #[test]
    fn rejects_width_overflow_and_truncation() {
        assert_eq!(
            decode_i32(&[0x80; 5]),
            Err(WireError::VariableIntegerTooLong {
                kind: "VarInt",
                maximum_bytes: 5,
            })
        );
        assert!(matches!(
            decode_i64(&[0x80]),
            Err(WireError::UnexpectedEnd { .. })
        ));
    }

    #[test]
    fn round_trips_varlong_boundaries() {
        for value in [0, 127, 128, i64::MAX, -1, i64::MIN] {
            let encoded = encode_i64(value);
            assert_eq!(
                decode_i64(encoded.as_slice()),
                Ok((value, encoded.as_slice().len()))
            );
        }
    }
}
