use ferrite_protocol::java_26_2::catalog::{ConnectionState, PacketCatalog, PacketDirection};
use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet as decode_required_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::live_tags::gate::{
    LiveTagsContext, LiveTagsDecision, LiveTagsEffect, LiveTagsGates,
};
use ferrite_protocol::java_26_2::play::clientbound::live_tags::packet::{
    LiveTagReloadStep, LiveTagsPacketKind,
};
use ferrite_protocol::java_26_2::play::clientbound::live_tags::resolution::resolve_members;
use ferrite_protocol::java_26_2::play::context::{PlayDecodeContext, RejectComponentValues};
use ferrite_protocol::java_26_2::play::registry::PlayRegistries;
use ferrite_protocol::java_26_2::wire::compression::{CompressionMode, encode_packet};
use ferrite_protocol::java_26_2::wire::frame::FrameLimits;
use ferrite_protocol::java_26_2::wire::primitive::WireWriter;

fn context() -> LiveTagsContext {
    LiveTagsContext {
        service_registered: true,
        reload_committed: true,
        remote_connection: true,
        all_registries_prepared: true,
    }
}

#[test]
fn c4_live_tags_inventory_locks_the_play_catalog_entry() {
    assert_eq!(LiveTagsPacketKind::ALL, [LiveTagsPacketKind::UpdateTags]);
    let packet = LiveTagsPacketKind::UpdateTags;
    let descriptor = PacketCatalog::by_wire_id(
        ConnectionState::Play,
        PacketDirection::Clientbound,
        packet.wire_id(),
    )
    .unwrap();
    assert_eq!(descriptor.identity(), packet.identity());
}

#[test]
fn c4_gold_update_tags_empty_locks_configuration_identical_map_grammar() {
    let mut writer = WireWriter::new(3);
    writer
        .write_var_i32(LiveTagsPacketKind::UpdateTags.wire_id())
        .unwrap();
    writer.write_var_i32(0).unwrap();
    assert_eq!(writer.as_slice(), [0x86, 0x01, 0x00]);
    assert_eq!(
        encode_packet(
            writer.as_slice(),
            CompressionMode::enabled(256).unwrap(),
            FrameLimits::default(),
        )
        .unwrap(),
        [0x04, 0x00, 0x86, 0x01, 0x00]
    );
}

#[test]
fn c4_live_tags_default_gate_omits_the_optional_packet() {
    assert_eq!(
        LiveTagsGates::default().decide(context()),
        LiveTagsDecision::OmitDisabled(LiveTagsPacketKind::UpdateTags)
    );
}

#[test]
fn c4_live_tags_enabled_gate_degrades_without_registered_service() {
    let gates = LiveTagsGates { live_reload: true };
    assert_eq!(
        gates.decide(LiveTagsContext {
            service_registered: false,
            ..context()
        }),
        LiveTagsDecision::DegradeServiceUnavailable
    );
}

#[test]
fn c4_live_tags_requires_committed_remote_all_registry_preparation() {
    let gates = LiveTagsGates { live_reload: true };
    assert_eq!(
        gates.decide(LiveTagsContext {
            reload_committed: false,
            ..context()
        }),
        LiveTagsDecision::OmitUncommittedReload
    );
    assert_eq!(
        gates.decide(LiveTagsContext {
            remote_connection: false,
            ..context()
        }),
        LiveTagsDecision::OmitInMemoryConnection
    );
    assert_eq!(
        gates.decide(LiveTagsContext {
            all_registries_prepared: false,
            ..context()
        }),
        LiveTagsDecision::PreserveExistingBindings
    );
}

#[test]
fn c4_live_tag_resolution_filters_invalid_ids_and_preserves_valid_order_duplicates() {
    assert_eq!(resolve_members(&[-1, 2, 0, 2, i32::MAX, 3], 3), [2, 0, 2]);
    assert!(resolve_members(&[0, 1], 0).is_empty());
}

#[test]
fn c4_live_tags_orders_tags_before_recipes_and_emits_only_adapter_local_effects() {
    assert_eq!(
        LiveTagReloadStep::ORDER,
        [
            LiveTagReloadStep::Tags,
            LiveTagReloadStep::Recipes,
            LiveTagReloadStep::RecipeBook,
        ]
    );
    assert_eq!(
        LiveTagsGates { live_reload: true }.decide(context()),
        LiveTagsDecision::Emit(LiveTagsEffect::ReplaceBindingsThenRefreshFuelAndSearchTrees)
    );
}

#[test]
fn c4_live_tags_required_play_decoder_remains_fail_closed() {
    let registries = PlayRegistries::default();
    let values = RejectComponentValues;
    let body = [0x86, 0x01, 0x00];
    assert!(matches!(
        decode_required_packet(
            &body,
            PlayDecodeContext {
                registries: &registries,
                component_values: &values,
                dimension_section_count: 24,
            },
        ),
        Err(PlayClientboundCodecError::UnsupportedPacketIdentity {
            identity: "minecraft:update_tags"
        })
    ));
}
