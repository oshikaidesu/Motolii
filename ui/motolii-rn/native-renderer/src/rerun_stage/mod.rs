mod adapter;
mod gizmo;
mod host_mesh;

// crate 外の rerun_stage:: 到達を分割前と同じ名前に保つ
pub(crate) use adapter::{EmbeddedSpatialStage, StageGizmoAction, StageTransformProjection};
pub(crate) use host_mesh::{
    apply_move_preview_to_geometry, aspect_fit_ndc_rect, canonical_to_fit_ndc,
    fixture_rect_fill_rgba, fixture_rect_stroke_rgba, host_layer_fill_is_visible,
    host_layer_id_from_entity_path, mesh_vertices_from_canonical_corners, rgba_from_u32,
};

const FIXTURE_RECT_FILL_COLOR: u32 = 0xE9_8C_6AFF;
const FIXTURE_RECT_STROKE_COLOR: u32 = 0xEC_D8_FFFF;
const DOCUMENT_RECT_FILL_COLOR: u32 = 0xFFFF_FFFF;
const STAGE_HOST_ERASE_COLOR: u32 = 0x0000_0000;
const DOCUMENT_FRAME_ENTITY: &str = "motolii/document/frame";

#[cfg(test)]
use crate::renderer_core::StagePointerButton;
#[cfg(test)]
use adapter::{egui_pointer_button, stage_modifiers, stage_navigation_events};
#[cfg(test)]
use egui::{Event, Modifiers, MouseWheelUnit, PointerButton, Pos2, Vec2};
#[cfg(test)]
use gizmo::{stage_transform_edit, stage_transform_edit_is_noop};
#[cfg(test)]
use host_mesh::{
    MeshData, hidden_layer_mesh, path_meshes_from_canonical_corners,
    path_stroke_from_canonical_corners, preview_path_operation, rectangle_path, tessellate_path,
};
#[cfg(test)]
use motolii_doc::pathgeom;
#[cfg(test)]
use motolii_ui::AppStageTransformEdit;
#[cfg(test)]
use transform_gizmo::{
    GizmoResult,
    math::{DQuat, DVec3, Transform},
};

#[cfg(test)]
mod tests;
