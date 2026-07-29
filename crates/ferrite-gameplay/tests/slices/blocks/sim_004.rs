use std::collections::BTreeSet;

use ferrite_foundation::direction::{Axis, Direction};
use ferrite_gameplay::block::amethyst::{
    AmethystGrowth, AmethystStage, AmethystTarget, BUDDING_AMETHYST_STATE_ID, budding_random_tick,
    cluster_shard_count,
};
use ferrite_gameplay::block::aquatic::{
    BubbleFlow, BubbleUpdate, FROGSPAWN_STATE_ID, FrogspawnTick, LILY_PAD_STATE_ID,
    bubble_boat_launch, bubble_from_below, bubble_occupiable, bubble_refilled_air, bubble_velocity,
    frogspawn_due, frogspawn_interval, lily_pad_boat_contact, lily_pad_survives,
    tadpole_horizontal_offset, tadpole_yaw,
};
use ferrite_gameplay::block::chorus::{
    ChorusTick, chorus_flower_state_id, chorus_flower_tick, chorus_fruit_offset,
    chorus_teleport_attempts, distinct_chorus_branches,
};
use ferrite_gameplay::block::contact_blocks::{
    COBWEB_STATE_ID, apply_deferred_stuck, cobweb_contact,
};
use ferrite_gameplay::block::copper::{
    AxeCopperTransform, CopperAge, CopperFullKind, GolemWeatherClock, StatuePose, StatueState,
    WeatheringDecision, axe_transform, copper_golem_weather_tick, golem_statue_conversion_admitted,
    statue_family_state_count, statue_use_non_axe, weathering_decision,
};
use ferrite_gameplay::block::crop::{
    BerryHarvest, CocoaState, CropKind, CropTick, FarmlandCell, PitcherGrowth, PitcherHalf,
    PitcherRandomTick, PitcherState, StemKind, StemTick, TorchflowerState, berry_contact_damage,
    berry_harvest, berry_random_growth, berry_state_id, bone_meal_growth, growth_speed,
    pitcher_grow, pitcher_random_tick, random_tick, stem_bone_meal_age, stem_random_tick,
    torchflower_advance, torchflower_random_tick,
};
use ferrite_gameplay::block::decorative::{
    CakeEat, CakeState, CarvedPumpkinFacing, DesiredEyeblossom, EMPTY_FLOWER_POT_STATE_ID,
    FLOWER_POT_ITEM_ID, FlowerPotUse, GolemPattern, LATER_POTTED_STATE_IDS, candle_cake_admitted,
    carve_pumpkin, carved_pumpkin_placement, first_creatable_golem, flower_pot_use,
    is_flower_pot_state, melon_slice_count, potted_eyeblossom_tick,
};
use ferrite_gameplay::block::incubation::{
    SnifferEgg, SnifferEggDue, sniffer_egg_due, sniffer_yaw,
};
use ferrite_gameplay::block::lodestone::{
    BLOCK_ID as LODESTONE_BLOCK_ID, POI_MAX_TICKETS, STATE_ID as LODESTONE_STATE_ID, Tracker,
    TrackerTick, bind_compass, tracker_tick,
};
use ferrite_gameplay::block::material::{
    ANCIENT_DEBRIS, BLACKSTONE, CLAY, ClayDrop, DRIPSTONE_BLOCK, END_STONE, GildedDrop, MELON,
    MOSSY_COBBLESTONE, MUD, NETHER_BRICKS, NETHER_PLANKS, NETHERRACK, PACKED_ICE, PRISMARINE,
    PUMPKIN, RESIN, SMOOTH_STONE, SULFUR_CINNABAR, TUFF, WORKSTATION_TABLES,
    ancient_debris_item_resists_fire, clay_ball_drop, crafting_table_use, gilded_blackstone_drop,
    muddy_mangrove_roots_state,
};
use ferrite_gameplay::block::mushroom::{
    HugeMushroomDrop, HugeMushroomFaces, HugeMushroomKind, MushroomKind, MushroomWalkStep,
    NetherFungus, fungus_bone_meal_succeeds, huge_mushroom_drop, huge_mushroom_height,
    huge_mushroom_placement, huge_mushroom_write_count, mushroom_bone_meal_succeeds,
    mushroom_spread_admitted, mushroom_survives, mushroom_walk_step,
};
use ferrite_gameplay::block::plant_growth::{
    BAMBOO_SAPLING_STATE_ID, BambooLeaves, BambooState, CactusTick, CaveVinePart, CaveVineState,
    NetherVineHead, NetherVineKind, SaplingKind, SaplingTick, SmallTreeChoice,
    bamboo_bone_meal_attempts, bamboo_grow, bamboo_random_admitted, cactus_flower_attempt,
    cactus_random_tick, cactus_state_id, cactus_survives, cave_vine_harvest,
    cave_vine_placement_age, cave_vine_random_growth, nether_vine_bone_meal_count,
    nether_vine_extension_ages, sapling_bone_meal_succeeds, sapling_random_tick, small_tree_choice,
    sugar_cane_random_tick, sugar_cane_state_id, sugar_cane_survives,
};
use ferrite_gameplay::block::snow::{
    POWDER_SNOW_STATE_ID, PowderCollision, SNOW_BLOCK_STATE_ID, SnowLayer, freeze_damage,
    freeze_damage_due, frozen_ticks, powder_snow_collision, powder_snow_fall_sound,
    powder_snow_walkable, snow_layer_melts, snow_layer_survives,
};
use ferrite_gameplay::block::sponge::{
    ABSORPTION_NODE_CAP, AbsorbAction, SPONGE_STATE_ID, WET_SPONGE_STATE_ID, WaterCandidate,
    absorb_candidate, absorption_result, furnace_bucket_result, wet_sponge_on_place,
};
use ferrite_gameplay::block::terrain::{
    AgriculturalGround, DirtKind, MoistureTick, MossKind, Nylium, SpreadTick, ToolTransform,
    farmland_random_tick, farmland_tramples, hoe_transform, mud_dripstone_converts_to_clay,
    netherrack_conversion, nylium_bone_meal, packed_ice_friction, shovel_transform,
    snowy_after_update, spreading_ground_survives, spreading_ground_tick, water_bottle_to_mud,
};

