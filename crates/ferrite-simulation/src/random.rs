//! Versioned named random streams whose creation order does not affect output.

use ferrite_foundation::resource::ResourceId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU64;
use thiserror::Error;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const SPLITMIX_INCREMENT: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RandomAlgorithm {
    Xoshiro256StarStarV1,
}

impl RandomAlgorithm {
    pub const fn stable_tag(self) -> u16 {
        match self {
            Self::Xoshiro256StarStarV1 => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WorldSeed(u64);

impl WorldSeed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "[u64; 4]", into = "[u64; 4]")]
pub struct RandomState([u64; 4]);

impl RandomState {
    pub const fn new(words: [u64; 4]) -> Result<Self, RandomError> {
        if words[0] == 0 && words[1] == 0 && words[2] == 0 && words[3] == 0 {
            return Err(RandomError::ZeroState);
        }
        Ok(Self(words))
    }

    pub const fn words(self) -> [u64; 4] {
        self.0
    }
}

impl TryFrom<[u64; 4]> for RandomState {
    type Error = RandomError;

    fn try_from(value: [u64; 4]) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<RandomState> for [u64; 4] {
    fn from(value: RandomState) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeterministicRng {
    algorithm: RandomAlgorithm,
    state: RandomState,
}

impl DeterministicRng {
    pub fn from_seed(seed: u64) -> Self {
        let mut splitmix = seed;
        let words = std::array::from_fn(|_| splitmix64_next(&mut splitmix));
        Self {
            algorithm: RandomAlgorithm::Xoshiro256StarStarV1,
            state: RandomState::new(words).expect("SplitMix64 cannot produce four zero words"),
        }
    }

    pub const fn from_state(algorithm: RandomAlgorithm, state: RandomState) -> Self {
        Self { algorithm, state }
    }

    pub const fn algorithm(&self) -> RandomAlgorithm {
        self.algorithm
    }

    pub const fn state(&self) -> RandomState {
        self.state
    }

    pub fn next_u64(&mut self) -> u64 {
        match self.algorithm {
            RandomAlgorithm::Xoshiro256StarStarV1 => self.next_xoshiro256_star_star(),
        }
    }

    pub fn uniform_u64(&mut self, upper_exclusive: NonZeroU64) -> u64 {
        let upper = upper_exclusive.get();
        let threshold = upper.wrapping_neg() % upper;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return value % upper;
            }
        }
    }

    pub fn choose_index(&mut self, length: usize) -> Result<usize, RandomError> {
        let upper = u64::try_from(length).map_err(|_| RandomError::SelectionTooLarge { length })?;
        let upper = NonZeroU64::new(upper).ok_or(RandomError::EmptySelection)?;
        let selected = self.uniform_u64(upper);
        usize::try_from(selected).map_err(|_| RandomError::SelectionTooLarge { length })
    }

    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Result<&'a T, RandomError> {
        let index = self.choose_index(values.len())?;
        Ok(&values[index])
    }

