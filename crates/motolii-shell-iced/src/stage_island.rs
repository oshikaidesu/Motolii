//! Stage 島 — Rerun `SpatialStage` の合成絵を iced の shader widget から出す。
//!
//! `spikes/iced-rerun-embed-probe` の製品 adapter 化(M-2)。GPU の実体は
//! `motolii_ui::rerun_stage::EmbeddedSpatialStage`(RN native surface と同じ
//! toolkit 中立の口)で、この module は iced 側の受け皿だけを持つ:
//!
//! ```text
//! iced::Event ─ stage_bridge::translate ─ stage_arbiter::route ─┐(素通し/orbit中だけ)
//!                                                               ▼
//! iced runtime device ─→ EmbeddedSpatialStage::render ─→ offscreen texture
//!                     └→ blit ─→ iced の render pass(この widget の bounds)
//! ```
//!
//! egui はこの crate に**入っていない**。`EmbeddedSpatialStage` が自前 egui を
//! 1周だけ回す(`render()` 経路)— 「egui は Stage 島の内側の実装詳細に縮む」
//! (2026-08-18 ホスト移行裁定)の実装がこの形である。
//!
//! ## 失敗は黙らない
//!
//! Stage の初期化・texture import・描画の失敗は [`Message::StageReported`] になって
//! `Shell::update` が transcript(帯 / `--status-log`)へ写す。溜め場は
//! この module の mailbox で、`RedrawRequested`(毎フレーム widget 木へ来る)で
//! 運転席からも同じ道で取り出せる。
//!
//! ## bind group の床(fork seam 2)
//!
//! iced fork の `iced_wgpu::device_limits` は device 要求時の `max_bind_groups` の
//! **床**を公開している([iced fork seam 台帳]
//! (../../../docs/reviews/2026-08-18-iced-fork-seam-ledger.md) §3)。
//! [`install_rerun_device_floor`] を**窓 / headless renderer を建てる前に**呼ぶ。
//! 効いていることの常設 oracle は `tests/stage_bind_groups_oracle.rs`。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use iced::mouse;
use iced::widget::shader::{self, Viewport};
use iced::{Element, Fill, Rectangle};

use crate::message::Message;
use crate::shell::Shell;
use crate::stage_arbiter::GrabRegion;

/// Rerun(`re_renderer`)が device に要求する bind group 数。
///
/// 出所は `re_renderer` の device 記述(`downlevel_webgl2_defaults` 由来の 4)。
/// 定数で持つのは、窓を建てる**前**(adapter がまだ無い時点)に床を上げる必要が
/// あるから。値が `re_renderer` の実要求を下回っていないことは
/// `tests/stage_bind_groups_oracle.rs` が adapter 実物で照合する。
pub const RERUN_MIN_MAX_BIND_GROUPS: u32 = 4;

/// fork seam 2 の床を上げる。**窓 / headless renderer を建てる前に**呼ぶ
/// (既に在る device には効かない)。何度呼んでも下がらない(床は上がる一方)。
pub fn install_rerun_device_floor() {
    // red: 未実装。
}

/// Stage 島の widget。座席が在る間の中央 pane。
pub fn stage_island(shell: &Shell) -> Element<'_, Message> {
    // red: 未実装 — まだ絵の出ない席だけを返す。
    let _ = shell;
    iced::widget::container(iced::widget::text(""))
        .width(Fill)
        .height(Fill)
        .into()
}

/// shader widget の `Program`。製品は `grab_probe = None`(掴みは発生しない)。
/// テストはダミー領域を差して3状態を審判する。
#[derive(Debug, Clone, Copy)]
pub struct StageIsland {
    /// composition の縦横比(幅/高さ)。座席の Document から。
    pub composition_aspect: Option<f32>,
    /// 掴み判定の seam(M-2 はテスト専用のダミー。ギズモ本体は M-2 後)。
    pub grab_probe: Option<GrabRegion>,
}

/// `Primitive` は `Debug + Send + Sync` を要求するので素のデータしか持てない。
/// GPU 資源は thread_local(`EMBED`)が持つ。
#[derive(Debug, Clone, Copy)]
pub struct StagePrimitive {
    composition_aspect: Option<f32>,
}

/// `Pipeline::new` は iced のランタイム device を渡してくる唯一の場所。
pub struct StagePipeline;

impl shader::Pipeline for StagePipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        // red: 未実装。
        let _ = (device, queue, format);
        Self
    }
}

impl shader::Primitive for StagePrimitive {
    type Pipeline = StagePipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        // red: 未実装。
        let _ = (pipeline, device, queue, bounds, viewport);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        // red: 未実装 — 何も描かない。
        let _ = (pipeline, render_pass);
        false
    }
}

impl shader::Program<Message> for StageIsland {
    type State = crate::stage_arbiter::StagePointerOwner;
    type Primitive = StagePrimitive;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        // red: 未実装。
        let _ = (state, event, bounds, cursor);
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        _cursor: mouse::Cursor,
        _bounds: Rectangle,
    ) -> Self::Primitive {
        StagePrimitive {
            composition_aspect: self.composition_aspect,
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        // red: 未実装。
        let _ = (state, bounds, cursor);
        mouse::Interaction::default()
    }
}

// ---------------------------------------------------------------------------
// 観測の口(常設 oracle・運転席が読む)
// ---------------------------------------------------------------------------

/// `Pipeline::new` が渡された device の `max_bind_groups` の履歴(古い順)。
///
/// fork seam 2 の**実効** oracle がここを読む: 床を上げる前は上流既定の 2、
/// 上げた後は [`RERUN_MIN_MAX_BIND_GROUPS`] 以上が**取得**できていること。
pub fn observed_max_bind_groups() -> Vec<u32> {
    OBSERVED_MAX_BIND_GROUPS
        .lock()
        .map(|seen| seen.clone())
        .unwrap_or_default()
}

/// ブリッジを渡った入力の内訳(forwarded, swallowed)。調停の GPU 側審判が読む。
pub fn input_tally() -> (u64, u64) {
    (
        FORWARDED.load(Ordering::Relaxed),
        SWALLOWED.load(Ordering::Relaxed),
    )
}

/// 評価済みフレーム(合成絵)の席。RGBA8(sRGB, premultiplied)を CPU から差す。
///
/// M-2 ではテストの既知絵(E0 と同じ4象限)がここへ入り、pixel oracle が
/// 正対既定を審判する。M-3 で `stage_frame_seat`(playhead 時刻の合成フレーム、
/// GPU 常駐)がこの席を置き換える。
pub fn install_probe_frame(width: u32, height: u32, rgba: Vec<u8>) {
    if let Ok(mut slot) = PROBE_FRAME.lock() {
        *slot = Some(ProbeFrame {
            width,
            height,
            rgba,
        });
    }
}

struct ProbeFrame {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

static OBSERVED_MAX_BIND_GROUPS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static FORWARDED: AtomicU64 = AtomicU64::new(0);
static SWALLOWED: AtomicU64 = AtomicU64::new(0);
static PROBE_FRAME: Mutex<Option<ProbeFrame>> = Mutex::new(None);
