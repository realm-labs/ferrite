//! Weighted connector graph expansion for jigsaw structures.

use std::collections::{BTreeMap, VecDeque};
use std::num::NonZeroU32;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use thiserror::Error;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::{BlockBox, offset_position};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Projection {
    Rigid,
    TerrainMatching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Joint {
    Rollable,
    Aligned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl Rotation {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::Clockwise90,
        Self::Clockwise180,
        Self::CounterClockwise90,
    ];

    pub fn rotate_direction(self, direction: Direction) -> Direction {
        match (self, direction) {
            (_, Direction::Up | Direction::Down) | (Self::None, _) => direction,
            (Self::Clockwise90, Direction::North) => Direction::East,
            (Self::Clockwise90, Direction::East) => Direction::South,
            (Self::Clockwise90, Direction::South) => Direction::West,
            (Self::Clockwise90, Direction::West) => Direction::North,
            (Self::Clockwise180, Direction::North) => Direction::South,
            (Self::Clockwise180, Direction::South) => Direction::North,
            (Self::Clockwise180, Direction::East) => Direction::West,
            (Self::Clockwise180, Direction::West) => Direction::East,
            (Self::CounterClockwise90, Direction::North) => Direction::West,
            (Self::CounterClockwise90, Direction::West) => Direction::South,
            (Self::CounterClockwise90, Direction::South) => Direction::East,
            (Self::CounterClockwise90, Direction::East) => Direction::North,
        }
    }

    pub fn rotate_local(self, position: BlockPos, size: [i32; 3]) -> BlockPos {
        match self {
            Self::None => position,
            Self::Clockwise90 => BlockPos::new(size[2] - 1 - position.z, position.y, position.x),
            Self::Clockwise180 => BlockPos::new(
                size[0] - 1 - position.x,
                position.y,
                size[2] - 1 - position.z,
            ),
            Self::CounterClockwise90 => {
                BlockPos::new(position.z, position.y, size[0] - 1 - position.x)
            }
        }
    }

    pub fn rotated_size(self, size: [i32; 3]) -> [i32; 3] {
        if matches!(self, Self::Clockwise90 | Self::CounterClockwise90) {
            [size[2], size[1], size[0]]
        } else {
            size
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connector {
    pub local_position: BlockPos,
    pub front: Direction,
    pub top: Direction,
    pub joint: Joint,
    pub name: String,
    pub target: String,
    pub pool: String,
    pub selection_priority: i32,
    pub placement_priority: i32,
}

impl Connector {
    pub fn rotated(&self, rotation: Rotation, size: [i32; 3]) -> Self {
        let mut connector = self.clone();
        connector.local_position = rotation.rotate_local(self.local_position, size);
        connector.front = rotation.rotate_direction(self.front);
        connector.top = rotation.rotate_direction(self.top);
        connector
    }

    pub fn world_position(&self, piece_origin: BlockPos) -> BlockPos {
        offset_position(
            piece_origin,
            [
                self.local_position.x,
                self.local_position.y,
                self.local_position.z,
            ],
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Empty,
    Feature { name: String },
    Single { template: String, legacy: bool },
    List(Vec<PoolElement>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolElement {
    pub kind: ElementKind,
    pub projection: Projection,
    pub size: [i32; 3],
    pub connectors: Vec<Connector>,
    pub ground_level_delta: i32,
    pub processor_list: Option<String>,
}

impl PoolElement {
    pub fn empty() -> Self {
        Self {
            kind: ElementKind::Empty,
            projection: Projection::TerrainMatching,
            size: [0; 3],
            connectors: Vec::new(),
            ground_level_delta: 0,
            processor_list: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self.kind, ElementKind::Empty)
    }

    pub fn box_at(&self, origin: BlockPos, rotation: Rotation) -> Option<BlockBox> {
        if self.is_empty() {
            return None;
        }
        if matches!(self.kind, ElementKind::Feature { .. }) {
            return Some(BlockBox::point(origin));
        }
        let size = rotation.rotated_size(self.size);
        let maximum = BlockPos::new(
            origin.x.wrapping_add(size[0].wrapping_sub(1)),
            origin.y.wrapping_add(size[1].wrapping_sub(1)),
            origin.z.wrapping_add(size[2].wrapping_sub(1)),
        );
        BlockBox::new(origin, maximum)
    }

    pub fn ordered_connectors(
        &self,
        rotation: Rotation,
        random: &mut impl GenerationRandom,
    ) -> Vec<Connector> {
        let mut connectors = self
            .connectors
            .iter()
            .map(|connector| connector.rotated(rotation, self.size))
            .collect::<Vec<_>>();
        shuffle(&mut connectors, random);
        connectors.sort_by_key(|connector| std::cmp::Reverse(connector.selection_priority));
        connectors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePool {
    pub fallback: String,
    expanded: Vec<PoolElement>,
}

impl TemplatePool {
    pub fn new(
        fallback: impl Into<String>,
        entries: Vec<(PoolElement, u16)>,
    ) -> Result<Self, JigsawError> {
        let mut expanded = Vec::new();
        for (element, weight) in entries {
            if !(1..=150).contains(&weight) {
                return Err(JigsawError::Weight(weight));
            }
            expanded.extend(std::iter::repeat_n(element, usize::from(weight)));
        }
        Ok(Self {
            fallback: fallback.into(),
            expanded,
        })
    }

    pub fn expanded(&self) -> &[PoolElement] {
        &self.expanded
    }

    pub fn random_element(&self, random: &mut impl GenerationRandom) -> PoolElement {
        if self.expanded.is_empty() {
            PoolElement::empty()
        } else {
            let bound = NonZeroU32::new(self.expanded.len() as u32).expect("pool is nonempty");
            self.expanded[random.next_u32(bound) as usize].clone()
        }
    }

    pub fn shuffled(&self, random: &mut impl GenerationRandom) -> Vec<PoolElement> {
        let mut elements = self.expanded.clone();
        shuffle(&mut elements, random);
        elements
    }

    pub fn maximum_y_span(&self) -> i32 {
        self.expanded
            .iter()
            .filter(|element| !element.is_empty())
            .map(|element| element.size[1])
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasBinding {
    Direct {
        alias: String,
        target: String,
    },
    Random {
        alias: String,
        targets: Vec<(String, u16)>,
    },
    RandomGroup(Vec<(Vec<AliasBinding>, u16)>),
}

pub fn resolve_aliases(
    bindings: &[AliasBinding],
    random: &mut impl GenerationRandom,
) -> Result<BTreeMap<String, String>, JigsawError> {
    let mut aliases = BTreeMap::new();
    for binding in bindings {
        apply_binding(binding, random, &mut aliases)?;
    }
    Ok(aliases)
}

fn apply_binding(
    binding: &AliasBinding,
    random: &mut impl GenerationRandom,
    aliases: &mut BTreeMap<String, String>,
) -> Result<(), JigsawError> {
    match binding {
        AliasBinding::Direct { alias, target } => insert_alias(aliases, alias, target),
        AliasBinding::Random { alias, targets } => {
            let target = weighted_choice(targets, random)?;
            insert_alias(aliases, alias, target)
        }
        AliasBinding::RandomGroup(groups) => {
            let group = weighted_choice(groups, random)?;
            for nested in group {
                apply_binding(nested, random, aliases)?;
            }
            Ok(())
        }
    }
}

fn insert_alias(
    aliases: &mut BTreeMap<String, String>,
    alias: &str,
    target: &str,
) -> Result<(), JigsawError> {
    if aliases
        .insert(alias.to_owned(), target.to_owned())
        .is_some()
    {
        Err(JigsawError::DuplicateAlias(alias.to_owned()))
    } else {
        Ok(())
    }
}

fn weighted_choice<'a, T>(
    values: &'a [(T, u16)],
    random: &mut impl GenerationRandom,
) -> Result<&'a T, JigsawError> {
    let total = values.iter().try_fold(0_u32, |total, (_, weight)| {
        total.checked_add(u32::from(*weight))
    });
    let total = total
        .filter(|total| *total > 0)
        .and_then(NonZeroU32::new)
        .ok_or(JigsawError::EmptyWeightedChoice)?;
    let mut selected = random.next_u32(total);
    for (value, weight) in values {
        if selected < u32::from(*weight) {
            return Ok(value);
        }
        selected -= u32::from(*weight);
    }
    unreachable!("bounded weighted choice must select one value")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    pub bottom: u32,
    pub top: u32,
    shared_zero: bool,
}

impl Padding {
    pub const ZERO: Self = Self {
        bottom: 0,
        top: 0,
        shared_zero: true,
    };

    pub const fn new(bottom: u32, top: u32) -> Self {
        Self {
            bottom,
            top,
            shared_zero: false,
        }
    }

    pub const fn is_shared_zero(self) -> bool {
        self.shared_zero
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Junction {
    pub source_x: i32,
    pub source_ground_y: i32,
    pub source_z: i32,
    pub delta_y: i32,
    pub destination_projection: Projection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawPiece {
    pub element: PoolElement,
    pub position: BlockPos,
    pub rotation: Rotation,
    pub bounding_box: BlockBox,
    pub ground_level_delta: i32,
    pub junctions: Vec<Junction>,
    pub depth: u8,
}

impl JigsawPiece {
    pub fn move_by(&mut self, offset: [i32; 3]) {
        self.position = offset_position(self.position, offset);
        self.bounding_box = self.bounding_box.moved(offset);
    }
}

pub fn can_attach(source: &Connector, target: &Connector) -> bool {
    source.front.opposite() == target.front
        && source.target == target.name
        && (source.joint == Joint::Rollable || source.top == target.top)
}

#[derive(Debug, Clone)]
pub struct PriorityQueue<T> {
    queues: BTreeMap<i32, VecDeque<T>>,
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self {
            queues: BTreeMap::new(),
        }
    }
}

impl<T> PriorityQueue<T> {
    pub fn push(&mut self, priority: i32, value: T) {
        self.queues.entry(priority).or_default().push_back(value);
    }

    pub fn pop(&mut self) -> Option<T> {
        loop {
            let priority = *self.queues.last_key_value()?.0;
            let queue = self.queues.get_mut(&priority).expect("key was selected");
            if let Some(value) = queue.pop_front() {
                if queue.is_empty() {
                    self.queues.remove(&priority);
                }
                return Some(value);
            }
            self.queues.remove(&priority);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queues.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeSpace {
    allowed: BlockBox,
    occupied: Vec<BlockBox>,
}

impl FreeSpace {
    pub fn new(allowed: BlockBox) -> Self {
        Self {
            allowed,
            occupied: Vec::new(),
        }
    }

    pub fn subtract(&mut self, occupied: BlockBox) {
        self.occupied.push(occupied);
    }

    pub fn admits_deflated_quarter(&self, candidate: BlockBox) -> bool {
        self.allowed.contains_box(candidate)
            && !self
                .occupied
                .iter()
                .any(|occupied| deflated_intersects(*occupied, candidate))
    }
}

fn deflated_intersects(left: BlockBox, right: BlockBox) -> bool {
    let separated = |left_min: i32, left_max: i32, right_min: i32, right_max: i32| {
        f64::from(left_max) + 0.75 < f64::from(right_min) + 0.25
            || f64::from(right_max) + 0.75 < f64::from(left_min) + 0.25
    };
    !separated(
        left.minimum.x,
        left.maximum.x,
        right.minimum.x,
        right.maximum.x,
    ) && !separated(
        left.minimum.y,
        left.maximum.y,
        right.minimum.y,
        right.maximum.y,
    ) && !separated(
        left.minimum.z,
        left.maximum.z,
        right.minimum.z,
        right.maximum.z,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExpansionConfig {
    pub allowed: BlockBox,
    pub maximum_depth: u8,
    pub use_expansion_hack: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JigsawStartConfig {
    pub dimension_min_y: i32,
    pub dimension_max_y: i32,
    pub padding: Padding,
    pub maximum_depth: u8,
    pub horizontal_distance: i32,
    pub vertical_distance: i32,
    pub use_expansion_hack: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawStart {
    pub stub_position: BlockPos,
    pub pieces: Vec<JigsawPiece>,
}

#[derive(Debug, Clone, Copy)]
pub struct JigsawStartRequest<'a> {
    pub position: BlockPos,
    pub pool: &'a str,
    pub connector_name: Option<&'a str>,
    pub aliases: &'a BTreeMap<String, String>,
    pub config: JigsawStartConfig,
}

pub fn generate_jigsaw_start(
    request: JigsawStartRequest<'_>,
    pools: &BTreeMap<String, TemplatePool>,
    random: &mut impl GenerationRandom,
    mut project_start: impl FnMut(i32, i32) -> Option<i32>,
    surface_height: impl FnMut(i32, i32) -> i32,
) -> Option<JigsawStart> {
    let JigsawStartRequest {
        position: start_position,
        pool: start_pool,
        connector_name: start_connector_name,
        aliases,
        config,
    } = request;
    if config.horizontal_distance < 0
        || config.vertical_distance < 0
        || config.dimension_min_y > config.dimension_max_y
    {
        return None;
    }
    let rotation_bound = NonZeroU32::new(Rotation::ALL.len() as u32).expect("four rotations");
    let rotation = Rotation::ALL[random.next_u32(rotation_bound) as usize];
    let pool_name = aliases.get(start_pool).map_or(start_pool, String::as_str);
    let pool = pools.get(pool_name)?;
    let element = pool.random_element(random);
    if element.is_empty() {
        return None;
    }

    let selected_connector = start_connector_name.and_then(|name| {
        element
            .ordered_connectors(rotation, random)
            .into_iter()
            .find(|connector| connector.name == name)
    });
    if start_connector_name.is_some() && selected_connector.is_none() {
        return None;
    }
    let connector_offset = selected_connector
        .as_ref()
        .map_or(BlockPos::new(0, 0, 0), |connector| connector.local_position);
    let origin = BlockPos::new(
        start_position.x.wrapping_sub(connector_offset.x),
        start_position.y.wrapping_sub(connector_offset.y),
        start_position.z.wrapping_sub(connector_offset.z),
    );
    let bounding_box = element.box_at(origin, rotation)?;
    let center = bounding_box.center();
    let target_ground_y = project_start(center.x, center.z)
        .map_or(origin.y, |height| start_position.y.wrapping_add(height));
    let vertical_move = target_ground_y.wrapping_sub(
        bounding_box
            .minimum
            .y
            .wrapping_add(element.ground_level_delta),
    );
    let moved_box = bounding_box.moved([0, vertical_move, 0]);
    if !config.padding.is_shared_zero()
        && !padding_admits(
            moved_box,
            config.dimension_min_y,
            config.dimension_max_y,
            config.padding,
        )
    {
        return None;
    }
    let moved_origin = BlockPos::new(origin.x, origin.y.wrapping_add(vertical_move), origin.z);
    let ground_level_delta = element.ground_level_delta;
    let stub_position = BlockPos::new(
        moved_box.center().x,
        target_ground_y.wrapping_add(connector_offset.y),
        moved_box.center().z,
    );
    let center_piece = JigsawPiece {
        element,
        position: moved_origin,
        rotation,
        bounding_box: moved_box,
        ground_level_delta,
        junctions: Vec::new(),
        depth: 0,
    };
    if config.maximum_depth == 0 {
        return Some(JigsawStart {
            stub_position,
            pieces: Vec::new(),
        });
    }
    let allowed = expansion_bounds(stub_position, config)?;
    let pieces = expand_pieces(
        center_piece,
        pools,
        aliases,
        ExpansionConfig {
            allowed,
            maximum_depth: config.maximum_depth,
            use_expansion_hack: config.use_expansion_hack,
        },
        random,
        surface_height,
    );
    Some(JigsawStart {
        stub_position,
        pieces,
    })
}

fn padding_admits(box_: BlockBox, minimum_y: i32, maximum_y: i32, padding: Padding) -> bool {
    i64::from(box_.minimum.y) >= i64::from(minimum_y) + i64::from(padding.bottom)
        && i64::from(box_.maximum.y) <= i64::from(maximum_y) - i64::from(padding.top)
}

fn expansion_bounds(stub: BlockPos, config: JigsawStartConfig) -> Option<BlockBox> {
    let horizontal = config.horizontal_distance;
    let vertical = config.vertical_distance;
    let bottom = i32::try_from(config.padding.bottom).ok()?;
    let top = i32::try_from(config.padding.top).ok()?;
    let minimum_y = stub
        .y
        .wrapping_sub(vertical)
        .max(config.dimension_min_y.wrapping_add(bottom));
    let maximum_y = stub
        .y
        .wrapping_add(vertical)
        .min(config.dimension_max_y.wrapping_sub(top));
    BlockBox::new(
        BlockPos::new(
            stub.x.wrapping_sub(horizontal),
            minimum_y,
            stub.z.wrapping_sub(horizontal),
        ),
        BlockPos::new(
            stub.x.wrapping_add(horizontal),
            maximum_y,
            stub.z.wrapping_add(horizontal),
        ),
    )
}

pub fn expand_pieces(
    center: JigsawPiece,
    pools: &BTreeMap<String, TemplatePool>,
    aliases: &BTreeMap<String, String>,
    config: ExpansionConfig,
    random: &mut impl GenerationRandom,
    mut surface_height: impl FnMut(i32, i32) -> i32,
) -> Vec<JigsawPiece> {
    let mut pieces = vec![center];
    if config.maximum_depth == 0 {
        return Vec::new();
    }
    let mut external = FreeSpace::new(config.allowed);
    external.subtract(pieces[0].bounding_box);
    let mut pending = PriorityQueue::default();
    pending.push(0, 0_usize);
    let rules = ExpansionRules {
        pools,
        aliases,
        maximum_depth: config.maximum_depth,
        use_expansion_hack: config.use_expansion_hack,
    };
    let mut runtime = ExpansionRuntime {
        random,
        surface_height: &mut surface_height,
        pending: &mut pending,
    };
    while let Some(source_index) = runtime.pending.pop() {
        expand_source(
            source_index,
            &mut pieces,
            &mut external,
            rules,
            &mut runtime,
        );
    }
    pieces
}

#[derive(Clone, Copy)]
struct ExpansionRules<'a> {
    pools: &'a BTreeMap<String, TemplatePool>,
    aliases: &'a BTreeMap<String, String>,
    maximum_depth: u8,
    use_expansion_hack: bool,
}

struct ExpansionRuntime<'a, R, S> {
    random: &'a mut R,
    surface_height: &'a mut S,
    pending: &'a mut PriorityQueue<usize>,
}

fn expand_source<R, S>(
    source_index: usize,
    pieces: &mut Vec<JigsawPiece>,
    external: &mut FreeSpace,
    rules: ExpansionRules<'_>,
    runtime: &mut ExpansionRuntime<'_, R, S>,
) where
    R: GenerationRandom,
    S: FnMut(i32, i32) -> i32,
{
    let source_snapshot = pieces[source_index].clone();
    let mut internal = FreeSpace::new(source_snapshot.bounding_box);
    for source_connector in source_snapshot
        .element
        .ordered_connectors(source_snapshot.rotation, runtime.random)
    {
        let source_position = source_connector.world_position(source_snapshot.position);
        let step = source_connector.front.step();
        let outward = offset_position(source_position, step);
        let uses_internal = source_snapshot.bounding_box.contains(outward);
        let free = if uses_internal {
            &mut internal
        } else {
            &mut *external
        };
        let Some(candidate) = attach_one(
            &source_snapshot,
            &source_connector,
            source_position,
            outward,
            free,
            rules,
            runtime,
        ) else {
            continue;
        };
        pieces[source_index]
            .junctions
            .push(candidate.source_junction);
        let target_index = pieces.len();
        pieces.push(candidate.piece);
        if pieces[target_index].depth <= rules.maximum_depth {
            runtime
                .pending
                .push(source_connector.placement_priority, target_index);
        }
    }
}

struct Attachment {
    piece: JigsawPiece,
    source_junction: Junction,
}

fn attach_one<R, S>(
    source: &JigsawPiece,
    source_connector: &Connector,
    source_position: BlockPos,
    outward: BlockPos,
    free: &mut FreeSpace,
    rules: ExpansionRules<'_>,
    runtime: &mut ExpansionRuntime<'_, R, S>,
) -> Option<Attachment>
where
    R: GenerationRandom,
    S: FnMut(i32, i32) -> i32,
{
    let pool_name = rules
        .aliases
        .get(&source_connector.pool)
        .map_or(source_connector.pool.as_str(), String::as_str);
    let primary = rules.pools.get(pool_name)?;
    if primary.expanded.is_empty() && pool_name != "minecraft:empty" && pool_name != "empty" {
        return None;
    }
    let fallback_name = rules
        .aliases
        .get(&primary.fallback)
        .map_or(primary.fallback.as_str(), String::as_str);
    let fallback = rules.pools.get(fallback_name)?;
    if fallback.expanded.is_empty()
        && fallback_name != "minecraft:empty"
        && fallback_name != "empty"
    {
        return None;
    }
    let mut candidates = if source.depth != rules.maximum_depth {
        primary.shuffled(runtime.random)
    } else {
        Vec::new()
    };
    candidates.extend(fallback.shuffled(runtime.random));
    for element in candidates {
        if element.is_empty() {
            return None;
        }
        let mut rotations = Rotation::ALL;
        shuffle(&mut rotations, runtime.random);
        for rotation in rotations {
            for target_connector in element.ordered_connectors(rotation, runtime.random) {
                if !can_attach(source_connector, &target_connector) {
                    continue;
                }
                let mut origin = BlockPos::new(
                    outward.x.wrapping_sub(target_connector.local_position.x),
                    outward.y.wrapping_sub(target_connector.local_position.y),
                    outward.z.wrapping_sub(target_connector.local_position.z),
                );
                let delta_y = source_connector
                    .local_position
                    .y
                    .wrapping_sub(target_connector.local_position.y)
                    .wrapping_add(source_connector.front.step()[1]);
                let both_rigid = source.element.projection == Projection::Rigid
                    && element.projection == Projection::Rigid;
                let cached_surface = if both_rigid {
                    None
                } else {
                    Some((runtime.surface_height)(
                        source_position.x,
                        source_position.z,
                    ))
                };
                let target_minimum_y = if both_rigid {
                    source.bounding_box.minimum.y.wrapping_add(delta_y)
                } else {
                    cached_surface
                        .expect("nonrigid attachment sampled a height")
                        .wrapping_sub(target_connector.local_position.y)
                };
                origin.y = target_minimum_y;
                let original_box = element.box_at(origin, rotation)?;
                let collision_box = if rules.use_expansion_hack && original_box.size()[1] <= 16 {
                    expand_collision_box(
                        original_box,
                        &element,
                        rotation,
                        rules.pools,
                        rules.aliases,
                    )
                } else {
                    original_box
                };
                if !free.admits_deflated_quarter(collision_box) {
                    continue;
                }
                free.subtract(collision_box);
                let ground_level_delta = if element.projection == Projection::Rigid {
                    source.ground_level_delta.wrapping_sub(delta_y)
                } else {
                    element.ground_level_delta
                };
                let junction_y = match (source.element.projection, element.projection) {
                    (Projection::Rigid, _) => source
                        .bounding_box
                        .minimum
                        .y
                        .wrapping_add(source_connector.local_position.y),
                    (Projection::TerrainMatching, Projection::Rigid) => original_box
                        .minimum
                        .y
                        .wrapping_add(target_connector.local_position.y),
                    (Projection::TerrainMatching, Projection::TerrainMatching) => cached_surface
                        .expect("two nonrigid pieces sampled a height")
                        .wrapping_add(delta_y / 2),
                };
                let source_junction = Junction {
                    source_x: outward.x,
                    source_ground_y: junction_y,
                    source_z: outward.z,
                    delta_y,
                    destination_projection: element.projection,
                };
                let target_junction = Junction {
                    source_x: source_position.x,
                    source_ground_y: junction_y,
                    source_z: source_position.z,
                    delta_y: -delta_y,
                    destination_projection: source.element.projection,
                };
                return Some(Attachment {
                    piece: JigsawPiece {
                        element,
                        position: origin,
                        rotation,
                        bounding_box: original_box,
                        ground_level_delta,
                        junctions: vec![target_junction],
                        depth: source.depth.saturating_add(1),
                    },
                    source_junction,
                });
            }
        }
    }
    None
}

fn expand_collision_box(
    mut bounding_box: BlockBox,
    element: &PoolElement,
    rotation: Rotation,
    pools: &BTreeMap<String, TemplatePool>,
    aliases: &BTreeMap<String, String>,
) -> BlockBox {
    let mut maximum_span = 0;
    for connector in element
        .connectors
        .iter()
        .map(|connector| connector.rotated(rotation, element.size))
    {
        let outward = offset_position(connector.local_position, connector.front.step());
        if !element
            .box_at(BlockPos::new(0, 0, 0), rotation)
            .is_some_and(|bounds| bounds.contains(outward))
        {
            continue;
        }
        let pool_name = aliases
            .get(&connector.pool)
            .map_or(connector.pool.as_str(), String::as_str);
        let Some(pool) = pools.get(pool_name) else {
            continue;
        };
        maximum_span = maximum_span.max(pool.maximum_y_span());
        let fallback_name = aliases
            .get(&pool.fallback)
            .map_or(pool.fallback.as_str(), String::as_str);
        if let Some(fallback) = pools.get(fallback_name) {
            maximum_span = maximum_span.max(fallback.maximum_y_span());
        }
    }
    if maximum_span > 0 {
        let old_span = bounding_box.maximum.y - bounding_box.minimum.y;
        bounding_box.maximum.y = bounding_box
            .minimum
            .y
            .wrapping_add(old_span.max(maximum_span + 1));
    }
    bounding_box
}

fn shuffle<T>(values: &mut [T], random: &mut impl GenerationRandom) {
    for index in (1..values.len()).rev() {
        let bound = NonZeroU32::new((index + 1) as u32).expect("shuffle bound is nonzero");
        values.swap(index, random.next_u32(bound) as usize);
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JigsawError {
    #[error("pool weight {0} lies outside 1..=150")]
    Weight(u16),
    #[error("weighted choice is empty, all-zero, or overflows")]
    EmptyWeightedChoice,
    #[error("pool alias {0} is assigned more than once")]
    DuplicateAlias(String),
}
