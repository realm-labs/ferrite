use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::item::runtime::container_storage::{LootCaller, PendingLoot};
use ferrite_gameplay::item::runtime::random::GameplayRandom;
use ferrite_gameplay::item::runtime::stack::ItemStack;
use ferrite_gameplay::item::runtime::transport::boat::{
    BoatHit, BoatInteractionAction, BoatInteractionInput, BoatRemovalContext, BoatUseInput,
    BoatUseResult, CHEST_BOAT_SLOTS, ChestBoatSave, ChestBoatStorage, DispenserTerrain, Position,
    RemovalReason, StackConfigurationStep, VehicleBaseInteraction, boat_recipe,
    destruction_item_custom_name, dispense_boat, interact_boat, passenger_ride_height,
    qualifies_goat_boat_advancement, remove_boat, use_boat,
};
use ferrite_gameplay::item::runtime::transport::catalog::{
    BOAT_FAMILIES, BOAT_FUEL_TICKS, BOAT_MAXIMUM_STACK, BoatGeometry, FISHERMAN_BOAT_TRADES,
    HARNESS_MAXIMUM_STACK, HARNESSES, boat_identity, harness_identity,
};
use ferrite_gameplay::item::runtime::transport::harness::{
    DispenserCandidate, EquipResult, HAPPY_GHAST_ENTITY_ID, HappyGhastInteraction,
    HarnessAdmission, RiddenInput, STILL_TIMEOUT_MAXIMUM, ShearInput, ShearStep, StillTimeout,
    TemptationSet, can_add_passenger, equip_harness, first_dispenser_candidate, harness_recipe,
    has_player_controller, interact_happy_ghast, ridden_input, ridden_rotation, shear_happy_ghast,
    temptation_set, travel_speed, valid_body_equipment,
};
use std::collections::{BTreeSet, VecDeque};

fn id(path: &str) -> ResourceId {
    ResourceId::minecraft(path).unwrap()
}

fn stack(identity: u64, path: &str, count: i32, maximum: i32) -> ItemStack {
    ItemStack::new(identity, id(path), count, maximum, 0)
}

#[derive(Default)]
struct ScriptRandom {
    integers: VecDeque<u32>,
    bounds: Vec<u32>,
}

impl GameplayRandom for ScriptRandom {
    fn next_int(&mut self, bound: u32) -> u32 {
        self.bounds.push(bound);
        self.integers.pop_front().unwrap_or(0)
    }

    fn next_float(&mut self) -> f32 {
        0.5
    }

    fn next_bool(&mut self) -> bool {
        false
    }
}

#[test]
fn boat_and_harness_catalogs_are_closed_and_identity_exact() {
    assert_eq!(BOAT_FAMILIES.len(), 10);
    assert_eq!(
        BOAT_FAMILIES
            .iter()
            .flat_map(|family| [family.ordinary_item_id, family.chest_item_id])
            .collect::<BTreeSet<_>>(),
        (891..=910).collect()
    );
    let entity_ids = BOAT_FAMILIES
        .iter()
        .flat_map(|family| [family.ordinary_entity_id, family.chest_entity_id])
        .collect::<BTreeSet<_>>();
    assert_eq!(entity_ids.len(), 20);
    assert_eq!(boat_identity("acacia_boat").unwrap().entity_id, 0);
    assert!(boat_identity("bamboo_chest_raft").unwrap().chest);
    assert_eq!(BOAT_FAMILIES[9].geometry, BoatGeometry::Raft);
    assert_eq!(BOAT_FUEL_TICKS, 1_200);
    assert_eq!(BOAT_MAXIMUM_STACK, 1);

    assert_eq!(HARNESSES.len(), 16);
    assert_eq!(
        HARNESSES
            .iter()
            .map(|harness| harness.item_id)
            .collect::<Vec<_>>(),
        (866..=881).collect::<Vec<_>>()
    );
    assert_eq!(
        harness_identity("red_harness").unwrap().asset,
        "red_harness"
    );
    assert_eq!(HARNESS_MAXIMUM_STACK, 1);
    assert_eq!(HAPPY_GHAST_ENTITY_ID, 58);
}

#[test]
fn fisherman_trade_records_cover_exact_types_and_constants() {
    assert_eq!(FISHERMAN_BOAT_TRADES.len(), 5);
    assert_eq!(
        FISHERMAN_BOAT_TRADES
            .iter()
            .flat_map(|trade| trade.villager_types.iter().copied())
            .collect::<BTreeSet<_>>(),
        [
            "desert", "jungle", "plains", "savanna", "snow", "swamp", "taiga"
        ]
        .into_iter()
        .collect()
    );
    let swamp = FISHERMAN_BOAT_TRADES
        .iter()
        .find(|trade| trade.villager_types == ["swamp"])
        .unwrap();
    assert_eq!(swamp.boat, "dark_oak_boat");
    assert_eq!(
        (
            swamp.emeralds,
            swamp.boats,
            swamp.maximum_uses,
            swamp.villager_experience,
            swamp.reputation_discount,
        ),
        (1, 1, 12, 30, 0.05)
    );
}

