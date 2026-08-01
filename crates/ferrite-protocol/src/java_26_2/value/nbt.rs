use std::collections::BTreeSet;

use thiserror::Error;

use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::frame::MAX_FRAME_LENGTH;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

pub const DEFAULT_NBT_QUOTA: u64 = 2_097_152;
pub const MAX_NBT_DEPTH: usize = 512;

/// The accumulator policy selected by the packet's locked Minecraft codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtQuota {
    Default,
    Trusted,
    Bounded {
        maximum_bytes: u64,
        maximum_depth: usize,
    },
}

impl NbtQuota {
    const fn bytes(self) -> u64 {
        match self {
            Self::Default => DEFAULT_NBT_QUOTA,
            Self::Trusted => u64::MAX,
            Self::Bounded { maximum_bytes, .. } => maximum_bytes,
        }
    }

    const fn depth(self) -> usize {
        match self {
            Self::Default | Self::Trusted => MAX_NBT_DEPTH,
            Self::Bounded { maximum_depth, .. } => maximum_depth,
        }
    }
}

/// One validated, unnamed network NBT tag, retaining its exact wire representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkNbt {
    bytes: Vec<u8>,
    root_tag_id: u8,
}

/// A structurally valid NBT representation in the root forms admitted by the component codec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextComponentNbt(NetworkNbt);

impl TextComponentNbt {
    pub fn from_network_nbt(value: NetworkNbt) -> Result<Self, NbtError> {
        if component_root_shape_is_valid(&value) {
            Ok(Self(value))
        } else {
            Err(NbtError::InvalidComponentShape {
                tag_id: value.root_tag_id(),
            })
        }
    }

    pub fn literal(text: &str) -> Result<Self, NbtError> {
        NetworkNbt::literal_component(text).map(Self)
    }

    pub fn translatable(key: &str) -> Result<Self, NbtError> {
        let name = encode_modified_utf("translate")?;
        let value = encode_modified_utf(key)?;
        let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
        writer.write_u8(10)?;
        writer.write_u8(8)?;
        writer.write_u16(name.len() as u16)?;
        writer.write_bytes(&name)?;
        writer.write_u16(value.len() as u16)?;
        writer.write_bytes(&value)?;
        writer.write_u8(0)?;
        NetworkNbt::from_bytes(writer.into_inner(), NbtQuota::Trusted).map(Self)
    }

    #[must_use]
    pub fn network_nbt(&self) -> &NetworkNbt {
        &self.0
    }
}

impl NetworkNbt {
    pub fn from_bytes(bytes: Vec<u8>, quota: NbtQuota) -> Result<Self, NbtError> {
        let mut reader = WireReader::new(&bytes);
        let value = Self::read(&mut reader, quota)?;
        reader.finish()?;
        Ok(value)
    }