    pub fn chance(&mut self, numerator: u64, denominator: u64) -> Result<bool, RandomError> {
        if denominator == 0 || numerator > denominator {
            return Err(RandomError::InvalidProbability {
                numerator,
                denominator,
            });
        }
        let denominator = NonZeroU64::new(denominator).ok_or(RandomError::InvalidProbability {
            numerator,
            denominator,
        })?;
        Ok(self.uniform_u64(denominator) < numerator)
    }

    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        for upper in (2..=values.len()).rev() {
            let upper = NonZeroU64::new(upper as u64).expect("shuffle upper bound is positive");
            let selected = self.uniform_u64(upper) as usize;
            values.swap(upper.get() as usize - 1, selected);
        }
    }

    fn next_xoshiro256_star_star(&mut self) -> u64 {
        let [mut s0, mut s1, mut s2, mut s3] = self.state.words();
        let result = s1.wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let temporary = s1 << 17;

        s2 ^= s0;
        s3 ^= s1;
        s1 ^= s2;
        s0 ^= s3;
        s2 ^= temporary;
        s3 = s3.rotate_left(45);
        self.state = RandomState([s0, s1, s2, s3]);
        result
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedRandomStreams {
    world_seed: WorldSeed,
    algorithm: RandomAlgorithm,
    streams: BTreeMap<ResourceId, DeterministicRng>,
}

impl NamedRandomStreams {
    pub const fn new(world_seed: WorldSeed) -> Self {
        Self {
            world_seed,
            algorithm: RandomAlgorithm::Xoshiro256StarStarV1,
            streams: BTreeMap::new(),
        }
    }

    pub const fn world_seed(&self) -> WorldSeed {
        self.world_seed
    }

    pub const fn algorithm(&self) -> RandomAlgorithm {
        self.algorithm
    }

    pub fn stream_mut(&mut self, name: &ResourceId) -> &mut DeterministicRng {
        let seed = derive_stream_seed(self.world_seed, name);
        self.streams
            .entry(name.clone())
            .or_insert_with(|| DeterministicRng::from_seed(seed))
    }

    pub fn snapshot(&self) -> RandomStreamsSnapshot {
        RandomStreamsSnapshot {
            world_seed: self.world_seed,
            algorithm: self.algorithm,
            streams: self
                .streams
                .iter()
                .map(|(name, stream)| RandomStreamSnapshot {
                    name: name.clone(),
                    state: stream.state(),
                })
                .collect(),
        }
    }

    pub fn restore(snapshot: RandomStreamsSnapshot) -> Result<Self, RandomError> {
        let mut names = BTreeSet::new();
        let mut streams = BTreeMap::new();
        for stream in snapshot.streams {
            if !names.insert(stream.name.clone()) {
                return Err(RandomError::DuplicateStream { name: stream.name });
            }
            streams.insert(
                stream.name,
                DeterministicRng::from_state(snapshot.algorithm, stream.state),
            );
        }
        Ok(Self {
            world_seed: snapshot.world_seed,
            algorithm: snapshot.algorithm,
            streams,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomStreamSnapshot {
    name: ResourceId,
    state: RandomState,
}

impl RandomStreamSnapshot {
    pub const fn name(&self) -> &ResourceId {
        &self.name
    }

    pub const fn state(&self) -> RandomState {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RandomStreamsSnapshot {
    world_seed: WorldSeed,
    algorithm: RandomAlgorithm,
    streams: Vec<RandomStreamSnapshot>,
}

impl RandomStreamsSnapshot {
    pub const fn world_seed(&self) -> WorldSeed {
        self.world_seed
    }

    pub const fn algorithm(&self) -> RandomAlgorithm {
        self.algorithm
    }

    pub fn streams(&self) -> impl ExactSizeIterator<Item = &RandomStreamSnapshot> {
        self.streams.iter()
    }
}

fn derive_stream_seed(world_seed: WorldSeed, name: &ResourceId) -> u64 {
    let mut hash = FNV_OFFSET;
    for byte in world_seed.get().to_le_bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
    }
    for byte in name.to_string().bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME);
    }
    mix64(hash)
}

fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(SPLITMIX_INCREMENT);
    mix64(*state)
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RandomError {
    #[error("random generator state cannot be all zero")]
    ZeroState,
    #[error("cannot select from an empty collection")]
    EmptySelection,
    #[error("selection length {length} cannot be represented")]
    SelectionTooLarge { length: usize },
    #[error("invalid probability {numerator}/{denominator}")]
    InvalidProbability { numerator: u64, denominator: u64 },
    #[error("random stream {name} occurs more than once in a snapshot")]
    DuplicateStream { name: ResourceId },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(name: &str) -> ResourceId {
        ResourceId::new("ferrite", name).unwrap()
    }

    #[test]
    fn stream_creation_order_does_not_change_output() {
        let mut first = NamedRandomStreams::new(WorldSeed::new(42));
        let weather_first = first.stream_mut(&stream("weather")).next_u64();
        let loot_first = first.stream_mut(&stream("loot")).next_u64();

        let mut second = NamedRandomStreams::new(WorldSeed::new(42));
        let loot_second = second.stream_mut(&stream("loot")).next_u64();
        let weather_second = second.stream_mut(&stream("weather")).next_u64();
        assert_eq!(weather_first, weather_second);
        assert_eq!(loot_first, loot_second);
        assert_ne!(weather_first, loot_first);
    }

    #[test]
    fn xoshiro_v1_has_a_locked_cross_platform_vector() {
        let mut random = DeterministicRng::from_seed(0);
        assert_eq!(
            std::array::from_fn::<_, 5, _>(|_| random.next_u64()),
            [
                0x99ec_5f36_cb75_f2b4,
                0xbf6e_1f78_4956_452a,
                0x1a5f_849d_4933_e6e0,
                0x6aa5_94f1_262d_2d2c,
                0xbba5_ad4a_1f84_2e59,
            ]
        );
    }

    #[test]
    fn invalid_selection_and_probability_do_not_consume_state() {
        let mut random = DeterministicRng::from_seed(7);
        let before = random.state();
        assert!(random.choose::<u8>(&[]).is_err());
        assert!(random.chance(2, 1).is_err());
        assert!(random.chance(0, 0).is_err());
        assert_eq!(random.state(), before);
    }

    #[test]
    fn snapshots_continue_each_named_stream_exactly() {
        let mut streams = NamedRandomStreams::new(WorldSeed::new(99));
        streams.stream_mut(&stream("ticks")).next_u64();
        streams.stream_mut(&stream("loot")).next_u64();
        let snapshot = streams.snapshot();

        let expected = streams.stream_mut(&stream("ticks")).next_u64();
        let mut restored = NamedRandomStreams::restore(snapshot).unwrap();
        assert_eq!(restored.stream_mut(&stream("ticks")).next_u64(), expected);
    }

    #[test]
    fn serialized_random_state_cannot_restore_the_zero_lockup() {
        assert!(serde_json::from_str::<RandomState>("[0,0,0,0]").is_err());
    }

    #[test]
    fn uniform_selection_is_bounded_and_shuffle_is_repeatable() {
        let mut first = DeterministicRng::from_seed(123);
        let mut second = DeterministicRng::from_seed(123);
        for _ in 0..100 {
            assert!(first.choose_index(7).unwrap() < 7);
        }
        let mut left = [1, 2, 3, 4, 5];
        let mut right = left;
        second.shuffle(&mut left);
        let mut third = DeterministicRng::from_seed(123);
        third.shuffle(&mut right);
        assert_eq!(left, right);
    }
}
