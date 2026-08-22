//! wraps: iced — front。**store への query の投影**であって、Document の写しを持たない。
//!
//! 背骨1 を型で作る:
//! - **書き口は [`Shell::update`] の1箇所だけ**。pane 関数は `StoreView`(不変)・
//!   `&Session`・[`tokens::Tokens`](裁定117、寸法・色。Document 由来ではなく書けない)
//!   しか受け取らないので、**書ける物を持っていない**
//! - `view(&self)` が `&self` を取るので、描画中に Document を触る道が無い
//!
//! Stage は **CPU 経路**(合成は CPU、`Engine::render_frame` の RGBA を作る所まで)
//! だが、**表示だけは裁定166 で GPU 常駐テクスチャへ変えた** — 合成結果の RGBA を
//! `iced::widget::shader` の自前 Program(永続 `wgpu::Texture` + 世代ゲート付き
//! `queue.write_texture`)へ渡す。旧実装(`image::Handle::from_rgba` を毎フレーム
//! 新規発行)は iced_wgpu の非同期アップロード境界(2MB)を超えると「その間
//! 何も描かない」穴があり、実機のイージングのガタつきの一次原因だった
//! (`docs/reviews/2026-08-21-stage-presenter-decision.md`)。永続テクスチャ経路
//! にはその穴が無いので、フル解像度のまま描ける。iced の device の上に
//! `re_renderer` を建てる道(合成そのものを GPU へ持ち込む道)は裁定44 で撤回
//! したまま — ここで変わったのは「CPU が作った RGBA を GPU へどう見せるか」だけ。
//!
//! **front が持ってよい状態**は [`Session`] だけ — 選択と再生位置。これらは
//! Document の写しではなく、undo の対象でもない(rerun も選択は blueprint store の
//! 外に置いている)。**1箇所で持ち、全 pane がそこを読む**ので M14 は満たされる。

use std::sync::Arc;

use iced::widget::{
    button, column, container, pane_grid, row, shader, stack, text, tooltip, Shader, Space,
};
use iced::{wgpu, Element, Length, Task};

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_engine::{Engine, ObservationCamera};
use motolii_store::{
    AssetDraft, Composition, DisplayRevision, Document, Intent, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, RationalTime, ResolvedLayer, Revision, SourceFingerprintV1,
    Speed, StoreView,
};

pub mod clipboard;
/// File 束(MB-1、裁定176)の OS 副作用注入口([`FileDialogs`] trait +
/// production 実装 [`RfdDialogs`])。`Shell::new_with_dialogs`/test の fake が
/// 外から参照するため `pub`(`file_dialogs.rs` 冒頭 doc 参照)。
pub mod file_dialogs;
pub mod fixture;
/// header のメニューバーの意味定義(MB-2、`menu.rs` 冒頭 doc 参照)。MB-2 で
/// `pub` 化した — `TOP_LEVEL_LABELS`/`menus()` が対応表の正本で、q0_fence
/// (menubar のバー領域除外)と menu_drive が外から読む。
pub mod menu;
/// shell の pane_grid 化(2026-08-22 実装レーン)。`Shell::view()` の layout
/// 状態と純粋な構成ロジック(`pane_layout.rs` 冒頭 doc 参照)。`screenshot.rs`
/// も左カラム幅の近似に [`pane_layout::Ratios`] の既定値を読むため `pub`。
pub mod pane_layout;
pub mod screenshot;
pub mod transport;

/// `state`(`Session`/`KeySelector`/`KeySelectionOp`)は裁定160 切片6 で
/// `motolii-shell` 内の module へ移設し、切片7(timeline-pane crate 抽出、
/// pane split survey `docs/reviews/2026-08-21-pane-split-survey.md` §2.3/§6
/// 切片7)で `motolii-shell-state` crate へさらに抽出した — `motolii-timeline-pane`
/// (pane crate)は `motolii-shell` へ依存できない(循環になる)ので、
/// `Session`/`KeySelector` を両者の共通の親として leaf crate 化する必要が
/// あった。**`motolii-inspector-pane`(切片8、下記)も同じ理由でこの leaf
/// crate へ依存する** — `inspector_pane::project` は `&Session` を直接取る
/// (切片7以前は `Session` が root に残っていたため、切片8は一時的に
/// `selection`/`playhead` の2引数へ分解する回避策を取っていたが、切片7の
/// leaf crate 化でこの回避策は不要になったので、この rebase で `&Session` を
/// 直接取る形へ戻した)。`pub use timeline as timeline_pane;`(旧)と同じ
/// 「型 alias で外部参照を壊さない」手口 — `crate::Session`/
/// `motolii_shell::Session` を読む既存参照(`tests/suite/*.rs`)は無改修で済む。
pub use motolii_shell_state as state;
pub use state::Session;

/// `inspector_pane` は裁定160 切片8 で `motolii-inspector-pane` crate へ抽出
/// 済み(pane split survey §6 切片8)。`pub use timeline as timeline_pane;` と
/// 同じ「型 alias で外部参照を壊さない」手口 — 既存の `crate::inspector_pane::X`・
/// `motolii_shell::inspector_pane::X` 参照(`screenshot.rs`・`tests/suite/*.rs`)
/// は無改修のまま通る。書き口(`commit_inspector_field`/`commit_inspector_name`/
/// `start_field_drag`/`continue_field_drag`/`finish_field_drag`/
/// `cancel_field_interaction`)もこの crate 側へ移設済み — `Shell` 側は
/// それらを呼ぶ薄い glue メソッドだけを持つ(下記参照)。
pub use motolii_inspector_pane as inspector_pane;

/// `tokens` は裁定160 切片1 で `motolii-tokens-rs` crate へ抽出済み(pane split
/// survey §2.2/§6)。純粋な再配置 — `tokens.rs` 自体の値・シグネチャは無改変。
/// 「型 alias で外部参照を壊さない」手口(既存の `crate::tokens::X`・
/// `motolii_shell::tokens::X` 参照はここを直せば無改修で済む)。
pub use motolii_tokens_rs as tokens;

/// `timeline`/`timeline_pane` は裁定160 切片7(pane split survey §6 切片7)で
/// `motolii-timeline-pane` crate へ抽出済み — `src/timeline/` 9ファイル +
/// 対応する write ロジック(§1.2 の584行主部)を丸ごと移した。両エイリアスとも
/// 「型 alias で外部参照を壊さない」手口(既存の `crate::timeline::X`・
/// `crate::timeline_pane::X`・`motolii_shell::timeline_pane::X` 参照
/// (`screenshot.rs`・`stage.rs`・`tests/suite/*.rs`)は無改修で済む)。
pub use motolii_timeline_pane as timeline;
pub use motolii_timeline_pane as timeline_pane;

/// `settings_pane` は裁定160 切片9 で `motolii-settings-pane` crate へ抽出済み
/// (pane split survey §6 切片9)。`pub use timeline as timeline_pane;` と同じ
/// 「型 alias で外部参照を壊さない」手口 — 既存の `crate::settings_pane::X`・
/// `motolii_shell::settings_pane::X` 参照(`screenshot.rs`・`tests/suite/*.rs`)
/// は無改修で済む。write ロジック(`apply_background_preset`/
/// `commit_background_channel`/`commit_ui_scale`)もこの crate 側へ移設済み —
/// `Shell::update_settings` はそれらを呼ぶ glue だけを持つ(下記参照)。
pub use motolii_settings_pane as settings_pane;

/// `chrome`(pane 横断スタイルヘルパ、裁定160 切片5)は settings-pane crate 抽出
/// (切片9)時に settings_pane が4関数全部を必要としていたため一緒に移設した
/// (survey §2.4「必要最小の共有を crate 側へ移す」)。`inspector_pane.rs`・
/// このファイル自身の `crate::chrome::X` 参照はこの re-export だけで無改修の
/// まま通る — `motolii-shell` は assembler として `motolii-settings-pane` に
/// 依存する側なので、新しい循環にはならない(root → pane の一方向)。
pub(crate) use motolii_settings_pane::chrome;

/// `stage` は裁定160 切片10 で `motolii-stage-pane` crate へ抽出済み(pane
/// split survey §6 切片10)。`pub use timeline as timeline_pane;` と同じ
/// 「型 alias で外部参照を壊さない」手口 — 既存の `crate::stage::X` 参照
/// (`screenshot.rs`・`main.rs` のコメント・`tests/suite/*.rs`)は無改修の
/// まま通る。write ロジック(`observation_preview_source`)もこの crate 側へ
/// 移設済み — `Shell::observation_preview_source` はそれを呼ぶ glue だけを
/// 持つ(下記参照、関数名は無改名)。
pub use motolii_stage_pane as stage;

/// `browser_pane` は ζ 縫い目調査(`docs/reviews/2026-08-21-browser-seam-survey.md`)
/// +裁定162 切片 B0 で新規追加した骨格 crate(`motolii-browser-pane`、既存 pane の
/// 「型 alias で外部参照を壊さない」手口と同じ命名 — こちらは移設ではなく新規
/// なので壊す既存参照は無い)。B2(rail/filter)で `state::Message`/`PaneState`
/// が非空になり、**B3 で `Shell::view` へ配線した**(header の "Browser" トグル
/// (`self.header()`)+ `self.browser.is_open()` の間だけ木へ現れる、`view()`
/// 参照)。開閉フラグは `PaneState::is_open`(`browser_pane` crate 冒頭 doc の
/// 「B1/B2 からの委譲形を崩さない」設計選択)。
pub use motolii_browser_pane as browser_pane;

use file_dialogs::{FileDialogs, RfdDialogs};
use inspector_pane::{FieldDraft, TransformField};
use settings_pane::BackgroundFieldDraft;
use transport::{open_real_playback, Transport};

use tokens::{Colors, Dimensions, Tokens};

/// Stage 描画の計測。**debug のみ実測**(実機チラつき調査、2026-08-20)。
/// release は `metrics::*` が全部 no-op になる(呼び出し側はどちらも同じ形で呼べる)。
#[cfg(debug_assertions)]
pub mod metrics;
#[cfg(not(debug_assertions))]
pub mod metrics {
    //! release では計測しない。呼び出し側([`crate::Shell::refresh_frame`])は
    //! debug と同じ関数名を no-op として叩くだけで、cfg 分岐を呼び出し箇所へ
    //! 増やさずに済む。
    pub fn record_handle_creation(_bytes: usize) {}
    pub fn record_render_frame(_elapsed: std::time::Duration) {}
    pub fn record_tokens_reload() {}
    /// 裁定166: shader Pipeline が実際に `queue.write_texture` した時に呼ぶ
    /// (debug の実体は `metrics.rs` 参照)。
    pub fn record_presenter_upload(_bytes: usize) {}
    pub fn handle_creations() -> u64 {
        0
    }
    pub fn last_handle_bytes() -> usize {
        0
    }
    pub fn presenter_uploads() -> u64 {
        0
    }
    pub fn last_presenter_upload_bytes() -> usize {
        0
    }
    /// 裁定171 v2(M4): `record_presenter_upload` と同じ no-op 規律。
    pub fn record_presenter_blit() {}
    pub fn presenter_blits() -> u64 {
        0
    }
    pub fn render_frame_calls() -> u64 {
        0
    }
    pub fn render_frame_nanos() -> u64 {
        0
    }
    pub fn tokens_reloads() -> u64 {
        0
    }
    pub fn reset() {}
}

/// Stage 表示用に RGBA を縮める。**裁定166**: 旧 `stage_handle_rgba` の
/// 置き換え——旧実装は iced の `image::Handle::from_rgba` が同期アップロード
/// できる上限(`iced_wgpu-0.14.0/src/image/cache.rs::upload_raster`の
/// `MAX_SYNC_SIZE = 2MB`)を超えないよう `stage_auto_scale`(sqrt 自動縮小)を
/// 掛けていたが、Stage の絵を shader Program の永続テクスチャへ移したことで
/// その非同期アップロード境界(「その間 draw_image は何も描かない」穴、
/// `docs/reviews/2026-08-21-stage-presenter-decision.md` 事実2)自体が経路に
/// 存在しなくなったので、`stage_auto_scale` は撤去した(常に `1.0` を渡す =
/// フル解像度復帰)。
///
/// 残るのは **裁定163 Stage 下縁状態帯**が持つ `resolution_cap`(ユーザーが
/// 明示的に選ぶ上限、Auto/½/¼)だけ——[`stage::effective_preview_scale`] で
/// auto 側 `1.0` と min 合成する。`Auto` は cap=1.0固定なので合成しても
/// 値が変わらず、この関数は入力をそのまま返す(EXACT TARGET (b) 「presenter
/// へ渡る寸法 == comp 寸法」)。½/¼ が選ばれている時だけ実際に縮む
/// (nearest-neighbor — プレビュー用途なので品質は問わない、
/// `screenshot.rs::blit_letterboxed` と同じ考え方)。**画面には
/// `Length::Fill` で引き伸ばして出すので実素材解像度である必要が無い**
/// (screenshot 器具は `frame_rgba()` が返す元解像度の RGBA を別途持っている
/// — 縮めるのは presenter 用のコピーだけで、pixel 精度が要る経路には触らない)。
fn stage_presenter_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    resolution_cap: stage::PreviewResolutionCap,
) -> (u32, u32, Vec<u8>) {
    if width == 0 || height == 0 {
        return (width, height, rgba.to_vec());
    }
    let scale = stage::effective_preview_scale(1.0, resolution_cap);
    if scale >= 1.0 {
        return (width, height, rgba.to_vec());
    }

    let dst_w = ((width as f64 * scale).floor() as u32).max(1);
    let dst_h = ((height as f64 * scale).floor() as u32).max(1);

    let mut out = vec![0u8; (dst_w as usize) * (dst_h as usize) * 4];
    for dy in 0..dst_h {
        let sy = ((u64::from(dy) * u64::from(height)) / u64::from(dst_h)).min(u64::from(height) - 1)
            as u32;
        for dx in 0..dst_w {
            let sx = ((u64::from(dx) * u64::from(width)) / u64::from(dst_w))
                .min(u64::from(width) - 1) as u32;
            let si = ((sy * width + sx) * 4) as usize;
            let di = ((dy * dst_w + dx) * 4) as usize;
            out[di..di + 4].copy_from_slice(&rgba[si..si + 4]);
        }
    }
    (dst_w, dst_h, out)
}

/// 純関数レベルの試験(裁定166 ORACLE (b) — 「presenter へ渡る寸法 ==
/// comp 寸法」を GPU/Shell を一切介さずに確かめる)。`tests/suite/
/// render_pipeline_fence.rs` は同じ主張を `Shell::stage_presenter_dims()`
/// 経由で統合試験として重ねて見ている(二重の証拠、どちらか片方が偶然
/// 通っただけではないことを示す)。
#[cfg(test)]
mod stage_presenter_rgba_tests {
    use super::*;

    /// fixture と同じ 1920×1080。**現状(裁定166 前)は red**: 旧
    /// `stage_handle_rgba` は `stage_auto_scale` が sqrt 縮小を掛けるので
    /// 816×459 になっていた — この関数はもう `stage_auto_scale` を呼ばない。
    #[test]
    fn auto_cap_passes_native_resolution_through_unchanged() {
        let width = 1920u32;
        let height = 1080u32;
        let rgba = vec![0u8; (width as usize) * (height as usize) * 4];

        let (out_w, out_h, out_rgba) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Auto);

        assert_eq!((out_w, out_h), (width, height), "Auto なのに縮んでいる");
        assert_eq!(out_rgba.len(), rgba.len());
    }

    /// ½/¼ cap は「明示的な縮小」として維持する(EXACT TARGET 2)。
    #[test]
    fn half_and_quarter_caps_still_shrink_relative_to_native_resolution() {
        let width = 1920u32;
        let height = 1080u32;
        let rgba = vec![0u8; (width as usize) * (height as usize) * 4];

        let (half_w, half_h, _) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Half);
        assert!(half_w < width && half_h < height, "½ cap で縮んでいない");

        let (quarter_w, quarter_h, _) =
            stage_presenter_rgba(width, height, &rgba, stage::PreviewResolutionCap::Quarter);
        assert!(
            quarter_w < half_w && quarter_h < half_h,
            "¼ cap が ½ よりさらに縮んでいない"
        );
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Undo,
    Redo,
    ScrubTo(i64),
    Select(LayerId),
    AddLayer,
    /// OS から落ちてきた path。**受理も拒否もここ1箇所**で決める。
    ///
    /// 窓の event として直に受けず Message にしてあるのは、運転席が窓を開けずに
    /// 同じ道を通せるようにするため(旧 workspace の `window_input` widget と
    /// 同じ目的を、より少ないコードで満たす)。
    AdmitPaths(Vec<std::path::PathBuf>),
    /// 落下を1件ずつ溜める。winit は1ファイル1事象で送ってくるので、
    /// **そのまま処理すると3本落として3 undo になる**。
    DropReceived(std::path::PathBuf),
    /// 落下の区切り。次の描画要求が来た時点で、溜めた分を**まとめて1操作**にする。
    FlushDrops,
    /// トークンファイル(寸法・色)が変わった。**debug ビルドでしか実際には届かない**
    /// (裁定117)— release は [`tokens::watch_subscription`] が何も発行しない。
    TokensFileChanged,

    // ---- Inspector pane(第1波 + drag-to-scrub、裁定160 切片8で pane ローカル
    // Message へ集約) ----
    /// `motolii_inspector_pane::Message` を1本で畳む(iced 標準型 — 子 pane の
    /// `Message` を親が wrap する形、切片9の `Settings(settings_pane::Message)`
    /// と同じ)。腕ごとの doc は `inspector_pane::Message` 側を参照。
    Inspector(inspector_pane::Message),

    // ---- Timeline pane(裁定160 切片7で `motolii-timeline-pane` crate へ
    // 抽出、pane split survey §6 切片7) ----
    /// レーンバー M/S/L・クリップ move/trim・property 行(キー行)のキー選択・
    /// キー時刻ドラッグ/リタイム・NudgeKeyframe(旧 `LaneBarToggleMute/Solo/
    /// Lock`・`TimelineBarGrabbed`・`TimelineDragMoved/Released/Cancelled`・
    /// `TimelineKeySelect`・`TimelineDeleteSelectedKeys`・`TimelineKeyGrabbed`・
    /// `TimelineKeyDragMoved/Released/Cancelled`・`NudgeKeyframe`、14腕)を
    /// pane-local `Message` へ1回だけ畳む(pane crate 化のために構造上必須、
    /// survey §3.1)。`Shell::update` の `Message::Timeline` 腕で中身を見る
    /// (`timeline_pane::Message::Select`/`ScrubTo`/`ToggleMute`/`ToggleSolo`/
    /// `ToggleLock` の5腕は survey §3.2 exception 1 により Shell が先取りし、
    /// 残りは [`timeline_pane::PaneState::update`] へ委譲する)。
    Timeline(timeline_pane::Message),

    // ---- Timeline playhead ナビゲーション動詞束(U2、正典 §5・§8.1) ----
    /// Step Forward/Back(正典 §5「矢印キー」)。符号つき frame 数(素で ±1、
    /// Shift で ±10 — `NudgeKeyframe` と同じ「歩幅はキー解決側が決める」役割
    /// 分担)。選択も clip も動かさず playhead だけを動かす。
    StepPlayhead(i64),
    /// JumpToCompStart(正典 §8.1)。既定割当 Home。
    JumpPlayheadToStart,
    /// JumpToCompEnd(正典 §8.1)。既定割当 End。
    JumpPlayheadToEnd,
    /// JumpPrev/NextMeaningPoint(正典 §8.1)。既定割当 J(Prev)/K(Next)。
    /// `layer_only` は Shift 付き — 選択 layer 自身のキーだけに絞る(marker は
    /// comp 単位なので対象から外れる、`Shell::jump_meaning_point` 参照)。
    JumpMeaningPoint {
        direction: timeline::nav::JumpDirection,
        layer_only: bool,
    },
    /// JumpToClipIn/Out(正典 §8.1)。既定割当 I(In)/O(Out)。選択 layer の
    /// clip の In/Out へ — トリムではない(playhead だけが動く)。
    JumpClipEdge(timeline::nav::ClipEdge),

    // ---- cross-cutting(timeline drag と inspector drag 両方が読む、pane split
    // survey §1.3「core 残留が妥当」) ----
    /// Shift の押下状態。`CursorMoved` 自体は modifiers を運ばないので
    /// `ModifiersChanged` を別途追って持つ(drag 中の1/10微調整に使う)。
    KeyboardModifiersChanged(iced::keyboard::Modifiers),
    /// Esc — drag 中なら復元、typing 下書き中(値セル/名前欄)ならそれを破棄。
    EscapePressed,

    // ---- Settings パネル(タスク#18、裁定160 切片9で pane ローカル Message へ集約) ----
    /// `motolii_settings_pane::Message` を1本で畳む(iced 標準型 — 子 pane の
    /// `Message` を親が wrap する形)。腕ごとの doc は `settings_pane::Message`
    /// 側を参照。
    Settings(settings_pane::Message),

    // ---- Stage 観測カメラ(裁定157、裁定160 切片10で `motolii-stage-pane`
    // crate へ抽出、pane split survey §6 切片10) ----
    /// `motolii_stage_pane::Message` を1本で畳む(iced 標準の「子 pane の
    /// Message を親が wrap する」形、`Message::Settings`/`Message::Timeline`
    /// と同型)。腕ごとの doc は `stage::Message` 側を参照。
    Stage(stage::Message),

    // ---- Browser pane(ζ 縫い目調査+裁定162 切片 B0/B1/B2/B3) ----
    /// `motolii_browser_pane::Message` を1本で畳む(`Message::Settings`/
    /// `Message::Stage` と同型)。B2 で rail scope 選択/検索欄/Clear の3腕が、
    /// B3 で `ToggleBrowserPanel` が増えた — `Shell::update` は
    /// `self.browser.update(msg)` (`timeline_pane::PaneState::update` と
    /// 同型の委譲)へそのまま渡す(`ToggleBrowserPanel` も含め — Shell 側は
    /// per-variant 分岐を増やさない、`browser_pane::state` crate 冒頭 doc
    /// 参照)。**B3 で `Shell::view` へ配線した**(`self.browser.is_open()`
    /// の間だけ木へ現れる、`view()` 参照)。
    Browser(browser_pane::Message),

    // ---- shell の pane_grid 化(2026-08-22 実装レーン、`pane_layout.rs`
    // 冒頭 doc 参照) ----
    /// pane 本体がクリックされた。`iced::widget::pane_grid::PaneGrid::
    /// on_click` が発行する。**Q0 適合に必須**(`pane_layout::Layout::
    /// focused` フィールド doc 参照): fork rev 73e686e の pane_grid は
    /// `on_click`/`on_resize` の設定有無に関わらず、境界ドラッグ検出のため
    /// 自分の bounds 内の `ButtonPressed` を無条件に capture する——
    /// `on_click` を配線しないと「pane 本体のどこを押しても capture される
    /// のに Message が出ない」という Q0 違反を pane_grid 内の全域で起こす
    /// (実測: `tests/suite/q0_fence.rs` が155件検出)。フォーカス追跡は
    /// この capture に正直な意味を与える最小機能として採用した。
    PaneClicked(iced::widget::pane_grid::Pane),
    /// 境界ドラッグでリサイズ。`iced::widget::pane_grid::PaneGrid::on_resize`
    /// が発行する。`Shell::update` は `self.panes.apply_resize(event)` へ
    /// そのまま委譲する(pane_grid 自身の決定論に乗る、`pane_layout::
    /// Layout::apply_resize` doc 参照)。
    PaneResized(iced::widget::pane_grid::ResizeEvent),
    /// パネルのドラッグ並べ替え(ドッキング)。
    /// `iced::widget::pane_grid::PaneGrid::on_drag` が発行する。`Shell::
    /// update` は `self.panes.apply_drag(event)` へそのまま委譲する
    /// (`Dropped` だけが実際に State を動かす、`pane_layout::Layout::
    /// apply_drag` doc 参照)。
    PaneDragged(iced::widget::pane_grid::DragEvent),

    // ---- layer クリップボード(普通地図 消化第1波 U1、正典 §4) ----
    // キーは Cmd+C/V/X/D/A・Cmd+Shift+A(`resolve_navigation_key` へ配線済み、S0
    // 段差 群0・κ 台帳 FINDING 1)。**割当自体はまだ仮**(keymap 層は未実装、
    // `next/reference/timeline-grammar.md` 拘束6) — menu 入口(menubar)は
    // まだ無いので、UI 入口はキーだけの半消化状態(normal-map の menu 側判定は
    // 未着手のまま)。
    /// `Session::selection` の layer をアプリ内クリップボードへ写す(`clipboard.rs`
    /// doc 参照 — OS clipboard ではない)。**Document は触らない** — capture のみ
    /// なので undo に乗らない。
    CopyLayer,
    /// クリップボードの layer を新規 layer として配置する。**元時刻のまま**
    /// (playhead ペーストは今回作らない)。1 `apply_all` = 1 undo。配置後は
    /// 増えた方を選ぶ。
    PasteLayer,
    /// Copy + 削除。**1 undo**(`Intent::RemoveLayer` 1つだけを apply する —
    /// capture 自体は Document を触らないため)。locked な layer は理由つきで拒む
    /// (M13、`Intent::RemoveLayer` の `check_not_locked` をそのまま使う)。
    CutLayer,
    /// クリップボードを経由しないその場複製(Cmd+D)。1 `apply_all` = 1 undo。
    /// 複製後は増えた方を選ぶ(正典 §4)。
    DuplicateLayer,
    /// 見えている行を選択する(正典 §4「Cmd+A 正: 見えている行だけ」)。fold は
    /// まだ shell に無いので、今は present な全 layer が「見えている」。
    SelectAllLayers,
    /// 選択を全解除する(正典: 空白クリックと同義のキーボード入口)。
    DeselectAllLayers,

    // ---- G1 グループ化動詞(裁定174「意図優先の原則」) ----
    // キーは Cmd+G/Cmd+Shift+G(`resolve_navigation_key` へ配線済み)。parent
    // ポインタを直接編集する UI(H3、廃止)ではなく、「グループを作る/解除する」
    // という意図の動詞だけを露出する(`Document::group_layers`/
    // `ungroup_layers` doc 参照)。
    /// ⌘G。`Session::selected_layers` を新しい `LayerSource::Group` layer の
    /// 子へ束ねる。1 `apply_all` = 1 undo。空選択は no-op。成功したら Group
    /// 自身を選ぶ(AE 同型、裁定174 選択規則)。
    GroupLayers,
    /// ⌘⇧G。選択に含まれる `LayerSource::Group` layer を解除する(Group でない
    /// 選択は無視)。子の world 位置は保存される(`Document::ungroup_layers`
    /// の焼き込み)。解除後は旧子らを選ぶ(裁定174 選択規則)。
    UngroupLayers,

    // ---- freeze 意図動詞(裁定119、MB-2 で UI 初露出) ----
    // 旧 `ToggleEditMenu`/`ToggleFileMenu`(MB-0/MB-1 の表示専用 view flag)は
    // MB-2 で廃止 — menubar の開閉は widget 内部状態(`motolii_menubar::
    // menu_bar` doc「shell 側に Toggle 系 Message は要らない」)。
    /// Layer メニューの Freeze。選択中の `LayerSource::Group` layer の
    /// `frozen` を立てる(`Intent::Freeze` — 汎用 `SetAttrs` では触れない
    /// 専用口、`motolii_store::attrs::LayerAttrs::frozen` doc 参照)。Group
    /// でない選択は `UngroupLayers` と同じく黙って飛ばす。1 `apply_all` =
    /// 1 undo。凍結ゲート(locked 等)の拒否理由は既存 status 経路で出る。
    FreezeGroups,
    /// Layer メニューの Unfreeze。`FreezeGroups` と対称(`Intent::Unfreeze`)。
    UnfreezeGroups,
    /// New Project(Cmd+N・File メニュー、normal-map id 1221)。dirty なら
    /// [`file_dialogs::FileDialogs::confirm_discard`] を経由してから
    /// `Shell::reset_document` を呼ぶ ── dirty でなければ確認なしで即リセット。
    NewProjectRequested,
    /// Save As(Cmd+Shift+S・File メニュー、id 1225)。
    /// [`file_dialogs::FileDialogs::pick_save_path`] で path を選び、既存の
    /// 汎用 persist 経路(`Document::save`、履歴を畳んだ flattened 書き)で
    /// 書く。成功したら以後の `current_path` はこの path になる。
    SaveAsRequested,
    /// Save a Copy(File メニューのみ、id 1227 — normal-map の shortcut 出典が
    /// ゼロなので shortcut を発明しない)。path 選択は Save As と同じ入口だが
    /// **`current_path`/dirty 状態は据え置く**(「現 path 維持のまま別名へ
    /// 書く」── 別ファイルへの書き出しであって、開いているプロジェクトの
    /// 身分は変わらない)。
    SaveACopyRequested,
    /// Quit(Cmd+Q・File メニュー、id 1223)。dirty なら confirm_discard を
    /// 経由してからプロセスを終了する([`file_dialogs::FileDialogs::quit`])。
    QuitRequested,

    // ---- 実時間再生(A2、正典 §2 拘束5) ----
    /// Space。Play/Pause をトグルする。**ドラッグ中は無効**(拘束5「再生と
    /// 掴みは相互排他」)— 判断は `Shell::toggle_playback` 側(`is_dragging()`)
    /// が持つ、翻訳層(`resolve_navigation_key`)は常にこの Message を出す。
    TogglePlayback,
    /// 再生中だけ発行される tick(`subscription()` が `is_running()` の間だけ
    /// 束ねる)。`Session::playhead` を `PlaybackClock::position()` へ追随させ、
    /// comp 終端に達したら自動で Pause する(発注書 ORACLE (a)/(e))。
    PlaybackTick,
}

