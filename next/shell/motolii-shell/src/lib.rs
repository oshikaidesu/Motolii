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

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use iced::Task;

use motolii_core::{CompSpec, ResolvedCamera};
use motolii_engine::{Engine, ObservationCamera};
use motolii_store::{
    AutoSaveConfig, Composition, DisplayRevision, Document, Fps, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, ResolvedLayer, Revision, StoreView, TextDocument, Value,
};
// 裁定205 施工第2号 §D: `Fill`/`Brush`/`Rgb`(塗り)だけが `motolii-store`
// 未輸出(Cargo.toml のコメント参照)。

/// 自動保存(AUTOSAVE、SET+ B12 第2切片の結線)の tick 購読口(`auto_save.rs`
/// 冒頭 doc 参照)。`transport`/`pane_layout` と同じ「意味は薄く、window/timer
/// 事象の翻訳だけ」の module。
pub mod auto_save;
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

// ---------------------------------------------------------------------------
// lib.rs 分割(2026-08-23、`docs/reviews/2026-08-23-shell-split-plan.md` の
// 移送表どおり)。**中身は移していない** — `impl Shell` の各メソッドが
// 物理的にどのファイルに書かれているかだけが変わった(inherent impl は
// 同一 crate 内の複数ファイルに分けてよい、Rust の性質)。可視性は
// `pub(crate)` を既定にし、crate 外から直接呼ばれる自由関数
// (`resolve_navigation_key`/`band_chrome_style`)だけ下記で `pub use` して
// 旧パス(`motolii_shell::resolve_navigation_key` 等)を無傷に保つ。
pub(crate) mod assets;
pub(crate) mod create;
pub(crate) mod document_io;
pub mod export_ops;
mod input;
pub(crate) mod inspector_ops;
pub(crate) mod playback;
pub(crate) mod render;
mod selection;
mod stage_presenter;
mod view;

pub use input::{inspector_pointer_event, resolve_navigation_key};
pub use view::band_chrome_style;

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

/// `export_pane` は B09 第1切片(2026-08-22 発注)で新規追加した骨格 crate
/// (`motolii-export-pane`)。既存 pane と同じ命名口(`pub use X as Y;`)—
/// こちらも新規追加なので壊す既存参照は無い。crate 冒頭 doc の「shell 結線」
/// 節がそのままこの波の仕様書(第6波 EXACT TARGET 8)。
pub use motolii_export_pane as export_pane;

use file_dialogs::{FileDialogs, RfdDialogs};
use inspector_pane::FieldDraft;
use settings_pane::BackgroundFieldDraft;
use transport::Transport;

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

    // ---- 窓台帳(S1 daemon 骨格、裁定182/188 —
    // `docs/reviews/2026-08-22-multiwindow-probe.md`) ----
    /// boot の `window::open`(`Shell::boot`/`boot_fixture`)が開いた main 窓。
    /// 台帳(`Shell::main_window`)は boot 時点で**先行記帳**済み
    /// (`window::open` は Id を同期で採番する — runtime 無しの headless 試験
    /// でも台帳が読める)なので、この腕は runtime が実際に窓を開いた後の
    /// 再記帳(冪等)。
    MainWindowOpened(iced::window::Id),
    /// `toggle_settings_window` の `window::open` が開いた Settings 窓(S2)。
    /// `MainWindowOpened` と同型 — 台帳は open 時点で先行記帳済み、この腕は
    /// runtime 側の再記帳(冪等)。
    SettingsWindowOpened(iced::window::Id),
    /// どれかの窓が閉じた(`iced::window::close_events` 購読)。**main 窓なら
    /// アプリ終了**(probe 注意点1: winit shell は全窓が閉じると compositor を
    /// `None` 化 = device 破棄する — 「窓ゼロ状態を作らない(main 閉=exit)」を
    /// 不変量として維持し、そこへ到達させない)。Settings 窓なら台帳から
    /// 抹消するだけ — main は生き続ける(probe 実測
    /// `main_alive_after_settings_close=true`)。
    WindowClosed(iced::window::Id),

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
    /// JumpToLoopStart(map 1064「作業範囲の先頭へ」、B18/第5波結線)。既定割当
    /// Shift+Home。着地点は [`timeline_pane::WorkArea::first_frame`] —
    /// 作業範囲が無ければ no-op(`JumpClipEdge` と同じ「跳ぶ先が無ければ
    /// 動かない」の形)。
    JumpToWorkAreaStart,
    /// JumpToLoopEnd(map 1064「作業範囲の末尾へ」)。既定割当 Shift+End。
    /// 着地点は [`timeline_pane::WorkArea::last_frame`](半開の `end - 1`)。
    JumpToWorkAreaEnd,

    // ---- cross-cutting(timeline drag と inspector drag 両方が読む、pane split
    // survey §1.3「core 残留が妥当」) ----
    /// Shift の押下状態。`CursorMoved` 自体は modifiers を運ばないので
    /// `ModifiersChanged` を別途追って持つ(drag 中の1/10微調整に使う)。
    KeyboardModifiersChanged(iced::keyboard::Modifiers),
    /// Esc — drag 中なら復元、typing 下書き中(値セル/名前欄)ならそれを破棄。
    EscapePressed,

    // ---- Settings パネル(タスク#18、裁定160 切片9で pane ローカル Message へ集約) ----
    /// `motolii_settings_pane::sections::Message` を1本で畳む(iced 標準型 —
    /// 子 pane の `Message` を親が wrap する形)。SET+(B12 第1切片)の結線で
    /// 旧 `settings_pane::Message` 直持ちから section 版へ差し替えた — 旧腕は
    /// [`settings_pane::sections::Message::Legacy`] が丸ごと包む(sections.rs
    /// 冒頭 doc「結線互換の縫い目」の手順どおり)。腕ごとの doc は
    /// `settings_pane::sections::Message` 側を参照。
    Settings(settings_pane::sections::Message),

    // ---- Stage 観測カメラ(裁定157、裁定160 切片10で `motolii-stage-pane`
    // crate へ抽出、pane split survey §6 切片10) ----
    /// `motolii_stage_pane::Message` を1本で畳む(iced 標準の「子 pane の
    /// Message を親が wrap する」形、`Message::Settings`/`Message::Timeline`
    /// と同型)。腕ごとの doc は `stage::Message` 側を参照。
    Stage(stage::Message),
    /// Stage ギズモの drag 事象(GZ 結線、第5波)。[`stage::GizmoDrag`] は
    /// 既存 [`stage::Message`] と独立の pane-local message(exhaustive match を
    /// 壊さないための独立型 — `gizmo.rs` 冒頭 doc)なので、root はこの腕で
    /// `.map` して畳む。契約(1 drag = Start → Move* → Commit|Cancel)の
    /// 意味づけは [`Shell::update_gizmo`] — Inspector の drag-to-scrub と同経路
    /// (`Document::set_transient` → 確定時 `Intent::SetTrack` 1回 = 1 undo)。
    Gizmo(stage::GizmoDrag),

    // ---- Stage 離散ズーム束(B24、A10 id1441/1442/1491 の結線 — 第7波)----
    /// map 1441「Zoom In」。[`stage::zoom::zoom_step`] を
    /// `ZoomStepDirection::In` で呼ぶだけの薄い腕(実装は `input.rs::zoom_in`)。
    ZoomIn,
    /// map 1442「Zoom Out」。[`stage::zoom::zoom_step`] を
    /// `ZoomStepDirection::Out` で呼ぶ(`input.rs::zoom_out`)。
    ZoomOut,
    /// map 1491/1492「Zoom to fit」。`bounds` に依存しない唯一の
    /// [`stage::zoom::NamedZoomLevel`]([`input.rs::zoom_to_fit`] の doc 参照 —
    /// letterbox が既に comp を bounds へ contain-fit しているため
    /// `ObservationCamera::default()` と一致する)。id1490「Zoom to 100%」は
    /// 実 viewport bounds(iced 描画時にしか手に入らず `Shell` は保持しない)
    /// が要るため、この波では結線しない(RETURN 参照)。
    ZoomToFit,

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
    /// Delete(軸台帳 A08 id431)。Cut(`CutLayer`)の副作用としてしか存在
    /// しなかった削除を独立させた専用動詞 — クリップボードを経由しないので
    /// `selected_layers` を丸ごと1回の `apply_all`(= 1 undo)で消せる
    /// (`selection.rs::delete_selected_layers` doc 参照)。**キーボード入口
    /// (Backspace/Delete)はまだ無い** — `resolve_navigation_key` は
    /// write-set 外(`input.rs`、波C C-4 レーン)なので配線できていない
    /// (RETURN 参照)。
    DeleteSelectedLayers,
    /// Timeline rail M glyph 一括版(軸台帳 A08「Hidden トグル」の穴)。
    /// 行ごとのクリック(`inspector_pane::Message::ToggleHidden`)とは併存
    /// (裁定195・S6) — キーボード入口は同じく未配線(RETURN 参照)。
    HideSelectedLayers,
    /// Timeline rail S glyph 一括版(軸台帳 A08 id314 の穴)。
    SoloSelectedLayers,
    /// Timeline rail L glyph 一括版(軸台帳 A08 id874「Lock selected
    /// layers」の穴 — 対応する動詞がゼロだった)。
    LockSelectedLayers,

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
    /// **非同期(2026-08-22 再発注)**: 実際の確認は `Task::perform` で
    /// `Message::NewProjectConfirmed` へ戻ってくる(`Shell::confirm_then`
    /// 参照)── ネイティブ dialog はモーダルなので同期呼び出しは iced の
    /// イベントループを塞ぐ、`file_dialogs.rs` 冒頭 doc 参照。
    NewProjectRequested,
    /// `NewProjectRequested` の確認結果(true = 破棄して `reset_document`、
    /// false = 何もしない)。dirty でなければ確認ダイアログ自体を出さず
    /// `Task::perform` は即 `true` を運ぶ(`confirm_discard_calls` を増やさない
    /// ── `tests/suite/file_drive.rs` の柵)。
    NewProjectConfirmed(bool),
    /// 平の Save(id 1224、C-1 波C 発注「保存と復帰」)。**`current_path` が
    /// 既に分かっていれば確認もダイアログも出さず同じ場所へ黙って上書きする**
    /// (先例: Photoshop/AE/Premiere/Figma いずれも「一度保存した後は毎回パスを
    /// 聞かない」)。まだ一度も保存していない新規 project(`current_path` が
    /// `None`)は `SaveAsRequested` と同じ path 選択(`SaveAsPathChosen`)へ
    /// 合流する(先例: 初回保存はどの製品でもパスを聞くしかない)。
    ///
    /// **既知の穴(RETURN 参照)**: keymap 経由の Cmd+S 割当は write-set
    /// 境界(`input.rs` は C-4 の write-set)のためこのレーンでは配線して
    /// いない。File メニューの「Save」項目(`shortcut: None`)からのみ届く。
    SaveRequested,
    /// Save As(Cmd+Shift+S・File メニュー、id 1225)。
    /// [`file_dialogs::FileDialogs::pick_save_path`] で path を選び、
    /// `Message::SaveAsPathChosen` へ戻ってきたら既存の汎用 persist 経路
    /// (`Document::save`、履歴を畳んだ flattened 書き)で書く。成功したら
    /// 以後の `current_path` はこの path になる。
    SaveAsRequested,
    /// `SaveAsRequested` の path 選択結果。`None` = キャンセル(何もしない)。
    SaveAsPathChosen(Option<std::path::PathBuf>),
    /// Save a Copy(File メニューのみ、id 1227 — normal-map の shortcut 出典が
    /// ゼロなので shortcut を発明しない)。path 選択は Save As と同じ入口だが
    /// **`current_path`/dirty 状態は据え置く**(「現 path 維持のまま別名へ
    /// 書く」── 別ファイルへの書き出しであって、開いているプロジェクトの
    /// 身分は変わらない)。
    SaveACopyRequested,
    /// `SaveACopyRequested` の path 選択結果。`None` = キャンセル。
    SaveACopyPathChosen(Option<std::path::PathBuf>),
    /// Open(File メニュー、normal-map id 1226「Open Project」── shortcut 出典
    /// ゼロ(entries 2:0:0:0)なので shortcut を発明しない。`KNOWN.md` の
    /// 「Cmd+O 衝突の実測」の教訓どおり、裸の `o`(`JumpClipEdge(Out)`)が
    /// `!modifiers.command()` で既に Cmd+O を空けているが、出典が無い以上
    /// ここを埋めない)。dirty なら確認 → 確認できたら open dialog、を1本の
    /// `Task` へ直列化する(`Shell::confirm_then_pick_open` 参照)。
    OpenRequested,
    /// `OpenRequested` の最終結果(確認キャンセル・path キャンセルのどちらも
    /// `None` に畳まれる ── 呼び手は区別しない、`Shell::perform_open` 参照)。
    OpenPathChosen(Option<std::path::PathBuf>),
    /// File > Import Media…(normal-map id 592「Import (media/file)」の第2の
    /// 入口 ── 従来は OS drop のみだった、`file_dialogs.rs::FileDialogs::
    /// pick_import_paths` 冒頭 doc 参照)。複数選択可。選ばれた path はそのまま
    /// 既存の `Message::AdmitPaths` へ渡す(新しい記帳経路を作らない)。
    ImportMediaRequested,
    /// Quit(Cmd+Q・File メニュー、id 1223)。dirty なら confirm_discard を
    /// 経由してからプロセスを終了する([`file_dialogs::FileDialogs::quit`])。
    QuitRequested,
    /// `QuitRequested` の確認結果(true = `self.dialogs.quit()`、
    /// false = 何もしない)。`NewProjectConfirmed` と同じ形。
    QuitConfirmed(bool),
    /// OS の赤信号(×)ボタン(C-1 波C「保存と復帰」、A06「閉じる確認」の穴)。
    /// **main 窓は `exit_on_close_request: false` で開く**(`Shell::
    /// with_main_window` 参照)ので、winit fork は `CloseRequested` を自動で
    /// `Action::Window(Close)` に変換しない ── ここで拾って `QuitRequested`
    /// と同じ dirty ガード(`confirm_then`)へ通す。Settings/Export 窓は既定
    /// (`exit_on_close_request: true`)のままなので、この Message は main 窓
    /// のぶんしか届かない。
    WindowCloseRequested(iced::window::Id),
    /// `WindowCloseRequested` の確認結果(true = `iced::exit()`、false = 何も
    /// しない ── 窓は `exit_on_close_request: false` のおかげでまだ閉じて
    /// いないので、キャンセルすれば見た目どおり編集を続けられる)。
    WindowCloseConfirmed(bool),
    /// 起動直後、`document_io::read_last_project_path` が返した前回プロジェクト
    /// の path(C-1 波C「再起動で続きが開く」)。`Shell::boot` だけが発行する
    /// (`boot_fixture`/`new_fixture`/screenshot 器具経路は発行しない ── 器具の
    /// 意図した Document を上書きしない)。`None` = 記録が無い/前回が新規未保存
    /// project だった ── 既定 Document のまま何もしない。
    LastProjectPathRead(Option<std::path::PathBuf>),
    /// 前回プロジェクトを黙って再オープンした直後、autosave 世代の方が
    /// 本体ファイルより新しいと分かった時の確認(C-1 波C「autosave の読み
    /// 返し」、4製品先例「クラッシュ復帰は黙って上書きしない・聞く」)。
    /// true = `Shell::perform_recover_autosave`(復元して dirty のまま止める
    /// ── 確定は利用者の明示 Save)、false = 何もしない(黙って再オープンした
    /// 状態のまま)。
    AutoSaveRecoveryConfirmed(bool),

    // ---- 実時間再生(A2、正典 §2 拘束5) ----
    /// Space。Play/Pause をトグルする。**ドラッグ中は無効**(拘束5「再生と
    /// 掴みは相互排他」)— 判断は `Shell::toggle_playback` 側(`is_dragging()`)
    /// が持つ、翻訳層(`resolve_navigation_key`)は常にこの Message を出す。
    TogglePlayback,
    /// 再生中だけ発行される tick(`subscription()` が `is_running()` の間だけ
    /// 束ねる)。`Session::playhead` を `PlaybackClock::position()` へ追随させ、
    /// comp 終端に達したら自動で Pause する(発注書 ORACLE (a)/(e))。
    PlaybackTick,

    // ---- AUTOSAVE(SET+ B12 第2切片、shell 結線) ----
    /// `auto_save::tick_subscription` が `auto_save_config.interval_secs` 秒
    /// ごとに発行する tick。`Shell::run_auto_save` が受け口 —
    /// **再生中・ドラッグ中はスキップ**(正典 §2 拘束5と同型、`run_auto_save`
    /// doc 参照)。`auto_save_enabled=false` の間は `subscription()` がこの
    /// tick 自体を発行しない。
    AutoSaveTick,

    // ---- 第6波 shell 結線(2026-08-22 発注、EXACT TARGET 1〜8) ----
    /// Stage 方眼シート束(`stage::sheets` 冒頭 doc「結線は次波」— この波で
    /// 結線)。`stage::SheetMessage` は既存 [`Message`] と独立の pane-local
    /// message(`Message::Stage`/`Message::Gizmo` と同じ「独立 enum を root が
    /// `.map` して畳む」形)。トグル状態(`stage::SheetToggles`)は
    /// [`Shell::sheet_toggles`] が Session 水準で持つ(市松トグルと同格)。
    Sheet(stage::SheetMessage),
    /// Stage 矩形選択(`stage::marquee` 冒頭 doc「結線は supervisor」)。
    /// `stage::marquee::SelectLayers` も同じ独立 enum の形。適用先は
    /// [`Shell::apply_stage_selection`](`stage::marquee::apply_selection` を
    /// 呼ぶだけ)。
    Marquee(stage::marquee::SelectLayers),
    /// Timeline マーカーレーン(B19、`timeline::markers` 冒頭 doc の統合手順)。
    /// `timeline::markers::MarkerMessage` も同じ独立 enum の形 — canvas 差し替え・
    /// input 優先順位・drag 状態は pane crate 側(`canvas.rs`/`input.rs`、
    /// pub(crate))を触れないため未結線(RETURN 参照)。**keymap M=AddAtPlayhead
    /// と JumpTo の先取りだけ、この腕で完結する**(`Shell::update_marker` 参照)。
    Marker(timeline::markers::MarkerMessage),
    /// Export ダイアログ(B09、`export_pane` crate doc「shell 結線」節)。
    /// `motolii_export_pane::Message` を1本で畳む(`Message::Settings`/
    /// `Message::Browser` と同型)。
    Export(export_pane::Message),
    /// `toggle_export_window` の `window::open` が開いた Export 窓。
    /// `SettingsWindowOpened` と同型 — 台帳は open 時点で先行記帳済み、この腕は
    /// runtime 側の再記帳(冪等)。
    ExportWindowOpened(iced::window::Id),
    /// Export の実行が背景スレッドから届ける進捗/完了(C-3、
    /// `export_ops.rs` module doc「非同期化」参照)。`start_export` が返す
    /// `Task::run` の1本目の腕がこれへ翻訳する。
    ExportProgressed(export_ops::ExportEvent),
    /// Enter(単一選択時)= rename 開始(正典 §6、`timeline::write` 冒頭 doc)。
    /// キー解決(`resolve_navigation_key`)は選択を知らないので、実際の
    /// `LayerId` 解決とディスパッチは `Shell::update` 側(`self.session.selection`)
    /// が行う。
    RenameSelectedLayer,
}

