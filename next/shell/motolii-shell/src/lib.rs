//! wraps: iced — front。**store への query の投影**であって、Document の写しを持たない。
//!
//! 背骨1 を型で作る:
//! - **書き口は [`Shell::update`] の1箇所だけ**。pane 関数は `StoreView`(不変)・
//!   `&Session`・[`tokens::Tokens`](裁定117、寸法・色。Document 由来ではなく書けない)
//!   しか受け取らないので、**書ける物を持っていない**
//! - `view(&self)` が `&self` を取るので、描画中に Document を触る道が無い
//!
//! Stage は **CPU 経路**(合成結果の RGBA を `image` widget へ渡す)。
//! iced の device の上に `re_renderer` を建てる道は裁定44 で撤回した。
//!
//! **front が持ってよい状態**は [`Session`] だけ — 選択と再生位置。これらは
//! Document の写しではなく、undo の対象でもない(rerun も選択は blueprint store の
//! 外に置いている)。**1箇所で持ち、全 pane がそこを読む**ので M14 は満たされる。

use iced::widget::{button, column, container, image, row, slider, stack, text};
use iced::{Element, Length, Task};

use motolii_engine::{Engine, ObservationCamera};
use motolii_store::{
    AssetDraft, Composition, DisplayRevision, Document, Intent, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, RationalTime, SourceFingerprintV1, Speed, StoreView,
};

pub mod clipboard;
pub mod fixture;
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
/// なので壊す既存参照は無い)。**B0 時点では `Message::Browser` の腕はあるが
/// `Shell::view` には組み込まない**(描画ゼロ = 挙動ゼロ変更の証明、view 配線は
/// B3 で絵と一緒に)。
pub use motolii_browser_pane as browser_pane;

