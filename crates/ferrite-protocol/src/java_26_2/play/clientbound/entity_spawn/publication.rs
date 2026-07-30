#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PairingAdmission {
    pub horizontal_distance_squared: f64,
    pub effective_range_blocks: u32,
    pub view_distance_blocks: u32,
    pub broadcast_allowed: bool,
    pub chunk_tracked: bool,
    pub viewer_is_entity: bool,
}

#[must_use]
pub fn pairing_allowed(admission: PairingAdmission) -> bool {
    if admission.viewer_is_entity || !admission.broadcast_allowed || !admission.chunk_tracked {
        return false;
    }
    let range = admission
        .effective_range_blocks
        .min(admission.view_distance_blocks);
    admission.horizontal_distance_squared <= f64::from(range).powi(2)
}

#[must_use]
pub fn effective_tracking_range(
    entity_range: u32,
    indirect_passenger_ranges: &[u32],
    scale: impl Fn(u32) -> u32,
) -> u32 {
    let maximum = indirect_passenger_ranges
        .iter()
        .copied()
        .fold(entity_range, u32::max);
    scale(maximum)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PairingContent {
    pub player_info: bool,
    pub metadata: bool,
    pub attributes: bool,
    pub equipment: bool,
    pub own_passengers: bool,
    pub vehicle_passengers: bool,
    pub leash: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingStep {
    UpdateDataBeforeSync,
    PlayerInfo,
    AddEntity,
    Metadata,
    Attributes,
    Equipment,
    OwnPassengers,
    VehiclePassengers,
    Leash,
    SendBundle,
    StartSeenByPlayer,
}

#[must_use]
pub fn pairing_plan(content: PairingContent) -> Vec<PairingStep> {
    let mut plan = vec![PairingStep::UpdateDataBeforeSync];
    if content.player_info {
        plan.push(PairingStep::PlayerInfo);
    }
    plan.push(PairingStep::AddEntity);
    for (present, step) in [
        (content.metadata, PairingStep::Metadata),
        (content.attributes, PairingStep::Attributes),
        (content.equipment, PairingStep::Equipment),
        (content.own_passengers, PairingStep::OwnPassengers),
        (content.vehicle_passengers, PairingStep::VehiclePassengers),
        (content.leash, PairingStep::Leash),
    ] {
        if present {
            plan.push(step);
        }
    }
    plan.extend([PairingStep::SendBundle, PairingStep::StartSeenByPlayer]);
    plan
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnpairingStep {
    StopSeenByPlayer,
    SendSingleEntityRemoval,
}

pub const UNPAIRING_ORDER: [UnpairingStep; 2] = [
    UnpairingStep::StopSeenByPlayer,
    UnpairingStep::SendSingleEntityRemoval,
];
