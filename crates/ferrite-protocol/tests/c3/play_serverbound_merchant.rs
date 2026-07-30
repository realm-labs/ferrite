use ferrite_protocol::java_26_2::play::clientbound::merchant::packet::{ItemCost, MerchantOffer};
use ferrite_protocol::java_26_2::play::item::{
    DataComponentPatch, EncodedComponentValue, ItemStack,
};
use ferrite_protocol::java_26_2::play::serverbound::codec::{decode_packet, encode_packet};
use ferrite_protocol::java_26_2::play::serverbound::merchant::packet::SelectTrade;
use ferrite_protocol::java_26_2::play::serverbound::merchant::transaction::{
    MerchantMenuTransaction, MerchantSelectionOutcome, MerchantSelectionStep, handle_select_trade,
    predict_trade_selection,
};
use ferrite_protocol::java_26_2::play::serverbound::packet::PlayServerboundEntryPacket;
use ferrite_protocol::java_26_2::value::identifier::Identifier;

fn id(value: &str) -> Identifier {
    Identifier::parse(value).unwrap()
}

fn component(name: &str, value: &[u8]) -> EncodedComponentValue {
    EncodedComponentValue {
        component: id(name),
        encoded_value: value.to_vec(),
    }
}

fn stack(item: &str, count: i32, components: Vec<EncodedComponentValue>) -> ItemStack {
    ItemStack::present(
        id(item),
        count,
        DataComponentPatch {
            added: components,
            removed: Vec::new(),
        },
    )
}

fn cost(item: &str, count: i32, components: Vec<EncodedComponentValue>) -> ItemCost {
    ItemCost {
        item: id(item),
        count,
        components,
    }
}

fn offer(
    cost_a: ItemCost,
    cost_b: Option<ItemCost>,
    result: ItemStack,
    experience: i32,
) -> MerchantOffer {
    MerchantOffer {
        cost_a,
        result,
        cost_b,
        uses: 0,
        max_uses: 10,
        experience,
        special_price_difference: 0,
        price_multiplier: 0.0,
        demand: 0,
        reward_experience: true,
    }
}

fn wrapped(selection_hint: i32) -> PlayServerboundEntryPacket {
    PlayServerboundEntryPacket::SelectTrade(SelectTrade { selection_hint })
}

#[test]
fn c3_gold_serverbound_merchant_locks_select_trade() {
    let packet = wrapped(0);
    assert_eq!(encode_packet(packet.clone()).unwrap(), [0x33, 0x00]);
    assert_eq!(decode_packet(&[0x33, 0x00]).unwrap(), packet);
}

#[test]
fn c3_merchant_codecs_preserve_every_signed_hint_and_fault_malformed_forms() {
    for selection_hint in [i32::MIN, -1, 0, 1, i32::MAX] {
        let packet = wrapped(selection_hint);
        assert_eq!(
            decode_packet(&encode_packet(packet.clone()).unwrap()).unwrap(),
            packet
        );
    }
    assert!(decode_packet(&[0x33]).is_err());
    assert!(decode_packet(&[0x33, 0x80]).is_err());
    assert!(decode_packet(&[0x33, 0x80, 0x80, 0x80, 0x80, 0x80, 0x00]).is_err());
    assert!(decode_packet(&[0x33, 0x00, 0x00]).is_err());
}

#[test]
fn c3_merchant_selection_admission_requires_only_current_valid_menu() {
    let packet = SelectTrade { selection_hint: -7 };
    assert_eq!(
        handle_select_trade(None, packet),
        (MerchantSelectionOutcome::IgnoredWrongMenu, None)
    );
    let mut invalid = MerchantMenuTransaction::new(false, Vec::new());
    assert_eq!(
        handle_select_trade(Some(&mut invalid), packet),
        (MerchantSelectionOutcome::IgnoredInvalidMenu, None)
    );
    assert_eq!(invalid.selection_hint, 0);

    let mut current = MerchantMenuTransaction::new(true, Vec::new());
    current.result = stack("minecraft:emerald", 1, Vec::new());
    current.future_experience = 11;
    current.payment_a = stack("minecraft:stone", 1, Vec::new());
    let (outcome, trace) = handle_select_trade(Some(&mut current), packet);
    assert_eq!(outcome, MerchantSelectionOutcome::Applied);
    assert_eq!(current.selection_hint, -7);
    assert_eq!(
        trace.unwrap().steps[0],
        MerchantSelectionStep::StoredHint(-7)
    );
    assert_eq!(
        current.result,
        stack("minecraft:emerald", 1, Vec::new()),
        "a nonempty-input empty-offer menu retains its stale result"
    );
    assert_eq!(current.future_experience, 11);
    assert_eq!(current.merchant_notifications.len(), 1);
}

