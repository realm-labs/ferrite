use std::collections::BTreeSet;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::dimension::{DimensionType, LockedDimension, Position};
use ferrite_world::generation::portal::nether::{
    NETHER_SEARCH_RADIUS, NetherExitInput, OVERWORLD_SEARCH_RADIUS, PortalBlock, PortalBorder,
    PortalCreationSiteKind, PortalCreationWorld, PortalPoi, PortalWrite, create_portal,
    largest_matching_rectangle, nether_destination_key, nether_exit, portal_search_plan,
    portal_search_radius, relative_entry_position, scaled_search_block, select_portal_poi,
    spiral_columns,
};
use ferrite_world::generation::portal::{HorizontalAxis, PortalRectangle, Rotation, Vec3};

fn border() -> PortalBorder {
    PortalBorder {
        minimum_x: -100.0,
        maximum_x: 100.0,
        minimum_z: -100.0,
        maximum_z: 100.0,
    }
}

#[test]
fn nether_route_scales_clamps_and_floors_after_key_selection() {
    assert_eq!(
        nether_destination_key("minecraft:the_nether"),
        "minecraft:overworld"
    );
    assert_eq!(
        nether_destination_key("example:custom"),
        "minecraft:the_nether"
    );
    assert_eq!(
        portal_search_radius("minecraft:the_nether"),
        NETHER_SEARCH_RADIUS
    );
    assert_eq!(
        portal_search_radius("minecraft:overworld"),
        OVERWORLD_SEARCH_RADIUS
    );
    assert_eq!(
        portal_search_plan(BlockPos::new(1, 2, 3), "minecraft:the_nether"),
        ferrite_world::generation::portal::nether::PortalSearchPlan {
            center: BlockPos::new(1, 2, 3),
            radius: 16,
            ensure_loaded_and_valid: true,
            inclusive_xz_square: true,
        }
    );
    let source = DimensionType::locked(LockedDimension::TheNether);
    let destination = DimensionType::locked(LockedDimension::Overworld);
    assert_eq!(
        scaled_search_block(
            Position {
                x: 20.25,
                y: 64.75,
                z: -20.25
            },
            &source,
            &destination,
            PortalBorder {
                minimum_x: -50.0,
                maximum_x: 50.0,
                minimum_z: -50.0,
                maximum_z: 50.0
            },
        ),
        BlockPos::new(49, 64, -50)
    );
    assert!(border().contains(BlockPos::new(-100, 0, -100)));
    assert!(!border().contains(BlockPos::new(100, 0, 100)));
}

#[test]
fn poi_search_uses_inclusive_square_3d_distance_y_and_encounter_ties() {
    let target = BlockPos::new(0, 64, 0);
    let selected = select_portal_poi(
        target,
        "minecraft:the_nether",
        border(),
        [
            PortalPoi {
                position: BlockPos::new(17, 64, 0),
                axis: Some(HorizontalAxis::X),
                encounter_order: 0,
            },
            PortalPoi {
                position: BlockPos::new(16, 70, 0),
                axis: Some(HorizontalAxis::X),
                encounter_order: 1,
            },
            PortalPoi {
                position: BlockPos::new(0, 70, 0),
                axis: None,
                encounter_order: 2,
            },
            PortalPoi {
                position: BlockPos::new(0, 60, 6),
                axis: Some(HorizontalAxis::Z),
                encounter_order: 9,
            },
            PortalPoi {
                position: BlockPos::new(0, 68, 6),
                axis: Some(HorizontalAxis::X),
                encounter_order: 3,
            },
        ],
    )
    .unwrap();
    assert_eq!(selected.position, BlockPos::new(0, 60, 6));

    let tied = select_portal_poi(
        target,
        "minecraft:the_nether",
        border(),
        [
            PortalPoi {
                position: BlockPos::new(3, 64, 4),
                axis: Some(HorizontalAxis::X),
                encounter_order: 8,
            },
            PortalPoi {
                position: BlockPos::new(-3, 64, -4),
                axis: Some(HorizontalAxis::Z),
                encounter_order: 2,
            },
        ],
    )
    .unwrap();
    assert_eq!(tied.encounter_order, 2);
}

#[test]
fn largest_rectangle_uses_state_identity_caps_dimensions_and_can_narrow_for_height() {
    let origin = BlockPos::new(0, 0, 5);
    let rectangle = largest_matching_rectangle(origin, HorizontalAxis::X, |position| {
        position.z == 5
            && ((position.y == 0 && (-10..=10).contains(&position.x))
                || ((-10..=10).contains(&position.y) && (0..=1).contains(&position.x)))
    });
    assert_eq!(
        rectangle,
        PortalRectangle {
            minimum: BlockPos::new(0, -10, 5),
            axis: HorizontalAxis::X,
            width: 2,
            height: 21,
        }
    );
    assert!(rectangle.contains(BlockPos::new(1, 10, 5)));
    assert!(!rectangle.contains(BlockPos::new(2, 10, 5)));
}

