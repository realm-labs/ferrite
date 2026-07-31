use ferrite_world::generation::beardifier::{
    BeardJunction, BeardPiece, Beardifier, TerrainAdjustment,
};
use ferrite_world::generation::density::DensityContext;

#[test]
fn empty_beardifier_is_exact_zero_with_zero_bounds() {
    let beardifier = Beardifier::new(Vec::new(), Vec::new());

    assert_eq!(beardifier.sample(context(0, 0, 0)), 0.0);
    assert_eq!(beardifier.bounds(), (0.0, 0.0));
    assert_eq!(beardifier.affected_box(), None);
}

#[test]
fn affected_union_inflates_piece_and_junction_extents_by_twenty_four() {
    let beardifier = Beardifier::new(
        vec![piece(TerrainAdjustment::None)],
        vec![BeardJunction {
            source_x: 20,
            source_ground_y: -5,
            source_z: 30,
        }],
    );

    assert_eq!(
        beardifier.affected_box(),
        Some(([-24, -29, -24], [44, 34, 54]))
    );
    assert_eq!(beardifier.sample(context(100, 0, 0)), 0.0);
}

#[test]
fn bury_center_is_one_while_none_contributes_zero() {
    let bury = Beardifier::new(vec![piece(TerrainAdjustment::Bury)], Vec::new());
    let none = Beardifier::new(vec![piece(TerrainAdjustment::None)], Vec::new());

    assert_eq!(bury.sample(context(5, 0, 5)), 1.0);
    assert_eq!(none.sample(context(5, 0, 5)), 0.0);
}

#[test]
fn beard_sign_tracks_raw_ground_y_across_the_half_block_boundary() {
    let beardifier = Beardifier::new(vec![piece(TerrainAdjustment::BeardThin)], Vec::new());

    assert!(beardifier.sample(context(5, 0, 5)) < 0.0);
    assert!(beardifier.sample(context(5, -1, 5)) > 0.0);
}

fn piece(adjustment: TerrainAdjustment) -> BeardPiece {
    BeardPiece {
        minimum: [0, 0, 0],
        maximum: [10, 10, 10],
        ground_level_delta: 0,
        adjustment,
    }
}

fn context(x: i32, y: i32, z: i32) -> DensityContext {
    DensityContext { x, y, z }
}
