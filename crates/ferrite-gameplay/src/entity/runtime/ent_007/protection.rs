//! Death-protection hand selection, server effects, and client event presentation.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Main,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionStack {
    pub present: bool,
    pub has_death_protection: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionSelection {
    pub hand: Option<Hand>,
    pub copy_full_stack_before_shrink: bool,
    pub shrink_held_by: u8,
    pub inspected_hands: u8,
}

#[must_use]
pub const fn select_protection(
    bypasses_invulnerability: bool,
    main: ProtectionStack,
    off: ProtectionStack,
) -> ProtectionSelection {
    if bypasses_invulnerability {
        return ProtectionSelection {
            hand: None,
            copy_full_stack_before_shrink: false,
            shrink_held_by: 0,
            inspected_hands: 0,
        };
    }
    if main.present && main.has_death_protection {
        ProtectionSelection {
            hand: Some(Hand::Main),
            copy_full_stack_before_shrink: true,
            shrink_held_by: 1,
            inspected_hands: 1,
        }
    } else if off.present && off.has_death_protection {
        ProtectionSelection {
            hand: Some(Hand::Off),
            copy_full_stack_before_shrink: true,
            shrink_held_by: 1,
            inspected_hands: 2,
        }
    } else {
        ProtectionSelection {
            hand: None,
            copy_full_stack_before_shrink: false,
            shrink_held_by: 0,
            inspected_hands: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtectionStage {
    AwardItemUsed,
    TriggerUsedTotem,
    FinishInteractionVibration,
    SetHealthOne,
    ClearEffects,
    ConsumeApplyEffectsDraw,
    AddRegeneration,
    AddAbsorption,
    AddFireResistance,
    BroadcastEvent35,
}

pub const PLAYER_TOTEM_ORDER: [ProtectionStage; 10] = [
    ProtectionStage::AwardItemUsed,
    ProtectionStage::TriggerUsedTotem,
    ProtectionStage::FinishInteractionVibration,
    ProtectionStage::SetHealthOne,
    ProtectionStage::ClearEffects,
    ProtectionStage::ConsumeApplyEffectsDraw,
    ProtectionStage::AddRegeneration,
    ProtectionStage::AddAbsorption,
    ProtectionStage::AddFireResistance,
    ProtectionStage::BroadcastEvent35,
];

pub const NONPLAYER_TOTEM_ORDER: [ProtectionStage; 7] = [
    ProtectionStage::SetHealthOne,
    ProtectionStage::ClearEffects,
    ProtectionStage::ConsumeApplyEffectsDraw,
    ProtectionStage::AddRegeneration,
    ProtectionStage::AddAbsorption,
    ProtectionStage::AddFireResistance,
    ProtectionStage::BroadcastEvent35,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotemEffect {
    pub amplifier: u8,
    pub duration: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TotemProtection {
    pub protected: bool,
    pub health: f32,
    pub clear_all_effects: bool,
    pub apply_draw_consumed: bool,
    pub regeneration: Option<TotemEffect>,
    pub absorption: Option<TotemEffect>,
    pub fire_resistance: Option<TotemEffect>,
    pub event: Option<u8>,
}

#[must_use]
pub const fn totem_protection(selection: ProtectionSelection) -> TotemProtection {
    let protected = selection.hand.is_some();
    TotemProtection {
        protected,
        health: if protected { 1.0 } else { 0.0 },
        clear_all_effects: protected,
        apply_draw_consumed: protected,
        regeneration: if protected {
            Some(TotemEffect {
                amplifier: 1,
                duration: 900,
            })
        } else {
            None
        },
        absorption: if protected {
            Some(TotemEffect {
                amplifier: 1,
                duration: 100,
            })
        } else {
            None
        },
        fire_resistance: if protected {
            Some(TotemEffect {
                amplifier: 0,
                duration: 800,
            })
        } else {
            None
        },
        event: if protected { Some(35) } else { None },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientTotemEvent {
    pub emitter_ticks: u8,
    pub play_local_sound: bool,
    pub display_activation: bool,
    pub display_hand: Option<Hand>,
    pub construct_fallback_totem: bool,
}

#[must_use]
pub const fn client_totem_event(
    local_player_entity: bool,
    current_main_has_protection: bool,
    current_off_has_protection: bool,
) -> ClientTotemEvent {
    let display_hand = if !local_player_entity {
        None
    } else if current_main_has_protection {
        Some(Hand::Main)
    } else if current_off_has_protection {
        Some(Hand::Off)
    } else {
        None
    };
    ClientTotemEvent {
        emitter_ticks: 30,
        play_local_sound: true,
        display_activation: local_player_entity,
        display_hand,
        construct_fallback_totem: local_player_entity && display_hand.is_none(),
    }
}
