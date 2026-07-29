//! Explicit canonical binary primitives from ADR-0015.

use std::str;
use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn write_magic(&mut self, magic: &[u8]) {
        self.bytes.extend_from_slice(magic);
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u64(&mut self, value: u64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u128(&mut self, value: u128) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), EncodeError> {
        if !value.is_finite() {
            return Err(EncodeError::NonFiniteFloat);
        }
        self.write_u32(value.to_bits());
        Ok(())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), EncodeError> {
        if !value.is_finite() {
            return Err(EncodeError::NonFiniteFloat);
        }
        self.write_u64(value.to_bits());
        Ok(())
    }

    pub fn write_var_u64(&mut self, mut value: u64) {
        loop {
            let payload = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.write_u8(payload);
                return;
            }
            self.write_u8(payload | 0x80);
        }
    }

    pub fn write_bytes(&mut self, value: &[u8], maximum: usize) -> Result<(), EncodeError> {
        if value.len() > maximum {
            return Err(EncodeError::LengthLimit {
                actual: value.len(),
                maximum,
            });
        }
        self.write_var_u64(value.len() as u64);
        self.bytes.extend_from_slice(value);
        Ok(())
    }

    pub fn write_string(&mut self, value: &str, maximum_bytes: usize) -> Result<(), EncodeError> {
        self.write_bytes(value.as_bytes(), maximum_bytes)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Debug, Clone)]
pub struct Decoder<'a> {
    input: &'a [u8],
    position: usize,
}

impl<'a> Decoder<'a> {
    pub const fn new(input: &'a [u8]) -> Self {
        Self { input, position: 0 }
    }

    pub fn expect_magic(&mut self, expected: &[u8]) -> Result<(), DecodeError> {
        let actual = self.take(expected.len())?;
        if actual != expected {
            return Err(DecodeError::WrongMagic);
        }
        Ok(())
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }

    pub fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(DecodeError::InvalidBoolean { value }),
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_u64(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_i64(&mut self) -> Result<i64, DecodeError> {
        Ok(i64::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_u128(&mut self) -> Result<u128, DecodeError> {
        Ok(u128::from_le_bytes(self.read_fixed()?))
    }

    pub fn read_f32(&mut self) -> Result<f32, DecodeError> {
        let value = f32::from_bits(self.read_u32()?);
        if !value.is_finite() {
            return Err(DecodeError::NonFiniteFloat);
        }
        Ok(value)
    }

    pub fn read_f64(&mut self) -> Result<f64, DecodeError> {
        let value = f64::from_bits(self.read_u64()?);
        if !value.is_finite() {
            return Err(DecodeError::NonFiniteFloat);
        }
        Ok(value)
    }

    pub fn read_var_u64(&mut self) -> Result<u64, DecodeError> {
        let mut value = 0_u64;
        for index in 0..10 {
            let byte = self.read_u8()?;
            let payload = byte & 0x7f;
            if index == 9 && payload > 1 {
                return Err(DecodeError::VarIntOverflow);
            }
            value |= u64::from(payload) << (index * 7);
            if byte & 0x80 == 0 {
                if index != 0 && payload == 0 {
                    return Err(DecodeError::NonMinimalVarInt);
                }
                return Ok(value);
            }
        }
        Err(DecodeError::VarIntOverflow)
    }

    pub fn read_bytes(&mut self, maximum: usize) -> Result<&'a [u8], DecodeError> {
        let length = self.read_length(maximum)?;
        self.take(length)
    }

    pub fn read_string(&mut self, maximum_bytes: usize) -> Result<&'a str, DecodeError> {
        str::from_utf8(self.read_bytes(maximum_bytes)?).map_err(|_| DecodeError::InvalidUtf8)
    }

    pub fn read_length(&mut self, maximum: usize) -> Result<usize, DecodeError> {
        let encoded = self.read_var_u64()?;
        let length = usize::try_from(encoded).map_err(|_| DecodeError::LengthLimit {
            actual: usize::MAX,
            maximum,
        })?;
        if length > maximum {
            return Err(DecodeError::LengthLimit {
                actual: length,
                maximum,
            });
        }
        Ok(length)
    }

    pub fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        self.take(N)?.try_into().map_err(|_| DecodeError::Truncated)
    }

    pub fn finish(self) -> Result<(), DecodeError> {
        if self.position != self.input.len() {
            return Err(DecodeError::TrailingBytes {
                remaining: self.input.len() - self.position,
            });
        }
        Ok(())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(DecodeError::Truncated)?;
        let value = self
            .input
            .get(self.position..end)
            .ok_or(DecodeError::Truncated)?;
        self.position = end;
        Ok(value)
    }
}

pub trait CanonicalEncode {
    fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError>;
}

pub trait CanonicalDecode: Sized {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError>;
}

pub fn encode_to_vec<T: CanonicalEncode>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut encoder = Encoder::new();
    value.encode(&mut encoder)?;
    Ok(encoder.into_bytes())
}

pub fn decode_exact<T: CanonicalDecode>(bytes: &[u8]) -> Result<T, DecodeError> {
    let mut decoder = Decoder::new(bytes);
    let value = T::decode(&mut decoder)?;
    decoder.finish()?;
    Ok(value)
}