#[test]
fn static_material_families_lock_protocol_ids_and_harvest_gates() {
    assert_eq!(
        (
            ANCIENT_DEBRIS.block_id,
            ANCIENT_DEBRIS.state_id,
            ANCIENT_DEBRIS.item_id
        ),
        (916, 21_819, 109)
    );
    assert_eq!(ANCIENT_DEBRIS.resistance, 1_200.0);
    assert!(!ANCIENT_DEBRIS.self_drop(false, true));
    assert!(ANCIENT_DEBRIS.self_drop(true, true));
    assert!(ancient_debris_item_resists_fire(7));
    assert!(!ancient_debris_item_resists_fire(8));

    assert_eq!(BLACKSTONE.len(), 6);
    assert_eq!(BLACKSTONE[1].state_id, 22_242);
    assert_eq!(END_STONE.map(|record| record.state_id), [9_477, 14_796]);
    assert_eq!(NETHER_BRICKS[2].state_id, 23_093);
    assert_eq!(NETHER_PLANKS[1].item_id, 74);
    assert_eq!(PRISMARINE[2].block_id, 529);
    assert_eq!(RESIN[0].hardness, 0.0);
    assert_eq!(TUFF[4].state_id, 24_686);
    assert_eq!(SULFUR_CINNABAR[7].item_id, 52);
}

#[test]
fn static_singletons_and_rotated_roots_preserve_locked_profiles() {
    assert_eq!((CLAY.block_id, CLAY.state_id), (281, 6_946));
    assert_eq!(DRIPSTONE_BLOCK.item_id, 53);
    assert_eq!(MELON.state_id, 8_333);
    assert_eq!(MOSSY_COBBLESTONE.state_id, 3_368);
    assert_eq!(MUD.state_id, 30_415);
    assert_eq!(NETHERRACK.state_id, 6_997);
    assert_eq!(PACKED_ICE.state_id, 12_914);
    assert_eq!(PUMPKIN.item_id, 384);
    assert_eq!(SMOOTH_STONE.state_id, 13_480);
    assert_eq!(
        [
            muddy_mangrove_roots_state(Axis::X),
            muddy_mangrove_roots_state(Axis::Y),
            muddy_mangrove_roots_state(Axis::Z),
        ],
        [165, 166, 167]
    );
}

#[test]
fn blackstone_clay_and_workstation_loot_keep_alternative_order() {
    assert_eq!(
        gilded_blackstone_drop(false, true, 0, true, 5),
        GildedDrop::GoldNuggets(5)
    );
    assert_eq!(
        gilded_blackstone_drop(false, false, 3, true, 5),
        GildedDrop::Nothing
    );
    assert_eq!(
        gilded_blackstone_drop(true, false, 0, false, 2),
        GildedDrop::Block
    );
    assert_eq!(
        clay_ball_drop(false, &[true, false, true, false]),
        ClayDrop::Balls(2)
    );
    assert_eq!(clay_ball_drop(true, &[]), ClayDrop::Block);

    assert_eq!(WORKSTATION_TABLES[0].state_id, 5_310);
    assert_eq!(WORKSTATION_TABLES[1].state_id, 20_771);
    let client = crafting_table_use(false);
    let server = crafting_table_use(true);
    assert!(client.result_success);
    assert!(!client.open_menu);
    assert_eq!((server.menu_type_id, server.slot_count), (12, 46));
    assert!(server.open_menu && server.award_stat);
}

#[test]
fn agricultural_ground_preserves_stale_path_tick_and_eight_step_drying() {
    let path = AgriculturalGround::DirtPath;
    let farmland = AgriculturalGround::Farmland { moisture: 7 };
    assert_eq!(path.state_id(), Some(14_815));
    assert_eq!(farmland.state_id(), Some(5_326));
    assert!(path.schedule_support_loss(Direction::Up, false));
    assert!(!path.schedule_support_loss(Direction::North, false));
    assert!(path.due_converts_to_dirt(true));
    assert!(!farmland.due_converts_to_dirt(true));

    let mut moisture = 7;
    for _ in 0..7 {
        moisture = match farmland_random_tick(moisture, false, false) {
            MoistureTick::Moisture(next) => next,
            other => panic!("unexpected dry step: {other:?}"),
        };
    }
    assert_eq!(moisture, 0);
    assert_eq!(farmland_random_tick(0, false, false), MoistureTick::Dirt);
    assert_eq!(farmland_random_tick(0, false, true), MoistureTick::NoChange);
    assert_eq!(
        farmland_random_tick(2, true, false),
        MoistureTick::Moisture(7)
    );
}

