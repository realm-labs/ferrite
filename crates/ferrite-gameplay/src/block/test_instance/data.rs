//! Durable test-instance records and block-entity synchronization.

use crate::block::test_instance::BLOCK_UPDATE_FLAGS;
use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::resource::ResourceId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TestComponent {
    Literal(String),
    OpaqueAdapterPayload(Vec<u8>),
    Sequence(Vec<TestComponent>),
}

impl TestComponent {
    pub fn literal(text: impl Into<String>) -> Self {
        Self::Literal(text.into())
    }

    pub const fn opaque_adapter_payload(payload: Vec<u8>) -> Self {
        Self::OpaqueAdapterPayload(payload)
    }

    pub fn sequence(parts: impl IntoIterator<Item = Self>) -> Self {
        Self::Sequence(parts.into_iter().collect())
    }
}

impl From<String> for TestComponent {
    fn from(value: String) -> Self {
        Self::Literal(value)
    }
}

impl From<&str> for TestComponent {
    fn from(value: &str) -> Self {
        Self::literal(value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IntVector {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl IntVector {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum QuarterRotation {
    #[default]
    None,
    Clockwise90,
    Clockwise180,
    Counterclockwise90,
}

impl QuarterRotation {
    pub const fn quarter_turns(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Clockwise90 => 1,
            Self::Clockwise180 => 2,
            Self::Counterclockwise90 => 3,
        }
    }

    pub const fn compose(self, extra: Self) -> Self {
        match (self.quarter_turns() + extra.quarter_turns()) % 4 {
            0 => Self::None,
            1 => Self::Clockwise90,
            2 => Self::Clockwise180,
            _ => Self::Counterclockwise90,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    #[default]
    Cleared,
    Running,
    Finished,
}

impl TestStatus {
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::Cleared => 0,
            Self::Running => 1,
            Self::Finished => 2,
        }
    }

    pub const fn from_wire_id(wire_id: i32) -> Self {
        match wire_id {
            1 => Self::Running,
            2 => Self::Finished,
            _ => Self::Cleared,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceData {
    pub test_key: Option<ResourceId>,
    pub size: IntVector,
    pub extra_rotation: QuarterRotation,
    pub ignore_entities: bool,
    pub status: TestStatus,
    pub error: Option<TestComponent>,
}

impl Default for TestInstanceData {
    fn default() -> Self {
        Self {
            test_key: None,
            size: IntVector::default(),
            extra_rotation: QuarterRotation::None,
            ignore_entities: false,
            status: TestStatus::Cleared,
            error: None,
        }
    }
}

impl TestInstanceData {
    pub fn with_status(&self, status: TestStatus) -> Self {
        Self {
            status,
            error: None,
            ..self.clone()
        }
    }

    pub fn with_error(&self, error: impl Into<TestComponent>) -> Self {
        Self {
            status: TestStatus::Finished,
            error: Some(error.into()),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorMarker {
    pub position: BlockPos,
    pub message: TestComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockEntityEffect {
    MarkChunkDirty,
    PublishAirToCurrentState { flags: u16 },
}

pub const SET_CHANGED_EFFECTS: [BlockEntityEffect; 2] = [
    BlockEntityEffect::MarkChunkDirty,
    BlockEntityEffect::PublishAirToCurrentState {
        flags: BLOCK_UPDATE_FLAGS,
    },
];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TestInstanceEntity {
    pub data: TestInstanceData,
    pub error_markers: Vec<ErrorMarker>,
}

impl TestInstanceEntity {
    pub fn set(
        &mut self,
        data: TestInstanceData,
        server_level_attached: bool,
    ) -> Vec<BlockEntityEffect> {
        self.data = data;
        set_changed_effects(server_level_attached)
    }

    pub fn set_status(
        &mut self,
        status: TestStatus,
        server_level_attached: bool,
    ) -> Vec<BlockEntityEffect> {
        self.set(self.data.with_status(status), server_level_attached)
    }

    pub fn set_error(
        &mut self,
        error: impl Into<TestComponent>,
        server_level_attached: bool,
    ) -> Vec<BlockEntityEffect> {
        self.set(self.data.with_error(error), server_level_attached)
    }

    pub fn add_error_marker(
        &mut self,
        marker: ErrorMarker,
        server_level_attached: bool,
    ) -> Vec<BlockEntityEffect> {
        self.error_markers.push(marker);
        set_changed_effects(server_level_attached)
    }

    pub fn clear_error_markers(&mut self, server_level_attached: bool) -> Vec<BlockEntityEffect> {
        if self.error_markers.is_empty() {
            return Vec::new();
        }
        self.error_markers.clear();
        set_changed_effects(server_level_attached)
    }

    pub fn load(
        &mut self,
        decoded_data: Option<TestInstanceData>,
        decoded_markers: Option<Vec<ErrorMarker>>,
        server_level_attached: bool,
    ) -> Vec<BlockEntityEffect> {
        let effects =
            decoded_data.map_or_else(Vec::new, |data| self.set(data, server_level_attached));
        self.error_markers.clear();
        self.error_markers
            .extend(decoded_markers.unwrap_or_default());
        effects
    }

    pub fn save(&self) -> SavedTestInstance {
        SavedTestInstance {
            data: self.data.clone(),
            errors: (!self.error_markers.is_empty()).then(|| self.error_markers.clone()),
        }
    }

    pub fn update_payload(&self) -> SavedTestInstance {
        self.save()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedTestInstance {
    pub data: TestInstanceData,
    pub errors: Option<Vec<ErrorMarker>>,
}

pub fn set_changed_effects(server_level_attached: bool) -> Vec<BlockEntityEffect> {
    if server_level_attached {
        SET_CHANGED_EFFECTS.to_vec()
    } else {
        Vec::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

impl TestAction {
    pub const fn wire_id(self) -> i32 {
        match self {
            Self::Init => 0,
            Self::Query => 1,
            Self::Set => 2,
            Self::Reset => 3,
            Self::Save => 4,
            Self::Export => 5,
            Self::Run => 6,
        }
    }

    pub const fn from_wire_id(wire_id: i32) -> Self {
        match wire_id {
            1 => Self::Query,
            2 => Self::Set,
            3 => Self::Reset,
            4 => Self::Save,
            5 => Self::Export,
            6 => Self::Run,
            _ => Self::Init,
        }
    }
}
