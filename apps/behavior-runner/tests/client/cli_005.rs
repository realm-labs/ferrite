use behavior_runner::client::menu::{
    BooleanControl, ControlDispatch, ControlError, ControlKind, KeyboardInput, MenuGestureState,
    MenuInput, NumberRangeControl, PointerModifiers, PointerPress, SemanticClick,
    SingleOptionControl, SingleOptionEntry, SlotBox, SlotGesture, SubmittedTag, TextControl,
    dispatch_control, hovered_slot,
};
use ferrite_testkit::service_conformance::menu::run_menu_convergence;

fn slot(index: i32, item: Option<&str>, count: i32) -> SlotGesture {
    SlotGesture {
        index,
        item: item.map(str::to_owned),
        count,
        maximum: 64,
        active: true,
        may_pickup: true,
        may_place: true,
        same_container: true,
    }
}

fn click(slot: i32, button: i32, input: MenuInput) -> SemanticClick {
    SemanticClick {
        slot,
        button,
        input,
    }
}

fn press(now_ms: u64, button: i32, carried_count: i32) -> PointerPress {
    PointerPress {
        now_ms,
        button,
        outside: false,
        carried_count,
        modifiers: PointerModifiers::default(),
        same_screen_interval: false,
    }
}

#[test]
fn hover_geometry_is_expanded_half_open_and_first_active_wins() {
    let mut inactive = slot(0, None, 0);
    inactive.active = false;
    let slots = vec![
        SlotBox {
            slot: inactive,
            x: 10,
            y: 20,
        },
        SlotBox {
            slot: slot(1, Some("stone"), 1),
            x: 10,
            y: 20,
        },
        SlotBox {
            slot: slot(2, Some("diamond"), 1),
            x: 10,
            y: 20,
        },
    ];
    assert_eq!(
        hovered_slot(&slots, 9.0, 19.0).map(|slot| slot.index),
        Some(1)
    );
    assert_eq!(
        hovered_slot(&slots, 26.999, 36.999).map(|slot| slot.index),
        Some(1)
    );
    assert_eq!(hovered_slot(&slots, 27.0, 20.0), None);
    assert_eq!(hovered_slot(&slots, 10.0, 37.0), None);
}

#[test]
fn empty_carried_presses_commit_once_and_double_click_is_strictly_below_250_ms() {
    let hovered = slot(3, Some("stone"), 1);
    let mut state = MenuGestureState::default();
    assert_eq!(
        state.press(
            Some(&hovered),
            PointerPress {
                same_screen_interval: true,
                ..press(0, 0, 0)
            },
        ),
        vec![click(3, 0, MenuInput::Pickup)]
    );
    assert!(
        state
            .release(0, Some(&hovered), 0, PointerModifiers::default(), &[])
            .is_empty()
    );

    state.press(
        Some(&hovered),
        PointerPress {
            same_screen_interval: true,
            ..press(100, 0, 0)
        },
    );
    assert_eq!(
        state.release(0, Some(&hovered), 0, PointerModifiers::default(), &[]),
        vec![click(3, 0, MenuInput::PickupAll)]
    );

    state.press(
        Some(&hovered),
        PointerPress {
            same_screen_interval: true,
            ..press(350, 0, 0)
        },
    );
    assert!(
        state
            .release(0, Some(&hovered), 0, PointerModifiers::default(), &[])
            .is_empty()
    );

    let shift = PointerModifiers {
        shift: true,
        ..PointerModifiers::default()
    };
    assert_eq!(
        state.press(
            Some(&hovered),
            PointerPress {
                modifiers: shift,
                ..press(600, 0, 0)
            },
        ),
        vec![click(3, 0, MenuInput::QuickMove)]
    );
    state.release(0, Some(&hovered), 0, shift, &[]);
    assert_eq!(
        state.press(
            None,
            PointerPress {
                outside: true,
                ..press(900, 0, 0)
            },
        ),
        vec![click(-999, 0, MenuInput::Throw)]
    );
}