#[test]
fn c3_merchant_selection_lookup_reproduces_hint_scan_swap_and_stock_rules() {
    let apple = offer(
        cost("minecraft:apple", 1, Vec::new()),
        None,
        stack("minecraft:emerald", 1, Vec::new()),
        2,
    );
    let carrot = offer(
        cost("minecraft:carrot", 1, Vec::new()),
        None,
        stack("minecraft:diamond", 1, Vec::new()),
        5,
    );
    let mut menu = MerchantMenuTransaction::new(true, vec![apple.clone(), carrot.clone()]);
    menu.payment_a = stack("minecraft:carrot", 1, Vec::new());
    menu.selection_hint = 0;
    menu.recompute_result();
    assert_eq!(
        menu.active_offer,
        Some(1),
        "hint zero scans from offer zero"
    );
    assert_eq!(menu.future_experience, 5);

    menu.selection_hint = 1;
    menu.payment_a = stack("minecraft:apple", 1, Vec::new());
    menu.recompute_result();
    assert_eq!(
        menu.active_offer, None,
        "a positive in-range mismatch does not fall back"
    );
    for selection_hint in [-1, 2, i32::MAX] {
        menu.selection_hint = selection_hint;
        menu.recompute_result();
        assert_eq!(menu.active_offer, Some(0));
    }

    let two_cost = offer(
        cost("minecraft:apple", 1, Vec::new()),
        Some(cost("minecraft:diamond", 1, Vec::new())),
        stack("minecraft:gold_ingot", 1, Vec::new()),
        9,
    );
    let mut swapped = MerchantMenuTransaction::new(true, vec![two_cost]);
    swapped.payment_a = stack("minecraft:diamond", 1, Vec::new());
    swapped.payment_b = stack("minecraft:apple", 1, Vec::new());
    swapped.recompute_result();
    assert_eq!(swapped.active_offer, Some(0));
    assert_eq!(swapped.future_experience, 9);

    swapped.offers[0].uses = swapped.offers[0].max_uses;
    swapped.recompute_result();
    assert_eq!(swapped.active_offer, None);
    assert!(swapped.result.is_empty());

    let notifications = swapped.merchant_notifications.len();
    swapped.payment_a = ItemStack::Empty;
    swapped.payment_b = ItemStack::Empty;
    swapped.recompute_result();
    assert!(swapped.result.is_empty());
    assert_eq!(
        swapped.merchant_notifications.len(),
        notifications,
        "entirely empty input returns before merchant notification"
    );
}

