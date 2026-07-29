//! Directional signal aggregation and default redstone-wire recomputation.

use ferrite_foundation::direction::Direction;

pub const MAX_SIGNAL: u8 = 15;
pub const WIRE_DECAY: u8 = 1;
pub const WIRE_POWER_WRITE_FLAGS: u16 = 2;
pub const WIRE_NOTIFICATION_SET_SIZE: usize = 7;
pub const DIRECT_SIGNAL_ORDER: [Direction; 6] = Direction::ALL;
pub const BEST_NEIGHBOR_ORDER: [Direction; 6] = Direction::ALL;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalSample {
    pub ordinary: u8,
    pub direct_into_block: u8,
    pub conductor: bool,
}

pub const fn combined_signal(sample: SignalSample) -> u8 {
    if sample.conductor && sample.direct_into_block > sample.ordinary {
        sample.direct_into_block
    } else {
        sample.ordinary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AggregatedSignal {
    pub signal: u8,
    pub probes: u8,
}

pub fn aggregate_signal(samples: [u8; 6]) -> AggregatedSignal {
    let mut signal = 0;
    for (index, sample) in samples.into_iter().enumerate() {
        signal = signal.max(sample);
        if signal == MAX_SIGNAL {
            return AggregatedSignal {
                signal,
                probes: index as u8 + 1,
            };
        }
    }
    AggregatedSignal { signal, probes: 6 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlSource {
    RedstoneBlock,
    Wire(u8),
    Diode(u8),
    OtherSignalSource(u8),
    Other,
}

pub const fn control_input(source: ControlSource, diode_only: bool) -> u8 {
    if diode_only {
        return match source {
            ControlSource::Diode(signal) => signal,
            _ => 0,
        };
    }
    match source {
        ControlSource::RedstoneBlock => MAX_SIGNAL,
        ControlSource::Wire(power) => power,
        ControlSource::Diode(signal) | ControlSource::OtherSignalSource(signal) => signal,
        ControlSource::Other => 0,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireEvaluator {
    Default,
    Experimental,
}

pub const fn selected_evaluator(experiments_enabled: bool) -> WireEvaluator {
    if experiments_enabled {
        WireEvaluator::Experimental
    } else {
        WireEvaluator::Default
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireRecomputeStage {
    DisableOwnSignal,
    BestBlockSignal,
    RestoreOwnSignal,
    AdjacentWireSignal,
    GuardedPowerWrite,
    NeighborSetDispatch,
}

pub const WIRE_RECOMPUTE_ORDER: [WireRecomputeStage; 6] = [
    WireRecomputeStage::DisableOwnSignal,
    WireRecomputeStage::BestBlockSignal,
    WireRecomputeStage::RestoreOwnSignal,
    WireRecomputeStage::AdjacentWireSignal,
    WireRecomputeStage::GuardedPowerWrite,
    WireRecomputeStage::NeighborSetDispatch,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireRoute {
    pub same_height: Option<u8>,
    pub neighbor_conductor: bool,
    pub above_neighbor: Option<u8>,
    pub below_neighbor: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WirePowerResult {
    pub power: u8,
    pub horizontal_routes_sampled: u8,
    pub returned_on_block_fifteen: bool,
}

pub fn default_wire_power(
    block_signal: u8,
    above_current_conductor: bool,
    routes: [WireRoute; 4],
) -> WirePowerResult {
    if block_signal == MAX_SIGNAL {
        return WirePowerResult {
            power: MAX_SIGNAL,
            horizontal_routes_sampled: 0,
            returned_on_block_fifteen: true,
        };
    }
    let mut incoming = 0;
    for route in routes {
        incoming = incoming.max(route.same_height.unwrap_or(0));
        if route.neighbor_conductor {
            if !above_current_conductor {
                incoming = incoming.max(route.above_neighbor.unwrap_or(0));
            }
        } else {
            incoming = incoming.max(route.below_neighbor.unwrap_or(0));
        }
    }
    WirePowerResult {
        power: block_signal.max(incoming.saturating_sub(WIRE_DECAY)),
        horizontal_routes_sampled: 4,
        returned_on_block_fifteen: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WirePowerCommit {
    pub offered_power: Option<u8>,
    pub write_flags: Option<u16>,
    pub unordered_notification_set_size: usize,
}

pub const fn wire_power_commit(
    old_power: u8,
    new_power: u8,
    exact_state_still_installed: bool,
) -> WirePowerCommit {
    if old_power == new_power || !exact_state_still_installed {
        WirePowerCommit {
            offered_power: None,
            write_flags: None,
            unordered_notification_set_size: 0,
        }
    } else {
        WirePowerCommit {
            offered_power: Some(new_power),
            write_flags: Some(WIRE_POWER_WRITE_FLAGS),
            unordered_notification_set_size: WIRE_NOTIFICATION_SET_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireConnection {
    None,
    Side,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireConnectionProbe {
    pub top_open: bool,
    pub neighbor_routes_up: bool,
    pub above_neighbor_connects: bool,
    pub neighbor_face_sturdy: bool,
    pub neighbor_is_wire: bool,
    pub repeater_on_axis: bool,
    pub observer_facing_wire: bool,
    pub neighbor_signal_source: bool,
    pub direction_supplied: bool,
    pub neighbor_conductor: bool,
    pub dust_below_neighbor: bool,
}

pub const fn wire_connection(probe: WireConnectionProbe) -> WireConnection {
    if probe.top_open
        && probe.neighbor_routes_up
        && probe.above_neighbor_connects
        && probe.neighbor_face_sturdy
    {
        return WireConnection::Up;
    }
    if probe.neighbor_is_wire
        || probe.repeater_on_axis
        || probe.observer_facing_wire
        || probe.neighbor_signal_source && probe.direction_supplied
        || !probe.neighbor_conductor && probe.dust_below_neighbor
    {
        WireConnection::Side
    } else {
        WireConnection::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireShape {
    Dot,
    Cross,
    Connected,
}

pub const fn normalized_placement_shape(has_connection: bool) -> WireShape {
    if has_connection {
        WireShape::Connected
    } else {
        WireShape::Cross
    }
}

pub const fn toggled_player_shape(current: WireShape, may_build: bool) -> Option<WireShape> {
    if !may_build {
        None
    } else {
        match current {
            WireShape::Dot => Some(WireShape::Cross),
            WireShape::Cross => Some(WireShape::Dot),
            WireShape::Connected => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireLifecycleStage {
    RecomputePower,
    VerticalNeighbors,
    HorizontalCorners,
}

pub const WIRE_PLACEMENT_ORDER: [WireLifecycleStage; 3] = [
    WireLifecycleStage::RecomputePower,
    WireLifecycleStage::VerticalNeighbors,
    WireLifecycleStage::HorizontalCorners,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireRemovalStage {
    SixNeighbors,
    RecomputeOldStateWithoutShape,
    HorizontalCorners,
}

pub const WIRE_REMOVAL_ORDER: [WireRemovalStage; 3] = [
    WireRemovalStage::SixNeighbors,
    WireRemovalStage::RecomputeOldStateWithoutShape,
    WireRemovalStage::HorizontalCorners,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireNeighborPlan {
    pub drop_and_remove: bool,
    pub recompute: bool,
    pub suppressed_experimental_self_callback: bool,
}

pub const fn wire_neighbor_plan(
    supported: bool,
    source_is_this_wire: bool,
    evaluator: WireEvaluator,
) -> WireNeighborPlan {
    if !supported {
        return WireNeighborPlan {
            drop_and_remove: true,
            recompute: false,
            suppressed_experimental_self_callback: false,
        };
    }
    let suppressed = source_is_this_wire && matches!(evaluator, WireEvaluator::Experimental);
    WireNeighborPlan {
        drop_and_remove: false,
        recompute: !suppressed,
        suppressed_experimental_self_callback: suppressed,
    }
}
