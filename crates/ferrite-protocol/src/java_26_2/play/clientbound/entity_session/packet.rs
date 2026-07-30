use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Animate {
    pub entity_id: i32,
    pub action: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageEvent {
    pub entity_id: i32,
    pub damage_type: Identifier,
    pub cause_entity_id: i32,
    pub direct_entity_id: i32,
    pub source_position: Option<Vector3>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HurtAnimation {
    pub entity_id: i32,
    pub yaw: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetCamera {
    pub entity_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TakeItemEntity {
    pub source_entity_id: i32,
    pub collector_entity_id: i32,
    pub amount: i32,
}