#[test]
fn clone_swap_and_double_shift_use_the_locked_semantic_order() {
    let hovered = slot(3, Some("stone"), 1);
    let creative_pick = PointerModifiers {
        creative: true,
        pick_button: true,
        ..PointerModifiers::default()
    };
    let mut state = MenuGestureState::default();
    assert_eq!(
        state.press(
            Some(&hovered),
            PointerPress {
                modifiers: creative_pick,
                same_screen_interval: true,
                ..press(0, 2, 0)
            },
        ),
        vec![click(3, 2, MenuInput::Clone)]
    );
    state.release(2, Some(&hovered), 0, creative_pick, &[]);
    assert!(
        state
            .press(
                Some(&hovered),
                PointerPress {
                    modifiers: PointerModifiers {
                        pick_button: true,
                        ..PointerModifiers::default()
                    },
                    ..press(200, 2, 0)
                },
            )
            .is_empty(),
        "pick is not a container click outside creative mode"
    );

    let offhand = PointerModifiers {
        offhand_button: true,
        ..PointerModifiers::default()
    };
    assert_eq!(
        state.press(
            Some(&hovered),
            PointerPress {
                modifiers: offhand,
                ..press(300, 4, 0)
            },
        ),
        vec![click(3, 40, MenuInput::Swap)]
    );
    let hotbar = PointerModifiers {
        hotbar_button: Some(8),
        ..PointerModifiers::default()
    };
    assert_eq!(
        state.press(
            Some(&hovered),
            PointerPress {
                modifiers: hotbar,
                ..press(600, 5, 0)
            },
        ),
        vec![click(3, 8, MenuInput::Swap)]
    );

    state.press(Some(&hovered), press(1_000, 0, 0));
    state.release(0, Some(&hovered), 0, PointerModifiers::default(), &[]);
    state.press(
        Some(&hovered),
        PointerPress {
            same_screen_interval: true,
            ..press(1_249, 0, 0)
        },
    );
    let mut other_container = slot(4, Some("stone"), 1);
    other_container.same_container = false;
    let mut blocked = slot(5, Some("stone"), 1);
    blocked.may_pickup = false;
    let matching = [
        slot(6, Some("stone"), 1),
        slot(7, Some("diamond"), 1),
        other_container,
        blocked,
        slot(8, Some("stone"), 1),
    ];
    assert_eq!(
        state.release(
            0,
            Some(&hovered),
            0,
            PointerModifiers {
                shift: true,
                ..PointerModifiers::default()
            },
            &matching,
        ),
        vec![
            click(6, 0, MenuInput::QuickMove),
            click(8, 0, MenuInput::QuickMove),
        ]
    );
}

#[test]
fn quick_craft_admits_once_recomputes_preview_and_emits_start_add_end() {
    let first = slot(1, None, 0);
    let mut second = slot(2, Some("stone"), 3);
    second.maximum = 5;
    let mut incompatible = slot(3, Some("diamond"), 1);
    incompatible.may_place = true;
    let mut state = MenuGestureState::default();
    state.press(
        Some(&first),
        PointerPress {
            same_screen_interval: true,
            ..press(0, 0, 8)
        },
    );
    state.drag(&first, Some("stone"), 8);
    assert_eq!(state.quick_remainder, 0);
    state.drag(&first, Some("stone"), 8);
    state.drag(&incompatible, Some("stone"), 8);
    state.drag(&second, Some("stone"), 8);
    assert_eq!(
        state.quick_remainder, 2,
        "four plus two items are previewed"
    );
    assert_eq!(
        state.release(0, Some(&second), 8, PointerModifiers::default(), &[]),
        vec![
            click(-999, 0, MenuInput::QuickCraft),
            click(1, 1, MenuInput::QuickCraft),
            click(2, 1, MenuInput::QuickCraft),
            click(-999, 2, MenuInput::QuickCraft),
        ]
    );

    state.press(Some(&first), press(500, 1, 8));
    state.drag(&first, Some("stone"), 8);
    assert!(
        state
            .release(0, Some(&first), 8, PointerModifiers::default(), &[])
            .is_empty()
    );
    assert_eq!(
        state.release(1, Some(&first), 8, PointerModifiers::default(), &[]),
        vec![click(1, 1, MenuInput::Pickup)]
    );

    state.press(Some(&first), press(1_000, 0, 8));
    state.drag(&first, Some("stone"), 8);
    state.close();
    assert_eq!(
        state.release(0, Some(&first), 8, PointerModifiers::default(), &[]),
        vec![click(1, 0, MenuInput::Pickup)]
    );
}

