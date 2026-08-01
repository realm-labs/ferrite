use std::collections::BTreeMap;

use ferrite_gameplay::environment::weather::{WeatherPacket, WeatherPacketKind};
use ferrite_protocol::java_26_2::play::clientbound::packet::{
    ClockState, GameEvent, PlayClientboundPacket, SetTime,
};
use ferrite_protocol::java_26_2::value::identifier::{Identifier, IdentifierError};

use crate::world_service::environment::{EnvironmentProjection, LevelEnvironment};

pub(super) fn join_packets(
    environment: LevelEnvironment,
) -> Result<Vec<PlayClientboundPacket>, IdentifierError> {
    let mut packets = vec![time_packet(
        environment.game_time(),
        environment.day_time(),
    )?];
    let strengths = environment.strengths();
    if strengths.rain > 0.2 {
        packets.extend([
            game_event(2, 0.0),
            game_event(7, strengths.rain),
            game_event(8, strengths.thunder),
        ]);
    }
    Ok(packets)
}

pub(super) fn tick_packets(
    projection: &EnvironmentProjection,
) -> Result<Vec<PlayClientboundPacket>, IdentifierError> {
    let mut packets = vec![time_packet(projection.game_time, projection.day_time)?];
    packets.extend(projection.weather.iter().copied().map(weather_packet));
    Ok(packets)
}

fn time_packet(game_time: i64, day_time: i64) -> Result<PlayClientboundPacket, IdentifierError> {
    Ok(PlayClientboundPacket::SetTime(SetTime {
        game_time,
        clocks: BTreeMap::from([(
            Identifier::parse("minecraft:day_time")?,
            ClockState {
                total_ticks: day_time,
                partial_tick: 0.0,
                rate: 1.0,
            },
        )]),
    }))
}

fn weather_packet(packet: WeatherPacket) -> PlayClientboundPacket {
    match packet.kind {
        WeatherPacketKind::StartRaining => game_event(2, 0.0),
        WeatherPacketKind::StopRaining => game_event(1, 0.0),
        WeatherPacketKind::RainStrength(value) => game_event(7, value),
        WeatherPacketKind::ThunderStrength(value) => game_event(8, value),
    }
}

const fn game_event(event: u8, parameter: f32) -> PlayClientboundPacket {
    PlayClientboundPacket::GameEvent(GameEvent { event, parameter })
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::DimensionId;

    use super::*;

    #[test]
    fn join_and_tick_project_the_authoritative_clock() {
        let dimension = DimensionId::new("minecraft:overworld".parse().unwrap());
        let mut environment = LevelEnvironment::new(4, &dimension);
        let projection = environment.tick(&dimension).unwrap();
        let join = join_packets(environment).unwrap();
        let tick = tick_packets(&projection).unwrap();
        assert!(matches!(join[0], PlayClientboundPacket::SetTime(_)));
        assert_eq!(join[0], tick[0]);
    }
}