#[test]
fn farmland_trampling_uses_strict_draw_and_volume_boundaries() {
    assert!(farmland_tramples(0.49, 1.0, true, true, false, 1.0, 1.0));
    assert!(!farmland_tramples(0.5, 1.0, true, true, true, 1.0, 1.0));
    assert!(!farmland_tramples(0.0, 1.0, true, false, false, 1.0, 1.0));
    assert!(!farmland_tramples(0.0, 1.0, true, true, true, 0.8, 0.8));
}

#[test]
fn dirt_snow_spread_and_tool_transforms_follow_directional_rules() {
    assert_eq!(DirtKind::Grass.state_id(true), 8);
    assert_eq!(DirtKind::Grass.state_id(false), 9);
    assert!(snowy_after_update(
        DirtKind::Podzol,
        Direction::Up,
        false,
        true
    ));
    assert!(!snowy_after_update(
        DirtKind::Podzol,
        Direction::North,
        false,
        true
    ));
    assert!(spreading_ground_survives(true, true, 15));
    assert!(!spreading_ground_survives(false, true, 0));
    assert_eq!(
        spreading_ground_tick(false, true, 15),
        SpreadTick::MissingDirtRegistry
    );
    assert_eq!(
        spreading_ground_tick(true, false, 15),
        SpreadTick::DecayToDirt
    );
    assert_eq!(
        shovel_transform(DirtKind::Mycelium, Direction::Up, true),
        Some(ToolTransform::DirtPath)
    );
    assert_eq!(
        hoe_transform(DirtKind::CoarseDirt, Direction::Up, true),
        Some(ToolTransform::Dirt)
    );
    assert_eq!(
        hoe_transform(DirtKind::RootedDirt, Direction::Down, false),
        Some(ToolTransform::Dirt)
    );
    assert!(water_bottle_to_mud(DirtKind::Dirt, Direction::North));
    assert!(!water_bottle_to_mud(DirtKind::Podzol, Direction::North));
}

#[test]
fn budding_amethyst_consumes_direction_only_after_probability_admission() {
    assert_eq!(BUDDING_AMETHYST_STATE_ID, 23_403);
    assert_eq!(AmethystStage::Small.state_range(), 23_440..=23_451);
    assert_eq!(AmethystStage::Cluster.default_state_id(), 23_413);
    assert_eq!(
        budding_random_tick(1, Direction::Up, AmethystTarget::Air),
        AmethystGrowth::ProbabilityRejected
    );
    assert_eq!(
        budding_random_tick(0, Direction::East, AmethystTarget::FullSourceWater),
        AmethystGrowth::PlaceSmall {
            facing: Direction::East,
            waterlogged: true,
        }
    );
    assert_eq!(
        budding_random_tick(
            0,
            Direction::East,
            AmethystTarget::Bud {
                stage: AmethystStage::Large,
                facing: Direction::East,
                waterlogged: true,
            }
        ),
        AmethystGrowth::Advance {
            stage: AmethystStage::Cluster,
            facing: Direction::East,
            waterlogged: true,
        }
    );
    assert_eq!(cluster_shard_count(false, 4, 9), 2);
    assert_eq!(cluster_shard_count(true, 4, 3), 7);
}

#[test]
fn bubble_columns_lock_source_only_occupancy_precedence_and_velocity_clamps() {
    assert_eq!(BubbleFlow::Down.state_id(), 15_294);
    assert_eq!(BubbleFlow::Up.state_id(), 15_295);
    assert!(bubble_occupiable(true, false));
    assert!(bubble_occupiable(false, true));
    assert!(!bubble_occupiable(false, false));
    assert_eq!(
        bubble_from_below(Some(BubbleFlow::Down), true, false, true),
        BubbleUpdate::Column(BubbleFlow::Down)
    );
    assert_eq!(
        bubble_from_below(None, true, true, false),
        BubbleUpdate::Column(BubbleFlow::Up)
    );
    assert_eq!(
        bubble_from_below(None, false, false, true),
        BubbleUpdate::Water
    );
    assert_eq!(
        bubble_from_below(None, false, false, false),
        BubbleUpdate::Preserve
    );
    assert_eq!(bubble_velocity(BubbleFlow::Down, -0.89, false, true), -0.9);
    assert_eq!(bubble_velocity(BubbleFlow::Up, 1.75, false, true), 1.8);
    assert_eq!(bubble_boat_launch(BubbleFlow::Up, true), 2.7);
    assert_eq!(bubble_boat_launch(BubbleFlow::Up, false), 0.6);
    assert_eq!(bubble_refilled_air(20), 24);
}

#[test]
fn bamboo_growth_preserves_leaf_rewrites_age_and_terminal_boundaries() {
    assert_eq!(BAMBOO_SAPLING_STATE_ID, 15_278);
    assert_eq!(BambooState::default().state_id(), Some(15_279));
    assert_eq!(
        BambooState {
            age: 1,
            leaves: BambooLeaves::Large,
            stage: 1,
        }
        .state_id(),
        Some(15_290)
    );
    assert!(!bamboo_random_admitted(1, true, 15));
    assert!(bamboo_random_admitted(0, true, 9));
    let below = BambooState {
        age: 0,
        leaves: BambooLeaves::Large,
        stage: 0,
    };
    let two_below = BambooState {
        age: 1,
        leaves: BambooLeaves::Small,
        stage: 0,
    };
    let growth = bamboo_grow(15, Some(below), Some(two_below), false).unwrap();
    assert_eq!(growth.new_top.age, 1);
    assert_eq!(growth.new_top.leaves, BambooLeaves::Large);
    assert_eq!(growth.new_top.stage, 1);
    assert_eq!(growth.rewrite_below, Some(BambooLeaves::Small));
    assert_eq!(growth.rewrite_two_below, Some(BambooLeaves::None));
    assert_eq!(bamboo_bone_meal_attempts(0), 1);
    assert_eq!(bamboo_bone_meal_attempts(1), 2);
    assert!(bamboo_grow(16, None, None, false).is_none());
}