use chrome::button_style;
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
    pub fn handle_creations() -> u64 {
        0
    }
    pub fn last_handle_bytes() -> usize {
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

/// iced(`next/` が実際に使う crates.io `iced 0.14.0`、`iced_wgpu-0.14.0/src/
/// image/cache.rs::upload_raster`)が同期アップロードを選ぶ上限を**転記した
/// 定数**(`MAX_SYNC_SIZE = 2 * 1024 * 1024`、実測済み)。これを超える RGBA を
/// `image::Handle::from_rgba` に渡すと、iced はバックグラウンドスレッドへ
/// 非同期アップロードへ回し、完了までの1フレーム以上 `draw_image` は何も
/// 描かない(`iced_core-0.14.0/src/image.rs` の `Allocation` doc comment に
/// 明記: "If you are animating images, this can cause undesirable flicker")。
///
/// fixture の comp は 1920×1080 = 8,294,400 byte(この上限の約4倍)。scrub の
/// たびに新しい Handle → 非同期アップロード → 空白フレーム、が実機チラつきの
/// 一次原因と特定した(2026-08-20)。上限ぴったりでなく余裕を持たせてある。
const STAGE_HANDLE_SYNC_BUDGET_BYTES: usize = 1_500_000;

/// 自動予算導出スケール(sync 予算を超える時だけ sqrt で縮める、超えなければ
/// 無変更=1.0)。[`stage::effective_preview_scale`] へ渡す「auto」側の値
/// そのもの(裁定163 Stage 下縁状態帯 ORACLE (a)) — 旧 `stage_handle_rgba`
/// が抱えていた分岐をこの関数へ切り出しただけで、`width`/`height` が既に
/// 予算内の時に1.0を返す挙動は無改変。
fn stage_auto_scale(width: u32, height: u32) -> f64 {
    let total_bytes = (width as usize) * (height as usize) * 4;
    if width == 0 || height == 0 || total_bytes <= STAGE_HANDLE_SYNC_BUDGET_BYTES {
        1.0
    } else {
        (STAGE_HANDLE_SYNC_BUDGET_BYTES as f64 / total_bytes as f64).sqrt()
    }
}

/// Stage 表示用に RGBA を縮める。**画面には `Length::Fill` で引き伸ばして出す
/// ので実素材解像度である必要が無い**(screenshot 器具は `frame_rgba()` が返す
/// 元解像度の RGBA を別途持っている — 縮めるのは Handle 用のコピーだけで、
/// pixel 精度が要る経路には触らない)。nearest-neighbor(プレビュー用途なので
/// 品質は問わない — `screenshot.rs::blit_letterboxed` と同じ考え方)。
///
/// **裁定163 Stage 下縁状態帯**: `resolution_cap` はユーザーが明示的に選ぶ
/// 上限(Auto/½/¼)——[`stage_auto_scale`] の自動導出値へ
/// [`stage::effective_preview_scale`] で min 合成する。`Auto` は cap=1.0固定
/// なので合成しても値が変わらず、旧来の「予算内なら無変更・超えたら sqrt
/// スケール」の挙動と完全に同値(ORACLE (a) 「auto=1.0で½cap→0.5・auto=0.4で
/// ½cap→0.4」のとおり、cap の方が緩ければ auto 側がそのまま勝つ)。
fn stage_handle_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    resolution_cap: stage::PreviewResolutionCap,
) -> (u32, u32, Vec<u8>) {
    if width == 0 || height == 0 {
        return (width, height, rgba.to_vec());
    }
    let scale = stage::effective_preview_scale(stage_auto_scale(width, height), resolution_cap);
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

    // ---- Browser pane 骨格(ζ 縫い目調査+裁定162 切片 B0、まだ何も描かない) ----
    /// `motolii_browser_pane::Message` を1本で畳む(`Message::Settings`/
    /// `Message::Stage` と同型)。**B0 時点では `browser_pane::Message` が
    /// 空 enum なので、この腕は実質発行されない**(B1 以降、素材列挙/rail・
    /// filter の腕が増えるのに追随して `Shell::update` 側の match 中身も足す)。
    Browser(browser_pane::Message),

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
    handle: image::Handle,
    /// `Engine::render_frame`(背景込み)の生 RGBA。**export/screenshot 真値専用**
    /// (`screenshot.rs`・`frame_rgba()`)— 通常描画は `handle` だけで足りる
    /// (iced の `image::Handle` から画素を取り戻す公開 API が無いため、この
    /// 用途だけのために複製して持つ)。**市松は絶対にここへ乗せない**し、
    /// 市松トグルで一切変わらない(`settings_pane` doc「合成器が出せる」と
    /// 「書き出しが吐く」は別問題、参照)。
    rgba: Vec<u8>,
    /// 市松 ON の間だけ `Some` — 裁定141「AE型の透明可視化モード」の入力
    /// (`Engine::render_frame_without_background`、背景 layer を省いた合成結果)。
    /// `handle`(Stage 表示)と `screenshot.rs` は市松 ON の間、`rgba` の代わりに
    /// これへ [`settings_pane::composite_checkerboard`] を当てる。市松 OFF の
    /// 間は `None`(`rgba` をそのまま使う)。**export 真値(`rgba`)には一切
    /// 影響しない** — 別フィールド。
    checkerboard_preview_rgba: Option<Vec<u8>>,
    /// `handle` が市松込みで作られているか。**Document・playhead 非依存**の
    /// 表示分岐なので、`revision()`/`playhead` が同じでもここが変わっていれば
    /// `refresh_frame` は Document の再評価をせず Handle だけ作り直す(市松 ON
    /// の間は `checkerboard_preview_rgba` を取り直すため engine を1回追加で
    /// 回すが、`Document`/`StoreView` の評価が増えるわけではない)。
    checkerboard: bool,
    /// この `handle` を作った時点の観測カメラ(裁定157)。`display_revision()`/
    /// `playhead`/`checkerboard` と同じ「キャッシュを落とすかどうか」の鍵の
    /// 一部 — `refresh_frame` の早期 return はこれも比較する(`checkerboard`
    /// と同格の表示専用の鍵拡張)。
    observation: Option<ObservationCamera>,
    /// 観測カメラ有効時の Stage 表示 RGBA(`Engine::render_frame_with_view_camera`
    /// の結果そのもの)。**`rgba`(export 真値)とは別物** — `checkerboard_preview_rgba`
    /// と同じ「表示専用の複製」の形。`observation` が `None` の間は常に `None`。
    observation_rgba: Option<Vec<u8>>,
    /// この `handle` を作った時点のプレビュー解像度 cap(裁定163 Stage 下縁
    /// 状態帯)。**`checkerboard`/`observation` と同格の鍵拡張** —
    /// `stage_handle_rgba` へ渡す実効スケールを変えるだけの表示専用の値なので、
    /// `revision()`/`playhead` が同じでもここが変わっていれば Handle だけ
    /// 作り直す(Document・engine の再評価は増えない)。
    resolution_cap: stage::PreviewResolutionCap,
}

