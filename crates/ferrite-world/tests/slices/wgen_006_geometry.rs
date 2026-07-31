use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::border::BORDER_EPSILON;
use ferrite_world::generation::border::BorderPoint3;
use ferrite_world::generation::border::collision::{BorderFace, BorderRayHit};
use ferrite_world::generation::border::geometry::{BorderAabb, BorderChunk};
use ferrite_world::generation::border::state::BorderStatus;
use ferrite_world::generation::border::state::WorldBorder;

fn border(size: f64) -> WorldBorder {
    let mut border = WorldBorder::default();
    border.set_size(size);
    border
}

#[test]
fn point_block_radius_and_chunk_edges_are_minimum_inclusive_maximum_exclusive() {
    let border = border(32.0);
    assert!(border.contains_point(-16.0, -16.0));
    assert!(!border.contains_point(16.0, 0.0));
    assert!(border.contains_point_with_radius(17.0, 0.0, 2.0));
    assert!(!border.contains_point_with_radius(18.0, 0.0, 2.0));
    assert!(border.contains_block(BlockPos::new(-16, 99, 15)));
    assert!(!border.contains_block(BlockPos::new(16, -99, 0)));
    assert!(border.contains_chunk(BorderChunk { x: -1, z: 0 }));
    assert!(!border.contains_chunk(BorderChunk { x: 1, z: 0 }));
    let wrapped = BorderChunk {
        x: i32::MAX,
        z: i32::MIN,
    };
    assert_eq!(wrapped.minimum_block_x(), -16);
    assert_eq!(wrapped.maximum_block_x(), -1);
    assert_eq!(wrapped.minimum_block_z(), 0);
    assert_eq!(wrapped.maximum_block_z(), 15);
}

#[test]
fn aabb_epsilon_accepts_an_exact_maximum_face_but_not_a_maximum_origin() {
    let border = border(10.0);
    assert!(border.contains_aabb(BorderAabb {
        minimum_x: -5.0,
        minimum_y: -100.0,
        minimum_z: -5.0,
        maximum_x: 5.0,
        maximum_y: 100.0,
        maximum_z: 5.0,
    }));
    assert!(!border.contains_aabb(BorderAabb {
        minimum_x: 5.0,
        minimum_y: 0.0,
        minimum_z: 0.0,
        maximum_x: 5.0 + BORDER_EPSILON,
        maximum_y: 1.0,
        maximum_z: 1.0,
    }));
    assert!(!border.contains_aabb(BorderAabb {
        maximum_x: 5.0 + 2.0 * BORDER_EPSILON,
        ..BorderAabb {
            minimum_x: 4.0,
            minimum_y: 0.0,
            minimum_z: 0.0,
            maximum_x: 5.0,
            maximum_y: 1.0,
            maximum_z: 1.0,
        }
    }));
}

#[test]
fn absolute_coordinate_clamp_distance_and_vector_clamp_are_exact() {
    let mut border = border(1_000.0);
    border.set_center(100.25, -100.75);
    border.set_absolute_max(120);
    let edges = border.edges();
    assert_eq!(
        (
            edges.minimum_x,
            edges.maximum_x,
            edges.minimum_z,
            edges.maximum_z,
        ),
        (-120.0, 120.0, -120.0, 120.0)
    );
    assert_eq!(border.distance_to_border(0.0, 0.0), 120.0);
    assert_eq!(border.distance_to_border(125.0, 0.0), -5.0);
    let clamped = border.clamp_vector(BorderPoint3 {
        x: 500.0,
        y: 7.25,
        z: -500.0,
    });
    assert_eq!(clamped.y, 7.25);
    assert_eq!(clamped.x, 120.0 - BORDER_EPSILON);
    assert_eq!(clamped.z, -120.0);
    assert_eq!(
        border.clamp_block(BorderPoint3 {
            x: 500.0,
            y: -1.2,
            z: -500.0,
        }),
        BlockPos::new(119, -2, -120)
    );
}

#[test]
fn collision_shape_rounds_outward_and_only_near_expanded_entities_receive_it() {
    let mut border = border(9.0);
    border.set_center(0.25, -0.25);
    let shape = border.collision_shape();
    assert_eq!(
        (
            shape.minimum_x,
            shape.maximum_x,
            shape.minimum_z,
            shape.maximum_z,
            shape.infinite_vertical,
            shape.complemented,
        ),
        (-5, 5, -5, 5, true, true)
    );
    let near_outside = BorderAabb {
        minimum_x: 4.7,
        minimum_y: 0.0,
        minimum_z: 0.0,
        maximum_x: 5.7,
        maximum_y: 2.0,
        maximum_z: 1.0,
    };
    assert!(border.collision_shape_for(near_outside).is_some());
    assert!(
        border
            .collision_shape_for(BorderAabb {
                minimum_x: 50.0,
                maximum_x: 51.0,
                ..near_outside
            })
            .is_none()
    );
}

#[test]
fn ray_clip_only_replaces_inside_to_outside_hits_and_marks_the_approximate_face() {
    let border = border(10.0);
    let start = BorderPoint3 {
        x: 0.0,
        y: 2.0,
        z: 0.0,
    };
    let hit = border.clip_including_border(
        start,
        BorderRayHit {
            location: BorderPoint3 {
                x: 20.0,
                y: 2.0,
                z: 1.0,
            },
            face: BorderFace::North,
            world_border_hit: false,
        },
    );
    assert!(hit.world_border_hit);
    assert_eq!(hit.face, BorderFace::East);
    assert_eq!(hit.location.x, 5.0 - BORDER_EPSILON);
    let tied = border.clip_including_border(
        start,
        BorderRayHit {
            location: BorderPoint3 {
                x: 20.0,
                y: 2.0,
                z: 20.0,
            },
            face: BorderFace::West,
            world_border_hit: false,
        },
    );
    assert_eq!(tied.face, BorderFace::South);
    let unchanged = border.clip_including_border(
        BorderPoint3 { x: 6.0, ..start },
        BorderRayHit {
            location: BorderPoint3 { x: 7.0, ..start },
            face: BorderFace::Up,
            world_border_hit: false,
        },
    );
    assert!(!unchanged.world_border_hit);
    assert_eq!(unchanged.location.x, 7.0);
    assert_eq!(unchanged.face, BorderFace::Up);
}

#[test]
fn direct_nan_extent_propagates_through_geometry_without_panicking() {
    let mut border = WorldBorder::default();
    border.lerp_size_between(f64::NAN, 10.0, 5, 0);
    assert_eq!(border.status(), BorderStatus::Growing);
    let edges = border.edges();
    assert!(edges.minimum_x.is_nan() && edges.maximum_x.is_nan());
    assert!(!border.contains_point(0.0, 0.0));
    assert!(border.distance_to_border(0.0, 0.0).is_nan());
    assert!(border.clamp_vector(BorderPoint3::default()).x.is_nan());
}
