//! Woodland-mansion roof and upper-wall scheduling.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::mansion_graph::{MansionLayout, SimpleGrid};
use crate::generation::structure::mansion_pieces::{
    MansionPieceSpec, above, push, relative_rotated, rotate,
};

pub(crate) fn create_roof(
    pieces: &mut Vec<MansionPieceSpec>,
    origin: BlockPos,
    rotation: Rotation,
    grid: &SimpleGrid,
    above_grid: Option<&SimpleGrid>,
    start_x: i32,
    start_y: i32,
) {
    for y in 0..grid.height() as i32 {
        for x in 0..grid.width() as i32 {
            let position = cell_position(origin, rotation, x, y, start_x, start_y);
            let covered = above_grid.is_some_and(|above| MansionLayout::is_house(above, x, y));
            if !MansionLayout::is_house(grid, x, y) || covered {
                continue;
            }
            push(pieces, "roof", above(position, 3), rotation);
            if !house(grid, x + 1, y) {
                push(
                    pieces,
                    "roof_front",
                    relative_rotated(position, rotation, Direction::East, 6),
                    rotation,
                );
            }
            if !house(grid, x - 1, y) {
                let edge = relative_rotated(position, rotation, Direction::South, 7);
                push(
                    pieces,
                    "roof_front",
                    edge,
                    rotate(rotation, Rotation::Clockwise180),
                );
            }
            if !house(grid, x, y - 1) {
                push(
                    pieces,
                    "roof_front",
                    relative_rotated(position, rotation, Direction::West, 1),
                    rotate(rotation, Rotation::CounterClockwise90),
                );
            }
            if !house(grid, x, y + 1) {
                let edge = relative_rotated(
                    relative_rotated(position, rotation, Direction::East, 6),
                    rotation,
                    Direction::South,
                    6,
                );
                push(
                    pieces,
                    "roof_front",
                    edge,
                    rotate(rotation, Rotation::Clockwise90),
                );
            }
        }
    }
    if let Some(above_grid) = above_grid {
        create_small_walls(pieces, origin, rotation, grid, above_grid, start_x, start_y);
    }
    create_corners(pieces, origin, rotation, grid, above_grid, start_x, start_y);
}

fn create_small_walls(
    pieces: &mut Vec<MansionPieceSpec>,
    origin: BlockPos,
    rotation: Rotation,
    grid: &SimpleGrid,
    above: &SimpleGrid,
    start_x: i32,
    start_y: i32,
) {
    for y in 0..grid.height() as i32 {
        for x in 0..grid.width() as i32 {
            if !house(grid, x, y) || !house(above, x, y) {
                continue;
            }
            let position = cell_position(origin, rotation, x, y, start_x, start_y);
            if !house(grid, x + 1, y) {
                push(
                    pieces,
                    "small_wall",
                    relative_rotated(position, rotation, Direction::East, 7),
                    rotation,
                );
            }
            if !house(grid, x - 1, y) {
                let edge = relative_rotated(
                    relative_rotated(position, rotation, Direction::West, 1),
                    rotation,
                    Direction::South,
                    6,
                );
                push(
                    pieces,
                    "small_wall",
                    edge,
                    rotate(rotation, Rotation::Clockwise180),
                );
            }
            if !house(grid, x, y - 1) {
                let edge = relative_rotated(position, rotation, Direction::North, 1);
                push(
                    pieces,
                    "small_wall",
                    edge,
                    rotate(rotation, Rotation::CounterClockwise90),
                );
            }
            if !house(grid, x, y + 1) {
                let edge = relative_rotated(
                    relative_rotated(position, rotation, Direction::East, 6),
                    rotation,
                    Direction::South,
                    7,
                );
                push(
                    pieces,
                    "small_wall",
                    edge,
                    rotate(rotation, Rotation::Clockwise90),
                );
            }
            create_small_wall_corners(pieces, position, rotation, grid, x, y);
        }
    }
}

