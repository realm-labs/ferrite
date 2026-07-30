use thiserror::Error;

use crate::java_26_2::play::registry::{MOB_EFFECT, PlayRegistries, PlayRegistryError};
use crate::java_26_2::play::serverbound::anvil_beacon::packet::SetBeacon;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconEffect {
    Speed,
    Haste,
    Resistance,
    JumpBoost,
    Strength,
    Regeneration,
}

impl BeaconEffect {
    #[must_use]
    pub const fn required_level(self) -> i32 {
        match self {
            Self::Speed | Self::Haste => 1,
            Self::Resistance | Self::JumpBoost => 2,
            Self::Strength => 3,
            Self::Regeneration => 4,
        }
    }

    #[must_use]
    pub fn identifier(self) -> Identifier {
        Identifier::minecraft(match self {
            Self::Speed => "speed",
            Self::Haste => "haste",
            Self::Resistance => "resistance",
            Self::JumpBoost => "jump_boost",
            Self::Strength => "strength",
            Self::Regeneration => "regeneration",
        })
        .expect("locked beacon effect identity is valid")
    }

    #[must_use]
    pub fn from_identifier(identifier: &Identifier) -> Option<Self> {
        if identifier.namespace() != "minecraft" {
            return None;
        }
        match identifier.path() {
            "speed" => Some(Self::Speed),
            "haste" => Some(Self::Haste),
            "resistance" => Some(Self::Resistance),
            "jump_boost" => Some(Self::JumpBoost),
            "strength" => Some(Self::Strength),
            "regeneration" => Some(Self::Regeneration),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconClientProjection {
    pub level: i32,
    pub payment_nonempty: bool,
    pub primary: Option<BeaconEffect>,
    pub secondary: Option<BeaconEffect>,
}

impl BeaconClientProjection {
    #[must_use]
    pub const fn new(level: i32, payment_nonempty: bool) -> Self {
        Self {
            level,
            payment_nonempty,
            primary: None,
            secondary: None,
        }
    }

    pub fn choose_primary(&mut self, effect: BeaconEffect) -> bool {
        let required = effect.required_level();
        if required >= 4 || required > self.level {
            return false;
        }
        self.primary = Some(effect);
        if self.secondary.is_some_and(|secondary| secondary != effect) {
            self.secondary = None;
        }
        true
    }

    pub fn choose_regeneration(&mut self) -> bool {
        if self.level < 4 {
            return false;
        }
        self.secondary = Some(BeaconEffect::Regeneration);
        true
    }

    pub fn choose_primary_upgrade(&mut self) -> bool {
        if self.level < 4 {
            return false;
        }
        let Some(primary) = self.primary else {
            return false;
        };
        self.secondary = Some(primary);
        true
    }

    #[must_use]
    pub fn done(&self) -> BeaconClientAction {
        let Some(primary) = self.primary else {
            return BeaconClientAction::Disabled;
        };
        if !self.payment_nonempty {
            return BeaconClientAction::Disabled;
        }
        BeaconClientAction::Emit(vec![
            BeaconClientEmission::SetBeacon(SetBeacon {
                primary: Some(primary.identifier()),
                secondary: self.secondary.map(BeaconEffect::identifier),
            }),
            BeaconClientEmission::CloseContainer,
        ])
    }

    #[must_use]
    pub fn cancel() -> BeaconClientAction {
        BeaconClientAction::Emit(vec![BeaconClientEmission::CloseContainer])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeaconClientAction {
    Disabled,
    Emit(Vec<BeaconClientEmission>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeaconClientEmission {
    SetBeacon(SetBeacon),
    CloseContainer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconMenuState {
    pub still_valid: bool,
    pub level: i32,
    pub payment_count: u32,
    pub primary: Option<BeaconEffect>,
    pub secondary: Option<BeaconEffect>,
    pub primary_data: i32,
    pub secondary_data: i32,
    pub beam_sections_nonempty: bool,
    pub selection_sounds: u64,
    pub chunk_unsaved: bool,
}

impl BeaconMenuState {
    #[must_use]
    pub const fn new(still_valid: bool, level: i32, payment_count: u32) -> Self {
        Self {
            still_valid,
            level,
            payment_count,
            primary: None,
            secondary: None,
            primary_data: 0,
            secondary_data: 0,
            beam_sections_nonempty: false,
            selection_sounds: 0,
            chunk_unsaved: false,
        }
    }

    #[must_use]
    pub const fn remaining_payment_on_close(&self) -> u32 {
        self.payment_count
    }
}

pub fn handle_set_beacon(
    current_beacon: Option<&mut BeaconMenuState>,
    packet: &SetBeacon,
    registries: &PlayRegistries,
) -> Result<BeaconCommitOutcome, BeaconAdmissionError> {
    let Some(menu) = current_beacon else {
        return Ok(BeaconCommitOutcome::IgnoredWrongMenu);
    };
    if !menu.still_valid {
        return Ok(BeaconCommitOutcome::IgnoredInvalidMenu);
    }
    if menu.payment_count == 0 {
        return Ok(BeaconCommitOutcome::DisconnectGeneric);
    }
    if !validate_effects(menu.level, packet)? {
        return Ok(BeaconCommitOutcome::DisconnectGeneric);
    }

    let primary_data = encode_menu_effect(packet.primary.as_ref(), registries)?;
    let secondary_data = encode_menu_effect(packet.secondary.as_ref(), registries)?;
    menu.primary_data = primary_data;
    menu.primary = packet
        .primary
        .as_ref()
        .and_then(BeaconEffect::from_identifier);
    if menu.beam_sections_nonempty {
        menu.selection_sounds = menu.selection_sounds.wrapping_add(1);
    }
    menu.secondary_data = secondary_data;
    menu.secondary = packet
        .secondary
        .as_ref()
        .and_then(BeaconEffect::from_identifier);
    menu.payment_count -= 1;
    menu.chunk_unsaved = true;

    Ok(BeaconCommitOutcome::Applied(BeaconCommit {
        data_writes: [
            BeaconDataWrite {
                field: BeaconDataField::Primary,
                value: primary_data,
            },
            BeaconDataWrite {
                field: BeaconDataField::Secondary,
                value: secondary_data,
            },
        ],
        primary: menu.primary,
        secondary: menu.secondary,
        remaining_payment: menu.payment_count,
        played_selection_sound: menu.beam_sections_nonempty,
        chunk_unsaved: true,
    }))
}

fn validate_effects(level: i32, packet: &SetBeacon) -> Result<bool, BeaconAdmissionError> {
    if packet.secondary.is_some() && level < 4 {
        return Ok(false);
    }
    let primary_level = packet
        .primary
        .as_ref()
        .map_or(0, required_level_for_identifier);
    let secondary_level = packet
        .secondary
        .as_ref()
        .map_or(0, required_level_for_identifier);
    if primary_level > level || secondary_level > level || primary_level >= 4 {
        return Ok(false);
    }
    if (1..=3).contains(&secondary_level) {
        let primary = packet
            .primary
            .as_ref()
            .ok_or(BeaconAdmissionError::NullPrimaryEquality)?;
        if packet.secondary.as_ref() != Some(primary) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn required_level_for_identifier(identifier: &Identifier) -> i32 {
    BeaconEffect::from_identifier(identifier).map_or(i32::MAX, BeaconEffect::required_level)
}

fn encode_menu_effect(
    effect: Option<&Identifier>,
    registries: &PlayRegistries,
) -> Result<i32, BeaconAdmissionError> {
    effect.map_or(Ok(0), |effect| {
        registries
            .raw_id(MOB_EFFECT, effect)?
            .checked_add(1)
            .ok_or(BeaconAdmissionError::MenuDataOverflow)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BeaconCommitOutcome {
    IgnoredWrongMenu,
    IgnoredInvalidMenu,
    DisconnectGeneric,
    Applied(BeaconCommit),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeaconCommit {
    pub data_writes: [BeaconDataWrite; 2],
    pub primary: Option<BeaconEffect>,
    pub secondary: Option<BeaconEffect>,
    pub remaining_payment: u32,
    pub played_selection_sound: bool,
    pub chunk_unsaved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeaconDataWrite {
    pub field: BeaconDataField,
    pub value: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeaconDataField {
    Primary,
    Secondary,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BeaconAdmissionError {
    #[error("beacon secondary equality dereferenced an absent primary")]
    NullPrimaryEquality,
    #[error(transparent)]
    Registry(#[from] PlayRegistryError),
    #[error("built-in mob-effect ID plus one exceeds signed menu-data range")]
    MenuDataOverflow,
}