/// 裁定171 v2(M4)。GPU zero-copy 経路で使う resolve 済みスナップショット。
/// `motolii_store::Document` を直接共有できない(`re_entity_db::EntityDb` が
/// `testing` feature 外では `Clone` を持たない)ので、`Shell::build_preview_snapshot`
/// が `StoreView` から抜き出した**所有データ**をここへ積む——
/// `motolii_engine::Engine::render_resolved_to_texture` の入力そのもの。
#[derive(Clone, Debug)]
struct PreviewSnapshot {
    comp: CompSpec,
    background: [f32; 4],
    camera: ResolvedCamera,
    resolved: Vec<ResolvedLayer>,
}

/// Stage presenter shader へ渡す実体(裁定171 v2 M4)。
#[derive(Clone, Debug)]
enum PresenterSource {
    /// **高速路**(EXACT TARGET 1〜3)。`StagePresenterPipeline::prepare` が
    /// 世代ゲート越しに [`PreviewSnapshot`] を GPU 直接描画する——CPU
    /// readback をしない。
    Gpu(Arc<PreviewSnapshot>),
    /// **フォールバック**(裁定171 v2 §0-6: 市松 ON、または観測カメラ/½・¼
    /// resolution cap のように CPU 側で作った RGBA をそのまま見せたい場合)。
    /// 旧 `presenter_rgba: Arc<Vec<u8>>` と同じ形——`queue.write_texture`
    /// 経由で永続テクスチャへ上げる(裁定166 の経路、無改造で残す)。
    Cpu(Arc<Vec<u8>>),
}

/// 描き上がった1フレーム。**Document の写しではなく、描画の成果物**。
///
/// いつ捨てるかは [`Document::revision`] が決める(store 世代 + edit 位置)。
/// front が「前回の値」を自分で持たないための口がこれ。
struct RenderedFrame {
    /// `Document::display_revision()`(履歴 + transient overlay の世代の組)。
    /// **`revision()` ではない** — drag-to-scrub 中は overlay だけが動き、履歴の
    /// `revision()` は不変のままなので、`revision()` だけを見ていると drag 中の
    /// 再描画が起きない(transient overlay 化の要点そのもの)。
    revision: DisplayRevision,
    playhead: i64,
    width: u32,
    height: u32,
    /// Stage 表示用の実体(裁定171 v2 — 高速路/フォールバックの両対応、上記
    /// [`PresenterSource`] 参照)。**裁定166**: 旧 `handle: image::Handle` の
    /// 置き換え — shader Program の `Primitive`(`StagePresenterPrimitive`)が
    /// 毎フレーム `Arc::clone`/`clone()` するだけで、内容が変わらない限り
    /// 複製しない(`Program::draw` は描画のたびに呼ばれる、
    /// `iced_widget::shader::Program` doc 参照)。
    presenter_source: PresenterSource,
    presenter_width: u32,
    presenter_height: u32,
    /// `presenter_source` を新しく作り直した回数(単調増加)。shader Pipeline
    /// 側(`StagePresenterPipeline::upload`/`resolve`)が「前回描いた世代と
    /// 同じか」をこれで比較し、違う時だけ実際に描く/アップロードする
    /// (EXACT TARGET 1/2 の核心 — oracle (a) の直接の鍵)。
    presenter_generation: u64,
    /// `Engine::render_frame`(背景込み)の生 RGBA。**export/screenshot 真値専用**
    /// (`screenshot.rs`・`frame_rgba()`)— 通常描画(GPU 高速路)は一切読まない。
    /// **市松は絶対にここへ乗せない**し、市松トグルで一切変わらない
    /// (`settings_pane` doc「合成器が出せる」と「書き出しが吐く」は別問題、参照)。
    ///
    /// **裁定171 v2 EXACT TARGET 4**: GPU 高速路(`refresh_frame` の新しい早期
    /// return 枝)はこのフィールドを更新しない——古いままにしておき、
    /// [`rgba_stale`](RenderedFrame::rgba_stale)を立てる。`frame_rgba()` が
    /// 実際に呼ばれた時だけ [`Shell::ensure_rgba_fresh`] が追いつかせる
    /// (「readback は要求された時だけ」を型で保つ)。
    rgba: Vec<u8>,
    /// `rgba` が今の `playhead` を反映していない(GPU 高速路がここを飛ばした)
    /// ことを示す。`frame_rgba()`(screenshot 器具・試験専用)が呼ばれた時だけ
    /// [`Shell::ensure_rgba_fresh`] がこれを見て CPU readback を1回だけ行う。
    rgba_stale: bool,
    /// 市松 ON の間だけ `Some` — 裁定141「AE型の透明可視化モード」の入力
    /// (`Engine::render_frame_without_background`、背景 layer を省いた合成結果)。
    /// `presenter_rgba`(Stage 表示)と `screenshot.rs` は市松 ON の間、`rgba` の
    /// 代わりにこれへ [`settings_pane::composite_checkerboard`] を当てる。
    /// 市松 OFF の間は `None`(`rgba` をそのまま使う)。**export 真値(`rgba`)
    /// には一切影響しない** — 別フィールド。
    checkerboard_preview_rgba: Option<Vec<u8>>,
    /// `presenter_rgba` が市松込みで作られているか。**Document・playhead
    /// 非依存**の表示分岐なので、`revision()`/`playhead` が同じでもここが
    /// 変わっていれば `refresh_frame` は Document の再評価をせず presenter
    /// だけ作り直す(市松 ON の間は `checkerboard_preview_rgba` を取り直すため
    /// engine を1回追加で回すが、`Document`/`StoreView` の評価が増える
    /// わけではない)。
    checkerboard: bool,
    /// この `presenter_rgba` を作った時点の観測カメラ(裁定157)。
    /// `display_revision()`/`playhead`/`checkerboard` と同じ「キャッシュを
    /// 落とすかどうか」の鍵の一部 — `refresh_frame` の早期 return はこれも
    /// 比較する(`checkerboard` と同格の表示専用の鍵拡張)。
    observation: Option<ObservationCamera>,
    /// 観測カメラ有効時の Stage 表示 RGBA(`Engine::render_frame_with_view_camera`
    /// の結果そのもの)。**`rgba`(export 真値)とは別物** — `checkerboard_preview_rgba`
    /// と同じ「表示専用の複製」の形。`observation` が `None` の間は常に `None`。
    observation_rgba: Option<Vec<u8>>,
    /// この `presenter_rgba` を作った時点のプレビュー解像度 cap(裁定163 Stage
    /// 下縁状態帯)。**`checkerboard`/`observation` と同格の鍵拡張** —
    /// `stage_presenter_rgba` へ渡す実効スケールを変えるだけの表示専用の値
    /// なので、`revision()`/`playhead` が同じでもここが変わっていれば
    /// presenter だけ作り直す(Document・engine の再評価は増えない)。
    resolution_cap: stage::PreviewResolutionCap,
}

/// [`Shell::compute_display_source`] の戻り値。Stage 表示用の入力を1箇所へ
/// まとめただけの内部型 — `RenderedFrame` のフィールドへの書き戻しと
/// `build_stage_presenter_rgba` への引数の両方をこれ1つから作る(呼び出し側の
/// `refresh_frame` が2箇所(キャッシュヒット/フル再計算)で同じ分岐を書かずに
/// 済む)。
struct DisplaySource {
    /// `build_stage_presenter_rgba` へ渡す実体。`None` なら呼び出し側は
    /// `RenderedFrame::rgba`(export 真値)をそのまま使う(市松・観測カメラの
    /// どちらも効いていない既定の場合)。
    full_rgba: Option<Vec<u8>>,
    /// `full_rgba` を市松タイルで覆うかどうか(`build_stage_presenter_rgba` の
    /// 第4引数)。
    checkerboard: bool,
    /// `RenderedFrame::checkerboard_preview_rgba` へそのまま書き戻す値。
    checkerboard_preview_rgba: Option<Vec<u8>>,
    /// `RenderedFrame::observation_rgba` へそのまま書き戻す値。
    observation_rgba: Option<Vec<u8>>,
}

pub struct Shell {
    doc: Document,
    session: Session,
    engine: Engine,
    frame: Option<RenderedFrame>,
    /// 直近の拒否理由。**握り潰さない**(M13: 無反応ゼロ)。
    status: Option<String>,
    /// 区切りが来るまで溜めておく落下 path。
    pending_drops: Vec<std::path::PathBuf>,
    /// デザイン値(裁定117)。全 pane がここ経由で寸法・色を読む — raw 値の直書き禁止。
    tokens: Tokens,
    /// Inspector の Transform 行、編集中の下書き。**Document ではない** —
    /// `Message::Inspector(inspector_pane::Message::FieldSubmit)` が来るまで
    /// store に触らない(`pending_drops` と同じ「確定するまで front だけが
    /// 持つ一時状態」の形)。
    inspector_field_draft: Option<FieldDraft>,
    /// Inspector の Name 欄、編集中の下書き。同上。
    inspector_name_draft: Option<String>,
    /// Inspector の Speed 欄(ATTRS、SP1 第一波)、編集中の下書き。同上 —
    /// `LayerTiming.speed` は `TransformField`/track を経由しないので
    /// `inspector_field_draft` とは別の下書き(`inspector_name_draft` と同型)。
    inspector_speed_draft: Option<String>,
    /// Inspector 値セルの drag-to-scrub。**Document ではない** — 同上
    /// (`inspector_pane::FieldDragState` doc comment 参照。型定義は裁定160
    /// 切片8で `motolii-inspector-pane` crate へ移設済み、置き場(この
    /// フィールド自身)は移設していない)。
    inspector_drag: Option<inspector_pane::FieldDragState>,
    /// 直近の Shift 押下状態。`CursorMoved` は modifiers を運ばないので
    /// `ModifiersChanged` から別途追う(drag の1/10微調整に使う)。
    keyboard_modifiers: iced::keyboard::Modifiers,
    /// Timeline pane 専用の transient 状態(クリップ move/trim・キー時刻
    /// ドラッグ/リタイム、進行中の一時状態)。**Document ではない**
    /// (`inspector_drag` と同じ「pane 側の transient」の形)。裁定160 切片7で
    /// `motolii-timeline-pane` crate へ抽出済み — 旧 `timeline_drag`/
    /// `timeline_key_drag` の2フィールドは `timeline_pane::PaneState` 内へ
    /// まとまった(`PaneState` doc comment 参照)。
    timeline: timeline_pane::PaneState,

    // ---- Browser pane(裁定162 切片 B2/B3) ----
    /// rail scope + 検索欄 + パネル開閉(B3)の transient 状態
    /// (`browser_pane::state::PaneState` doc 参照)。**Document ではない** —
    /// `timeline` フィールドと同じ「pane 側の transient を1個の PaneState へ
    /// 集約する」形だが、Document/Session を触らないぶん更に薄い
    /// (`Message::Browser` の match 腕は `self.browser.update(msg)` だけで
    /// 完結する — `settings_panel_open`/`edit_menu_open` と違い、パネル開閉
    /// フラグもこの `PaneState` の内側にある)。
    browser: browser_pane::PaneState,

    // ---- shell の pane_grid 化(2026-08-22 実装レーン) ----
    /// リサイズ・ドッキングの layout 状態(`pane_layout.rs` 冒頭 doc 参照)。
    /// **Session 水準** — `browser`/`checkerboard`/`observation` と同格の
    /// 「意味を持たない純表示状態」、Document には乗らない。`browser.is_open()`
    /// との同期は `Message::Browser` の腕(`update()`)が
    /// `panes.set_browser_open(...)` を呼ぶことで保つ(2箇所の真実源に
    /// 見えるが、`browser_pane::PaneState::is_open` が唯一の真実源で、
    /// `panes` 側は常にそれへ追随するだけの写し——`browser_panel_open()`
    /// アクセサが `browser` を読むのと同じ非対称)。
    panes: pane_layout::Layout,

    // ---- Settings パネル(タスク#18) ----
    /// パネルの開閉。**表示だけの状態** — Document でも `Session`(選択・再生
    /// 位置)でもない。発注書は「Workspace 側」と指示しているが、Workspace 永続
    /// 機構がまだ無い(裁定127/128)ため、`tokens::Dimensions::ui_scale` の
    /// 「仮の置き場」と同じ理由でここに仮置きする。
    settings_panel_open: bool,
    // 旧 `edit_menu_open`/`file_menu_open`(MB-0/MB-1 の表示専用 view flag)は
    // MB-2 で廃止 — menubar の開閉は widget 内部状態(`menu.rs` 冒頭 doc)。
    /// Stage の下に市松を敷くか。**表示専用** — Document には一切乗らない
    /// (`settings_pane::composite_checkerboard` 参照、書き出しに影響しない)。
    checkerboard: bool,
    /// 背景 RGBA チャンネルの編集下書き。**Document ではない**
    /// (`inspector_field_draft` と同じ形 — Enter まで store に触らない)。
    background_draft: Option<BackgroundFieldDraft>,
    /// ui_scale(%)欄の編集下書き。同上。
    ui_scale_draft: Option<String>,

    // ---- Stage 観測カメラ(裁定157) ----
    /// 「自由に見る」ときの作業視点。**Document には乗らない** — `checkerboard`
    /// と同格の表示専用状態(`docs/reviews/2026-08-21-camera-seam-survey.md` §3
    /// の precedent どおり、`motolii_engine::ObservationCamera` の doc も参照)。
    /// `None` = 「カメラを通して見る」(既定 — レンダリングカメラの絵とバイト一致、
    /// `refresh_frame` が export 経路を一切汚さないことの直接の型的裏付け)。
    observation: Option<ObservationCamera>,
    /// Stage 下縁状態帯(裁定163 S 空間スコア)のプレビュー解像度 cap。
    /// **セッション状態**(Document・export 不変 — S 空間スコア文書「種別 a.
    /// 視界状態」、undo に乗らない・`checkerboard`/`observation` と同格)。
    /// 既定 `Auto`(予算導出のみ、cap を掛けない)。
    resolution_cap: stage::PreviewResolutionCap,

    // ---- layer クリップボード(普通地図 消化第1波 U1) ----
    /// アプリ内クリップボード(`clipboard.rs` doc 参照 — OS clipboard ではない)。
    /// **表示専用の front 状態** — Document には乗らない、`Session` とも別の身分
    /// (undo/redo に一切関わらない)。
    clipboard: clipboard::Clipboard,

    // ---- 実時間再生(A2、2026-08-21) ----
    /// 再生セッションの生死(`transport.rs` doc 参照)。**Document でも
    /// `Session` でもない** — undo に一切乗らない表示/デバイス専用の状態
    /// (`observation`/`clipboard` と同じ身分)。
    transport: Transport,

    // ---- File 束(MB-1、裁定176) ----
    /// OS 副作用の注入口(`file_dialogs.rs` 冒頭 doc 参照)。production は
    /// `Shell::new()` が [`RfdDialogs`] を渡す。test は
    /// `Shell::new_with_dialogs` へ缶詰応答の fake を渡す。
    dialogs: Box<dyn FileDialogs>,
    /// 直近の Save As が書いた path。**Save a Copy では更新しない**
    /// (`Message::SaveACopyRequested` doc 参照 — 「現 path 維持のまま別名へ
    /// 書く」)。New Project でリセットされる。
    current_path: Option<std::path::PathBuf>,
    /// 直近の保存(Save As)時点の `Document::revision()`。**dirty 判定の唯一の
    /// 鍵**(`Shell::is_dirty` 参照)── `revision()` は履歴の意味だけを表す
    /// (transient overlay は含まない、`document.rs::Revision` doc)ので、
    /// drag 中の途中経過だけで dirty が揺れることはない。
    saved_revision: Revision,
}

impl Shell {
    pub fn new() -> (Self, Task<Message>) {
        Self::new_with_dialogs(Box::new(RfdDialogs))
    }

    /// [`Shell::new`] の実体。[`file_dialogs::FileDialogs`] を注入できる形の
    /// 入口(`file_dialogs.rs` 冒頭 doc「dialog 呼び出しを注入可能な境界へ」)。
    /// production は `new()` がここへ [`RfdDialogs`] を渡すだけの薄い glue。
    /// test(`tests/suite/file_drive.rs`)はここへ缶詰応答の fake を渡す —
    /// `Shell::new()` の boot 関数ポインタとしての型(`fn() -> (Shell,
    /// Task<Message>)`、`main.rs` の `boot` 参照)を崩さずに済む分割。
    pub fn new_with_dialogs(dialogs: Box<dyn FileDialogs>) -> (Self, Task<Message>) {
        let mut doc = Self::default_document();
        // 既定値は「編集」ではないので戻せてはいけない。
        doc.mark_undo_floor();
        // 起動直後は「保存済み」扱い(未編集で Quit/New Project しても確認しない)
        // — `mark_undo_floor` は `floor` だけを動かし `revision()` には効かない
        // (`Document::mark_undo_floor` doc 参照)ので前後どちらで読んでも同じ値。
        let saved_revision = doc.revision();

        let engine = Engine::new().expect("GPU を用意できない");
        (
            Self {
                doc,
                session: Session::default(),
                engine,
                frame: None,
                status: None,
                pending_drops: Vec::new(),
                tokens: Tokens::load(),
                inspector_field_draft: None,
                inspector_name_draft: None,
                inspector_speed_draft: None,
                inspector_drag: None,
                keyboard_modifiers: iced::keyboard::Modifiers::default(),
                timeline: timeline_pane::PaneState::new(),
                browser: browser_pane::PaneState::new(),
                panes: pane_layout::Layout::new(),
                settings_panel_open: false,
                checkerboard: false,
                background_draft: None,
                ui_scale_draft: None,
                observation: None,
                resolution_cap: stage::PreviewResolutionCap::default(),
                clipboard: clipboard::Clipboard::default(),
                transport: Transport::new(),
                dialogs,
                current_path: None,
                saved_revision,
            },
            Task::none(),
        )
    }