    /// Creates the canonical primitive-string representation accepted by the component codec.
    pub fn literal_component(text: &str) -> Result<Self, NbtError> {
        let encoded = encode_modified_utf(text)?;
        let mut writer = WireWriter::new(MAX_FRAME_LENGTH);
        writer.write_u8(8)?;
        writer.write_u16(encoded.len() as u16)?;
        writer.write_bytes(&encoded)?;
        Self::from_bytes(writer.into_inner(), NbtQuota::Trusted)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub const fn root_tag_id(&self) -> u8 {
        self.root_tag_id
    }

    pub(crate) fn read(reader: &mut WireReader<'_>, quota: NbtQuota) -> Result<Self, NbtError> {
        Self::read_nullable(reader, quota)?.ok_or(NbtError::NullTag)
    }

    pub(crate) fn read_nullable(
        reader: &mut WireReader<'_>,
        quota: NbtQuota,
    ) -> Result<Option<Self>, NbtError> {
        let start = reader.consumed();
        let root_tag_id = reader.read_u8()?;
        if root_tag_id == 0 {
            return Ok(None);
        }
        let mut accounter = NbtAccounter::new(quota.bytes(), quota.depth());
        scan_payload(reader, root_tag_id, &mut accounter)?;
        Ok(Some(Self {
            bytes: reader.bytes_since(start).to_vec(),
            root_tag_id,
        }))
    }

    pub(crate) fn write(&self, writer: &mut WireWriter) -> Result<(), WireError> {
        writer.write_bytes(&self.bytes)
    }

    pub(crate) fn write_nullable(
        value: Option<&Self>,
        writer: &mut WireWriter,
    ) -> Result<(), WireError> {
        if let Some(value) = value {
            value.write(writer)
        } else {
            writer.write_u8(0)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NbtError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("network NBT used the null/end root tag")]
    NullTag,
    #[error("component NBT root tag type {tag_id} has no valid component shape")]
    InvalidComponentShape { tag_id: u8 },
    #[error("NBT tag type {tag_id} is outside 0..=12")]
    InvalidTagType { tag_id: u8 },
    #[error("{kind} NBT length is negative: {length}")]
    NegativeLength { kind: &'static str, length: i32 },
    #[error("NBT list has end element type with nonzero length {length}")]
    NonemptyEndList { length: usize },
    #[error("NBT modified UTF data is malformed")]
    InvalidModifiedUtf,
    #[error("NBT modified UTF encoding contains {length} bytes, above 65535")]
    ModifiedUtfTooLong { length: usize },
    #[error("NBT accumulator usage would exceed its {quota}-byte quota")]
    QuotaExceeded { quota: u64 },
    #[error("NBT nesting exceeds the locked depth {maximum}")]
    DepthExceeded { maximum: usize },
}

#[derive(Debug)]
struct NbtAccounter {
    quota: u64,
    maximum_depth: usize,
    usage: u64,
    depth: usize,
}

impl NbtAccounter {
    const fn new(quota: u64, maximum_depth: usize) -> Self {
        Self {
            quota,
            maximum_depth,
            usage: 0,
            depth: 0,
        }
    }

    fn account(&mut self, bytes: u64) -> Result<(), NbtError> {
        let usage = self
            .usage
            .checked_add(bytes)
            .ok_or(NbtError::QuotaExceeded { quota: self.quota })?;
        if usage > self.quota {
            Err(NbtError::QuotaExceeded { quota: self.quota })
        } else {
            self.usage = usage;
            Ok(())
        }
    }

    fn account_entries(&mut self, bytes: u64, count: usize) -> Result<(), NbtError> {
        let total = bytes
            .checked_mul(count as u64)
            .ok_or(NbtError::QuotaExceeded { quota: self.quota })?;
        self.account(total)
    }

    fn push(&mut self) -> Result<(), NbtError> {
        if self.depth >= self.maximum_depth {
            Err(NbtError::DepthExceeded {
                maximum: self.maximum_depth,
            })
        } else {
            self.depth += 1;
            Ok(())
        }
    }

    fn pop(&mut self) {
        self.depth -= 1;
    }
}

fn scan_payload(
    reader: &mut WireReader<'_>,
    tag_id: u8,
    accounter: &mut NbtAccounter,
) -> Result<(), NbtError> {
    match tag_id {
        1 => {
            reader.read_i8()?;
            accounter.account(9)
        }
        2 => {
            reader.read_i16()?;
            accounter.account(10)
        }
        3 => {
            reader.read_i32()?;
            accounter.account(12)
        }
        4 => {
            reader.read_i64()?;
            accounter.account(16)
        }
        5 => {
            reader.read_f32()?;
            accounter.account(12)
        }
        6 => {
            reader.read_f64()?;
            accounter.account(16)
        }
        7 => scan_array(reader, accounter, "byte array", 1, 24),
        8 => {
            accounter.account(36)?;
            let units = scan_modified_utf(reader)?;
            accounter.account_entries(2, units.len())
        }
        9 => scan_list(reader, accounter),
        10 => scan_compound(reader, accounter),
        11 => scan_array(reader, accounter, "int array", 4, 24),
        12 => scan_array(reader, accounter, "long array", 8, 24),
        _ => Err(NbtError::InvalidTagType { tag_id }),
    }
}

fn scan_array(
    reader: &mut WireReader<'_>,
    accounter: &mut NbtAccounter,
    kind: &'static str,
    element_bytes: usize,
    base_usage: u64,
) -> Result<(), NbtError> {
    let count = read_nonnegative_i32(reader, kind)?;
    accounter.account(base_usage)?;
    accounter.account_entries(element_bytes as u64, count)?;
    let byte_length = element_bytes
        .checked_mul(count)
        .ok_or(NbtError::QuotaExceeded {
            quota: accounter.quota,
        })?;
    reader.read_bytes(byte_length, kind)?;
    Ok(())
}

fn scan_list(reader: &mut WireReader<'_>, accounter: &mut NbtAccounter) -> Result<(), NbtError> {
    accounter.push()?;
    let result = (|| {
        let element_tag_id = reader.read_u8()?;
        let count = read_nonnegative_i32(reader, "list")?;
        if element_tag_id == 0 && count != 0 {
            return Err(NbtError::NonemptyEndList { length: count });
        }
        if element_tag_id > 12 {
            return Err(NbtError::InvalidTagType {
                tag_id: element_tag_id,
            });
        }
        accounter.account(36)?;
        accounter.account_entries(4, count)?;
        for _ in 0..count {
            scan_payload(reader, element_tag_id, accounter)?;
        }
        Ok(())
    })();
    accounter.pop();
    result
}

fn scan_compound(
    reader: &mut WireReader<'_>,
    accounter: &mut NbtAccounter,
) -> Result<(), NbtError> {
    accounter.push()?;
    let result = (|| {
        accounter.account(48)?;
        let mut keys = BTreeSet::new();
        loop {
            let tag_id = reader.read_u8()?;
            if tag_id == 0 {
                break;
            }
            if tag_id > 12 {
                return Err(NbtError::InvalidTagType { tag_id });
            }
            let key_units = scan_modified_utf(reader)?;
            accounter.account(28)?;
            accounter.account_entries(2, key_units.len())?;
            scan_payload(reader, tag_id, accounter)?;
            if keys.insert(key_units) {
                accounter.account(36)?;
            }
        }
        Ok(())
    })();
    accounter.pop();
    result
}

fn read_nonnegative_i32(
    reader: &mut WireReader<'_>,
    kind: &'static str,
) -> Result<usize, NbtError> {
    let value = reader.read_i32()?;
    usize::try_from(value).map_err(|_| NbtError::NegativeLength {
        kind,
        length: value,
    })
}

fn scan_modified_utf(reader: &mut WireReader<'_>) -> Result<Vec<u16>, NbtError> {
    let length = usize::from(reader.read_u16()?);
    let bytes = reader.read_bytes(length, "NBT modified UTF")?;
    let mut offset = 0;
    let mut units = Vec::new();
    while offset < bytes.len() {
        let first = bytes[offset];
        let width = match first {
            0x00..=0x7f => 1,
            0xc0..=0xdf => 2,
            0xe0..=0xef => 3,
            _ => return Err(NbtError::InvalidModifiedUtf),
        };
        if offset + width > bytes.len()
            || bytes[offset + 1..offset + width]
                .iter()
                .any(|byte| byte & 0xc0 != 0x80)
        {
            return Err(NbtError::InvalidModifiedUtf);
        }
        let unit = match width {
            1 => u16::from(first),
            2 => (u16::from(first & 0x1f) << 6) | u16::from(bytes[offset + 1] & 0x3f),
            3 => {
                (u16::from(first & 0x0f) << 12)
                    | (u16::from(bytes[offset + 1] & 0x3f) << 6)
                    | u16::from(bytes[offset + 2] & 0x3f)
            }
            _ => unreachable!(),
        };
        units.push(unit);
        offset += width;
    }
    Ok(units)
}

fn component_root_shape_is_valid(value: &NetworkNbt) -> bool {
    match value.root_tag_id() {
        8 => true,
        9 => {
            let bytes = value.as_bytes();
            let element_id = bytes.get(1).copied();
            let count = bytes
                .get(2..6)
                .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
                .map(i32::from_be_bytes);
            element_id.is_some_and(|id| matches!(id, 8..=10))
                && count.is_some_and(|count| count > 0)
        }
        10 => value.as_bytes().get(1).is_some_and(|tag_id| *tag_id != 0),
        _ => false,
    }
}

fn encode_modified_utf(value: &str) -> Result<Vec<u8>, NbtError> {
    let mut encoded = Vec::new();
    for unit in value.encode_utf16() {
        match unit {
            0 => encoded.extend_from_slice(&[0xc0, 0x80]),
            1..=0x7f => encoded.push(unit as u8),
            0x80..=0x7ff => {
                encoded.push((0xc0 | (unit >> 6)) as u8);
                encoded.push((0x80 | (unit & 0x3f)) as u8);
            }
            _ => {
                encoded.push((0xe0 | (unit >> 12)) as u8);
                encoded.push((0x80 | ((unit >> 6) & 0x3f)) as u8);
                encoded.push((0x80 | (unit & 0x3f)) as u8);
            }
        }
    }
    if encoded.len() > usize::from(u16::MAX) {
        Err(NbtError::ModifiedUtfTooLong {
            length: encoded.len(),
        })
    } else {
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use crate::java_26_2::value::nbt::{
        DEFAULT_NBT_QUOTA, MAX_NBT_DEPTH, NbtError, NbtQuota, NetworkNbt, TextComponentNbt,
    };

    #[test]
    fn literal_component_uses_modified_utf_and_round_trips() {
        let value = NetworkNbt::literal_component("a\0😀").unwrap();
        assert_eq!(value.root_tag_id(), 8);
        assert_eq!(
            value.as_bytes(),
            &[
                8, 0, 9, b'a', 0xc0, 0x80, 0xed, 0xa0, 0xbd, 0xed, 0xb8, 0x80
            ]
        );
        assert_eq!(
            NetworkNbt::from_bytes(value.as_bytes().to_vec(), NbtQuota::Trusted).unwrap(),
            value
        );
    }

    #[test]
    fn rejects_null_malformed_and_trailing_tags() {
        assert_eq!(
            NetworkNbt::from_bytes(vec![0], NbtQuota::Trusted),
            Err(NbtError::NullTag)
        );
        assert!(matches!(
            NetworkNbt::from_bytes(vec![8, 0, 1, 0xff], NbtQuota::Trusted),
            Err(NbtError::InvalidModifiedUtf)
        ));
        assert!(NetworkNbt::from_bytes(vec![1, 0, 0], NbtQuota::Trusted).is_err());
    }

    #[test]
    fn component_wrapper_rejects_non_component_root_shapes() {
        let scalar = NetworkNbt::from_bytes(vec![1, 0], NbtQuota::Trusted).unwrap();
        assert!(TextComponentNbt::from_network_nbt(scalar).is_err());
        let empty_list = NetworkNbt::from_bytes(vec![9, 0, 0, 0, 0, 0], NbtQuota::Trusted).unwrap();
        assert!(TextComponentNbt::from_network_nbt(empty_list).is_err());
        let empty_compound = NetworkNbt::from_bytes(vec![10, 0], NbtQuota::Trusted).unwrap();
        assert!(TextComponentNbt::from_network_nbt(empty_compound).is_err());
    }

    #[test]
    fn enforces_the_default_accumulator_quota() {
        let count = (DEFAULT_NBT_QUOTA - 24 + 1) as i32;
        let mut bytes = vec![7];
        bytes.extend_from_slice(&count.to_be_bytes());
        assert_eq!(
            NetworkNbt::from_bytes(bytes, NbtQuota::Default),
            Err(NbtError::QuotaExceeded {
                quota: DEFAULT_NBT_QUOTA
            })
        );
    }

    #[test]
    fn enforces_locked_container_depth() {
        let mut allowed = vec![9, 9, 0, 0, 0, 1];
        for _ in 2..MAX_NBT_DEPTH {
            allowed.extend_from_slice(&[9, 0, 0, 0, 1]);
        }
        allowed.extend_from_slice(&[0, 0, 0, 0, 0]);
        let result = NetworkNbt::from_bytes(allowed.clone(), NbtQuota::Trusted);
        assert!(result.is_ok(), "{result:?}");

        let mut denied = vec![9, 9, 0, 0, 0, 1];
        for _ in 1..MAX_NBT_DEPTH {
            denied.extend_from_slice(&[9, 0, 0, 0, 1]);
        }
        denied.extend_from_slice(&[0, 0, 0, 0, 0]);
        assert_eq!(
            NetworkNbt::from_bytes(denied, NbtQuota::Trusted),
            Err(NbtError::DepthExceeded {
                maximum: MAX_NBT_DEPTH
            })
        );
    }
}
