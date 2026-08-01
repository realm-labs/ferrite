//! Authoritative border extent, mutations, listener delivery, and snapshots.

use super::{
    DEFAULT_ABSOLUTE_MAX, DEFAULT_DAMAGE_PER_BLOCK, DEFAULT_SAFE_ZONE, DEFAULT_SIZE,
    DEFAULT_WARNING_BLOCKS, DEFAULT_WARNING_TIME,
};

pub type BorderListenerId = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MovingExtent {
    pub from: f64,
    pub to: f64,
    pub duration: f64,
    pub remaining_ticks: i64,
    pub previous_size: f64,
    pub current_size: f64,
    pub begin_game_time: i64,
    pub end_game_time: i64,
}

impl MovingExtent {
    fn new(from: f64, to: f64, duration_ticks: i64, begin_game_time: i64) -> Self {
        let duration = duration_ticks as f64;
        let current_size = calculated_size(from, to, duration, duration_ticks);
        Self {
            from,
            to,
            duration,
            remaining_ticks: duration_ticks,
            previous_size: current_size,
            current_size,
            begin_game_time,
            end_game_time: begin_game_time.wrapping_add(duration_ticks),
        }
    }

    pub fn partial_size(self, partial_tick: f64) -> f64 {
        lerp(partial_tick, self.previous_size, self.current_size)
    }

    pub fn speed(self) -> f64 {
        (self.to - self.from).abs() / self.duration
    }
}

fn calculated_size(from: f64, to: f64, duration: f64, remaining_ticks: i64) -> f64 {
    let progress = (duration - remaining_ticks as f64) / duration;
    if progress < 1.0 {
        lerp(progress, from, to)
    } else {
        to
    }
}

