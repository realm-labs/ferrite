use ferrite_protocol::java_26_2::play::serverbound::packet::{
    AcceptTeleportation, KeepAlive, PlayServerboundEntryPacket, PlayerInput, Pong,
};
use ferrite_server_runtime::player::dispatch::{
    ServerboundDisposition, ServerboundResponsibility, classify_serverbound,
};

#[test]
fn protocol_application_and_future_packets_have_explicit_default_outcomes() {
    for packet in [
        PlayServerboundEntryPacket::AcceptTeleportation(AcceptTeleportation { challenge: 1 }),
        PlayServerboundEntryPacket::KeepAlive(KeepAlive { challenge: 2 }),
        PlayServerboundEntryPacket::PlayerLoaded,
    ] {
        assert_eq!(
            classify_serverbound(&packet).disposition(),
            ServerboundDisposition::Handled
        );
    }

    let player_input = classify_serverbound(&PlayServerboundEntryPacket::PlayerInput(
        PlayerInput::default(),
    ));
    assert_eq!(
        player_input.responsibility(),
        ServerboundResponsibility::PlayerModeAndInput
    );
    assert_eq!(
        player_input.disposition(),
        ServerboundDisposition::Unsupported
    );

    let pong = classify_serverbound(&PlayServerboundEntryPacket::Pong(Pong { payload: 3 }));
    assert_eq!(pong.responsibility(), ServerboundResponsibility::Pong);
    assert_eq!(pong.disposition(), ServerboundDisposition::Unsupported);
}
