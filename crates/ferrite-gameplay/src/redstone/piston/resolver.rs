//! Atomic piston structure resolver and pushability matrix.

use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

pub const MAX_PUSH_DEPTH: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushReaction {
    Normal,
    Block,
    Destroy,
    PushOnly,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PistonBlockKind {
    Air,
    Obsidian,
    CryingObsidian,
    RespawnAnchor,
    ReinforcedDeepslate,
    Piston { extended: bool },
    StickyPiston { extended: bool },
    Slime,
    Honey,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PistonBlock {
    pub kind: PistonBlockKind,
    pub reaction: PushReaction,
    pub destroy_speed: f32,
    pub has_block_entity: bool,
}

impl PistonBlock {
    pub const AIR: Self = Self {
        kind: PistonBlockKind::Air,
        reaction: PushReaction::Normal,
        destroy_speed: 0.0,
        has_block_entity: false,
    };

    pub const fn ordinary(reaction: PushReaction) -> Self {
        Self {
            kind: PistonBlockKind::Other,
            reaction,
            destroy_speed: 1.0,
            has_block_entity: false,
        }
    }

    pub const fn is_air(self) -> bool {
        matches!(self.kind, PistonBlockKind::Air)
    }

    pub const fn is_piston(self) -> bool {
        matches!(
            self.kind,
            PistonBlockKind::Piston { .. } | PistonBlockKind::StickyPiston { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolverWorld {
    pub min_y: i32,
    pub max_y: i32,
    blocks: BTreeMap<BlockPos, PistonBlock>,
    outside_border: BTreeSet<BlockPos>,
}

impl ResolverWorld {
    pub fn new(min_y: i32, max_y: i32) -> Self {
        Self {
            min_y,
            max_y,
            blocks: BTreeMap::new(),
            outside_border: BTreeSet::new(),
        }
    }

    pub fn insert(&mut self, position: BlockPos, block: PistonBlock) {
        self.blocks.insert(position, block);
    }

    pub fn mark_outside_border(&mut self, position: BlockPos) {
        self.outside_border.insert(position);
    }

    pub fn block(&self, position: BlockPos) -> PistonBlock {
        self.blocks
            .get(&position)
            .copied()
            .unwrap_or(PistonBlock::AIR)
    }

    pub fn within_border(&self, position: BlockPos) -> bool {
        !self.outside_border.contains(&position)
    }
}

pub fn is_pushable(
    block: PistonBlock,
    world: &ResolverWorld,
    position: BlockPos,
    movement: Direction,
    allow_destroyable: bool,
    connection_direction: Direction,
) -> bool {
    if position.y < world.min_y || position.y > world.max_y || !world.within_border(position) {
        return false;
    }
    if block.is_air() {
        return true;
    }
    if matches!(
        block.kind,
        PistonBlockKind::Obsidian
            | PistonBlockKind::CryingObsidian
            | PistonBlockKind::RespawnAnchor
            | PistonBlockKind::ReinforcedDeepslate
    ) {
        return false;
    }
    if movement == Direction::Down && position.y == world.min_y
        || movement == Direction::Up && position.y == world.max_y
    {
        return false;
    }
    match block.kind {
        PistonBlockKind::Piston { extended: true }
        | PistonBlockKind::StickyPiston { extended: true } => return false,
        PistonBlockKind::Piston { extended: false }
        | PistonBlockKind::StickyPiston { extended: false } => {}
        _ => {
            if block.destroy_speed == -1.0 {
                return false;
            }
            match block.reaction {
                PushReaction::Block => return false,
                PushReaction::Destroy => return allow_destroyable,
                PushReaction::PushOnly => return movement == connection_direction,
                PushReaction::Normal | PushReaction::Ignore => {}
            }
        }
    }
    !block.has_block_entity
}

pub const fn is_sticky(block: PistonBlock) -> bool {
    matches!(block.kind, PistonBlockKind::Slime | PistonBlockKind::Honey)
}

pub const fn can_stick(first: PistonBlock, second: PistonBlock) -> bool {
    if matches!(
        (first.kind, second.kind),
        (PistonBlockKind::Honey, PistonBlockKind::Slime)
            | (PistonBlockKind::Slime, PistonBlockKind::Honey)
    ) {
        false
    } else {
        is_sticky(first) || is_sticky(second)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStructure {
    pub push_direction: Direction,
    pub to_push: Vec<BlockPos>,
    pub to_destroy: Vec<BlockPos>,
}

pub fn resolve_structure(
    world: &ResolverWorld,
    piston_position: BlockPos,
    piston_direction: Direction,
    extending: bool,
) -> Option<ResolvedStructure> {
    let mut resolver = StructureResolver::new(world, piston_position, piston_direction, extending)?;
    if resolver.resolve() {
        Some(ResolvedStructure {
            push_direction: resolver.push_direction,
            to_push: resolver.to_push,
            to_destroy: resolver.to_destroy,
        })
    } else {
        None
    }
}

struct StructureResolver<'a> {
    world: &'a ResolverWorld,
    piston_position: BlockPos,
    piston_direction: Direction,
    extending: bool,
    start_position: BlockPos,
    push_direction: Direction,
    to_push: Vec<BlockPos>,
    to_destroy: Vec<BlockPos>,
}

impl<'a> StructureResolver<'a> {
    fn new(
        world: &'a ResolverWorld,
        piston_position: BlockPos,
        piston_direction: Direction,
        extending: bool,
    ) -> Option<Self> {
        let push_direction = if extending {
            piston_direction
        } else {
            piston_direction.opposite()
        };
        let distance = if extending { 1 } else { 2 };
        Some(Self {
            world,
            piston_position,
            piston_direction,
            extending,
            start_position: piston_position
                .checked_offset(piston_direction, distance)
                .ok()?,
            push_direction,
            to_push: Vec::new(),
            to_destroy: Vec::new(),
        })
    }

    fn resolve(&mut self) -> bool {
        self.to_push.clear();
        self.to_destroy.clear();
        let initial = self.world.block(self.start_position);
        if !is_pushable(
            initial,
            self.world,
            self.start_position,
            self.push_direction,
            false,
            self.piston_direction,
        ) {
            if self.extending && matches!(initial.reaction, PushReaction::Destroy) {
                self.to_destroy.push(self.start_position);
                return true;
            }
            return false;
        }
        if !self.add_block_line(self.start_position, self.push_direction) {
            return false;
        }
        let mut index = 0;
        while index < self.to_push.len() {
            let position = self.to_push[index];
            if is_sticky(self.world.block(position)) && !self.add_branches(position) {
                return false;
            }
            index += 1;
        }
        true
    }

    fn add_block_line(&mut self, start: BlockPos, connection_direction: Direction) -> bool {
        let mut next = self.world.block(start);
        if next.is_air()
            || !is_pushable(
                next,
                self.world,
                start,
                self.push_direction,
                false,
                connection_direction,
            )
            || start == self.piston_position
            || self.to_push.contains(&start)
        {
            return true;
        }
        let mut block_count = 1_usize;
        if block_count + self.to_push.len() > MAX_PUSH_DEPTH {
            return false;
        }
        while is_sticky(next) {
            let Some(position) = start
                .checked_offset(self.push_direction.opposite(), block_count as i32)
                .ok()
            else {
                return false;
            };
            let previous = next;
            next = self.world.block(position);
            if next.is_air()
                || !can_stick(previous, next)
                || !is_pushable(
                    next,
                    self.world,
                    position,
                    self.push_direction,
                    false,
                    self.push_direction.opposite(),
                )
                || position == self.piston_position
            {
                break;
            }
            block_count += 1;
            if block_count + self.to_push.len() > MAX_PUSH_DEPTH {
                return false;
            }
        }
        let mut blocks_added = 0;
        for distance in (0..block_count).rev() {
            let Some(position) = start
                .checked_offset(self.push_direction.opposite(), distance as i32)
                .ok()
            else {
                return false;
            };
            self.to_push.push(position);
            blocks_added += 1;
        }
        let mut distance = 1_i32;
        loop {
            let Some(position) = start.checked_offset(self.push_direction, distance).ok() else {
                return false;
            };
            if let Some(collision) = self.to_push.iter().position(|known| *known == position) {
                self.reorder_at_collision(blocks_added, collision);
                for index in 0..=collision + blocks_added {
                    let branch_position = self.to_push[index];
                    if is_sticky(self.world.block(branch_position))
                        && !self.add_branches(branch_position)
                    {
                        return false;
                    }
                }
                return true;
            }
            next = self.world.block(position);
            if next.is_air() {
                return true;
            }
            if !is_pushable(
                next,
                self.world,
                position,
                self.push_direction,
                true,
                self.push_direction,
            ) || position == self.piston_position
            {
                return false;
            }
            if matches!(next.reaction, PushReaction::Destroy) {
                self.to_destroy.push(position);
                return true;
            }
            if self.to_push.len() >= MAX_PUSH_DEPTH {
                return false;
            }
            self.to_push.push(position);
            blocks_added += 1;
            distance += 1;
        }
    }

    fn reorder_at_collision(&mut self, blocks_added: usize, collision: usize) {
        let last_line_start = self.to_push.len() - blocks_added;
        let mut reordered = Vec::with_capacity(self.to_push.len());
        reordered.extend_from_slice(&self.to_push[..collision]);
        reordered.extend_from_slice(&self.to_push[last_line_start..]);
        reordered.extend_from_slice(&self.to_push[collision..last_line_start]);
        self.to_push = reordered;
    }

    fn add_branches(&mut self, from_position: BlockPos) -> bool {
        let from = self.world.block(from_position);
        for direction in Direction::ALL {
            if direction.axis() == self.push_direction.axis() {
                continue;
            }
            let Some(neighbor_position) = from_position.checked_offset(direction, 1).ok() else {
                return false;
            };
            let neighbor = self.world.block(neighbor_position);
            if can_stick(neighbor, from) && !self.add_block_line(neighbor_position, direction) {
                return false;
            }
        }
        true
    }
}
