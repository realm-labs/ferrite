use thiserror::Error;

use crate::java_26_2::play::clientbound::container::publication::{
    ContainerPublicationError, ContainerPublisher, MenuSnapshot,
};
use crate::java_26_2::play::clientbound::merchant::packet::{MerchantOffer, MerchantOffers};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantSnapshot {
    pub offers: Vec<MerchantOffer>,
    pub villager_level: i32,
    pub villager_experience: i32,
    pub show_progress: bool,
    pub can_restock: bool,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MerchantPublisher {
    containers: ContainerPublisher,
}

impl MerchantPublisher {
    pub fn open_trading(
        &mut self,
        menu: MenuSnapshot,
        merchant: &MerchantSnapshot,
    ) -> Result<Vec<PlayClientboundPacket>, MerchantPublicationError> {
        let mut packets = self.containers.open(menu)?;
        if merchant.offers.is_empty() {
            return Ok(packets);
        }
        let container_id = self
            .containers
            .current_container_id()
            .ok_or(MerchantPublicationError::MissingOpenedMenu)?;
        packets.push(PlayClientboundPacket::MerchantOffers(MerchantOffers {
            container_id,
            offers: merchant.offers.clone(),
            villager_level: merchant.villager_level,
            villager_experience: merchant.villager_experience,
            show_progress: merchant.show_progress,
            can_restock: merchant.can_restock,
        }));
        Ok(packets)
    }

    #[must_use]
    pub const fn containers(&self) -> &ContainerPublisher {
        &self.containers
    }

    pub const fn containers_mut(&mut self) -> &mut ContainerPublisher {
        &mut self.containers
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MerchantPublicationError {
    #[error(transparent)]
    Container(#[from] ContainerPublicationError),
    #[error("merchant open completed without installing a current menu")]
    MissingOpenedMenu,
}