    /// 既定 comp だけを持つ、空の Document を組む(`new_with_dialogs`/
    /// `reset_document`(New Project、MB-1)が共有する)。空の Document には
    /// comp が無く Stage が何も出せない(M17 違反)ので、起動直後・New Project
    /// 直後のどちらも既定の comp を置く。**undo floor はここでは立てない**
    /// (呼び手が `saved_revision` を確定させたい時点を制御できるように —
    /// `new_with_dialogs`/`reset_document` の doc 参照)。
    fn default_document() -> Document {
        let mut doc = Document::new();
        let _ = doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: motolii_store::Fps::try_new(30, 1).expect("30fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }));
        doc
    }

    /// `--fixture` 起動が使う口。**トンマナ検分の器具**(発注書)— `fixture::build()`
    /// が既存 Intent(`apply_all`)だけで組んだ Document を、通常の `new()` と同じ形で
    /// `Shell` へ包む。`update()` を経由しない点だけが `new()` と違う(初期状態の
    /// 組み立ては元々 `new()` も `doc.apply` を直に呼んでおり、同じ扱い)。
    pub fn new_fixture() -> (Self, Task<Message>) {
        let built = fixture::build();
        // 器具の Document は「未編集」扱い(起動直後と同格 — dirty ではない)。
        // `new_with_dialogs` の `saved_revision` と同じ考え方(doc 参照)。
        let saved_revision = built.doc.revision();
        let engine = Engine::new().expect("GPU を用意できない");
        let mut shell = Self {
            doc: built.doc,
            session: Session {
                playhead: built.playhead,
                selection: Some(built.selected),
                ..Session::default()
            },
            engine,
            frame: None,
            status: Some(built.status),
            pending_drops: Vec::new(),
            tokens: Tokens::load(),
            inspector_field_draft: None,
            inspector_name_draft: None,
            inspector_speed_draft: None,
            inspector_drag: None,
            keyboard_modifiers: iced::keyboard::Modifiers::default(),
            timeline: timeline_pane::PaneState::new(),
            browser: browser_pane::PaneState::new(),
            panes: pane_layout::Layout::new(),
            settings_panel_open: false,
            checkerboard: false,
            background_draft: None,
            ui_scale_draft: None,
            observation: None,
            resolution_cap: stage::PreviewResolutionCap::default(),
            clipboard: clipboard::Clipboard::default(),
            transport: Transport::new(),
            // 器具は screenshot 検分専用(発注書「トンマナ検分の器具」)なので
            // production の rfd ではなく`RfdDialogs` をそのまま渡しておく ──
            // 器具経路は `Message::NewProjectRequested` 等を一切発行しない
            // (`main.rs` の `--fixture` フラグ群を参照、File 束の Message は
            // 無い)ため実際に呼ばれることはない。
            dialogs: Box::new(RfdDialogs),
            current_path: None,
            saved_revision,
        };
        // `update()` を経由しないので、通常なら `update` の末尾が呼ぶ
        // `refresh_frame` をここで代わりに呼ぶ(Stage を空のまま起動しない、M17)。
        shell.refresh_frame();
        (shell, Task::none())
    }

    pub fn title(&self) -> String {
        "Motolii".to_owned()
    }

    /// 窓の事象 → Message。**ここは翻訳だけで、判断を持たない**。
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let window = iced::window::events().map(|(_id, event)| match event {
            iced::window::Event::FileDropped(path) => Message::DropReceived(path),
            // winit は1ファイル1事象で送るので、描画要求を落下の区切りにする。
            // 3本まとめて落として1操作になるのはこのため。
            _ => Message::FlushDrops,
        });
        // debug ビルドのみ実際に発行する(裁定117)。release は `Subscription::none()`。
        let tokens = tokens::watch_subscription().map(|()| Message::TokensFileChanged);
        // Inspector の drag-to-scrub 用。`mouse_area` は自分の bounds を出た
        // cursor を追えない(iced 0.14 に pointer capture が無い実測)ので、
        // move/release/Escape の主経路を window 全体からここで拾う
        // (`inspector_pointer_event` — 翻訳だけで、drag 中かどうかの判断は
        // `Shell::update` 側 = `inspector_drag` の有無)。
        let pointer = iced::event::listen_with(inspector_pointer_event);
        // 実時間再生(A2): 再生中だけtickを束ねる — Pause中はSubscriptionから
        // 落ちる。裁定166: tickは`iced::window::frames()`(vsync由来)へ
        // 置き換え済みで、OSスレッドのsleepは無い(`transport.rs`のdoc参照)。
        let ticks = if self.transport.is_running() {
            transport::tick_subscription().map(|()| Message::PlaybackTick)
        } else {
            iced::Subscription::none()
        };
        iced::Subscription::batch([window, tokens, pointer, ticks])
    }