#[test]
fn cactus_and_sugar_cane_lock_support_age_and_growth_cap() {
    assert_eq!(cactus_state_id(0), Some(6_929));
    assert_eq!(cactus_state_id(15), Some(6_944));
    assert_eq!(sugar_cane_state_id(0), Some(6_947));
    assert_eq!(sugar_cane_state_id(15), Some(6_962));
    assert!(cactus_survives([false; 4], true, false));
    assert!(!cactus_survives([false, true, false, false], true, false));
    assert_eq!(cactus_random_tick(14, 2, true), CactusTick::Age(15));
    assert_eq!(
        cactus_random_tick(15, 2, true),
        CactusTick::Grow {
            upper_age: 0,
            reset_age: 0,
            notify_upper: true,
        }
    );
    assert_eq!(cactus_random_tick(15, 3, true), CactusTick::HeightCap);
    assert!(cactus_flower_attempt(3, 8, 0.25));
    assert!(!cactus_flower_attempt(3, 8, 0.250_001));

    assert!(sugar_cane_survives(true, false, false));
    assert!(sugar_cane_survives(false, true, true));
    assert!(!sugar_cane_survives(false, true, false));
    assert_eq!(
        sugar_cane_random_tick(15, 2, true),
        CactusTick::Grow {
            upper_age: 0,
            reset_age: 0,
            notify_upper: false,
        }
    );
}

#[test]
fn cake_eating_candles_and_analog_output_are_exact() {
    let cake = CakeState::new(0).unwrap();
    assert_eq!(cake.state_id(), 7_027);
    assert_eq!(cake.min_x_sixteenths(), 1);
    assert_eq!(cake.analog_output(), 14);
    assert!(candle_cake_admitted(cake, true));
    assert_eq!(cake.eat(), CakeEat::Next(CakeState { bites: 1 }));
    let last = CakeState::new(6).unwrap();
    assert_eq!(last.analog_output(), 2);
    assert_eq!(last.eat(), CakeEat::Remove);
}

#[test]
fn flower_pot_routes_mapping_before_empty_hand_extraction() {
    assert_eq!(EMPTY_FLOWER_POT_STATE_ID, 10_629);
    assert_eq!(FLOWER_POT_ITEM_ID, 1_256);
    assert!(is_flower_pot_state(10_630));
    assert!(LATER_POTTED_STATE_IDS.into_iter().all(is_flower_pot_state));
    assert!(!is_flower_pot_state(21_825));
    assert!(potted_eyeblossom_tick(false, DesiredEyeblossom::Default, [0.0; 4]).is_none());
    let opening =
        potted_eyeblossom_tick(false, DesiredEyeblossom::Open, [0.5, 1.0, 0.0, 0.5]).unwrap();
    assert_eq!(
        (
            opening.target_state_id,
            opening.flags,
            opening.particle_id,
            opening.color,
            opening.sound_id,
            opening.lifetime,
        ),
        (32_363, 3, 56, 0xFC_78_12, 619, 20)
    );
    assert_eq!(opening.target_offset, [0.5, 1.0, 0.0]);
    assert_eq!(
        flower_pot_use(false, true, false),
        FlowerPotUse::Insert {
            flags: 3,
            award_stat: true,
            consume_one: true,
        }
    );
    assert_eq!(
        flower_pot_use(true, true, false),
        FlowerPotUse::AlreadyFilled
    );
    assert_eq!(
        flower_pot_use(true, false, true),
        FlowerPotUse::Extract {
            flags: 3,
            add_or_drop_item: true,
        }
    );
    assert_eq!(flower_pot_use(false, false, true), FlowerPotUse::Empty);
}

#[test]
fn pumpkin_carving_and_golem_selection_keep_source_order() {
    assert_eq!(
        carved_pumpkin_placement(Direction::South),
        Some(CarvedPumpkinFacing::North)
    );
    assert_eq!(CarvedPumpkinFacing::East.state_id(), 7_022);
    let carve = carve_pumpkin(true).unwrap();
    assert_eq!(
        (carve.seeds, carve.write_flags, carve.durability_cost),
        (4, 11, 1)
    );
    assert_eq!(
        first_creatable_golem(&[
            (GolemPattern::Snow, true, false),
            (GolemPattern::Iron, true, true),
            (GolemPattern::Copper, true, true),
        ]),
        Some(GolemPattern::Iron)
    );
    assert_eq!(melon_slice_count(7, 5), 9);
}

