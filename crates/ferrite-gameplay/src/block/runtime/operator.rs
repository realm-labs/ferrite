//! Operator-block records and edit ordering independent of packet/worldgen adapters.

use crate::block::runtime::geometry::QuarterTurn;
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrontAndTop {
    pub front: Direction,
    pub top: Direction,
}

pub const JIGSAW_ORIENTATIONS: [FrontAndTop; 12] = [
    orientation(Direction::Down, Direction::East),
    orientation(Direction::Down, Direction::North),
    orientation(Direction::Down, Direction::South),
    orientation(Direction::Down, Direction::West),
    orientation(Direction::Up, Direction::East),
    orientation(Direction::Up, Direction::North),
    orientation(Direction::Up, Direction::South),
    orientation(Direction::Up, Direction::West),
    orientation(Direction::West, Direction::Up),
    orientation(Direction::East, Direction::Up),
    orientation(Direction::North, Direction::Up),
    orientation(Direction::South, Direction::Up),
];

const fn orientation(front: Direction, top: Direction) -> FrontAndTop {
    FrontAndTop { front, top }
}

pub const fn jigsaw_placement(clicked_face: Direction, horizontal: Direction) -> FrontAndTop {
    let top = if clicked_face.is_horizontal() {
        Direction::Up
    } else {
        horizontal.opposite()
    };
    FrontAndTop {
        front: clicked_face,
        top,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JigsawJoint {
    Rollable,
    Aligned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawRecord {
    pub name: ResourceId,
    pub target: ResourceId,
    pub pool: ResourceId,
    pub final_state: String,
    pub joint: JigsawJoint,
    pub placement_priority: i32,
    pub selection_priority: i32,
}

impl JigsawRecord {
    pub fn fresh() -> Self {
        Self {
            name: empty_id(),
            target: empty_id(),
            pool: empty_id(),
            final_state: "minecraft:air".to_owned(),
            joint: JigsawJoint::Rollable,
            placement_priority: 0,
            selection_priority: 0,
        }
    }

    pub fn load_defaults(front: Direction) -> Self {
        Self {
            joint: if front.is_horizontal() {
                JigsawJoint::Aligned
            } else {
                JigsawJoint::Rollable
            },
            ..Self::fresh()
        }
    }

    pub fn apply(&mut self, edit: JigsawEdit) {
        self.name = edit.name;
        self.target = edit.target;
        self.pool = edit.pool;
        self.final_state = edit.final_state;
        self.joint = edit.joint;
        self.placement_priority = edit.placement_priority;
        self.selection_priority = edit.selection_priority;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JigsawEdit {
    pub name: ResourceId,
    pub target: ResourceId,
    pub pool: ResourceId,
    pub final_state: String,
    pub joint: JigsawJoint,
    pub placement_priority: i32,
    pub selection_priority: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMode {
    Save,
    Load,
    Corner,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureMirror {
    None,
    LeftRight,
    FrontBack,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureRecord {
    pub name: Option<ResourceId>,
    pub author: String,
    pub metadata: String,
    pub offset: [i32; 3],
    pub size: [i32; 3],
    pub mirror: StructureMirror,
    pub rotation: QuarterTurn,
    pub mode: StructureMode,
    pub ignore_entities: bool,
    pub strict: bool,
    pub powered: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
}

impl StructureRecord {
    pub fn fresh(mode: StructureMode) -> Self {
        Self {
            name: None,
            author: String::new(),
            metadata: String::new(),
            offset: [0, 1, 0],
            size: [0; 3],
            mirror: StructureMirror::None,
            rotation: QuarterTurn::None,
            mode,
            ignore_entities: true,
            strict: false,
            powered: false,
            show_air: false,
            show_bounding_box: true,
            integrity: 1.0,
            seed: 0,
        }
    }

    pub fn load_defaults() -> Self {
        Self::fresh(StructureMode::Data)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureAction {
    UpdateData,
    SaveArea,
    LoadArea,
    ScanArea,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructureEdit {
    pub mode: StructureMode,
    pub raw_name: String,
    pub offset: [i32; 3],
    pub size: [i32; 3],
    pub mirror: StructureMirror,
    pub rotation: QuarterTurn,
    pub metadata: String,
    pub ignore_entities: bool,
    pub strict: bool,
    pub show_air: bool,
    pub show_bounding_box: bool,
    pub integrity: f32,
    pub seed: i64,
    pub action: StructureAction,
}

impl StructureEdit {
    pub fn bounded(mut self) -> Self {
        self.offset = self.offset.map(|value| value.clamp(-48, 48));
        self.size = self.size.map(|value| value.clamp(0, 48));
        self.integrity = self.integrity.clamp(0.0, 1.0);
        self.metadata.truncate(128);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateProbe {
    NotChecked,
    SaveSucceeded,
    SaveFailed,
    Missing,
    EqualSize,
    UnequalSize([i32; 3]),
    ScanSucceeded([i32; 3], [i32; 3]),
    ScanFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructureEditOutcome {
    InvalidName,
    Updated,
    Saved(bool),
    Missing,
    Loaded,
    Prepared,
    Scanned(bool),
}

pub fn apply_structure_edit(
    record: &mut StructureRecord,
    edit: StructureEdit,
    probe: TemplateProbe,
) -> StructureEditOutcome {
    let edit = edit.bounded();
    record.mode = edit.mode;
    record.name = ResourceId::parse_with_default_namespace(&edit.raw_name).ok();
    record.offset = edit.offset;
    record.size = edit.size;
    record.mirror = edit.mirror;
    record.rotation = edit.rotation;
    record.metadata = edit.metadata;
    record.ignore_entities = edit.ignore_entities;
    record.strict = edit.strict;
    record.show_air = edit.show_air;
    record.show_bounding_box = edit.show_bounding_box;
    record.integrity = edit.integrity;
    record.seed = edit.seed;

    if record.name.is_none() {
        return StructureEditOutcome::InvalidName;
    }
    match (edit.action, probe) {
        (StructureAction::UpdateData, _) => StructureEditOutcome::Updated,
        (StructureAction::SaveArea, TemplateProbe::SaveSucceeded) => {
            StructureEditOutcome::Saved(true)
        }
        (StructureAction::SaveArea, _) => StructureEditOutcome::Saved(false),
        (StructureAction::LoadArea, TemplateProbe::Missing) => StructureEditOutcome::Missing,
        (StructureAction::LoadArea, TemplateProbe::EqualSize) => StructureEditOutcome::Loaded,
        (StructureAction::LoadArea, TemplateProbe::UnequalSize(size)) => {
            record.size = size;
            StructureEditOutcome::Prepared
        }
        (StructureAction::ScanArea, TemplateProbe::ScanSucceeded(offset, size)) => {
            record.offset = offset;
            record.size = size;
            StructureEditOutcome::Scanned(true)
        }
        (StructureAction::ScanArea, _) => StructureEditOutcome::Scanned(false),
        (StructureAction::LoadArea, _) => StructureEditOutcome::Missing,
    }
}

pub fn detect_structure_bounds(
    save_position: BlockPos,
    corners: &[BlockPos],
) -> Option<([i32; 3], [i32; 3])> {
    let (minimum, maximum) = match corners {
        [] => return None,
        [corner] => (
            [
                corner.x.min(save_position.x),
                corner.y.min(save_position.y),
                corner.z.min(save_position.z),
            ],
            [
                corner.x.max(save_position.x),
                corner.y.max(save_position.y),
                corner.z.max(save_position.z),
            ],
        ),
        corners => {
            let mut minimum = [i32::MAX; 3];
            let mut maximum = [i32::MIN; 3];
            for corner in corners {
                for (axis, value) in [corner.x, corner.y, corner.z].into_iter().enumerate() {
                    minimum[axis] = minimum[axis].min(value);
                    maximum[axis] = maximum[axis].max(value);
                }
            }
            (minimum, maximum)
        }
    };
    let delta = [
        maximum[0] - minimum[0],
        maximum[1] - minimum[1],
        maximum[2] - minimum[2],
    ];
    if delta.into_iter().any(|value| value <= 1) {
        return None;
    }
    Some((
        [
            minimum[0] - save_position.x + 1,
            minimum[1] - save_position.y + 1,
            minimum[2] - save_position.z + 1,
        ],
        [delta[0] - 1, delta[1] - 1, delta[2] - 1],
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedstoneStructureAction {
    None,
    SaveMemory,
    LoadImmediately,
    RemoveCached,
}

pub fn structure_redstone_edge(
    record: &mut StructureRecord,
    neighbor_powered: bool,
) -> RedstoneStructureAction {
    if record.powered == neighbor_powered {
        return RedstoneStructureAction::None;
    }
    record.powered = neighbor_powered;
    if !neighbor_powered {
        return RedstoneStructureAction::None;
    }
    match record.mode {
        StructureMode::Save => RedstoneStructureAction::SaveMemory,
        StructureMode::Load => RedstoneStructureAction::LoadImmediately,
        StructureMode::Corner => RedstoneStructureAction::RemoveCached,
        StructureMode::Data => RedstoneStructureAction::None,
    }
}

fn empty_id() -> ResourceId {
    ResourceId::minecraft("empty").expect("locked identifier")
}