    /// **唯一の書き口**。ここ以外に `doc.apply` を呼ぶ場所を作らない。
    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.status = None;
        // 旧 MB-0/MB-1 の自動クローズ規律(edit_menu_open/file_menu_open)は
        // MB-2 で不要になった — menubar の開閉・項目クリック後の自動クローズは
        // widget 内部(vendored iced_aw menu の `close_on_item_click` 既定)。
        // click→type 編集への切り替え(`finish_field_drag`)だけがフォーカス
        // task を返す。他の枝は既定どおり `Task::none()`。
        let mut task = Task::none();
        match message {
            Message::Undo => {
                if !self.doc.undo() {
                    self.status = Some("これ以上戻せない".to_owned());
                }
            }
            Message::Redo => {
                if !self.doc.redo() {
                    self.status = Some("これ以上進めない".to_owned());
                }
            }
            Message::ScrubTo(frame) => self.scrub_to(frame),
            Message::Select(layer) => self.select_single(layer),
            Message::AdmitPaths(paths) => self.admit(paths),
            Message::DropReceived(path) => self.pending_drops.push(path),
            Message::FlushDrops => {
                if !self.pending_drops.is_empty() {
                    let paths = std::mem::take(&mut self.pending_drops);
                    self.admit(paths);
                }
            }
            Message::TokensFileChanged => {
                self.tokens = Tokens::load();
                metrics::record_tokens_reload();
            }
            Message::Inspector(msg) => {
                task = self.update_inspector(msg);
            }
            // pane split survey §3.2 exception 1/裁定160 切片7: `Select`/
            // `ScrubTo` は本来 core 腕、`ToggleMute`/`ToggleSolo`/`ToggleLock`
            // は `toggle_layer_hidden` が Inspector とも共有する Shell 側の
            // ヘルパーのため、この5腕だけ `timeline_pane::PaneState::update`
            // へ渡す前に Shell が先取りする(`timeline_pane::write` モジュール
            // doc 参照)。残りは pane 側の唯一の書き口(`PaneState::update`)へ
            // 委譲する — 拒否理由があれば `self.status` へそのまま渡す。
            Message::Timeline(msg) => match msg {
                timeline_pane::Message::Select(layer) => self.select_single(layer),
                timeline_pane::Message::ScrubTo(frame) => self.scrub_to(frame),
                timeline_pane::Message::ToggleMute(layer) => self.toggle_layer_hidden(layer),
                timeline_pane::Message::ToggleSolo(layer) => self.toggle_layer_solo(layer),
                timeline_pane::Message::ToggleLock(layer) => self.toggle_layer_lock(layer),
                // transport 帯(裁定180)— 意味は shell の既存腕そのもの(5例外と
                // 同じ先取りの型。pane 側 `PaneState::update` は no-op)。
                timeline_pane::Message::TogglePlayback => self.toggle_playback(),
                timeline_pane::Message::StepPlayhead(delta) => self.step_playhead(delta),
                timeline_pane::Message::JumpPlayheadToStart => self.session.playhead = 0,
                timeline_pane::Message::JumpPlayheadToEnd => {
                    let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
                    self.session.playhead = timeline::nav::comp_end_frame(duration);
                }
                other => {
                    if let Some(reason) =
                        self.timeline.update(other, &mut self.doc, &mut self.session, self.keyboard_modifiers)
                    {
                        self.status = Some(reason);
                    }
                }
            },
            Message::StepPlayhead(delta) => self.step_playhead(delta),
            Message::JumpPlayheadToStart => self.session.playhead = 0,
            Message::JumpPlayheadToEnd => {
                let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
                self.session.playhead = timeline::nav::comp_end_frame(duration);
            }
            Message::JumpMeaningPoint { direction, layer_only } => {
                self.jump_meaning_point(direction, layer_only);
            }
            Message::JumpClipEdge(edge) => self.jump_clip_edge(edge),
            Message::KeyboardModifiersChanged(modifiers) => self.keyboard_modifiers = modifiers,
            // Esc は Timeline ドラッグを優先してキャンセルする(clip → key の順、
            // どちらも掴んでいなければ Inspector 側(drag/typing 下書き)を試す
            // — 同時に成立するのは片方だけなので順序自体に意味は無い、排他)。
            Message::EscapePressed => {
                if !self.timeline.cancel_drag() && !self.timeline.cancel_key_drag() {
                    self.cancel_inspector_interaction();
                }
            }
            Message::Settings(msg) => self.update_settings(msg),
            Message::Stage(msg) => self.update_stage(msg),
            // B2/B3: rail scope 選択/検索欄/Clear/ToggleBrowserPanel の4腕
            // (`browser_pane::Message`)を pane 側の唯一の書き口
            // (`PaneState::update`)へそのまま委譲する(`timeline_pane::
            // PaneState::update` への委譲と同型)。Document/Session を一切
            // 触らない pane-local 状態なので `&mut self.browser` だけで完結
            // する(引数を追加で貸す必要が無い、`browser_pane::state` crate
            // doc 参照)。
            Message::Browser(msg) => {
                self.browser.update(msg);
                // pane_grid 側は `browser_pane::PaneState::is_open()` が唯一の
                // 真実源(`panes` フィールド doc 参照)——ここで追随させる。
                // `set_browser_open` は同値なら no-op(`pane_layout::Layout`
                // doc)なので、`ToggleBrowserPanel` 以外の3腕(rail/検索欄)で
                // 毎回呼んでも他 split の ratio・ドラッグ配置を潰さない。
                self.panes.set_browser_open(self.browser.is_open());
            }
            Message::PaneClicked(pane) => self.panes.set_focused(pane),
            Message::PaneResized(event) => self.panes.apply_resize(event),
            Message::PaneDragged(event) => self.panes.apply_drag(event),
            Message::AddLayer => {
                let id = LayerId(self.next_layer_id());
                // **1操作 = 1 undo**。`AddLayer`/`SetMeta`/`SetAttrs`(差し色の
                // 自動割当)を別々に書くと利用者は Undo を複数回押すことになる
                // (ui-quality-bar Q2)。
                let placed = self.doc.apply_all([
                    Intent::AddLayer(id),
                    Intent::SetMeta {
                        layer: id,
                        meta: LayerMeta {
                            source: LayerSource::Solid {
                                rgba: [80, 160, 220, 255],
                                width: 240,
                                height: 135,
                            },
                            order: id.0 as i16,
                            // 尺の決め方は Document が持つ(M4)。
                            timing: LayerTiming::place(
                                self.session.playhead,
                                None,
                                self.comp_duration(),
                            ),
                        },
                    },
                    Intent::SetAttrs {
                        layer: id,
                        patch: LayerAttrsPatch {
                            label_color: Some(Some(Self::label_color_for_new_layer(id))),
                            ..Default::default()
                        },
                    },
                ]);
                match placed {
                    Ok(()) => self.select_single(id),
                    // 拒否は必ず出す。黙って消さない。
                    Err(error) => self.status = Some(format!("layer を置けない: {error}")),
                }
            }
            Message::CopyLayer => self.copy_layer(),
            Message::PasteLayer => self.paste_layer(),
            Message::CutLayer => self.cut_layer(),
            Message::DuplicateLayer => self.duplicate_layer(),
            Message::SelectAllLayers => self.select_all_layers(),
            Message::DeselectAllLayers => self.deselect_all_layers(),
            Message::GroupLayers => self.group_selected_layers(),
            Message::UngroupLayers => self.ungroup_selected_layers(),
            // MB-2: freeze 意図動詞(裁定119)の UI 初露出(Layer メニュー)。
            Message::FreezeGroups => self.set_selected_groups_frozen(true),
            Message::UnfreezeGroups => self.set_selected_groups_frozen(false),
            Message::NewProjectRequested => {
                if self.confirm_discard_if_dirty() {
                    self.reset_document();
                }
            }
            Message::SaveAsRequested => self.perform_save_as(),
            Message::SaveACopyRequested => self.perform_save_a_copy(),
            Message::QuitRequested => {
                if self.confirm_discard_if_dirty() {
                    self.dialogs.quit();
                }
            }
            Message::TogglePlayback => self.toggle_playback(),
            Message::PlaybackTick => self.advance_playback_tick(),
        }
        self.refresh_frame();
        task
    }

    /// 単一 layer を選ぶ(既存の `Session::selection` に加え、`selected_layers` も
    /// 単一集合へ揃える — Select All(複数選択)から普通のクリックへ戻る時に
    /// 古い複数選択が居座る事故を防ぐ)。
    fn select_single(&mut self, layer: LayerId) {
        self.session.selection = Some(layer);
        self.session.selected_layers = vec![layer];
    }

    // ---- File 束(MB-1、裁定176) ----

    /// 未保存の変更があるか。**`saved_revision` フィールドの doc が唯一の
    /// 判定根拠** — `Document::revision()`(履歴のみ、transient overlay は
    /// 含まない)を最後に保存した時点の値と比べるだけ。
    fn is_dirty(&self) -> bool {
        self.doc.revision() != self.saved_revision
    }

    /// New Project/Quit の dirty ガード。dirty でなければ確認そのものを
    /// 出さない(不要な dialog を挟まない)。true = 続行してよい。
    fn confirm_discard_if_dirty(&self) -> bool {
        !self.is_dirty() || self.dialogs.confirm_discard()
    }

    /// New Project(id 1221)本体。**Document を丸ごと差し替える**
    /// (`default_document` — 起動直後と同じ既定 comp)。`current_path`/
    /// `saved_revision` も新しい Document 基準へ揃えるので、直後は dirty では
    /// ない。`Session` も既定へ戻す(古い selection が存在しない layer を指す
    /// 事故を避ける — playhead/selection は前の project の物なので引き継がない)。
    fn reset_document(&mut self) {
        let mut doc = Self::default_document();
        doc.mark_undo_floor();
        self.saved_revision = doc.revision();
        self.doc = doc;
        self.current_path = None;
        self.session = Session::default();
    }

    /// Save As(id 1225)。path 選択→保存(既存の汎用 persist 経路、
    /// `Document::save` = `flattened()` で履歴を畳んでから書く、`persist.rs`
    /// doc 参照)→成功したら `current_path`/`saved_revision` を更新して dirty を
    /// 解消する。キャンセル・書き込み失敗のどちらも `current_path` は不変。
    fn perform_save_as(&mut self) {
        let Some(path) = self.dialogs.pick_save_path() else {
            return;
        };
        match self.doc.save(&path) {
            Ok(()) => {
                self.current_path = Some(path);
                self.saved_revision = self.doc.revision();
            }
            // 拒否は必ず出す。黙って消さない(M13 と同じ規律)。
            Err(error) => self.status = Some(format!("保存できない: {error}")),
        }
    }

    /// Save a Copy(id 1227)。Save As と同じ path 選択・同じ persist 経路だが、
    /// **`current_path`/`saved_revision` は据え置く**(`Message::
    /// SaveACopyRequested` doc「現 path 維持のまま別名へ書く」)——開いている
    /// project の身分(どの path と紐付いているか・dirty かどうか)は変わらない。
    fn perform_save_a_copy(&mut self) {
        let Some(path) = self.dialogs.pick_save_path() else {
            return;
        };
        if let Err(error) = self.doc.save(&path) {
            self.status = Some(format!("コピーを保存できない: {error}"));
        }
    }

    // ---- layer クリップボード(普通地図 消化第1波 U1、正典 §4) ----

    /// Copy。**Document は触らない**(capture のみ)ので undo に一切乗らない。
    fn copy_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("コピーする layer が選ばれていない".to_owned());
            return;
        };
        match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => self.clipboard.set(snapshot),
            Err(error) => self.status = Some(format!("layer をコピーできない: {error}")),
        }
    }

    /// Paste。**元時刻のまま**(playhead ペーストは今回作らない)。
    /// `LayerSnapshot::instantiate` が組む intent 列を1回の `apply_all` で書くので
    /// 1操作 = 1 undo。配置後は増えた方を選ぶ(正典 §4)。
    fn paste_layer(&mut self) {
        let Some(snapshot) = self.clipboard.get().cloned() else {
            self.status = Some("クリップボードが空".to_owned());
            return;
        };
        let new_id = LayerId(self.next_layer_id());
        match self.doc.apply_all(snapshot.instantiate(new_id)) {
            Ok(()) => self.select_single(new_id),
            Err(error) => self.status = Some(format!("layer を貼り付けられない: {error}")),
        }
    }

    /// Cut = Copy + 削除。**削除は `Intent::RemoveLayer` 1回だけ**(capture 自体は
    /// Document を触らないので、apply 1回 = 1 undo)。locked な layer は
    /// `Intent::RemoveLayer` の `check_not_locked` が理由つきで拒む(M13) —
    /// 拒否された時はクリップボードも書き換えない(コピーだけ成立してしまう
    /// 中途半端を作らない)。
    fn cut_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("切り取る layer が選ばれていない".to_owned());
            return;
        };
        let snapshot = match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.status = Some(format!("layer をコピーできない: {error}"));
                return;
            }
        };
        match self.doc.apply(Intent::RemoveLayer(layer)) {
            Ok(()) => {
                self.clipboard.set(snapshot);
                if self.session.selection == Some(layer) {
                    self.session.selection = None;
                }
                self.session.selected_layers.retain(|&id| id != layer);
            }
            Err(error) => self.status = Some(format!("layer を切り取れない: {error}")),
        }
    }

    /// Duplicate(Cmd+D)。**クリップボードを経由しないその場複製** — capture と
    /// instantiate は clipboard.rs の同じ形を使い回すが、`self.clipboard` へは
    /// 一切触らない(Copy の中身を上書きしない)。1 `apply_all` = 1 undo。
    /// 複製後は増えた方を選ぶ(正典 §4)。
    fn duplicate_layer(&mut self) {
        let Some(layer) = self.session.selection else {
            self.status = Some("複製する layer が選ばれていない".to_owned());
            return;
        };
        let snapshot = match clipboard::LayerSnapshot::capture(&self.doc.view(), layer) {
            Ok(snapshot) => snapshot,
            Err(error) => {
                self.status = Some(format!("layer を複製できない: {error}"));
                return;
            }
        };
        let new_id = LayerId(self.next_layer_id());
        match self.doc.apply_all(snapshot.instantiate(new_id)) {
            Ok(()) => self.select_single(new_id),
            Err(error) => self.status = Some(format!("layer を複製できない: {error}")),
        }
    }

    /// Select All(正典 §4「Cmd+A 正: 見えている行だけ」)。fold はまだ shell に
    /// 無いので、今は present な全 layer が「見えている」全部(`clipboard::select_all`
    /// doc 参照)。複数選択に入るので単一 focus(`selection`)は持たない。
    fn select_all_layers(&mut self) {
        let visible = self.doc.view().layers();
        self.session.selected_layers = clipboard::select_all(&visible);
        self.session.selection = None;
    }

    /// Deselect All(正典: 空白クリックと同義のキーボード入口)。単一 focus・
    /// 複数選択の両方を解除する。
    fn deselect_all_layers(&mut self) {
        self.session.selection = None;
        self.session.selected_layers.clear();
    }

    // ---- G1 グループ化動詞(裁定174) ----

    /// ⌘G。`selected_layers` は `select_single`/`select_all_layers` が常に
    /// `selection` と同期させているので(単一選択でも `[layer]` が入っている)、
    /// ここ1本を選択の正本として読めばよい。空選択は no-op(status も出さない
    /// — 動詞が意味を持たない状態なので「拒否」ではない)。成功したら
    /// Group 自身を選ぶ(AE 同型、裁定174 選択規則)。
    fn group_selected_layers(&mut self) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        match self.doc.group_layers(&self.session.selected_layers.clone()) {
            Ok(Some(group)) => self.select_single(group),
            Ok(None) => {}
            // 拒否は必ず出す(M13: 無反応ゼロ) — locked な layer が選択に
            // 混じっていた場合、`Document::group_layers` の `Intent::SetAttrs`
            // 柵がバッチ全体を `Err` にする。
            Err(error) => self.status = Some(format!("layer をグループ化できない: {error}")),
        }
    }

    /// ⌘⇧G。選択に含まれる `LayerSource::Group` layer だけを解除する(Group
    /// でない選択は `Document::ungroup_layers` が黙って飛ばす)。解除後は
    /// 旧子らを選ぶ(裁定174 選択規則) — 1層だけなら `select_single` と同型、
    /// 複数なら Select All と同型(単一 focus は持たない)。
    fn ungroup_selected_layers(&mut self) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        match self.doc.ungroup_layers(&self.session.selected_layers.clone()) {
            Ok(children) if children.is_empty() => {}
            Ok(children) if children.len() == 1 => self.select_single(children[0]),
            Ok(children) => {
                self.session.selected_layers = children;
                self.session.selection = None;
            }
            Err(error) => self.status = Some(format!("グループを解除できない: {error}")),
        }
    }

    /// Freeze/Unfreeze(裁定119 の意図動詞、MB-2 で UI 初露出)。選択に含まれる
    /// `LayerSource::Group` layer だけを対象にする(Group でない選択は
    /// `ungroup_selected_layers` と同じく黙って飛ばす — store 側の
    /// `freeze_attrs_batch` は非 Group を `Err` にするため、ここで先に絞る)。
    /// 1 `apply_all` = 1 undo(Q2)。選択は動かさない(層構造が変わらない)。
    /// 凍結ゲートの拒否(locked な Group 等)は既存 status 経路で理由つきで出す
    /// (M13: 無反応ゼロ)。
    fn set_selected_groups_frozen(&mut self, frozen: bool) {
        if self.session.selected_layers.is_empty() {
            return;
        }
        let groups: Vec<LayerId> = {
            let view = self.doc.view();
            self.session
                .selected_layers
                .iter()
                .copied()
                .filter(|&layer| {
                    view.meta(layer)
                        .ok()
                        .flatten()
                        .is_some_and(|meta| meta.source == LayerSource::Group)
                })
                .collect()
        };
        if groups.is_empty() {
            return;
        }
        let intents = groups.into_iter().map(|group| {
            if frozen {
                Intent::Freeze { group }
            } else {
                Intent::Unfreeze { group }
            }
        });
        if let Err(error) = self.doc.apply_all(intents) {
            let verb = if frozen { "凍結できない" } else { "解凍できない" };
            self.status = Some(format!("グループを{verb}: {error}"));
        }
    }

    /// 落ちてきた path を素材として受ける。
    ///
    /// **開けない物は理由つきで飛ばす**(M2)。黙って消すと利用者は
    /// 「落としたのに何も起きない」としか分からない。
    ///
    /// 裁定162(B1、bin-first の下地): 各 path は**まず台帳へ記帳**
    /// (`Intent::AdmitAsset`)し、その上で従来どおり layer として配置する。
    /// 記帳と配置は別の関心事 — 記帳は「fingerprint が計算できたか」だけを見て
    /// 判定し、配置できるかどうか(`motolii_media::probe` が成功するか)を
    /// 問わない。junk file(probe が失敗する物)でも fingerprint さえ読めれば
    /// 台帳には載る(bin-first: 取り込みと配置は別の判断)。同一ファイルの
    /// 再 drop は `AssetTable::admit` の content_hash 重複統合にそのまま乗る
    /// (shell 側で先回りの dedupe はしない、EXACT TARGET #3)。
    fn admit(&mut self, paths: Vec<std::path::PathBuf>) {
        let mut intents = Vec::new();
        let mut rejected = Vec::new();
        let mut admission_skipped = Vec::new();
        let mut next = self.next_layer_id();

        let comp_duration = self.comp_duration();
        let start = self.session.playhead;
        let _ = start;

        for path in paths {
            let text = path.to_string_lossy().into_owned();
            let file_name = || {
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| text.clone())
            };

            // **記帳**(台帳、裁定162)。fingerprint 計算(ファイル IO)が失敗
            // したら記帳だけスキップする — 配置(下の probe)は独立に続行する。
            match Self::fingerprint_source(&path) {
                Ok(fingerprint) => {
                    let draft = AssetDraft::from_probed_source(
                        Self::guess_asset_type(&path),
                        &fingerprint,
                        &path,
                        None,
                    );
                    intents.push(Intent::AdmitAsset { draft });
                }
                Err(error) => {
                    admission_skipped.push(format!("{}: {error}", file_name()));
                }
            }

            // **配置**(従来どおり)。
            match motolii_media::probe(&path) {
                Ok(info) => {
                    let id = LayerId(next);
                    next += 1;
                    intents.push(Intent::AddLayer(id));
                    intents.push(Intent::SetMeta {
                        layer: id,
                        meta: LayerMeta {
                            source: LayerSource::Media {
                                path: text,
                                fingerprint: None,
                            },
                            order: id.0 as i16,
                            timing: LayerTiming::place(
                                self.session.playhead,
                                info.nb_frames,
                                comp_duration,
                            ),
                        },
                    });
                    // 差し色の自動割当(`Message::AddLayer` と同じ決定論、
                    // `label_color_for_new_layer` 参照)。
                    intents.push(Intent::SetAttrs {
                        layer: id,
                        patch: LayerAttrsPatch {
                            label_color: Some(Some(Self::label_color_for_new_layer(id))),
                            ..Default::default()
                        },
                    });
                }
                Err(error) => {
                    rejected.push(format!("{}: {error}", file_name()));
                }
            }
        }

        // 落とした分は**まとめて1 undo**(1操作 = 1 undo)。台帳記帳(AdmitAsset)も
        // 同じ batch に同居させる — 呼び手(`Message::AdmitPaths`/`FlushDrops`)が
        // 渡した path 列ぜんぶで1 undo という既存の粒をそのまま保つ(1 path = 1
        // undo ではない、`admit` の doc 冒頭参照)。
        if !intents.is_empty() {
            if let Err(error) = self.doc.apply_all(intents) {
                rejected.push(format!("置けなかった: {error}"));
            }
        }
        let mut notices = Vec::new();
        if !rejected.is_empty() {
            notices.push(format!(
                "受け取れない素材 {}件 — {}",
                rejected.len(),
                rejected.join(" / ")
            ));
        }
        if !admission_skipped.is_empty() {
            notices.push(format!(
                "台帳への記帳をスキップ {}件 — {}",
                admission_skipped.len(),
                admission_skipped.join(" / ")
            ));
        }
        if !notices.is_empty() {
            self.status = Some(notices.join(" / "));
        }
    }

    /// `Intent::AdmitAsset` の draft を組むための fingerprint 計算(ファイル IO)。
    /// `motolii_media::probe`(ffprobe サイドカー)とは独立 — 記帳は「読めるか」
    /// だけを見る(EXACT TARGET #2)。
    fn fingerprint_source(
        path: &std::path::Path,
    ) -> Result<SourceFingerprintV1, motolii_store::SourceFingerprintError> {
        let file = std::fs::File::open(path)?;
        SourceFingerprintV1::from_reader(file)
    }

    /// 台帳の `asset_type`(opaque 文字列)を拡張子から粗く推定する。**種別判定の
    /// 精度はこの切片(B1)の非目標** — rail/filter(B2)以降が正確な種別判定
    /// (意味起草タスク#14 の空席)を持つまでの暫定値。
    fn guess_asset_type(path: &std::path::Path) -> String {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        match ext.as_str() {
            "mp4" | "mov" | "mkv" | "webm" | "avi" | "m4v" => format!("video/{ext}"),
            "jpg" | "jpeg" => "image/jpeg".to_owned(),
            "png" | "gif" | "webp" | "bmp" | "svg" => format!("image/{ext}"),
            "wav" | "mp3" | "aac" | "flac" | "ogg" | "m4a" => format!("audio/{ext}"),
            "" => "application/octet-stream".to_owned(),
            other => format!("application/{other}"),
        }
    }

    /// 今の playhead を comp の fps で時刻へ写す。comp が無い/fps が壊れているなら
    /// `None`(M16: panic しない)。
    fn time_at_playhead(&self) -> Option<RationalTime> {
        let composition = self.doc.view().composition().ok().flatten()?;
        RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()
    }

    /// [`Message::Inspector`] の唯一の分配口。腕ごとの意味は
    /// `inspector_pane::Message` 側の doc を参照。書き込み本体は
    /// `motolii-inspector-pane` crate 側の自由関数([`inspector_pane::
    /// commit_inspector_field`] 等)が持ち、ここは `self.doc`/`self.session`/
    /// 下書きフィールドをそのまま貸す glue だけ(裁定160 切片8、
    /// `update_settings` と同じ形)。
    fn update_inspector(&mut self, message: inspector_pane::Message) -> Task<Message> {
        match message {
            inspector_pane::Message::FieldInput(field, text) => {
                self.inspector_field_draft = Some(FieldDraft { field, text });
                Task::none()
            }
            inspector_pane::Message::FieldSubmit(field) => {
                self.commit_inspector_field(field);
                Task::none()
            }
            inspector_pane::Message::NameInput(text) => {
                self.inspector_name_draft = Some(text);
                Task::none()
            }
            inspector_pane::Message::NameSubmit => {
                self.commit_inspector_name();
                Task::none()
            }
            inspector_pane::Message::ToggleHidden => {
                self.toggle_inspector_hidden();
                Task::none()
            }
            inspector_pane::Message::CycleBlendMode => {
                self.cycle_inspector_blend_mode();
                Task::none()
            }
            inspector_pane::Message::ValuePressed(field) => {
                self.start_field_drag(field);
                Task::none()
            }
            inspector_pane::Message::PointerMoved(point) => {
                self.continue_field_drag(point);
                Task::none()
            }
            inspector_pane::Message::PointerReleased => self.finish_field_drag(),
            inspector_pane::Message::SpeedInput(text) => {
                self.inspector_speed_draft = Some(text);
                Task::none()
            }
            inspector_pane::Message::SpeedSubmit => {
                self.commit_inspector_speed();
                Task::none()
            }
            inspector_pane::Message::ResetSpeed => {
                self.reset_inspector_speed();
                Task::none()
            }
            inspector_pane::Message::KeyPressed(row) => {
                self.toggle_inspector_key(row);
                Task::none()
            }
        }
    }

    /// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
    /// (1 gesture = 1 undo)。書き込み本体は [`inspector_pane::
    /// commit_inspector_field`](自由関数、`&mut self.doc`/`&mut self.
    /// inspector_field_draft`/`self.session.selection` をそのまま貸す)——
    /// ここは `Err` を status 帯へ渡す glue だけ(M13)。
    fn commit_inspector_field(&mut self, field: TransformField) {
        // comp が無い(= layer も選択も無い)なら下書きを捨てるだけ —
        // 旧実装(選択なしで draft を消費して no-op)と同じ安全側。
        let Ok(Some(composition)) = self.doc.view().composition() else {
            self.inspector_field_draft = None;
            return;
        };
        if let Err(error) = inspector_pane::commit_inspector_field(
            &mut self.doc,
            &mut self.inspector_field_draft,
            self.session.selection,
            self.session.playhead,
            composition.fps,
            field,
        ) {
            self.status = Some(error);
        }
    }

    /// Attrs の Name 欄 — 下書きを確定して1回の `Intent::SetAttrs` を出す。
    /// [`commit_inspector_field`](上記)と同じ glue の形。
    fn commit_inspector_name(&mut self) {
        if let Err(error) = inspector_pane::commit_inspector_name(
            &mut self.doc,
            &mut self.inspector_name_draft,
            self.session.selection,
        ) {
            self.status = Some(error);
        }
    }

    /// Key セル click(K1)— 即1回の `Intent::SetTrack` を出す(下書きを経由
    /// しない、[`toggle_inspector_hidden`] と同じ即時操作の形 — 1 click = 1
    /// undo)。3状態の意味と新 track の組み立ては純関数
    /// [`inspector_pane::toggled_key_track`] が持ち、ここは playhead・track・
    /// 現在の評価値を貸して `Err` を status 帯へ渡す glue だけ(M13)。
    /// 選択なし・comp なしは黙って無視(`commit_inspector_field` と同じ柵)。
    fn toggle_inspector_key(&mut self, row: inspector_pane::KeyRow) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return;
        };
        let Ok(property) = inspector_pane::key_row_property_id(row) else {
            return; // 標準 property なので起こらない — 安全側で無視。
        };
        let Some(t) = self.time_at_playhead() else {
            return;
        };
        let store = self.doc.view();
        let Ok(track) = store.track(layer, &property) else {
            return;
        };
        // 初キー化(track 無し)の値の正本: 解決済みの現在値(スロット参照も
        // ここで track へ戻る — `SetTrack` がスロットを普通の track に置き換える
        // 既存の意味論)。値が読めなければ行の既定値。
        let current_value = match store.value_at(layer, &property, t) {
            Ok(Some(value)) => value,
            _ => inspector_pane::key_row_default_value(row),
        };
        let new_track = match inspector_pane::toggled_key_track(
            track.as_ref(),
            self.session.playhead,
            composition.fps,
            current_value,
        ) {
            Ok(new_track) => new_track,
            Err(error) => {
                self.status = Some(error);
                return;
            }
        };
        drop(store);
        if let Err(error) = self.doc.apply(Intent::SetTrack {
            layer,
            property,
            track: new_track,
        }) {
            self.status = Some(format!("キーを書けない: {error}"));
        }
    }

    /// Attrs の Hidden トグル — 即 `Intent::SetAttrs` を1回出す(下書きを経由しない)。
    /// **`motolii-inspector-pane` crate へは移設していない**: 対象を
    /// `Session::selection` から引くだけで、書き込み自体は `LaneBarToggleMute`
    /// とも共有する cross-cutting な [`toggle_layer_hidden`] へ委譲する —
    /// Inspector 固有の write ロジックを1行も持たないため(RETURN の
    /// write-set 外 finding 参照)。
    fn toggle_inspector_hidden(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        self.toggle_layer_hidden(layer);
    }

    /// Attrs の Blend 巡回ボタン — 即 `Intent::SetAttrs` を1回出す(下書きを経由
    /// しない、[`toggle_inspector_hidden`] と同じ即時操作の形)。**lane bar には
    /// 無い**(発注書 EXACT TARGET — 対象は Inspector の選択レイヤのみ)ので
    /// `toggle_layer_hidden` のような cross-cutting な共有関数へは切り出さない。
    /// 対応 mode の一覧([`inspector_pane::SUPPORTED_BLEND_MODES`])は Inspector
    /// 側が持つ(発注書「決定済み事項」)。
    fn cycle_inspector_blend_mode(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .blend_mode;
        let patch = LayerAttrsPatch {
            blend_mode: Some(inspector_pane::next_blend_mode(current)),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("blend_mode を書けない: {error}"));
        }
    }

    /// Speed 欄(ATTRS、SP1 第一波、supervisor 決定1-7)— 下書きを確定して
    /// 1回の `Intent::SetTiming` を出す(1 gesture = 1 undo)。**`LayerTiming`
    /// の組み立て・duration 再計算はここで行う**(`inspector_pane` crate は
    /// `motolii-timeline-pane::clip_gesture` へ依存できないため — `inspector_pane`
    /// crate doc 参照)。数値として読めない・0以下は `Err` の理由文を status
    /// 帯へ渡す(`commit_inspector_field` と同じ glue の形、M13)。
    fn commit_inspector_speed(&mut self) {
        let Some(text) = self.inspector_speed_draft.take() else {
            return;
        };
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(percent) = chrome::parse_number(&text) else {
            self.status = Some(format!("数値として読めない: {text}"));
            return;
        };
        let Some((new_num, new_den)) = inspector_pane::percent_to_speed_ratio(percent) else {
            self.status = Some(format!("speed は正の値のみ: {text}"));
            return;
        };
        self.apply_speed(layer, new_num, new_den);
    }

    /// Speed 行の Reset ボタン — 下書きを経由せず即 100%(`Speed::NORMAL`)へ。
    /// [`commit_inspector_speed`] と同じ [`apply_speed`] を呼ぶので、既に100%
    /// なら no-op(Undo を積まない、決定7)は自動的に成り立つ。
    fn reset_inspector_speed(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        self.apply_speed(layer, Speed::NORMAL.num(), Speed::NORMAL.den());
    }

    /// [`commit_inspector_speed`]/[`reset_inspector_speed`] 共通の書き口。
    /// **start・source_in は不変**(決定4「source 窓が保存される」)、duration
    /// だけ [`timeline_pane::clip_gesture::retimed_duration`](第二波
    /// Shift+端drag と共有する純関数、δ 採択理由)で再計算する。**ロック
    /// layer は `Document::apply` の `check_not_locked` がそのまま拒む**
    /// (M13、move/trim と同じ理由文の型 — ここで重複判定しない)。**現在値と
    /// 同じ speed なら `Intent` を出さない**(決定7 — reset の no-op と、
    /// 打鍵で同値を submit した場合の両方をこの1箇所で満たす)。
    fn apply_speed(&mut self, layer: LayerId, new_num: i64, new_den: i64) {
        let Ok(new_speed) = Speed::try_new(new_num, new_den) else {
            self.status = Some("speed の分母は正でなければならない".to_owned());
            return;
        };
        let Ok(Some(meta)) = self.doc.view().meta(layer) else {
            return; // 素材が無い layer(起こらないはず) — 安全側で無視。
        };
        let old_timing = meta.timing;
        if old_timing.speed == new_speed {
            return; // 決定7: 同値は Undo を積まない。
        }
        let new_duration = timeline_pane::clip_gesture::retimed_duration(
            old_timing.duration,
            (old_timing.speed.num(), old_timing.speed.den()),
            (new_speed.num(), new_speed.den()),
        );
        let new_timing = LayerTiming {
            duration: new_duration,
            speed: new_speed,
            ..old_timing
        };
        if let Err(error) = self.doc.apply(Intent::SetTiming { layer, timing: new_timing }) {
            self.status = Some(format!("speed を書けない: {error}"));
        }
    }

    // ---- Timeline レーンバー(裁定147・第2波T1) ----
    //
    // 3つとも同じ形: 現在値を読んで反転した `LayerAttrsPatch` を1回出す
    // (`toggle_inspector_hidden` と同じ即時操作)。**対象は明示の `layer`**
    // (`Session::selection` ではない) — レーンバーは選択と無関係にどの行の
    // M/S/L も直接叩ける(裁定147「M/S/L もレーンバーで直接設定できる」)。
    //
    // 拒否の経路(M13)は書き口(`Document::write`)が既に持つ:
    // `hidden`/`solo` は locked な行への書き込みを理由つき `Err` で拒む
    // (`motolii_store::document::check`)。`locked` 自身は「解除/再ロックだけ
    // 常に通す」規則があるので `toggle_layer_lock` だけは常に成功する。

    /// M glyph。`LayerAttrs.hidden` をトグルする(`inspector_pane::Message::
    /// ToggleHidden` と同じ書き口 — 対象の layer が違うだけ)。
    fn toggle_layer_hidden(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .hidden;
        let patch = LayerAttrsPatch {
            hidden: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("hidden を書けない: {error}"));
        }
    }

    /// S glyph。`LayerAttrs.solo` をトグルする。
    fn toggle_layer_solo(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .solo;
        let patch = LayerAttrsPatch {
            solo: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("solo を書けない: {error}"));
        }
    }

    /// L glyph。`LayerAttrs.locked` をトグルする。locked 自身への書き込みは
    /// `Document::write` が locked な行でも常に通す(先に触れなくなる詰みを
    /// 作らない規則、`motolii_store::attrs::LayerAttrs::locked` の doc 参照)。
    fn toggle_layer_lock(&mut self, layer: LayerId) {
        let current = self
            .doc
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .unwrap_or_default()
            .locked;
        let patch = LayerAttrsPatch {
            locked: Some(!current),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("locked を書けない: {error}"));
        }
    }

    /// Step Forward/Back(正典 §5・U2)。`delta` の符号・歩幅はキー解決側
    /// (`resolve_navigation_key`)が既に決めている — ここは
    /// `timeline::nav::step_playhead` の clamp をそのまま適用するだけ。
    fn step_playhead(&mut self, delta: i64) {
        let duration = self.composition().map(|c| c.duration_frames).unwrap_or(0);
        self.session.playhead = timeline::nav::step_playhead(self.session.playhead, delta, duration);
    }

    /// JumpPrev/NextMeaningPoint(正典 §8.1・U2)。**見えている意味点**を集めて
    /// `timeline::nav::nearest_meaning_point` へ渡すだけ:
    /// - 常に: 選択 layer の表示中 property 行のキー菱形時刻(`timeline_property_rows`
    ///   — 選択 layer 1本ぶんしか描かれない、`projection::property_rows` の
    ///   EXACT TARGET 1 どおり)
    /// - `layer_only` が false の時だけ追加: comp locator(`markers()`)。
    ///   locator は layer に紐付かない(comp 単位)ので「選択レイヤー限定」
    ///   (Shift 付き)では対象から外れる — これが `layer_only` の意味そのもの
    ///
    /// 渡る先が無ければ何もしない(`nearest_meaning_point` が `None` を返す =
    /// no-op、既存の「拒否理由の無い no-op」と同じ形 — 意味点が無いのは
    /// エラーではない)。
    fn jump_meaning_point(&mut self, direction: timeline::nav::JumpDirection, layer_only: bool) {
        let mut points: Vec<i64> = self
            .timeline_property_rows()
            .iter()
            .flat_map(|row| row.keys.iter().map(|key| key.frame))
            .collect();
        if !layer_only {
            if let Some(fps) = self.composition().map(|c| c.fps) {
                points.extend(
                    self.markers()
                        .iter()
                        .filter_map(|marker| marker.time.try_to_frame_floor(fps).ok()),
                );
            }
        }
        if let Some(frame) = timeline::nav::nearest_meaning_point(&points, self.session.playhead, direction) {
            self.session.playhead = frame;
        }
    }

    /// JumpToClipIn/Out(正典 §8.1・U2)。対象は `Session::selection`(単一
    /// focus)の clip — 選択が無ければ何もしない(`nudge_keyframe` と同じ
    /// 「選択が無ければ no-op」の形。跳ぶ先を持たない操作を理由つき拒否に
    /// するほどの重さではない)。
    fn jump_clip_edge(&mut self, edge: timeline::nav::ClipEdge) {
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(row) = self.timeline_rows().into_iter().find(|row| row.id == layer) else {
            return;
        };
        self.session.playhead = timeline::nav::clip_edge_frame(row.start, row.duration, edge);
    }

    // ---- 実時間再生(A2、正典 §2 拘束5) ----

    /// `Message::ScrubTo`/`timeline_pane::Message::ScrubTo` の唯一の書き口。
    /// **再生中の scrub = seek**(発注書 ORACLE (c))— `Transport::seek` は
    /// `PlaybackClock::seek`(純粋、counters非依存)へ委譲するので実デバイス
    /// 無しで検証できる(`transport.rs` doc参照)。
    fn scrub_to(&mut self, frame: i64) {
        let frame = frame.max(0);
        self.session.playhead = frame;
        if self.transport.is_running() {
            if let Some(fps) = self.composition().map(|c| c.fps) {
                if let Ok(at) = RationalTime::try_from_frame(frame, fps) {
                    self.transport.seek(at);
                }
            }
        }
    }

    /// Space(発注書 ORACLE (d))。**ドラッグ中は無効**(正典 §2 拘束5「再生と
    /// 掴みは相互排他」)— Timeline の clip/key ドラッグと Inspector の
    /// 値セルドラッグのどちらでも封じる(掴み全般が対象、Timeline に限らない)。
    fn toggle_playback(&mut self) {
        if self.is_dragging() {
            return;
        }
        if self.transport.is_running() {
            self.freeze_playhead_from_transport();
            self.transport.stop();
        } else if let Err(error) = self.transport.start(open_real_playback, &self.doc, &self.session) {
            self.status = Some(error);
        }
    }

    /// 進行中の掴みがあるか(Timeline clip/key ドラッグ + Inspector 値セル
    /// ドラッグ)。`toggle_playback`(拘束5)専用の判定 — 個々の drag 状態は
    /// それぞれの pane/フィールドの持ち物のまま(このメソッドは束ねて読むだけ)。
    fn is_dragging(&self) -> bool {
        self.timeline.is_dragging() || self.inspector_drag.is_some()
    }

    /// Pause の直前に呼ぶ: 今の再生位置を`Session::playhead`へ確定させる
    /// (`transport.stop()`は位置を保存しないので、呼ぶ前にこれが要る —
    /// `transport.rs::Transport::stop`のdoc参照)。
    fn freeze_playhead_from_transport(&mut self) {
        let Some(fps) = self.composition().map(|c| c.fps) else {
            return;
        };
        if let Some(frame) = self.transport.position_frame(fps) {
            self.session.playhead = frame.max(0);
        }
    }

    /// 再生中tick(発注書 ORACLE (a)/(e))。`PlaybackClock::position()` を
    /// `Session::playhead` へ写す。comp 終端に達したら位置を終端へ揃えて
    /// 自動 Pause する(`JumpPlayheadToEnd`と同じ`comp_end_frame`を使うので
    /// 「終端」の定義が二重にならない)。
    fn advance_playback_tick(&mut self) {
        let Some(fps) = self.composition().map(|c| c.fps) else {
            self.transport.stop();
            return;
        };
        let Some(frame) = self.transport.position_frame(fps) else {
            return;
        };
        let duration = self.comp_duration();
        let end = timeline::nav::comp_end_frame(duration);
        if frame >= end {
            self.session.playhead = end;
            self.transport.stop();
        } else {
            self.session.playhead = frame.max(0);
        }
    }

    /// **ORACLE の試験専用の縫い目**(「デバイス抽象はフェイクで — A1と同じ手」)。
    /// `motolii_audio::PlaybackSession::for_simulation` で組んだフェイク
    /// セッション(実cpal無し、`PlaybackCounters`を`advance_supplied_for_
    /// simulation`で手動で進める)を、実デバイスを一切開かずに再生中状態へ
    /// 直接採用する。本番経路(`toggle_playback`)はこれを経由しない
    /// (`open_real_playback`を直接呼ぶ)。
    pub fn debug_start_playback_with_session(&mut self, session: motolii_audio::PlaybackSession) {
        self.transport.start_with_session(session);
    }

    /// 運転席が見るための口(`can_undo`/`can_redo`と同じ形)。
    pub fn is_playing(&self) -> bool {
        self.transport.is_running()
    }

    // ---- Settings パネル(タスク#18、裁定160 切片9) ----

    /// pane ローカル `Message` を畳んで書き口へ渡す glue。write ロジックの実体は
    /// `motolii_settings_pane::{apply_background_preset, commit_background_channel,
    /// commit_ui_scale}`(自由関数、`&mut Document`/`&mut Tokens`/下書きを明示
    /// 引数で受け取る形 — pane crate は `&mut self` を持てないため)。ここでは
    /// `self.doc`/`self.tokens`/下書きフィールドをそのまま貸すだけで、拒否理由
    /// (`Result::Err`)を `self.status` へ writeする以外の判断は持たない。
    fn update_settings(&mut self, message: settings_pane::Message) {
        match message {
            settings_pane::Message::ToggleSettingsPanel => {
                self.settings_panel_open = !self.settings_panel_open;
            }
            settings_pane::Message::BackgroundPreset(preset) => {
                if let Err(error) = settings_pane::apply_background_preset(&mut self.doc, preset) {
                    self.status = Some(error);
                }
            }
            settings_pane::Message::BackgroundChannelInput(channel, text) => {
                self.background_draft = Some(BackgroundFieldDraft { channel, text });
            }
            settings_pane::Message::BackgroundChannelSubmit(channel) => {
                if let Err(error) = settings_pane::commit_background_channel(
                    &mut self.doc,
                    &mut self.background_draft,
                    channel,
                ) {
                    self.status = Some(error);
                }
            }
            settings_pane::Message::UiScaleInput(text) => self.ui_scale_draft = Some(text),
            settings_pane::Message::UiScaleSubmit => {
                if let Err(error) =
                    settings_pane::commit_ui_scale(&mut self.tokens, &mut self.ui_scale_draft)
                {
                    self.status = Some(error);
                }
            }
        }
    }

    // ---- Stage 観測カメラ(裁定157、裁定160 切片10) ----

    /// pane ローカル `Message` を畳んで書き口へ渡す glue(`update_settings` と
    /// 同じ形)。**最初の2腕は元々 `self.observation` への直代入だけ**(計算を
    /// 持たない)だったので、pane crate 側には移していない。`CycleResolutionCap`/
    /// `ToggleCheckerboard`(裁定163 Stage 下縁状態帯)も同型の直代入 —
    /// `ToggleCheckerboard` は旧 `settings_pane::Message::ToggleCheckerboard`
    /// と同じ本体(`self.checkerboard` の反転)をここへ引っ越しただけ
    /// (`update_settings` 側の対応する腕は削除済み)。
    fn update_stage(&mut self, message: stage::Message) {
        match message {
            stage::Message::Observe(camera) => self.observation = Some(camera),
            stage::Message::ResetToRenderCamera => self.observation = None,
            stage::Message::CycleResolutionCap => {
                self.resolution_cap = self.resolution_cap.next();
            }
            stage::Message::ToggleCheckerboard => {
                self.checkerboard = !self.checkerboard;
            }
        }
    }

    // ---- Inspector の drag-to-scrub ----
    //
    // 5関数とも書き込み本体は `motolii-inspector-pane` crate 側の自由関数
    // (裁定160 切片8)——ここは `self.doc`/`self.inspector_drag`/
    // `self.session`/`self.keyboard_modifiers` をそのまま貸す glue だけ。
    // `enter_field_editing` だけは focus task(`iced::widget::operation::
    // focus`)の構築自体が Document を読み書きしない UI 純粋な orchestration
    // なので、crate を跨いだ `Task` の型変換を増やさないよう root 側に残した
    // (`inspector_pane` crate doc 参照)。

    /// 値セルの press — click か drag かはまだ未確定
    /// (`inspector_pane::FieldDragState::origin_x` が `None` のまま)。選択
    /// なし・comp なし・対応する field が投影に無い、のいずれも黙って無視。
    /// playhead(frame)と fps は press 時点の物を渡す — キー持ち track の
    /// 確定(キー upsert、`inspector_pane::edited_value_track`)の宛先になる。
    fn start_field_drag(&mut self, field: TransformField) {
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return; // comp が無ければ投影も無い — drag は始まらない。
        };
        let projection = self.inspector_selection();
        inspector_pane::start_field_drag(
            &mut self.inspector_drag,
            self.session.selection,
            projection.as_ref(),
            field,
            self.session.playhead,
            composition.fps,
        );
    }

    /// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op。
    /// **1px = 感度表の刻み**(`inspector_pane::dragged_value`)。
    fn continue_field_drag(&mut self, point: iced::Point) {
        let fine = self.keyboard_modifiers.shift();
        inspector_pane::continue_field_drag(&mut self.doc, &mut self.inspector_drag, point, fine);
    }

    /// 左クリック release(window 全体から — `mouse_area` 自身の `on_release` は
    /// bounds を出た drag を捉えられないので使わない)。**drag が実際に動いて
    /// いたら確定**: 最後の transient 値そのものを1回の本編集 `Intent` として
    /// `apply` してから `clear_transient`(1 gesture = 1 undo、overlay を残さない)。
    /// 動いていなければ click として type 編集へ切り替える
    /// (`inspector_pane::finish_field_drag` が `Ok(Some(field))` で知らせる)。
    fn finish_field_drag(&mut self) -> Task<Message> {
        match inspector_pane::finish_field_drag(&mut self.doc, &mut self.inspector_drag) {
            Ok(Some(field)) => self.enter_field_editing(field),
            Ok(None) => Task::none(),
            Err(error) => {
                self.status = Some(error);
                Task::none()
            }
        }
    }

    /// click(ドラッグせず release)→ type 編集。下書きを立て、text_input へ
    /// フォーカスを戻す(値セルは編集していない間は `mouse_area` + 静止
    /// `text` なので、click 直後にはまだ text_input が木に無く自動フォーカス
    /// されない — 明示的な focus task が要る)。**focus task の構築自体は
    /// Document を触らない**ので、値の計算(`inspector_pane::drag_origin`/
    /// `format_number`/`field_decimals`)以外はここへ移設していない
    /// (`inspector_pane` crate doc 参照)。
    fn enter_field_editing(&mut self, field: TransformField) -> Task<Message> {
        let Some(selection) = self.inspector_selection() else {
            return Task::none();
        };
        let Some((value, _)) = inspector_pane::drag_origin(&selection, field) else {
            return Task::none();
        };
        self.inspector_field_draft = Some(FieldDraft {
            field,
            text: inspector_pane::format_number(value, inspector_pane::field_decimals(field)),
        });
        iced::widget::operation::focus(inspector_pane::field_input_id(field))
    }

    /// Esc — 進行中の drag があれば復元、無ければ typing 下書き(値セル/名前欄)
    /// を破棄する(hint 行「Esc to cancel」を両方について正直にする)。
    ///
    /// drag/field_draft の分岐は [`inspector_pane::cancel_field_interaction`]
    /// (自由関数)——`true` を返したらここで終わる(元実装の早期 return を
    /// 保つ)。名前欄・Settings 下書きの破棄は Inspector pane の write-set 外
    /// (`settings_pane` の下書きも同じ Esc で破棄する、hint 文言との整合)
    /// なのでここに残した。
    fn cancel_inspector_interaction(&mut self) {
        if inspector_pane::cancel_field_interaction(
            &mut self.doc,
            &mut self.inspector_drag,
            &mut self.inspector_field_draft,
        ) {
            return;
        }
        self.inspector_name_draft = None;
        self.inspector_speed_draft = None;
        // Settings パネルの下書きも同じ Esc で破棄する(hint 文言との整合)。
        self.background_draft = None;
        self.ui_scale_draft = None;
    }

    // ---- 運転席が見るための口。**書けない** ----

    pub fn layer_count(&self) -> usize {
        self.doc.view().layers().len()
    }

    /// `StoreView` をそのまま返す(読むだけ)。**運転席の検分器具用**
    /// (`layer_count`/`composition` と同じ「運転席が見るための口」の1つ) —
    /// G1(裁定174)の ungroup 数値証明(`tests/suite/group_drive.rs`)が
    /// `resolve()`/`local_transform()` を直接叩くのに使う。`view` という名前は
    /// 既に `Shell::view() -> Element` が使っているので `store_view` にした。
    pub fn store_view(&self) -> StoreView<'_> {
        self.doc.view()
    }

    fn comp_duration(&self) -> i64 {
        self.doc
            .view()
            .composition()
            .ok()
            .flatten()
            .map(|c| c.duration_frames)
            .unwrap_or(0)
    }

    pub fn can_undo(&self) -> bool {
        self.doc.can_undo()
    }

    /// 直近の Save As が書いた path(未保存・Save a Copy 直後は前回のまま —
    /// `Message::SaveACopyRequested` doc 参照)。**運転席が見るための口**。
    pub fn current_path(&self) -> Option<&std::path::Path> {
        self.current_path.as_deref()
    }

    /// 未保存の変更があるか。**運転席が見るための口**(`Shell::is_dirty` の
    /// 公開版 — MB-1、`saved_revision` フィールド doc 参照)。
    pub fn is_project_dirty(&self) -> bool {
        self.is_dirty()
    }

    /// `Message::Redo` の可否。**運転席が見るための口**(`can_undo` と同じ形)。
    /// drag-to-scrub がキャンセル時に redo 空間を汚していないかを運転席から
    /// 確かめるのに使う(`inspector_drive.rs`)。
    pub fn can_redo(&self) -> bool {
        self.doc.can_redo()
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// 描き上がったフレームの識別。同じなら描き直していない。
    pub fn frame_token(&self) -> Option<(DisplayRevision, i64)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.revision.clone(), frame.playhead))
    }

    /// 今の comp 設定。**screenshot 器具**が Stage の letterbox を組むのに使う
    /// (`timeline_pane::TimelinePane::new` も同じ `composition()` 呼び出しをする)。
    pub fn composition(&self) -> Option<Composition> {
        self.doc.view().composition().ok().flatten()
    }

    /// 今の Session(選択・再生位置)。**読むだけ** — `Session` 自体のフィールドは
    /// pub だが、書ける口は `Message` 経由の `update()` だけ。
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// 市松トグルの今の状態。**screenshot 器具**が「実際に画面へ出る絵」を
    /// 再現するのに使う(`frame_rgba()` は市松を絶対に乗せない生値なので、
    /// この状態と `settings_pane::composite_checkerboard` を screenshot 側が
    /// 自分で組み合わせる必要がある — `lib.rs::build_stage_presenter_rgba` と
    /// 同じ形)。
    pub fn checkerboard_enabled(&self) -> bool {
        self.checkerboard
    }

    /// Settings パネルの開閉状態。**screenshot 器具専用**の読み口
    /// (`checkerboard_enabled` と同じ形) — `--settings-open` CLI フラグ
    /// (`main.rs`)経由で `Message::ToggleSettingsPanel` を実際に通した後の
    /// 状態を screenshot.rs が読み、Settings 領域を描くかどうかを分岐する。
    pub fn settings_panel_open(&self) -> bool {
        self.settings_panel_open
    }

    /// Browser パネルの開閉状態(B3)。**screenshot 器具専用**の読み口
    /// (`settings_panel_open` と同じ形) — `--browser-open` CLI フラグ
    /// (`main.rs`)経由で `Message::Browser(browser_pane::Message::
    /// ToggleBrowserPanel)` を実際に通した後の状態を screenshot.rs が読める
    /// ようにする。フラグそのものは `browser::PaneState::is_open` に住む
    /// (`state.rs` 冒頭 doc「Shell 側に per-variant 分岐を増やさない」) —
    /// この口は単なる薄い委譲。
    pub fn browser_panel_open(&self) -> bool {
        self.browser.is_open()
    }

    /// 描き上がった Stage フレームの生 RGBA。**常に背景込みの export 真値**
    /// (`Engine::render_frame`)— 市松トグルで一切変わらない。**screenshot
    /// 器具専用**(`screenshot.rs`)— 通常描画は shader Program(`stage_pane`)を
    /// 通る(裁定166 — GPU 高速路の間は `presenter_source: PresenterSource::Gpu`
    /// を渡す、`image::Handle` はもう作らない)。
    ///
    /// **裁定171 v2(M4)で `&mut self` になった** — GPU 高速路(`refresh_frame`)
    /// はこのフィールドを更新しない代わりに `rgba_stale` を立てるので、ここで
    /// 呼ばれた時だけ [`Self::ensure_rgba_fresh`] が CPU readback を1回払って
    /// 追いつかせる(EXACT TARGET 4「readback は要求された時だけ」)。呼び出し元は
    /// `screenshot.rs`(CLI 器具、`&mut Shell` は元から手元にある)と試験のみ —
    /// 通常描画(`Shell::view`)からは呼ばれない。
    pub fn frame_rgba(&mut self) -> Option<(u32, u32, &[u8])> {
        self.ensure_rgba_fresh();
        self.frame
            .as_ref()
            .map(|frame| (frame.width, frame.height, frame.rgba.as_slice()))
    }

    /// 市松 ON の間だけ `Some` — 裁定141「AE型の透明可視化モード」の入力
    /// (`Engine::render_frame_without_background` の結果そのもの、市松タイルは
    /// **まだ乗っていない**生値)。**screenshot 器具専用**(`screenshot.rs`)。
    /// `frame_rgba()` とは別物 — あちらは常に背景込みの export 真値。
    pub fn checkerboard_preview_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.frame.as_ref().and_then(|frame| {
            frame
                .checkerboard_preview_rgba
                .as_deref()
                .map(|rgba| (frame.width, frame.height, rgba))
        })
    }

    /// 今の観測カメラの状態(裁定157)。運転席/screenshot 器具が「カメラを通して
    /// 見る」(`None`)/「自由に見る」(`Some`)のどちらかを確かめる口
    /// (`checkerboard_enabled` と同じ形)。
    pub fn observation(&self) -> Option<ObservationCamera> {
        self.observation
    }

    /// 観測カメラ有効時の Stage 表示 RGBA(`Engine::render_frame_with_view_camera`
    /// の結果そのもの)。**`frame_rgba()`(export 真値)とは別物** —
    /// `checkerboard_preview_rgba` と同じ「screenshot 器具/試験専用」の形。
    /// `observation()` が `None` の間は常に `None`。
    pub fn observation_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.frame.as_ref().and_then(|frame| {
            frame
                .observation_rgba
                .as_deref()
                .map(|rgba| (frame.width, frame.height, rgba))
        })
    }

    /// Stage 下縁状態帯(裁定163)の今のプレビュー解像度 cap。運転席/試験が
    /// 見るための口(`checkerboard_enabled`/`observation` と同じ形)。
    pub fn resolution_cap(&self) -> stage::PreviewResolutionCap {
        self.resolution_cap
    }

    /// **裁定166 EXACT TARGET (b) の読み口**: shader Primitive へ実際に渡る
    /// RGBA の寸法。`frame_rgba()`(常に comp 解像度の export 真値)とは別に、
    /// 「今 Stage へ upload する寸法」だけを独立に確かめられるようにする
    /// (`resolution_cap()` と同じ形の試験用アクセサ)。
    pub fn stage_presenter_dims(&self) -> Option<(u32, u32)> {
        self.frame
            .as_ref()
            .map(|frame| (frame.presenter_width, frame.presenter_height))
    }

    /// Stage presenter の内容が変わった回数(裁定166 EXACT TARGET 1 の CPU 側
    /// の鍵)。shader Pipeline はこれを「前回アップロードした世代」と比較して
    /// `queue.write_texture` を省くかどうか決める(`StagePresenterPipeline::
    /// upload` 参照)。運転席/試験が「同じ内容の再描画では世代が動かない」
    /// ことを確かめる口。
    pub fn stage_presenter_generation(&self) -> Option<u64> {
        self.frame.as_ref().map(|frame| frame.presenter_generation)
    }

    /// **裁定171 v2 M4 / 残コスト調査(§1-4)の読み口**: 今の presenter が
    /// GPU 高速路(`PresenterSource::Gpu`)か CPU フォールバック
    /// (`PresenterSource::Cpu`)かを、実際に GPU device を動かさずに確かめる
    /// (`metrics::presenter_blits()` は shader Pipeline の実描画時にしか
    /// 増えない——`iced_test::simulator` は `Widget::draw` を叩かないため
    /// headless 試験では観測できない、`STAGE_PRESENTER_WGSL` doc 参照。この
    /// アクセサは `Shell::refresh_frame` が選んだ経路を `RenderedFrame` から
    /// 直接読むだけなので headless でも確かな証拠になる)。
    pub fn stage_presenter_is_gpu_backed(&self) -> Option<bool> {
        self.frame
            .as_ref()
            .map(|frame| matches!(frame.presenter_source, PresenterSource::Gpu(_)))
    }

    /// 今の Timeline の行。運転席が「層3枚の行が立つ」「選択が行と一致する」を
    /// 確かめる口(pane 自身が使う投影と同じ関数を呼ぶ)。
    pub fn timeline_rows(&self) -> Vec<timeline_pane::RowProjection> {
        timeline_pane::rows(&self.doc.view(), &self.session)
    }

    /// 今の property 行(キー行、第2波 T3)。選択 layer がキーを持つ property を
    /// 持たなければ空。運転席/`screenshot.rs` 器具が pane 自身と同じ投影を読む口
    /// (`timeline_rows` と同じ形)。
    pub fn timeline_property_rows(&self) -> Vec<timeline_pane::PropertyRowProjection> {
        let fps = self.composition().map(|c| c.fps);
        timeline_pane::property_rows(&self.doc.view(), &self.session, fps)
    }

    /// 今のマーカー一覧。**screenshot 器具**が Timeline のマーカー線を描くのに使う
    /// (`timeline_pane::TimelinePane::new` も同じ `markers()` 呼び出しをする)。
    pub fn markers(&self) -> Vec<motolii_store::Marker> {
        self.doc.view().markers().unwrap_or_default()
    }

    /// 素材台帳の一覧投影(裁定162 B1)。運転席/`browser_drive.rs` が
    /// 「AdmitPaths → 台帳に載る」を確かめる口(`timeline_rows`/`markers` と
    /// 同じ形 — pane 側の projection 関数をそのまま呼ぶだけ)。
    pub fn assets(&self) -> Vec<browser_pane::AssetListItem> {
        browser_pane::model::assets(&self.doc.view())
    }

    /// 今の Inspector 投影。運転席が「選択→行が出る」「編集→store が変わる」を
    /// 確かめる口(pane 自身が `view()` で使う投影と同じ関数を呼ぶ)。
    pub fn inspector_selection(&self) -> Option<inspector_pane::SelectionProjection> {
        inspector_pane::project(&self.doc.view(), &self.session)
            .ok()
            .flatten()
    }

    /// 今の Inspector 値セル編集下書き。運転席が「click(ドラッグせず release)
    /// → type 編集」への切り替わりを確かめる口(`pane` 自身が `view()` で
    /// 使うのと同じ状態)。
    pub fn inspector_field_draft(&self) -> Option<&FieldDraft> {
        self.inspector_field_draft.as_ref()
    }

    /// 今のデザイン値。運転席がトークン再読込を確かめる口。
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    /// `ui_scale` を適用した寸法。**全 pane・全 instrument(`screenshot.rs` 含む)は
    /// ここ経由で寸法を読む** — `tokens.dims` を直接読まない。`ui_scale` を掛ける
    /// のはこの関数(=[`tokens::Dimensions::scaled`] を呼ぶ唯一の場所)だけ
    /// (発注書「適用点1箇所」)。
    pub fn dims(&self) -> Dimensions {
        self.tokens.dims.scaled(self.tokens.ui_scale)
    }

    /// 現在の色トークン。`main.rs` の `iced::application(...).theme(...)` 結線
    /// (`tokens::theme_from_colors` 参照)が窓の外から読む唯一の口 — `dims()`
    /// と対になる公開アクセサ(`tokens` フィールド自体は private のまま)。
    pub fn colors(&self) -> Colors {
        self.tokens.colors
    }

    /// `TimelinePane` の組み立て。`view()` はこれを呼ぶだけ(第2波T5、正典
    /// §5.5「プレビューは毎フレーム」) — ドラッグ preview(`self.timeline` =
    /// `timeline_pane::PaneState`)を投影へ焼き込む経路を運転席が検査できる
    /// よう関数化した。**`TimelinePane::new` 自体のシグネチャ・既存呼び出し元は
    /// 汚さない** — `with_key_drag_active` と同じ「薄い builder を積み増す
    /// だけ」の形をもう2つ足しただけ。裁定160 切片7で `self.timeline_drag`/
    /// `timeline_key_drag` の2フィールド直読みから `self.timeline`(pane crate
    /// 所有の `PaneState`)経由の読み取り専用アクセサへ差し替えた
    /// (`clip_preview`/`key_preview`/`key_drag_active`、値は無改変)。
    pub fn build_timeline_pane(&self) -> timeline_pane::TimelinePane {
        let store = self.doc.view();
        // `ui_scale` 適用済み(`Shell::dims` — 適用点1箇所)。
        let dims = self.dims();
        let colors = self.tokens.colors;
        timeline_pane::TimelinePane::new(&store, &self.session, dims, colors, self.keyboard_modifiers)
            // 第2波T4: `timeline::key_rows` が継続イベント(move/release/右
            // クリック)を拾うかどうかの唯一の判断材料
            // (`TimelinePane::with_key_drag_active` の doc comment 参照)。
            .with_key_drag_active(self.timeline.key_drag_active())
            .with_clip_preview(self.timeline.clip_preview())
            .with_key_preview(self.timeline.key_preview())
    }

    /// `stage::StageOverlay` の組み立て(裁定157・S1〜S3)。`Shell::view` が
    /// 毎フレーム呼ぶ(`build_timeline_pane` と同じ「不変な投影を作り直す」形)。
    /// comp が無ければ `None`(`stage_pane` はその時 Stage 自体を出さないので
    /// 呼ばれないが、防御的に `Option` にして panic しない、M16)。
    ///
    /// **`screenshot.rs` も呼ぶ**(`pub` — `checkerboard_enabled` 等と同じ
    /// 「screenshot 器具専用」の公開理由)— 観測中のフレーム枠を同じ計算
    /// (`stage::StageOverlay::frame_corners_on_screen`)で再現するため。
    pub fn stage_overlay(&self) -> Option<stage::StageOverlay> {
        let composition = self.composition()?;
        let comp = motolii_core::CompSpec {
            width: composition.width,
            height: composition.height,
        };
        // レンダリングカメラ(`Composition.camera`、裁定113/115/116)。
        // track が無ければ既定値 — `resolve_camera` 自体がその規約を守るので
        // ここでは `unwrap_or_default` は「時刻が引けない」時のためだけの床。
        let render_camera = self
            .time_at_playhead()
            .and_then(|t| self.doc.view().resolve_camera(t).ok())
            .unwrap_or_default();
        Some(stage::StageOverlay::new(
            comp,
            render_camera,
            self.observation,
            self.dims(),
            self.tokens.colors,
        ))
    }

    pub fn view(&self) -> Element<'_, Message> {
        // pane が受け取るのは不変の投影だけ。
        let dims = self.dims();
        let colors = self.tokens.colors;
        let store = self.doc.view();

        // Settings パネル(タスク#18)。**表示だけの分岐** — 開いていなければ
        // 木に一切現れない(Q0: 効かない chrome を並べない、閉じている間は
        // 下書き入力欄も存在しないので誤操作の的にならない)。
        let mut layout = column![self.header()];
        // 旧 MB-0/MB-1 のドロップダウン表示分岐(file_menu_open/edit_menu_open)
        // は MB-2 で廃止 — menubar の開いた menu は widget 自身の overlay
        // (`motolii_menubar` の vendored `MenuBarOverlay`)として木に現れる。
        if self.settings_panel_open {
            // 題帯(2026-08-22 題帯レーン): pane 名の常設は5面すべて —
            // Settings は pane_grid 外なので `panel_title_band`(名札のみ、
            // drag なし)を pane 本体の上へ積む。
            layout = layout.push(
                column![
                    Self::panel_title_band("Settings", dims, colors),
                    settings_pane::view(
                        self.composition().as_ref(),
                        self.background_draft.as_ref(),
                        self.tokens.ui_scale,
                        self.ui_scale_draft.as_deref(),
                        dims,
                        colors,
                    )
                    .map(Message::Settings),
                ],
            );
        }

        // Browser/Inspector/Stage/Timeline は `pane_grid`(shell の pane_grid
        // 化、2026-08-22 実装レーン、`pane_layout.rs` 冒頭 doc 参照)。
        // Browser パネル(裁定162 切片 B3)は**表示だけの分岐ではなくなった**
        // — `self.panes.state` 自体が「開いていれば木にある・閉じていれば
        // 無い」を体現する(`pane_layout::build_configuration` doc、Q0)。
        // 各 pane の内容は closure の中で組み立てる(`Element` は `Clone` が
        // 無いので、外側で1回だけ作って使い回すことができない——`Fn` closure
        // は `state.panes.iter()` の各エントリごとに1回ずつ呼ばれるので、
        // 各腕がその場で組み立てれば十分・複製にはならない)。
        let browser_items = browser_pane::model::assets(&store);
        let grid = pane_grid::PaneGrid::new(&self.panes.state, |_pane, kind, _is_maximized| {
            let content: Element<'_, Message> = match kind {
                pane_layout::PaneKind::Browser => browser_pane::pane_view(
                    &self.browser,
                    &browser_items,
                    dims,
                    colors,
                )
                .map(Message::Browser),
                pane_layout::PaneKind::Inspector => {
                    // Inspector は canvas を使わない標準 widget 構成
                    // (inspector_pane crate 冒頭の doc comment)なので、
                    // 投影自体が `Element<'static, _>` を返す。
                    let inspector_selection = inspector_pane::project(&store, &self.session)
                        .ok()
                        .flatten();
                    inspector_pane::view_with_speed_draft(
                        inspector_selection.as_ref(),
                        self.inspector_field_draft.as_ref(),
                        self.inspector_name_draft.as_deref(),
                        self.inspector_speed_draft.as_deref(),
                        dims,
                        colors,
                    )
                    .map(Message::Inspector)
                }
                pane_layout::PaneKind::Stage => stage_pane(
                    self.frame.as_ref(),
                    self.stage_overlay(),
                    self.observation,
                    self.resolution_cap,
                    self.checkerboard,
                    dims,
                    colors,
                ),
                pane_layout::PaneKind::Timeline => {
                    // pane crate 化(裁定160 切片7)で `timeline.view()` は
                    // `Element<'static, timeline_pane::Message>` を返す
                    // (root の `Message` を pane crate から参照できないため
                    // — 循環回避)。`.map(Message::Timeline)` で1回だけ畳む
                    // (§3.1 の「pane-local Message を親が畳む」構成そのもの)。
                    // transport 帯込み(裁定180 — 下部 Play バーは撤去済み、
                    // 再生系の顔は timeline pane 上端の帯が正本)。
                    self.build_timeline_pane()
                        .with_playing(self.is_playing())
                        .view_with_transport()
                        .map(Message::Timeline)
                }
            };
            pane_grid::Content::new(content).title_bar(Self::pane_title_bar(*kind, dims, colors))
        })
        .width(Length::Fill)
        .height(Length::Fill)
        // フラット文法: リサイズグリップ = 8px(装飾余白としては使用不可、
        // `docs/reviews/2026-08-19-flat-grammar-canon-revision.md`)。
        // `spacing_m` が既にその値(8.0、`motolii-tokens-rs` 既定)——新しい
        // token を作らず既存を読む。`on_resize` の leeway=0 なので掴める幅は
        // `spacing + leeway` = `spacing_m` ちょうど(`PaneGrid::on_resize` doc)。
        .spacing(dims.spacing_m)
        // 退化(潰れて使えなくなる)パネルを防ぐ床(M13 無反応ゼロの一環)。
        .min_size(dims.row_height * 3.0)
        // Q0 適合に必須(`Message::PaneClicked` doc 参照) — pane_grid は
        // これを配線しないと本体全域が「capture されるのに無反応」になる。
        .on_click(Message::PaneClicked)
        .on_resize(0.0, Message::PaneResized)
        .on_drag(Message::PaneDragged)
        // drop 先の可視化(題帯レーン #3): drag 中、cursor が乗っている
        // 受け入れ region を pane_grid 自身が塗る(`widget/src/pane_grid.rs::
        // draw` の hovered_region 描画、fork rev 73e686e 実測)。色は既存
        // ロールのみ(S4): 面=`surface_hover`(「hover」の意味役割そのもの —
        // drag 中に cursor が乗っている受け入れ面)、縁=`focus`(操作が着地
        // する場所の合図)。split 線(picked/hovered)も `focus` — 太さは
        // `border_width * 2.0`(ln 器具の強調線と同じ導出、
        // `tests/suite/tonmana_token_fence.rs` の許容形)。
        .style(move |_theme| pane_grid::Style {
            hovered_region: pane_grid::Highlight {
                background: iced::Background::Color(colors.surface_hover),
                border: iced::Border {
                    color: colors.focus,
                    width: dims.border_width * 2.0,
                    radius: 0.0.into(),
                },
            },
            picked_split: pane_grid::Line {
                color: colors.focus,
                width: dims.border_width * 2.0,
            },
            hovered_split: pane_grid::Line {
                color: colors.focus,
                width: dims.border_width * 2.0,
            },
        });

        layout
            .push(container(grid).width(Length::Fill).height(Length::Fill))
            .push(status_band(self.status.as_deref(), &self.doc, dims, colors))
            .spacing(dims.spacing_m)
            .padding(dims.spacing_l)
            .into()
    }

    /// pane_grid の各 pane の題帯(pane 名入りの薄い常設帯 = drag ハンドル、
    /// 2026-08-22 題帯レーン。`view()` から呼ぶ)。
    ///
    /// **必須である理由**(fork rev 73e686e の pane_grid を実測): `Content`
    /// の `Draggable` 実装(`widget/src/pane_grid/content.rs::
    /// can_be_dragged_at`)は `title_bar` が無いと常に `false` を返す —
    /// `.on_drag(...)` を配線しただけではドラッグは一切始まらない(掴む
    /// 場所が無い)。
    ///
    /// **旧 grip 帯(匿名 8px `Space`)からの置き換え理由**: (1) S6 —
    /// 見えない帯はつかめない(利用者実窓検分「レイアウト変更ができない。
    /// ハンドルが無いせいか」)。(2) **旧帯は構造的にも死んでいた** —
    /// `TitleBar::is_over_pick_area`(`title_bar.rs` 実測)は title content の
    /// bounds を pick 対象から**除外**するため、全幅 `Space` を content に
    /// していた旧帯は pick 面積ゼロ=ドラッグが一切始まらなかった
    /// (`tests/suite/pane_band_drive.rs` が red→green で検分)。
    ///
    /// 新帯: pane 名(`pane_layout::title` 正本)を左端に置き、**残りの全幅が
    /// pick area**(S1 — 帯全体が大きい的。ラベル矩形だけは上記実測理由で
    /// pick 対象外という構造的限界が残る)。pick area の hover では pane_grid
    /// 自身が `Interaction::Grab` を返す(`content.rs::grid_interaction`
    /// 実測 — カーソル予告は追加配線なしで効く)。寸法は全て tokens 由来
    /// (裁定 2026-08-22「デザイン値の外出し徹底」): 帯高=
    /// `pane_header_height`(導出は `tokens/dimensions.json` の
    /// `_note_pane_header_height`)、文字=`micro_text`(正典バンド最小段 —
    /// 本帯が最初の消費者)、左右余白=`spacing_m`(ident/cols 帯と同段)。
    /// 色は既存ロールのみ(S4): 地=`surface_raised`(旧 grip と同じ)、
    /// 文字=`text_secondary`(章立ての控えめな見出し)。リサイズは従来どおり
    /// pane 間の 8px 境界(`PaneGrid::spacing`)が担う — drag 責務はこの帯へ
    /// 一本化。
    fn pane_title_bar<'a>(
        kind: pane_layout::PaneKind,
        dims: Dimensions,
        colors: Colors,
    ) -> pane_grid::TitleBar<'a, Message> {
        pane_grid::TitleBar::new(
            container(
                text(pane_layout::title(kind))
                    .size(dims.micro_text)
                    .color(colors.text_secondary),
            )
            .height(Length::Fixed(dims.pane_header_height))
            .align_y(iced::alignment::Vertical::Center)
            .padding([0.0, dims.spacing_m]),
        )
        .padding(0)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            ..container::Style::default()
        })
    }

    /// pane_grid 外のパネル(Settings — 全幅ストリップ)用の題帯。pane_grid の
    /// 題帯([`Self::pane_title_bar`])と同じ文法(帯高・文字・余白・色)だが、
    /// **drag ハンドルではない**(Settings は pane_grid の pane ではない —
    /// grab カーソルも出ないため「掴めそうで掴めない」嘘はつかない。名札のみ)。
    fn panel_title_band<'a>(label: &'a str, dims: Dimensions, colors: Colors) -> Element<'a, Message> {
        container(
            text(label)
                .size(dims.micro_text)
                .color(colors.text_secondary),
        )
        .width(Length::Fill)
        .height(Length::Fixed(dims.pane_header_height))
        .align_y(iced::alignment::Vertical::Center)
        .padding([0.0, dims.spacing_m])
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_raised)),
            ..container::Style::default()
        })
        .into()
    }

    /// shell chrome の線化(裁定137/139 の Inspector 以外の面への展開)。
    /// 旧実装はこの帯にコンテナが無く、地(背景)も境界(hairline)も持たない
    /// 生の `row!` だった — 帯の下の Stage/Inspector 行とは `spacing_m` の
    /// gap だけで離れており「面色の塗り分けで区切る」違反ではなかったが、
    /// 帯自身が「パネル」だと分かる縁を持っていなかった。Timeline の `.tp`
    /// (transport 帯、background=panel + border-bottom hairline)と同じ
    /// grammar をここへも延長する — 新しい視覚言語の発明ではない。
    /// MB-2(裁定179 D1 根治): 旧「輪郭箱ボタンの列」(File/Edit/Undo/Redo/
    /// + Layer/Browser/Settings)を `motolii-menubar::menu_bar`(左)+icon
    /// ボタン2つ(右端 — Browser トグル/Settings、裁定187 の icon+tooltip
    /// ペア第1号)へ差し替えた。メニューの中身(全て既存 `Message` の露出)は
    /// `menu.rs::menus()` が正本、見た目は menubar crate の「枠の文法」
    /// (裁定179: 輪郭なし・hover で面)。旧 Undo/Redo 箱ボタンは廃止 —
    /// 入口は Edit メニューと Cmd+Z/Cmd+Shift+Z の2本(S6 併存)。
    fn header(&self) -> Element<'_, Message> {
        let dims = self.dims();
        let colors = self.tokens.colors;
        let content = row![
            motolii_menubar::menu_bar(crate::menu::menus(), dims, colors),
            Space::new().width(Length::Fill),
            // **Browser トグル**(裁定162 切片 B3、normal-map id980 — panel 型
            // 出典のみなので S6 併設要件は無い)。Icon::GridView+tooltip
            // "Browser"。
            Self::header_icon_action(
                motolii_icons::Icon::GridView,
                "Browser",
                Message::Browser(browser_pane::Message::ToggleBrowserPanel),
                dims,
                colors,
            ),
            // **Settings**(歯車)。Icon::Settings+tooltip "Settings"。
            Self::header_icon_action(
                motolii_icons::Icon::Settings,
                "Settings",
                Message::Settings(settings_pane::Message::ToggleSettingsPanel),
                dims,
                colors,
            ),
        ]
        .spacing(dims.spacing_m)
        .align_y(iced::alignment::Vertical::Center);

        // 線化 D5(裁定179 文法1): 帯の輪郭線は廃止 — `surface_panel` の面が
        // app 地から明度1段浮くことが帯の輪郭([`band_chrome_style`] doc 参照)。
        container(content)
            .width(Length::Fill)
            .height(Length::Fixed(dims.panel_header_height))
            .padding([0.0, dims.spacing_s])
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_theme| band_chrome_style(dims, colors))
            .into()
    }

    /// header 右端の icon ボタン(裁定187 icon+tooltip ペア第1号)。輪郭なし・
    /// hover/press で面(裁定179 — `timeline_pane::transport` の
    /// `transport_button` と同じ枠の文法)。アイコン枠寸は旧文言ボタンの字寸
    /// (`body_text`)を [`motolii_icons::frame_px_for_glyph_px`](Material
    /// live area 比 24/20)で写した視覚同等寸 — 中間比の発明ではなく上流定数の
    /// 転写(transport のアイコン化と同じ判断)。tooltip が語(動詞名)を運ぶ
    /// (裁定187「アイコンは tooltip と対で使うのが標準」)— 面は menubar の
    /// 開いた menu と同じ `surface_raised`+hairline。
    fn header_icon_action<'a>(
        icon: motolii_icons::Icon,
        label: &'a str,
        message: Message,
        dims: Dimensions,
        colors: Colors,
    ) -> Element<'a, Message> {
        let glyph = motolii_icons::icon(
            icon,
            motolii_icons::frame_px_for_glyph_px(dims.body_text),
            colors.text_secondary,
        );
        let action = button(glyph)
            // 踏面はアイコンより大きく(S1、transport_button と同じ判断)。
            .padding(dims.spacing_s)
            .on_press(message)
            .style(move |_theme, status| {
                let background = match status {
                    // hover/押下: 面が浮く(輪郭は出さない — 裁定179)。
                    button::Status::Pressed | button::Status::Hovered => {
                        Some(iced::Background::Color(colors.surface_hover))
                    }
                    // 常時: 素のアイコン(輪郭なし・面なし)。
                    _ => None,
                };
                button::Style {
                    background,
                    // svg には効かない(tint が正)が、契約として ink を宣言しておく
                    // (`transport_button` と同じ注記)。
                    text_color: colors.text_secondary,
                    ..button::Style::default()
                }
            });
        tooltip(
            action,
            container(text(label).size(dims.caption_text).color(colors.text_primary))
                .padding([dims.spacing_xs, dims.spacing_s])
                .style(move |_theme| container::Style {
                    background: Some(iced::Background::Color(colors.surface_raised)),
                    border: iced::Border {
                        color: colors.border_default,
                        width: dims.border_width,
                        radius: 0.0.into(),
                    },
                    ..container::Style::default()
                }),
            tooltip::Position::Bottom,
        )
        .gap(dims.spacing_xs)
        .into()
    }

    /// 採番の正本は store 側([`StoreView::next_layer_id`])。**墓標を含む最大 id + 1**
    /// を返すので、削除した layer の id が再利用されない(2026-08-20 の敵対的レビュー修正)。
    fn next_layer_id(&self) -> u64 {
        self.doc.view().next_layer_id()
    }

    /// レイヤー差し色の自動割当(利用者裁定2026-08-21「色が足りない。Ableton は
    /// レイヤー全部に色」)。**決定論**(`LayerId % パレット長`) — Session に依存
    /// しない・undo/redo で結果が変わらない・同じ layer は常に同じ色になる。
    /// パレットの実体色は `tokens::Colors::label_palette`(トンマナ従属パレット、
    /// 発注書の候補C)にあり、ここは index を計算するだけ(色そのものはここに
    /// 埋め込まない)。生成点(`Message::AddLayer` 腕・`admit`)専用 — 既存 layer
    /// の色を後から変えるための関数ではない(その UI は後続波)。
    fn label_color_for_new_layer(id: LayerId) -> u8 {
        (id.0 % tokens::LABEL_PALETTE_LEN as u64) as u8
    }

    /// Document・再生位置・市松トグルのいずれかが変わった時だけ描き直す。
    /// 判定は `display_revision()`(履歴 + transient overlay の世代の組) —
    /// front が「前回の Document」を自分で持たないため。drag-to-scrub 中は
    /// overlay だけが動いて履歴の `revision()` は不変なので、`display_revision()`
    /// を見ないと drag 中の再描画が起きない。
    ///
    /// **市松は Document・playhead に依存しない表示分岐**だが、裁定141以降は
    /// 「背景を敷かない」別入力(`Engine::render_frame_without_background`)を
    /// 見せるモードなので、市松の有無だけ変わった時でも
    /// [`Self::checkerboard_preview_source`] 経由で engine をもう一度だけ回す
    /// (`Document`/`StoreView` 自体の再評価が増えるわけではない — 合成の
    /// 入力差分を取り直すだけ、裁定141「同一合成器への入力の違い」)。
    fn refresh_frame(&mut self) {
        let revision = self.doc.display_revision();
        let playhead = self.session.playhead;
        let checkerboard = self.checkerboard;
        let observation = self.observation;
        let resolution_cap = self.resolution_cap;
        let colors = self.tokens.colors;
        let ui_scale = self.tokens.ui_scale;

        if let Some(frame) = &self.frame {
            if frame.revision == revision && frame.playhead == playhead {
                if frame.checkerboard == checkerboard
                    && frame.observation == observation
                    && frame.resolution_cap == resolution_cap
                {
                    return;
                }
                let width = frame.width;
                let height = frame.height;
                let display = self.compute_display_source(observation, checkerboard, playhead);
                let (presenter_width, presenter_height, presenter_rgba) = match &display.full_rgba {
                    Some(rgba) => build_stage_presenter_rgba(
                        width,
                        height,
                        rgba,
                        display.checkerboard,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                    None => {
                        let frame = self.frame.as_ref().expect("直前の if let で確認済み");
                        build_stage_presenter_rgba(width, height, &frame.rgba, false, resolution_cap, colors, ui_scale)
                    }
                };
                if let Some(frame) = self.frame.as_mut() {
                    frame.presenter_source = PresenterSource::Cpu(Arc::new(presenter_rgba));
                    frame.presenter_width = presenter_width;
                    frame.presenter_height = presenter_height;
                    // 世代を進める(裁定166 EXACT TARGET 1) — shader Pipeline
                    // 側の「前回アップロードした世代」との比較でこれが鍵になる。
                    // ここへ来るのは中身が実際に変わった時だけ(市松/観測/cap の
                    // いずれかが変わった時 = このブロック自体が「変化があった」
                    // 早期return の否定側)なので、無条件に+1してよい
                    // (`metrics::record_handle_creation`はもう呼ばない — Stage
                    // 描画経路から `image::Handle` 生成そのものが無くなった)。
                    frame.presenter_generation += 1;
                    frame.checkerboard = checkerboard;
                    frame.checkerboard_preview_rgba = display.checkerboard_preview_rgba;
                    frame.observation = observation;
                    frame.observation_rgba = display.observation_rgba;
                    frame.resolution_cap = resolution_cap;
                }
                return;
            }

            // ---------------------------------------------------------------
            // 裁定171 v2(M4)GPU 高速路 — playhead だけが動いた時
            // (revision 不変・市松/観測がフォールバックを要求しない組み合わせの
            // 時)。**ここでは `self.engine.render_frame` を一切呼ばない**
            // (CPU readback ゼロ、ORACLE (a) の核心)—— `frame.rgba`(export
            // 真値)は更新せず `rgba_stale` を立てるだけ(`Self::ensure_rgba_fresh`
            // doc 参照)。
            //
            // 除外条件(いずれも裁定171 v2 §0-6 のフォールバックへ委ねる):
            // - `checkerboard`: CPU 合成フォールバック(市松の GPU 化は NON-GOAL)
            // - `observation.is_some()`: 観測視点は今回まだ zero-copy 経路に
            //   繋いでいない(NON-GOALS 外だが今回のスコープでもない、
            //   `render_resolved_to_texture` は camera を差し替えられる形なので
            //   将来はここを広げられる)
            //
            // **`resolution_cap`(½/¼)はもう除外条件ではない**(残コスト調査
            // `docs/reviews/2026-08-22-residual-bottleneck-survey.md` §1-4 の
            // 修理)。旧配線は cap≠Auto を理由にここを弾いて「フル再計算」
            // (CPU readback)へフォールスルーしていた——「速くするための cap」が
            // 実際には毎フレーム readback を払う遅い経路に自ら戻る bug だった。
            // GPU 高速路はここでは常に comp ネイティブ解像度のまま描く(cap は
            // GPU 側の描画コストを一切減らさない — r1 probe 実測「comp 出力の
            // 縮小はほぼ効かない、律速は素材帯域」と整合させたまま、無駄な
            // 縮小描画を足さない)。cap の見た目(粗さ)は presenter シェーダの
            // fragment 側サンプリング粒度で表現する(`StagePresenterProgram`
            // 構築側、`stage_pane` 関数の `pixel_scale` 参照)——CPU 側の
            // `stage_presenter_rgba` 縮小と同じ「明示的な縮小」を、テクスチャの
            // 実サイズは変えずに blit 時のサンプリングだけで再現する。
            //
            // 上のどれかに当たる、または snapshot が作れない(comp 消滅等)場合は
            // 下の「フル再計算」(既存、無改造)へフォールスルーする——
            // 「無反応より安全側」(M16)を保つ。
            if frame.revision == revision && !checkerboard && observation.is_none() {
                if let Some(snapshot) = self.build_preview_snapshot(playhead) {
                    if let Some(frame) = self.frame.as_mut() {
                        frame.playhead = playhead;
                        frame.width = snapshot.comp.width;
                        frame.height = snapshot.comp.height;
                        frame.presenter_width = snapshot.comp.width;
                        frame.presenter_height = snapshot.comp.height;
                        frame.presenter_source = PresenterSource::Gpu(Arc::new(snapshot));
                        frame.presenter_generation += 1;
                        frame.rgba_stale = true;
                        frame.checkerboard_preview_rgba = None;
                        frame.observation_rgba = None;
                    }
                    return;
                }
            }
        }

        let Ok(Some(composition)) = self.doc.view().composition() else {
            self.frame = None;
            return;
        };
        let Ok(t) = RationalTime::try_from_frame(playhead, composition.fps) else {
            self.status = Some("再生位置を時刻へ写せない".to_owned());
            return;
        };

        let render_start = std::time::Instant::now();
        // **export 真値**(`RenderedFrame::rgba`)— 観測カメラ・市松に一切
        // 影響されない唯一の経路(`Engine::render_frame`)。EXACT TARGET (d) の
        // 「export 用経路は observation 中でもレンダリングカメラの絵のまま」の
        // 直接の型的裏付け: この呼び出しは `observation`/`checkerboard` を
        // 一切引数に取らない。
        let render_result = self.engine.render_frame(&self.doc.view(), t);
        metrics::record_render_frame(render_start.elapsed());
        match render_result {
            Ok(rgba) => {
                let display = self.compute_display_source(observation, checkerboard, playhead);
                let (presenter_width, presenter_height, presenter_rgba) = match &display.full_rgba {
                    Some(preview) => build_stage_presenter_rgba(
                        composition.width,
                        composition.height,
                        preview,
                        display.checkerboard,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                    None => build_stage_presenter_rgba(
                        composition.width,
                        composition.height,
                        &rgba,
                        false,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                };
                // 世代は前フレームから引き継いで+1する(裁定166 EXACT TARGET 1)。
                // **ここは scrub/edit のたびに毎回通る経路**(revision か
                // playhead が変わった時点でこの分岐に落ちる — 「新規フレーム
                // だから0にリセット」ではない、`self.frame` がまだ無い最初の
                // 1回だけ0になる)。固定で0を書くと presenter_generation が
                // 常に0のまま動かなくなる事故を踏んだので明示的に注意書きした。
                let presenter_generation =
                    self.frame.as_ref().map(|frame| frame.presenter_generation + 1).unwrap_or(0);
                self.frame = Some(RenderedFrame {
                    revision,
                    playhead,
                    width: composition.width,
                    height: composition.height,
                    presenter_source: PresenterSource::Cpu(Arc::new(presenter_rgba)),
                    presenter_width,
                    presenter_height,
                    presenter_generation,
                    rgba,
                    rgba_stale: false,
                    checkerboard_preview_rgba: display.checkerboard_preview_rgba,
                    checkerboard,
                    observation,
                    observation_rgba: display.observation_rgba,
                    resolution_cap,
                });
            }
            Err(error) => {
                // 絵が出せなくても**画面は空にしない**(M16)。理由は帯に出す。
                self.status = Some(format!("Stage を描けない: {error}"));
            }
        }
    }

    /// 裁定171 v2(M4)GPU 高速路専用。`playhead` の時刻の resolve 済み
    /// スナップショットを作る——GPU への実描画は Pipeline 側
    /// (`StagePresenterPipeline::prepare`)がやる、ここは `Document` を読んで
    /// **所有データ**へ変換するだけ(`motolii_engine::Engine::render_resolved_to_texture`
    /// の入力そのもの)。comp が無い/時刻を写せない/camera・layer が解決でき
    /// ない、のいずれかなら `None` — 呼び出し側([`Self::refresh_frame`])は
    /// フル再計算(既存の CPU 経路)へ安全側フォールバックする。
    fn build_preview_snapshot(&self, playhead: i64) -> Option<PreviewSnapshot> {
        let view = self.doc.view();
        let composition = view.composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        let camera = view.resolve_camera(t).ok()?;
        let resolved = view.resolved_layers(t).ok()?;
        Some(PreviewSnapshot {
            comp: composition.spec(),
            background: composition.background,
            camera,
            resolved,
        })
    }

    /// 裁定171 v2(M4)EXACT TARGET 4:「readback は要求された時だけ」。GPU
    /// 高速路(`refresh_frame` の早期 return 枝)は `frame.rgba`(export 真値)
    /// を更新せず [`RenderedFrame::rgba_stale`] を立てる——このメソッドが
    /// [`Self::frame_rgba`] から呼ばれた時だけ、その場で1回 CPU readback して
    /// 追いつかせる。`checkerboard`/観測カメラ/½・¼ cap のいずれかが有効な
    /// 間は GPU 高速路自体を通らない(`rgba_stale` は常に `false` のまま)ので、
    /// このパスは「GPU 高速路を経由した後」だけ実際に readback を1回払う。
    fn ensure_rgba_fresh(&mut self) {
        let Some(frame) = &self.frame else { return };
        if !frame.rgba_stale {
            return;
        }
        let playhead = frame.playhead;
        let Ok(Some(composition)) = self.doc.view().composition() else {
            return;
        };
        let Ok(t) = RationalTime::try_from_frame(playhead, composition.fps) else {
            return;
        };
        match self.engine.render_frame(&self.doc.view(), t) {
            Ok(rgba) => {
                if let Some(frame) = self.frame.as_mut() {
                    frame.rgba = rgba;
                    frame.rgba_stale = false;
                }
            }
            Err(error) => {
                self.status = Some(format!("Stage を描けない: {error}"));
            }
        }
    }

    /// 市松 ON の間だけ「背景を敷かない」合成をもう一度取る(裁定141)。
    /// `checkerboard` が `false` なら常に `None`(呼び出し側は `RenderedFrame::rgba`
    /// を使う)。comp が無い/時刻を写せない/engine が描けない、のいずれかなら
    /// `None` を返し、呼び出し側は背景込みへ**安全側にフォールバック**する
    /// (無反応より、背景込みのまま出す方が M16 に近い — 市松が一時的に効かない
    /// だけで Stage 自体は空にならない)。描けなかった時は理由を status へ出す
    /// (M13)。
    fn checkerboard_preview_source(&mut self, checkerboard: bool, playhead: i64) -> Option<Vec<u8>> {
        if !checkerboard {
            return None;
        }
        let composition = self.doc.view().composition().ok().flatten()?;
        let t = RationalTime::try_from_frame(playhead, composition.fps).ok()?;
        match self.engine.render_frame_without_background(&self.doc.view(), t) {
            Ok(rgba) => Some(rgba),
            Err(error) => {
                self.status = Some(format!("市松プレビューを描けない: {error}"));
                None
            }
        }
    }

    /// 観測カメラ(裁定157)が有効な間だけ、その視点で再合成する
    /// (`Engine::render_frame_with_view_camera`)。`checkerboard_preview_source`
    /// と同じ「無反応より安全側フォールバック」— comp が無い/時刻を写せない/
    /// engine が描けない、のいずれかなら `None` を返し、呼び出し側は従来経路
    /// (市松/背景込み)へフォールバックする。描けなかった理由は status へ出す
    /// (M13)。
    ///
    /// **裁定160 切片10**: 計算の実体は `stage::observation_preview_source`
    /// (`&mut Engine`/`&StoreView` を明示引数で受け取る自由関数、`motolii-stage-pane`
    /// crate 側)へ移設済み — ここは `self.engine`/`self.doc.view()` を貸し、
    /// `Some(Err(_))` の枝でだけ `self.status` へ書く glue(関数名・シグネチャは
    /// 無改名、`update_settings` と同じ glue の形)。
    fn observation_preview_source(&mut self, observation: &ObservationCamera, playhead: i64) -> Option<Vec<u8>> {
        match stage::observation_preview_source(&mut self.engine, &self.doc.view(), observation, playhead) {
            None => None,
            Some(Ok(rgba)) => Some(rgba),
            Some(Err(error)) => {
                self.status = Some(error);
                None
            }
        }
    }

    /// Stage 表示(presenter)用の入力を決める。**`rgba`(export 真値)そのものには
    /// 一切触れない** — ここが返す物は表示専用の複製(`build_stage_presenter_rgba`
    /// へそのまま渡すか、`full_rgba: None` の時は呼び出し側が `RenderedFrame::rgba`
    /// を使う、既存の市松分岐と同じ形)。
    ///
    /// **優先順位**(裁定157): 観測カメラが有効なら観測視点の再合成を最優先で
    /// 使う([`Self::observation_preview_source`])。描けなければ(comp が無い等)
    /// 安全側で従来経路へフォールバックする。観測カメラが無効(`None`)なら
    /// 従来どおり市松の有無で分岐する([`Self::checkerboard_preview_source`]、
    /// 裁定141)。
    ///
    /// **既知の限界**: 観測カメラ有効中は市松プレビューを試みない
    /// (`Engine::render_frame_with_view_camera` は常に背景込み — 裁定157 の
    /// engine 側実装がそう組んである、`motolii_engine` のモジュール doc 参照)。
    /// 観測カメラは Stage 表示専用の別軸機能で、この2軸を同時に満たす engine
    /// エントリは今回のスコープ外(NON-GOALS外だが、必要になれば
    /// `render_frame_without_background_with_view_camera` 相当を engine 側へ
    /// 追加する形で拡張できる)。
    fn compute_display_source(
        &mut self,
        observation: Option<ObservationCamera>,
        checkerboard: bool,
        playhead: i64,
    ) -> DisplaySource {
        if let Some(observation) = observation {
            if let Some(rgba) = self.observation_preview_source(&observation, playhead) {
                return DisplaySource {
                    full_rgba: Some(rgba.clone()),
                    checkerboard: false,
                    checkerboard_preview_rgba: None,
                    observation_rgba: Some(rgba),
                };
            }
        }
        match self.checkerboard_preview_source(checkerboard, playhead) {
            Some(preview) => DisplaySource {
                full_rgba: Some(preview.clone()),
                checkerboard: true,
                checkerboard_preview_rgba: Some(preview),
                observation_rgba: None,
            },
            None => DisplaySource {
                full_rgba: None,
                checkerboard: false,
                checkerboard_preview_rgba: None,
                observation_rgba: None,
            },
        }
    }
}

/// Stage 表示用の RGBA を作る唯一の場所(裁定166: 旧 `build_stage_handle` の
/// 置き換え — 戻り値が `image::Handle` ではなく shader Primitive が直接使う
/// `(width, height, rgba)` になった)。`stage_presenter_rgba` で縮め(resolution
/// cap ½/¼ の時だけ)、**市松が有効なら display 用の複製にだけ**
/// [`settings_pane::composite_checkerboard_with_tile_px`] を乗せる — 呼び出し
/// 側が渡す `full_rgba` 自体は一切変更しない。
///
/// `full_rgba` は呼び出し側(`refresh_frame`)が選ぶ: 市松 OFF なら
/// `RenderedFrame::rgba`(背景込みの export 真値)、市松 ON なら
/// `Engine::render_frame_without_background` の結果(裁定141、背景を敷かない
/// 可視化専用の合成)— どちらの場合も、export/screenshot が読む生値
/// (`RenderedFrame::rgba`)自体はここでは一切変更しない。
///
/// **市松v2(利用者較正 2026-08-21「市松が見えない」の根治)**: `ui_scale` を
/// 明示的に受け取り、`stage_presenter_rgba` と同じ縮小率
/// (`stage::effective_preview_scale(1.0, resolution_cap)` — 裁定166 で auto 側
/// は常に `1.0`)を自分でも算出して [`settings_pane::checkerboard_tile_px`] に
/// 渡す — comp 画素空間固定だった旧タイル寸(8px)が縮小後にさらに痩せて実質
/// 不可視になっていた根因1をここで補正する
/// (`settings_pane::checkerboard_tile_px` doc 参照)。
fn build_stage_presenter_rgba(
    width: u32,
    height: u32,
    full_rgba: &[u8],
    checkerboard: bool,
    resolution_cap: stage::PreviewResolutionCap,
    colors: Colors,
    ui_scale: f32,
) -> (u32, u32, Vec<u8>) {
    let (presenter_width, presenter_height, mut presenter_rgba) =
        stage_presenter_rgba(width, height, full_rgba, resolution_cap);
    if checkerboard {
        let effective_scale = stage::effective_preview_scale(1.0, resolution_cap);
        let tile_px = settings_pane::checkerboard_tile_px(ui_scale, effective_scale);
        settings_pane::composite_checkerboard_with_tile_px(
            presenter_width,
            presenter_height,
            &mut presenter_rgba,
            colors,
            tile_px,
        );
    }
    (presenter_width, presenter_height, presenter_rgba)
}

// ---------------------------------------------------------------------------
// Stage presenter — shader widget の永続テクスチャ(裁定166)。
//
// `image(frame.handle.clone())` の置き換え。`iced::widget::shader::Program`
// (`Shader<Message, P>` widget)を自前実装する — `P::Primitive` は毎フレーム
// `Program::draw` が新しく作る軽い値(Arc の参照カウントを増やすだけ)、
// `P::Primitive::Pipeline` が実際の `wgpu::Texture`/`wgpu::RenderPipeline` を
// 持つ永続状態(`iced_wgpu::primitive::Storage` に `TypeId` 単位で1個だけ
// 生きる、`iced_wgpu-0.14.0/src/primitive.rs::BlackBox::prepare` 実測)。
//
// wgpu 型はすべて `iced::wgpu`(`iced_wgpu` の re-export、workspace の
// `wgpu 27.0.1` そのもの)を通す — 新規の wgpu 直接依存を足さない
// (裁定166 決定文書、fork の re_renderer は wgpu 29.0.4 で型が別物のため
// 混ぜられない)。
// ---------------------------------------------------------------------------

/// uniform buffer のレイアウト: letterbox(vertex shader 側、NDC 空間での
/// [offset_x, offset_y, scale_x, scale_y] — widget の `bounds` を viewport その
/// ものとして扱う shader Primitive の性質上(`iced_wgpu-0.14.0/src/lib.rs` の
/// render ループが `render_pass.set_viewport` を primitive の `bounds` へ
/// 設定してから `draw` を呼ぶ、実測)、この4値だけで letterbox 矩形が NDC 上に
/// 定まる)16 byte + `pixel_scale`(fragment shader 側、残コスト調査 §1-4の
/// 修理 — cap ½/¼ の「明示的な縮小」を GPU 高速路でも表現する fragment 側
/// サンプリング粒度、`fs_main` 参照)4 byte + WGSL 構造体アラインメント
/// (`vec2<f32>` の align=8 に揃えるための)4 byte padding = 24 byte。
const STAGE_PRESENTER_UNIFORM_BYTES: u64 = 24;

/// Stage 提示 shader の WGSL。頂点は `vertex_index`(0..6)から生成する
/// full-screen quad(2三角形)——専用の vertex buffer は持たない(letterbox の
/// 位置/大きさは uniform 側で表現する)。
///
/// **裁定171 v2(M4)`fs_main` の unmultiply(実窓検分要)**: `stage_texture` は
/// 常に `Rgba8UnormSrgb`(CPU 経路の `upload_cpu`・GPU 経路の main_target
/// 双方)——GPU が `textureSample` 時に自動で sRGB→linear decode する。fork の
/// `composite.wgsl`(`crates/viewer/re_renderer/shader/composite.wgsl`)は
/// `BlendWithBackground::Premultiplied` モードで
/// 「source is already premultiplied」と明記しており、CPU 経路の
/// `frame.rgba`/`presenter_rgba` も同じ compositor 出力(`Compositor::render*`)
/// を経由するので、**サンプル結果は経路によらず premultiplied な linear 値**
/// になる(main_target を直接サンプルする GPU 高速路は composite.wgsl を
/// 一切通らないが、`Compositor::render_to_texture` のモジュール doc
/// 「main_target の生存期間」が main_target 自体は composite 前の
/// premultiplied 値であることを示している)。この render pipeline の blend
/// state(下記、`SrcAlpha`/`OneMinusSrcAlpha` — 非 premultiplied over)は
/// straight alpha を前提にしているため、`fs_main` 側で明示的に unmultiply
/// してから返す(alpha=0 での 0 除算は `max(a, eps)` で回避)。**不透明画素
/// (alpha=1)では unmultiply は数学的に恒等**(`rgb/1.0 == rgb`)なので、
/// 既定の不透明黒背景コンポジションでは無改造時と見た目が変わらないはず——
/// 変わるのは半透明が絡む場合(市松 ON・透明背景プリセット)だけ。
/// **KNOWN.md 記載どおり、Stage の GPU 実描画は headless では検証できない
/// (`iced_test::simulator` が `Widget::draw` を叩かない)ので実窓検分が必須**。
const STAGE_PRESENTER_WGSL: &str = r#"
struct Uniforms {
    offset: vec2<f32>,
    scale: vec2<f32>,
    pixel_scale: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var stage_texture: texture_2d<f32>;
@group(0) @binding(2) var stage_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 0.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 1.0),
    );
    let uv = corners[vertex_index];

    var out: VertexOutput;
    out.position = vec4<f32>(
        uniforms.offset.x + uv.x * uniforms.scale.x,
        uniforms.offset.y - uv.y * uniforms.scale.y,
        0.0,
        1.0,
    );
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 残コスト調査(§1-4)の修理: `pixel_scale` < 1.0(GPU 高速路 + cap ½/¼)の
    // 間だけ、UV を「comp ネイティブ解像度 × pixel_scale」個のブロックへ量子化
    // してからサンプルする——GPU 高速路(main_target はネイティブ解像度のまま、
    // `Shell::refresh_frame` doc 参照)でも cap の「明示的な縮小」を、テクスチャ
    // の実サイズは変えずに再現する(CPU 経路の nearest-neighbor 事前縮小と
    // 同じ見た目、`stage_presenter_rgba` 参照)。
    //
    // `pixel_scale == 1.0`(CPU 経路は常にこれ——既にテクスチャ自体が cap
    // 相当に縮小済み、`StagePresenterProgram` doc 参照。GPU 経路も cap=Auto
    // の間は 1.0)の間は量子化を一切しない——素通しの `textureSample` のまま
    // (裁定166 の見た目を無改変で保つ。仮に `grid == dims` として量子化しても
    // 数学的にはテクセル中心へのスナップに退化するだけだが、それでも通常の
    // bilinear 補間からわずかにズレるため、"変える理由が無い経路は本当に
    // 何も変えない" を優先する)。
    var uv = in.uv;
    if (uniforms.pixel_scale < 1.0) {
        let dims = vec2<f32>(textureDimensions(stage_texture));
        let grid = max(dims * uniforms.pixel_scale, vec2<f32>(1.0));
        // WGSL の `/` は vecN/vecN か T/T のみ(scalar/vector 混在は `*` だけ
        // 許される) — `1.0 / grid` は無効なので `vec2<f32>(1.0) / grid` にする。
        let cell = vec2<f32>(1.0) / grid;
        uv = (floor(uv / cell) + vec2<f32>(0.5)) * cell;
    }
    let sampled = textureSample(stage_texture, stage_sampler, uv);
    // 裁定171 v2(M4、上のモジュール doc 参照): サンプル値は premultiplied
    // alpha。この pipeline の blend state は straight alpha を前提にしている
    // ので、ここで unmultiply する。alpha=1(不透明)では恒等。
    let straight_rgb = sampled.rgb / max(sampled.a, 1e-6);
    return vec4<f32>(straight_rgb, sampled.a);
}
"#;

