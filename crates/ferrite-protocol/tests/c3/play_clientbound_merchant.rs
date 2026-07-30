use ferrite_protocol::java_26_2::play::clientbound::codec::{
    PlayClientboundCodecError, decode_packet, encode_packet,
};
use ferrite_protocol::java_26_2::play::clientbound::container::publication::MenuSnapshot;
use ferrite_protocol::java_26_2::play::clientbound::merchant::packet::{
    ItemCost, MerchantOffer, MerchantOffers,
};
use ferrite_protocol::java_26_2::play::clientbound::merchant::projection::{
    MerchantClientProjection, MerchantUpdate,
};
use ferrite_protocol::java_26_2::play::clientbound::merchant::publication::{
    MerchantPublisher, MerchantSnapshot,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::context::{
    ComponentValueDecoder, ComponentValueError, PlayDecodeContext,
};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack,
};
use ferrite_protocol::java_26_2::play::registry::{DATA_COMPONENT_TYPE, ITEM, PlayRegistries};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;
use ferrite_protocol::java_26_2::wire::primitive::WireReader;

struct OneByteComponent;

impl ComponentValueDecoder for OneByteComponent {
    fn decode_value(
        &self,
        component: &Identifier,
        reader: &mut WireReader<'_>,
    ) -> Result<Vec<u8>, ComponentValueError> {
        reader
            .read_u8()
            .map(|value| vec![value])
            .map_err(|error| ComponentValueError::Malformed {
                component: component.clone(),
                reason: error.to_string(),
            })
    }
}

static COMPONENTS: OneByteComponent = OneByteComponent;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn registries() -> PlayRegistries {
    let mut registries = PlayRegistries::default();
    registries.insert(
        id(ITEM),
        vec![
            id("minecraft:emerald"),
            id("minecraft:diamond"),
            id("minecraft:stone"),
            id("minecraft:air"),
        ],
    );
    registries.insert(
        id(DATA_COMPONENT_TYPE),
        vec![id("minecraft:custom_data"), id("minecraft:custom_name")],
    );
    registries
}

fn context(registries: &PlayRegistries) -> PlayDecodeContext<'_> {
    PlayDecodeContext {
        registries,
        component_values: &COMPONENTS,
        dimension_section_count: 24,
    }
}

fn component(identity: &str, value: u8) -> EncodedComponentValue {
    EncodedComponentValue {
        component: id(identity),
        encoded_value: vec![value],
    }
}

fn stack(identity: &str, count: i32, components: Vec<EncodedComponentValue>) -> ItemStack {
    ItemStack::present(
        id(identity),
        count,
        DataComponentPatch {
            added: components,
            removed: Vec::new(),
        },
    )
}

fn cost(identity: &str, count: i32) -> ItemCost {
    ItemCost {
        item: id(identity),
        count,
        components: Vec::new(),
    }
}

fn offer() -> MerchantOffer {
    MerchantOffer {
        cost_a: cost("minecraft:emerald", 3),
        result: stack("minecraft:diamond", 1, Vec::new()),
        cost_b: None,
        uses: 2,
        max_uses: 12,
        experience: 5,
        special_price_difference: -1,
        price_multiplier: 0.05,
        demand: 4,
        reward_experience: true,
    }
}

fn packet() -> MerchantOffers {
    MerchantOffers {
        container_id: 7,
        offers: vec![offer()],
        villager_level: 2,
        villager_experience: 10,
        show_progress: true,
        can_restock: false,
    }
}

fn wrapped(packet: MerchantOffers) -> PlayClientboundPacket {
    PlayClientboundPacket::MerchantOffers(packet)
}

fn merchant_menu() -> MenuSnapshot {
    MenuSnapshot {
        menu_type: id("minecraft:merchant"),
        title: TextComponentNbt::literal("Trader").unwrap(),
        slots: vec![ItemStack::Empty; 3],
        carried: ItemStack::Empty,
        data: vec![0],
    }
}

#[test]
fn c3_gold_clientbound_merchant_is_locked() {
    let registry = registries();
    let encoded = encode_packet(&wrapped(packet()), &registry).unwrap();
    assert_eq!(
        encoded,
        vec![
            0x34, 0x07, 0x01, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x02, 0x00, 0x00, 0x00, 0x0c, 0x00, 0x00, 0x00, 0x05, 0xff, 0xff, 0xff, 0xff,
            0x3d, 0x4c, 0xcc, 0xcd, 0x00, 0x00, 0x00, 0x04, 0x02, 0x0a, 0x01, 0x00,
        ]
    );
    assert_eq!(
        decode_packet(&encoded, context(&registry)).unwrap(),
        wrapped(packet())
    );
}

