//! Cross-crate repeated combat-kill presentation fixture.

use ferrite_protocol::java_26_2::play::clientbound::combat_look::packet::{
    LookPosition, PlayerCombatKill,
};
use ferrite_protocol::java_26_2::play::clientbound::combat_look::projection::{
    CombatLookAction, CombatLookClientProjection, TrackedEntityPosition,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombatRuleReport {
    pub repeated_death_screens: usize,
    pub repeated_respawn_requests: u32,
    pub repeated_toggle_resets: u32,
    pub missing_local_ignored: bool,
}

pub fn run_combat_rule_projection() -> CombatRuleReport {
    let local = TrackedEntityPosition {
        feet: LookPosition {
            x: 0.0,
            y: 64.0,
            z: 0.0,
        },
        eye_height: 1.62,
        current_local_player: true,
    };
    let packet = PlayClientboundPacket::PlayerCombatKill(PlayerCombatKill {
        player_entity_id: 7,
        message: TextComponentNbt::literal("defeated").expect("static message"),
    });

    let mut screens = CombatLookClientProjection::new(7, local, true, true, true);
    let repeated_death_screens = (0..2)
        .filter(|_| {
            matches!(
                screens.apply(&packet),
                CombatLookAction::DeathScreenInstalled(_)
            )
        })
        .count();

    let mut respawns = CombatLookClientProjection::new(7, local, false, false, true);
    for _ in 0..2 {
        assert_eq!(
            respawns.apply(&packet),
            CombatLookAction::RespawnRequestedAndToggleKeysReset
        );
    }

    let mut missing = CombatLookClientProjection::new(7, local, true, false, true);
    missing.remove_entity(7);
    let missing_local_ignored = missing.apply(&packet) == CombatLookAction::Ignored;

    CombatRuleReport {
        repeated_death_screens,
        repeated_respawn_requests: respawns.respawn_requests(),
        repeated_toggle_resets: respawns.toggle_key_resets(),
        missing_local_ignored,
    }
}
