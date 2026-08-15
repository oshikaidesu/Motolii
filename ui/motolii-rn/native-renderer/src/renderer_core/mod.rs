mod host;
mod input;
mod lifecycle;
mod present;
mod scrub;
mod sync;
mod types;

#[cfg(test)]
mod tests_input;
#[cfg(test)]
mod tests_projection;

pub use types::RenderStats;
pub(crate) use types::{NativeHostTerminalEvent, PointerPhase, SceneKind, StagePointerButton};

use std::time::Instant;

use host::HostTerminalLatch;
use scrub::ScrubTimePump;
use types::{StageResources, TimelineResources};

// 子moduleのimplがprivate fieldへ届くよう、struct本体は親に置く。
pub(crate) struct RendererCore {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    _adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    stage: Option<StageResources>,
    timeline: Option<TimelineResources>,
    timeline_session: Option<crate::timeline_skia::TimelineSession>,
    /// Host snapshotのrevision。変化時だけsceneを差し替える。
    host_revision: Option<String>,
    /// process内host再生成とterminal遅配を区別するstable identity。
    host_handle: Option<String>,
    /// set_timeはrevisionを進めないため、playhead追従はgenerationで見る。
    host_projection_generation: Option<String>,
    host_stage_geometry: Option<crate::host_bridge::HostStageGeometry>,
    /// host 投影メッシュ適用時の viewport（aspect 再適用判定）。
    host_stage_viewport: Option<(u32, u32)>,
    host_fps: Option<(i64, i64)>,
    stage_gizmo_pointer_active: bool,
    scrubbing: bool,
    scrub_time_pump: ScrubTimePump,
    scrub_clock_start: Instant,
    force_next_host_snapshot: bool,
    /// F9: 前回読んだhost stamp。(revision, generation)。未取得はNone。
    host_projection_stamp: Option<(u64, u64)>,
    /// F11: mount時warm-upを1回だけ先払いしたか。resizeでは再実行しない。
    mount_warmup_done: bool,
    /// native操作の終端だけをRNへ一度返す。preview moveは記録しない。
    host_terminal_latch: HostTerminalLatch,
    scene: SceneKind,
    selected_object_index: i32,
    playhead: f64,
    frame: u64,
    stats: RenderStats,
}
