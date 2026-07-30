use crate::java_26_2::play::clientbound::packet::Vector3;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq)]
pub struct AddEntity {
    pub entity_id: i32,
    pub uuid: u128,
    pub entity_type: Identifier,
    pub position: Vector3,
    pub motion: Vector3,
    pub pitch: i8,
    pub yaw: i8,
    pub head_yaw: i8,
    pub data: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveEntities {
    pub entity_ids: Vec<i32>,
}