/// [`Shell::compute_display_source`] の戻り値。Stage 表示(`handle`)用の入力を
/// 1箇所へまとめただけの内部型 — `RenderedFrame` のフィールドへの書き戻しと
/// `build_stage_handle` への引数の両方をこれ1つから作る(呼び出し側の
/// `refresh_frame` が2箇所(キャッシュヒット/フル再計算)で同じ分岐を書かずに
/// 済む)。
struct DisplaySource {
    /// `build_stage_handle` へ渡す実体。`None` なら呼び出し側は
    /// `RenderedFrame::rgba`(export 真値)をそのまま使う(市松・観測カメラの
    /// どちらも効いていない既定の場合)。
    full_rgba: Option<Vec<u8>>,
    /// `full_rgba` を市松タイルで覆うかどうか(`build_stage_handle` の第4引数)。
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

    // ---- Settings パネル(タスク#18) ----
    /// パネルの開閉。**表示だけの状態** — Document でも `Session`(選択・再生
    /// 位置)でもない。発注書は「Workspace 側」と指示しているが、Workspace 永続
    /// 機構がまだ無い(裁定127/128)ため、`tokens::Dimensions::ui_scale` の
    /// 「仮の置き場」と同じ理由でここに仮置きする。
    settings_panel_open: bool,
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
}

