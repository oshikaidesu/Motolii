use crate::rerun_stage::EmbeddedSpatialStage;
use motolii_gpu::GpuCtx;
use motolii_render::RenderSession;
use motolii_ui::AppStageFrame;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SceneKind {
    Stage,
    Timeline,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderStats {
    pub frame_count: u64,
    pub last_cpu_us: u64,
    pub max_cpu_us: u64,
    pub vertex_bytes: u64,
    pub overlay_uploads: u64,
    pub overlay_last_us: u64,
    pub pointer_downs: u64,
    pub pointer_moves: u64,
    pub pointer_ups: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointerPhase {
    Down,
    Move,
    Up,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StagePointerButton {
    Primary,
    Secondary,
    Middle,
}

impl StagePointerButton {
    pub(crate) fn from_raw(raw: u32) -> Option<Self> {
        match raw {
            0 => Some(Self::Primary),
            1 => Some(Self::Secondary),
            2 => Some(Self::Middle),
            _ => None,
        }
    }
}

pub(super) struct TimelineResources {
    pub(super) surface_texture: wgpu::Texture,
    pub(super) blit_pipeline: wgpu::RenderPipeline,
    pub(super) blit_bind_group: wgpu::BindGroup,
    pub(super) pixels: Vec<u8>,
    pub(super) dirty: bool,
}

pub(super) struct StageResources {
    pub(super) rerun: EmbeddedSpatialStage,
    pub(super) preview_active: bool,
    pub(super) gpu: GpuCtx,
    pub(super) session: RenderSession,
    pub(super) frame: Option<AppStageFrame>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeHostTerminalEvent {
    pub(crate) accepted: bool,
    pub(crate) message: String,
}