pub(crate) fn lerp(progress: f64, from: f64, to: f64) -> f64 {
    from + progress * (to - from)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderExtent {
    Static { size: f64 },
    Moving(MovingExtent),
}

impl BorderExtent {
    pub fn size(self) -> f64 {
        match self {
            Self::Static { size } => size,
            Self::Moving(moving) => moving.current_size,
        }
    }

    pub fn size_at(self, partial_tick: f64) -> f64 {
        match self {
            Self::Static { size } => size,
            Self::Moving(moving) => moving.partial_size(partial_tick),
        }
    }

    pub fn target(self) -> f64 {
        match self {
            Self::Static { size } => size,
            Self::Moving(moving) => moving.to,
        }
    }

    pub fn remaining_ticks(self) -> i64 {
        match self {
            Self::Static { .. } => 0,
            Self::Moving(moving) => moving.remaining_ticks,
        }
    }

    pub fn speed(self) -> f64 {
        match self {
            Self::Static { .. } => 0.0,
            Self::Moving(moving) => moving.speed(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BorderStatus {
    Stationary,
    Growing,
    Shrinking,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderEvent {
    SetSize {
        size: f64,
    },
    LerpSize {
        from: f64,
        to: f64,
        duration_ticks: i64,
    },
    SetCenter {
        x: f64,
        z: f64,
    },
    SetWarningBlocks {
        blocks: i32,
    },
    SetWarningTime {
        ticks: i32,
    },
    SetDamagePerBlock {
        rate: f64,
    },
    SetSafeZone {
        blocks: f64,
    },
    SetAbsoluteMax {
        coordinate: i32,
    },
}

impl BorderEvent {
    pub const fn has_client_packet(self) -> bool {
        matches!(
            self,
            Self::SetSize { .. }
                | Self::LerpSize { .. }
                | Self::SetCenter { .. }
                | Self::SetWarningBlocks { .. }
                | Self::SetWarningTime { .. }
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderDelivery {
    pub listener: BorderListenerId,
    pub event: BorderEvent,
    pub broadcast_to_dimension: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorderMutation {
    pub dirty_revision: u64,
    pub deliveries: Vec<BorderDelivery>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SavedBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub size: f64,
    pub target_size: f64,
    pub remaining_ticks: i64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BorderSnapshot {
    pub center_x: f64,
    pub center_z: f64,
    pub old_size: f64,
    pub new_size: f64,
    pub remaining_ticks: i64,
    pub absolute_max: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorldBorder {
    pub center_x: f64,
    pub center_z: f64,
    pub absolute_max: i32,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub extent: BorderExtent,
    pub dirty_revision: u64,
    listeners: Vec<BorderListenerId>,
}

impl Default for WorldBorder {
    fn default() -> Self {
        Self {
            center_x: 0.0,
            center_z: 0.0,
            absolute_max: DEFAULT_ABSOLUTE_MAX,
            damage_per_block: DEFAULT_DAMAGE_PER_BLOCK,
            safe_zone: DEFAULT_SAFE_ZONE,
            warning_blocks: DEFAULT_WARNING_BLOCKS,
            warning_time: DEFAULT_WARNING_TIME,
            extent: BorderExtent::Static { size: DEFAULT_SIZE },
            dirty_revision: 0,
            listeners: Vec::new(),
        }
    }
}

impl WorldBorder {
    pub fn from_saved(saved: SavedBorder, begin_game_time: i64) -> Self {
        let extent = if saved.remaining_ticks > 0 && saved.size != saved.target_size {
            BorderExtent::Moving(MovingExtent::new(
                saved.size,
                saved.target_size,
                saved.remaining_ticks,
                begin_game_time,
            ))
        } else {
            BorderExtent::Static { size: saved.size }
        };
        Self {
            center_x: saved.center_x,
            center_z: saved.center_z,
            absolute_max: DEFAULT_ABSOLUTE_MAX,
            damage_per_block: saved.damage_per_block,
            safe_zone: saved.safe_zone,
            warning_blocks: saved.warning_blocks,
            warning_time: saved.warning_time,
            extent,
            dirty_revision: 0,
            listeners: Vec::new(),
        }
    }

    pub fn from_snapshot(snapshot: BorderSnapshot, begin_game_time: i64) -> Self {
        let extent = if snapshot.old_size == snapshot.new_size {
            BorderExtent::Static {
                size: snapshot.new_size,
            }
        } else {
            BorderExtent::Moving(MovingExtent::new(
                snapshot.old_size,
                snapshot.new_size,
                snapshot.remaining_ticks,
                begin_game_time,
            ))
        };
        Self {
            center_x: snapshot.center_x,
            center_z: snapshot.center_z,
            absolute_max: snapshot.absolute_max,
            warning_blocks: snapshot.warning_blocks,
            warning_time: snapshot.warning_time,
            extent,
            ..Self::default()
        }
    }

    pub fn add_listener(&mut self, listener: BorderListenerId) {
        self.listeners.push(listener);
    }

    pub fn get_size(&self) -> f64 {
        self.extent.size()
    }

    pub fn target_size(&self) -> f64 {
        self.extent.target()
    }

    pub fn remaining_ticks(&self) -> i64 {
        self.extent.remaining_ticks()
    }

    pub fn status(&self) -> BorderStatus {
        match self.extent {
            BorderExtent::Static { .. } => BorderStatus::Stationary,
            BorderExtent::Moving(moving) if moving.to < moving.from => BorderStatus::Shrinking,
            BorderExtent::Moving(_) => BorderStatus::Growing,
        }
    }

    pub fn set_size(&mut self, size: f64) -> BorderMutation {
        self.extent = BorderExtent::Static { size };
        self.mutate(BorderEvent::SetSize { size })
    }

    pub fn lerp_size_between(
        &mut self,
        from: f64,
        to: f64,
        duration_ticks: i64,
        begin_game_time: i64,
    ) -> BorderMutation {
        self.extent = if from == to {
            BorderExtent::Static { size: to }
        } else {
            BorderExtent::Moving(MovingExtent::new(from, to, duration_ticks, begin_game_time))
        };
        self.mutate(BorderEvent::LerpSize {
            from,
            to,
            duration_ticks,
        })
    }

    pub fn set_center(&mut self, x: f64, z: f64) -> BorderMutation {
        self.center_x = x;
        self.center_z = z;
        self.mutate(BorderEvent::SetCenter { x, z })
    }

    pub fn set_warning_blocks(&mut self, blocks: i32) -> BorderMutation {
        self.warning_blocks = blocks;
        self.mutate(BorderEvent::SetWarningBlocks { blocks })
    }

    pub fn set_warning_time(&mut self, ticks: i32) -> BorderMutation {
        self.warning_time = ticks;
        self.mutate(BorderEvent::SetWarningTime { ticks })
    }

    pub fn set_damage_per_block(&mut self, rate: f64) -> BorderMutation {
        self.damage_per_block = rate;
        self.mutate(BorderEvent::SetDamagePerBlock { rate })
    }

    pub fn set_safe_zone(&mut self, blocks: f64) -> BorderMutation {
        self.safe_zone = blocks;
        self.mutate(BorderEvent::SetSafeZone { blocks })
    }

    pub fn set_absolute_max(&mut self, coordinate: i32) -> BorderMutation {
        self.absolute_max = coordinate;
        self.mutate(BorderEvent::SetAbsoluteMax { coordinate })
    }

    pub fn tick_if_running(&mut self, running_normally: bool) -> bool {
        if !running_normally {
            return false;
        }
        let BorderExtent::Moving(mut moving) = self.extent else {
            return false;
        };
        moving.remaining_ticks = moving.remaining_ticks.wrapping_sub(1);
        moving.previous_size = moving.current_size;
        moving.current_size = calculated_size(
            moving.from,
            moving.to,
            moving.duration,
            moving.remaining_ticks,
        );
        self.dirty_revision = self.dirty_revision.wrapping_add(1);
        self.extent = if moving.remaining_ticks <= 0 {
            BorderExtent::Static { size: moving.to }
        } else {
            BorderExtent::Moving(moving)
        };
        true
    }

    pub fn saved(&self) -> SavedBorder {
        SavedBorder {
            center_x: self.center_x,
            center_z: self.center_z,
            size: self.get_size(),
            target_size: self.target_size(),
            remaining_ticks: self.remaining_ticks(),
            damage_per_block: self.damage_per_block,
            safe_zone: self.safe_zone,
            warning_blocks: self.warning_blocks,
            warning_time: self.warning_time,
        }
    }

    pub fn snapshot(&self) -> BorderSnapshot {
        BorderSnapshot {
            center_x: self.center_x,
            center_z: self.center_z,
            old_size: self.get_size(),
            new_size: self.target_size(),
            remaining_ticks: self.remaining_ticks(),
            absolute_max: self.absolute_max,
            warning_blocks: self.warning_blocks,
            warning_time: self.warning_time,
        }
    }

    fn mutate(&mut self, event: BorderEvent) -> BorderMutation {
        self.dirty_revision = self.dirty_revision.wrapping_add(1);
        let listeners = self.listeners.clone();
        BorderMutation {
            dirty_revision: self.dirty_revision,
            deliveries: listeners
                .into_iter()
                .map(|listener| BorderDelivery {
                    listener,
                    event,
                    broadcast_to_dimension: event.has_client_packet(),
                })
                .collect(),
        }
    }
}