fn placement_input(hit: BoatHit) -> BoatUseInput {
    BoatUseInput {
        hit,
        eye_inside_pickable_box: false,
        factory_created: true,
        collision_free: true,
        server_side: true,
        admission_accepted: true,
        player_yaw: 45.0,
    }
}

#[test]
fn held_boat_placement_preserves_all_abort_boundaries() {
    assert_eq!(
        use_boat(placement_input(BoatHit::Miss)).result,
        BoatUseResult::Pass
    );
    let mut obstructed = placement_input(BoatHit::Block(Position {
        x: 1.0,
        y: 2.0,
        z: 3.0,
    }));
    obstructed.eye_inside_pickable_box = true;
    assert_eq!(use_boat(obstructed).result, BoatUseResult::Pass);
    obstructed.eye_inside_pickable_box = false;
    obstructed.factory_created = false;
    assert_eq!(use_boat(obstructed).result, BoatUseResult::Fail);
    obstructed.factory_created = true;
    obstructed.collision_free = false;
    let collision = use_boat(obstructed);
    assert!(collision.entity_created);
    assert_eq!(collision.result, BoatUseResult::Fail);
    assert!(!collision.spawn_attempted && !collision.awarded_item_used_stat);
}

#[test]
fn held_boat_ignores_post_collision_admission_but_client_only_predicts() {
    let position = Position {
        x: -1.25,
        y: 64.0,
        z: 8.5,
    };
    let mut input = placement_input(BoatHit::Block(position));
    input.admission_accepted = false;
    let server = use_boat(input);
    assert_eq!(server.result, BoatUseResult::Success);
    assert!(server.spawn_attempted && !server.admitted);
    assert!(server.placement_event && server.awarded_item_used_stat);
    assert_eq!(server.consumed, 1);
    assert_eq!(
        server.configuration,
        [
            StackConfigurationStep::ImplicitComponents,
            StackConfigurationStep::ExplicitEntityData,
        ]
    );

    input.server_side = false;
    let client = use_boat(input);
    assert_eq!(client.result, BoatUseResult::Success);
    assert!(!client.spawn_attempted && !client.placement_event);
    assert_eq!(client.consumed, 0);
    assert!(client.configuration.is_empty());
}

#[test]
fn boat_interaction_keeps_mount_and_chest_open_fallthrough_asymmetric() {
    let base = BoatInteractionInput {
        base: VehicleBaseInteraction::Pass,
        chest: true,
        secondary_use: false,
        out_of_control_ticks: 0.0,
        client_side: false,
        start_riding_succeeds: true,
        can_add_passenger: true,
    };
    assert_eq!(interact_boat(base).action, BoatInteractionAction::Mount);

    let failed_mount = BoatInteractionInput {
        start_riding_succeeds: false,
        ..base
    };
    assert_eq!(
        interact_boat(failed_mount).action,
        BoatInteractionAction::Pass
    );
    let occupied = BoatInteractionInput {
        can_add_passenger: false,
        ..failed_mount
    };
    let opened = interact_boat(occupied);
    assert_eq!(opened.action, BoatInteractionAction::OpenContainer);
    assert!(opened.container_open_event && opened.anger_piglins);
    let secondary = BoatInteractionInput {
        secondary_use: true,
        can_add_passenger: true,
        ..base
    };
    assert_eq!(
        interact_boat(secondary).action,
        BoatInteractionAction::OpenContainer
    );
    let uncontrolled = BoatInteractionInput {
        chest: false,
        out_of_control_ticks: 60.0,
        ..base
    };
    assert_eq!(
        interact_boat(uncontrolled).action,
        BoatInteractionAction::Pass
    );
}

