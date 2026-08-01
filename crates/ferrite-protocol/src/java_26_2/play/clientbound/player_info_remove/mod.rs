//! Clientbound player-info removal projection.

pub mod codec;
pub mod projection;
pub mod publication;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoRemove {
    pub profile_ids: Vec<u128>,
}