#[test]
fn cave_vines_preserve_rng_age_water_and_harvest_state() {
    assert_eq!(cave_vine_placement_age(24), 24);
    let head = CaveVineState {
        part: CaveVinePart::Head,
        age: 24,
        berries: false,
    };
    assert_eq!(head.state_id(), Some(30_298));
    assert_eq!(
        cave_vine_random_growth(head, 0.099, true, 0.109),
        Some(CaveVineState {
            part: CaveVinePart::Head,
            age: 25,
            berries: true,
        })
    );
    assert!(cave_vine_random_growth(head, 0.1, true, 0.0).is_none());
    assert_eq!(
        cave_vine_harvest(CaveVineState {
            berries: true,
            ..head
        }),
        Some(head)
    );
}

#[test]
fn nether_vines_preserve_age_growth_and_uncapped_bone_meal_loop() {
    let head = NetherVineHead {
        kind: NetherVineKind::Weeping,
        age: 24,
    };
    assert_eq!(head.state_id(), Some(21_001));
    assert_eq!(head.body_state_id(), 21_003);
    assert_eq!(
        head.random_growth(0.099, true),
        Some(NetherVineHead {
            kind: NetherVineKind::Weeping,
            age: 25,
        })
    );
    let count = nether_vine_bone_meal_count(&[0.0, 0.8, 0.7, 0.9]);
    assert_eq!(count, 2);
    assert_eq!(nether_vine_extension_ages(24, 4), [25, 25, 25, 25]);
}

#[test]
fn chorus_growth_branching_and_teleport_draw_boundaries_are_locked() {
    assert_eq!(chorus_flower_state_id(0), Some(14_706));
    assert_eq!(chorus_flower_state_id(5), Some(14_711));
    assert_eq!(
        chorus_flower_tick(2, true, true, true, false, 3),
        ChorusTick::GrowUp { next_age: 2 }
    );
    assert_eq!(
        chorus_flower_tick(3, true, false, false, true, 2),
        ChorusTick::Branch {
            attempts: 4,
            next_age: 4,
        }
    );
    assert_eq!(
        chorus_flower_tick(4, true, false, false, true, 3),
        ChorusTick::Die
    );
    assert_eq!(
        distinct_chorus_branches(&[
            Direction::North,
            Direction::North,
            Direction::Up,
            Direction::West,
        ]),
        [Direction::North, Direction::West]
    );
    assert_eq!(chorus_teleport_attempts(), 16);
    assert_eq!(chorus_fruit_offset(0.0), -8.0);
    assert_eq!(chorus_fruit_offset(1.0), 8.0);
}

#[test]
fn mushrooms_lock_survival_density_walk_and_huge_feature_geometry() {
    assert!(mushroom_survives(true, 15, false));
    assert!(mushroom_survives(false, 12, true));
    assert!(!mushroom_survives(false, 13, true));
    assert!(mushroom_spread_admitted(0, 4));
    assert!(!mushroom_spread_admitted(0, 5));
    assert_eq!(
        mushroom_walk_step(0, 1, 0, 2),
        MushroomWalkStep {
            dx: -1,
            dy: 1,
            dz: 1,
        }
    );
    assert!(mushroom_bone_meal_succeeds(0.399));
    assert!(!mushroom_bone_meal_succeeds(0.4));
    assert_eq!(huge_mushroom_height(2, 0), 12);
    assert_eq!(huge_mushroom_write_count(MushroomKind::Brown, 12), 57);
}

#[test]
fn huge_mushroom_faces_are_same_identity_only_and_monotonic() {
    let faces = huge_mushroom_placement(
        HugeMushroomKind::Brown,
        &[
            (Direction::Down, true),
            (Direction::East, false),
            (Direction::North, true),
        ],
    );
    assert!(!faces.down);
    assert!(faces.east);
    assert!(!faces.north);
    assert_eq!(
        HugeMushroomFaces {
            down: false,
            east: false,
            north: false,
            south: false,
            up: false,
            west: false,
        }
        .state_id(HugeMushroomKind::Brown),
        7_829
    );
    assert_eq!(
        huge_mushroom_drop(HugeMushroomKind::Stem, false, 2),
        HugeMushroomDrop::Nothing
    );
    assert_eq!(
        huge_mushroom_drop(HugeMushroomKind::Red, true, -6),
        HugeMushroomDrop::Block
    );
}

#[test]
fn nether_fungi_separate_support_from_bone_meal_base() {
    assert_eq!(NetherFungus::Crimson.state_id(), 20_975);
    assert_eq!(NetherFungus::Warped.potted_state_id(), 21_827);
    assert!(NetherFungus::Crimson.valid_bone_meal_target(true, true));
    assert!(!NetherFungus::Crimson.valid_bone_meal_target(false, true));
    assert!(fungus_bone_meal_succeeds(0.399));
    assert!(!fungus_bone_meal_succeeds(0.4));
}

#[test]
fn crop_speed_growth_and_beetroot_outer_draw_are_source_ordered() {
    let speed = growth_speed(
        &[
            FarmlandCell {
                grows_crop: true,
                moisture: 7,
                off_center: false,
            },
            FarmlandCell {
                grows_crop: true,
                moisture: 0,
                off_center: true,
            },
        ],
        false,
        false,
        false,
    );
    assert_eq!(speed, 4.25);
    assert_eq!(
        random_tick(CropKind::Beetroot, 0, Some(0), 15, speed, 0),
        CropTick::BeetrootOuterRejected
    );
    assert_eq!(
        random_tick(CropKind::Wheat, 0, None, 9, speed, 0),
        CropTick::Advance { age: 1, flags: 2 }
    );
    assert_eq!(CropKind::Potatoes.state_id(7), Some(10_674));
    assert_eq!(bone_meal_growth(CropKind::Wheat, 3), 5);
    assert_eq!(bone_meal_growth(CropKind::Beetroot, 0), 0);
}