/// 裁定171 v2(M4)。GPU zero-copy 経路で使う resolve 済みスナップショット。
/// `motolii_store::Document` を直接共有できない(`re_entity_db::EntityDb` が
/// `testing` feature 外では `Clone` を持たない)ので、`Shell::build_preview_snapshot`
/// が `StoreView` から抜き出した**所有データ**をここへ積む——
/// `motolii_engine::Engine::render_resolved_to_texture` の入力そのもの。
///
/// **`time`/`text_documents` は2026-08-22(ゼロコピー経路にも matte とテキストを
/// 通す発注)で新設**——`render_resolved_to_texture` がテキストの Hold 評価と
/// `TextDocument` 本体を要るようになったのに合わせた(`motolii_engine::Engine`
/// の doc 参照)。`resolved` の中の `LayerSource::Text` layer(matte 元も含む)
/// だけを対象に、その場で持っている `StoreView` から `text_document(id)` を
/// 引いて詰める——`resolved_layers(t)` を呼ぶのと同じ `view` から取るので
/// 追加の Document 走査は増えない。
#[derive(Clone, Debug)]
struct PreviewSnapshot {
    comp: CompSpec,
    background: [f32; 4],
    camera: ResolvedCamera,
    time: RationalTime,
    resolved: Vec<ResolvedLayer>,
    text_documents: HashMap<LayerId, TextDocument>,
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

/// Stage ギズモ drag、shell 側の transient(GZ 結線、第5波)。**Document では
/// ない** — Inspector の `FieldDragState` と同じ「確定まで front だけが持つ」
/// 身分。ギズモの座標解(`stage::GizmoDragState`)は canvas 内部に住み、shell は
/// 「どの layer のどの property を書いているか」と、確定のキー upsert の宛先
/// (Start 時点の playhead/fps — Inspector drag と同じ press 時点固定)だけを
/// 持つ。
struct GizmoShellDrag {
    layer: LayerId,
    /// Start が申告した property(Esc 連鎖 [`Shell::cancel_gizmo_drag`] の
    /// transient 掃除の宛先。Move/Commit は値側 [`stage::GizmoValue::property`]
    /// を読む — 契約上 1 drag = 1 property なので同じ値)。
    property: stage::GizmoProperty,
    /// Start 時点の playhead(frame)と fps。確定のキー upsert
    /// (`inspector_pane::edited_value_track`)の宛先 — drag の起点値は Start
    /// 時点の絵から読まれているので、確定の宛先も同じ時刻に固定する
    /// (`inspector_pane::FieldDragState::playhead_frame` と同じ判断)。
    playhead_frame: i64,
    fps: Fps,
    /// 1回でも `set_transient` を書いたか(Cancel 時に overlay を外す要否)。
    moved: bool,
}

/// [`stage::GizmoValue`](store の単位そのまま — `gizmo.rs` doc)→ store の
/// [`Value`]。shell 側は写すだけ(GZ 契約「shell 側は `Value::Vec2`/`Value::F64`
/// へ写すだけ」そのもの)。**`Anchor` はここへは来ない** — anchor drag は
/// anchor と position の2 property を対で書く必要があるため、
/// [`Shell::update_gizmo`] が `GizmoValue::Anchor { .. }` を専用の分岐で
/// 個別に処理する(`gizmo.rs::GizmoValue::Anchor` doc「shell は両方を同時に
/// 書く」参照)。
fn gizmo_store_value(value: stage::GizmoValue) -> Value {
    match value {
        stage::GizmoValue::Position(v) | stage::GizmoValue::Scale(v) => Value::Vec2(v),
        stage::GizmoValue::Rotation(v) => Value::F64(v),
        stage::GizmoValue::Anchor { .. } => {
            unreachable!("Anchor は update_gizmo が個別分岐で処理する — ここへは来ない")
        }
    }
}

// ---------------------------------------------------------------------------
// 連続量 drag(裁定217、E-5)。`Shell::value_drag` doc 参照 — `inspector_drag`
// (`LayerId + PropertyId + Intent::SetTrack` 固定)とは別の、track を持たない
// 値向けの第2の経路。書き込み本体は既存の commit_* 自由関数をそのまま呼ぶ
// (`Shell::finish_value_drag` 参照、write ロジックの複製ゼロ)。
// ---------------------------------------------------------------------------

/// [`ValueDragState`] が指す先。4つの家系(Composition/AutoSave/Background/
/// TextDocumentStyle 色)を1つの `Option` へ束ねる — press は排他的に1つしか
/// 起きない(`inspector_drag`/`inspector_text_style_drag` の排他と同じ形)。
#[derive(Clone, Copy, Debug, PartialEq)]
enum ValueDragTarget {
    CompWidth,
    CompHeight,
    CompFps,
    CompDuration,
    AutoSaveIntervalMinutes,
    AutoSaveGenerations,
    Background(settings_pane::BackgroundChannel),
    Color(inspector_pane::color::ColorTarget, inspector_pane::color::ColorChannel),
}

/// 値セルのキャプション drag-to-scrub、進行中の一時状態。
/// [`inspector_pane::FieldDragState`] と同じ形の縮小版 — Document の
/// transient overlay は使わない(対象に `LayerId + PropertyId` の宛先が無い
/// 家系がある)。**move 中は「既存の draft へ書き戻す」だけ** — text_input が
/// 下書きから表示を読む既存の経路(`comp_field_cell`/`channel_cell` 等)を
/// そのまま使うので、drag 中の値も Enter 編集中と同じ見た目になる。
struct ValueDragState {
    target: ValueDragTarget,
    /// press 時点の値(対象ごとの「表示単位」— px・fps 小数・フレーム数・分・
    /// 世代数・0..255 チャンネル)。
    start_value: f64,
    /// 最初の `PointerMoved` で確定する基準 x。`None` の間は click か drag か
    /// まだ未確定(`FieldDragState::origin_x` と同じ理由)。
    origin_x: Option<f32>,
    /// 少なくとも1回動いたか。release の確定要否の判定に使う。
    moved: bool,
}

/// [`ValueDragTarget`] ごとの px あたりの感度。`inspector_pane::transform::
/// drag_step_per_pixel` と同じ「値の意味域に合わせた目安」(実窓較正はこの
/// 発注の範囲外)。
fn value_drag_step_per_pixel(target: ValueDragTarget) -> f64 {
    match target {
        // 解像度・尺は 1px = 1単位(Position と同じ 1:1、`drag_step_per_pixel` 参照)。
        ValueDragTarget::CompWidth | ValueDragTarget::CompHeight | ValueDragTarget::CompDuration => 1.0,
        // fps は 1..240 の域を 100px 強で走査できる程度。
        ValueDragTarget::CompFps => 0.1,
        // 間隔(分)は 1..1440 の域。
        ValueDragTarget::AutoSaveIntervalMinutes => 0.5,
        // 世代数は 1..50 の域、10px で1段動く。
        ValueDragTarget::AutoSaveGenerations => 0.1,
        // RGBA は 0..255、Position と同じ 1:1。
        ValueDragTarget::Background(_) | ValueDragTarget::Color(_, _) => 1.0,
    }
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
    /// Inspector の TEXT section(B46 第1切片、裁定184)、編集中の下書き。同上
    /// — `TextDocumentStyle` は `TransformField`/track を経由しないので
    /// `inspector_speed_draft` と同型の別下書き(`TextField` で対象を区別)。
    inspector_text_field_draft: Option<inspector_pane::TextFieldDraft>,
    /// Inspector の TEXT section 色エディタ(`crate::color`、2026-08-22 発注
    /// 「歌詞が入れられる道を通す」で結線)、編集中の下書き。同上 —
    /// `inspector_text_field_draft` と同型の別下書き(対象は `ColorTarget`/
    /// `ColorChannel` の組で区別する)。
    inspector_color_field_draft: Option<inspector_pane::color::ColorFieldDraft>,
    /// Inspector TEXT section の Content 行(S4、#46 の穴塞ぎ)、**永続する**
    /// `text_editor::Content`(cursor/selection/undo history を内部に持つ実体
    /// — フレームごとに作り直すとカーソルが飛ぶ、`inspector_pane::text_section`
    /// doc「なぜ2つの経路が要るか」参照)。他の下書き(`Option<...>`)と違い
    /// **常に実在する**(空 = `Content::default()`)——`text_editor::new` が
    /// `&Content` を要求するので、選択が無い間も widget を組める空バッファを
    /// 切らさない。同期先レイヤーは [`Self::inspector_content_editor_layer`]。
    inspector_content_editor: iced::widget::text_editor::Content,
    /// 直近で [`Self::inspector_content_editor`] を同期した対象レイヤー。
    /// `None` = 「テキストレイヤーが選ばれていない」。`Shell::update` の
    /// 末尾(`sync_inspector_content_editor`)が選択と食い違えば再同期する —
    /// 再同期の直前、**古いレイヤーに未確定の編集が残っていれば1回自動で
    /// 確定する**(クリックで他レイヤーへ移る = blur-commit、マウス完遂路
    /// 裁定216)。
    inspector_content_editor_layer: Option<LayerId>,
    /// Inspector 値セルの drag-to-scrub。**Document ではない** — 同上
    /// (`inspector_pane::FieldDragState` doc comment 参照。型定義は裁定160
    /// 切片8で `motolii-inspector-pane` crate へ移設済み、置き場(この
    /// フィールド自身)は移設していない)。
    inspector_drag: Option<inspector_pane::FieldDragState>,
    /// TEXT section の Size/Line Height/Tracking の drag-to-scrub(D-1
    /// 結線、2026-08-23)。**`inspector_drag` とは別状態** — 対象が
    /// `TransformField`/`KeyRow` ではなく `TextStyleField`(A-1b の
    /// `TextStyleDragState`、`text.rs` doc 参照)なので同じ `Option` を
    /// 共有できない。press は排他(`ValuePressed`/`TextStyleValuePressed`)
    /// なので、window 全体購読(`PointerMoved`/`PointerReleased`)はこの
    /// 2つの `Option` を両方とも見て、`Some` な方だけ実際に動く
    /// (`inspector_ops.rs::continue_text_style_field_drag`/
    /// `finish_text_style_field_drag` 参照)。
    inspector_text_style_drag: Option<inspector_pane::TextStyleDragState>,
    /// track を持たない連続量(Composition W/H/FPS/尺・AutoSave 間隔/世代数・
    /// Background/Fill/Stroke RGBA)の drag-to-scrub(裁定217、E-5)。
    /// **`inspector_drag`/`inspector_text_style_drag` とは別状態** —
    /// あちらは `LayerId + PropertyId + Intent::SetTrack` に固く結合した
    /// 宛先を持つ(KNOWN.md 2026-08-23 A-2 実測「drag 機構は layer +
    /// PropertyId + track に固く結合している」)。ここの対象は `LayerId` を
    /// 持たない値(Composition/AutoSave は Document 全体 or shell-local)か、
    /// 持っていても track 化しない値(Background/色 RGBA は静的な
    /// read-modify-write)なので、既存の型をそのまま流用できない —
    /// **宛先を抽象するのではなく、形が違う対象向けの第2の経路を作った**
    /// (裁定215「借りられるなら借りる」は経路の**形**(press→drag→release、
    /// window 全体購読、既存 draft への書き戻し)を借りることで満たす —
    /// 書き込み本体は1行も重複させず、既存の `commit_comp_field`/
    /// `commit_auto_save_field`/`commit_background_channel`/
    /// `commit_text_style_color` をそのまま呼ぶ(`finish_value_drag` 参照)。
    /// これは `inspector_text_style_drag` 自体が A-1b で同じ理由により
    /// `inspector_drag` から独立させた先例と同型の判断)。
    value_drag: Option<ValueDragState>,
    /// 直近の Shift 押下状態。`CursorMoved` は modifiers を運ばないので
    /// `ModifiersChanged` から別途追う(drag の1/10微調整に使う)。
    keyboard_modifiers: iced::keyboard::Modifiers,
    /// Timeline rail の layer 行クリックの Shift 範囲選択の基点(E-2、軸台帳
    /// A08「Timeline clip move/trim」隣接の穴)。`Session::key_anchor`
    /// (`selected_keys` 側の同役)と同じ役だが、`selection.rs`(C-2 の家、
    /// `set_selected_layers` 以外は書き換え不可)を侵さないため、`Session`
    /// ではなくここ(`Shell` 自身、UI transient の置き場 — `keyboard_modifiers`
    /// と同格)に置く。単独/Cmd クリックで直近クリックへ更新、Shift 範囲では
    /// 不変(`timeline_pane::rows::resolve_layer_selection` の doc どおり)。
    layer_selection_anchor: Option<LayerId>,
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
    /// 完結する — `settings_window`(旧 `settings_panel_open`)/`edit_menu_open`
    /// と違い、パネル開閉フラグもこの `PaneState` の内側にある)。
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

    // ---- Settings 窓(タスク#18 → S2 で窓移住、裁定182/188) ----
    /// Settings 窓の台帳(旧 `settings_panel_open`)。**表示だけの状態** —
    /// Document でも `Session`(選択・再生位置)でもない(旧 doc の身分そのまま)。
    /// S2(窓の浮かし第1号)で「レイアウト分岐の bool」→「OS 窓の Id」へ意味が
    /// 変わった: `Some` = Settings 窓が開いている。`ToggleSettingsPanel` が
    /// open/close の両方を駆動する([`Shell::toggle_settings_window`])。
    /// Settings の**中身**の状態(`background_draft`/`ui_scale_draft`)は従来
    /// どおり `Shell` に住むので、窓を閉じても何も失われない(probe §Q3)。
    settings_window: Option<iced::window::Id>,
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
    /// Settings 窓の Composition 数値欄(W/H/FPS/尺、SET+ B12 第1切片)の編集
    /// 下書き。**Document ではない**(`background_draft` の隣に住む同じ身分 —
    /// Enter で `settings_pane::sections::commit_comp_field` が1回の
    /// `Intent::SetComposition` を出すまで store に触らない)。
    comp_draft: Option<settings_pane::sections::CompFieldDraft>,
    /// AUTOSAVE 有効/無効(SET+ B12 第2切片、`ToggleSettingsPanel` と同じ身分 —
    /// Document/undo を経由しない shell-local bool)。既定 `true`(AE `Auto-Save`
    /// の既定「有効」に合わせる)。`false` の間は `subscription()` が
    /// `Message::AutoSaveTick` そのものを発行しない。
    auto_save_enabled: bool,
    /// 自動保存の間隔・世代数。**Document ではない**
    /// (`motolii_store::persist::AutoSaveConfig` doc「Settings が読める形の
    /// 置き場」参照 — `ui_scale`/`Tokens` と同じ「Settings が直接持ち回す値」)。
    auto_save_config: AutoSaveConfig,
    /// AUTOSAVE 数値欄(間隔・世代数)の編集下書き。`comp_draft` の隣に住む
    /// 同じ身分(確定するまで front だけが持つ)。
    auto_save_draft: Option<settings_pane::sections::AutoSaveFieldDraft>,
    /// 最後に自動保存した時点の `Document::revision()`
    /// (`motolii_store::Document::auto_save` の `since` 引数)。**`saved_revision`
    /// とは別の鍵**: 自動保存は `current_path` の隣(`<name> auto-save/`)へ
    /// 別ファイルを書くだけで、明示 Save(Save As)が指す本体は更新しない —
    /// `saved_revision`(`is_dirty`/Quit確認の唯一の判定根拠、`saved_revision`
    /// フィールド doc 参照)を自動保存の成否で動かすと「本体は未保存なのに
    /// dirty 表示が消える」事故になる。
    last_auto_saved: Revision,

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
    /// JKL シャトルの現在倍率(B21、第5波結線)。意味(1→2→4→8 の状態機械)は
    /// [`timeline_pane::ShuttleState::apply`] が正本 — shell はこの値を持ち、
    /// tick(`Message::PlaybackTick`)ごとに `rate` フレームを
    /// [`timeline_pane::work_area::advanced_playhead`] で進めるだけ。
    /// `transport` と同格の表示/再生専用状態(undo に乗らない)。実時間
    /// transport とは相互排他([`Shell::apply_shuttle`] 参照)。
    shuttle: timeline_pane::ShuttleState,
    /// Stage ギズモ drag の shell 側 transient(GZ 結線 — [`GizmoShellDrag`]
    /// doc 参照)。`inspector_drag` と同格。
    gizmo_drag: Option<GizmoShellDrag>,
    /// Timeline マーカーレーンの drag 進行中状態(第6波、
    /// `timeline::markers::MarkerDrag` doc)。`gizmo_drag` と同格 — 現状は
    /// canvas 側(pub(crate))から `MarkerMessage::Grabbed` を publish する道が
    /// 無い(RETURN 参照)ため、実際には `None` のまま推移するが、意味の
    /// 配線(`Shell::update_marker`)は完結させてある。
    marker_drag: Option<timeline::markers::MarkerDrag>,
    /// Media 素材の実寸 cache(path → probe 結果、GZ 結線)。ギズモの
    /// [`stage::GizmoTarget::size`] は「Document が寸法を知らない素材は
    /// 呼び出し側が実寸を渡す」契約 — engine の texture 実寸は公開 API が
    /// 無いため、同じ実寸源(`motolii_media::probe`、engine も ffprobe 系で
    /// 実寸を得る)を path ごとに1回だけ叩いて控える。`view(&self)` から
    /// 読むため `RefCell`(表示専用 cache の interior mutability — Document
    /// でも Session でもない)。probe 失敗も `None` で控える(失敗する path を
    /// 毎フレーム叩き直さない)。
    media_size_cache: RefCell<HashMap<String, Option<[f32; 2]>>>,

    // ---- File 束(MB-1、裁定176) ----
    /// OS 副作用の注入口(`file_dialogs.rs` 冒頭 doc 参照)。production は
    /// `Shell::new()` が [`RfdDialogs`] を渡す。test は
    /// `Shell::new_with_dialogs` へ缶詰応答の fake を渡す。
    dialogs: Box<dyn FileDialogs>,
    /// 直近の Save As が書いた path。**Save a Copy では更新しない**
    /// (`Message::SaveACopyRequested` doc 参照 — 「現 path 維持のまま別名へ
    /// 書く」)。New Project でリセットされる。
    current_path: Option<std::path::PathBuf>,
    /// 素材の在り処の解決結果(`AssetId` → `AssetStatus`)。
    /// `Asset::status` は保存されない(環境の事実であって作品の内容ではない)ので
    /// shell が持つ。更新は離散イベントのときだけ — `sweep_asset_status()` 参照。
    asset_status: std::collections::HashMap<motolii_store::AssetId, motolii_store::AssetStatus>,
    /// 直近の保存(Save As)時点の `Document::revision()`。**dirty 判定の唯一の
    /// 鍵**(`Shell::is_dirty` 参照)── `revision()` は履歴の意味だけを表す
    /// (transient overlay は含まない、`document.rs::Revision` doc)ので、
    /// drag 中の途中経過だけで dirty が揺れることはない。
    saved_revision: Revision,
    /// 起動時に見つかった、本体ファイルより新しい autosave 世代の path。
    /// `Message::AutoSaveRecoveryConfirmed` を待っている間だけ `Some`
    /// (`document_io::recoverable_autosave` が起動時に一度だけ埋める)。
    /// 確認が届いたら(true/false どちらでも)`take()` して空にする。
    pending_recovery: Option<std::path::PathBuf>,

    // ---- 窓台帳(S1 daemon 骨格、裁定182/188) ----
    /// main 窓の Id(窓台帳: Id → 種別 の main 側)。**表示専用の front 状態**
    /// (`observation`/`clipboard` と同じ身分 — Document でも Session でもない)。
    /// `Shell::boot`/`boot_fixture` が boot 時に先行記帳する。`None` = 窓を
    /// 開いていない(headless 試験・`--screenshot` 一発ツール経路)。
    main_window: Option<iced::window::Id>,

    // ---- 第6波 shell 結線(2026-08-22 発注) ----
    /// Stage 方眼シート束のトグル状態(B22、`stage::sheets` doc「shell が
    /// Session 状態として持つ」)。**Document ではない** — `checkerboard` と
    /// 同格の視界状態(κ台帳 a型)。
    sheet_toggles: stage::SheetToggles,
    /// Export 窓の台帳(B09、Settings 窓(S2)と同じ型 — `settings_window` doc
    /// 参照)。`Some` = Export 窓が開いている。
    export_window: Option<iced::window::Id>,
    /// Export の品質選択(`export_pane::ExportQuality`)。窓を閉じても失われない
    /// (Settings の `background_draft` 等と同じ「窓の外に住む状態」)。
    export_quality: export_pane::ExportQuality,
    /// Export の範囲選択(`export_pane::ExportRange`)。同上。
    export_range: export_pane::ExportRange,
    /// Export 先 path。`None` = 未設定(Export ボタンは押せない、
    /// `export_pane::ViewModel::out_path` doc)。path 選択(rfd)は次波 — 見送り
    /// (`export_pane` crate doc の逸脱参照)。
    export_out_path: Option<std::path::PathBuf>,
    /// 実行中の進捗(`export_pane::ExportProgress`)。`None` = 実行していない。
    /// **型だけ**(`export_pane` crate doc「進捗の器」)—
    /// `motolii_export::export_with_cancel` はフレーム単位のコールバックを
    /// 持たない1回きりのバッチ呼び出しなので、実際に更新されるのは開始時
    /// (0/total)と完了時(total/total)の2点だけ(RETURN 参照)。
    export_progress: Option<export_pane::ExportProgress>,
    /// 実行中の `motolii_export::Cancel` ハンドル。`Message::Export` が
    /// export を始める時に発行し、`Message::Export(CancelExport)` が
    /// `.cancel()` を呼ぶ。
    export_cancel: Option<motolii_export::Cancel>,
}

// ---------------------------------------------------------------------------
// 連続量 drag(裁定217、E-5) — `value_drag` の press/move/release。書き込み
// 本体は既存の commit_* 自由関数をそのまま呼ぶ(`finish_value_drag` 参照)。
// ---------------------------------------------------------------------------

impl Shell {
    /// 値セルのキャプション press — click か drag かはまだ未確定
    /// (`ValueDragState::origin_x` が `None` のまま、`start_field_drag` と
    /// 同じ形)。対応する値が読めない(comp が無い・選択レイヤが無い等)なら
    /// 黙って無視 — drag は始まらない。既に別の drag が進行中なら多重起動しない。
    fn start_value_drag(&mut self, target: ValueDragTarget) {
        if self.value_drag.is_some() {
            return;
        }
        let Some(start_value) = self.value_drag_start_value(target) else {
            return;
        };
        self.value_drag = Some(ValueDragState {
            target,
            start_value,
            origin_x: None,
            moved: false,
        });
    }