#[derive(Clone)]
struct World {
    border: PortalBorder,
    height: i32,
    dry: BTreeSet<BlockPos>,
    solid: BTreeSet<BlockPos>,
}

impl PortalCreationWorld for World {
    fn border(&self) -> PortalBorder {
        self.border
    }

    fn motion_blocking_height(&self, _x: i32, _z: i32) -> i32 {
        self.height
    }

    fn is_dry_replaceable(&self, position: BlockPos) -> bool {
        self.dry.contains(&position)
    }

    fn is_solid(&self, position: BlockPos) -> bool {
        self.solid.contains(&position)
    }
}

fn site_world(preferred: bool) -> World {
    let mut dry = BTreeSet::new();
    let mut solid = BTreeSet::new();
    let planes: &[i32] = if preferred { &[-1, 0, 1] } else { &[0] };
    for plane in planes {
        for width in -1..=2 {
            solid.insert(BlockPos::new(width, 69, *plane));
            for y in 70..=73 {
                dry.insert(BlockPos::new(width, y, *plane));
            }
        }
    }
    World {
        border: border(),
        height: 73,
        dry,
        solid,
    }
}

#[test]
fn creation_prefers_full_site_then_center_only_and_writes_exact_frame() {
    let target = BlockPos::new(0, 70, 0);
    let preferred =
        create_portal(&site_world(true), target, HorizontalAxis::X, 0, 255, 128).unwrap();
    assert_eq!(preferred.site_kind, PortalCreationSiteKind::Preferred);
    assert_eq!(preferred.rectangle.minimum, target);
    assert_eq!(preferred.writes.len(), 20);
    assert_eq!(
        preferred
            .writes
            .iter()
            .filter(|write| matches!(write.block, PortalBlock::Obsidian))
            .count(),
        14
    );
    assert_eq!(
        preferred
            .writes
            .iter()
            .filter(|write| matches!(write.block, PortalBlock::Portal(HorizontalAxis::X)))
            .count(),
        6
    );
    assert!(
        preferred.writes[..14]
            .iter()
            .all(|write| { write.block == PortalBlock::Obsidian && write.flags == 3 })
    );
    assert!(preferred.writes[14..].iter().all(|write| {
        write.block == PortalBlock::Portal(HorizontalAxis::X) && write.flags == 18
    }));

    let center = create_portal(&site_world(false), target, HorizontalAxis::X, 0, 255, 128).unwrap();
    assert_eq!(center.site_kind, PortalCreationSiteKind::CenterOnly);
    assert!(create_portal(&site_world(true), target, HorizontalAxis::X, 0, 255, 74).is_none());
    assert!(create_portal(&site_world(true), target, HorizontalAxis::X, 0, 255, 75).is_some());
}

#[test]
fn creation_fallback_clamps_y_builds_clearance_and_rejects_inverted_range() {
    let world = World {
        border: border(),
        height: 0,
        dry: BTreeSet::new(),
        solid: BTreeSet::new(),
    };
    let fallback = create_portal(
        &world,
        BlockPos::new(0, 500, 0),
        HorizontalAxis::Z,
        0,
        255,
        128,
    )
    .unwrap();
    assert_eq!(fallback.site_kind, PortalCreationSiteKind::Fallback);
    assert_eq!(fallback.rectangle.minimum, BlockPos::new(0, 118, -1));
    assert_eq!(fallback.writes.len(), 44);
    assert_eq!(
        fallback.writes[0],
        PortalWrite {
            position: BlockPos::new(1, 117, -1),
            block: PortalBlock::Obsidian,
            flags: 3,
        }
    );
    assert_eq!(
        fallback.writes[24],
        PortalWrite {
            position: BlockPos::new(0, 117, -2),
            block: PortalBlock::Obsidian,
            flags: 3,
        }
    );
    assert_eq!(
        fallback.writes[38],
        PortalWrite {
            position: fallback.rectangle.minimum,
            block: PortalBlock::Portal(HorizontalAxis::Z),
            flags: 18,
        }
    );
    assert_eq!(
        fallback
            .writes
            .iter()
            .filter(|write| matches!(write.block, PortalBlock::Obsidian))
            .count(),
        20
    );
    assert_eq!(
        fallback
            .writes
            .iter()
            .filter(|write| matches!(write.block, PortalBlock::Air))
            .count(),
        18
    );
    assert!(fallback.writes[38..].iter().all(|write| {
        matches!(write.block, PortalBlock::Portal(HorizontalAxis::Z)) && write.flags == 18
    }));
    assert!(
        create_portal(
            &world,
            BlockPos::new(0, 0, 0),
            HorizontalAxis::X,
            -64,
            319,
            16
        )
        .is_none()
    );
}

