use crate::java_26_2::play::clientbound::entity_effects::packet::SoundEventHolder;
use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundSource {
    Master,
    Music,
    Records,
    Weather,
    Blocks,
    Hostile,
    Neutral,
    Players,
    Ambient,
    Voice,
    Ui,
}

impl SoundSource {
    #[must_use]
    pub const fn from_id(raw_id: i32) -> Option<Self> {
        match raw_id {
            0 => Some(Self::Master),
            1 => Some(Self::Music),
            2 => Some(Self::Records),
            3 => Some(Self::Weather),
            4 => Some(Self::Blocks),
            5 => Some(Self::Hostile),
            6 => Some(Self::Neutral),
            7 => Some(Self::Players),
            8 => Some(Self::Ambient),
            9 => Some(Self::Voice),
            10 => Some(Self::Ui),
            _ => None,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundAtPosition {
    pub sound: SoundEventHolder,
    pub source: SoundSource,
    pub encoded_position: [i32; 3],
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

impl SoundAtPosition {
    #[must_use]
    pub fn new(
        sound: SoundEventHolder,
        source: SoundSource,
        position: Vector3,
        volume: f32,
        pitch: f32,
        seed: i64,
    ) -> Self {
        Self {
            sound,
            source,
            encoded_position: [
                java_sound_coordinate(position.x),
                java_sound_coordinate(position.y),
                java_sound_coordinate(position.z),
            ],
            volume,
            pitch,
            seed,
        }
    }

    #[must_use]
    pub fn position(&self) -> Vector3 {
        Vector3 {
            x: f64::from(self.encoded_position[0] as f32 / 8.0),
            y: f64::from(self.encoded_position[1] as f32 / 8.0),
            z: f64::from(self.encoded_position[2] as f32 / 8.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SoundAtEntity {
    pub sound: SoundEventHolder,
    pub source: SoundSource,
    pub entity_id: i32,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopSound {
    pub source: Option<SoundSource>,
    pub sound: Option<Identifier>,
}

fn java_sound_coordinate(value: f64) -> i32 {
    (value * 8.0) as i32
}