#[test]
fn chest_boat_storage_materializes_before_fill_and_persists_one_branch() {
    let mut chest = ChestBoatStorage::empty();
    chest.storage.pending_loot = Some(PendingLoot {
        table_fingerprint: 77,
        seed: 0,
    });
    assert!(!chest.open(true, 9, 2.5, |_, _| panic!("spectator filled loot")));
    assert!(matches!(
        chest.save(),
        ChestBoatSave::PendingLoot {
            table_fingerprint: 77,
            seed: None
        }
    ));
    let filled = std::cell::Cell::new(false);
    assert!(chest.open(false, 9, 2.5, |pending, slots| {
        assert_eq!(pending.table_fingerprint, 77);
        slots[0].stack = stack(1, "diamond", 2, 64);
        filled.set(true);
    }));
    assert!(filled.get());
    assert_eq!(
        chest.storage.materialized_by,
        Some(LootCaller::Player {
            player_fingerprint: 9,
            luck_bits: 2.5_f32.to_bits(),
        })
    );
    assert_eq!(chest.storage.inventory.slots[0].stack.count, 2);
    let loaded = ChestBoatStorage::load(chest.save());
    assert_eq!(loaded.storage.inventory.slots.len(), CHEST_BOAT_SLOTS);
    assert_eq!(loaded.storage.inventory.slots[0].stack.count, 2);
    assert!(ChestBoatStorage::still_valid(false, 24.9, 1.0));
    assert!(!ChestBoatStorage::still_valid(false, 25.0, 1.0));
    assert!(!ChestBoatStorage::still_valid(true, 0.0, 100.0));
}

#[test]
fn chest_removal_scatters_before_itemization_independent_of_entity_drops() {
    let mut chest = ChestBoatStorage::empty();
    chest.storage.pending_loot = Some(PendingLoot {
        table_fingerprint: 1,
        seed: 0,
    });
    let mut random = ScriptRandom {
        integers: [0, 20, 0].into_iter().collect(),
        ..ScriptRandom::default()
    };
    let identity = std::cell::Cell::new(10_u64);
    let outcome = remove_boat(
        Some(&mut chest),
        BoatRemovalContext {
            reason: RemovalReason::Discarded,
            server_side: true,
            entity_drops: false,
            itemize_vehicle: false,
            direct_player_damage: true,
        },
        &mut random,
        || {
            identity.set(identity.get() + 1);
            identity.get()
        },
        |_, slots| slots[0].stack = stack(1, "apple", 35, 64),
    )
    .unwrap();
    assert_eq!(
        outcome
            .scattered_contents
            .iter()
            .map(|stack| stack.count)
            .collect::<Vec<_>>(),
        [10, 25]
    );
    assert!(!outcome.matching_vehicle_item && !outcome.anger_piglins);
    assert_eq!(random.bounds, [21, 21]);
    assert_eq!(chest.storage.materialized_by, Some(LootCaller::NullPlayer));

    let mut empty = ChestBoatStorage::empty();
    let killed = remove_boat(
        Some(&mut empty),
        BoatRemovalContext {
            reason: RemovalReason::Killed,
            server_side: true,
            entity_drops: true,
            itemize_vehicle: true,
            direct_player_damage: true,
        },
        &mut ScriptRandom::default(),
        || 1,
        |_, _| {},
    )
    .unwrap();
    assert!(killed.matching_vehicle_item && killed.anger_piglins);
}

#[test]
fn dispenser_offsets_and_passenger_heights_distinguish_boats_and_rafts() {
    let origin = Position {
        x: 10.0,
        y: 20.0,
        z: 30.0,
    };
    let water = dispense_boat(origin, 1, 0, 0, 90.0, DispenserTerrain::Water, true);
    assert_eq!(water.position.y, 21.0);
    assert_eq!(water.position.x, 11.25);
    assert_eq!(water.yaw, 90.0);
    assert!(water.consume_after_creation);
    let air = dispense_boat(origin, 0, -1, 0, 0.0, DispenserTerrain::AirOverWater, false);
    assert_eq!(air.position.y, 18.875);
    assert!(!air.consume_after_creation);
    assert!(dispense_boat(origin, 0, 0, 1, 0.0, DispenserTerrain::Fallback, true).fallback);
    assert_eq!(passenger_ride_height(1.8, false), 0.599_999_96);
    assert_eq!(passenger_ride_height(1.8, true), 1.6);
    assert!(qualifies_goat_boat_advancement(false, true));
    assert!(!qualifies_goat_boat_advancement(true, true));
    assert!(!boat_recipe(true).copies_source_components);
    assert_eq!(
        destruction_item_custom_name(Some("Ferry")).as_deref(),
        Some("Ferry")
    );
}

#[test]
fn harness_equip_consumes_on_server_including_creative_and_selects_first_candidate() {
    let mut held = stack(7, "white_harness", 2, 1);
    held.count = 2;
    let outcome = equip_harness(
        &mut held,
        HarnessAdmission {
            server_side: true,
            target_alive: true,
            target_adult: true,
            allowed_by_live_tag: true,
            body_slot_empty: true,
        },
        false,
    );
    assert_eq!(outcome.result, EquipResult::Success);
    assert_eq!(
        (held.count, outcome.equipped.count, outcome.consumed),
        (1, 1, 1)
    );
    assert!(outcome.guaranteed_drop && outcome.equip_event);
    assert!(!outcome.persistence_required && !outcome.item_used_stat);

    let candidates = [
        DispenserCandidate {
            living: true,
            alive: true,
            spectator: false,
            allowed_by_live_tag: true,
            slot_admitting: false,
            body_slot_empty: true,
        },
        DispenserCandidate {
            living: true,
            alive: true,
            spectator: false,
            allowed_by_live_tag: true,
            slot_admitting: true,
            body_slot_empty: true,
        },
    ];
    assert_eq!(first_dispenser_candidate(&candidates), Some(1));
}

