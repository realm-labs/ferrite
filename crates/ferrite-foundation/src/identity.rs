//! Stable world, dimension, entity, and activation identities.

use crate::resource::{ResourceId, ResourceIdError};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::num::{NonZeroU64, NonZeroU128};
use std::str::FromStr;
use thiserror::Error;

macro_rules! stable_id {
    ($name:ident, $description:literal) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(NonZeroU128);

        impl $name {
            pub const fn new(value: u128) -> Result<Self, StableIdError> {
                match NonZeroU128::new(value) {
                    Some(value) => Ok(Self(value)),
                    None => Err(StableIdError::Zero),
                }
            }

            pub const fn get(self) -> u128 {
                self.0.get()
            }

            pub const fn to_be_bytes(self) -> [u8; 16] {
                self.get().to_be_bytes()
            }
        }

        impl Display for $name {
            fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
                write!(formatter, "{:032x}", self.get())
            }
        }

        impl FromStr for $name {
            type Err = StableIdError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                parse_stable_id(value).and_then(Self::new)
            }
        }

        impl TryFrom<String> for $name {
            type Error = StableIdError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                value.parse()
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.to_string()
            }
        }
    };
}

stable_id!(WorldId, "stable world identity");
stable_id!(StableEntityId, "stable entity identity");

fn parse_stable_id(value: &str) -> Result<u128, StableIdError> {
    if value.len() != 32 {
        return Err(StableIdError::InvalidLength {
            actual: value.len(),
        });
    }
    u128::from_str_radix(value, 16).map_err(|_| StableIdError::InvalidHex)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum StableIdError {
    #[error("stable identity cannot be zero")]
    Zero,
    #[error("stable identity must contain exactly 32 hexadecimal bytes, got {actual}")]
    InvalidLength { actual: usize },
    #[error("stable identity contains a non-hexadecimal character")]
    InvalidHex,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DimensionId(ResourceId);

impl DimensionId {
    pub const fn new(identifier: ResourceId) -> Self {
        Self(identifier)
    }

    pub const fn resource(&self) -> &ResourceId {
        &self.0
    }
}

impl Display for DimensionId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

impl FromStr for DimensionId {
    type Err = ResourceIdError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse().map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ActivationGeneration(NonZeroU64);

impl ActivationGeneration {
    pub const INITIAL: Self = Self(NonZeroU64::MIN);

    pub const fn new(value: u64) -> Result<Self, ActivationGenerationError> {
        match NonZeroU64::new(value) {
            Some(value) => Ok(Self(value)),
            None => Err(ActivationGenerationError::Zero),
        }
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }

    pub const fn checked_next(self) -> Result<Self, ActivationGenerationError> {
        match self.get().checked_add(1) {
            Some(value) => Self::new(value),
            None => Err(ActivationGenerationError::Exhausted),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ActivationGenerationError {
    #[error("activation generation cannot be zero")]
    Zero,
    #[error("activation generation is exhausted")]
    Exhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_ids_use_fixed_width_canonical_hex() {
        let world = WorldId::new(0x2a).unwrap();
        assert_eq!(world.to_string(), "0000000000000000000000000000002a");
        assert_eq!(world.to_string().parse::<WorldId>().unwrap(), world);
        assert!(WorldId::new(0).is_err());
        assert!("2a".parse::<WorldId>().is_err());
    }

    #[test]
    fn stable_id_deserialization_preserves_invariants() {
        let entity = StableEntityId::new(7).unwrap();
        let encoded = serde_json::to_string(&entity).unwrap();
        assert_eq!(
            serde_json::from_str::<StableEntityId>(&encoded).unwrap(),
            entity
        );
        assert!(
            serde_json::from_str::<StableEntityId>("\"00000000000000000000000000000000\"").is_err()
        );
    }

    #[test]
    fn activation_generation_is_monotonic_and_checked() {
        assert_eq!(
            ActivationGeneration::INITIAL.checked_next().unwrap().get(),
            2
        );
        assert!(ActivationGeneration::new(0).is_err());
        assert!(
            ActivationGeneration::new(u64::MAX)
                .unwrap()
                .checked_next()
                .is_err()
        );
    }
}
