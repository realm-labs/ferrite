//! Cross-crate container prediction, convergence, and overwrite fixture.

use std::collections::BTreeMap;

use ferrite_protocol::java_26_2::play::clientbound::container::packet::{
    ContainerClose, ContainerSetContent, OpenScreen,
};
use ferrite_protocol::java_26_2::play::clientbound::container::projection::{
    ContainerClientProjection, ContainerProjectionAction, MenuDefinition,
};
use ferrite_protocol::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use ferrite_protocol::java_26_2::play::item::ItemStack;
use ferrite_protocol::java_26_2::play::serverbound::container::packet::ContainerInput;
use ferrite_protocol::java_26_2::play::serverbound::container::transaction::{
    ContainerActor, ContainerAuthoritativeState, ContainerClickOutcome, ContainerClientClick,
    ContainerClientMenu, ContainerMenuTransaction,
};
use ferrite_protocol::java_26_2::value::identifier::Identifier;
use ferrite_protocol::java_26_2::value::nbt::TextComponentNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuConvergenceReport {
    pub wrong_prediction_ignored: bool,
    pub prediction_count: u64,
    pub stale_click_executed: bool,
    pub stale_full_resync: bool,
    pub delayed_content_ignored: bool,
    pub close_abandoned_open_menu: bool,
}

pub fn run_menu_convergence() -> MenuConvergenceReport {
    let mut predicted = ContainerClientMenu::new(9, 7, vec![ItemStack::Empty], ItemStack::Empty);
    let wrong_prediction_ignored = matches!(
        predicted
            .predict_click(10, 0, 0, ContainerInput::Pickup, |_, _| {
                panic!("a wrong-container click must not run local prediction")
            })
            .expect("checked click widths"),
        ContainerClientClick::IgnoredWrongContainer
    );
    let ContainerClientClick::PredictedAndSend(mut click) = predicted
        .predict_click(9, 0, 0, ContainerInput::Pickup, |_, _| {})
        .expect("checked click widths")
    else {
        panic!("the matching container must emit a predicted click");
    };

    let mut server = ContainerMenuTransaction::new(
        9,
        8,
        ContainerAuthoritativeState {
            slots: vec![ItemStack::Empty],
            carried: ItemStack::Empty,
            data: Vec::new(),
        },
    )
    .expect("bounded authoritative menu");
    click.state_id = 7;
    let outcome = server
        .handle_click(
            click,
            ContainerActor {
                spectator: false,
                dead_or_dying: false,
            },
            |_, _, _, _| Ok(()),
        )
        .expect("authoritative click executes");
    let (stale_click_executed, stale_full_resync) = match outcome {
        ContainerClickOutcome::Converged {
            click_executed,
            stale_state: true,
            packets,
            ..
        } => (
            click_executed,
            matches!(
                packets.as_slice(),
                [PlayClientboundPacket::ContainerSetContent(_)]
            ),
        ),
        other => panic!("stale click must execute then fully converge: {other:?}"),
    };

    let menu_type = Identifier::minecraft("generic_9x1").expect("static identifier");
    let mut projection = ContainerClientProjection::new(
        46,
        BTreeMap::from([(
            menu_type.clone(),
            MenuDefinition {
                slots: 1,
                data_slots: 0,
                has_screen: true,
            },
        )]),
        64,
    )
    .expect("bounded client projection");
    projection
        .apply(&PlayClientboundPacket::OpenScreen(OpenScreen {
            container_id: 9,
            menu_type,
            title: TextComponentNbt::literal("menu").expect("static title"),
        }))
        .expect("known screen opens");
    projection
        .apply(&PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: 9,
                state_id: 8,
                slots: vec![ItemStack::Empty],
                carried: ItemStack::Empty,
            },
        ))
        .expect("matching content initializes the menu");
    projection
        .apply(&PlayClientboundPacket::ContainerSetContent(
            ContainerSetContent {
                container_id: 10,
                state_id: 99,
                slots: vec![ItemStack::Empty],
                carried: ItemStack::Empty,
            },
        ))
        .expect("delayed content is ignored");
    let delayed_content_ignored = projection.active_menu().state_id == 8;
    let close_abandoned_open_menu = projection
        .apply(&PlayClientboundPacket::ContainerClose(ContainerClose {
            container_id: -999,
        }))
        .is_ok_and(|action| action == ContainerProjectionAction::ScreenClosed)
        && projection.active_menu().container_id == 0;

    MenuConvergenceReport {
        wrong_prediction_ignored,
        prediction_count: predicted.predictions,
        stale_click_executed,
        stale_full_resync,
        delayed_content_ignored,
        close_abandoned_open_menu,
    }
}
