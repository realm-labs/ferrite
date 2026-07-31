//! Bounded big-endian NBT values used by official structure templates.

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use flate2::read::GzDecoder;
use thiserror::Error;

pub const MAX_NBT_DEPTH: usize = 512;
pub const MAX_NBT_BYTES: usize = 64 * 1024 * 1024;

pub type NbtCompound = BTreeMap<String, NbtValue>;

#[derive(Debug, Clone, PartialEq)]
pub enum NbtValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    String(String),
    List(Vec<Self>),
    Compound(NbtCompound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NbtValue {
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Self::Byte(value) => Some(i32::from(*value)),
            Self::Short(value) => Some(i32::from(*value)),
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(f64::from(*value)),
            Self::Double(value) => Some(*value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Self]> {
        match self {
            Self::List(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&NbtCompound> {
        match self {
            Self::Compound(value) => Some(value),
            _ => None,
        }
    }
}

pub fn decode_gzip_compound(bytes: &[u8]) -> Result<NbtCompound, NbtDecodeError> {
    let mut decoder = GzDecoder::new(Cursor::new(bytes));
    let mut uncompressed = Vec::new();
    decoder
        .by_ref()
        .take((MAX_NBT_BYTES + 1) as u64)
        .read_to_end(&mut uncompressed)
        .map_err(NbtDecodeError::Gzip)?;
    if uncompressed.len() > MAX_NBT_BYTES {
        return Err(NbtDecodeError::SizeLimit(MAX_NBT_BYTES));
    }
    decode_compound(&uncompressed)
}

pub fn decode_compound(bytes: &[u8]) -> Result<NbtCompound, NbtDecodeError> {
    let mut reader = NbtReader::new(bytes);
    let root_type = reader.byte()?;
    if root_type != 10 {
        return Err(NbtDecodeError::RootType(root_type));
    }
    reader.string()?;
    let NbtValue::Compound(compound) = reader.payload(10, 0)? else {
        unreachable!("compound payload has compound value")
    };
    if reader.remaining() != 0 {
        return Err(NbtDecodeError::Trailing(reader.remaining()));
    }
    Ok(compound)
}

struct NbtReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> NbtReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn payload(&mut self, tag: u8, depth: usize) -> Result<NbtValue, NbtDecodeError> {
        if depth >= MAX_NBT_DEPTH {
            return Err(NbtDecodeError::Depth(MAX_NBT_DEPTH));
        }
        match tag {
            1 => Ok(NbtValue::Byte(self.byte()? as i8)),
            2 => Ok(NbtValue::Short(self.i16()?)),
            3 => Ok(NbtValue::Int(self.i32()?)),
            4 => Ok(NbtValue::Long(self.i64()?)),
            5 => Ok(NbtValue::Float(f32::from_bits(self.u32()?))),
            6 => Ok(NbtValue::Double(f64::from_bits(self.u64()?))),
            7 => {
                let length = self.length("byte array")?;
                let values = self
                    .take(length)?
                    .iter()
                    .map(|value| *value as i8)
                    .collect();
                Ok(NbtValue::ByteArray(values))
            }
            8 => Ok(NbtValue::String(self.string()?)),
            9 => self.list(depth + 1),
            10 => self.compound(depth + 1),
            11 => {
                let length = self.length("int array")?;
                self.require_elements(length, 4)?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.i32()?);
                }
                Ok(NbtValue::IntArray(values))
            }
            12 => {
                let length = self.length("long array")?;
                self.require_elements(length, 8)?;
                let mut values = Vec::with_capacity(length);
                for _ in 0..length {
                    values.push(self.i64()?);
                }
                Ok(NbtValue::LongArray(values))
            }
            _ => Err(NbtDecodeError::TagType(tag)),
        }
    }

    fn list(&mut self, depth: usize) -> Result<NbtValue, NbtDecodeError> {
        let element_type = self.byte()?;
        let length = self.length("list")?;
        if element_type == 0 && length != 0 {
            return Err(NbtDecodeError::EndList(length));
        }
        if element_type > 12 {
            return Err(NbtDecodeError::TagType(element_type));
        }
        let minimum = minimum_payload_size(element_type);
        self.require_elements(length, minimum)?;
        let mut values = Vec::with_capacity(length);
        for _ in 0..length {
            values.push(self.payload(element_type, depth)?);
        }
        Ok(NbtValue::List(values))
    }

    fn compound(&mut self, depth: usize) -> Result<NbtValue, NbtDecodeError> {
        let mut values = NbtCompound::new();
        loop {
            let tag = self.byte()?;
            if tag == 0 {
                break;
            }
            if tag > 12 {
                return Err(NbtDecodeError::TagType(tag));
            }
            let name = self.string()?;
            let value = self.payload(tag, depth)?;
            if values.insert(name.clone(), value).is_some() {
                return Err(NbtDecodeError::Duplicate(name));
            }
        }
        Ok(NbtValue::Compound(values))
    }

    fn string(&mut self) -> Result<String, NbtDecodeError> {
        let length = usize::from(self.u16()?);
        decode_modified_utf(self.take(length)?)
    }

    fn length(&mut self, kind: &'static str) -> Result<usize, NbtDecodeError> {
        let value = self.i32()?;
        usize::try_from(value).map_err(|_| NbtDecodeError::NegativeLength { kind, value })
    }

    fn require_elements(&self, count: usize, minimum: usize) -> Result<(), NbtDecodeError> {
        let bytes = count
            .checked_mul(minimum)
            .ok_or(NbtDecodeError::LengthOverflow)?;
        if bytes > self.remaining() {
            return Err(NbtDecodeError::UnexpectedEnd {
                offset: self.offset,
                needed: bytes,
                remaining: self.remaining(),
            });
        }
        Ok(())
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], NbtDecodeError> {
        let end = self
            .offset
            .checked_add(length)
            .ok_or(NbtDecodeError::LengthOverflow)?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or(NbtDecodeError::UnexpectedEnd {
                offset: self.offset,
                needed: length,
                remaining: self.remaining(),
            })?;
        self.offset = end;
        Ok(value)
    }

    fn byte(&mut self) -> Result<u8, NbtDecodeError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, NbtDecodeError> {
        Ok(u16::from_be_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }

    fn i16(&mut self) -> Result<i16, NbtDecodeError> {
        Ok(i16::from_be_bytes(
            self.take(2)?.try_into().expect("two bytes"),
        ))
    }

    fn u32(&mut self) -> Result<u32, NbtDecodeError> {
        Ok(u32::from_be_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    fn i32(&mut self) -> Result<i32, NbtDecodeError> {
        Ok(i32::from_be_bytes(
            self.take(4)?.try_into().expect("four bytes"),
        ))
    }

    fn u64(&mut self) -> Result<u64, NbtDecodeError> {
        Ok(u64::from_be_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }

    fn i64(&mut self) -> Result<i64, NbtDecodeError> {
        Ok(i64::from_be_bytes(
            self.take(8)?.try_into().expect("eight bytes"),
        ))
    }
}

fn minimum_payload_size(tag: u8) -> usize {
    match tag {
        1 => 1,
        2 => 2,
        3 | 5 => 4,
        4 | 6 => 8,
        7 | 9 | 11 | 12 => 4,
        8 => 2,
        10 => 1,
        _ => 0,
    }
}

fn decode_modified_utf(bytes: &[u8]) -> Result<String, NbtDecodeError> {
    let mut units = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        let first = bytes[index];
        index += 1;
        let unit = match first {
            0x01..=0x7f => u16::from(first),
            0xc0..=0xdf => {
                let second = continuation(bytes, &mut index)?;
                let value = (u16::from(first & 0x1f) << 6) | u16::from(second);
                if value == 0 && first == 0xc0 && second == 0 {
                    0
                } else if value < 0x80 {
                    return Err(NbtDecodeError::ModifiedUtf);
                } else {
                    value
                }
            }
            0xe0..=0xef => {
                let second = continuation(bytes, &mut index)?;
                let third = continuation(bytes, &mut index)?;
                let value =
                    (u16::from(first & 0x0f) << 12) | (u16::from(second) << 6) | u16::from(third);
                if value < 0x800 {
                    return Err(NbtDecodeError::ModifiedUtf);
                }
                value
            }
            _ => return Err(NbtDecodeError::ModifiedUtf),
        };
        units.push(unit);
    }
    String::from_utf16(&units).map_err(|_| NbtDecodeError::ModifiedUtf)
}

fn continuation(bytes: &[u8], index: &mut usize) -> Result<u8, NbtDecodeError> {
    let byte = *bytes.get(*index).ok_or(NbtDecodeError::ModifiedUtf)?;
    *index += 1;
    if byte & 0xc0 != 0x80 {
        return Err(NbtDecodeError::ModifiedUtf);
    }
    Ok(byte & 0x3f)
}

#[derive(Debug, Error)]
pub enum NbtDecodeError {
    #[error("gzip decode failed: {0}")]
    Gzip(std::io::Error),
    #[error("decompressed NBT exceeds {0} bytes")]
    SizeLimit(usize),
    #[error("NBT root tag is {0}, expected compound tag 10")]
    RootType(u8),
    #[error("NBT tag type {0} is outside 1..=12")]
    TagType(u8),
    #[error("NBT {kind} length is negative: {value}")]
    NegativeLength { kind: &'static str, value: i32 },
    #[error("NBT end-tag list has nonzero length {0}")]
    EndList(usize),
    #[error("NBT length arithmetic overflow")]
    LengthOverflow,
    #[error("NBT ended at {offset}: needed {needed} bytes, {remaining} remain")]
    UnexpectedEnd {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    #[error("NBT nesting exceeds {0}")]
    Depth(usize),
    #[error("NBT compound repeats key {0}")]
    Duplicate(String),
    #[error("NBT modified UTF-8 is malformed")]
    ModifiedUtf,
    #[error("NBT has {0} trailing bytes")]
    Trailing(usize),
}