/// `bounds`(widget local、論理px)へ comp(`width`×`height`)を letterbox で
/// 収めた矩形を、shader の viewport(=widget `bounds` そのもの)基準の NDC
/// offset/scale へ変換する。letterbox の実際の幾何は
/// [`stage::letterboxed_rect`](`image` widget の既定 `ContentFit::Contain` を
/// Rust で再現した単一源、`screenshot.rs::blit_letterboxed` と共有)をそのまま
/// 呼ぶ — 2箇所目の letterbox 実装を作らない(裁定166 EXACT TARGET 1)。
///
/// 退化(bounds/comp が 0 幅高)した時は `[0.0; 4]` を返す — 頂点が全て同じ
/// NDC 点に潰れるだけで、`draw` 自体は panic せず何も見えない矩形を描いて
/// 終わる(M16: 描けなくても panic しない)。
fn stage_presenter_letterbox_ndc(bounds: iced::Rectangle, width: u32, height: u32) -> [f32; 4] {
    if bounds.width <= 0.0 || bounds.height <= 0.0 {
        return [0.0; 4];
    }
    let comp = CompSpec { width, height };
    let Some(rect) = stage::letterboxed_rect(bounds, comp) else {
        return [0.0; 4];
    };

    let rel_x = (rect.x - bounds.x) / bounds.width;
    let rel_y = (rect.y - bounds.y) / bounds.height;
    let rel_w = rect.width / bounds.width;
    let rel_h = rect.height / bounds.height;

    // NDC: x+ は右、y+ は上。widget 左上(rel_x, rel_y)が NDC の
    // (offset_x, offset_y)、右下(rel_x+rel_w, rel_y+rel_h)が
    // (offset_x + 2*rel_w, offset_y - 2*rel_h) になるよう解く。
    [rel_x * 2.0 - 1.0, 1.0 - rel_y * 2.0, rel_w * 2.0, rel_h * 2.0]
}

