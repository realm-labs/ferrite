use crate::java_26_2::play::clientbound::projection::{BorderProjection, BorderSize};
use crate::java_26_2::play::clientbound::world_border::packet::{
    SetBorderCenter, SetBorderLerpSize, SetBorderSize, SetBorderWarningDelay,
    SetBorderWarningDistance,
};

impl BorderProjection {
    pub fn apply_center(&mut self, packet: SetBorderCenter) {
        self.center_x = packet.center_x;
        self.center_z = packet.center_z;
    }

    pub fn apply_lerp(&mut self, packet: SetBorderLerpSize, current_client_game_time: i64) {
        self.size = if packet.old_size == packet.new_size {
            BorderSize::Immediate(packet.new_size)
        } else {
            BorderSize::Lerp {
                old_size: packet.old_size,
                new_size: packet.new_size,
                duration_millis: packet.duration_millis,
                begin_game_time: current_client_game_time,
            }
        };
    }

    pub fn apply_size(&mut self, packet: SetBorderSize) {
        self.size = BorderSize::Immediate(packet.size);
    }

    pub fn apply_warning_delay(&mut self, packet: SetBorderWarningDelay) {
        self.warning_time = packet.warning_time;
    }

    pub fn apply_warning_distance(&mut self, packet: SetBorderWarningDistance) {
        self.warning_blocks = packet.warning_blocks;
    }
}
