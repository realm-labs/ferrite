use std::str;
use thiserror::Error;

#[derive(Debug, Default)]
pub(crate) struct Encoder {
    bytes: Vec<u8>,
}

impl Encoder {
    pub(crate) const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub(crate) fn fixed(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    pub(crate) fn u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub(crate) fn u16(&mut self, value: u16) {
        self.fixed(&value.to_le_bytes());
    }

    pub(crate) fn i32(&mut self, value: i32) {
        self.fixed(&value.to_le_bytes());
    }

    pub(crate) fn u64(&mut self, value: u64) {
        self.fixed(&value.to_le_bytes());
    }

    pub(crate) fn u128(&mut self, value: u128) {
        self.fixed(&value.to_le_bytes());
    }

    pub(crate) fn var_u64(&mut self, mut value: u64) {
        loop {
            let payload = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                self.u8(payload);
                break;
            }
            self.u8(payload | 0x80);
        }
    }

    pub(crate) fn bytes(&mut self, bytes: &[u8], maximum: usize) -> Result<(), CodecError> {
        if bytes.len() > maximum {
            return Err(CodecError::LengthLimit {
                actual: bytes.len(),
                maximum,
            });
        }
        self.var_u64(bytes.len() as u64);
        self.fixed(bytes);
        Ok(())
    }

    pub(crate) fn string(&mut self, value: &str, maximum: usize) -> Result<(), CodecError> {
        self.bytes(value.as_bytes(), maximum)
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub(crate) struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub(crate) fn expect(&mut self, expected: &[u8]) -> Result<(), CodecError> {
        if self.take(expected.len())? != expected {
            return Err(CodecError::WrongMagic);
        }
        Ok(())
    }

    pub(crate) fn u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.take(1)?[0])
    }

    pub(crate) fn u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_le_bytes(self.fixed()?))
    }

    pub(crate) fn i32(&mut self) -> Result<i32, CodecError> {
        Ok(i32::from_le_bytes(self.fixed()?))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_le_bytes(self.fixed()?))
    }

    pub(crate) fn u128(&mut self) -> Result<u128, CodecError> {
        Ok(u128::from_le_bytes(self.fixed()?))
    }

    pub(crate) fn var_u64(&mut self) -> Result<u64, CodecError> {
        let mut value = 0_u64;
        for index in 0..10 {
            let byte = self.u8()?;
            let payload = byte & 0x7f;
            if index == 9 && payload > 1 {
                return Err(CodecError::VarIntOverflow);
            }
            value |= u64::from(payload) << (index * 7);
            if byte & 0x80 == 0 {
                if index != 0 && payload == 0 {
                    return Err(CodecError::NonMinimalVarInt);
                }
                return Ok(value);
            }
        }
        Err(CodecError::VarIntOverflow)
    }

    pub(crate) fn length(&mut self, maximum: usize) -> Result<usize, CodecError> {
        let value = usize::try_from(self.var_u64()?).map_err(|_| CodecError::LengthLimit {
            actual: usize::MAX,
            maximum,
        })?;
        if value > maximum {
            return Err(CodecError::LengthLimit {
                actual: value,
                maximum,
            });
        }
        Ok(value)
    }

    pub(crate) fn bytes(&mut self, maximum: usize) -> Result<&'a [u8], CodecError> {
        let length = self.length(maximum)?;
        self.take(length)
    }

    pub(crate) fn string(&mut self, maximum: usize) -> Result<&'a str, CodecError> {
        str::from_utf8(self.bytes(maximum)?).map_err(|_| CodecError::InvalidUtf8)
    }

    pub(crate) fn fixed<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        self.take(N)?.try_into().map_err(|_| CodecError::Truncated)
    }

    pub(crate) fn finish(self) -> Result<(), CodecError> {
        if self.offset == self.bytes.len() {
            Ok(())
        } else {
            Err(CodecError::TrailingBytes)
        }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], CodecError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(CodecError::Truncated)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(CodecError::Truncated)?;
        self.offset = end;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CodecError {
    #[error("persistence record has the wrong magic")]
    WrongMagic,
    #[error("persistence record is truncated")]
    Truncated,
    #[error("persistence record has trailing bytes")]
    TrailingBytes,
    #[error("persistence record contains invalid UTF-8")]
    InvalidUtf8,
    #[error("persistence record contains a non-minimal variable integer")]
    NonMinimalVarInt,
    #[error("persistence record variable integer overflows u64")]
    VarIntOverflow,
    #[error("persistence record length {actual} exceeds limit {maximum}")]
    LengthLimit { actual: usize, maximum: usize },
}
