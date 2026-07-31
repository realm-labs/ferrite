//! Shared oriented structure-piece placement primitives.

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::generation::structure::BlockBox;
use crate::generation::structure::processor::StructureState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalDirection {
    North,
    East,
    South,
    West,
}

impl HorizontalDirection {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrientedPiece {
    pub bounds: BlockBox,
    pub orientation: HorizontalDirection,
}

impl OrientedPiece {
    pub fn from_anchor(
        anchor: BlockPos,
        local_offset: BlockPos,
        size: [i32; 3],
        orientation: HorizontalDirection,
    ) -> Self {
        let [width, height, depth] = size;
        let minimum_y = anchor.y + local_offset.y;
        let (minimum_x, minimum_z, maximum_x, maximum_z) = match orientation {
            HorizontalDirection::North => (
                anchor.x + local_offset.x,
                anchor.z - depth + 1 - local_offset.z,
                anchor.x + local_offset.x + width - 1,
                anchor.z - local_offset.z,
            ),
            HorizontalDirection::South => (
                anchor.x + local_offset.x,
                anchor.z + local_offset.z,
                anchor.x + local_offset.x + width - 1,
                anchor.z + local_offset.z + depth - 1,
            ),
            HorizontalDirection::West => (
                anchor.x - depth + 1 - local_offset.z,
                anchor.z + local_offset.x,
                anchor.x - local_offset.z,
                anchor.z + local_offset.x + width - 1,
            ),
            HorizontalDirection::East => (
                anchor.x + local_offset.z,
                anchor.z + local_offset.x,
                anchor.x + local_offset.z + depth - 1,
                anchor.z + local_offset.x + width - 1,
            ),
        };
        Self {
            bounds: BlockBox::new(
                BlockPos {
                    x: minimum_x,
                    y: minimum_y,
                    z: minimum_z,
                },
                BlockPos {
                    x: maximum_x,
                    y: minimum_y + height - 1,
                    z: maximum_z,
                },
            )
            .expect("oriented piece dimensions must be positive"),
            orientation,
        }
    }

    pub fn world_position(self, local: BlockPos) -> BlockPos {
        let (x, z) = match self.orientation {
            HorizontalDirection::North => (
                self.bounds.minimum.x + local.x,
                self.bounds.maximum.z - local.z,
            ),
            HorizontalDirection::South => (
                self.bounds.minimum.x + local.x,
                self.bounds.minimum.z + local.z,
            ),
            HorizontalDirection::West => (
                self.bounds.maximum.x - local.z,
                self.bounds.minimum.z + local.x,
            ),
            HorizontalDirection::East => (
                self.bounds.minimum.x + local.z,
                self.bounds.minimum.z + local.x,
            ),
        };
        BlockPos {
            x,
            y: self.bounds.minimum.y + local.y,
            z,
        }
    }

    pub fn transform_state(self, mut state: StructureState) -> StructureState {
        if let Some(facing) = state.properties.get_mut("facing") {
            *facing = rotate_horizontal(facing, self.orientation).into();
        }
        rotate_directional_properties(&mut state, self.orientation);
        state
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidState {
    Empty,
    Water,
    Lava,
    Other,
}

impl FluidState {
    pub const fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }
}

pub trait PieceWorld {
    fn state_at(&mut self, position: BlockPos) -> StructureState;

    fn fluid_at(&mut self, position: BlockPos) -> FluidState;

    fn set_state(&mut self, position: BlockPos, state: StructureState, flags: u32) -> bool;

    fn schedule_fluid_tick(&mut self, position: BlockPos, fluid: FluidState, delay: u32);

    fn mark_shape_postprocessing(&mut self, position: BlockPos);

    fn solid_render(&mut self, position: BlockPos) -> bool;

    fn is_loot_container(&mut self, position: BlockPos) -> bool;

    fn install_loot(&mut self, position: BlockPos, table: &str, seed: i64);
}

#[derive(Debug, Clone, Copy)]
pub struct PiecePlacement<'a> {
    pub piece: OrientedPiece,
    pub clip: &'a BlockBox,
}

impl PiecePlacement<'_> {
    pub fn place_block(
        self,
        world: &mut impl PieceWorld,
        local: BlockPos,
        state: StructureState,
    ) -> bool {
        let position = self.piece.world_position(local);
        if !self.clip.contains(position) {
            return false;
        }
        let state = self.piece.transform_state(state);
        let shape_sensitive = is_shape_sensitive(&state.block);
        let written = world.set_state(position, state, 2);
        let fluid = world.fluid_at(position);
        if !fluid.is_empty() {
            world.schedule_fluid_tick(position, fluid, 0);
        }
        if shape_sensitive {
            world.mark_shape_postprocessing(position);
        }
        written
    }