#[cfg(test)]
mod stage_presenter_letterbox_ndc_tests {
    use super::*;

    /// bounds と comp が同じアスペクト(16:9)なら letterbox 帯が無い —
    /// widget いっぱいに描く、つまり NDC の [-1,1]×[-1,1] を丸ごと使う。
    #[test]
    fn matching_aspect_fills_the_full_ndc_range() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 1600.0,
            height: 900.0,
        };
        let [offset_x, offset_y, scale_x, scale_y] = stage_presenter_letterbox_ndc(bounds, 1920, 1080);
        assert!((offset_x - -1.0).abs() < 1e-6);
        assert!((offset_y - 1.0).abs() < 1e-6);
        assert!((scale_x - 2.0).abs() < 1e-6);
        assert!((scale_y - 2.0).abs() < 1e-6);
    }

    /// 正方形の bounds へ 16:9 comp を収めると上下に帯ができる —
    /// scale_y は 2.0 未満(全高は使わない)、offset_y は 1.0 未満(上端から
    /// 少し内側)。
    #[test]
    fn narrower_bounds_letterbox_shrinks_the_vertical_scale() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 900.0,
            height: 900.0,
        };
        let [_offset_x, offset_y, _scale_x, scale_y] = stage_presenter_letterbox_ndc(bounds, 1920, 1080);
        assert!(scale_y < 2.0, "letterbox 帯があるのに scale_y が全高のまま: {scale_y}");
        assert!(offset_y < 1.0, "letterbox 帯があるのに offset_y が上端のまま: {offset_y}");
    }

    /// 退化した bounds(幅0)では panic せず全ゼロを返す(M16)。
    #[test]
    fn degenerate_bounds_returns_all_zero_without_panicking() {
        let bounds = iced::Rectangle {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 900.0,
        };
        assert_eq!(stage_presenter_letterbox_ndc(bounds, 1920, 1080), [0.0; 4]);
    }
}

