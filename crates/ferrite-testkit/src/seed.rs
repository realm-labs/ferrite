//! Stable named seeds for tests and authored scenarios.

use ferrite_foundation::resource::ResourceId;

const TEST_SEED_DOMAIN: &[u8] = b"ferrite:test-seed:v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestSeed(u64);

impl TestSeed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn derive(self, name: &ResourceId) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(TEST_SEED_DOMAIN);
        hasher.update(&self.0.to_le_bytes());
        hasher.update(name.to_string().as_bytes());
        let mut bytes = [0; 8];
        bytes.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
        Self(u64::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_derivation_has_a_locked_vector() {
        let name = ResourceId::new("ferrite", "scenario/spawn").unwrap();
        assert_eq!(
            TestSeed::new(42).derive(&name).get(),
            4_561_292_695_296_433_824
        );
        assert_ne!(
            TestSeed::new(42).derive(&name),
            TestSeed::new(43).derive(&name)
        );
    }
}
