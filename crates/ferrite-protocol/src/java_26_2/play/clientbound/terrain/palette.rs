use thiserror::Error;

use crate::java_26_2::wire::error::WireError;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

const BLOCK_ENTRY_COUNT: usize = 4_096;
const BIOME_ENTRY_COUNT: usize = 64;
const BLOCK_STATE_COUNT: i32 = 32_366;
const GLOBAL_BLOCK_BITS: u8 = 15;

#[derive(Debug, Clone, Copy)]
pub enum PaletteKind {
    Blocks,
    Biomes { registry_size: usize },
}

impl PaletteKind {
    pub const fn entry_count(self) -> usize {
        match self {
            Self::Blocks => BLOCK_ENTRY_COUNT,
            Self::Biomes { .. } => BIOME_ENTRY_COUNT,
        }
    }

    fn canonical(self, unique_count: usize) -> PaletteConfiguration {
        if unique_count == 1 {
            return PaletteConfiguration::Single;
        }
        match self {
            Self::Blocks if unique_count <= 256 => PaletteConfiguration::Local {
                bits: bits_for(unique_count.saturating_sub(1) as u32).max(4),
            },
            Self::Blocks => PaletteConfiguration::Global {
                bits: GLOBAL_BLOCK_BITS,
            },
            Self::Biomes { .. } if unique_count <= 8 => PaletteConfiguration::Local {
                bits: bits_for(unique_count.saturating_sub(1) as u32),
            },
            Self::Biomes { .. } => PaletteConfiguration::Global {
                bits: self.global_bits(),
            },
        }
    }

    fn decode_configuration(self, selector: i8) -> PaletteConfiguration {
        match self {
            Self::Blocks => match selector {
                0 => PaletteConfiguration::Single,
                1..=4 => PaletteConfiguration::Local { bits: 4 },
                5..=8 => PaletteConfiguration::Local {
                    bits: selector as u8,
                },
                _ => PaletteConfiguration::Global {
                    bits: GLOBAL_BLOCK_BITS,
                },
            },
            Self::Biomes { .. } => match selector {
                0 => PaletteConfiguration::Single,
                1..=3 => PaletteConfiguration::Local {
                    bits: selector as u8,
                },
                _ => PaletteConfiguration::Global {
                    bits: self.global_bits(),
                },
            },
        }
    }

    fn global_bits(self) -> u8 {
        match self {
            Self::Blocks => GLOBAL_BLOCK_BITS,
            Self::Biomes { registry_size } => {
                bits_for(registry_size.saturating_sub(1) as u32).max(1)
            }
        }
    }

