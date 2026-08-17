mod adapter;
mod document_camera;
mod gizmo;
mod host_mesh;

// crate 外の rerun_stage:: 到達を分割前と同じ名前に保つ
pub use adapter::{
    EmbeddedSpatialStage, PointerPhase, StageGizmoAction, StagePointerButton,
    StageTransformProjection,
};
pub use host_mesh::{
    apply_move_preview_to_geometry, host_layer_fill_is_visible, host_layer_id_from_entity_path,
    mesh_vertices_from_canonical_corners,
};

// なぜ FIXTURE_RECT_FILL_COLOR が無いか: fixture 経路(fixture_rect_fill_rgba 等)を
// 今回は移さないため、この色を読む者がいない。
const FIXTURE_RECT_STROKE_COLOR: u32 = 0xEC_D8_FFFF;
const DOCUMENT_RECT_FILL_COLOR: u32 = 0xFFFF_FFFF;
const STAGE_HOST_ERASE_COLOR: u32 = 0x0000_0000;
const DOCUMENT_FRAME_ENTITY: &str = "motolii/document/frame";