pub fn encode_sorted_set<'a, T>(
    encoder: &mut Encoder,
    values: impl IntoIterator<Item = &'a T>,
    maximum: usize,
) -> Result<(), EncodeError>
where
    T: CanonicalEncode + 'a,
{
    let mut encoded = values
        .into_iter()
        .map(encode_to_vec)
        .collect::<Result<Vec<_>, _>>()?;
    if encoded.len() > maximum {
        return Err(EncodeError::LengthLimit {
            actual: encoded.len(),
            maximum,
        });
    }
    encoded.sort();
    if encoded.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(EncodeError::DuplicateCanonicalKey);
    }
    encoder.write_var_u64(encoded.len() as u64);
    for value in encoded {
        encoder.write_magic(&value);
    }
    Ok(())
}

pub fn encode_sorted_map<'a, K, V>(
    encoder: &mut Encoder,
    entries: impl IntoIterator<Item = (&'a K, &'a V)>,
    maximum: usize,
) -> Result<(), EncodeError>
where
    K: CanonicalEncode + 'a,
    V: CanonicalEncode + 'a,
{
    let mut encoded = entries
        .into_iter()
        .map(|(key, value)| Ok((encode_to_vec(key)?, encode_to_vec(value)?)))
        .collect::<Result<Vec<_>, EncodeError>>()?;
    if encoded.len() > maximum {
        return Err(EncodeError::LengthLimit {
            actual: encoded.len(),
            maximum,
        });
    }
    encoded.sort_by(|left, right| left.0.cmp(&right.0));
    if encoded.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(EncodeError::DuplicateCanonicalKey);
    }
    encoder.write_var_u64(encoded.len() as u64);
    for (key, value) in encoded {
        encoder.write_magic(&key);
        encoder.write_magic(&value);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EncodeError {
    #[error("canonical value length {actual} exceeds {maximum}")]
    LengthLimit { actual: usize, maximum: usize },
    #[error("authoritative floating-point values must be finite")]
    NonFiniteFloat,
    #[error("canonical map or set contains a duplicate encoded key")]
    DuplicateCanonicalKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DecodeError {
    #[error("canonical input is truncated")]
    Truncated,
    #[error("canonical input has the wrong magic or domain")]
    WrongMagic,
    #[error("canonical boolean tag {value} is invalid")]
    InvalidBoolean { value: u8 },
    #[error("canonical unsigned LEB128 overflows u64")]
    VarIntOverflow,
    #[error("canonical unsigned LEB128 is not minimally encoded")]
    NonMinimalVarInt,
    #[error("canonical value length {actual} exceeds {maximum}")]
    LengthLimit { actual: usize, maximum: usize },
    #[error("canonical string is not valid UTF-8")]
    InvalidUtf8,
    #[error("authoritative floating-point values must be finite")]
    NonFiniteFloat,
    #[error("canonical {kind} value violates its semantic invariant")]
    InvalidSemantic { kind: &'static str },
    #[error("canonical {kind} enum tag {tag} is unknown")]
    InvalidEnumTag { kind: &'static str, tag: u64 },
    #[error("canonical input has {remaining} trailing bytes")]
    TrailingBytes { remaining: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Byte(u8);

    impl CanonicalEncode for Byte {
        fn encode(&self, encoder: &mut Encoder) -> Result<(), EncodeError> {
            encoder.write_u8(self.0);
            Ok(())
        }
    }

    #[test]
    fn unsigned_leb128_has_locked_minimal_vectors() {
        let mut encoder = Encoder::new();
        for value in [0, 1, 127, 128, 255, 16_384, u64::MAX] {
            encoder.write_var_u64(value);
        }
        assert_eq!(
            encoder.into_bytes(),
            [
                0x00, 0x01, 0x7f, 0x80, 0x01, 0xff, 0x01, 0x80, 0x80, 0x01, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0x01,
            ]
        );
        assert_eq!(Decoder::new(&[0x80, 0x01]).read_var_u64().unwrap(), 128);
        assert!(matches!(
            Decoder::new(&[0x80, 0x00]).read_var_u64(),
            Err(DecodeError::NonMinimalVarInt)
        ));
    }

    #[test]
    fn bounds_trailing_data_and_nonfinite_values_fail_closed() {
        let mut encoder = Encoder::new();
        assert!(encoder.write_bytes(&[1, 2, 3], 2).is_err());
        assert!(encoder.write_f64(f64::NAN).is_err());
        assert!(Decoder::new(&[2, 1, 2]).read_bytes(1).is_err());
        assert!(
            Decoder::new(&f32::INFINITY.to_bits().to_le_bytes())
                .read_f32()
                .is_err()
        );
        assert!(Decoder::new(&[1]).finish().is_err());
    }

    #[test]
    fn canonical_maps_sort_key_bytes_and_reject_duplicates() {
        let keys = [Byte(2), Byte(1)];
        let values = [Byte(20), Byte(10)];
        let mut first = Encoder::new();
        encode_sorted_map(
            &mut first,
            [(&keys[0], &values[0]), (&keys[1], &values[1])],
            2,
        )
        .unwrap();
        let mut second = Encoder::new();
        encode_sorted_map(
            &mut second,
            [(&keys[1], &values[1]), (&keys[0], &values[0])],
            2,
        )
        .unwrap();
        assert_eq!(first.into_bytes(), second.into_bytes());

        let mut duplicate = Encoder::new();
        assert!(
            encode_sorted_map(
                &mut duplicate,
                [(&keys[0], &values[0]), (&keys[0], &values[1])],
                2,
            )
            .is_err()
        );
    }
}