    /// press 時点の「表示単位」の現在値。`None` なら drag を始めない
    /// (`inspector_pane::drag_origin` と同じ「投影に無ければ何もしない」形)。
    fn value_drag_start_value(&self, target: ValueDragTarget) -> Option<f64> {
        match target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => {
                let composition = self.doc.view().composition().ok().flatten()?;
                Some(match target {
                    ValueDragTarget::CompWidth => f64::from(composition.width),
                    ValueDragTarget::CompHeight => f64::from(composition.height),
                    ValueDragTarget::CompFps => composition.fps.as_f64(),
                    ValueDragTarget::CompDuration => composition.duration_frames as f64,
                    _ => unreachable!("上の match arm が尽くす"),
                })
            }
            ValueDragTarget::AutoSaveIntervalMinutes => {
                Some(self.auto_save_config.interval_secs as f64 / 60.0)
            }
            ValueDragTarget::AutoSaveGenerations => Some(self.auto_save_config.generations as f64),
            ValueDragTarget::Background(channel) => {
                let composition = self.doc.view().composition().ok().flatten()?;
                Some(f64::from(composition.background[channel.index()]) * 255.0)
            }
            ValueDragTarget::Color(color_target, channel) => {
                let layer = self.session.selection?;
                let current = self.doc.view().text_document(layer).ok()?;
                let document = current.unwrap_or_else(inspector_pane::default_text_document);
                let style = document
                    .styles
                    .first()
                    .cloned()
                    .unwrap_or_else(inspector_pane::default_text_style);
                let rgba = inspector_pane::color::text_style_color(&style, color_target);
                Some(rgba[channel.index()] * 255.0)
            }
        }
    }

