use thiserror::Error;

use crate::java_26_2::play::clientbound::merchant::packet::{MerchantOffer, MerchantOffers};
use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerchantUpdate {
    Offers,
    Experience,
    Level,
    ShowProgress,
    CanRestock,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MerchantMenuProjection {
    pub container_id: i32,
    pub offers: Vec<MerchantOffer>,
    pub villager_experience: i32,
    pub villager_level: i32,
    pub show_progress: bool,
    pub can_restock: bool,
    pub last_update_order: Vec<MerchantUpdate>,
}

impl MerchantMenuProjection {
    fn new(container_id: i32) -> Self {
        Self {
            container_id,
            offers: Vec::new(),
            villager_experience: 0,
            villager_level: 0,
            show_progress: false,
            can_restock: false,
            last_update_order: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MerchantClientProjection {
    current_container_id: Option<i32>,
    current: Option<MerchantMenuProjection>,
}

impl MerchantClientProjection {
    pub fn open_menu(&mut self, container_id: i32, merchant: bool) {
        self.current_container_id = Some(container_id);
        self.current = merchant.then(|| MerchantMenuProjection::new(container_id));
    }

    pub fn close_menu(&mut self) {
        self.current_container_id = None;
        self.current = None;
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<bool, MerchantProjectionError> {
        let PlayClientboundPacket::MerchantOffers(packet) = packet else {
            return Err(MerchantProjectionError::WrongPacketFamily);
        };
        if self.current_container_id != Some(packet.container_id) {
            return Ok(false);
        }
        let Some(menu) = self.current.as_mut() else {
            return Ok(false);
        };
        apply_offer_snapshot(menu, packet);
        Ok(true)
    }

    #[must_use]
    pub const fn current(&self) -> Option<&MerchantMenuProjection> {
        self.current.as_ref()
    }
}

fn apply_offer_snapshot(menu: &mut MerchantMenuProjection, packet: &MerchantOffers) {
    menu.last_update_order.clear();
    menu.offers = packet.offers.clone();
    menu.last_update_order.push(MerchantUpdate::Offers);
    menu.villager_experience = packet.villager_experience;
    menu.last_update_order.push(MerchantUpdate::Experience);
    menu.villager_level = packet.villager_level;
    menu.last_update_order.push(MerchantUpdate::Level);
    menu.show_progress = packet.show_progress;
    menu.last_update_order.push(MerchantUpdate::ShowProgress);
    menu.can_restock = packet.can_restock;
    menu.last_update_order.push(MerchantUpdate::CanRestock);
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MerchantProjectionError {
    #[error("packet does not belong to the merchant clientbound family")]
    WrongPacketFamily,
}
