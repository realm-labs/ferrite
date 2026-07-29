use std::borrow::Cow;

use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::varint::{decode_i32, decode_i64, encode_i32, encode_i64};

/// A packet-bounded reader for Minecraft's structured wire primitives.
#[derive(Debug, Clone)]
pub struct WireReader<'a> {
    input: &'a [u8],
    offset: usize,
}

impl<'a> WireReader<'a> {
    #[must_use]
    pub const fn new(input: &'a [u8]) -> Self {
        Self { input, offset: 0 }
    }

    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.input.len() - self.offset
    }

    #[must_use]
    pub const fn consumed(&self) -> usize {
        self.offset
    }

    pub fn finish(self) -> Result<(), WireError> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(WireError::LengthLimit {
                field: "trailing packet data",
                length: self.remaining(),
                maximum: 0,
            })
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, WireError> {
        Ok(self.take(1, "byte")?[0])
    }

    pub fn read_i8(&mut self) -> Result<i8, WireError> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_bool(&mut self) -> Result<bool, WireError> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> Result<u16, WireError> {
        let bytes = self.take(2, "unsigned short")?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_i16(&mut self) -> Result<i16, WireError> {
        let bytes = self.take(2, "signed short")?;
        Ok(i16::from_be_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_i32(&mut self) -> Result<i32, WireError> {
        let bytes = self.take(4, "signed int")?;
        Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_i64(&mut self) -> Result<i64, WireError> {
        let bytes = self.take(8, "signed long")?;
        Ok(i64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_u128(&mut self) -> Result<u128, WireError> {
        let bytes = self.take(16, "unsigned 128-bit value")?;
        Ok(u128::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]))
    }

    pub fn read_f32(&mut self) -> Result<f32, WireError> {
        let bytes = self.take(4, "float")?;
        Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_f64(&mut self) -> Result<f64, WireError> {
        let bytes = self.take(8, "double")?;
        Ok(f64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_var_i32(&mut self) -> Result<i32, WireError> {
        let (value, consumed) = decode_i32(&self.input[self.offset..])?;
        self.offset += consumed;
        Ok(value)
    }

    pub fn read_var_i64(&mut self) -> Result<i64, WireError> {
        let (value, consumed) = decode_i64(&self.input[self.offset..])?;
        self.offset += consumed;
        Ok(value)
    }

    pub fn read_byte_array(&mut self, maximum: usize) -> Result<&'a [u8], WireError> {
        let length = self.read_nonnegative_length("byte array")?;
        if length > maximum {
            return Err(WireError::LengthLimit {
                field: "byte array",
                length,
                maximum,
            });
        }
        self.take(length, "byte array")
    }

    pub fn read_count(&mut self, field: &'static str, maximum: usize) -> Result<usize, WireError> {
        let count = self.read_nonnegative_length(field)?;
        if count > maximum {
            Err(WireError::LengthLimit {
                field,
                length: count,
                maximum,
            })
        } else {
            Ok(count)
        }
    }

    pub fn read_utf(&mut self, maximum_code_units: usize) -> Result<Cow<'a, str>, WireError> {
        let maximum_bytes = maximum_code_units.saturating_mul(3);
        let length = self.read_nonnegative_length("UTF")?;
        if length > maximum_bytes {
            return Err(WireError::LengthLimit {
                field: "UTF",
                length,
                maximum: maximum_bytes,
            });
        }
        let decoded = String::from_utf8_lossy(self.take(length, "UTF")?);
        let actual_code_units = decoded.encode_utf16().count();
        if actual_code_units > maximum_code_units {
            return Err(WireError::UtfCodeUnitLimit {
                actual: actual_code_units,
                maximum: maximum_code_units,
            });
        }
        Ok(decoded)
    }

    pub fn read_bounded_remaining(
        &mut self,
        field: &'static str,
        maximum: usize,
    ) -> Result<&'a [u8], WireError> {
        let length = self.remaining();
        if length > maximum {
            Err(WireError::LengthLimit {
                field,
                length,
                maximum,
            })
        } else {
            Ok(self.take_remaining())
        }
    }

    pub(crate) fn take_remaining(&mut self) -> &'a [u8] {
        let remaining = &self.input[self.offset..];
        self.offset = self.input.len();
        remaining
    }

    fn read_nonnegative_length(&mut self, field: &'static str) -> Result<usize, WireError> {
        let value = self.read_var_i32()?;
        usize::try_from(value).map_err(|_| WireError::NegativeLength { field, value })
    }

    fn take(&mut self, length: usize, field: &'static str) -> Result<&'a [u8], WireError> {
        if length > self.remaining() {
            return Err(WireError::UnexpectedEnd {
                field,
                needed: length,
                remaining: self.remaining(),
            });
        }
        let start = self.offset;
        self.offset += length;
        Ok(&self.input[start..self.offset])
    }
}

/// A size-bounded writer for Minecraft's structured wire primitives.
#[derive(Debug, Clone)]
pub struct WireWriter {
    output: Vec<u8>,
    maximum: usize,
}

impl WireWriter {
    #[must_use]
    pub fn new(maximum: usize) -> Self {
        Self {
            output: Vec::new(),
            maximum,
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.output.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.output
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.output
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), WireError> {
        self.append(&[value])
    }

    pub fn write_i8(&mut self, value: i8) -> Result<(), WireError> {
        self.write_u8(value as u8)
    }

    pub fn write_bool(&mut self, value: bool) -> Result<(), WireError> {
        self.write_u8(u8::from(value))
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_i16(&mut self, value: i16) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_i32(&mut self, value: i32) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_i64(&mut self, value: i64) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_u128(&mut self, value: u128) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), WireError> {
        self.append(&value.to_be_bytes())
    }

    pub fn write_var_i32(&mut self, value: i32) -> Result<(), WireError> {
        self.append(encode_i32(value).as_slice())
    }

    pub fn write_var_i64(&mut self, value: i64) -> Result<(), WireError> {
        self.append(encode_i64(value).as_slice())
    }

    pub fn write_byte_array(&mut self, value: &[u8], maximum: usize) -> Result<(), WireError> {
        self.check_field_limit("byte array", value.len(), maximum)?;
        let length = i32::try_from(value.len()).map_err(|_| WireError::LengthLimit {
            field: "byte array",
            length: value.len(),
            maximum: i32::MAX as usize,
        })?;
        let encoded_length = encode_i32(length);
        self.reserve_for(encoded_length.as_slice().len() + value.len())?;
        self.output.extend_from_slice(encoded_length.as_slice());
        self.output.extend_from_slice(value);
        Ok(())
    }

    pub fn write_count(
        &mut self,
        field: &'static str,
        count: usize,
        maximum: usize,
    ) -> Result<(), WireError> {
        self.check_field_limit(field, count, maximum)?;
        let count = i32::try_from(count).map_err(|_| WireError::LengthLimit {
            field,
            length: count,
            maximum: i32::MAX as usize,
        })?;
        self.write_var_i32(count)
    }

    pub fn write_utf(&mut self, value: &str, maximum_code_units: usize) -> Result<(), WireError> {
        let actual_code_units = value.encode_utf16().count();
        if actual_code_units > maximum_code_units {
            return Err(WireError::UtfCodeUnitLimit {
                actual: actual_code_units,
                maximum: maximum_code_units,
            });
        }
        let maximum_bytes = maximum_code_units.saturating_mul(3);
        self.check_field_limit("UTF", value.len(), maximum_bytes)?;
        let length = i32::try_from(value.len()).map_err(|_| WireError::LengthLimit {
            field: "UTF",
            length: value.len(),
            maximum: i32::MAX as usize,
        })?;
        let encoded_length = encode_i32(length);
        self.reserve_for(encoded_length.as_slice().len() + value.len())?;
        self.output.extend_from_slice(encoded_length.as_slice());
        self.output.extend_from_slice(value.as_bytes());
        Ok(())
    }

    pub fn write_bytes(&mut self, value: &[u8]) -> Result<(), WireError> {
        self.append(value)
    }

    fn check_field_limit(
        &self,
        field: &'static str,
        length: usize,
        maximum: usize,
    ) -> Result<(), WireError> {
        if length > maximum {
            Err(WireError::LengthLimit {
                field,
                length,
                maximum,
            })
        } else {
            Ok(())
        }
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), WireError> {
        self.reserve_for(bytes.len())?;
        self.output.extend_from_slice(bytes);
        Ok(())
    }

    fn reserve_for(&self, additional: usize) -> Result<(), WireError> {
        let attempted = self.output.len().saturating_add(additional);
        if attempted > self.maximum {
            Err(WireError::OutputLimit {
                attempted,
                maximum: self.maximum,
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::java_26_2::wire::error::WireError;
    use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

    #[test]
    fn reads_locked_boolean_and_numeric_rules() {
        let mut reader = WireReader::new(&[
            0, 2, 0x12, 0x34, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
        ]);
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.read_i64().unwrap(), -2);
        assert_eq!(reader.finish(), Ok(()));
    }

    #[test]
    fn utf_uses_utf16_units_and_lossy_decode() {
        let mut writer = WireWriter::new(32);
        writer.write_utf("a😀", 3).unwrap();
        let mut reader = WireReader::new(writer.as_slice());
        assert_eq!(reader.read_utf(3).unwrap(), Cow::Borrowed("a😀"));

        let mut malformed = WireReader::new(&[2, 0xc3, 0x28]);
        assert_eq!(malformed.read_utf(2).unwrap(), "�(");

        let mut locked_malformed = WireReader::new(&[1, 0xff]);
        assert_eq!(locked_malformed.read_utf(1).unwrap(), "�");
    }

    #[test]
    fn utf_rejects_encoded_and_decoded_limits() {
        let mut encoded_too_long = WireReader::new(&[4, b'a', b'b', b'c', b'd']);
        assert_eq!(
            encoded_too_long.read_utf(1),
            Err(WireError::LengthLimit {
                field: "UTF",
                length: 4,
                maximum: 3,
            })
        );

        let mut too_many_units = WireReader::new(&[3, b'a', b'b', b'c']);
        assert_eq!(
            too_many_units.read_utf(2),
            Err(WireError::UtfCodeUnitLimit {
                actual: 3,
                maximum: 2,
            })
        );
    }

    #[test]
    fn bounded_writes_are_atomic() {
        let mut writer = WireWriter::new(3);
        writer.write_u8(7).unwrap();
        assert!(matches!(
            writer.write_byte_array(&[1, 2], 2),
            Err(WireError::OutputLimit { .. })
        ));
        assert_eq!(writer.as_slice(), &[7]);
    }

    #[test]
    fn byte_arrays_reject_negative_and_oversized_lengths() {
        let mut negative = WireReader::new(&[0xff, 0xff, 0xff, 0xff, 0x0f]);
        assert!(matches!(
            negative.read_byte_array(8),
            Err(WireError::NegativeLength { .. })
        ));
        let mut oversized = WireReader::new(&[3, 1, 2, 3]);
        assert!(matches!(
            oversized.read_byte_array(2),
            Err(WireError::LengthLimit { .. })
        ));
    }

    #[test]
    fn counted_and_fixed_width_structures_are_bounded() {
        let value = 0x0011_2233_4455_6677_8899_aabb_ccdd_eeff;
        let mut writer = WireWriter::new(32);
        writer.write_count("properties", 2, 16).unwrap();
        writer.write_u128(value).unwrap();
        let mut reader = WireReader::new(writer.as_slice());
        assert_eq!(reader.read_count("properties", 16).unwrap(), 2);
        assert_eq!(reader.read_u128().unwrap(), value);
        assert_eq!(reader.finish(), Ok(()));

        let mut count = WireReader::new(&[17]);
        assert!(matches!(
            count.read_count("properties", 16),
            Err(WireError::LengthLimit { .. })
        ));
    }
}