#[test]
fn c3_merchant_autofill_is_exact_non_atomic_and_uses_source_maximum() {
    let custom_name = component("minecraft:custom_name", &[1]);
    let custom_data = component("minecraft:custom_data", &[2]);
    let selected = offer(
        cost("minecraft:apple", 2, vec![custom_name.clone()]),
        None,
        stack("minecraft:emerald", 1, Vec::new()),
        3,
    );
    let mut menu = MerchantMenuTransaction::new(true, vec![selected]);
    menu.set_maximum_stack_size(id("minecraft:apple"), 16);
    menu.player_inventory[0] = stack(
        "minecraft:apple",
        10,
        vec![custom_name.clone(), custom_data],
    );
    menu.player_inventory[1] = stack("minecraft:apple", 10, vec![custom_name]);
    let trace = menu.apply_selection(0);
    assert_eq!(
        menu.payment_a.count(),
        10,
        "a cost of two fills to the first source stack's maximum domain"
    );
    assert_eq!(
        menu.player_inventory[1].count(),
        10,
        "a second predicate match with different complete components cannot merge into payment"
    );
    assert_eq!(menu.active_offer, Some(0));
    assert!(!trace.direct_response);

    let mut invalid = menu.clone();
    invalid.payment_a = stack("minecraft:stone", 2, Vec::new());
    invalid.player_inventory = vec![ItemStack::Empty; 36];
    invalid.apply_selection(-1);
    assert_eq!(
        invalid.payment_a,
        stack("minecraft:stone", 2, Vec::new()),
        "invalid hints recompute but never return or fill payments"
    );

    let mut partial = MerchantMenuTransaction::new(
        true,
        vec![offer(
            cost("minecraft:apple", 1, Vec::new()),
            None,
            stack("minecraft:emerald", 1, Vec::new()),
            1,
        )],
    );
    partial.player_inventory = vec![stack("minecraft:diamond", 64, Vec::new()); 36];
    partial.player_inventory[35] = stack("minecraft:stone", 63, Vec::new());
    partial.payment_a = stack("minecraft:stone", 2, Vec::new());
    partial.payment_b = stack("minecraft:apple", 1, Vec::new());
    let trace = partial.apply_selection(0);
    assert_eq!(partial.player_inventory[35].count(), 64);
    assert_eq!(partial.payment_a.count(), 1);
    assert_eq!(
        partial.payment_b.count(),
        1,
        "second-slot failure follows the already-partial first return"
    );
    assert!(
        trace
            .steps
            .contains(&MerchantSelectionStep::ReturnedPayment { slot: 0, moved: 1 })
    );
    assert!(
        trace
            .steps
            .contains(&MerchantSelectionStep::ReturnedPayment { slot: 1, moved: 0 })
    );
}

#[test]
fn c3_merchant_order_predicts_locally_before_the_tokenless_request() {
    let mut menu = MerchantMenuTransaction::new(
        true,
        vec![offer(
            cost("minecraft:apple", 1, Vec::new()),
            None,
            stack("minecraft:emerald", 1, Vec::new()),
            1,
        )],
    );
    menu.player_inventory[0] = stack("minecraft:apple", 4, Vec::new());
    let selection = predict_trade_selection(&mut menu, 0, 0);
    assert_eq!(selection.packet.selection_hint, 0);
    assert_eq!(
        selection.trace.steps.first(),
        Some(&MerchantSelectionStep::StoredHint(0))
    );
    assert_eq!(
        selection.trace.steps.last(),
        Some(&MerchantSelectionStep::SentRequest(0))
    );
    assert_eq!(menu.payment_a.count(), 4);
    assert!(!selection.trace.direct_response);
}

#[test]
fn c3_merchant_end_to_end_replays_prediction_through_current_server_menu() {
    let packet = wrapped(0);
    let encoded = encode_packet(packet).unwrap();
    let PlayServerboundEntryPacket::SelectTrade(packet) = decode_packet(&encoded).unwrap() else {
        panic!("select trade packet");
    };
    let offers = vec![offer(
        cost("minecraft:apple", 1, Vec::new()),
        None,
        stack("minecraft:emerald", 1, Vec::new()),
        4,
    )];
    let mut client = MerchantMenuTransaction::new(true, offers.clone());
    let mut server = MerchantMenuTransaction::new(true, offers);
    client.player_inventory[0] = stack("minecraft:apple", 3, Vec::new());
    server.player_inventory[0] = stack("minecraft:apple", 3, Vec::new());
    let predicted = predict_trade_selection(&mut client, 0, 0);
    assert_eq!(predicted.packet, packet);
    let (outcome, convergence) = handle_select_trade(Some(&mut server), packet);
    assert_eq!(outcome, MerchantSelectionOutcome::Applied);
    assert!(!convergence.unwrap().direct_response);
    assert_eq!(server.payment_a, client.payment_a);
    assert_eq!(server.result, client.result);
    assert_eq!(server.future_experience, client.future_experience);
}
