//! Local editor behavior and client-observable test-instance rendering.

use crate::block::test_instance::data::{
    ErrorMarker, IntVector, QuarterRotation, TestAction, TestComponent, TestInstanceData,
    TestStatus,
};
use crate::block::test_instance::geometry::{effective_rotation, transformed_size};
use crate::block::test_instance::operations::{ConfiguredTest, StatusResponse};
use ferrite_foundation::resource::ResourceId;

pub const IDENTIFIER_CODE_UNIT_LIMIT: usize = 128;
pub const SIZE_CODE_UNIT_LIMIT: usize = 15;
pub const SIZE_MINIMUM: i32 = 1;
pub const SIZE_MAXIMUM: i32 = 48;
pub const BEAM_FINAL_HEIGHT: u16 = 2_048;
pub const MARKER_INFLATION: f32 = 0.02;
pub const MARKER_ALPHA: f32 = 0.375;
pub const MARKER_TEXT_HEIGHT: f32 = 1.2;
pub const MARKER_TEXT_SCALE: f32 = 0.16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseWithoutItem {
    Pass,
    Success { open_local_editor: bool },
}

pub const fn use_without_item(
    matching_block_entity: bool,
    can_use_game_master_blocks: bool,
    client_side: bool,
) -> UseWithoutItem {
    if !matching_block_entity || !can_use_game_master_blocks {
        UseWithoutItem::Pass
    } else {
        UseWithoutItem::Success {
            open_local_editor: client_side,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorButton {
    Reset,
    Save,
    Export,
    Run,
    Done,
    Cancel,
    Escape,
    OrdinaryClose,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPacket {
    pub action: TestAction,
    pub data: TestInstanceData,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorSubmission {
    pub close_screen: bool,
    pub packet: Option<ActionPacket>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierEdit {
    pub packet: ActionPacket,
    pub local_invalid_identifier: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestInstanceEditor {
    pub identifier: String,
    pub size_x: String,
    pub size_y: String,
    pub size_z: String,
    pub selected_rotation: QuarterRotation,
    pub include_entities: bool,
    pub description: TestComponent,
    pub export_visible: bool,
}

impl TestInstanceEditor {
    pub fn open(
        data: &TestInstanceData,
        intrinsic_rotation: QuarterRotation,
        ide_build: bool,
    ) -> (Self, ActionPacket) {
        let editor = Self {
            identifier: data
                .test_key
                .as_ref()
                .map_or_else(String::new, ToString::to_string),
            size_x: data.size.x.to_string(),
            size_y: data.size.y.to_string(),
            size_z: data.size.z.to_string(),
            selected_rotation: effective_rotation(intrinsic_rotation, data.extra_rotation),
            include_entities: !data.ignore_entities,
            description: TestComponent::literal(""),
            export_visible: ide_build,
        };
        let packet = editor.packet(TestAction::Init);
        (editor, packet)
    }

    pub fn edit_identifier(&mut self, text: &str) -> IdentifierEdit {
        self.identifier = truncate_utf16(text, IDENTIFIER_CODE_UNIT_LIMIT);
        let local_invalid_identifier = parse_identifier(&self.identifier).is_none();
        if local_invalid_identifier {
            self.description = TestComponent::literal("Invalid test identifier");
        }
        IdentifierEdit {
            packet: self.packet(TestAction::Query),
            local_invalid_identifier,
        }
    }

    pub fn set_size_text(&mut self, axis: usize, text: &str) {
        let value = truncate_utf16(text, SIZE_CODE_UNIT_LIMIT);
        match axis {
            0 => self.size_x = value,
            1 => self.size_y = value,
            _ => self.size_z = value,
        }
    }

    pub fn save_or_export_active(&self) -> bool {
        parse_identifier(&self.identifier).is_some()
            && self.selected_rotation == QuarterRotation::None
    }

    pub fn submit(&self, button: EditorButton) -> EditorSubmission {
        let action = match button {
            EditorButton::Reset => Some(TestAction::Reset),
            EditorButton::Save => Some(TestAction::Save),
            EditorButton::Export => Some(TestAction::Export),
            EditorButton::Run => Some(TestAction::Run),
            EditorButton::Done => Some(TestAction::Set),
            EditorButton::Cancel | EditorButton::Escape | EditorButton::OrdinaryClose => None,
        };
        EditorSubmission {
            close_screen: true,
            packet: action.map(|action| self.packet(action)),
        }
    }

    pub fn receive_status(
        &mut self,
        response: &StatusResponse,
        synchronized_entity_error: Option<&TestComponent>,
    ) {
        self.description = synchronized_entity_error.map_or_else(
            || response.description.clone(),
            |error| {
                TestComponent::sequence([
                    error.clone(),
                    TestComponent::literal(": "),
                    response.description.clone(),
                ])
            },
        );
        if let Some(size) = response.size {
            self.size_x = size.x.to_string();
            self.size_y = size.y.to_string();
            self.size_z = size.z.to_string();
        }
    }

    pub fn packet(&self, action: TestAction) -> ActionPacket {
        ActionPacket {
            action,
            data: TestInstanceData {
                test_key: parse_identifier(&self.identifier),
                size: IntVector::new(
                    parse_size(&self.size_x),
                    parse_size(&self.size_y),
                    parse_size(&self.size_z),
                ),
                extra_rotation: self.selected_rotation,
                ignore_entities: !self.include_entities,
                status: TestStatus::Cleared,
                error: None,
            },
        }
    }
}

fn parse_identifier(text: &str) -> Option<ResourceId> {
    (!text.is_empty())
        .then(|| ResourceId::parse_with_default_namespace(text).ok())
        .flatten()
}

fn parse_size(text: &str) -> i32 {
    text.parse::<i32>()
        .unwrap_or(SIZE_MINIMUM)
        .clamp(SIZE_MINIMUM, SIZE_MAXIMUM)
}

pub fn truncate_utf16(text: &str, maximum_units: usize) -> String {
    let mut units = 0;
    text.chars()
        .take_while(|character| {
            let next = units + character.len_utf16();
            if next > maximum_units {
                false
            } else {
                units = next;
                true
            }
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeamColor {
    Gray,
    Green,
    Red,
    Orange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BeamProjection {
    pub color: BeamColor,
    pub opaque: bool,
    pub final_height: u16,
    pub permission_gated: bool,
    pub ordinary_beacon_animation: bool,
    pub distance_scaling: bool,
    pub horizontal_distance_admission: bool,
}

pub fn beam_projection(
    data: &TestInstanceData,
    configured: Option<&ConfiguredTest>,
) -> Option<BeamProjection> {
    let color = match (data.status, data.error.is_some()) {
        (TestStatus::Cleared, _) => return None,
        (TestStatus::Running, _) => BeamColor::Gray,
        (TestStatus::Finished, false) => BeamColor::Green,
        (TestStatus::Finished, true) => match configured {
            Some(test) if !test.required => BeamColor::Orange,
            Some(_) | None => BeamColor::Red,
        },
    };
    Some(BeamProjection {
        color,
        opaque: true,
        final_height: BEAM_FINAL_HEIGHT,
        permission_gated: false,
        ordinary_beacon_animation: true,
        distance_scaling: true,
        horizontal_distance_admission: true,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundsProjection {
    pub offset: IntVector,
    pub transformed_size: IntVector,
    pub always_box: bool,
    pub opaque_light_gray: bool,
    pub invisible_cells: bool,
}

pub fn bounds_projection(
    data: &TestInstanceData,
    configured: Option<&ConfiguredTest>,
    can_use_game_master_blocks: bool,
    spectator: bool,
) -> Option<BoundsProjection> {
    if !can_use_game_master_blocks && !spectator {
        return None;
    }
    let (intrinsic, padding) = configured.map_or((QuarterRotation::None, 0), |test| {
        (test.intrinsic_rotation, test.padding)
    });
    let size = transformed_size(
        data.size,
        effective_rotation(intrinsic, data.extra_rotation),
    );
    if size.x < 1 || size.y < 1 || size.z < 1 {
        return None;
    }
    Some(BoundsProjection {
        offset: IntVector::new(padding, padding.wrapping_add(1), padding.wrapping_add(1)),
        transformed_size: size,
        always_box: true,
        opaque_light_gray: true,
        invisible_cells: false,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkerProjection {
    pub marker: ErrorMarker,
    pub red_filled_cube: bool,
    pub cube_inflation: f32,
    pub cube_alpha: f32,
    pub white_centered_text: bool,
    pub always_on_top: bool,
    pub text_height: f32,
    pub text_scale: f32,
    pub permission_gated: bool,
}

pub fn marker_projections(markers: &[ErrorMarker]) -> Vec<MarkerProjection> {
    markers
        .iter()
        .cloned()
        .map(|marker| MarkerProjection {
            marker,
            red_filled_cube: true,
            cube_inflation: MARKER_INFLATION,
            cube_alpha: MARKER_ALPHA,
            white_centered_text: true,
            always_on_top: true,
            text_height: MARKER_TEXT_HEIGHT,
            text_scale: MARKER_TEXT_SCALE,
            permission_gated: false,
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CombinedRenderAdmission {
    pub offscreen_eligible: bool,
    pub admitted: bool,
}

pub const fn combined_render_admission(
    beam_horizontal_distance_admitted: bool,
    bounds_renderer_admitted: bool,
) -> CombinedRenderAdmission {
    CombinedRenderAdmission {
        offscreen_eligible: true,
        admitted: beam_horizontal_distance_admitted || bounds_renderer_admitted,
    }
}
