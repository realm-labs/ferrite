#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PackedStorage<const N: usize> {
    bits_per_entry: u8,
    words: Vec<u64>,
}

impl<const N: usize> PackedStorage<N> {
    pub(crate) fn zeroed(bits_per_entry: u8) -> Self {
        debug_assert!((1..=32).contains(&bits_per_entry));
        let bit_count = N
            .checked_mul(usize::from(bits_per_entry))
            .expect("palette bit storage size must fit usize");
        Self {
            bits_per_entry,
            words: vec![0; bit_count.div_ceil(u64::BITS as usize)],
        }
    }

    pub(crate) const fn bits_per_entry(&self) -> u8 {
        self.bits_per_entry
    }

    pub(crate) fn get(&self, index: usize) -> u32 {
        debug_assert!(index < N);
        let bits = usize::from(self.bits_per_entry);
        let bit_index = index * bits;
        let word_index = bit_index / u64::BITS as usize;
        let shift = bit_index % u64::BITS as usize;
        let mask = value_mask(self.bits_per_entry);
        if shift + bits <= u64::BITS as usize {
            ((self.words[word_index] >> shift) & mask) as u32
        } else {
            let lower_bits = u64::BITS as usize - shift;
            let upper_bits = bits - lower_bits;
            let lower = self.words[word_index] >> shift;
            let upper = self.words[word_index + 1] & ((1_u64 << upper_bits) - 1);
            ((lower | (upper << lower_bits)) & mask) as u32
        }
    }

    pub(crate) fn set(&mut self, index: usize, value: u32) {
        debug_assert!(index < N);
        debug_assert!(u64::from(value) <= value_mask(self.bits_per_entry));
        let bits = usize::from(self.bits_per_entry);
        let bit_index = index * bits;
        let word_index = bit_index / u64::BITS as usize;
        let shift = bit_index % u64::BITS as usize;
        let mask = value_mask(self.bits_per_entry);
        let encoded = u64::from(value);
        self.words[word_index] = (self.words[word_index] & !(mask << shift)) | (encoded << shift);
        if shift + bits > u64::BITS as usize {
            let lower_bits = u64::BITS as usize - shift;
            let upper_bits = bits - lower_bits;
            let upper_mask = (1_u64 << upper_bits) - 1;
            self.words[word_index + 1] =
                (self.words[word_index + 1] & !upper_mask) | ((encoded >> lower_bits) & upper_mask);
        }
    }

    pub(crate) fn repack(&self, bits_per_entry: u8) -> Self {
        let mut replacement = Self::zeroed(bits_per_entry);
        for index in 0..N {
            replacement.set(index, self.get(index));
        }
        replacement
    }
}

const fn value_mask(bits_per_entry: u8) -> u64 {
    (1_u64 << bits_per_entry) - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entries_that_cross_word_boundaries_round_trip() {
        let mut storage = PackedStorage::<65>::zeroed(5);
        for index in 0..65 {
            storage.set(index, (index % 31) as u32);
        }
        for index in 0..65 {
            assert_eq!(storage.get(index), (index % 31) as u32);
        }
        let expanded = storage.repack(8);
        for index in 0..65 {
            assert_eq!(expanded.get(index), (index % 31) as u32);
        }
    }
}