/// Stage の絵を描く shader widget の `Program`(裁定166)。**書ける状態を
/// 持たない**(`State = ()`)— カメラ操作等は別 widget(`stage::StageOverlay`、
/// `stack!` でこの上に重なる)が受ける、既存構造は無改変(`stage_pane` 参照)。
#[derive(Debug)]
struct StagePresenterProgram {
    source: PresenterSource,
    width: u32,
    height: u32,
    generation: u64,
    /// 残コスト調査(§1-4)の修理: fragment 側サンプリング粒度(`fs_main` の
    /// `pixel_scale` uniform へそのまま渡す)。`1.0` = 通常サンプリング
    /// (縮小無し)、`0.5`/`0.25` = ½/¼ cap 相当の粗さ。`stage_pane` 側が
    /// `PresenterSource::Cpu`/`Gpu` を見て決める(doc 参照)。
    pixel_scale: f32,
}

impl shader::Program<Message> for StagePresenterProgram {
    type State = ();
    type Primitive = StagePresenterPrimitive;

    fn draw(&self, _state: &Self::State, _cursor: iced::mouse::Cursor, bounds: iced::Rectangle) -> Self::Primitive {
        StagePresenterPrimitive {
            source: self.source.clone(),
            width: self.width,
            height: self.height,
            generation: self.generation,
            pixel_scale: self.pixel_scale,
            letterbox: stage_presenter_letterbox_ndc(bounds, self.width, self.height),
        }
    }
}

/// 1描画分の Stage 提示データ。**`Program::draw` が描画のたびに新しく作る**
/// (`iced_widget::shader::Program::draw` の契約)——だが [`PresenterSource`] は
/// `Arc` を貸す/複製するだけなので、内容が変わらない限り実コピーのコストは
/// ゼロ。実際に GPU 側の資源(CPU 経路= `queue.write_texture`・GPU 経路=
/// `Engine::render_resolved_to_texture`)を動かすかどうかは `generation` を
/// `StagePresenterPipeline` 側の記憶と比較して決める(裁定166/裁定171 v2
/// EXACT TARGET 1/2「フレーム内容が変わった時だけ」)。
#[derive(Debug)]
struct StagePresenterPrimitive {
    source: PresenterSource,
    width: u32,
    height: u32,
    generation: u64,
    /// `fs_main` の `pixel_scale` uniform(`StagePresenterProgram` doc 参照)。
    /// letterbox と同じく世代ゲートの対象外 — cap を巡回するだけなら世代を
    /// 進めない Message は無い(`CycleResolutionCap` は presenter_generation を
    /// 進める側)が、万一ズレても軽い float 1個の書き込みなので実害は無い。
    pixel_scale: f32,
    /// NDC 空間での letterbox 矩形 [offset_x, offset_y, scale_x, scale_y]
    /// (`stage_presenter_letterbox_ndc` 参照)。widget bounds が変わるたび
    /// (pane resize)再計算が要るので、世代ゲートの対象外(4 float の書き込み
    /// は軽い — `iced_wgpu::image::Layer::prepare` も transform uniform を
    /// 毎フレーム書いている、同じ考え方)。
    letterbox: [f32; 4],
}

impl shader::Primitive for StagePresenterPrimitive {
    type Pipeline = StagePresenterPipeline;

    fn prepare(
        &self,
        pipeline: &mut Self::Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &iced::Rectangle,
        _viewport: &shader::Viewport,
    ) {
        pipeline.resolve(device, queue, self);
    }

    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        pipeline.draw(render_pass)
    }
}

/// comp 寸法変化時だけ再作成する実体(裁定166 EXACT TARGET 1「永続
/// `wgpu::Texture`」)。`bind_group` はテクスチャ view を束ねているので、
/// テクスチャ再作成のたびに一緒に作り直す(`uniform_buffer`/`sampler` は
/// `StagePresenterPipeline` 側で使い回す)。**CPU フォールバック経路専用**
/// (裁定171 v2 §0-6、`PresenterSource::Cpu`)——GPU 高速路は
/// [`StagePresenterGpuTarget`] を使う。
struct StagePresenterTexture {
    width: u32,
    height: u32,
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    /// 直近でこのテクスチャへ実際に書き込んだ世代。`None` = まだ一度も
    /// 書いていない(テクスチャ再作成直後は必ず `None` に戻す — 新しい
    /// テクスチャの中身は不定なので、世代が偶然一致しても再アップロードが
    /// 要る)。
    uploaded_generation: Option<u64>,
}

/// 裁定171 v2(M4)。GPU 高速路が [`Engine::render_resolved_to_texture`] から
/// 直接受け取った main_target(+それを束ねた bind_group)。**CPU readback も
/// `queue.write_texture` もしない** — `texture`/`view` は fork の
/// `GpuTexture`(main_target)から `clone()` した薄いハンドル
/// (`motolii-compositor::Compositor::render_to_texture` のモジュール doc
/// 「main_target の生存期間」参照——次にこの Pipeline が GPU 高速路を再度
/// 呼ぶ時まで有効)。
struct StagePresenterGpuTarget {
    width: u32,
    height: u32,
    /// `bind_group` が参照している view の親 texture。**明示的に握り続ける**
    /// (drop すると view 経由の参照だけが残る形になり得るため、texture 自体も
    /// このスコープに留める——wgpu は resource の生存を内部で追跡するので
    /// 実害は無いはずだが、疑わしきは持つ側に倒す)。
    #[allow(dead_code)]
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    /// 直近でこの target を作った時の世代。`None` = まだ一度も描いていない。
    resolved_generation: Option<u64>,
}

/// `StagePresenterPipeline::draw` がどちらの bind_group を使うかの選択
/// (裁定171 v2 M4)。`prepare`(`resolve`)が世代ゲート越しに更新する。
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ActivePresenter {
    #[default]
    None,
    Cpu,
    Gpu,
}

/// Stage 提示 shader の永続 GPU 状態。`iced_widget::shader::Storage` に
/// `TypeId::of::<StagePresenterPrimitive>()` を鍵として1個だけ生きる
/// (iced の仕組みそのもの、`shader::Program`/`Pipeline` の doc 参照)。
struct StagePresenterPipeline {
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    uniform_buffer: wgpu::Buffer,
    /// CPU フォールバック経路(裁定171 v2 §0-6)。裁定166 の経路そのまま、
    /// 無改造。
    cpu_texture: Option<StagePresenterTexture>,
    /// 裁定171 v2(M4)。`Compositor::with_device` の上に組んだ Engine —
    /// **この Pipeline インスタンスが所有**(decode/upload キャッシュもここに
    /// 付いてくる、supervisor 裁定の推奨構造どおり)。Shell 側の headless
    /// `Engine`(export/screenshot 真値専用)とは完全に別インスタンス。
    gpu_engine: Engine,
    /// GPU 高速路が直近描いた main_target(裁定171 v2 M4)。
    gpu_target: Option<StagePresenterGpuTarget>,
    /// 直近の `resolve` がどちらの経路を使ったか——`draw` はこれで bind_group
    /// を選ぶ(CPU/GPU 両方の bind_group が生きていても、表示すべきは
    /// 「今のフレームで実際に描いた方」だけ)。
    active: ActivePresenter,
}

impl shader::Pipeline for StagePresenterPipeline {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("motolii-shell::stage_presenter sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            // wgpu 29(M01 統一後): mipmap は専用の `MipmapFilterMode` 型に分離された
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("motolii-shell::stage_presenter uniforms"),
            size: STAGE_PRESENTER_UNIFORM_BYTES,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("motolii-shell::stage_presenter bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // 残コスト調査(§1-4)の修理: `pixel_scale` を `fs_main` も
                    // 読むようになったので FRAGMENT を足す(letterbox の
                    // offset/scale は引き続き vertex 側専用)。
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(STAGE_PRESENTER_UNIFORM_BYTES),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("motolii-shell::stage_presenter pipeline layout"),
            // wgpu 29: layout は Option 化・push_constant_ranges は immediate_size へ
            bind_group_layouts: &[Some(&texture_bind_group_layout)],
            immediate_size: 0,
        });

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("motolii-shell::stage_presenter shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(STAGE_PRESENTER_WGSL)),
        });

        // blend state は `iced_wgpu::image` の pipeline(`src/image/mod.rs`)と
        // 同じ非 premultiplied alpha "over" — Stage の絵は元々 image widget
        // 経由でこの blend で描かれていたので、見た目のパリティをそのまま保つ。
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("motolii-shell::stage_presenter render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // 裁定171 v2(M4): iced が渡す device/queue の上に、この Pipeline
        // 専用の Engine(`Compositor::with_device` 版)を組む——供給者側
        // (compositor)のクローンではなく、`wgpu::Device`/`Queue` 自体が薄い
        // ハンドル(clone 可能、compositor 側 doc・`with_device` の実測どおり)
        // なので、ここで clone しても新しい GPU を建てるわけではない。
        // 失敗したら panic(`Shell::new` の `Engine::new().expect(...)` と
        // 同じ規律 — GPU が無ければ Stage 自体が成立しない)。
        let gpu_engine =
            Engine::with_device(device.clone(), queue.clone()).expect("GPU 高速路の Engine を用意できない");

        Self {
            render_pipeline,
            sampler,
            texture_bind_group_layout,
            uniform_buffer,
            cpu_texture: None,
            gpu_engine,
            gpu_target: None,
            active: ActivePresenter::None,
        }
    }
}