#[test]
fn cocoa_pitcher_torchflower_and_stems_keep_distinct_transitions() {
    let cocoa = CocoaState::new(1).unwrap();
    assert_eq!(cocoa.state_id(Direction::West), Some(9_487));
    assert_eq!(cocoa.random_tick(0), CocoaState::new(2));
    assert_eq!(cocoa.bone_meal().loot_count(), 3);

    let pitcher = PitcherState {
        age: 2,
        half: PitcherHalf::Lower,
    };
    assert_eq!(pitcher.state_id(), Some(14_804));
    assert_eq!(
        pitcher_grow(pitcher, 8, true, true),
        PitcherGrowth::Double {
            lower: PitcherState {
                age: 3,
                half: PitcherHalf::Lower,
            },
            lower_flags: 2,
            upper: PitcherState {
                age: 3,
                half: PitcherHalf::Upper,
            },
            upper_flags: 3,
        }
    );
    assert_eq!(
        pitcher_random_tick(pitcher, 4.0, 1, 0, false, false),
        PitcherRandomTick::GrowthDrawRejected { bound: 7 }
    );
    let crop = torchflower_advance(0);
    let flower = torchflower_advance(1);
    assert_eq!(crop, TorchflowerState::Crop(1));
    assert_eq!(flower, TorchflowerState::Flower);
    assert_eq!(crop.state_id(), Some(14_798));
    assert_eq!(flower.state_id(), Some(2_323));
    assert_eq!(
        torchflower_random_tick(0, 0, 15, 1.0, 0),
        CropTick::BeetrootOuterRejected
    );
    assert_eq!(StemKind::Pumpkin.state_id(7), Some(8_349));
    assert_eq!(
        StemKind::Melon.attached_state_id(Direction::East),
        Some(8_341)
    );

    assert_eq!(
        stem_random_tick(6, 9, 1.0, 0, false, false),
        StemTick::Age(7)
    );
    assert_eq!(
        stem_random_tick(7, 9, 1.0, 0, true, true),
        StemTick::Fruit {
            fruit_flags: 3,
            attached_flags: 3,
        }
    );
    assert_eq!(stem_bone_meal_age(4, 3), (7, true));
}

#[test]
fn berry_growth_harvest_and_motion_damage_use_strict_boundaries() {
    assert_eq!(berry_state_id(0), Some(20_941));
    assert_eq!(berry_state_id(3), Some(20_944));
    assert_eq!(berry_random_growth(2, 0, 9), Some(3));
    assert_eq!(berry_random_growth(2, 1, 15), None);
    assert_eq!(
        berry_harvest(3, 1),
        Some(BerryHarvest {
            berries: 3,
            next_age: 1,
            flags: 2,
        })
    );
    assert!(!berry_contact_damage(3, true, 1.0, 1.0));
    assert!(berry_contact_damage(1, false, 0.003, 0.0));
    assert!(!berry_contact_damage(1, false, 0.002_999, 0.0));
}

#[test]
fn copper_weathering_aborts_on_younger_neighbors_and_uses_strict_thresholds() {
    assert_eq!(
        CopperFullKind::Block.state_id(CopperAge::Unaffected, false),
        27_782
    );
    assert_eq!(
        CopperFullKind::Chiseled.state_id(CopperAge::Oxidized, true),
        27_807
    );
    assert_eq!(
        weathering_decision(CopperAge::Unaffected, 0.05688889, false, 0, 0, None),
        WeatheringDecision::FirstDrawRejected
    );
    assert_eq!(
        weathering_decision(CopperAge::Exposed, 0.0, true, 0, 4, None),
        WeatheringDecision::YoungerNeighborAbort
    );
    assert_eq!(
        weathering_decision(CopperAge::Weathered, 0.0, false, 0, 0, Some(0.999)),
        WeatheringDecision::Advance(CopperAge::Oxidized)
    );
    assert_eq!(
        axe_transform(CopperAge::Weathered, false),
        AxeCopperTransform::Scrape(CopperAge::Exposed)
    );
    assert_eq!(
        axe_transform(CopperAge::Oxidized, true),
        AxeCopperTransform::WaxOff(CopperAge::Oxidized)
    );
}

