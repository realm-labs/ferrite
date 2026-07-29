//! Connection-local assembly for clientbound protocol bundles.

use thiserror::Error;

use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::clientbound::terrain::packet::TerrainPacket;

pub const MAX_BUNDLE_SUBPACKETS: usize = 4_096;

#[derive(Debug, Clone, PartialEq)]
pub enum BundledPlayPackets {
    Single(PlayClientboundPacket),
    Bundle(Vec<PlayClientboundPacket>),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ClientboundBundleAssembler {
    open: Option<Vec<PlayClientboundPacket>>,
}

impl ClientboundBundleAssembler {
    #[must_use]
    pub const fn new() -> Self {
        Self { open: None }
    }

    pub fn push(
        &mut self,
        packet: PlayClientboundPacket,
    ) -> Result<Option<BundledPlayPackets>, BundleError> {
        if matches!(
            packet,
            PlayClientboundPacket::Terrain(TerrainPacket::BundleDelimiter)
        ) {
            return Ok(match self.open.take() {
                Some(packets) => Some(BundledPlayPackets::Bundle(packets)),
                None => {
                    self.open = Some(Vec::new());
                    None
                }
            });
        }

        let Some(packets) = self.open.as_mut() else {
            return Ok(Some(BundledPlayPackets::Single(packet)));
        };
        if matches!(packet, PlayClientboundPacket::Disconnect(_)) {
            return Err(BundleError::TerminalPacket);
        }
        if packets.len() == MAX_BUNDLE_SUBPACKETS {
            return Err(BundleError::TooManySubpackets {
                maximum: MAX_BUNDLE_SUBPACKETS,
            });
        }
        packets.push(packet);
        Ok(None)
    }

    #[must_use]
    pub fn open_len(&self) -> Option<usize> {
        self.open.as_ref().map(Vec::len)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BundleError {
    #[error("a terminal clientbound packet cannot occur inside a bundle")]
    TerminalPacket,
    #[error("a clientbound bundle exceeds its {maximum}-packet limit")]
    TooManySubpackets { maximum: usize },
}