    /// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op
    /// (`continue_field_drag` と同じ形)。**既存の draft へ書き戻すだけ** —
    /// text_input はその draft から表示を読むので、Enter 編集中と同じ見た目で
    /// 値が動く(Document へは release まで一切触らない、`FieldDragState` の
    /// 「transient overlay」に相当する部分をこの家系では draft が兼ねる)。
    fn continue_value_drag(&mut self, point: iced::Point) {
        let Some(state) = self.value_drag.as_mut() else {
            return;
        };
        let Some(origin_x) = state.origin_x else {
            state.origin_x = Some(point.x);
            return;
        };
        let delta_px = point.x - origin_x;
        if delta_px == 0.0 && !state.moved {
            return;
        }
        let target = state.target;
        let start_value = state.start_value;
        let fine = self.keyboard_modifiers.shift();
        let factor = if fine { inspector_pane::DRAG_SHIFT_FACTOR } else { 1.0 };
        let raw = start_value + f64::from(delta_px) * value_drag_step_per_pixel(target) * factor;
        self.write_value_drag_draft(target, raw);
        if let Some(state) = self.value_drag.as_mut() {
            state.moved = true;
        }
    }

    /// drag 中の draft 書き戻し。既存の表示関数(`comp_field_display`/
    /// `auto_save_field_display`/`color_channel_display` 等)をそのまま呼び、
    /// クランプは既存の `parse_*`/定数をそのまま使う(**別の式を発明しない**、
    /// 裁定215)。Background の8bit整形だけは `motolii_settings_pane::
    /// channel_cell` の私有ローカル計算と同じ1行を独立に持つ(pub で公開
    /// されていないため — 式自体は既存の1行の転記)。
    fn write_value_drag_draft(&mut self, target: ValueDragTarget, raw: f64) {
        use settings_pane::sections::{self, AutoSaveField, CompField};
        match target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => {
                let Ok(Some(mut composition)) = self.doc.view().composition() else {
                    return;
                };
                let field = match target {
                    ValueDragTarget::CompWidth => CompField::Width,
                    ValueDragTarget::CompHeight => CompField::Height,
                    ValueDragTarget::CompFps => CompField::Fps,
                    ValueDragTarget::CompDuration => CompField::DurationFrames,
                    _ => unreachable!("上の match arm が尽くす"),
                };
                match field {
                    // 下限1・上限 MAX_COMP_DIMENSION_PX(`parse_comp_dimension` と
                    // 同じクランプ — 0px/負の comp を drag では作らせない、裁定217
                    // 「判断が割れたら厳しい側」)。
                    CompField::Width => {
                        composition.width =
                            raw.round().clamp(1.0, f64::from(sections::MAX_COMP_DIMENSION_PX)) as u32;
                    }
                    CompField::Height => {
                        composition.height =
                            raw.round().clamp(1.0, f64::from(sections::MAX_COMP_DIMENSION_PX)) as u32;
                    }
                    CompField::Fps => {
                        let clamped = raw.clamp(1.0, sections::MAX_COMP_FPS);
                        let per_mille = (clamped * 1000.0).round() as i64;
                        if let Ok(fps) = Fps::try_new(per_mille, 1000) {
                            composition.fps = fps;
                        }
                    }
                    CompField::DurationFrames => {
                        composition.duration_frames = raw
                            .round()
                            .clamp(1.0, sections::MAX_COMP_DURATION_FRAMES as f64)
                            as i64;
                    }
                }
                let text = sections::comp_field_display(field, &composition);
                self.comp_draft = Some(sections::CompFieldDraft { field, text });
            }
            ValueDragTarget::AutoSaveIntervalMinutes | ValueDragTarget::AutoSaveGenerations => {
                let mut config = self.auto_save_config;
                let field = match target {
                    ValueDragTarget::AutoSaveIntervalMinutes => AutoSaveField::IntervalMinutes,
                    ValueDragTarget::AutoSaveGenerations => AutoSaveField::Generations,
                    _ => unreachable!("上の match arm が尽くす"),
                };
                match field {
                    AutoSaveField::IntervalMinutes => {
                        let clamped_minutes = raw.clamp(
                            sections::MIN_AUTO_SAVE_INTERVAL_MINUTES,
                            sections::MAX_AUTO_SAVE_INTERVAL_MINUTES,
                        );
                        config.interval_secs = (clamped_minutes * 60.0).round() as u64;
                    }
                    AutoSaveField::Generations => {
                        config.generations = raw
                            .round()
                            .clamp(1.0, sections::MAX_AUTO_SAVE_GENERATIONS as f64)
                            as usize;
                    }
                }
                let text = sections::auto_save_field_display(field, &config);
                self.auto_save_draft = Some(sections::AutoSaveFieldDraft { field, text });
            }
            ValueDragTarget::Background(channel) => {
                // comp が無ければ投影も無い(`comp_field_cell`/`channel_cell` と
                // 同じ柵) — 値自体は `raw` から直接組む(`Composition` は
                // read-modify-write の確定側(`finish_value_drag` →
                // `commit_background_channel`)でのみ触るので、ここで読んだ
                // 値をそのまま書き戻す必要は無い)。
                if self.doc.view().composition().ok().flatten().is_none() {
                    return;
                }
                let clamped = raw.clamp(0.0, 255.0);
                // `motolii_settings_pane::lib.rs::channel_cell` の `current_u8`
                // 計算と同じ1行(`round().clamp(0,255) as u32 → to_string()`)。
                let text = (clamped.round() as u32).to_string();
                self.background_draft = Some(BackgroundFieldDraft { channel, text });
            }
            ValueDragTarget::Color(color_target, channel) => {
                let Some(layer) = self.session.selection else {
                    return;
                };
                let Ok(current) = self.doc.view().text_document(layer) else {
                    return;
                };
                let document = current.unwrap_or_else(inspector_pane::default_text_document);
                let mut style = document
                    .styles
                    .first()
                    .cloned()
                    .unwrap_or_else(inspector_pane::default_text_style);
                let clamped = raw.clamp(0.0, 255.0);
                inspector_pane::color::set_text_style_color_channel(
                    &mut style,
                    color_target,
                    channel,
                    clamped / 255.0,
                );
                let text = inspector_pane::color::color_channel_display(&style, color_target, channel);
                self.inspector_color_field_draft = Some(inspector_pane::color::ColorFieldDraft {
                    target: color_target,
                    channel,
                    text,
                });
            }
        }
    }

    /// 左クリック release(window 全体から)。**drag が実際に動いていたら確定**
    /// — drag 中に書き戻した draft を、既存の Enter 確定と**同じ commit_*
    /// 自由関数**へそのまま渡す(1 gesture = 1 undo、書き込みロジックの複製
    /// ゼロ)。動いていなければ(click)何もしない — draft は press 時点で
    /// まだ触っていないので、そのまま text_input への通常の click→type 編集に
    /// 委ねる。
    fn finish_value_drag(&mut self) {
        let Some(state) = self.value_drag.take() else {
            return;
        };
        if !state.moved {
            return;
        }
        match state.target {
            ValueDragTarget::CompWidth
            | ValueDragTarget::CompHeight
            | ValueDragTarget::CompFps
            | ValueDragTarget::CompDuration => {
                let field = match state.target {
                    ValueDragTarget::CompWidth => settings_pane::sections::CompField::Width,
                    ValueDragTarget::CompHeight => settings_pane::sections::CompField::Height,
                    ValueDragTarget::CompFps => settings_pane::sections::CompField::Fps,
                    ValueDragTarget::CompDuration => {
                        settings_pane::sections::CompField::DurationFrames
                    }
                    _ => unreachable!("上の match arm が尽くす"),
                };
                if let Err(error) =
                    settings_pane::sections::commit_comp_field(&mut self.doc, &mut self.comp_draft, field)
                {
                    self.status = Some(error);
                }
            }
            ValueDragTarget::AutoSaveIntervalMinutes | ValueDragTarget::AutoSaveGenerations => {
                let field = match state.target {
                    ValueDragTarget::AutoSaveIntervalMinutes => {
                        settings_pane::sections::AutoSaveField::IntervalMinutes
                    }
                    ValueDragTarget::AutoSaveGenerations => {
                        settings_pane::sections::AutoSaveField::Generations
                    }
                    _ => unreachable!("上の match arm が尽くす"),
                };
                if let Err(error) = settings_pane::sections::commit_auto_save_field(
                    &mut self.auto_save_config,
                    &mut self.auto_save_draft,
                    field,
                ) {
                    self.status = Some(error);
                }
            }
            ValueDragTarget::Background(channel) => {
                if let Err(error) = settings_pane::commit_background_channel(
                    &mut self.doc,
                    &mut self.background_draft,
                    channel,
                ) {
                    self.status = Some(error);
                }
            }
            ValueDragTarget::Color(color_target, channel) => {
                if let Err(error) = inspector_pane::color::commit_text_style_color(
                    &mut self.doc,
                    &mut self.inspector_color_field_draft,
                    self.session.selection,
                    color_target,
                    channel,
                ) {
                    self.status = Some(error);
                }
            }
        }
    }
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
                inspector_text_field_draft: None,
                inspector_color_field_draft: None,
                inspector_content_editor: iced::widget::text_editor::Content::new(),
                inspector_content_editor_layer: None,
                inspector_drag: None,
                inspector_text_style_drag: None,
                value_drag: None,
                keyboard_modifiers: iced::keyboard::Modifiers::default(),
                layer_selection_anchor: None,
                timeline: timeline_pane::PaneState::new(),
                browser: browser_pane::PaneState::new(),
                panes: pane_layout::Layout::new(),
                settings_window: None,
                checkerboard: false,
                background_draft: None,
                ui_scale_draft: None,
                comp_draft: None,
                auto_save_enabled: true,
                auto_save_config: AutoSaveConfig::default(),
                auto_save_draft: None,
                last_auto_saved: saved_revision.clone(),
                observation: None,
                resolution_cap: stage::PreviewResolutionCap::default(),
                clipboard: clipboard::Clipboard::default(),
                transport: Transport::new(),
                shuttle: timeline_pane::ShuttleState::stopped(),
                gizmo_drag: None,
                marker_drag: None,
                media_size_cache: RefCell::new(HashMap::new()),
                dialogs,
                current_path: None,
            asset_status: std::collections::HashMap::new(),
                saved_revision,
                pending_recovery: None,
                main_window: None,
                sheet_toggles: stage::SheetToggles::default(),
                export_window: None,
                export_quality: export_pane::ExportQuality::Normal,
                export_range: export_pane::ExportRange::Whole,
                export_out_path: None,
                export_progress: None,
                export_cancel: None,
            },
            Task::none(),
        )
    }

    // ---- daemon boot(S1、裁定182/188) ----

    /// daemon の製品入口(`main.rs`)。[`Shell::new`] で組んだ Shell に main 窓を
    /// 1枚開く Task を添える([`iced::daemon`] は自分では窓を開かない)。
    /// 窓台帳(`main_window`)は open Task の完了を**待たずに先行記帳**する —
    /// `iced::window::open` は Id を同期で採番する(fork
    /// `runtime/src/window.rs:260`)ので、runtime 無しの headless 試験でも
    /// 台帳が読める。
    pub fn boot() -> (Self, Task<Message>) {
        let (shell, task) = Self::with_main_window(Self::new());
        // C-1 波C「再起動で続きが開く」: 前回プロジェクトの path を読む
        // だけの軽い I/O(小さな sidecar ファイル1本)なので同期でもよいが、
        // `Task::perform` に包む ── `tests/suite/window_drive.rs` は返って
        // 来た `Task` を一切 poll しない(`let (booted, _task) = Shell::boot();`
        // の形、`drain_task` を通さない)ので、この Task の中身は headless
        // 試験では実行されない = 開発機のホームディレクトリを試験が触らない
        // (production の runtime executor だけが実際に polling する)。
        let reopen = Task::perform(
            async { Self::read_last_project_path() },
            Message::LastProjectPathRead,
        );
        (shell, Task::batch([task, reopen]))
    }

    /// `--fixture` 起動の daemon boot([`Shell::boot`] の fixture 版)。
    pub fn boot_fixture() -> (Self, Task<Message>) {
        Self::with_main_window(Self::new_fixture())
    }

    /// [`Shell::boot`]/[`Shell::boot_fixture`] の共通部: main 窓を開く Task を
    /// 添え、台帳へ先行記帳する。窓の性質は従前の `iced::application` の既定
    /// (`window::Settings::default()`)からただ1点だけ変えてある —
    /// **`exit_on_close_request: false`**(C-1 波C「閉じる確認」)。既定
    /// (true)のままだと、winit fork(`winit/src/lib.rs:1031-1033`)は
    /// `CloseRequested` を一切アプリへ渡さず直接 `Action::Window(Close)` へ
    /// 変換する ── dirty ガードを挟む余地が無い(A06「OS の閉じるボタンが
    /// 未保存確認を飛ばす」の実測どおり)。`false` にすると
    /// `CloseRequested` がそのまま `Message`(`iced::window::close_requests`)
    /// として届くので、`Message::WindowCloseRequested` が dirty ガードを
    /// 挟んでから明示的に `iced::exit()` する形に変えられる(Cmd+Q の
    /// `confirm_then` と同型)。
    fn with_main_window((mut shell, task): (Self, Task<Message>)) -> (Self, Task<Message>) {
        let (id, open) = iced::window::open(iced::window::Settings {
            exit_on_close_request: false,
            ..iced::window::Settings::default()
        });
        shell.main_window = Some(id);
        (
            shell,
            Task::batch([task, open.map(Message::MainWindowOpened)]),
        )
    }

    /// 窓台帳の読み口(main 窓)。試験(`tests/suite/window_drive.rs`)が
    /// 台帳の記帳を検分するための口 — [`Shell::settings_window`] と同じ形。
    pub fn main_window(&self) -> Option<iced::window::Id> {
        self.main_window
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
            inspector_text_field_draft: None,
            inspector_color_field_draft: None,
            inspector_content_editor: iced::widget::text_editor::Content::new(),
            inspector_content_editor_layer: None,
            inspector_drag: None,
            inspector_text_style_drag: None,
            value_drag: None,
            keyboard_modifiers: iced::keyboard::Modifiers::default(),
            layer_selection_anchor: None,
            timeline: timeline_pane::PaneState::new(),
            browser: browser_pane::PaneState::new(),
            panes: pane_layout::Layout::new(),
            settings_window: None,
            checkerboard: false,
            background_draft: None,
            ui_scale_draft: None,
            comp_draft: None,
            auto_save_enabled: true,
            auto_save_config: AutoSaveConfig::default(),
            auto_save_draft: None,
            last_auto_saved: saved_revision.clone(),
            observation: None,
            resolution_cap: stage::PreviewResolutionCap::default(),
            clipboard: clipboard::Clipboard::default(),
            transport: Transport::new(),
            shuttle: timeline_pane::ShuttleState::stopped(),
            gizmo_drag: None,
            marker_drag: None,
            media_size_cache: RefCell::new(HashMap::new()),
            // 器具は screenshot 検分専用(発注書「トンマナ検分の器具」)なので
            // production の rfd ではなく`RfdDialogs` をそのまま渡しておく ──
            // 器具経路は `Message::NewProjectRequested` 等を一切発行しない
            // (`main.rs` の `--fixture` フラグ群を参照、File 束の Message は
            // 無い)ため実際に呼ばれることはない。
            dialogs: Box::new(RfdDialogs),
            current_path: None,
            asset_status: std::collections::HashMap::new(),
            saved_revision,
            pending_recovery: None,
            main_window: None,
            sheet_toggles: stage::SheetToggles::default(),
            export_window: None,
            export_quality: export_pane::ExportQuality::Normal,
            export_range: export_pane::ExportRange::Whole,
            export_out_path: None,
            export_progress: None,
            export_cancel: None,
        };
        // `update()` を経由しないので、通常なら `update` の末尾が呼ぶ
        // `refresh_frame` をここで代わりに呼ぶ(Stage を空のまま起動しない、M17)。
        shell.refresh_frame();
        (shell, Task::none())
    }

    /// main 窓の titlebar 文言(C-1 波C「未保存●が無い」の穴を塞ぐ)。
    /// **先例**: VS Code/Sublime Text は「`● filename`」(ファイル名の前に
    /// 点)、AE/Premiere/Figma は「`filename*`」(末尾にアスタリスク)—
    /// どちらも「未保存の変更がある」を常設のテキストで示す(ダイアログや
    /// 別ボタンを開かせない、裁定185「説明は下部バーへ」と同じ「常設で
    /// 読める」思想)。ここは前者(先頭の点)を採る ── ファイル名は右側が
    /// 長くなりがちで、点を先頭に固定した方が窓が狭くても消えない。
    /// `current_path` が無い(一度も保存していない新規 project)は
    /// "Untitled" とする(先例: 全4製品共通)。
    pub fn title(&self) -> String {
        let name = self
            .current_path
            .as_deref()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled");
        if self.is_dirty() {
            format!("• {name} — Motolii")
        } else {
            format!("{name} — Motolii")
        }
    }

    /// 窓の事象 → Message。**ここは翻訳だけで、判断を持たない**。
    pub fn subscription(&self) -> iced::Subscription<Message> {
        let window = iced::window::events().map(|(_id, event)| match event {
            iced::window::Event::FileDropped(path) => Message::DropReceived(path),
            // 第6波(B08 取り込み UX 結線): OS file-drag の入/出をそのまま
            // `browser_pane::state::Message::DropHoverChanged` へ翻訳する
            // (`browser_pane` crate 冒頭 doc「shell 結線(次波)」— この波で
            // 結線)。真偽の意味は pane 側で完結するので、ここは翻訳だけ。
            iced::window::Event::FileHovered(_) => {
                Message::Browser(browser_pane::Message::DropHoverChanged(true))
            }
            iced::window::Event::FilesHoveredLeft => {
                Message::Browser(browser_pane::Message::DropHoverChanged(false))
            }
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
        // JKL シャトル(B21、第5波結線)も同じ tick に乗る — シャトルは実時間
        // clock を持たない(1 tick = `rate` フレーム、`advance_playback_tick`)
        // ので、走っている間だけ購読が要るのは transport と同型。
        let ticks = if self.transport.is_running() || !self.shuttle.is_stopped() {
            transport::tick_subscription().map(|()| Message::PlaybackTick)
        } else {
            iced::Subscription::none()
        };
        // AUTOSAVE(SET+ B12 第2切片): `auto_save_enabled` の間だけ tick を
        // 束ねる(`ticks` と同じ「無効ならそもそも購読しない」形)。実際の
        // dirty 判定・再生中/ドラッグ中のスキップは `Shell::run_auto_save`
        // (`Message::AutoSaveTick` の受け口)が持つ — ここは翻訳だけ。
        let auto_save = if self.auto_save_enabled {
            auto_save::tick_subscription(self.auto_save_config.interval_secs)
                .map(|()| Message::AutoSaveTick)
        } else {
            iced::Subscription::none()
        };
        // 窓台帳(S1 daemon 骨格): どの窓が閉じたかを台帳へ届ける。daemon は
        // 窓が全部閉じても自分では終了しない(fork `src/daemon.rs` doc)ので、
        // 「main 閉=exit」の判断を `Shell::update`(`Message::WindowClosed`)が
        // 持つ — ここは規律どおり翻訳(map)だけ。
        let closes = iced::window::close_events().map(Message::WindowClosed);
        // C-1 波C「閉じる確認」: main 窓は `exit_on_close_request: false`
        // (`Shell::with_main_window`)で開くので、赤信号ボタンは `Closed` では
        // なく `CloseRequested` を発行する ── ここを拾って dirty ガード
        // (`Message::WindowCloseRequested` 腕)へ渡す。Settings/Export 窓は
        // 既定のままなのでこの Subscription からは届かない。
        let close_requests =
            iced::window::close_requests().map(Message::WindowCloseRequested);
        iced::Subscription::batch([window, tokens, pointer, ticks, auto_save, closes, close_requests])
    }

    /// Timeline rail の layer 行クリック(E-2、軸台帳 A08 隣接の穴)。
    /// **裸クリック=単独選択・Cmd=トグル(足し引き)・Shift=範囲**
    /// (`timeline_pane::rows::LayerSelectionOp` の3形そのまま)。解決自体は
    /// [`timeline_pane::rows::resolve_layer_selection`](純関数、`Session` を
    /// 書き換えない)へ委譲し、確定は必ず [`Self::set_selected_layers`]
    /// (C-2 の唯一の書き手)を経由する — この関数の外で `session.selection`/
    /// `selected_layers` を直接書き換えない。
    ///
    /// `order`(範囲の基準)は今 rail に見えている行(`timeline_pane::rows`、
    /// 畳まれて非表示の行は対象外 — `key_order` と同じ「見えているものだけ」
    /// の姿勢、`resolve_layer_selection` doc 参照)。`anchor` は `Session` では
    /// なく `Shell::layer_selection_anchor`(このレーンの write-set は
    /// `lib.rs`/`input.rs`/`timeline-pane::write.rs` のみ — `selection.rs` は
    /// `set_selected_layers` を呼ぶだけで書き換えないため、anchor はここに置く)。
    fn click_select_layer(&mut self, layer: LayerId) {
        let order: Vec<LayerId> =
            timeline_pane::rows(&self.doc.view(), &self.session).into_iter().map(|row| row.id).collect();
        let op = if self.keyboard_modifiers.command() {
            timeline_pane::rows::LayerSelectionOp::Toggle(layer)
        } else if self.keyboard_modifiers.shift() {
            timeline_pane::rows::LayerSelectionOp::Range(layer)
        } else {
            timeline_pane::rows::LayerSelectionOp::Single(layer)
        };
        let (selected, anchor) = timeline_pane::rows::resolve_layer_selection(
            &order,
            self.layer_selection_anchor,
            &self.session.selected_layers,
            op,
        );
        self.layer_selection_anchor = anchor;
        self.set_selected_layers(selected);
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
            // ---- 窓台帳(S1 daemon 骨格 + S2 Settings 窓、裁定182/188) ----
            Message::MainWindowOpened(id) => self.main_window = Some(id),
            Message::SettingsWindowOpened(id) => self.settings_window = Some(id),
            Message::ExportWindowOpened(id) => self.export_window = Some(id),
            Message::WindowClosed(id) => {
                if self.main_window == Some(id) {
                    // main 閉=アプリ終了(probe 注意点1)。daemon は放って
                    // おくと窓ゼロで生き続け、winit shell が compositor を
                    // `None` 化(device 破棄)する — zero-copy presenter
                    // (裁定170/171)の単一 device 前提を守るため、窓ゼロ状態
                    // そのものを作らない。
                    task = iced::exit();
                } else if self.settings_window == Some(id) {
                    // OS の閉じるボタン経路(トグル経由の close は
                    // `toggle_settings_window` が先行抹消済み — その場合
                    // ここへ来る時点で台帳は既に None なので何もしない)。
                    self.settings_window = None;
                } else if self.export_window == Some(id) {
                    // Export 窓の OS 閉じるボタン経路(`toggle_export_window`
                    // と同じ扱い、Settings 窓と同型)。
                    self.export_window = None;
                }
            }
            Message::Inspector(msg) => {
                // 裁定217 連続量 drag 化(E-5)。`self.value_drag`(track を
                // 持たない値向けの第2の経路、struct 冒頭 doc 参照)は
                // Inspector の `inspector_drag`/`inspector_text_style_drag` と
                // 同じ window 全体購読(`PointerMoved`/`PointerReleased`)を
                // 共有する — `inspector_ops.rs::update_inspector` を触らずに
                // 済むよう、ここで先取りして両方へ配る(`inspector_drag`/
                // `inspector_text_style_drag` 自身は `update_inspector` 側で
                // 従来どおり動く、片方が `None` の間は他方も no-op なので
                // 二重発火しても無害)。
                match &msg {
                    inspector_pane::Message::PointerMoved(point) => self.continue_value_drag(*point),
                    inspector_pane::Message::PointerReleased => self.finish_value_drag(),
                    _ => {}
                }
                match msg {
                    // 色欄(Fill/Stroke RGBA)のキャプション press。`color::Message`
                    // に3つ目の variant を足したことで `inspector_ops.rs::
                    // update_inspector` の `Message::Color(...)` 網羅 match が
                    // 非網羅になるため、そちらへも1腕(no-op)を足した
                    // (RETURN「lib.rs と input.rs に追加/変更した行」参照 —
                    // ここで先取りするので実際にはそちらへは来ない)。
                    inspector_pane::Message::Color(inspector_pane::color::Message::ChannelDragPressed(
                        target,
                        channel,
                    )) => {
                        self.start_value_drag(ValueDragTarget::Color(target, channel));
                    }
                    other => task = self.update_inspector(other),
                }
            }
            // pane split survey §3.2 exception 1/裁定160 切片7: `Select`/
            // `ScrubTo` は本来 core 腕、`ToggleMute`/`ToggleSolo`/`ToggleLock`
            // は `toggle_layer_hidden` が Inspector とも共有する Shell 側の
            // ヘルパーのため、この5腕だけ `timeline_pane::PaneState::update`
            // へ渡す前に Shell が先取りする(`timeline_pane::write` モジュール
            // doc 参照)。残りは pane 側の唯一の書き口(`PaneState::update`)へ
            // 委譲する — 拒否理由があれば `self.status` へそのまま渡す。
            Message::Timeline(msg) => match msg {
                timeline_pane::Message::Select(layer) => self.click_select_layer(layer),
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
                // JKL シャトル(B21、第5波結線)— transport 4腕と同じ「shell
                // 先取りの例外」(`timeline_pane::Message::Shuttle` doc): 実時間
                // 再生の clock は shell(A2)が持つので、状態遷移と tick 駆動を
                // ここで畳む(`PaneState::update` では no-op)。
                timeline_pane::Message::Shuttle(command) => self.apply_shuttle(command),
                // ルーラ locator lane 右クリック(S2 発注 #22「マーカー追加
                // UI が無い」の穴埋め、2入口目)— キーボード M
                // (`Message::Marker(MarkerMessage::AddAtPlayhead)`)と同じ
                // `update_marker` 経路へ畳む(S6 併存、裁定195)。
                timeline_pane::Message::AddMarkerAt(frame) => {
                    self.update_marker(timeline::markers::MarkerMessage::AddAtFrame(frame))
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
            // map 1064(B18、第5波結線): 作業範囲の先頭/末尾へ。範囲が無ければ
            // no-op(`jump_clip_edge` と同じ「跳ぶ先が無ければ動かない」)。
            Message::JumpToWorkAreaStart => {
                if let Some(area) = self.timeline.work_area() {
                    self.session.playhead = area.first_frame();
                }
            }
            Message::JumpToWorkAreaEnd => {
                if let Some(area) = self.timeline.work_area() {
                    self.session.playhead = area.last_frame();
                }
            }
            Message::KeyboardModifiersChanged(modifiers) => self.keyboard_modifiers = modifiers,
            // Esc は Timeline ドラッグを優先してキャンセルする(clip → key →
            // ループ帯 → gizmo の順、どれも掴んでいなければ Inspector 側
            // (drag/typing 下書き)を試す — 同時に成立するのは1つだけなので
            // 順序自体に意味は無い、排他)。ループ帯は捨てるだけでは戻らない
            // (live 更新)ので `cancel_loop_drag` が origin を書き戻す
            // (裁定151「キャンセルの一般化」の柵、B18 の supervisor 結線)。
            // gizmo は canvas 側も Esc で `GizmoPhase::Cancel` を publish する
            // (`gizmo.rs`)が、こちらの連鎖にも置く — どちらが先でも
            // `cancel_gizmo_drag` は冪等。第6波: rename も同じ連鎖へ足す
            // (`timeline::write` 冒頭 doc「Esc は shell の EscapePressed が
            // cancel_rename を直接呼ぶ」)— rename 中は drag 状態と排他なので
            // 挿し込み位置に意味は無い(既存コメントと同じ理由)。
            Message::EscapePressed => {
                if !self.timeline.cancel_drag()
                    && !self.timeline.cancel_key_drag()
                    && !self.timeline.cancel_loop_drag()
                    && !self.timeline.cancel_rename()
                    && !self.cancel_gizmo_drag()
                {
                    self.cancel_inspector_interaction();
                }
            }
            Message::Settings(msg) => task = self.update_settings(msg),
            Message::Stage(msg) => self.update_stage(msg),
            Message::Gizmo(event) => self.update_gizmo(event),
            Message::ZoomIn => self.zoom_in(),
            Message::ZoomOut => self.zoom_out(),
            Message::ZoomToFit => self.zoom_to_fit(),
            // B2/B3: rail scope 選択/検索欄/Clear/ToggleBrowserPanel の4腕
            // (`browser_pane::Message`)を pane 側の唯一の書き口
            // (`PaneState::update`)へそのまま委譲する(`timeline_pane::
            // PaneState::update` への委譲と同型)。Document/Session を一切
            // 触らない pane-local 状態なので `&mut self.browser` だけで完結
            // する(引数を追加で貸す必要が無い、`browser_pane::state` crate
            // doc 参照)。
            Message::Browser(msg) => {
                // **畳んだ口**(MC-1、2026-08-23、`create.rs::
                // dispatch_browser_card_intent` doc 参照)。カード発の意図
                // (`CreateFromCard`/`AddMaskFromCard`/`ApplyEffectFromCard`/
                // `ReplaceSelectedLayerSource`/`RemoveAssetFromCard`)を
                // ここで1つずつ `if let` で横取りしていた5本の分岐は、
                // 1関数呼び出しへ畳んだ——pane側は元から no-op(`state.rs`の
                // ORACLE)なので、`&msg` を渡して先に処理しても
                // `self.browser.update(msg)` との二重処理にはならない。
                // カードの意図がもう1種類増えても、この行は変えず
                // `create.rs` の match へ腕を1本足すだけで済む
                // (write-set が `lib.rs` を引きずらなくなる)。
                self.dispatch_browser_card_intent(&msg);
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
            Message::DeleteSelectedLayers => self.delete_selected_layers(),
            Message::HideSelectedLayers => self.hide_selected_layers(),
            Message::SoloSelectedLayers => self.solo_selected_layers(),
            Message::LockSelectedLayers => self.lock_selected_layers(),
            Message::GroupLayers => self.group_selected_layers(),
            Message::UngroupLayers => self.ungroup_selected_layers(),
            // MB-2: freeze 意図動詞(裁定119)の UI 初露出(Layer メニュー)。
            Message::FreezeGroups => self.set_selected_groups_frozen(true),
            Message::UnfreezeGroups => self.set_selected_groups_frozen(false),
            Message::NewProjectRequested => task = self.confirm_then(Message::NewProjectConfirmed),
            Message::NewProjectConfirmed(confirmed) => {
                if confirmed {
                    self.reset_document();
                }
            }
            Message::SaveRequested => {
                if let Some(path) = self.current_path.clone() {
                    self.perform_save_as(path);
                } else {
                    task = Task::perform(self.dialogs.pick_save_path(), Message::SaveAsPathChosen);
                }
            }
            Message::SaveAsRequested => {
                task = Task::perform(self.dialogs.pick_save_path(), Message::SaveAsPathChosen);
            }
            Message::SaveAsPathChosen(Some(path)) => self.perform_save_as(path),
            Message::SaveAsPathChosen(None) => {}
            Message::SaveACopyRequested => {
                task = Task::perform(self.dialogs.pick_save_path(), Message::SaveACopyPathChosen);
            }
            Message::SaveACopyPathChosen(Some(path)) => self.perform_save_a_copy(path),
            Message::SaveACopyPathChosen(None) => {}
            Message::OpenRequested => task = self.confirm_then_pick_open(),
            Message::OpenPathChosen(Some(path)) => self.perform_open(path),
            Message::OpenPathChosen(None) => {}
            Message::ImportMediaRequested => {
                task = Task::perform(self.dialogs.pick_import_paths(), Message::AdmitPaths);
            }
            Message::QuitRequested => task = self.confirm_then(Message::QuitConfirmed),
            Message::QuitConfirmed(confirmed) => {
                if confirmed {
                    self.dialogs.quit();
                }
            }
            Message::WindowCloseRequested(id) => {
                if self.main_window == Some(id) {
                    task = self.confirm_then(Message::WindowCloseConfirmed);
                }
                // Settings/Export 窓は `exit_on_close_request: true`(既定)の
                // ままなので、この Message は main 以外の id では実際には
                // 届かない(防御的に無視するだけ)。
            }
            Message::WindowCloseConfirmed(confirmed) => {
                if confirmed {
                    task = iced::exit();
                }
                // false: 何もしない。main 窓は `exit_on_close_request: false`
                // のおかげでまだ閉じていない(OS へ Close を送っていない)ので、
                // 見た目どおり編集を続けられる。
            }
            Message::LastProjectPathRead(Some(path)) => {
                match Document::load(&path) {
                    Ok(doc) => {
                        self.saved_revision = doc.revision();
                        self.last_auto_saved = self.saved_revision.clone();
                        self.doc = doc;
                        self.current_path = Some(path.clone());
                        self.session = Session::default();
                    }
                    // 拒否は必ず出す(M13)。既定 Document のまま起動を続ける
                    // ── 黙って上書きしない/黙って落とさない、どちらも避ける。
                    Err(error) => {
                        self.status = Some(format!("前回のプロジェクトを開けない: {error}"));
                    }
                }
                if let Some(autosave_path) = Self::recoverable_autosave(&path) {
                    self.pending_recovery = Some(autosave_path);
                    task = Task::perform(
                        self.dialogs.confirm_recover_autosave(),
                        Message::AutoSaveRecoveryConfirmed,
                    );
                }
            }
            Message::LastProjectPathRead(None) => {}
            Message::AutoSaveRecoveryConfirmed(confirmed) => {
                if let Some(autosave_path) = self.pending_recovery.take() {
                    if confirmed {
                        self.perform_recover_autosave(autosave_path);
                    }
                }
            }
            Message::TogglePlayback => self.toggle_playback(),
            Message::PlaybackTick => self.advance_playback_tick(),
            Message::AutoSaveTick => self.run_auto_save(),

            // ---- 第6波 shell 結線 ----
            Message::Sheet(msg) => self.sheet_toggles = self.sheet_toggles.apply(msg),
            Message::Marquee(select) => {
                let next = stage::marquee::apply_selection(
                    &self.session.selected_layers,
                    &select.ids,
                    select.additive,
                );
                self.apply_stage_selection(next);
            }
            Message::Marker(msg) => self.update_marker(msg),
            Message::Export(msg) => task = self.update_export(msg),
            Message::ExportProgressed(event) => self.update_export_progressed(event),
            Message::RenameSelectedLayer => {
                if let Some(layer) = self.session.selection {
                    if let Some(reason) = self.timeline.update(
                        timeline_pane::Message::RenameBegin(layer),
                        &mut self.doc,
                        &mut self.session,
                        self.keyboard_modifiers,
                    ) {
                        self.status = Some(reason);
                    }
                }
            }
        }
        self.refresh_frame();
        // S4(#46 の穴塞ぎ): Content 行の永続 `text_editor::Content` を選択と
        // 同期する(`inspector_ops::sync_inspector_content_editor` doc 参照)。
        // 上のどの腕が選択を動かしても、ここで必ず1回チェックが通る
        // (`self.session.selection` は上の match でもう更新済みの値)。
        self.sync_inspector_content_editor();
        Task::batch([task, self.poll_waveform_fetches()])
    }






    /// 今の playhead を comp の fps で時刻へ写す。comp が無い/fps が壊れているなら
    /// `None`(M16: panic しない)。
    fn time_at_playhead(&self) -> Option<RationalTime> {
        let composition = self.doc.view().composition().ok().flatten()?;
        RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()
    }



    // ---- Settings パネル(タスク#18、裁定160 切片9) ----

    /// pane ローカル `Message`(SET+ の [`settings_pane::sections::Message`])を
    /// 畳んで書き口へ渡す glue。sections.rs 冒頭 doc「結線互換の縫い目」の手順
    /// 2そのもの: 新項目2腕(`CompFieldInput`/`CompFieldSubmit` —
    /// `commit_comp_field` が read-modify-write の `Intent::SetComposition` を
    /// 1回出す)+ 旧腕は [`Self::update_settings_legacy`] へ丸ごと委譲。
    fn update_settings(&mut self, message: settings_pane::sections::Message) -> Task<Message> {
        use settings_pane::sections;
        use sections::{AutoSaveField, CompField};
        match message {
            sections::Message::Legacy(legacy) => return self.update_settings_legacy(legacy),
            sections::Message::CompFieldInput(field, text) => {
                self.comp_draft = Some(sections::CompFieldDraft { field, text });
            }
            sections::Message::CompFieldSubmit(field) => {
                if let Err(error) =
                    sections::commit_comp_field(&mut self.doc, &mut self.comp_draft, field)
                {
                    self.status = Some(error);
                }
            }
            sections::Message::AutoSaveToggle(enabled) => {
                self.auto_save_enabled = enabled;
            }
            sections::Message::AutoSaveFieldInput(field, text) => {
                self.auto_save_draft = Some(sections::AutoSaveFieldDraft { field, text });
            }
            sections::Message::AutoSaveFieldSubmit(field) => {
                if let Err(error) = sections::commit_auto_save_field(
                    &mut self.auto_save_config,
                    &mut self.auto_save_draft,
                    field,
                ) {
                    self.status = Some(error);
                }
            }
            // 裁定217 連続量 drag 化(E-5)。`start_value_drag` と同じ
            // 「press だけ own する」形 — move/release は window 全体購読
            // (`inspector_pointer_event`)を Inspector と共有する。
            sections::Message::CompFieldDragPressed(field) => {
                self.start_value_drag(match field {
                    CompField::Width => ValueDragTarget::CompWidth,
                    CompField::Height => ValueDragTarget::CompHeight,
                    CompField::Fps => ValueDragTarget::CompFps,
                    CompField::DurationFrames => ValueDragTarget::CompDuration,
                });
            }
            sections::Message::AutoSaveFieldDragPressed(field) => {
                self.start_value_drag(match field {
                    AutoSaveField::IntervalMinutes => ValueDragTarget::AutoSaveIntervalMinutes,
                    AutoSaveField::Generations => ValueDragTarget::AutoSaveGenerations,
                });
            }
        }
        Task::none()
    }

    /// 旧 `settings_pane::Message` の腕(SET+ 以前の全項目)。write ロジックの
    /// 実体は `motolii_settings_pane::{apply_background_preset,
    /// commit_background_channel, commit_ui_scale}`(自由関数、`&mut Document`/
    /// `&mut Tokens`/下書きを明示引数で受け取る形 — pane crate は `&mut self` を
    /// 持てないため)。ここでは `self.doc`/`self.tokens`/下書きフィールドを
    /// そのまま貸すだけで、拒否理由(`Result::Err`)を `self.status` へ write
    /// する以外の判断は持たない。
    fn update_settings_legacy(&mut self, message: settings_pane::Message) -> Task<Message> {
        match message {
            settings_pane::Message::ToggleSettingsPanel => {
                // S2(裁定182/188): 意味が「レイアウト分岐」→「窓 open/close」
                // へ変わった(probe §Q3)。トグル以外の腕は従来どおり
                // Task を返さない。
                return self.toggle_settings_window();
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
            // 裁定217 連続量 drag 化(E-5)。`sections::Message::CompFieldDragPressed`
            // と同じ形。
            settings_pane::Message::BackgroundChannelDragPressed(channel) => {
                self.start_value_drag(ValueDragTarget::Background(channel));
            }
        }
        Task::none()
    }

    /// S2(裁定182/188): Settings の入口 — header の歯車が出す
    /// `ToggleSettingsPanel` を OS 窓の open/close へ配線する(浮かし第1号、
    /// 裁定188「Settings はだいたいポップアップだから」)。
    ///
    /// 台帳(`settings_window`)は**同期で先行記帳/先行抹消**する —
    /// `window::open` は Id を同期で採番し(fork `runtime/src/window.rs:260`)、
    /// close も「閉じるつもり」の時点で台帳から下ろす。runtime 無しの headless
    /// 試験(Task は走らない)でも open/close/再open の状態遷移が読めるのは
    /// この設計のため(`tests/suite/window_drive.rs` の oracle)。OS の閉じる
    /// ボタン経由は `Message::WindowClosed`(`close_events` 購読)が同じ抹消を
    /// 行う。
    fn toggle_settings_window(&mut self) -> Task<Message> {
        match self.settings_window.take() {
            Some(id) => iced::window::close(id),
            None => {
                let (id, open) = iced::window::open(iced::window::Settings {
                    // 小さめ・リサイズ可(発注どおり、probe 実証の形)。raw 値は
                    // pane の意匠値ではなく**窓の初期ジオメトリ**(トンマナ柵
                    // (裁定142)の対象マーカー外 — `Size::new` は widget 構築
                    // 呼び出しではない): 幅はプリセット4ボタン+数値欄が
                    // 折り返さない程度、高さは4行+見出し(probe の 420×320 と
                    // 同桁)。リサイズ可なので初期値以上の拘束は持たない。
                    size: iced::Size::new(480.0, 400.0),
                    resizable: true,
                    ..iced::window::Settings::default()
                });
                self.settings_window = Some(id);
                open.map(Message::SettingsWindowOpened)
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

    // ---- Stage ギズモ(GZ 結線、第5波) ----

    /// ギズモ drag の契約(`stage::GizmoDrag` doc: 1 drag = Start → Move* →
    /// Commit|Cancel)を Inspector の drag-to-scrub と同じ経路へ写す:
    /// - Start: shell 側 transient([`GizmoShellDrag`])を立てるだけ(Document
    ///   は触らない)。宛先時刻(playhead/fps)はこの時点で凍結。
    /// - Move: `Document::set_transient`(edit timeline に触れない overlay —
    ///   undo/redo の意味論は drag 中ずっと不変)。
    /// - Commit: transient を外し、`Intent::SetTrack` を**1回**だけ出す
    ///   (1 drag = 1 undo)。track の意味は値セル編集と同じ
    ///   [`inspector_pane::edited_value_track`](キー無し=静的書き換え・
    ///   キー持ち= playhead へのキー upsert、AE 作法)。
    /// - Cancel: transient を外すだけ(Esc・空クリック)。
    fn update_gizmo(&mut self, event: stage::GizmoDrag) {
        match event.phase {
            stage::GizmoPhase::Start { property } => {
                let Ok(Some(composition)) = self.doc.view().composition() else {
                    return;
                };
                self.gizmo_drag = Some(GizmoShellDrag {
                    layer: event.layer,
                    property,
                    playhead_frame: self.session.playhead,
                    fps: composition.fps,
                    moved: false,
                });
            }
            stage::GizmoPhase::Move { value } => {
                let Some(drag) = self.gizmo_drag.as_mut() else {
                    return;
                };
                let layer = drag.layer;
                drag.moved = true;
                match value {
                    // 第6波(anchor drag pairing): anchor と補償済み position を
                    // 対で transient へ書く(`GizmoValue::Anchor` doc「shell は
                    // 両方を同時に書く」— 片方だけ書くと絵が跳ぶ)。
                    stage::GizmoValue::Anchor { anchor, position } => {
                        if let Ok(anchor_property) =
                            PropertyId::new(stage::GizmoProperty::Anchor.property_name())
                        {
                            self.doc.set_transient(layer, anchor_property, Value::Vec2(anchor));
                        }
                        if let Ok(position_property) =
                            PropertyId::new(stage::GizmoProperty::Position.property_name())
                        {
                            self.doc.set_transient(layer, position_property, Value::Vec2(position));
                        }
                    }
                    other => {
                        let Ok(property) = PropertyId::new(other.property().property_name()) else {
                            return;
                        };
                        self.doc.set_transient(layer, property, gizmo_store_value(other));
                    }
                }
            }
            stage::GizmoPhase::Commit { value } => {
                let Some(drag) = self.gizmo_drag.take() else {
                    return;
                };
                match value {
                    // anchor drag の確定: 2 property(anchor/position)を
                    // `Document::apply_all` で**1 undo**へ束ねる(1 gesture =
                    // 1 commit の契約は変わらない — `GizmoValue::Anchor` doc)。
                    stage::GizmoValue::Anchor { anchor, position } => {
                        let (Ok(anchor_property), Ok(position_property)) = (
                            PropertyId::new(stage::GizmoProperty::Anchor.property_name()),
                            PropertyId::new(stage::GizmoProperty::Position.property_name()),
                        ) else {
                            return;
                        };
                        let store = self.doc.view();
                        let anchor_base = store.track(drag.layer, &anchor_property).ok().flatten();
                        let position_base =
                            store.track(drag.layer, &position_property).ok().flatten();
                        let mut write_error = None;
                        match (
                            inspector_pane::edited_value_track(
                                anchor_base.as_ref(),
                                drag.playhead_frame,
                                drag.fps,
                                Value::Vec2(anchor),
                            ),
                            inspector_pane::edited_value_track(
                                position_base.as_ref(),
                                drag.playhead_frame,
                                drag.fps,
                                Value::Vec2(position),
                            ),
                        ) {
                            (Ok(anchor_track), Ok(position_track)) => {
                                let intents = [
                                    Intent::SetTrack {
                                        layer: drag.layer,
                                        property: anchor_property.clone(),
                                        track: anchor_track,
                                    },
                                    Intent::SetTrack {
                                        layer: drag.layer,
                                        property: position_property.clone(),
                                        track: position_track,
                                    },
                                ];
                                if let Err(error) = self.doc.apply_all(intents) {
                                    write_error = Some(format!("値を書けない: {error}"));
                                }
                            }
                            (Err(error), _) | (_, Err(error)) => write_error = Some(error),
                        }
                        self.doc.clear_transient(drag.layer, &anchor_property);
                        self.doc.clear_transient(drag.layer, &position_property);
                        if let Some(error) = write_error {
                            self.status = Some(error);
                        }
                    }
                    other => {
                        let Ok(property) = PropertyId::new(other.property().property_name()) else {
                            return;
                        };
                        // transient は `track()` に映らないので、ここで読むのは drag
                        // 開始前の本 track そのもの(`finish_field_drag` と同じ注記)。
                        let base_track = self.doc.view().track(drag.layer, &property).ok().flatten();
                        let mut write_error = None;
                        match inspector_pane::edited_value_track(
                            base_track.as_ref(),
                            drag.playhead_frame,
                            drag.fps,
                            gizmo_store_value(other),
                        ) {
                            Ok(track) => {
                                if let Err(error) = self.doc.apply(Intent::SetTrack {
                                    layer: drag.layer,
                                    property: property.clone(),
                                    track,
                                }) {
                                    write_error = Some(format!("値を書けない: {error}"));
                                }
                            }
                            Err(error) => write_error = Some(error),
                        }
                        // 書き込み失敗時も overlay は必ず外す(`finish_field_drag` と
                        // 同じ — overlay を残さない)。
                        self.doc.clear_transient(drag.layer, &property);
                        if let Some(error) = write_error {
                            self.status = Some(error);
                        }
                    }
                }
            }
            stage::GizmoPhase::Cancel => {
                self.cancel_gizmo_drag();
            }
        }
    }

    /// Esc 連鎖用(clip/key/loop の並び — `Message::EscapePressed` 腕)。
    /// transient overlay は edit timeline に触れていないので、外すだけで復元が
    /// 成立する(`inspector_pane::cancel_field_interaction` と同型)。冪等 —
    /// canvas 側の Esc(`GizmoPhase::Cancel`)と二重に届いても2回目は `false`。
    fn cancel_gizmo_drag(&mut self) -> bool {
        let Some(drag) = self.gizmo_drag.take() else {
            return false;
        };
        if drag.moved {
            // anchor drag は2 property を対で transient へ書いている
            // (`update_gizmo` の `Move` 分岐)ので、cancel も両方外す —
            // 片方だけ残すと絵が跳んだまま止まる。
            if matches!(drag.property, stage::GizmoProperty::Anchor) {
                if let Ok(property) = PropertyId::new(stage::GizmoProperty::Anchor.property_name()) {
                    self.doc.clear_transient(drag.layer, &property);
                }
                if let Ok(property) = PropertyId::new(stage::GizmoProperty::Position.property_name()) {
                    self.doc.clear_transient(drag.layer, &property);
                }
            } else if let Ok(property) = PropertyId::new(drag.property.property_name()) {
                self.doc.clear_transient(drag.layer, &property);
            }
        }
        true
    }

    // ---- Timeline マーカーレーン(B19、第6波、`timeline::markers` 冒頭 doc の
    // 統合手順2「Message::Marker 畳み+JumpTo 先取り」) ----

    /// `Message::Marker` の畳み。**canvas 差し替え・input 優先順位・実際の
    /// mouse capture(`MarkerMessage::Grabbed`/`DragMoved`/`DragReleased`/
    /// `DragCancelled` を publish する側)は未結線**(`motolii-timeline-pane`
    /// の `canvas.rs`/`input.rs` が `pub(crate)` のため、EXACT TARGET
    /// 「pane crate は読み専用」の範囲で shell からは触れない — RETURN の
    /// API 要求参照)。この関数は Document 書き込みの意味だけを完結させる
    /// (keymap M=AddAtPlayhead は実際に届く経路、他は将来 canvas 側が
    /// publish するようになった時にそのまま機能する形で用意してある)。
    fn update_marker(&mut self, message: timeline::markers::MarkerMessage) {
        use timeline::markers::MarkerMessage;
        match message {
            MarkerMessage::AddAtPlayhead => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                if let Some(next) =
                    timeline::markers::added_at_playhead(&markers, self.session.playhead, fps)
                {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを置けない: {error}"));
                    }
                }
            }
            // S2 発注 #22 の2入口目(ルーラ locator lane 右クリック)。
            // `AddAtPlayhead` と同じ意味・同じ Intent、位置だけ呼び出し元
            // (`Message::AddMarkerAt(frame)`)が決める。
            MarkerMessage::AddAtFrame(frame) => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                if let Some(next) = timeline::markers::added_at_frame(&markers, frame, fps) {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを置けない: {error}"));
                    }
                }
            }
            // JumpTo は先取り(`ScrubTo`/`timeline_pane::Message::ScrubTo` と
            // 同じ経路 — playhead を直接書く、正典 §5「K/J ナビの補完」)。
            MarkerMessage::JumpTo(frame) => self.session.playhead = frame,
            MarkerMessage::Remove(index) => {
                let markers = self.markers();
                if let Some(next) = timeline::markers::removed(&markers, index) {
                    if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                        self.status = Some(format!("マーカーを削除できない: {error}"));
                    }
                }
            }
            MarkerMessage::Grabbed { index, at_frame } => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let markers = self.markers();
                self.marker_drag = timeline::markers::MarkerDrag::start(&markers, index, at_frame, fps);
            }
            MarkerMessage::DragMoved { at_frame } => {
                let Some(fps) = self.composition().map(|c| c.fps) else {
                    return;
                };
                let duration = self.comp_duration();
                if let Some(drag) = self.marker_drag.as_mut() {
                    drag.dragged(at_frame, fps, duration);
                }
            }
            MarkerMessage::DragReleased => {
                if let Some(drag) = self.marker_drag.take() {
                    if let Some(next) = drag.finish() {
                        if let Err(error) = self.doc.apply(Intent::SetMarkers { markers: next }) {
                            self.status = Some(format!("マーカーを移動できない: {error}"));
                        }
                    }
                }
            }
            MarkerMessage::DragCancelled => {
                self.marker_drag = None;
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

    /// Settings 窓の台帳の読み口(S2、裁定182/188)。旧 `settings_panel_open()`
    /// (screenshot 器具専用の bool)は廃止 — Settings は OS 窓になり、単窓
    /// オフスクリーン合成の screenshot 器具の**対象外**(`screenshot.rs` 冒頭
    /// doc の明示コメント参照)。この口は窓台帳の検分
    /// (`tests/suite/window_drive.rs`/`q0_fence.rs`)が使う。
    pub fn settings_window(&self) -> Option<iced::window::Id> {
        self.settings_window
    }

    /// Browser パネルの開閉状態(B3)。**screenshot 器具専用**の読み口
    /// (`checkerboard_enabled` と同じ形) — `--browser-open` CLI フラグ
    /// (`main.rs`)経由で `Message::Browser(browser_pane::Message::
    /// ToggleBrowserPanel)` を実際に通した後の状態を screenshot.rs が読める
    /// ようにする。フラグそのものは `browser::PaneState::is_open` に住む
    /// (`state.rs` 冒頭 doc「Shell 側に per-variant 分岐を増やさない」) —
    /// この口は単なる薄い委譲。
    pub fn browser_panel_open(&self) -> bool {
        self.browser.is_open()
    }

    /// Export 窓の台帳の読み口(B09、第6波)。`settings_window()` と同型 —
    /// 運転席(`tests/suite/export_drive.rs`)が open/close の状態遷移を読む。
    pub fn export_window(&self) -> Option<iced::window::Id> {
        self.export_window
    }

    /// Stage 方眼シート束のトグル状態の読み口(B22、第6波)。運転席が
    /// 「View メニューを押す → トグルが反転する」を確かめる口。
    pub fn sheet_toggles(&self) -> stage::SheetToggles {
        self.sheet_toggles
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
        browser_pane::model::assets_with_status(&self.doc.view(), &|id| {
            self.asset_status.get(&id).cloned()
        })
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
            // B21+B18(第5波結線): 作業範囲/ループの状態は `PaneState` が持ち
            // (`work_area.rs` doc「型の置き場」)、絵と当たりへはこの読み口
            // 経由で毎フレーム運ぶ(`with_playing` と同じ薄い builder)。
            .with_work_area(self.timeline.work_area(), self.timeline.loop_enabled())
            // 第6波(rename 統合手順1): inline rename の下書きを rail の
            // `text_input` へ運ぶ(`rail.rs` の `pane.rename` 読み — 供給は
            // supervisor の仕事、`write.rs` 冒頭 doc 参照)。
            .with_rename(
                self.timeline
                    .rename_draft()
                    .map(|(layer, draft)| (layer, draft.to_owned())),
            )
            // 波形取得状態(TL7 統合手順3、S2 発注 #17「shell 側の呼び出し
            // 経路が無い」の穴埋め)。`self.timeline.waveforms()` を
            // `with_rename` と同じ「薄い builder で読み取り専用に運ぶだけ」
            // の形でそのまま渡す(実際の要求発火は `poll_waveform_fetches`)。
            .with_waveforms(self.timeline.waveforms().clone())
    }

    /// 音声 layer の波形取得を計画し、必要な分だけ非同期に発火する(TL7
    /// 統合手順1・5、S2 発注 #17「shell 側の呼び出し経路が無い」の穴埋め)。
    /// `Shell::update` の末尾から毎メッセージ後に呼ぶ(`refresh_frame` と
    /// 同じ「都度呼んでも安いので判断を持たせない」形 — `plan_waveforms`
    /// 自体が `Loading`/`Ready` を見て何もしない側へ落ちるので、音声 layer が
    /// 無い/既に取得済みの通常のフレームでは実質 no-op)。
    ///
    /// **画面幅は未知(EXACT TARGET 外)**: 実際の bar 幅は canvas 描画時の
    /// window 幅に依存する(`ruler.rs`/`canvas.rs` の `bounds.width`)が、
    /// `Shell` は window サイズを保持していない(`grep -n window_size` 0件、
    /// 実測)。ここでは固定の目安幅
    /// (`NOMINAL_WAVEFORM_WIDTH_PX`)を渡す — bucket 数が実窓とズレるのは
    /// 承知の上(発注書「波形は呼び出し経路の説明で足りる。描画の正しさは
    /// 窓が要るので【未確認】のまま残してよい」)。呼び出し経路(plan→
    /// Task::perform→WaveformFetched→Ready→canvas 描画)自体は実働する。
    fn poll_waveform_fetches(&mut self) -> Task<Message> {
        const NOMINAL_WAVEFORM_WIDTH_PX: f32 = 960.0;
        let store = self.doc.view();
        let rows = timeline_pane::audio_rows(&store);
        if rows.is_empty() {
            return Task::none();
        }
        let requests = self.timeline.plan_waveforms(&rows, |_layer| NOMINAL_WAVEFORM_WIDTH_PX);
        if requests.is_empty() {
            return Task::none();
        }
        Task::batch(requests.into_iter().map(|(layer, path, buckets)| {
            Task::perform(
                async move { motolii_media::waveform_peaks(path, buckets) },
                move |result| match result {
                    Ok(peaks) => Message::Timeline(timeline_pane::Message::WaveformFetched {
                        layer,
                        buckets,
                        peaks,
                    }),
                    Err(_) => Message::Timeline(timeline_pane::Message::WaveformFetchFailed {
                        layer,
                        buckets,
                    }),
                },
            )
        }))
    }



    /// 作業範囲の現在値(B18、第5波結線)。運転席(`tests/suite/`)が
    /// 「B/N・ループ帯ドラッグ → 範囲が立つ」「Esc → 復元」を検分する読み口
    /// (`timeline_rows`/`markers` と同じ「pane 自身が読むのと同じ状態」の形)。
    pub fn timeline_work_area(&self) -> Option<timeline_pane::WorkArea> {
        self.timeline.work_area()
    }

    /// ループ on/off の現在値(同上 — `advance_playback_tick` が読むのと同じ値)。
    pub fn timeline_loop_enabled(&self) -> bool {
        self.timeline.loop_enabled()
    }



}