#[test]
fn copper_statues_cycle_pose_and_preserve_weather_clock_sentinels() {
    let state = StatueState {
        age: CopperAge::Exposed,
        waxed: true,
        pose: StatuePose::Star,
        facing: Direction::East,
        waterlogged: true,
    };
    let cycled = statue_use_non_axe(state);
    assert_eq!(statue_family_state_count(), 256);
    assert_eq!(state.state_id(), Some(29_950));
    let mut state_ids = BTreeSet::new();
    for age in [
        CopperAge::Unaffected,
        CopperAge::Exposed,
        CopperAge::Weathered,
        CopperAge::Oxidized,
    ] {
        for waxed in [false, true] {
            for pose in [
                StatuePose::Standing,
                StatuePose::Sitting,
                StatuePose::Running,
                StatuePose::Star,
            ] {
                for facing in Direction::HORIZONTAL {
                    for waterlogged in [true, false] {
                        state_ids.insert(
                            StatueState {
                                age,
                                waxed,
                                pose,
                                facing,
                                waterlogged,
                            }
                            .state_id()
                            .unwrap(),
                        );
                    }
                }
            }
        }
    }
    assert_eq!(state_ids.len(), 256);
    assert_eq!(
        (state_ids.first().copied(), state_ids.last().copied(),),
        (Some(29_760), Some(30_015))
    );
    assert_eq!(cycled.pose, StatuePose::Standing);
    assert_eq!(cycled.pose.comparator_output(), 1);
    assert_eq!(
        copper_golem_weather_tick(-2, 10, CopperAge::Unaffected, 504_000),
        GolemWeatherClock::Waxed
    );
    assert_eq!(
        copper_golem_weather_tick(-1, 10, CopperAge::Unaffected, 504_000),
        GolemWeatherClock::Initialize { deadline: 504_010 }
    );
    assert_eq!(
        copper_golem_weather_tick(10, 10, CopperAge::Weathered, 504_000),
        GolemWeatherClock::Advanced {
            age: CopperAge::Oxidized,
            next_deadline: 0,
        }
    );
    assert!(golem_statue_conversion_admitted(true, 0.0058));
    assert!(!golem_statue_conversion_admitted(true, 0.005_801));
}

#[test]
fn frogspawn_and_lily_pad_lock_source_water_and_nonatomic_hatching() {
    assert_eq!(FROGSPAWN_STATE_ID, 32_084);
    assert_eq!(LILY_PAD_STATE_ID, 8_920);
    assert_eq!(frogspawn_interval(0, 3_600, 11_999), 3_600);
    assert_eq!(frogspawn_interval(20_000, 3_600, 11_999), 11_999);
    assert_eq!(frogspawn_due(true, 6), FrogspawnTick::Hatch { tadpoles: 5 });
    assert_eq!(frogspawn_due(false, 2), FrogspawnTick::DestroyUnsupported);
    assert_eq!(tadpole_horizontal_offset(0.0), 0.2);
    assert_eq!(tadpole_horizontal_offset(1.0), 0.8);
    assert_eq!(tadpole_yaw(0), 1);
    assert_eq!(tadpole_yaw(361), 360);

    assert!(lily_pad_survives(true, true));
    assert!(!lily_pad_survives(true, false));
    let contact = lily_pad_boat_contact(true, true).unwrap();
    assert!(contact.destroy_with_drops && contact.breaking_entity_is_boat);
    assert!(lily_pad_boat_contact(false, true).is_none());
}

#[test]
fn lodestone_binding_and_lazy_tracker_validation_are_exact() {
    assert_eq!((LODESTONE_BLOCK_ID, LODESTONE_STATE_ID), (923, 21_830));
    assert_eq!(POI_MAX_TICKETS, 0);
    let in_place = bind_compass(1, false);
    assert!(in_place.mutate_held_in_place);
    assert_eq!(in_place.consume_source, 0);
    let split = bind_compass(5, false);
    assert!(split.create_bound_copy && split.drop_if_inventory_full);
    assert_eq!(split.consume_source, 1);
    let creative = bind_compass(1, true);
    assert_eq!(creative.consume_source, 0);
    assert!(creative.create_bound_copy);

    let tracker = Tracker {
        has_target: true,
        tracked: true,
    };
    assert_eq!(
        tracker_tick(tracker, false, false, false),
        TrackerTick::SameRecord
    );
    assert_eq!(
        tracker_tick(tracker, true, false, true),
        TrackerTick::ClearTarget
    );
    assert_eq!(
        tracker_tick(tracker, true, true, true),
        TrackerTick::SameRecord
    );
}

#[test]
fn moss_mud_nylium_and_packed_ice_preserve_environment_boundaries() {
    assert_eq!(MossKind::Block.state_id(), 30_355);
    assert!(MossKind::Block.survives(true));
    assert!(!MossKind::Carpet.survives(true));
    assert_eq!(MossKind::Carpet.compost_chance(), 0.3);
    assert!(mud_dripstone_converts_to_clay(0.175));
    assert!(!mud_dripstone_converts_to_clay(0.17578125));
    assert_eq!(packed_ice_friction(), 0.98);

    assert!(Nylium::Crimson.random_tick_decays(15));
    assert!(!Nylium::Warped.random_tick_decays(14));
    let warped = nylium_bone_meal(Nylium::Warped, Some(0));
    assert!(warped.primary_feature && warped.sprouts_feature && warped.twisting_feature);
    assert_eq!(
        netherrack_conversion(true, true, Some(true)),
        Some(Nylium::Warped)
    );
    assert_eq!(netherrack_conversion(false, false, None), None);
}

#[test]
fn snow_layers_lock_geometry_support_stacking_and_light_melt() {
    assert_eq!(SNOW_BLOCK_STATE_ID, 6_928);
    assert_eq!(POWDER_SNOW_STATE_ID, 27_162);
    let one = SnowLayer::new(1).unwrap();
    let eight = SnowLayer::new(8).unwrap();
    assert_eq!((one.state_id(), eight.state_id()), (6_919, 6_926));
    assert_eq!(one.outline_height_sixteenths(), 2);
    assert_eq!(one.collision_height_sixteenths(), 0);
    assert_eq!(eight.collision_height_sixteenths(), 14);
    assert!(one.land_pathfindable());
    assert!(!eight.land_pathfindable());
    assert!(one.can_stack(true, true, true));
    assert!(!one.can_stack(true, true, false));
    assert_eq!(one.stacked().layers, 2);
    assert!(!snow_layer_survives(true, true, true, true));
    assert!(snow_layer_survives(false, true, false, false));
    assert!(!snow_layer_melts(11));
    assert!(snow_layer_melts(12));
}

