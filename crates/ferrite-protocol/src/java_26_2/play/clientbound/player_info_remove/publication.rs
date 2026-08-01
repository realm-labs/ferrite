use crate::java_26_2::play::clientbound::player_info_remove::PlayerInfoRemove;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDepartureStep {
    SavePlayer(u128),
    RemoveEntity(u128),
    RemoveServerMembership(u128),
    DisconnectPresentationServices(u128),
    PublishTrackerRemoval(u128),
    PublishPlayerInfoRemoval { recipient: u128, departed: u128 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerInfoRemovalDelivery {
    pub recipient: u128,
    pub packet: PlayerInfoRemove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerDeparturePublication {
    pub steps: Vec<PlayerDepartureStep>,
    pub deliveries: Vec<PlayerInfoRemovalDelivery>,
}

#[must_use]
pub fn publish_departure(
    departed: u128,
    remaining_global_players: &[u128],
) -> PlayerDeparturePublication {
    let mut steps = vec![
        PlayerDepartureStep::SavePlayer(departed),
        PlayerDepartureStep::RemoveEntity(departed),
        PlayerDepartureStep::RemoveServerMembership(departed),
        PlayerDepartureStep::DisconnectPresentationServices(departed),
        PlayerDepartureStep::PublishTrackerRemoval(departed),
    ];
    let deliveries = remaining_global_players
        .iter()
        .map(|recipient| {
            steps.push(PlayerDepartureStep::PublishPlayerInfoRemoval {
                recipient: *recipient,
                departed,
            });
            PlayerInfoRemovalDelivery {
                recipient: *recipient,
                packet: PlayerInfoRemove {
                    profile_ids: vec![departed],
                },
            }
        })
        .collect();
    PlayerDeparturePublication { steps, deliveries }
}

#[must_use]
pub const fn respawn_replacement_packet() -> Option<PlayerInfoRemove> {
    None
}
