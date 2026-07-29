use crate::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, read_common_spawn, write_common_spawn,
};
use crate::java_26_2::play::clientbound::packet::CommonSpawnInfo;
use crate::java_26_2::play::registry::PlayRegistries;
use crate::java_26_2::wire::primitive::{WireReader, WireWriter};

#[derive(Debug, Clone, PartialEq)]
pub struct Respawn {
    pub spawn: CommonSpawnInfo,
    pub data_to_keep: i8,
}

impl Respawn {
    #[must_use]
    pub const fn retention(&self) -> RespawnRetention {
        RespawnRetention {
            attributes: self.data_to_keep & 0x01 != 0,
            entity_data: self.data_to_keep & 0x02 != 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RespawnRetention {
    pub attributes: bool,
    pub entity_data: bool,
}

pub(super) fn read(
    reader: &mut WireReader<'_>,
    registries: &PlayRegistries,
) -> Result<Respawn, PlayClientboundCodecError> {
    Ok(Respawn {
        spawn: read_common_spawn(reader, registries)?,
        data_to_keep: reader.read_i8()?,
    })
}

pub(super) fn write(
    writer: &mut WireWriter,
    packet: &Respawn,
    registries: &PlayRegistries,
) -> Result<(), PlayClientboundCodecError> {
    write_common_spawn(writer, &packet.spawn, registries)?;
    writer.write_i8(packet.data_to_keep)?;
    Ok(())
}