#[test]
fn harness_validity_controls_temptation_mount_capacity_and_controller() {
    assert!(valid_body_equipment(true, true, true, true));
    assert!(!valid_body_equipment(true, true, true, false));
    assert_eq!(
        temptation_set(false, false),
        TemptationSet::FoodAndHarnesses
    );
    assert_eq!(temptation_set(false, true), TemptationSet::FoodOnly);
    assert_eq!(
        interact_happy_ghast(false, true, true, false),
        HappyGhastInteraction::Equip
    );
    assert_eq!(
        interact_happy_ghast(false, false, true, false),
        HappyGhastInteraction::Mount
    );
    assert_eq!(
        interact_happy_ghast(false, false, true, true),
        HappyGhastInteraction::Generic
    );
    assert!(can_add_passenger(3));
    assert!(!can_add_passenger(4));
    assert!(has_player_controller(true, 0, true));
    assert!(!has_player_controller(true, 1, true));
}

#[test]
fn leash_cut_precedes_shearing_and_equipment_steps_are_exact() {
    let leash = shear_happy_ghast(ShearInput {
        leashed: true,
        secondary_use: false,
        passengers: 0,
        body_harness: true,
        prevent_armor_change: false,
        creative: false,
    });
    assert_eq!(leash.steps, [ShearStep::CutLeash, ShearStep::DamageShears]);
    assert!(!leash.recovered_harness);

    let shear = shear_happy_ghast(ShearInput {
        leashed: false,
        secondary_use: false,
        passengers: 0,
        body_harness: true,
        prevent_armor_change: true,
        creative: true,
    });
    assert_eq!(
        shear.steps,
        [
            ShearStep::DamageShears,
            ShearStep::ClearBody,
            ShearStep::UnequipEvent,
            ShearStep::ShearEvent,
            ShearStep::SpawnEquipment,
            ShearStep::PlayerShearedCriterion,
            ShearStep::UnequipSound,
        ]
    );
    assert!(shear.recovered_harness);
}

#[test]
fn ridden_input_rotation_timeout_and_recipes_lock_source_constants() {
    let forward = ridden_input(RiddenInput {
        strafe: 1.0,
        forward: 1.0,
        jumping: true,
        pitch_degrees: 90.0,
        flying_speed: 0.05,
    });
    let scale = f64::from(3.9_f32) * 0.05;
    assert!((forward.x - scale).abs() < f64::EPSILON);
    assert!((forward.y - (-0.5 * scale)).abs() < 1.0e-7);
    assert!(forward.z.abs() < 1.0e-7);
    let backward = ridden_input(RiddenInput {
        forward: -1.0,
        jumping: false,
        ..RiddenInput {
            strafe: 0.0,
            forward: 0.0,
            jumping: false,
            pitch_degrees: 0.0,
            flying_speed: 0.05,
        }
    });
    assert!((backward.z - (-0.5 * scale)).abs() < f64::EPSILON);
    assert_eq!(travel_speed(0.05), 0.083_333_336);
    let rotation = ridden_rotation(179.0, -179.0, 40.0);
    assert_eq!(rotation.yaw, 179.16);
    assert_eq!(rotation.pitch, 20.0);
    assert_eq!(rotation.body_yaw, rotation.head_yaw);

    let mut timeout = StillTimeout {
        tick_count: 59,
        remaining: 10,
    };
    timeout.tick(false);
    assert_eq!(timeout.remaining, 10);
    timeout.tick(false);
    assert_eq!(timeout.remaining, 9);
    timeout.tick(true);
    assert_eq!(timeout.remaining, STILL_TIMEOUT_MAXIMUM);
    timeout.passenger_removed();
    assert_eq!(timeout.remaining, STILL_TIMEOUT_MAXIMUM);
    timeout.passenger_added(false);
    assert_eq!(timeout.remaining, 0);

    let recipe = harness_recipe("red");
    assert_eq!(
        (
            recipe.base_leather,
            recipe.base_glass,
            recipe.base_wool,
            recipe.recolor_source_count,
        ),
        (3, 2, 1, 15)
    );
    assert!(recipe.excludes_same_color && !recipe.copies_source_components);
}