#[test]
fn c3_merchant_codecs_preserve_structured_values_and_reject_malformed_forms() {
    let registry = registries();
    let mut structured = packet();
    structured.container_id = -1;
    structured.villager_level = i32::MIN;
    structured.villager_experience = i32::MAX;
    structured.offers[0].cost_a.components = vec![
        component("minecraft:custom_data", 7),
        component("minecraft:custom_data", 7),
    ];
    structured.offers[0].result = stack(
        "minecraft:diamond",
        2,
        vec![component("minecraft:custom_name", 9)],
    );
    structured.offers[0].cost_b = Some(ItemCost {
        item: id("minecraft:stone"),
        count: -4,
        components: vec![component("minecraft:custom_data", 3)],
    });
    structured.offers[0].price_multiplier = f32::from_bits(0x7fc0_1234);
    let encoded = encode_packet(&wrapped(structured.clone()), &registry).unwrap();
    let PlayClientboundPacket::MerchantOffers(decoded) =
        decode_packet(&encoded, context(&registry)).unwrap()
    else {
        panic!("merchant packet expected");
    };
    assert_eq!(decoded.container_id, structured.container_id);
    assert_eq!(decoded.offers[0].cost_a, structured.offers[0].cost_a);
    assert_eq!(decoded.offers[0].result, structured.offers[0].result);
    assert_eq!(decoded.offers[0].cost_b, structured.offers[0].cost_b);
    assert_eq!(
        decoded.offers[0].price_multiplier.to_bits(),
        structured.offers[0].price_multiplier.to_bits()
    );
    assert!(matches!(
        decode_packet(
            &[0x34, 0x00, 0xff, 0xff, 0xff, 0xff, 0x0f],
            context(&registry)
        ),
        Err(PlayClientboundCodecError::Merchant(_))
    ));
    let mut empty = packet();
    empty.offers[0].result = ItemStack::Empty;
    assert!(matches!(
        encode_packet(&wrapped(empty), &registry),
        Err(PlayClientboundCodecError::Merchant(_))
    ));
}

#[test]
fn c3_merchant_cost_mapping_retains_duplicate_exact_predicates_and_allows_extras() {
    let expected = component("minecraft:custom_data", 7);
    let mut exact = ItemCost {
        item: id("minecraft:emerald"),
        count: 2,
        components: vec![expected.clone(), expected],
    };
    let candidate = stack(
        "minecraft:emerald",
        4,
        vec![
            component("minecraft:custom_data", 7),
            component("minecraft:custom_name", 9),
        ],
    );
    assert!(exact.matches(&candidate));
    assert!(exact.accepts_count(&candidate, 4));
    assert!(!exact.accepts_count(&candidate, 5));
    exact.components[1].encoded_value = vec![8];
    assert!(!exact.matches(&candidate));
    assert!(!exact.matches(&stack(
        "minecraft:stone",
        64,
        vec![component("minecraft:custom_data", 7)]
    )));
}

#[test]
fn c3_merchant_offer_decode_normalizes_stock_and_reward_experience() {
    let registry = registries();
    let mut raw = encode_packet(&wrapped(packet()), &registry).unwrap();
    raw[11] = 1;
    raw[15] = 7;
    let PlayClientboundPacket::MerchantOffers(decoded) =
        decode_packet(&raw, context(&registry)).unwrap()
    else {
        panic!("merchant packet expected");
    };
    assert_eq!(decoded.offers[0].uses, 12);
    assert!(decoded.offers[0].reward_experience);
    assert!(decoded.offers[0].is_out_of_stock());

    raw[11] = 0;
    raw[15] = 12;
    let PlayClientboundPacket::MerchantOffers(decoded) =
        decode_packet(&raw, context(&registry)).unwrap()
    else {
        panic!("merchant packet expected");
    };
    assert!(decoded.offers[0].is_out_of_stock());
    let canonical = encode_packet(&wrapped(decoded), &registry).unwrap();
    assert_eq!(canonical[11], 1);
}

