use ferrite_foundation::identity::{StableEntityId, StableIdError};
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_foundation::resource::{ResourceId, ResourceIdError};
use ferrite_protocol::semantic::{
    ChatVisibility, ClientSettings, MainHand, ParticleStatus, SessionId, SessionIdError,
    SessionIdentity,
};
use ferrite_simulation::command::{CommandError, CommandSource, RegionCommand};
use ferrite_simulation::tick::GameTick;
use thiserror::Error;

const JOIN_PAYLOAD_MAGIC: [u8; 4] = *b"FSJ1";
const MAX_SEMANTIC_STRING_BYTES: usize = u16::MAX as usize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionJoinPayload {
    pub session: SessionId,
    pub player: StableEntityId,
    pub identity: SessionIdentity,
    pub settings: ClientSettings,
    pub transferred: bool,
}

impl SessionJoinPayload {
    pub fn encode(&self) -> Result<Vec<u8>, SessionCommandError> {
        let mut output = Vec::with_capacity(
            64usize
                .saturating_add(self.identity.name.len())
                .saturating_add(self.settings.language.len()),
        );
        output.extend_from_slice(&JOIN_PAYLOAD_MAGIC);
        output.extend_from_slice(&self.session.get().to_be_bytes());
        output.extend_from_slice(&self.player.to_be_bytes());
        output.extend_from_slice(&self.identity.profile_id.to_be_bytes());
        output.push(u8::from(self.transferred));
        write_string(&mut output, "profile name", &self.identity.name)?;
        write_string(&mut output, "client language", &self.settings.language)?;
        output.push(self.settings.view_distance as u8);
        output.push(chat_visibility_id(self.settings.chat_visibility));
        output.push(u8::from(self.settings.chat_colors));
        output.push(self.settings.model_customization);
        output.push(main_hand_id(self.settings.main_hand));
        output.push(u8::from(self.settings.text_filtering));
        output.push(u8::from(self.settings.allows_listing));
        output.push(particle_status_id(self.settings.particle_status));
        Ok(output)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SessionCommandError> {
        let mut cursor = Cursor::new(bytes);
        if cursor.take(4)? != JOIN_PAYLOAD_MAGIC {
            return Err(SessionCommandError::InvalidMagic);
        }
        let session = SessionId::new(cursor.read_u64()?)?;
        let player = StableEntityId::new(cursor.read_u128()?)?;
        let profile_id = cursor.read_u128()?;
        let transferred = cursor.read_bool()?;
        let name = cursor.read_string("profile name")?;
        let language = cursor.read_string("client language")?;
        let view_distance = cursor.read_u8()? as i8;
        let chat_visibility = chat_visibility_from_id(cursor.read_u8()?)?;
        let chat_colors = cursor.read_bool()?;
        let model_customization = cursor.read_u8()?;
        let main_hand = main_hand_from_id(cursor.read_u8()?)?;
        let text_filtering = cursor.read_bool()?;
        let allows_listing = cursor.read_bool()?;
        let particle_status = particle_status_from_id(cursor.read_u8()?)?;
        cursor.finish()?;
        Ok(Self {
            session,
            player,
            identity: SessionIdentity { profile_id, name },
            settings: ClientSettings {
                language,
                view_distance,
                chat_visibility,
                chat_colors,
                model_customization,
                main_hand,
                text_filtering,
                allows_listing,
                particle_status,
            },
            transferred,
        })
    }

    pub fn into_region_command(
        self,
        target: SimulationRegionKey,
        tick: GameTick,
        sequence: u64,
    ) -> Result<RegionCommand, SessionCommandError> {
        let player = self.player;
        let payload = self.encode()?;
        Ok(RegionCommand::new(
            target,
            tick,
            CommandSource::Player(player),
            sequence,
            ResourceId::new("ferrite", "session/join")?,
            payload,
        )?)
    }
}

#[derive(Debug, Error)]
pub enum SessionCommandError {
    #[error("session join payload has invalid magic")]
    InvalidMagic,
    #[error("session join payload ended before all fields were decoded")]
    Truncated,
    #[error("session join payload contains trailing bytes")]
    TrailingBytes,
    #[error("{field} contains {actual} UTF-8 bytes, exceeding {maximum}")]
    StringTooLong {
        field: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("{field} contains malformed UTF-8")]
    InvalidUtf8 { field: &'static str },
    #[error("session join payload boolean byte {value} is invalid")]
    InvalidBoolean { value: u8 },
    #[error("{kind} semantic enum ID {value} is invalid")]
    InvalidEnum { kind: &'static str, value: u8 },
    #[error(transparent)]
    SessionIdentity(#[from] SessionIdError),
    #[error(transparent)]
    StableIdentity(#[from] StableIdError),
    #[error(transparent)]
    ResourceIdentity(#[from] ResourceIdError),
    #[error(transparent)]
    Command(#[from] CommandError),
}

fn write_string(
    output: &mut Vec<u8>,
    field: &'static str,
    value: &str,
) -> Result<(), SessionCommandError> {
    let length = u16::try_from(value.len()).map_err(|_| SessionCommandError::StringTooLong {
        field,
        actual: value.len(),
        maximum: MAX_SEMANTIC_STRING_BYTES,
    })?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
    Ok(())
}

struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], SessionCommandError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(SessionCommandError::Truncated)?;
        let bytes = self
            .bytes
            .get(self.position..end)
            .ok_or(SessionCommandError::Truncated)?;
        self.position = end;
        Ok(bytes)
    }

    fn read_u8(&mut self) -> Result<u8, SessionCommandError> {
        Ok(self.take(1)?[0])
    }

    fn read_u64(&mut self) -> Result<u64, SessionCommandError> {
        let bytes = self
            .take(8)?
            .try_into()
            .map_err(|_| SessionCommandError::Truncated)?;
        Ok(u64::from_be_bytes(bytes))
    }

    fn read_u128(&mut self) -> Result<u128, SessionCommandError> {
        let bytes = self
            .take(16)?
            .try_into()
            .map_err(|_| SessionCommandError::Truncated)?;
        Ok(u128::from_be_bytes(bytes))
    }

    fn read_bool(&mut self) -> Result<bool, SessionCommandError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(SessionCommandError::InvalidBoolean { value }),
        }
    }

    fn read_string(&mut self, field: &'static str) -> Result<String, SessionCommandError> {
        let length = usize::from(u16::from_be_bytes(
            self.take(2)?
                .try_into()
                .map_err(|_| SessionCommandError::Truncated)?,
        ));
        let bytes = self.take(length)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| SessionCommandError::InvalidUtf8 { field })
    }