#[test]
fn powder_snow_collision_freezing_and_fall_sounds_use_exact_boundaries() {
    assert_eq!(
        powder_snow_collision(true, 2.5001, false, false, false, false),
        PowderCollision::FallPlatform
    );
    assert_eq!(
        powder_snow_collision(true, 0.0, false, true, true, false),
        PowderCollision::FullCube
    );
    assert_eq!(
        powder_snow_collision(false, 10.0, true, true, true, false),
        PowderCollision::Empty
    );
    assert!(powder_snow_walkable(false, true));
    assert_eq!(frozen_ticks(139, true), 140);
    assert_eq!(frozen_ticks(1, false), 0);
    assert!(freeze_damage_due(140, 80));
    assert!(!freeze_damage_due(139, 80));
    assert_eq!(freeze_damage(true), 5);
    assert_eq!(powder_snow_fall_sound(3.999), None);
    assert_eq!(powder_snow_fall_sound(4.0), Some(false));
    assert_eq!(powder_snow_fall_sound(7.0), Some(true));
}

#[test]
fn sponge_absorption_caps_accepted_nodes_and_keeps_branch_precedence() {
    assert_eq!((SPONGE_STATE_ID, WET_SPONGE_STATE_ID), (560, 561));
    assert_eq!(ABSORPTION_NODE_CAP, 65);
    assert_eq!(
        absorb_candidate(WaterCandidate::BucketPickup {
            returned_nonempty: true,
        }),
        AbsorbAction::AcceptBucketPickup
    );
    assert_eq!(
        absorb_candidate(WaterCandidate::BucketPickup {
            returned_nonempty: false,
        }),
        AbsorbAction::Skip
    );
    assert_eq!(
        absorb_candidate(WaterCandidate::LiquidBlock),
        AbsorbAction::AcceptLiquidWriteAir
    );
    assert_eq!(
        absorb_candidate(WaterCandidate::WaterPlant),
        AbsorbAction::AcceptDropPlantThenWriteAir
    );
    assert!(!absorption_result(1).wet_write);
    assert!(absorption_result(2).play_absorb_sound);
    assert!(furnace_bucket_result(true, true));
    assert!(!furnace_bucket_result(true, false));
    let dry = wet_sponge_on_place(true, 0.5).unwrap();
    assert_eq!((dry.write_dry_flags, dry.level_event), (3, 2_009));
    assert_eq!(dry.pitch, 0.77);
}

#[test]
fn sniffer_egg_reschedules_each_crack_and_hatches_nonatomically() {
    let egg = SnifferEgg::new(0).unwrap();
    assert_eq!(egg.state_id(), 15_102);
    assert_eq!(egg.scheduled_delay(true, 299), 4_299);
    assert_eq!(egg.scheduled_delay(false, 500), 8_299);
    assert_eq!(
        sniffer_egg_due(egg, 0.5),
        SnifferEggDue::Crack {
            next: SnifferEgg { hatch: 1 },
            pitch: 1.0,
            flags: 2,
        }
    );
    assert_eq!(
        sniffer_egg_due(SnifferEgg { hatch: 2 }, 0.0),
        SnifferEggDue::Hatch {
            pitch: 0.9,
            destroy_with_drops: false,
            create_baby: true,
        }
    );
    assert_eq!(sniffer_yaw(0.75), -90.0);
}

#[test]
fn cobweb_stores_deferred_multiplier_and_spiders_skip_contact_hook() {
    assert_eq!(COBWEB_STATE_ID, 2_247);
    assert!(cobweb_contact(false, true).is_none());
    let ordinary = cobweb_contact(false, false).unwrap();
    assert_eq!((ordinary.x, ordinary.z), (0.25, 0.25));
    let weaving = cobweb_contact(true, false).unwrap();
    assert_eq!((weaving.x, weaving.y, weaving.z), (0.5, 0.25, 0.5));
    assert_eq!(
        apply_deferred_stuck([1.0, 1.0, 1.0], ordinary, false),
        [0.25, 0.05000000074505806, 0.25]
    );
    assert_eq!(
        apply_deferred_stuck([1.0, 1.0, 1.0], ordinary, true),
        [1.0, 1.0, 1.0]
    );
}

#[test]
fn saplings_lock_stage_draw_height_and_small_feature_selection() {
    assert_eq!(SaplingKind::Oak.state_id(0), Some(29));
    assert_eq!(SaplingKind::PaleOak.state_id(1), Some(44));
    assert_eq!(SaplingKind::Cherry.minimum_height(), 7);
    assert_eq!(sapling_random_tick(0, 8, 0), SaplingTick::TooDark);
    assert_eq!(sapling_random_tick(0, 9, 0), SaplingTick::StageOne);
    assert_eq!(sapling_random_tick(1, 9, 0), SaplingTick::GrowTree);
    assert!(sapling_bone_meal_succeeds(0.449));
    assert!(!sapling_bone_meal_succeeds(0.45));
    assert_eq!(
        small_tree_choice(SaplingKind::Oak, 0.099),
        SmallTreeChoice::FancyOak
    );
    assert_eq!(
        small_tree_choice(SaplingKind::Spruce, 0.5),
        SmallTreeChoice::Primary
    );
}