impl StagePresenterPipeline {
    /// **裁定171 v2(M4)入口**。`primitive.source` を見て CPU/GPU いずれかの
    /// 経路で実際に描き(世代ゲート越し)、letterbox uniform を書く。
    /// letterbox は経路に関わらず毎回書く(widget bounds は世代と無関係に
    /// 変わりうる — pane resize)。旧 `upload` の後継 — 引数を
    /// `&StagePresenterPrimitive` 1本にまとめる規律(clippy
    /// `too_many_arguments`)はそのまま引き継ぐ。
    fn resolve(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, primitive: &StagePresenterPrimitive) {
        match &primitive.source {
            PresenterSource::Cpu(rgba) => {
                self.upload_cpu(device, queue, primitive.width, primitive.height, primitive.generation, rgba);
                self.active = ActivePresenter::Cpu;
            }
            PresenterSource::Gpu(snapshot) => {
                self.resolve_gpu(device, primitive.width, primitive.height, primitive.generation, snapshot);
                self.active = ActivePresenter::Gpu;
            }
        }

        let letterbox = primitive.letterbox;
        let mut uniform_bytes = [0u8; STAGE_PRESENTER_UNIFORM_BYTES as usize];
        uniform_bytes[0..4].copy_from_slice(&letterbox[0].to_ne_bytes());
        uniform_bytes[4..8].copy_from_slice(&letterbox[1].to_ne_bytes());
        uniform_bytes[8..12].copy_from_slice(&letterbox[2].to_ne_bytes());
        uniform_bytes[12..16].copy_from_slice(&letterbox[3].to_ne_bytes());
        // 残コスト調査(§1-4)の修理: `fs_main` の `pixel_scale`(WGSL 構造体
        // `Uniforms.pixel_scale`、offset=16)。bytes[20..24] は WGSL 側の
        // `vec2<f32>` アラインメント(8 byte)に揃えるための padding —
        // ゼロのままで良い(`fs_main` は読まない)。
        uniform_bytes[16..20].copy_from_slice(&primitive.pixel_scale.to_ne_bytes());
        queue.write_buffer(&self.uniform_buffer, 0, &uniform_bytes);
    }

    /// 裁定166 の経路——**無改造**(旧 `upload` のこの部分をそのまま移した)。
    /// comp 寸法変化時だけテクスチャを作り直し、世代が前回と違う時だけ
    /// `queue.write_texture` する(裁定166 EXACT TARGET 1)。裁定171 v2 §0-6
    /// の CPU フォールバック(市松 ON・観測カメラ中・½/¼ cap 中)がここを使う。
    fn upload_cpu(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        generation: u64,
        rgba: &Arc<Vec<u8>>,
    ) {
        if width == 0 || height == 0 {
            self.cpu_texture = None;
            return;
        }

        let needs_new_texture = match &self.cpu_texture {
            Some(existing) => existing.width != width || existing.height != height,
            None => true,
        };

        if needs_new_texture {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("motolii-shell::stage_presenter cpu texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // `iced_wgpu::image` の atlas と同じ sRGB フォーマット(`color::
                // GAMMA_CORRECTION` が既定 true の時に選ぶ物、実測)— iced 全体が
                // 線形空間で合成する前提と合わせておかないと、他 widget(背景色
                // 等)と並んだ時に明るさがズレる。GPU 高速路の main_target
                // (`re_renderer::ViewBuilder::MAIN_TARGET_COLOR_FORMAT`)も同じ
                // sRGB タグ付き format なので、`fs_main` は経路を区別せず同じ
                // sampling で扱える(下の WGSL doc 参照)。
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("motolii-shell::stage_presenter cpu bind group"),
                layout: &self.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.cpu_texture = Some(StagePresenterTexture {
                width,
                height,
                texture,
                bind_group,
                uploaded_generation: None,
            });
        }

        let presenter_texture = self.cpu_texture.as_mut().expect("直前で確実に作成済み");

        if presenter_texture.uploaded_generation != Some(generation) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &presenter_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                rgba,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
            presenter_texture.uploaded_generation = Some(generation);
            metrics::record_presenter_upload(rgba.len());
        }
    }

    /// **裁定171 v2(M4)高速路**。CPU readback を一切しない —
    /// `Engine::render_resolved_to_texture`(→ 内部で
    /// `Compositor::render_to_texture`)が返す GPU texture/view をそのまま
    /// bind_group へ束ねるだけ(EXACT TARGET 3「readback/write_texture が
    /// 表示経路から消滅」)。世代が前回と同じなら何もしない(EXACT TARGET 2)。
    /// 描画に失敗したら(comp/layer が読めない等)前回の `gpu_target` を
    /// そのまま残す——M16「無反応より前フレームのまま」。
    fn resolve_gpu(
        &mut self,
        device: &wgpu::Device,
        width: u32,
        height: u32,
        generation: u64,
        snapshot: &Arc<PreviewSnapshot>,
    ) {
        if width == 0 || height == 0 {
            self.gpu_target = None;
            return;
        }

        let needs_render = match &self.gpu_target {
            Some(existing) => {
                existing.width != width || existing.height != height || existing.resolved_generation != Some(generation)
            }
            None => true,
        };
        if !needs_render {
            return;
        }

        let Ok((texture, view)) = self.gpu_engine.render_resolved_to_texture(
            snapshot.comp,
            snapshot.background,
            snapshot.camera,
            &snapshot.resolved,
        ) else {
            return;
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("motolii-shell::stage_presenter gpu bind group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        self.gpu_target = Some(StagePresenterGpuTarget {
            width,
            height,
            texture,
            bind_group,
            resolved_generation: Some(generation),
        });
        metrics::record_presenter_blit();
    }

    fn draw(&self, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let bind_group = match self.active {
            ActivePresenter::Cpu => self.cpu_texture.as_ref().map(|texture| &texture.bind_group),
            ActivePresenter::Gpu => self.gpu_target.as_ref().map(|target| &target.bind_group),
            ActivePresenter::None => None,
        };
        let Some(bind_group) = bind_group else {
            return false;
        };
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
        true
    }
}

/// `Shell::subscription` が使う、Inspector drag-to-scrub 用の window 全体の
/// 事象フィルタ。**翻訳だけ**(`subscription()` 冒頭の規律どおり、判断は持たない)
/// — 実際に drag 中かどうかの判断・Shift の要否は `Shell::update` 側
/// (`inspector_drag`/`keyboard_modifiers` の状態)。
///
/// `iced::event::listen_with` を選んだ理由: `status` を見ずに常に拾える
/// (`iced::keyboard::listen()`(Ignored 限定)だと、typing 中の text_input は
/// Escape を自分で `shell.capture_event()` する(`iced_widget::text_input`
/// 実測)ので、typing の Esc-cancel に使いたい場合に届かなくなる)。既存の
/// Escape/Backspace/Delete/NudgeKeyframe/ResetToRenderCamera はその方針どおり
/// `status` を無視する。**playhead ナビゲーション動詞束(U2)だけは逆**
/// — `resolve_navigation_key` へ `status == Captured` を渡し、text_input が
/// 既にそのキーを消費していれば一切出さない(正典 §5「テキスト入力中は
/// 横取りしない」。Home/End/裸の j/k/i/o は text_input 内でもカーソル移動・
/// 文字入力として意味を持つので、Escape 系と同じ「常に拾う」は採らない)。
fn inspector_pointer_event(
    event: iced::Event,
    status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::Inspector(inspector_pane::Message::PointerMoved(position)))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::Inspector(inspector_pane::Message::PointerReleased))
        }
        iced::Event::Keyboard(iced::keyboard::Event::ModifiersChanged(modifiers)) => {
            Some(Message::KeyboardModifiersChanged(modifiers))
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            ..
        }) => Some(Message::EscapePressed),
        // Timeline のキー削除(正典 §3)。**Mac の「Delete」キーラベルは
        // `Named::Backspace` として届く**(`iced_core::keyboard::key` の doc
        // コメント実測 — 主部の物理キーは Backspace、`Named::Delete` は
        // `Fn+Delete`/外付けキーボードの forward-delete)。両方拾う —
        // `Shell::delete_selected_keys` は選択が空なら no-op なので、text
        // 編集中に Backspace で文字を消す操作とは(選択キーが無い限り)衝突
        // しない。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace),
            ..
        })
        | iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete),
            ..
        }) => Some(Message::Timeline(timeline_pane::Message::DeleteSelectedKeys)),
        // NudgeKeyframe(正典 §8.1)。**既定割当は仮**(拘束6・裁定146の隣接注記
        // どおり、キーの皮は keymap 層が無い今だけ直結) — アクション名
        // (`timeline_pane::Message::NudgeKeyframe`)だけを正本として残す。
        // Alt+←/→=1フレーム、Alt+Shift+←/→=10フレーム。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft),
            modifiers,
            ..
        }) if modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(-step)))
        }
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight),
            modifiers,
            ..
        }) if modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::Timeline(timeline_pane::Message::NudgeKeyframe(step)))
        }
        // ResetToRenderCamera(裁定157・EXACT TARGET 1「カメラへ戻るは1アクション」)。
        // **既定割当は仮**(NudgeKeyframe と同じ「keymap 層が無い今だけ直結」の
        // 注記どおり) — アクション名(`stage::Message::ResetToRenderCamera`)だけを
        // 正本として残す。Shift+F。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. })
            if modifiers.shift()
                && matches!(key.as_ref(), iced::keyboard::Key::Character(c) if c.eq_ignore_ascii_case("f")) =>
        {
            Some(Message::Stage(stage::Message::ResetToRenderCamera))
        }
        // playhead ナビゲーション動詞束(U2、正典 §5・§8.1)。**この分岐だけ
        // `status` を見る**(上の doc 参照)— キーそのものの解決は
        // `resolve_navigation_key` へ委譲する(試験(`tests/suite/nav_drive.rs`)
        // が `iced::Event`/`Status` を毎回組み立てずにその関数を直接叩ける
        // ようにするための分割。既存の Alt+Arrow(NudgeKeyframe)/Shift+F
        // (ResetToRenderCamera)は上の枝で先に確定しているのでここには落ちて
        // 来ない)。
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            resolve_navigation_key(&key, modifiers, status == iced::event::Status::Captured)
        }
        _ => None,
    }
}

/// U2: playhead ナビゲーション動詞束のキー解決(正典 §5・§8.1)。**pure** —
/// `key`/`modifiers`/`captured`(他の widget が既にこのキーを消費したか、
/// `inspector_pointer_event` の `status == Captured` をそのまま渡す)だけを見て
/// `Message` を返す。`inspector_pointer_event` から分離してあるのは、試験が
/// `iced::Event`/`iced::event::Status` を毎回組み立てずにここを直接叩ける
/// ようにするため(`tests/suite/nav_drive.rs` の (e)/(f))。
///
/// - `captured` の間は何も返さない(正典 §5「テキスト入力中は横取りしない」)。
///   Home/End/裸の j/k/i/o は text_input 内でもカーソル移動・文字入力として
///   意味を持つキーなので、renaming/InspectorField 編集中に奪ってはいけない
/// - j/k/i/o は裸キー — `modifiers.command()` が立っていれば何も返さない
///   (前任レーンの実測教訓: Cmd+O 等の既存/将来ショートカットを奪わない)
/// - ←/→ は Alt 修飾時は対象外(`NudgeKeyframe` が既に使っている、二重発火
///   防止)
/// - Cmd+Z/Cmd+Shift+Z(Undo/Redo)・Cmd+C/V/X/D(Copy/Paste/Cut/Duplicate)・
///   Cmd+A/Cmd+Shift+A(SelectAll/DeselectAll)は S0 段差 群0(κ 台帳 FINDING 1)
///   で追加した編集ショートカット腕 — 対応する `Message` は既に実装・テスト済み
///   だったが、この関数に腕が無かったため UI からは header の Undo/Redo ボタン
///   経由でしか届かなかった
/// - Cmd+N/Cmd+Shift+S/Cmd+Q(New Project/Save As/Quit)は MB-1(裁定176)で
///   足した File 束 — `menu.rs::file_items` と同じ4動詞・同じ割当(Save a
///   Copy はメニューのみ、shortcut 出典ゼロ)
///
/// **既定割当は仮**(拘束6・NudgeKeyframe と同じ「keymap 層が無い今だけ直結」
/// の注記どおり) — アクション名(`Message::StepPlayhead`/`JumpPlayheadToStart`/
/// `JumpPlayheadToEnd`/`JumpMeaningPoint`/`JumpClipEdge`/`Message::Undo`/`Redo`/
/// `CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/`SelectAllLayers`/
/// `DeselectAllLayers`/`NewProjectRequested`/`SaveAsRequested`/
/// `QuitRequested`)だけを正本として残す。
pub fn resolve_navigation_key(
    key: &iced::keyboard::Key,
    modifiers: iced::keyboard::Modifiers,
    captured: bool,
) -> Option<Message> {
    if captured {
        return None;
    }
    use iced::keyboard::key::Named;
    use iced::keyboard::Key;
    match key.as_ref() {
        // Step Forward/Back(素=1フレーム、Shift=10フレーム)。Alt 付きは
        // NudgeKeyframe の領分なのでここでは扱わない(上の枝が先に取る)。
        Key::Named(Named::ArrowLeft) if !modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::StepPlayhead(-step))
        }
        Key::Named(Named::ArrowRight) if !modifiers.alt() => {
            let step = if modifiers.shift() { 10 } else { 1 };
            Some(Message::StepPlayhead(step))
        }
        Key::Named(Named::Home) => Some(Message::JumpPlayheadToStart),
        Key::Named(Named::End) => Some(Message::JumpPlayheadToEnd),
        // JumpPrev/NextMeaningPoint。Shift 付きで選択レイヤー限定(§8.1)。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("j") => {
            Some(Message::JumpMeaningPoint {
                direction: timeline::nav::JumpDirection::Prev,
                layer_only: modifiers.shift(),
            })
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("k") => {
            Some(Message::JumpMeaningPoint {
                direction: timeline::nav::JumpDirection::Next,
                layer_only: modifiers.shift(),
            })
        }
        // JumpToClipIn/Out。
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("i") => {
            Some(Message::JumpClipEdge(timeline::nav::ClipEdge::In))
        }
        Key::Character(c) if !modifiers.command() && c.eq_ignore_ascii_case("o") => {
            Some(Message::JumpClipEdge(timeline::nav::ClipEdge::Out))
        }
        // ---- 編集ショートカット(S0 段差 群0、κ 台帳 FINDING 1)。`Message::Undo`/
        // `Redo`/`CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/
        // `SelectAllLayers`/`DeselectAllLayers` は実装・テスト済みだったのに UI 入口
        // (キー)が1本も無かった(header の Undo/Redo ボタンのみ) — ここへ Cmd+文字 の
        // 腕を足して消化する。**既定割当は仮**(上の j/k/i/o と同じ「keymap 層が
        // 無い今だけ直結」の注記どおり、拘束6)。Shift の有無で Undo/Redo・
        // SelectAll/DeselectAll を振り分ける(NudgeKeyframe の歩幅振り分けと同じ形)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("z") => {
            Some(Message::Undo)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("z") => {
            Some(Message::Redo)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("c") => {
            Some(Message::CopyLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("v") => {
            Some(Message::PasteLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("x") => {
            Some(Message::CutLayer)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("d") => {
            Some(Message::DuplicateLayer)
        }
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("a") => {
            Some(Message::SelectAllLayers)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("a") => {
            Some(Message::DeselectAllLayers)
        }
        // ---- File 束(MB-1、裁定176)。メニュー(`menu.rs::file_items`)と
        // 同じ4動詞・同じ割当 — S6 併存(発注書「メニューと shortcut を同切片
        // で併設する義務」、M-menu 調査の該当4行は着手前は入口ゼロだった)。
        // `!modifiers.shift()` は Undo/SelectAll と同じ「将来の Cmd+Shift+N 系
        // に予約を残す」防衛ガード(KNOWN.md の Cmd+O 教訓と同じ理由 —
        // 修飾キーを厳密にしないと後から足す動詞と衝突する)。Save a Copy は
        // normal-map の shortcut 出典がゼロなのでキーを発明しない
        // (`menu.rs::file_items` doc 参照)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("n") => {
            Some(Message::NewProjectRequested)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("s") => {
            Some(Message::SaveAsRequested)
        }
        Key::Character(c) if modifiers.command() && c.eq_ignore_ascii_case("q") => {
            Some(Message::QuitRequested)
        }
        // ---- G1 グループ化動詞(裁定174)。Undo/Redo・SelectAll/DeselectAll と
        // 同じ Shift 振り分けの形(既定割当は仮、上の注記どおり)。
        Key::Character(c) if modifiers.command() && !modifiers.shift() && c.eq_ignore_ascii_case("g") => {
            Some(Message::GroupLayers)
        }
        Key::Character(c) if modifiers.command() && modifiers.shift() && c.eq_ignore_ascii_case("g") => {
            Some(Message::UngroupLayers)
        }
        // Play/Pause(A2、正典 §2 拘束5)。`captured`(text_input 入力中)なら
        // 上の早期returnで既に弾かれている — typing 中の Space は普通の
        // スペース文字入力のまま(playback を奪わない)。ドラッグ中かどうかの
        // 判断(拘束5)はここでは持たない — `Shell::toggle_playback` 側
        // (`is_dragging()`)。
        Key::Named(Named::Space) => Some(Message::TogglePlayback),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// pane — **`StoreView`(不変)・`&Session`・`Tokens`(読み取り専用の意匠値)しか
// 取らない**。書ける物を持たない。`timeline_pane::TimelinePane` も同じ制約。
// ---------------------------------------------------------------------------

/// **裁定163 S 空間スコア — 発注書 EXACT TARGET**: Stage pane の下縁に1行の
/// 状態帯を追加した(S5「下縁=状態帯」・S6「状態は隠れない」の初適用)。
/// `body`(ヒーロー、S5a 占有率)は `Length::Fill` のまま、帯は自然高
/// (`stage::state_band_view` 自身が `.padding`/`.spacing` だけで決める、
/// `status_band` と同じ「明示 `.height()` を持たない」形)——ヒーローの縁へ
/// 退く低重み要素として全体高を食わない。
fn stage_pane(
    frame: Option<&RenderedFrame>,
    overlay: Option<stage::StageOverlay>,
    observation: Option<ObservationCamera>,
    resolution_cap: stage::PreviewResolutionCap,
    checkerboard: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'_, Message> {
    let body: Element<'_, Message> = match frame {
        Some(frame) => {
            // 裁定166: Stage の絵は shader Program の永続テクスチャで提示する
            // (旧 `image(frame.handle.clone())` の置き換え — image widget の
            // 非同期アップロード「その間 draw_image は何も描かない」穴を構造で
            // 消す)。letterbox は `Program::draw` が widget bounds を受け取った
            // 時点で `stage::letterboxed_rect` を呼んで組む(2箇所目の
            // letterbox 実装を作らない、EXACT TARGET 1)。
            // 残コスト調査(§1-4)の修理: GPU 高速路(`PresenterSource::Gpu`)は
            // テクスチャ自体を comp ネイティブ解像度のまま描く(§refresh_frame
            // 参照)ので、cap の「明示的な縮小」は fragment 側のサンプリング
            // 粒度で再現する(`pixel_scale` uniform、下記 WGSL `fs_main` 参照)。
            // CPU 経路(`PresenterSource::Cpu`)は `build_stage_presenter_rgba`
            // が既にテクスチャ自体を cap 相当の寸法へ縮めてアップロード済みな
            // ので、ここでさらに縮小粒度を足すと二重適用になる——常に `1.0`
            // (無 no-op、`fs_main` 側の grid はテクスチャ実寸そのものになり、
            // 通常のサンプリングと事実上同じ)を渡す。
            let pixel_scale = match &frame.presenter_source {
                PresenterSource::Cpu(_) => 1.0,
                PresenterSource::Gpu(_) => stage::effective_preview_scale(1.0, resolution_cap) as f32,
            };
            let picture: Element<'_, Message> = Shader::new(StagePresenterProgram {
                source: frame.presenter_source.clone(),
                width: frame.presenter_width,
                height: frame.presenter_height,
                generation: frame.presenter_generation,
                pixel_scale,
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
            // 観測カメラの入力(ホイール/中ボタンドラッグ)とフレーム枠 overlay
            // (裁定157)。shader widget の上に重ねるだけ — 変形はしない
            // (Stage は letterbox 貼りのまま、`stage.rs` モジュール doc 参照)。
            match overlay {
                // 裁定160 切片10: `StageOverlay::view()` は `stage::Message`
                // (pane ローカル)を返すようになった — `.map(Message::Stage)`
                // で root `Message` へ畳んでから `picture` と同じ `stack!` へ
                // 積む(`timeline.view().map(Message::Timeline)` と同じ形)。
                Some(overlay) => stack![picture, overlay.view().map(Message::Stage)].into(),
                None => picture,
            }
        }
        None => text("comp がまだ無い")
            .size(dims.body_text)
            .color(colors.text_muted)
            .into(),
    };

    // 裁定166: Auto は 1.0 固定(iced 同期アップロード予算からの自動縮小柵は
    // 撤去 — `stage_auto_scale` は無くなった、フル解像度復帰)。状態帯の
    // 実効値表示は `effective_preview_scale(1.0, cap)` へそのまま追随する
    // (発注書 EXACT TARGET 2「常時表示」・「実効値表示の追随を確認」)。
    let auto_scale = 1.0;
    let band = stage::state_band_view(observation, resolution_cap, auto_scale, checkerboard, dims, colors)
        .map(Message::Stage);

    // letterbox は neutral dark(D8: 装飾 gradient 禁止・余白は neutral)。raw 値ではなく
    // token 経由の面色 + 罫線幅。
    // **高さは `Length::Fill`**(Inspector と並ぶ `row!` の中にいるため、以前の
    // `FillPortion(3)` は `Shell::view` 側のその `row!` 自身が持つ — 2箇所で
    // portion を重ねて割合をずらさない)。
    // 線化 D5(裁定179 文法1): Stage 容器の輪郭線も透明化(幅だけ残す=幾何
    // 不変)。letterbox は neutral dark(D8)のまま app 地と同族 — Stage の
    // 範囲は上の pane 題帯(`surface_raised`)・下の状態帯・隣接 pane の
    // `surface_panel` 明度段が読ませる(AE=「暗い隙間」の viewer と同文法)。
    container(column![container(body).width(Length::Fill).height(Length::Fill), band].spacing(0.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_app)),
            border: iced::Border {
                color: iced::Color::TRANSPARENT,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}


/// header 帯・status 帯の共通スタイル。線化 D5(裁定179 文法1、
/// `docs/reviews/2026-08-22-chrome-grammar-audit.md`): 帯は `surface_panel` の
/// 面で app 地(`surface_app`)から**明度1段**浮く — 輪郭線は描かない(透明
/// border で幅だけ残す=幾何不変)。参照3製品の「区切りは明度1段+間隔」の
/// shell chrome への適用(旧: 裁定139 の hairline 縁)。`pub`:
/// `tests/suite/band_line_fence.rs` が機械照合する。
pub fn band_chrome_style(dims: Dimensions, colors: Colors) -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(colors.surface_panel)),
        border: iced::Border {
            color: iced::Color::TRANSPARENT,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

/// shell chrome の status 帯。線化 D5(裁定179 文法1)で旧「border のみ・背景は
/// 塗らない」(裁定139 の hairline grammar)を上書き — 帯は
/// [`band_chrome_style`](`surface_panel` の明度1段+透明 border)で header と
/// 同じ器になり、「今どこからが summary か」は線でなく面の段差が示す。
fn status_band<'a>(
    status: Option<&str>,
    doc: &Document,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let layers = doc.view().layers().len();
    // 拒否・警告は status 帯の警告色(D2/D7: 文脈連動の status 帯文法)。
    // 通常の要約(layer数/edit位置)は弱文字 — 警告と同格に見せない。
    let (message, color) = match status {
        Some(status) => (status.to_owned(), colors.status_warning),
        None => (
            format!("layer {layers} / edit {}", doc.edit_head()),
            colors.text_muted,
        ),
    };
    container(text(message).size(dims.caption_text).color(color))
        .width(Length::Fill)
        .padding([dims.spacing_xs, dims.spacing_m])
        .style(move |_theme| band_chrome_style(dims, colors))
        .into()
}

// `button_style` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::button_style` へ移設した(純粋な再配置・挙動ゼロ変更)。