fn create_small_wall_corners(
    pieces: &mut Vec<MansionPieceSpec>,
    position: BlockPos,
    rotation: Rotation,
    grid: &SimpleGrid,
    x: i32,
    y: i32,
) {
    if !house(grid, x + 1, y) {
        if !house(grid, x, y - 1) {
            let corner = relative_rotated(
                relative_rotated(position, rotation, Direction::East, 7),
                rotation,
                Direction::North,
                2,
            );
            push(pieces, "small_wall_corner", corner, rotation);
        }
        if !house(grid, x, y + 1) {
            let corner = relative_rotated(
                relative_rotated(position, rotation, Direction::East, 8),
                rotation,
                Direction::South,
                7,
            );
            push(
                pieces,
                "small_wall_corner",
                corner,
                rotate(rotation, Rotation::Clockwise90),
            );
        }
    }
    if !house(grid, x - 1, y) {
        if !house(grid, x, y - 1) {
            let corner = relative_rotated(
                relative_rotated(position, rotation, Direction::West, 2),
                rotation,
                Direction::North,
                1,
            );
            push(
                pieces,
                "small_wall_corner",
                corner,
                rotate(rotation, Rotation::CounterClockwise90),
            );
        }
        if !house(grid, x, y + 1) {
            let corner = relative_rotated(
                relative_rotated(position, rotation, Direction::West, 1),
                rotation,
                Direction::South,
                8,
            );
            push(
                pieces,
                "small_wall_corner",
                corner,
                rotate(rotation, Rotation::Clockwise180),
            );
        }
    }
}

fn create_corners(
    pieces: &mut Vec<MansionPieceSpec>,
    origin: BlockPos,
    rotation: Rotation,
    grid: &SimpleGrid,
    above_grid: Option<&SimpleGrid>,
    start_x: i32,
    start_y: i32,
) {
    for y in 0..grid.height() as i32 {
        for x in 0..grid.width() as i32 {
            let covered = above_grid.is_some_and(|above| house(above, x, y));
            if !house(grid, x, y) || covered {
                continue;
            }
            let position = cell_position(origin, rotation, x, y, start_x, start_y);
            if !house(grid, x + 1, y) {
                let east = relative_rotated(position, rotation, Direction::East, 6);
                if !house(grid, x, y + 1) {
                    push(
                        pieces,
                        "roof_corner",
                        relative_rotated(east, rotation, Direction::South, 6),
                        rotation,
                    );
                } else if house(grid, x + 1, y + 1) {
                    push(
                        pieces,
                        "roof_inner_corner",
                        relative_rotated(east, rotation, Direction::South, 5),
                        rotation,
                    );
                }
                if !house(grid, x, y - 1) {
                    push(
                        pieces,
                        "roof_corner",
                        east,
                        rotate(rotation, Rotation::CounterClockwise90),
                    );
                } else if house(grid, x + 1, y - 1) {
                    let corner = relative_rotated(
                        relative_rotated(position, rotation, Direction::East, 9),
                        rotation,
                        Direction::North,
                        2,
                    );
                    push(
                        pieces,
                        "roof_inner_corner",
                        corner,
                        rotate(rotation, Rotation::Clockwise90),
                    );
                }
            }
            if house(grid, x - 1, y) {
                continue;
            }
            if !house(grid, x, y + 1) {
                push(
                    pieces,
                    "roof_corner",
                    relative_rotated(position, rotation, Direction::South, 6),
                    rotate(rotation, Rotation::Clockwise90),
                );
            } else if house(grid, x - 1, y + 1) {
                let corner = relative_rotated(
                    relative_rotated(position, rotation, Direction::South, 8),
                    rotation,
                    Direction::West,
                    3,
                );
                push(
                    pieces,
                    "roof_inner_corner",
                    corner,
                    rotate(rotation, Rotation::CounterClockwise90),
                );
            }
            if !house(grid, x, y - 1) {
                push(
                    pieces,
                    "roof_corner",
                    position,
                    rotate(rotation, Rotation::Clockwise180),
                );
            } else if house(grid, x - 1, y - 1) {
                push(
                    pieces,
                    "roof_inner_corner",
                    relative_rotated(position, rotation, Direction::South, 1),
                    rotate(rotation, Rotation::Clockwise180),
                );
            }
        }
    }
}

fn cell_position(
    origin: BlockPos,
    rotation: Rotation,
    x: i32,
    y: i32,
    start_x: i32,
    start_y: i32,
) -> BlockPos {
    relative_rotated(
        relative_rotated(origin, rotation, Direction::South, 8 + (y - start_y) * 8),
        rotation,
        Direction::East,
        (x - start_x) * 8,
    )
}

fn house(grid: &SimpleGrid, x: i32, y: i32) -> bool {
    MansionLayout::is_house(grid, x, y)
}
