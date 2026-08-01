//! Boss-bar and waypoint server publication semantics.

use crate::java_26_2::play::clientbound::boss_waypoint::packet::{
    BossColor, BossEvent, BossOperation, BossOverlay, TrackedWaypoint, WaypointIcon,
    WaypointIdentifier, WaypointLocation, WaypointOperation, WaypointPacket,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, PartialEq)]
pub struct BossBroadcast {
    pub recipient: u64,
    pub packet: BossEvent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BossPublisher {
    pub id: u128,
    pub name: TextComponentNbt,
    pub progress: f32,
    pub color: BossColor,
    pub overlay: BossOverlay,
    pub darken_screen: bool,
    pub play_music: bool,
    pub create_fog: bool,
    pub visible: bool,
    pub dirty_updates: u64,
    audience: Vec<u64>,
}

impl BossPublisher {
    #[must_use]
    pub fn new(id: u128, name: TextComponentNbt) -> Self {
        Self {
            id,
            name,
            progress: 1.0,
            color: BossColor::White,
            overlay: BossOverlay::Progress,
            darken_screen: false,
            play_music: false,
            create_fog: false,
            visible: true,
            dirty_updates: 0,
            audience: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player: u64) -> Vec<BossBroadcast> {
        if self.audience.contains(&player) {
            return Vec::new();
        }
        self.audience.push(player);
        if self.visible {
            vec![self.message(player, self.add_operation())]
        } else {
            Vec::new()
        }
    }

    pub fn remove_player(&mut self, player: u64) -> Vec<BossBroadcast> {
        let Some(index) = self
            .audience
            .iter()
            .position(|candidate| *candidate == player)
        else {
            return Vec::new();
        };
        self.audience.remove(index);
        if self.visible {
            vec![self.message(player, BossOperation::Remove)]
        } else {
            Vec::new()
        }
    }

    pub fn set_visible(&mut self, visible: bool) -> Vec<BossBroadcast> {
        if self.visible == visible {
            return Vec::new();
        }
        self.visible = visible;
        let operation = visible.then(|| self.add_operation());
        self.audience
            .iter()
            .copied()
            .map(|player| self.message(player, operation.clone().unwrap_or(BossOperation::Remove)))
            .collect()
    }

    pub fn set_progress(&mut self, progress: f32) -> Vec<BossBroadcast> {
        if self.progress == progress {
            return Vec::new();
        }
        self.progress = progress;
        self.dirty_updates += 1;
        self.broadcast(BossOperation::UpdateProgress(progress))
    }

    pub fn set_name(&mut self, name: TextComponentNbt) -> Vec<BossBroadcast> {
        if self.name == name {
            return Vec::new();
        }
        self.name = name.clone();
        self.dirty_updates += 1;
        self.broadcast(BossOperation::UpdateName(name))
    }

    pub fn set_style(&mut self, color: BossColor, overlay: BossOverlay) -> Vec<BossBroadcast> {
        if self.color == color && self.overlay == overlay {
            return Vec::new();
        }
        self.color = color;
        self.overlay = overlay;
        self.dirty_updates += 1;
        self.broadcast(BossOperation::UpdateStyle { color, overlay })
    }

    pub fn set_properties(
        &mut self,
        darken_screen: bool,
        play_music: bool,
        create_fog: bool,
    ) -> Vec<BossBroadcast> {
        if (self.darken_screen, self.play_music, self.create_fog)
            == (darken_screen, play_music, create_fog)
        {
            return Vec::new();
        }
        self.darken_screen = darken_screen;
        self.play_music = play_music;
        self.create_fog = create_fog;
        self.dirty_updates += 1;
        self.broadcast(BossOperation::UpdateProperties(self.property_byte()))
    }

    fn broadcast(&self, operation: BossOperation) -> Vec<BossBroadcast> {
        if !self.visible {
            return Vec::new();
        }
        self.audience
            .iter()
            .copied()
            .map(|player| self.message(player, operation.clone()))
            .collect()
    }

    fn message(&self, recipient: u64, operation: BossOperation) -> BossBroadcast {
        BossBroadcast {
            recipient,
            packet: BossEvent {
                id: self.id,
                operation,
            },
        }
    }

    fn add_operation(&self) -> BossOperation {
        BossOperation::Add {
            name: self.name.clone(),
            progress: self.progress,
            color: self.color,
            overlay: self.overlay,
            properties: self.property_byte(),
        }
    }

    const fn property_byte(&self) -> u8 {
        self.darken_screen as u8 | (self.play_music as u8) << 1 | (self.create_fog as u8) << 2
    }
}

#[must_use]
pub fn resolved_icon(
    explicit_color: Option<u32>,
    team_color: Option<u32>,
    team_is_black: bool,
) -> Option<u32> {
    if explicit_color.is_some() {
        explicit_color
    } else if team_is_black {
        Some(0xff30_3030)
    } else {
        team_color.map(|color| 0xff00_0000 | color & 0x00ff_ffff)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaypointSource {
    pub uuid: u128,
    pub position: [f64; 3],
    pub block_position: [i32; 3],
    pub chunk: [i32; 2],
    pub spectator: bool,
    pub first_tick: bool,
    pub transmit_range: f64,
    pub icon: WaypointIcon,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaypointReceiver {
    pub uuid: u128,
    pub position: [f64; 3],
    pub spectator: bool,
    pub riding_source: bool,
    pub receive_range: f64,
    pub source_chunk_visible: bool,
}

#[must_use]
pub fn select_waypoint(
    source: &WaypointSource,
    receiver: WaypointReceiver,
    locator_bar_enabled: bool,
) -> Option<TrackedWaypoint> {
    if source.uuid == receiver.uuid || source.first_tick || !locator_bar_enabled {
        return None;
    }
    let distance = euclidean_distance(source.position, receiver.position);
    let range = source.transmit_range.min(receiver.receive_range);
    let within_range = !distance.is_nan() && distance < range;
    if !receiver.spectator && (source.spectator || receiver.riding_source || !within_range) {
        return None;
    }
    let location = if distance > 332.0 {
        let dx = receiver.position[0] - source.position[0];
        let dz = receiver.position[2] - source.position[2];
        WaypointLocation::Azimuth {
            angle: (dz.atan2(dx) - std::f64::consts::FRAC_PI_2) as f32,
        }
    } else if !receiver.source_chunk_visible {
        WaypointLocation::Chunk {
            x: source.chunk[0],
            z: source.chunk[1],
        }
    } else {
        WaypointLocation::Position {
            x: source.block_position[0],
            y: source.block_position[1],
            z: source.block_position[2],
        }
    };
    Some(TrackedWaypoint {
        identifier: WaypointIdentifier::Uuid(source.uuid),
        icon: source.icon.clone(),
        location,
    })
}

#[must_use]
pub fn waypoint_transition(
    previous: Option<&TrackedWaypoint>,
    next: Option<TrackedWaypoint>,
    previous_chunk_visible: bool,
    default_style: Identifier,
) -> Option<WaypointPacket> {
    let Some(next) = next else {
        let previous = previous?;
        return Some(WaypointPacket {
            operation: WaypointOperation::Untrack,
            waypoint: TrackedWaypoint {
                identifier: previous.identifier.clone(),
                icon: WaypointIcon {
                    style: default_style,
                    color: None,
                },
                location: WaypointLocation::Empty,
            },
        });
    };
    let Some(previous) = previous else {
        return Some(WaypointPacket {
            operation: WaypointOperation::Track,
            waypoint: next,
        });
    };
    if previous.identifier != next.identifier || previous.icon != next.icon {
        return Some(track(next));
    }
    let operation = match (&previous.location, &next.location) {
        (
            WaypointLocation::Position { x, y, z },
            WaypointLocation::Position {
                x: next_x,
                y: next_y,
                z: next_z,
            },
        ) => {
            if (x, y, z) == (next_x, next_y, next_z) {
                return None;
            }
            let manhattan = i64::from(x.abs_diff(*next_x))
                + i64::from(y.abs_diff(*next_y))
                + i64::from(z.abs_diff(*next_z));
            if manhattan > 1 {
                WaypointOperation::Track
            } else {
                WaypointOperation::Update
            }
        }
        (
            WaypointLocation::Chunk { x, z },
            WaypointLocation::Chunk {
                x: next_x,
                z: next_z,
            },
        ) => {
            if (x, z) == (next_x, next_z) {
                return None;
            }
            let chessboard = x.abs_diff(*next_x).max(z.abs_diff(*next_z));
            if chessboard > 1 || previous_chunk_visible {
                WaypointOperation::Track
            } else {
                WaypointOperation::Update
            }
        }
        (WaypointLocation::Azimuth { angle }, WaypointLocation::Azimuth { angle: next }) => {
            if (angle - next).abs() > 0.008_726_646 {
                WaypointOperation::Update
            } else {
                return None;
            }
        }
        _ => WaypointOperation::Track,
    };
    Some(WaypointPacket {
        operation,
        waypoint: next,
    })
}

fn track(waypoint: TrackedWaypoint) -> WaypointPacket {
    WaypointPacket {
        operation: WaypointOperation::Track,
        waypoint,
    }
}

fn euclidean_distance(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right) * (left - right))
        .sum::<f64>()
        .sqrt()
}
