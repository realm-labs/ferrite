use thiserror::Error;

use crate::player::state::{PlayerPose, PlayerSessionState, Rotation, Vec3};

const MAGIC: [u8; 4] = *b"FPS1";
const ENCODED_LENGTH: usize = 4 + (15 * 8) + (2 * 4) + 1 + 4 + 1 + 4;

pub(super) fn encode(state: &PlayerSessionState) -> Vec<u8> {
    let mut output = Vec::with_capacity(ENCODED_LENGTH);
    output.extend_from_slice(&MAGIC);
    write_vec3(&mut output, state.pose.position);
    output.extend_from_slice(&state.pose.rotation.yaw.to_bits().to_be_bytes());
    output.extend_from_slice(&state.pose.rotation.pitch.to_bits().to_be_bytes());
    write_vec3(&mut output, state.first_good_position);
    write_vec3(&mut output, state.last_good_position);
    write_vec3(&mut output, state.velocity);
    write_vec3(&mut output, state.known_movement);
    let flags = state.on_ground as u8
        | ((state.horizontal_collision as u8) << 1)
        | ((state.movement_seen_this_client_interval as u8) << 2)
        | ((state.floating as u8) << 3);
    output.push(flags);
    output.extend_from_slice(&state.movement_packets_this_tick.to_be_bytes());
    output.push(state.client_load_ticks_remaining);
    output.extend_from_slice(&state.floating_ticks.to_be_bytes());
    output
}

pub(super) fn decode(bytes: &[u8]) -> Result<PlayerSessionState, PlayerStateCodecError> {
    if bytes.len() != ENCODED_LENGTH {
        return Err(PlayerStateCodecError::WrongLength {
            actual: bytes.len(),
            expected: ENCODED_LENGTH,
        });
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take::<4>() != MAGIC {
        return Err(PlayerStateCodecError::InvalidMagic);
    }
    let position = cursor.read_vec3();
    let rotation = Rotation {
        yaw: cursor.read_f32(),
        pitch: cursor.read_f32(),
    };
    let first_good_position = cursor.read_vec3();
    let last_good_position = cursor.read_vec3();
    let velocity = cursor.read_vec3();
    let known_movement = cursor.read_vec3();
    let flags = cursor.read_u8();
    if flags & 0xf0 != 0 {
        return Err(PlayerStateCodecError::UnknownFlags(flags));
    }
    let movement_packets_this_tick = cursor.read_u32();
    let client_load_ticks_remaining = cursor.read_u8();
    let floating_ticks = cursor.read_u32();
    Ok(PlayerSessionState {
        pose: PlayerPose::new(position, rotation),
        first_good_position,
        last_good_position,
        velocity,
        known_movement,
        on_ground: flags & 0x01 != 0,
        horizontal_collision: flags & 0x02 != 0,
        movement_packets_this_tick,
        movement_seen_this_client_interval: flags & 0x04 != 0,
        client_load_ticks_remaining,
        floating: flags & 0x08 != 0,
        floating_ticks,
    })
}

fn write_vec3(output: &mut Vec<u8>, vector: Vec3) {
    for value in [vector.x, vector.y, vector.z] {
        output.extend_from_slice(&value.to_bits().to_be_bytes());
    }
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take<const N: usize>(&mut self) -> [u8; N] {
        let end = self.offset + N;
        let value = self.bytes[self.offset..end]
            .try_into()
            .expect("the transfer length is checked before decoding");
        self.offset = end;
        value
    }

    fn read_u8(&mut self) -> u8 {
        self.take::<1>()[0]
    }

    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.take())
    }

    fn read_f32(&mut self) -> f32 {
        f32::from_bits(self.read_u32())
    }

    fn read_f64(&mut self) -> f64 {
        f64::from_bits(u64::from_be_bytes(self.take()))
    }

    fn read_vec3(&mut self) -> Vec3 {
        Vec3::new(self.read_f64(), self.read_f64(), self.read_f64())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PlayerStateCodecError {
    #[error("player transfer state has {actual} bytes, expected {expected}")]
    WrongLength { actual: usize, expected: usize },
    #[error("player transfer state has an invalid magic prefix")]
    InvalidMagic,
    #[error("player transfer state has unknown flags {0:#04x}")]
    UnknownFlags(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_state_round_trips_every_session_field() {
        let mut state = PlayerSessionState::new(PlayerPose::new(
            Vec3::new(-1.5, 65.0, f64::INFINITY),
            Rotation {
                yaw: -180.0,
                pitch: 12.5,
            },
        ));
        state.accept_player_loaded();
        state.set_velocity(Vec3::new(1.0, 2.0, 3.0));
        state.accept_movement(
            PlayerPose::new(
                Vec3::new(4.0, 5.0, 6.0),
                Rotation {
                    yaw: 7.0,
                    pitch: 8.0,
                },
            ),
            Vec3::new(0.25, -0.5, 0.75),
            true,
            true,
            true,
        );
        state.movement_packets_this_tick = 4;
        state.floating_ticks = 9;
        let encoded = encode(&state);
        assert_eq!(encoded.len(), ENCODED_LENGTH);
        assert_eq!(decode(&encoded).unwrap(), state);
    }

    #[test]
    fn transfer_state_rejects_wrong_length_magic_and_flags() {
        let state = PlayerSessionState::new(PlayerPose::default());
        let mut encoded = encode(&state);
        assert!(decode(&encoded[..encoded.len() - 1]).is_err());
        encoded[0] ^= 1;
        assert_eq!(decode(&encoded), Err(PlayerStateCodecError::InvalidMagic));
        let mut encoded = encode(&state);
        encoded[132] = 0x80;
        assert_eq!(
            decode(&encoded),
            Err(PlayerStateCodecError::UnknownFlags(0x80))
        );
    }
}