    pub fn fill_box<F>(
        self,
        world: &mut impl PieceWorld,
        minimum: BlockPos,
        maximum: BlockPos,
        skip_live_air: bool,
        mut state: F,
    ) where
        F: FnMut(BlockPos, bool) -> StructureState,
    {
        for y in minimum.y..=maximum.y {
            for x in minimum.x..=maximum.x {
                for z in minimum.z..=maximum.z {
                    let local = BlockPos { x, y, z };
                    let position = self.piece.world_position(local);
                    if skip_live_air && world.state_at(position).block == "minecraft:air" {
                        continue;
                    }
                    let edge = x == minimum.x
                        || x == maximum.x
                        || y == minimum.y
                        || y == maximum.y
                        || z == minimum.z
                        || z == maximum.z;
                    self.place_block(world, local, state(local, edge));
                }
            }
        }
    }

    pub fn fill_column_down<F>(
        self,
        world: &mut impl PieceWorld,
        start: BlockPos,
        state: StructureState,
        minimum_y: i32,
        mut replaceable: F,
    ) where
        F: FnMut(&StructureState, FluidState) -> bool,
    {
        let mut position = self.piece.world_position(start);
        if !self.clip.contains(position) {
            return;
        }
        while position.y > minimum_y + 1 {
            let current = world.state_at(position);
            let fluid = world.fluid_at(position);
            if !replaceable(&current, fluid) {
                break;
            }
            world.set_state(position, state.clone(), 2);
            position.y -= 1;
        }
    }

    pub fn create_chest<F>(
        self,
        world: &mut impl PieceWorld,
        local: BlockPos,
        table: &str,
        seed: F,
    ) -> bool
    where
        F: FnOnce() -> i64,
    {
        let position = self.piece.world_position(local);
        if !self.clip.contains(position) || world.state_at(position).block == "minecraft:chest" {
            return false;
        }
        let facing = reorient_chest(world, position);
        let mut chest = StructureState::new("minecraft:chest");
        chest.properties.insert("facing".into(), facing.into());
        world.set_state(position, chest, 2);
        if world.is_loot_container(position) {
            world.install_loot(position, table, seed());
        }
        true
    }
}

fn reorient_chest(world: &mut impl PieceWorld, position: BlockPos) -> &'static str {
    let mut solid = Vec::new();
    for direction in Direction::HORIZONTAL {
        let [x, y, z] = direction.step();
        let neighbor = BlockPos {
            x: position.x + x,
            y: position.y + y,
            z: position.z + z,
        };
        if world.state_at(neighbor).block == "minecraft:chest" {
            return "north";
        }
        if world.solid_render(neighbor) {
            solid.push(direction);
        }
    }
    if solid.len() == 1 {
        return direction_name(solid[0].opposite());
    }
    for candidate in [
        Direction::North,
        Direction::South,
        Direction::West,
        Direction::East,
    ] {
        if !solid.contains(&candidate) {
            return direction_name(candidate);
        }
    }
    "north"
}

fn rotate_horizontal(value: &str, orientation: HorizontalDirection) -> &str {
    let turns = match orientation {
        HorizontalDirection::South => 0,
        HorizontalDirection::West => 1,
        HorizontalDirection::North => 2,
        HorizontalDirection::East => 3,
    };
    let index = match value {
        "north" => 0,
        "east" => 1,
        "south" => 2,
        "west" => 3,
        _ => return value,
    };
    ["north", "east", "south", "west"][(index + turns) % 4]
}

fn rotate_directional_properties(state: &mut StructureState, orientation: HorizontalDirection) {
    let old = ["north", "east", "south", "west"]
        .map(|name| state.properties.remove(name).map(|value| (name, value)));
    for entry in old.into_iter().flatten() {
        let name = rotate_horizontal(entry.0, orientation).to_owned();
        state.properties.insert(name, entry.1);
    }
}

fn direction_name(direction: Direction) -> &'static str {
    match direction {
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
        Direction::Down => "down",
        Direction::Up => "up",
    }
}

fn is_shape_sensitive(block: &str) -> bool {
    block.ends_with("_fence")
        || block.ends_with("_torch")
        || block.ends_with("_wall_torch")
        || block.ends_with("_ladder")
        || block == "minecraft:iron_bars"
}
