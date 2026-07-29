//! Dense runtime-ID palette containers for section data.

mod packed;

use packed::PackedStorage;
use thiserror::Error;

pub const LOCAL_PALETTE_MAX_ENTRIES: usize = 256;
const DIRECT_MINIMUM_BITS: u8 = 9;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PalettedContainer<T: PaletteEntry, const N: usize> {
    mode: PaletteMode<T, N>,
}

impl<T: PaletteEntry, const N: usize> PalettedContainer<T, N> {
    pub fn new(value: T) -> Self {
        assert!(N > 0, "palette containers cannot be empty");
        Self {
            mode: PaletteMode::Single(value),
        }
    }

    pub const fn len(&self) -> usize {
        N
    }

    pub const fn is_empty(&self) -> bool {
        N == 0
    }

    pub fn encoding(&self) -> PaletteEncoding {
        match &self.mode {
            PaletteMode::Single(_) => PaletteEncoding::Single,
            PaletteMode::Local { palette, packed } => PaletteEncoding::Local {
                palette_len: palette.len(),
                bits_per_entry: packed.bits_per_entry(),
            },
            PaletteMode::Direct { packed } => PaletteEncoding::Direct {
                bits_per_entry: packed.bits_per_entry(),
            },
        }
    }

    pub fn get(&self, index: usize) -> Result<T, PaletteError> {
        check_index::<N>(index)?;
        Ok(self.get_unchecked(index))
    }

    pub fn set(&mut self, index: usize, value: T) -> Result<T, PaletteError> {
        check_index::<N>(index)?;
        let previous = self.get_unchecked(index);
        if previous == value {
            return Ok(previous);
        }

        match &mut self.mode {
            PaletteMode::Single(single) => {
                let mut packed = PackedStorage::zeroed(1);
                packed.set(index, 1);
                self.mode = PaletteMode::Local {
                    palette: vec![*single, value],
                    packed,
                };
            }
            PaletteMode::Local { palette, packed } => {
                if let Some(palette_index) = palette.iter().position(|entry| *entry == value) {
                    packed.set(index, palette_index as u32);
                } else if palette.len() < LOCAL_PALETTE_MAX_ENTRIES {
                    let palette_index = palette.len();
                    palette.push(value);
                    let required_bits = bits_for(palette_index as u32);
                    if required_bits > packed.bits_per_entry() {
                        *packed = packed.repack(required_bits);
                    }
                    packed.set(index, palette_index as u32);
                } else {
                    let values = (0..N)
                        .map(|entry_index| palette[packed.get(entry_index) as usize])
                        .collect::<Vec<_>>();
                    self.mode = direct_from_values(values, index, value);
                }
            }
            PaletteMode::Direct { packed } => {
                let required_bits = bits_for(value.to_raw()).max(DIRECT_MINIMUM_BITS);
                if required_bits > packed.bits_per_entry() {
                    *packed = packed.repack(required_bits);
                }
                packed.set(index, value.to_raw());
            }
        }
        Ok(previous)
    }

    pub fn values(&self) -> impl ExactSizeIterator<Item = T> + '_ {
        (0..N).map(|index| self.get_unchecked(index))
    }

    fn get_unchecked(&self, index: usize) -> T {
        match &self.mode {
            PaletteMode::Single(value) => *value,
            PaletteMode::Local { palette, packed } => palette[packed.get(index) as usize],
            PaletteMode::Direct { packed } => T::from_raw(packed.get(index)),
        }
    }
}

fn direct_from_values<T: PaletteEntry, const N: usize>(
    mut values: Vec<T>,
    changed_index: usize,
    changed_value: T,
) -> PaletteMode<T, N> {
    values[changed_index] = changed_value;
    let maximum = values
        .iter()
        .map(|value| value.to_raw())
        .max()
        .unwrap_or_default();
    let mut packed = PackedStorage::zeroed(bits_for(maximum).max(DIRECT_MINIMUM_BITS));
    for (index, value) in values.into_iter().enumerate() {
        packed.set(index, value.to_raw());
    }
    PaletteMode::Direct { packed }
}

const fn bits_for(value: u32) -> u8 {
    let bits = u32::BITS - value.leading_zeros();
    if bits == 0 { 1 } else { bits as u8 }
}

fn check_index<const N: usize>(index: usize) -> Result<(), PaletteError> {
    if index >= N {
        return Err(PaletteError::IndexOutOfBounds { index, length: N });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PaletteMode<T: PaletteEntry, const N: usize> {
    Single(T),
    Local {
        palette: Vec<T>,
        packed: PackedStorage<N>,
    },
    Direct {
        packed: PackedStorage<N>,
    },
}

pub trait PaletteEntry: Copy + Eq {
    fn to_raw(self) -> u32;
    fn from_raw(value: u32) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteEncoding {
    Single,
    Local {
        palette_len: usize,
        bits_per_entry: u8,
    },
    Direct {
        bits_per_entry: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PaletteError {
    #[error("palette index {index} is outside container length {length}")]
    IndexOutOfBounds { index: usize, length: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::BlockStateId;

    #[test]
    fn transitions_from_single_to_local_and_expands_bits() {
        let mut container = PalettedContainer::<BlockStateId, 16>::new(BlockStateId::new(4));
        assert_eq!(container.encoding(), PaletteEncoding::Single);
        for index in 0..8 {
            assert_eq!(
                container.set(index, BlockStateId::new(index as u32 + 10)),
                Ok(BlockStateId::new(4))
            );
        }
        assert_eq!(
            container.encoding(),
            PaletteEncoding::Local {
                palette_len: 9,
                bits_per_entry: 4,
            }
        );
        assert_eq!(container.get(7), Ok(BlockStateId::new(17)));
        assert_eq!(container.get(15), Ok(BlockStateId::new(4)));
    }

    #[test]
    fn local_palette_promotes_to_direct_runtime_ids() {
        let mut container = PalettedContainer::<BlockStateId, 300>::new(BlockStateId::new(0));
        for index in 0..256 {
            container
                .set(index, BlockStateId::new(index as u32 + 1))
                .unwrap();
        }
        assert_eq!(
            container.encoding(),
            PaletteEncoding::Direct {
                bits_per_entry: DIRECT_MINIMUM_BITS,
            }
        );
        assert_eq!(container.get(0), Ok(BlockStateId::new(1)));
        assert_eq!(container.get(255), Ok(BlockStateId::new(256)));
        assert_eq!(container.get(299), Ok(BlockStateId::new(0)));
    }

    #[test]
    fn direct_storage_expands_for_large_runtime_ids_and_checks_bounds() {
        let mut container = PalettedContainer::<BlockStateId, 300>::new(BlockStateId::new(0));
        for index in 0..256 {
            container
                .set(index, BlockStateId::new(index as u32 + 1))
                .unwrap();
        }
        container.set(299, BlockStateId::new(1 << 20)).unwrap();
        assert_eq!(
            container.encoding(),
            PaletteEncoding::Direct { bits_per_entry: 21 }
        );
        assert_eq!(container.get(299), Ok(BlockStateId::new(1 << 20)));
        assert!(container.get(300).is_err());
        assert!(container.set(300, BlockStateId::new(1)).is_err());
    }
}