impl Shell {
    pub fn new() -> (Self, Task<Message>) {
        let mut doc = Document::new();
        // 空の Document には comp が無く、Stage が何も出せない。
        // 起動直後に何も見えないのは M17 に反するので、既定の comp を置く。
        let _ = doc.apply(Intent::SetComposition(Composition {
            width: 640,
            height: 360,
            fps: motolii_store::Fps::try_new(30, 1).expect("30fps"),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }));

        // 既定値は「編集」ではないので戻せてはいけない。
        doc.mark_undo_floor();

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
                settings_panel_open: false,
                checkerboard: false,
                background_draft: None,
                ui_scale_draft: None,
                observation: None,
                resolution_cap: stage::PreviewResolutionCap::default(),
                clipboard: clipboard::Clipboard::default(),
                transport: Transport::new(),
            },
            Task::none(),
        )
    }

    /// `--fixture` 起動が使う口。**トンマナ検分の器具**(発注書)— `fixture::build()`
    /// が既存 Intent(`apply_all`)だけで組んだ Document を、通常の `new()` と同じ形で
    /// `Shell` へ包む。`update()` を経由しない点だけが `new()` と違う(初期状態の
    /// 組み立ては元々 `new()` も `doc.apply` を直に呼んでおり、同じ扱い)。
    pub fn new_fixture() -> (Self, Task<Message>) {
        let built = fixture::build();
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
            settings_panel_open: false,
            checkerboard: false,
            background_draft: None,
            ui_scale_draft: None,
            observation: None,
            resolution_cap: stage::PreviewResolutionCap::default(),
            clipboard: clipboard::Clipboard::default(),
            transport: Transport::new(),
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
        // 落ちるので`transport::tick_stream`のOSスレッドも後始末される
        // (`transport.rs`のdoc参照)。
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
            // B0: `browser_pane::Message` はまだ空 enum なので、この match は
            // 中身が無い(`msg` に variant が無い = 到達しない、B1 以降で腕が
            // 増えたらここへ追随させる)。
            Message::Browser(msg) => match msg {},
            Message::AddLayer => {
                let id = LayerId(self.next_layer_id());
                // **1操作 = 1 undo**。`AddLayer` と `SetMeta` を別々に書くと
                // 利用者は Undo を2回押すことになる(ui-quality-bar Q2)。
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
        }
    }

    /// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
    /// (1 gesture = 1 undo)。書き込み本体は [`inspector_pane::
    /// commit_inspector_field`](自由関数、`&mut self.doc`/`&mut self.
    /// inspector_field_draft`/`self.session.selection` をそのまま貸す)——
    /// ここは `Err` を status 帯へ渡す glue だけ(M13)。
    fn commit_inspector_field(&mut self, field: TransformField) {
        let t = self.time_at_playhead().unwrap_or(RationalTime::ZERO);
        if let Err(error) = inspector_pane::commit_inspector_field(
            &mut self.doc,
            &mut self.inspector_field_draft,
            self.session.selection,
            t,
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
    /// なし・animated(編集不可)・対応する field が投影に無い、のいずれも
    /// 黙って無視。
    fn start_field_drag(&mut self, field: TransformField) {
        let projection = self.inspector_selection();
        inspector_pane::start_field_drag(
            &mut self.inspector_drag,
            self.session.selection,
            projection.as_ref(),
            field,
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
    /// 自分で組み合わせる必要がある — `lib.rs::build_stage_handle` と同じ形)。
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

    /// 描き上がった Stage フレームの生 RGBA。**常に背景込みの export 真値**
    /// (`Engine::render_frame`)— 市松トグルで一切変わらない。**screenshot
    /// 器具専用**(`screenshot.rs`)— 通常描画は `image::Handle` を持つ
    /// `stage_pane` を通る。
    pub fn frame_rgba(&self) -> Option<(u32, u32, &[u8])> {
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
        let timeline = self.build_timeline_pane();
        // Inspector は canvas を使わない標準 widget 構成(inspector_pane crate 冒頭の
        // doc comment)なので、投影自体が `Element<'static, _>` を返す — Stage の
        // `self.frame` を借りる `stage_pane` と同じ `row!` に同居できる(共変性)。
        let inspector_selection = inspector_pane::project(&store, &self.session)
            .ok()
            .flatten();
        let inspector = inspector_pane::view_with_speed_draft(
            inspector_selection.as_ref(),
            self.inspector_field_draft.as_ref(),
            self.inspector_name_draft.as_deref(),
            self.inspector_speed_draft.as_deref(),
            dims,
            colors,
        )
        .map(Message::Inspector);

        // Settings パネル(タスク#18)。**表示だけの分岐** — 開いていなければ
        // 木に一切現れない(Q0: 効かない chrome を並べない、閉じている間は
        // 下書き入力欄も存在しないので誤操作の的にならない)。
        let mut layout = column![self.header()];
        if self.settings_panel_open {
            layout = layout.push(
                settings_pane::view(
                    self.composition().as_ref(),
                    self.background_draft.as_ref(),
                    self.tokens.ui_scale,
                    self.ui_scale_draft.as_deref(),
                    dims,
                    colors,
                )
                .map(Message::Settings),
            );
        }

        layout
            .push(
                row![
                    inspector,
                    stage_pane(
                        self.frame.as_ref(),
                        self.stage_overlay(),
                        self.observation,
                        self.resolution_cap,
                        self.checkerboard,
                        dims,
                        colors
                    )
                ]
                .spacing(dims.spacing_m)
                .height(Length::FillPortion(3)),
            )
            // pane crate 化(裁定160 切片7)で `timeline.view()` は
            // `Element<'static, timeline_pane::Message>` を返すようになった
            // (root の `Message` を pane crate から参照できないため — 循環
            // 回避)。`.map(Message::Timeline)` で1回だけ畳む(§3.1 の
            // 「pane-local Message を親が畳む」構成そのもの)。
            .push(timeline.view().map(Message::Timeline))
            .push(transport(&self.session, &store, self.transport.is_running(), dims, colors))
            .push(status_band(self.status.as_deref(), &self.doc, dims, colors))
            .spacing(dims.spacing_m)
            .padding(dims.spacing_l)
            .into()
    }

    /// shell chrome の線化(裁定137/139 の Inspector 以外の面への展開)。
    /// 旧実装はこの帯にコンテナが無く、地(背景)も境界(hairline)も持たない
    /// 生の `row!` だった — 帯の下の Stage/Inspector 行とは `spacing_m` の
    /// gap だけで離れており「面色の塗り分けで区切る」違反ではなかったが、
    /// 帯自身が「パネル」だと分かる縁を持っていなかった。Timeline の `.tp`
    /// (transport 帯、background=panel + border-bottom hairline)と同じ
    /// grammar をここへも延長する — 新しい視覚言語の発明ではない。
    fn header(&self) -> Element<'_, Message> {
        let dims = self.dims();
        let colors = self.tokens.colors;
        let buttons = row![
            button(text("Undo").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press_maybe(self.doc.can_undo().then_some(Message::Undo)),
            button(text("Redo").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press_maybe(self.doc.can_redo().then_some(Message::Redo)),
            button(text("+ Layer").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press(Message::AddLayer),
            // **歯車ボタン**(発注書)。他3ボタンと同じく文言ボタン — この codebase
            // は一貫してアイコンではなく文字で chrome を作る(M/S glyph も文字、
            // `inspector_pane.rs` 冒頭 doc 参照)ので、絵文字/unicode グリフの
            // フォント欠け(`../reference/KNOWN.md` の letter-spacing 欠けと同種の
            // iced 0.14 の未確認リスク)を踏まない選択。
            button(text("Settings").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press(Message::Settings(settings_pane::Message::ToggleSettingsPanel)),
        ]
        .spacing(dims.spacing_m)
        .align_y(iced::alignment::Vertical::Center);

        container(buttons)
            .width(Length::Fill)
            .height(Length::Fixed(dims.panel_header_height))
            .padding([0.0, dims.spacing_s])
            .align_y(iced::alignment::Vertical::Center)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(colors.surface_panel)),
                border: iced::Border {
                    color: colors.border_default,
                    width: dims.border_width,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            })
            .into()
    }

    /// 採番の正本は store 側([`StoreView::next_layer_id`])。**墓標を含む最大 id + 1**
    /// を返すので、削除した layer の id が再利用されない(2026-08-20 の敵対的レビュー修正)。
    fn next_layer_id(&self) -> u64 {
        self.doc.view().next_layer_id()
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
                let (handle, handle_bytes) = match &display.full_rgba {
                    Some(rgba) => build_stage_handle(
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
                        build_stage_handle(width, height, &frame.rgba, false, resolution_cap, colors, ui_scale)
                    }
                };
                metrics::record_handle_creation(handle_bytes);
                if let Some(frame) = self.frame.as_mut() {
                    frame.handle = handle;
                    frame.checkerboard = checkerboard;
                    frame.checkerboard_preview_rgba = display.checkerboard_preview_rgba;
                    frame.observation = observation;
                    frame.observation_rgba = display.observation_rgba;
                    frame.resolution_cap = resolution_cap;
                }
                return;
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
                let (handle, handle_bytes) = match &display.full_rgba {
                    Some(preview) => build_stage_handle(
                        composition.width,
                        composition.height,
                        preview,
                        display.checkerboard,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                    None => build_stage_handle(
                        composition.width,
                        composition.height,
                        &rgba,
                        false,
                        resolution_cap,
                        colors,
                        ui_scale,
                    ),
                };
                metrics::record_handle_creation(handle_bytes);
                self.frame = Some(RenderedFrame {
                    revision,
                    playhead,
                    width: composition.width,
                    height: composition.height,
                    handle,
                    rgba,
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

    /// Stage 表示(`handle`)用の入力を決める。**`rgba`(export 真値)そのものには
    /// 一切触れない** — ここが返す物は表示専用の複製(`build_stage_handle` へ
    /// そのまま渡すか、`full_rgba: None` の時は呼び出し側が `RenderedFrame::rgba`
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

/// Stage 表示用の Handle を作る唯一の場所。`stage_handle_rgba` で縮め、
/// **市松が有効なら display 用の複製にだけ**
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
/// 明示的に受け取り、`stage_handle_rgba` と同じ縮小率
/// (`stage::effective_preview_scale(stage_auto_scale(width, height),
/// resolution_cap)`)を自分でも算出して
/// [`settings_pane::checkerboard_tile_px`] に渡す — comp 画素空間固定だった
/// 旧タイル寸(8px)が Auto 縮小後にさらに痩せて実質不可視になっていた
/// 根因1をここで補正する(`settings_pane::checkerboard_tile_px` doc 参照)。
fn build_stage_handle(
    width: u32,
    height: u32,
    full_rgba: &[u8],
    checkerboard: bool,
    resolution_cap: stage::PreviewResolutionCap,
    colors: Colors,
    ui_scale: f32,
) -> (image::Handle, usize) {
    let (handle_width, handle_height, mut handle_rgba) =
        stage_handle_rgba(width, height, full_rgba, resolution_cap);
    if checkerboard {
        let effective_scale = stage::effective_preview_scale(stage_auto_scale(width, height), resolution_cap);
        let tile_px = settings_pane::checkerboard_tile_px(ui_scale, effective_scale);
        settings_pane::composite_checkerboard_with_tile_px(
            handle_width,
            handle_height,
            &mut handle_rgba,
            colors,
            tile_px,
        );
    }
    let handle_bytes = handle_rgba.len();
    (
        image::Handle::from_rgba(handle_width, handle_height, handle_rgba),
        handle_bytes,
    )
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
///
/// **既定割当は仮**(拘束6・NudgeKeyframe と同じ「keymap 層が無い今だけ直結」
/// の注記どおり) — アクション名(`Message::StepPlayhead`/`JumpPlayheadToStart`/
/// `JumpPlayheadToEnd`/`JumpMeaningPoint`/`JumpClipEdge`/`Message::Undo`/`Redo`/
/// `CopyLayer`/`PasteLayer`/`CutLayer`/`DuplicateLayer`/`SelectAllLayers`/
/// `DeselectAllLayers`)だけを正本として残す。
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
            let picture: Element<'_, Message> = image(frame.handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
            // 観測カメラの入力(ホイール/中ボタンドラッグ)とフレーム枠 overlay
            // (裁定157)。`image` の上に重ねるだけ — `image` 自体は変形しない
            // (Stage は image 貼りのまま、`stage.rs` モジュール doc 参照)。
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

    // 自動導出スケール(`stage_auto_scale` — sync 予算内なら1.0、超えれば
    // sqrt スケール)。frame が無ければ縮める対象自体が無いので1.0固定
    // (Auto 表示は「1.00×」になるが、band 自体は comp が無くても常時表示 —
    // 発注書 EXACT TARGET 2「常時表示」)。
    let auto_scale = frame.map(|f| stage_auto_scale(f.width, f.height)).unwrap_or(1.0);
    let band = stage::state_band_view(observation, resolution_cap, auto_scale, checkerboard, dims, colors)
        .map(Message::Stage);

    // letterbox は neutral dark(D8: 装飾 gradient 禁止・余白は neutral)。raw 値ではなく
    // token 経由の面色 + 罫線幅。
    // **高さは `Length::Fill`**(Inspector と並ぶ `row!` の中にいるため、以前の
    // `FillPortion(3)` は `Shell::view` 側のその `row!` 自身が持つ — 2箇所で
    // portion を重ねて割合をずらさない)。
    container(column![container(body).width(Length::Fill).height(Length::Fill), band].spacing(0.0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(colors.surface_app)),
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// `is_playing` は `Shell::transport`(Document/`Session`とは別の身分の
/// front 状態、`transport.rs` doc 参照)から呼び出し側が渡す — pane 関数は
/// `StoreView`/`&Session`/`Tokens` しか取らない制約(このファイル冒頭の
/// doc)の例外を増やさないため、他の pane と同じ「呼び出し側が明示引数で
/// 渡す」形(`stage_pane`の`overlay`と同じ)にした。
fn transport<'a>(
    session: &Session,
    store: &StoreView<'a>,
    is_playing: bool,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let last = store
        .composition()
        .ok()
        .flatten()
        .map(|c| (c.duration_frames - 1).max(0) as i32)
        .unwrap_or(0);

    // Play/Pause(A2)。**マウス完結**(ui-hand-feel-direction: 手触り方向 —
    // Spaceキーだけに頼らない、クリックでも成立する)。ラベルは今の状態の
    // 逆(ボタンは「次に何が起きるか」を示す慣習、多くの動画編集/音楽
    // ソフトと同じ)。
    let toggle_label = if is_playing { "Pause" } else { "Play" };

    row![
        button(text(toggle_label).size(dims.body_text))
            .style(move |_theme, status| button_style(dims, colors, status))
            .on_press(Message::TogglePlayback),
        text(format!("frame {}", session.playhead))
            .size(dims.body_text)
            .color(colors.action_active),
        slider(0..=last, session.playhead as i32, |frame| {
            Message::ScrubTo(i64::from(frame))
        }),
    ]
    .spacing(dims.spacing_m)
    .height(Length::Fixed(dims.transport_band))
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

/// shell chrome の線化(裁定137/139)。旧実装は帯に境界を一切持たない生の
/// `text` で、Stage/Timeline との違いが `spacing_m` の gap だけに頼っていた。
/// `inspector_pane.rs::hint_row`(footer 注記、border のみ・背景は塗らない)
/// と同じ grammar をそのまま延長する — status 帯も「今どこからが summary か」
/// を線で示す。
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
        .style(move |_theme| container::Style {
            border: iced::Border {
                color: colors.border_default,
                width: dims.border_width,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        })
        .into()
}

// `button_style` は裁定160 切片5(pane split survey §2.4/§6)で
// `chrome::button_style` へ移設した(純粋な再配置・挙動ゼロ変更)。
