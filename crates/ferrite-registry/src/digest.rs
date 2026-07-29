//! Stable BLAKE3 content fingerprints.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

const DIGEST_BYTES: usize = 32;
const DIGEST_HEX_BYTES: usize = DIGEST_BYTES * 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ContentDigest([u8; DIGEST_BYTES]);

impl ContentDigest {
    pub const fn from_bytes(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    pub fn blake3(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }
}

impl Display for ContentDigest {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl FromStr for ContentDigest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != DIGEST_HEX_BYTES {
            return Err(DigestError::InvalidLength {
                actual: value.len(),
            });
        }
        let mut bytes = [0_u8; DIGEST_BYTES];
        for (index, byte) in bytes.iter_mut().enumerate() {
            let offset = index * 2;
            let pair = &value.as_bytes()[offset..offset + 2];
            let high = decode_nibble(pair[0]).ok_or(DigestError::InvalidHex { index: offset })?;
            let low =
                decode_nibble(pair[1]).ok_or(DigestError::InvalidHex { index: offset + 1 })?;
            *byte = (high << 4) | low;
        }
        Ok(Self(bytes))
    }
}

impl TryFrom<String> for ContentDigest {
    type Error = DigestError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<ContentDigest> for String {
    fn from(value: ContentDigest) -> Self {
        value.to_string()
    }
}

const fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DigestError {
    #[error("content digest must contain exactly 64 hexadecimal bytes, got {actual}")]
    InvalidLength { actual: usize },
    #[error("content digest contains invalid hexadecimal at byte {index}")]
    InvalidHex { index: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_has_canonical_lowercase_hex_encoding() {
        let digest = ContentDigest::blake3(b"ferrite");
        assert_eq!(digest.to_string().len(), 64);
        assert_eq!(digest.to_string().parse::<ContentDigest>().unwrap(), digest);
        assert!(
            digest
                .to_string()
                .to_uppercase()
                .parse::<ContentDigest>()
                .is_err()
        );
    }

    #[test]
    fn serde_revalidates_digest_strings() {
        let digest = ContentDigest::blake3(b"content");
        let encoded = serde_json::to_string(&digest).unwrap();
        assert_eq!(
            serde_json::from_str::<ContentDigest>(&encoded).unwrap(),
            digest
        );
        assert!(serde_json::from_str::<ContentDigest>("\"00\"").is_err());
    }
}