    fn validate(self, value: i32) -> Result<(), PaletteCodecError> {
        let valid = match self {
            Self::Blocks => (0..BLOCK_STATE_COUNT).contains(&value),
            Self::Biomes { registry_size } => {
                usize::try_from(value).is_ok_and(|index| index < registry_size)
            }
        };
        if valid {
            Ok(())
        } else {
            Err(PaletteCodecError::UnknownRegistryValue { value })
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PaletteConfiguration {
    Single,
    Local { bits: u8 },
    Global { bits: u8 },
}

pub fn write(
    writer: &mut WireWriter,
    values: &[i32],
    kind: PaletteKind,
) -> Result<(), PaletteCodecError> {
    if values.len() != kind.entry_count() {
        return Err(PaletteCodecError::EntryCount {
            expected: kind.entry_count(),
            actual: values.len(),
        });
    }
    for value in values {
        kind.validate(*value)?;
    }
    let palette = unique_values(values);
    match kind.canonical(palette.len()) {
        PaletteConfiguration::Single => {
            writer.write_i8(0)?;
            writer.write_var_i32(palette[0])?;
            writer.write_var_i32(0)?;
        }
        PaletteConfiguration::Local { bits } => {
            writer.write_i8(bits as i8)?;
            writer.write_count("palette entries", palette.len(), values.len())?;
            for value in &palette {
                writer.write_var_i32(*value)?;
            }
            let indices = values
                .iter()
                .map(|value| {
                    palette
                        .iter()
                        .position(|entry| entry == value)
                        .expect("unique palette contains every value") as u32
                })
                .collect::<Vec<_>>();
            write_storage(writer, bits, &indices)?;
        }
        PaletteConfiguration::Global { bits } => {
            writer.write_i8(bits as i8)?;
            let raw = values.iter().map(|value| *value as u32).collect::<Vec<_>>();
            write_storage(writer, bits, &raw)?;
        }
    }
    Ok(())
}

pub fn read(reader: &mut WireReader<'_>, kind: PaletteKind) -> Result<Vec<i32>, PaletteCodecError> {
    let configuration = kind.decode_configuration(reader.read_i8()?);
    match configuration {
        PaletteConfiguration::Single => {
            let value = reader.read_var_i32()?;
            kind.validate(value)?;
            require_storage_length(reader, 0)?;
            Ok(vec![value; kind.entry_count()])
        }
        PaletteConfiguration::Local { bits } => {
            let count = reader.read_count("palette entries", kind.entry_count())?;
            let mut palette = Vec::with_capacity(count);
            for _ in 0..count {
                let value = reader.read_var_i32()?;
                kind.validate(value)?;
                palette.push(value);
            }
            let indices = read_storage(reader, bits, kind.entry_count())?;
            indices
                .into_iter()
                .map(|index| {
                    palette.get(index as usize).copied().ok_or(
                        PaletteCodecError::MissingPaletteEntry {
                            index,
                            entries: palette.len(),
                        },
                    )
                })
                .collect()
        }
        PaletteConfiguration::Global { bits } => {
            let values = read_storage(reader, bits, kind.entry_count())?
                .into_iter()
                .map(|value| value as i32)
                .collect::<Vec<_>>();
            for value in &values {
                kind.validate(*value)?;
            }
            Ok(values)
        }
    }
}

fn unique_values(values: &[i32]) -> Vec<i32> {
    let mut unique = Vec::new();
    for value in values {
        if !unique.contains(value) {
            unique.push(*value);
        }
    }
    unique
}

fn write_storage(
    writer: &mut WireWriter,
    bits: u8,
    values: &[u32],
) -> Result<(), PaletteCodecError> {
    let per_long = 64 / usize::from(bits);
    let long_count = values.len().div_ceil(per_long);
    writer.write_count("palette storage longs", long_count, values.len())?;
    let mask = (1u64 << bits) - 1;
    for chunk in values.chunks(per_long) {
        let mut packed = 0u64;
        for (index, value) in chunk.iter().enumerate() {
            packed |= (u64::from(*value) & mask) << (index * usize::from(bits));
        }
        writer.write_i64(packed as i64)?;
    }
    Ok(())
}

fn read_storage(
    reader: &mut WireReader<'_>,
    bits: u8,
    entry_count: usize,
) -> Result<Vec<u32>, PaletteCodecError> {
    let per_long = 64 / usize::from(bits);
    let expected = entry_count.div_ceil(per_long);
    require_storage_length(reader, expected)?;
    let mask = (1u64 << bits) - 1;
    let mut values = Vec::with_capacity(entry_count);
    for _ in 0..expected {
        let packed = reader.read_i64()? as u64;
        for index in 0..per_long {
            if values.len() == entry_count {
                break;
            }
            values.push(((packed >> (index * usize::from(bits))) & mask) as u32);
        }
    }
    Ok(values)
}

fn require_storage_length(
    reader: &mut WireReader<'_>,
    expected: usize,
) -> Result<(), PaletteCodecError> {
    let actual = reader.read_count("palette storage longs", reader.remaining() / 8)?;
    if actual == expected {
        Ok(())
    } else {
        Err(PaletteCodecError::StorageLength { expected, actual })
    }
}

const fn bits_for(value: u32) -> u8 {
    let bits = u32::BITS - value.leading_zeros();
    if bits == 0 { 1 } else { bits as u8 }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PaletteCodecError {
    #[error(transparent)]
    Wire(#[from] WireError),
    #[error("palette has {actual} values, expected {expected}")]
    EntryCount { expected: usize, actual: usize },
    #[error("palette registry value {value} is absent")]
    UnknownRegistryValue { value: i32 },
    #[error("palette storage has {actual} longs, expected {expected}")]
    StorageLength { expected: usize, actual: usize },
    #[error("palette index {index} is absent from {entries} entries")]
    MissingPaletteEntry { index: u32, entries: usize },
}