    fn finish(self) -> Result<(), SessionCommandError> {
        if self.position == self.bytes.len() {
            Ok(())
        } else {
            Err(SessionCommandError::TrailingBytes)
        }
    }
}

const fn chat_visibility_id(value: ChatVisibility) -> u8 {
    match value {
        ChatVisibility::Full => 0,
        ChatVisibility::System => 1,
        ChatVisibility::Hidden => 2,
    }
}

fn chat_visibility_from_id(value: u8) -> Result<ChatVisibility, SessionCommandError> {
    match value {
        0 => Ok(ChatVisibility::Full),
        1 => Ok(ChatVisibility::System),
        2 => Ok(ChatVisibility::Hidden),
        value => Err(SessionCommandError::InvalidEnum {
            kind: "chat visibility",
            value,
        }),
    }
}

const fn main_hand_id(value: MainHand) -> u8 {
    match value {
        MainHand::Left => 0,
        MainHand::Right => 1,
    }
}

fn main_hand_from_id(value: u8) -> Result<MainHand, SessionCommandError> {
    match value {
        0 => Ok(MainHand::Left),
        1 => Ok(MainHand::Right),
        value => Err(SessionCommandError::InvalidEnum {
            kind: "main hand",
            value,
        }),
    }
}

const fn particle_status_id(value: ParticleStatus) -> u8 {
    match value {
        ParticleStatus::All => 0,
        ParticleStatus::Decreased => 1,
        ParticleStatus::Minimal => 2,
    }
}

fn particle_status_from_id(value: u8) -> Result<ParticleStatus, SessionCommandError> {
    match value {
        0 => Ok(ParticleStatus::All),
        1 => Ok(ParticleStatus::Decreased),
        2 => Ok(ParticleStatus::Minimal),
        value => Err(SessionCommandError::InvalidEnum {
            kind: "particle status",
            value,
        }),
    }
}
