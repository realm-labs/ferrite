//! Minimal random-source contract shared by generation algorithms.

use std::num::NonZeroU32;

pub trait GenerationRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32;

    fn next_i32(&mut self) -> i32 {
        let word = NonZeroU32::new(1 << 16).expect("65536 is nonzero");
        let high = self.next_u32(word);
        let low = self.next_u32(word);
        (high << 16 | low) as i32
    }

    fn next_f32(&mut self) -> f32;

    fn next_f64(&mut self) -> f64;

    fn next_gaussian(&mut self) -> f64;

    fn next_bool(&mut self) -> bool {
        self.next_u32(NonZeroU32::new(2).expect("two is nonzero")) != 0
    }
}

#[derive(Debug, Clone)]
pub struct LegacyRandom {
    seed: u64,
    gaussian: Option<f64>,
}

impl LegacyRandom {
    const MULTIPLIER: u64 = 0x5deece66d;
    const ADDEND: u64 = 0xb;
    const MASK: u64 = (1_u64 << 48) - 1;

    pub fn new(seed: i64) -> Self {
        let mut random = Self {
            seed: 0,
            gaussian: None,
        };
        random.set_seed(seed);
        random
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed as u64 ^ Self::MULTIPLIER) & Self::MASK;
        self.gaussian = None;
    }

    pub fn set_large_feature_seed(&mut self, seed: i64, chunk_x: i32, chunk_z: i32) {
        self.set_seed(seed);
        let x_multiplier = self.next_i64();
        let z_multiplier = self.next_i64();
        let mixed = i64::from(chunk_x).wrapping_mul(x_multiplier)
            ^ i64::from(chunk_z).wrapping_mul(z_multiplier)
            ^ seed;
        self.set_seed(mixed);
    }

    pub fn next_i64(&mut self) -> i64 {
        (i64::from(self.next_bits(32)) << 32).wrapping_add(i64::from(self.next_bits(32)))
    }

    pub fn next_i32(&mut self) -> i32 {
        self.next_bits(32)
    }

    fn next_bits(&mut self, bits: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(Self::ADDEND)
            & Self::MASK;
        (self.seed >> (48 - bits)) as u32 as i32
    }

    fn next_bounded(&mut self, bound: i32) -> i32 {
        debug_assert!(bound > 0);
        if bound & -bound == bound {
            return ((i64::from(bound) * i64::from(self.next_bits(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next_bits(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }
}

impl GenerationRandom for LegacyRandom {
    fn next_u32(&mut self, bound: NonZeroU32) -> u32 {
        let bound = i32::try_from(bound.get()).expect("legacy nextInt bound fits positive i32");
        self.next_bounded(bound) as u32
    }

    fn next_i32(&mut self) -> i32 {
        LegacyRandom::next_i32(self)
    }

    fn next_f32(&mut self) -> f32 {
        self.next_bits(24) as f32 / (1_u32 << 24) as f32
    }

    fn next_f64(&mut self) -> f64 {
        let high = i64::from(self.next_bits(26)) << 27;
        let low = i64::from(self.next_bits(27));
        (high + low) as f64 / (1_u64 << 53) as f64
    }

    fn next_gaussian(&mut self) -> f64 {
        if let Some(value) = self.gaussian.take() {
            return value;
        }
        loop {
            let left = 2.0 * self.next_f64() - 1.0;
            let right = 2.0 * self.next_f64() - 1.0;
            let radius = left * left + right * right;
            if radius >= 1.0 || radius == 0.0 {
                continue;
            }
            let multiplier = (-2.0 * radius.ln() / radius).sqrt();
            self.gaussian = Some(right * multiplier);
            return left * multiplier;
        }
    }
}