#[test]
fn c3_merchant_offer_pricing_uses_java_wrapping_floor_and_clamp_boundaries() {
    let mut priced = offer();
    priced.cost_a.count = 5;
    priced.demand = 2;
    priced.price_multiplier = 0.2;
    priced.special_price_difference = -1;
    assert_eq!(priced.modified_cost_a_count(64), 6);

    priced.price_multiplier = f32::NAN;
    assert_eq!(priced.modified_cost_a_count(64), 4);
    priced.price_multiplier = f32::INFINITY;
    assert_eq!(priced.modified_cost_a_count(64), 1);
    priced.price_multiplier = f32::NEG_INFINITY;
    assert_eq!(priced.modified_cost_a_count(64), 1);

    priced.price_multiplier = 0.0;
    priced.special_price_difference = i32::MAX;
    assert_eq!(priced.modified_cost_a_count(64), 1);
    assert!(priced.satisfied_by(
        &stack("minecraft:emerald", 1, Vec::new()),
        &ItemStack::Empty,
        64
    ));
    assert_eq!(priced.assemble(), priced.result);
}

#[test]
fn c3_merchant_client_application_gates_menu_and_replaces_in_source_order() {
    let mut client = MerchantClientProjection::default();
    assert!(!client.apply(&wrapped(packet())).unwrap());
    client.open_menu(7, false);
    assert!(!client.apply(&wrapped(packet())).unwrap());
    client.open_menu(8, true);
    assert!(!client.apply(&wrapped(packet())).unwrap());
    client.open_menu(7, true);
    assert!(client.apply(&wrapped(packet())).unwrap());
    let menu = client.current().unwrap();
    assert_eq!(menu.offers, packet().offers);
    assert_eq!(menu.villager_experience, 10);
    assert_eq!(menu.villager_level, 2);
    assert_eq!(
        menu.last_update_order,
        vec![
            MerchantUpdate::Offers,
            MerchantUpdate::Experience,
            MerchantUpdate::Level,
            MerchantUpdate::ShowProgress,
            MerchantUpdate::CanRestock,
        ]
    );
}

#[test]
fn c3_merchant_order_opens_and_converges_before_optional_offer_projection() {
    let mut publisher = MerchantPublisher::default();
    let merchant = MerchantSnapshot {
        offers: vec![offer()],
        villager_level: 2,
        villager_experience: 10,
        show_progress: true,
        can_restock: true,
    };
    let packets = publisher.open_trading(merchant_menu(), &merchant).unwrap();
    assert!(matches!(packets[0], PlayClientboundPacket::OpenScreen(_)));
    assert!(matches!(
        packets[1],
        PlayClientboundPacket::ContainerSetContent(_)
    ));
    assert!(matches!(
        packets[2],
        PlayClientboundPacket::ContainerSetData(_)
    ));
    let PlayClientboundPacket::MerchantOffers(projected) = &packets[3] else {
        panic!("offer projection must follow initial menu convergence");
    };
    assert_eq!(projected.container_id, 1);
    assert_eq!(projected.offers, merchant.offers);

    let empty = MerchantSnapshot {
        offers: Vec::new(),
        ..merchant
    };
    let packets = publisher.open_trading(merchant_menu(), &empty).unwrap();
    assert!(
        packets
            .iter()
            .all(|packet| !matches!(packet, PlayClientboundPacket::MerchantOffers(_)))
    );
}

#[test]
fn c3_merchant_end_to_end_snapshot_is_owned_and_needs_no_acknowledgement() {
    let registry = registries();
    let mut source = MerchantSnapshot {
        offers: vec![offer()],
        villager_level: 3,
        villager_experience: 20,
        show_progress: true,
        can_restock: false,
    };
    let mut publisher = MerchantPublisher::default();
    let packets = publisher.open_trading(merchant_menu(), &source).unwrap();
    source.offers[0].uses = 99;
    let packet = packets.last().unwrap();
    let encoded = encode_packet(packet, &registry).unwrap();
    let decoded = decode_packet(&encoded, context(&registry)).unwrap();
    let mut client = MerchantClientProjection::default();
    client.open_menu(1, true);
    assert!(client.apply(&decoded).unwrap());
    assert_eq!(client.current().unwrap().offers[0].uses, 2);
    assert_eq!(client.current().unwrap().villager_level, 3);
}
