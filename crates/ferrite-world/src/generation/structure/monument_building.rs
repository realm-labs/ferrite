//! Ocean-monument building orchestration, foundations, and water skirt.

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::BlockBox;
use crate::generation::structure::monument_graph::MonumentGraph;
use crate::generation::structure::monument_place::{
    MonumentWorld, box_tuple, bricks, fill_column_down, place_monument_child, placement, water_box,
};
use crate::generation::structure::monument_shell_front::{
    entrance_arches, entrance_wall, roof_piece, wing,
};
use crate::generation::structure::monument_shell_walls::{lower_wall, middle_wall, upper_wall};
use crate::generation::structure::piece::PiecePlacement;

pub(crate) fn place_monument_building(
    world: &mut impl MonumentWorld,
    graph: &MonumentGraph,
    clip: &BlockBox,
    random: &mut impl GenerationRandom,
) {
    let p = placement(&graph.building, clip);
    let water_height = world.sea_level().max(64) - graph.building.bounding_box.minimum.y;
    water_box(world, p, pos(0, 0, 0), pos(58, water_height, 58));
    wing(world, p, false, 0);
    wing(world, p, true, 33);
    entrance_arches(world, p);
    entrance_wall(world, p);
    roof_piece(world, p);
    lower_wall(world, p);
    middle_wall(world, p);
    upper_wall(world, p);
    foundations(world, p);
    water_skirt(world, p);
    for child in &graph.children {
        if child.bounding_box.intersects(*clip) {
            place_monument_child(world, graph, child, clip, random);
        }
    }
}

fn foundations(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    for pillar_x in 0..7 {
        let zs: &[i32] = if pillar_x == 0 || pillar_x == 6 {
            &[0, 1, 2, 3, 4, 5, 6]
        } else {
            &[0, 6]
        };
        for &pillar_z in zs {
            let bx = pillar_x * 9;
            let bz = pillar_z * 9;
            for x in bx..bx + 4 {
                for z in bz..bz + 4 {
                    p.place_block(world, pos(x, 0, z), bricks());
                    fill_column_down(world, p, x, z);
                }
            }
        }
    }
}

fn water_skirt(world: &mut impl MonumentWorld, p: PiecePlacement<'_>) {
    for i in 0..5 {
        water_box(
            world,
            p,
            pos(-1 - i, i * 2, -1 - i),
            pos(-1 - i, 23, 58 + i),
        );
        water_box(
            world,
            p,
            pos(58 + i, i * 2, -1 - i),
            pos(58 + i, 23, 58 + i),
        );
        water_box(world, p, pos(-i, i * 2, -1 - i), pos(57 + i, 23, -1 - i));
        water_box(world, p, pos(-i, i * 2, 58 + i), pos(57 + i, 23, 58 + i));
    }
}

pub(crate) fn intersects(
    p: PiecePlacement<'_>,
    minimum_x: i32,
    minimum_z: i32,
    maximum_x: i32,
    maximum_z: i32,
) -> bool {
    let corners = [
        p.piece.world_position(pos(minimum_x, 0, minimum_z)),
        p.piece.world_position(pos(minimum_x, 0, maximum_z)),
        p.piece.world_position(pos(maximum_x, 0, minimum_z)),
        p.piece.world_position(pos(maximum_x, 0, maximum_z)),
    ];
    let world_minimum_x = corners.iter().map(|point| point.x).min().unwrap();
    let world_maximum_x = corners.iter().map(|point| point.x).max().unwrap();
    let world_minimum_z = corners.iter().map(|point| point.z).min().unwrap();
    let world_maximum_z = corners.iter().map(|point| point.z).max().unwrap();
    p.clip.minimum.x <= world_maximum_x
        && p.clip.maximum.x >= world_minimum_x
        && p.clip.minimum.z <= world_maximum_z
        && p.clip.maximum.z >= world_minimum_z
}

pub(crate) fn b(
    world: &mut impl MonumentWorld,
    p: PiecePlacement<'_>,
    minimum: (i32, i32, i32),
    maximum: (i32, i32, i32),
    state: crate::generation::structure::processor::StructureState,
) {
    box_tuple(world, p, minimum, maximum, state);
}

pub(crate) fn pos(x: i32, y: i32, z: i32) -> BlockPos {
    BlockPos::new(x, y, z)
}