#[test]
fn keyboard_actions_preserve_priority_and_empty_carried_swap_gate() {
    let hovered = slot(3, Some("stone"), 1);
    let state = MenuGestureState::default();
    assert!(
        state
            .keyboard(
                Some(&hovered),
                KeyboardInput {
                    inventory_key: true,
                    pick_key: true,
                    drop_key: true,
                    control: true,
                    carried_empty: true,
                    offhand_key: true,
                    hotbar_key: Some(2),
                },
            )
            .close
    );
    assert_eq!(
        state
            .keyboard(
                Some(&hovered),
                KeyboardInput {
                    pick_key: true,
                    drop_key: true,
                    control: true,
                    carried_empty: true,
                    offhand_key: true,
                    hotbar_key: Some(2),
                    ..KeyboardInput::default()
                },
            )
            .clicks,
        vec![click(3, 0, MenuInput::Clone), click(3, 40, MenuInput::Swap),]
    );
    assert_eq!(
        state
            .keyboard(
                Some(&hovered),
                KeyboardInput {
                    drop_key: true,
                    control: true,
                    carried_empty: false,
                    offhand_key: true,
                    hotbar_key: Some(2),
                    ..KeyboardInput::default()
                },
            )
            .clicks,
        vec![click(3, 1, MenuInput::Throw)]
    );
}

#[test]
fn dialog_controls_validate_defaults_bounds_order_and_submission_shapes() {
    let boolean = BooleanControl::default();
    assert_eq!(
        boolean.submit(false),
        (SubmittedTag::Byte(0), "false".to_owned())
    );
    assert_eq!(
        boolean.submit(true),
        (SubmittedTag::Byte(1), "true".to_owned())
    );

    let descending = NumberRangeControl::new(10.0, 0.0, Some(8.0), Some(3.0)).unwrap();
    assert_eq!(descending.width, 200);
    assert_eq!(descending.slider_position(5.0), 0.5);
    assert_eq!(descending.normalize(10.0), 8.0);
    assert_eq!(
        descending.submit(5.0),
        (SubmittedTag::Int(5), "5".to_owned())
    );
    let equal = NumberRangeControl::new(4.5, 4.5, None, None).unwrap();
    assert_eq!(equal.slider_position(4.5), 0.5);
    assert_eq!(
        equal.submit(4.5),
        (SubmittedTag::Float(4.5), "4.5".to_owned())
    );
    assert_eq!(
        NumberRangeControl::new(0.0, 1.0, Some(2.0), None),
        Err(ControlError::InitialOutOfRange)
    );
    assert_eq!(
        NumberRangeControl::new(0.0, 1.0, None, Some(0.0)),
        Err(ControlError::InvalidStep)
    );

    let mut options = SingleOptionControl::new(
        vec![
            SingleOptionEntry {
                id: "first".to_owned(),
                display: None,
            },
            SingleOptionEntry {
                id: "second".to_owned(),
                display: Some("Second display".to_owned()),
            },
        ],
        &[],
    )
    .unwrap();
    assert_eq!(options.display(), "first");
    options.cycle();
    assert_eq!(options.display(), "Second display");
    assert_eq!(
        options.submit(),
        (
            SubmittedTag::String("second".to_owned()),
            "second".to_owned()
        )
    );
    assert_eq!(
        SingleOptionControl::new(Vec::new(), &[]),
        Err(ControlError::EmptyOptions)
    );
}

#[test]
fn text_and_registry_controls_preserve_layout_escaping_and_unknown_dispatch() {
    let default = TextControl::default();
    assert_eq!(
        (
            default.width,
            default.label_visible,
            default.max_length,
            default.height
        ),
        (200, true, 32, 20)
    );
    let multiline = TextControl::multiline(String::new(), 64, None, None).unwrap();
    assert_eq!((multiline.max_lines, multiline.height), (Some(4), 44));
    let tall = TextControl::multiline(String::new(), 64, Some(100), None).unwrap();
    assert_eq!(tall.height, 512);
    assert_eq!(
        multiline.submit("a\\\"b", "run %s").unwrap(),
        (
            SubmittedTag::String("a\\\"b".to_owned()),
            "run a\\\\\\\"b".to_owned()
        )
    );
    assert_eq!(
        TextControl::single_line("long".to_owned(), 3),
        Err(ControlError::InvalidTextLength)
    );
    assert_eq!(
        TextControl::multiline(String::new(), 32, Some(0), None),
        Err(ControlError::InvalidLineCount)
    );

    assert_eq!(
        dispatch_control("number_range"),
        ControlDispatch::Registered(ControlKind::NumberRange)
    );
    assert_eq!(
        dispatch_control("future_control"),
        ControlDispatch::Ignored {
            logged: true,
            widget_added: false,
            getter_added: false,
        }
    );
}

#[test]
fn prediction_server_convergence_and_identity_gated_overwrite_join_cross_crate() {
    let report = run_menu_convergence();
    assert!(report.wrong_prediction_ignored);
    assert_eq!(report.prediction_count, 1);
    assert!(report.stale_click_executed);
    assert!(report.stale_full_resync);
    assert!(report.delayed_content_ignored);
    assert!(report.close_abandoned_open_menu);
}
