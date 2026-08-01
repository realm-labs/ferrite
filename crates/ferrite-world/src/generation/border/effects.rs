//! Player damage, HUD warning, and force-field presentation formulas.

use super::{BorderPoint3, java_clamp, java_max, java_min};
use crate::generation::border::geometry::BorderAabb;
use crate::generation::border::state::WorldBorder;

pub const OUTSIDE_BORDER_DAMAGE_TYPE: &str = "minecraft:outside_border";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OutsideBorderDamageType {
    pub identifier: &'static str,
    pub exhaustion: f32,
    pub message_id: &'static str,
    pub scaling: &'static str,
    pub bypasses_armor: bool,
    pub bypasses_wolf_armor: bool,
    pub no_knockback: bool,
}

pub const OUTSIDE_BORDER_DAMAGE: OutsideBorderDamageType = OutsideBorderDamageType {
    identifier: OUTSIDE_BORDER_DAMAGE_TYPE,
    exhaustion: 0.0,
    message_id: "outsideBorder",
    scaling: "when_caused_by_living_non_player",
    bypasses_armor: true,
    bypasses_wolf_armor: true,
    no_knockback: true,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorderDamageInput {
    pub alive_living: bool,
    pub in_wall_hit: bool,
    pub is_player: bool,
    pub bounds: BorderAabb,
    pub center_x: f64,
    pub center_z: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BorderDamageDecision {
    SkippedDead,
    InWallPrecedence,
    NotPlayer,
    BoundsContained,
    SafeOrDisabled { outside_distance: f64 },
    Submit { amount: f32, outside_distance: f64 },
}

impl WorldBorder {
    pub fn damage_decision(&self, input: BorderDamageInput) -> BorderDamageDecision {
        if !input.alive_living {
            return BorderDamageDecision::SkippedDead;
        }
        if input.in_wall_hit {
            return BorderDamageDecision::InWallPrecedence;
        }
        if !input.is_player {
            return BorderDamageDecision::NotPlayer;
        }
        if self.contains_aabb(input.bounds) {
            return BorderDamageDecision::BoundsContained;
        }
        let outside_distance =
            self.distance_to_border(input.center_x, input.center_z) + self.safe_zone;
        if outside_distance >= 0.0 || self.damage_per_block <= 0.0 {
            return BorderDamageDecision::SafeOrDisabled { outside_distance };
        }
        let floored = (-outside_distance * self.damage_per_block).floor() as i32;
        let amount = floored.max(1) as f32;
        BorderDamageDecision::Submit {
            amount,
            outside_distance,
        }
    }

    pub fn hud_warning(&self, camera_x: f64, camera_z: f64) -> HudWarning {
        let distance = self.distance_to_border(camera_x, camera_z) as f32;
        let projected = java_min(
            self.extent.speed() * f64::from(self.warning_time),
            (self.target_size() - self.get_size()).abs(),
        );
        let threshold = java_max(f64::from(self.warning_blocks), projected) as f32;
        let intensity = if distance < threshold {
            (1.0 - distance / threshold).clamp(0.0, 1.0)
        } else {
            0.0
        };
        HudWarning {
            distance,
            projected,
            threshold,
            intensity,
        }
    }

    pub fn force_field_frame(
        &self,
        camera: BorderPoint3,
        partial_tick: f64,
        render_distance: f64,
    ) -> ForceFieldFrame {
        let edges = self.edges_at(partial_tick);
        let previous_distance = self.distance_to_border(camera.x, camera.z);
        let alpha = if previous_distance < render_distance {
            java_clamp(
                (1.0 - previous_distance / render_distance).powi(4),
                0.0,
                1.0,
            )
        } else {
            0.0
        };
        ForceFieldFrame {
            minimum_x: edges.minimum_x,
            maximum_x: edges.maximum_x,
            minimum_z: edges.minimum_z,
            maximum_z: edges.maximum_z,
            previous_distance,
            alpha,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HudWarning {
    pub distance: f32,
    pub projected: f64,
    pub threshold: f32,
    pub intensity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ForceFieldFrame {
    pub minimum_x: f64,
    pub maximum_x: f64,
    pub minimum_z: f64,
    pub maximum_z: f64,
    pub previous_distance: f64,
    pub alpha: f64,
}