#[test]
fn spiral_starts_east_south_and_covers_each_column_once() {
    let target = BlockPos::new(5, 7, 9);
    let columns = spiral_columns(target, 2);
    assert_eq!(columns.len(), 25);
    assert_eq!(
        columns[..5],
        [
            target,
            BlockPos::new(6, 7, 9),
            BlockPos::new(6, 7, 10),
            BlockPos::new(5, 7, 10),
            BlockPos::new(4, 7, 10),
        ]
    );
    assert_eq!(columns.iter().copied().collect::<BTreeSet<_>>().len(), 25);
    assert_eq!(spiral_columns(target, -1), [target]);
}

#[test]
fn exit_geometry_preserves_motion_pitch_adjusts_yaw_and_tickets_correct_block() {
    let source = PortalRectangle {
        minimum: BlockPos::new(0, 10, 0),
        axis: HorizontalAxis::X,
        width: 4,
        height: 5,
    };
    let (axis, relative) = relative_entry_position(
        Some(source),
        Vec3 {
            x: 2.0,
            y: 12.0,
            z: 0.75,
        },
        1.0,
        2.0,
    );
    assert_eq!(axis, HorizontalAxis::X);
    assert!((relative.horizontal_fraction - 0.5).abs() < f64::EPSILON);
    assert!((relative.vertical_fraction - 2.0 / 3.0).abs() < f64::EPSILON);
    assert_eq!(relative.perpendicular_offset, 0.25);

    let destination = PortalRectangle {
        minimum: BlockPos::new(20, 30, 40),
        axis: HorizontalAxis::Z,
        width: 3,
        height: 4,
    };
    let mut adjustment_calls = 0;
    let exit = nether_exit(
        NetherExitInput {
            destination,
            source_axis: axis,
            relative,
            entity_size: [1.0, 2.0],
            velocity: Vec3 {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            rotation: Rotation {
                yaw: 15.0,
                pitch: -10.0,
            },
            is_server_player: true,
            existing_poi: Some(BlockPos::new(20, 30, 40)),
        },
        |position, volume| {
            adjustment_calls += 1;
            assert_eq!(volume, [3.0, 5.0, 3.0]);
            Some(Vec3 {
                y: position.y + 1.0,
                ..position
            })
        },
    );
    assert_eq!(
        exit.velocity,
        Vec3 {
            x: 1.0,
            y: 2.0,
            z: 3.0
        }
    );
    assert_eq!(
        exit.rotation,
        Rotation {
            yaw: 105.0,
            pitch: -10.0
        }
    );
    assert_eq!(exit.ticket.position, BlockPos::new(20, 30, 40));
    assert_eq!(exit.player_level_event, Some(1032));
    assert_eq!(adjustment_calls, 1);

    let (_, missing) = relative_entry_position(None, Vec3::ZERO, 9.0, 9.0);
    assert_eq!(
        (
            missing.horizontal_fraction,
            missing.vertical_fraction,
            missing.perpendicular_offset
        ),
        (0.5, 0.0, 0.0)
    );
}

#[test]
fn oversized_or_colliding_exit_keeps_computed_position_and_new_portal_tickets_final_floor() {
    let destination = PortalRectangle {
        minimum: BlockPos::new(-2, 4, -6),
        axis: HorizontalAxis::X,
        width: 2,
        height: 3,
    };
    let exit = nether_exit(
        NetherExitInput {
            destination,
            source_axis: HorizontalAxis::X,
            relative: ferrite_world::generation::portal::nether::PortalRelativePosition {
                horizontal_fraction: 0.5,
                vertical_fraction: 0.0,
                perpendicular_offset: -0.25,
            },
            entity_size: [5.0, 5.0],
            velocity: Vec3::ZERO,
            rotation: Rotation {
                yaw: 0.0,
                pitch: 0.0,
            },
            is_server_player: false,
            existing_poi: None,
        },
        |_, _| panic!("large entity skips collision search"),
    );
    assert_eq!(
        exit.position,
        Vec3 {
            x: -1.0,
            y: 4.0,
            z: -5.75
        }
    );
    assert_eq!(exit.ticket.position, BlockPos::new(-1, 4, -6));
    assert_eq!(exit.player_level_event, None);
}
