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

use iced::widget::{button, column, container, image, row, slider, text};
use iced::{Element, Length, Task};

use motolii_engine::Engine;
use motolii_store::{
    Composition, DisplayRevision, Document, Intent, KeyframeTrack, LayerAttrsPatch, LayerId,
    LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, StoreView, Value,
};

pub mod fixture;
pub mod inspector_pane;
pub mod screenshot;
pub mod settings_pane;
pub mod timeline;
pub mod tokens;

/// `timeline_pane` は分割前の module path の互換エイリアス(第2波第1切片:
/// 純粋なファイル分割 — `src/timeline/`(`projection`/`hit`/`canvas`/`input`))。
/// `crate::timeline_pane::X` を読む既存参照(`screenshot.rs`・
/// `tests/suite/*.rs`)を壊さないための re-export。
pub use timeline as timeline_pane;

use inspector_pane::{FieldDraft, TransformField};
use settings_pane::{BackgroundChannel, BackgroundFieldDraft, BackgroundPreset};

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

/// Stage 表示用に RGBA を縮める。**画面には `Length::Fill` で引き伸ばして出す
/// ので実素材解像度である必要が無い**(screenshot 器具は `frame_rgba()` が返す
/// 元解像度の RGBA を別途持っている — 縮めるのは Handle 用のコピーだけで、
/// pixel 精度が要る経路には触らない)。nearest-neighbor(プレビュー用途なので
/// 品質は問わない — `screenshot.rs::blit_letterboxed` と同じ考え方)。
fn stage_handle_rgba(width: u32, height: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let total_bytes = (width as usize) * (height as usize) * 4;
    if width == 0 || height == 0 || total_bytes <= STAGE_HANDLE_SYNC_BUDGET_BYTES {
        return (width, height, rgba.to_vec());
    }

    let scale = (STAGE_HANDLE_SYNC_BUDGET_BYTES as f64 / total_bytes as f64).sqrt();
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

/// front だけが持つ状態。**Document の写しは1つも入れないこと**。
#[derive(Debug, Clone)]
pub struct Session {
    /// 再生位置(フレーム番号)。
    pub playhead: i64,
    pub selection: Option<LayerId>,
    /// Timeline property 行(キー行)の選択(第2波 T3・EXACT TARGET 3)。
    /// **Document には乗らない** — layer 選択と同じ Session の身分。
    pub selected_keys: Vec<timeline::KeySelector>,
    /// Shift 範囲選択の基点(直前に単独/Cmd クリックしたキー)。`key_order`
    /// (行順→時刻順)上の範囲は毎回この基点から張り直す(正典 §3・§4 と同じ
    /// 「anchor」文法)。
    pub key_anchor: Option<timeline::KeySelector>,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            playhead: 0,
            selection: None,
            selected_keys: Vec::new(),
            key_anchor: None,
        }
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

    // ---- Inspector pane(第1波) ----
    /// Transform 行の値セルへの打鍵。**まだ Document を書かない** — 下書きを
    /// 更新するだけ(`Shell::inspector_field_draft`、`pending_drops` と同じ形)。
    InspectorFieldInput(TransformField, String),
    /// Transform 行の Enter — **ここで初めて `Intent::SetTrack` を1回出す**
    /// (1 gesture = 1 undo)。
    InspectorFieldSubmit(TransformField),
    /// Attrs の Name 欄への打鍵。同上、まだ書かない。
    InspectorNameInput(String),
    /// Attrs の Name 欄の Enter — `Intent::SetAttrs` を1回出す。
    InspectorNameSubmit,
    /// Attrs の Hidden トグル。下書きを経由せず即 `Intent::SetAttrs` を1回出す
    /// (header の Undo/Redo ボタンと同じ即時操作の形)。
    InspectorToggleHidden,

    // ---- Timeline レーンバー(裁定147・第2波T1) ----
    /// レーンバーの M glyph クリック。`Intent::SetAttrs{hidden}` を1回出す
    /// (`InspectorToggleHidden` と同じ形だが、対象は `Session::selection` では
    /// なくクリックした行そのもの — 選択と無関係に M/S/L を操作できる)。
    LaneBarToggleMute(LayerId),
    /// レーンバーの S glyph クリック。`Intent::SetAttrs{solo}` を1回出す。
    LaneBarToggleSolo(LayerId),
    /// レーンバーの L glyph クリック。`Intent::SetAttrs{locked}` を1回出す。
    /// **`locked` 自身の解除/再ロックだけは locked な行でも常に通る**
    /// (`motolii_store::document::Intent::SetAttrs` 腕の規則、正典 §6)。
    LaneBarToggleLock(LayerId),

    // ---- Timeline property 行(キー行、第2波 T3・裁定148/151) ----
    /// キー菱形クリック。`timeline::key_rows` が「どのキーを・どの操作で」まで
    /// 判定し、確定(`Session::selected_keys`/`key_anchor` の読み書き)は
    /// `Shell::apply_key_selection`(唯一の書き口)へ委ねる。
    TimelineKeySelect(timeline::KeySelectionOp),
    /// 選択中のキーを消す(正典 §3「Delete はキー選択が層選択より優先」)。
    /// `Session::selected_keys` が空なら no-op — layer 選択の Delete(未配線)
    /// と衝突しない。1回の `apply_all` で複数 property をまとめて書くので
    /// **1操作 = 1 undo**。
    TimelineDeleteSelectedKeys,

    // ---- Inspector の drag-to-scrub ----
    /// 値セルの press。**まだ Document を書かない** — click か drag かは
    /// release まで未確定(`Shell::inspector_drag`)。
    InspectorValuePressed(TransformField),
    /// window 全体の cursor 移動(`subscription()` の `inspector_pointer_event`
    /// 経由)。`mouse_area` 自身の bounds を出た cursor は iced 0.14 に pointer
    /// capture が無く追えない(実測)ので、drag 中の主経路はここ。drag が
    /// armed/dragging でなければ即 no-op。
    InspectorPointerMoved(iced::Point),
    /// 左クリック release(同じく window 全体から)。drag が実際に動いていれば
    /// 直前の move が確定値(1 gesture = 1 undo)、動いていなければ click として
    /// type 編集へ切り替える。
    InspectorPointerReleased,
    /// Shift の押下状態。`CursorMoved` 自体は modifiers を運ばないので
    /// `ModifiersChanged` を別途追って持つ(drag 中の1/10微調整に使う)。
    KeyboardModifiersChanged(iced::keyboard::Modifiers),
    /// Esc — drag 中なら復元、typing 下書き中(値セル/名前欄)ならそれを破棄。
    EscapePressed,

    // ---- Settings パネル(タスク#18) ----
    /// ヘッダの歯車ボタン。表示だけのトグル — Document にも undo 履歴にも乗らない。
    ToggleSettingsPanel,
    /// Stage の下に市松を敷くかどうか。**表示専用** — Document には一切乗らない
    /// (書き出しに影響しない、`settings_pane` モジュール doc 参照)。
    ToggleCheckerboard,
    /// 背景色プリセット(黒/白/グレー18%)。押した瞬間に確定する
    /// (`Intent::SetComposition` を1回、1 gesture = 1 undo)。
    SettingsBackgroundPreset(BackgroundPreset),
    /// 背景 RGBA の1チャンネルへの打鍵。**まだ Document を書かない** —
    /// 下書きを更新するだけ(`InspectorFieldInput` と同じ形)。
    SettingsBackgroundChannelInput(BackgroundChannel, String),
    /// 背景 RGBA の1チャンネルの Enter — ここで初めて `Intent::SetComposition` を
    /// 1回出す(read-modify-write、他チャンネルは現在値のまま)。
    SettingsBackgroundChannelSubmit(BackgroundChannel),
    /// ui_scale(%)欄への打鍵。まだ書かない。
    UiScaleInput(String),
    /// ui_scale(%)欄の Enter — 50..200 にクランプして `Tokens`/`Dimensions` を
    /// 更新し、debug ビルドでは正本 JSON へも書き戻す(`tokens::save_ui_scale`)。
    UiScaleSubmit,
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
}

/// Inspector 値セルの drag-to-scrub、進行中の一時状態。**Document ではない**
/// (`FieldDraft` と同じ「pane が持つ transient」の形)。値そのものの置き場は
/// `Document` の transient overlay(`Document::set_transient`)— ここは overlay
/// の宛先と、click/drag 判定・確定時の Intent 組み立てに要る最小限だけを持つ。
struct FieldDragState {
    field: TransformField,
    layer: LayerId,
    /// press 時点の表示単位の値(`inspector_pane::drag_origin` が投影から読む)。
    /// 確定 Intent・Esc(overlay を外すだけで使わない)双方が参照する起点。
    start_value: f64,
    /// Vec2 系(Position/Scale/Anchor)の動かさない方の成分。scalar 系では未使用。
    current_vec2: [f64; 2],
    /// 最初の `InspectorPointerMoved` で確定する基準 x(window 座標)。`None` の
    /// 間は click か drag かまだ未確定 — 確定前に値を動かすと press 直後の
    /// sub-pixel な揺れで値が動いてしまう。
    origin_x: Option<f32>,
    /// 少なくとも1回 `set_transient` を呼んだか。release 時の click/drag 判定と、
    /// Esc で overlay を外す必要があるかどうかの両方に使う(`applied` に代わる —
    /// 履歴には一切触れないので「squash」の意味は無くなった)。
    moved: bool,
    /// 直近の `set_transient` に渡した値。release の確定 Intent はこれをそのまま
    /// 1回 `apply` する — pointer の最終座標を release 時に持っていない
    /// (`InspectorPointerReleased` は位置を運ばない)ので、最後に計算した値を
    /// ここへ持ち回す。`moved` が `false` の間は未使用。
    last_value: Option<Value>,
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
    /// `Message::InspectorFieldSubmit` が来るまで store に触らない
    /// (`pending_drops` と同じ「確定するまで front だけが持つ一時状態」の形)。
    inspector_field_draft: Option<FieldDraft>,
    /// Inspector の Name 欄、編集中の下書き。同上。
    inspector_name_draft: Option<String>,
    /// Inspector 値セルの drag-to-scrub。**Document ではない** — 同上
    /// (`FieldDragState` doc comment 参照)。
    inspector_drag: Option<FieldDragState>,
    /// 直近の Shift 押下状態。`CursorMoved` は modifiers を運ばないので
    /// `ModifiersChanged` から別途追う(drag の1/10微調整に使う)。
    keyboard_modifiers: iced::keyboard::Modifiers,

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
                inspector_drag: None,
                keyboard_modifiers: iced::keyboard::Modifiers::default(),
                settings_panel_open: false,
                checkerboard: false,
                background_draft: None,
                ui_scale_draft: None,
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
            inspector_drag: None,
            keyboard_modifiers: iced::keyboard::Modifiers::default(),
            settings_panel_open: false,
            checkerboard: false,
            background_draft: None,
            ui_scale_draft: None,
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
        iced::Subscription::batch([window, tokens, pointer])
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
            Message::ScrubTo(frame) => self.session.playhead = frame.max(0),
            Message::Select(layer) => self.session.selection = Some(layer),
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
            Message::InspectorFieldInput(field, text) => {
                self.inspector_field_draft = Some(FieldDraft { field, text });
            }
            Message::InspectorFieldSubmit(field) => self.commit_inspector_field(field),
            Message::InspectorNameInput(text) => {
                self.inspector_name_draft = Some(text);
            }
            Message::InspectorNameSubmit => self.commit_inspector_name(),
            Message::InspectorToggleHidden => self.toggle_inspector_hidden(),
            Message::LaneBarToggleMute(layer) => self.toggle_layer_hidden(layer),
            Message::LaneBarToggleSolo(layer) => self.toggle_layer_solo(layer),
            Message::LaneBarToggleLock(layer) => self.toggle_layer_lock(layer),
            Message::TimelineKeySelect(op) => self.apply_key_selection(op),
            Message::TimelineDeleteSelectedKeys => self.delete_selected_keys(),
            Message::InspectorValuePressed(field) => self.start_field_drag(field),
            Message::InspectorPointerMoved(point) => self.continue_field_drag(point),
            Message::InspectorPointerReleased => {
                task = self.finish_field_drag();
            }
            Message::KeyboardModifiersChanged(modifiers) => self.keyboard_modifiers = modifiers,
            Message::EscapePressed => self.cancel_inspector_interaction(),
            Message::ToggleSettingsPanel => self.settings_panel_open = !self.settings_panel_open,
            Message::ToggleCheckerboard => self.checkerboard = !self.checkerboard,
            Message::SettingsBackgroundPreset(preset) => self.apply_background_preset(preset),
            Message::SettingsBackgroundChannelInput(channel, text) => {
                self.background_draft = Some(BackgroundFieldDraft { channel, text });
            }
            Message::SettingsBackgroundChannelSubmit(channel) => {
                self.commit_background_channel(channel);
            }
            Message::UiScaleInput(text) => self.ui_scale_draft = Some(text),
            Message::UiScaleSubmit => self.commit_ui_scale(),
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
                    Ok(()) => self.session.selection = Some(id),
                    // 拒否は必ず出す。黙って消さない。
                    Err(error) => self.status = Some(format!("layer を置けない: {error}")),
                }
            }
        }
        self.refresh_frame();
        task
    }

    /// 落ちてきた path を素材として受ける。
    ///
    /// **開けない物は理由つきで飛ばす**(M2)。黙って消すと利用者は
    /// 「落としたのに何も起きない」としか分からない。
    fn admit(&mut self, paths: Vec<std::path::PathBuf>) {
        let mut intents = Vec::new();
        let mut rejected = Vec::new();
        let mut next = self.next_layer_id();

        let comp_duration = self.comp_duration();
        let start = self.session.playhead;
        let _ = start;

        for path in paths {
            let text = path.to_string_lossy().into_owned();
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
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or(text);
                    rejected.push(format!("{name}: {error}"));
                }
            }
        }

        // 落とした分は**まとめて1 undo**(1操作 = 1 undo)。
        if !intents.is_empty() {
            if let Err(error) = self.doc.apply_all(intents) {
                rejected.push(format!("置けなかった: {error}"));
            }
        }
        if !rejected.is_empty() {
            self.status = Some(format!(
                "受け取れない素材 {}件 — {}",
                rejected.len(),
                rejected.join(" / ")
            ));
        }
    }

    /// 今の playhead を comp の fps で時刻へ写す。comp が無い/fps が壊れているなら
    /// `None`(M16: panic しない)。
    fn time_at_playhead(&self) -> Option<RationalTime> {
        let composition = self.doc.view().composition().ok().flatten()?;
        RationalTime::try_from_frame(self.session.playhead, composition.fps).ok()
    }

    /// Inspector の Transform 行 — 下書きを確定して1回の `Intent::SetTrack` を出す
    /// (1 gesture = 1 undo)。数値として読めない・選択が無い等は**黙って消さず**
    /// status 帯へ理由を出す(M13)。
    fn commit_inspector_field(&mut self, field: TransformField) {
        let Some(draft) = self.inspector_field_draft.take() else {
            return;
        };
        if draft.field != field {
            // 別の field の submit(起こらないはずだが、安全側で下書きを戻す)。
            self.inspector_field_draft = Some(draft);
            return;
        }
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(input) = inspector_pane::parse_number(&draft.text) else {
            self.status = Some(format!("数値として読めない: {}", draft.text));
            return;
        };
        let Ok(property) = inspector_pane::property_id(field) else {
            self.status = Some("property を作れない".to_owned());
            return;
        };

        // 編集不可(animated = 2キー以上)の field は、UI が control を出していない
        // はずだが、**書き口自体でも二重に拒む**(M13/Q0 — chrome と書き口の食い違いを
        // 構造的に作らない)。
        let store = self.doc.view();
        if let Ok(Some(track)) = store.track(layer, &property) {
            if track.keys().len() > 1 {
                self.status = Some("animated な property はこの第1波では編集できない".to_owned());
                return;
            }
        }

        let t = self.time_at_playhead().unwrap_or(RationalTime::ZERO);
        let current_vec2 = match store.value_at(layer, &property, t) {
            Ok(Some(motolii_store::Value::Vec2(v))) => v,
            _ => inspector_pane::default_vec2(field),
        };
        let value = inspector_pane::next_value(field, input, current_vec2);
        let track = inspector_pane::single_hold_track(value);
        if let Err(error) = self.doc.apply(Intent::SetTrack {
            layer,
            property,
            track,
        }) {
            self.status = Some(format!("値を書けない: {error}"));
        }
    }

    /// Attrs の Name 欄 — 下書きを確定して1回の `Intent::SetAttrs` を出す。
    fn commit_inspector_name(&mut self) {
        let Some(text) = self.inspector_name_draft.take() else {
            return;
        };
        let Some(layer) = self.session.selection else {
            return;
        };
        let patch = LayerAttrsPatch {
            name: Some(text),
            ..Default::default()
        };
        if let Err(error) = self.doc.apply(Intent::SetAttrs { layer, patch }) {
            self.status = Some(format!("名前を書けない: {error}"));
        }
    }

    /// Attrs の Hidden トグル — 即 `Intent::SetAttrs` を1回出す(下書きを経由しない)。
    fn toggle_inspector_hidden(&mut self) {
        let Some(layer) = self.session.selection else {
            return;
        };
        self.toggle_layer_hidden(layer);
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

    /// M glyph。`LayerAttrs.hidden` をトグルする(`InspectorToggleHidden` と
    /// 同じ書き口 — 対象の layer が違うだけ)。
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

    // ---- Timeline property 行(キー行、第2波 T3・裁定148/151) ----

    /// キー選択の確定。`timeline::key_rows::update` は「どのキーを・どの操作で」
    /// までしか判定しない(canvas 側は Document/Session を直接書けない、mod
    /// doc の背骨どおり)ので、`Session::selected_keys`/`key_anchor` の実際の
    /// 読み書きはここ(唯一の書き口)で行う。
    fn apply_key_selection(&mut self, op: timeline_pane::KeySelectionOp) {
        use timeline_pane::KeySelectionOp;
        match op {
            KeySelectionOp::Single(key) => {
                self.session.selected_keys = vec![key.clone()];
                self.session.key_anchor = Some(key);
            }
            KeySelectionOp::Toggle(key) => {
                if let Some(pos) = self.session.selected_keys.iter().position(|k| *k == key) {
                    self.session.selected_keys.remove(pos);
                } else {
                    self.session.selected_keys.push(key.clone());
                }
                self.session.key_anchor = Some(key);
            }
            KeySelectionOp::Range(key) => {
                let Some(anchor) = self.session.key_anchor.clone() else {
                    // 基点が無ければ単独選択と同じ扱いへ安全側で倒す
                    // (正典 §4 の「Shift=anchor から」— anchor が無い最初の
                    // クリックは単独扱いにする既存の行選択と同じ考え方)。
                    self.session.selected_keys = vec![key.clone()];
                    self.session.key_anchor = Some(key);
                    return;
                };
                let fps = self.composition().map(|c| c.fps);
                let rows = timeline_pane::property_rows(&self.doc.view(), &self.session, fps);
                let order = timeline_pane::key_order(&rows);
                let anchor_pos = order.iter().position(|k| *k == anchor);
                let clicked_pos = order.iter().position(|k| *k == key);
                match (anchor_pos, clicked_pos) {
                    (Some(a), Some(c)) => {
                        let (lo, hi) = if a <= c { (a, c) } else { (c, a) };
                        self.session.selected_keys = order[lo..=hi].to_vec();
                    }
                    _ => {
                        // anchor/clicked のどちらかが今の property_rows に無い
                        // (行の表示が変わった等) — 黙って壊れた選択のまま
                        // 進めるより単独選択へ安全側で倒す(M16)。
                        self.session.selected_keys = vec![key];
                    }
                }
                // anchor は不変 — 同じ基点から Shift 連打で範囲を伸縮できる。
            }
        }
    }

    /// 選択中のキーを消す(正典 §3「Delete はキー選択が層選択より優先」)。
    /// property ごとにまとめて読み直し、選択されたフレームだけを落とした
    /// `KeyframeTrack` を1回の `apply_all` で書き戻す — **1操作 = 1 undo**
    /// (`AddLayer` と同じ「まとめて1回」の形)。選択が空なら no-op。
    fn delete_selected_keys(&mut self) {
        if self.session.selected_keys.is_empty() {
            return;
        }
        let keys = std::mem::take(&mut self.session.selected_keys);
        self.session.key_anchor = None;
        let Some(composition) = self.composition() else {
            return;
        };
        let fps = composition.fps;

        let mut groups: std::collections::BTreeMap<(LayerId, PropertyId), Vec<i64>> =
            std::collections::BTreeMap::new();
        for key in keys {
            groups.entry((key.layer, key.property)).or_default().push(key.frame);
        }

        let store = self.doc.view();
        let mut intents = Vec::new();
        for ((layer, property), frames) in groups {
            let Ok(Some(track)) = store.track(layer, &property) else {
                continue;
            };
            let mut new_track = KeyframeTrack::new();
            for existing in track.keys() {
                let Ok(frame) = existing.t.try_to_frame_round(fps) else {
                    continue;
                };
                if frames.contains(&frame) {
                    continue; // 選択されたキーは書き戻さない = 削除。
                }
                new_track.insert(existing.clone());
            }
            intents.push(Intent::SetTrack { layer, property, track: new_track });
        }
        drop(store);

        if !intents.is_empty() {
            if let Err(error) = self.doc.apply_all(intents) {
                self.status = Some(format!("キーを消せない: {error}"));
            }
        }
    }

    // ---- Settings パネル(タスク#18) ----

    /// 背景色プリセット — 現在の `Composition` を読み、`background` だけ書き換えて
    /// 丸ごと書き戻す(read-modify-write、`Intent::SetComposition` は丸ごと置換の
    /// intent なので width/height/fps/duration_frames を巻き込まないよう毎回読む)。
    fn apply_background_preset(&mut self, preset: BackgroundPreset) {
        let Some(mut composition) = self.doc.view().composition().ok().flatten() else {
            self.status = Some("comp が無い".to_owned());
            return;
        };
        composition.background = settings_pane::preset_rgba(preset);
        if let Err(error) = self.doc.apply(Intent::SetComposition(composition)) {
            self.status = Some(format!("背景を書けない: {error}"));
        }
    }

    /// 背景 RGBA の1チャンネル — 下書きを確定して1回の `Intent::SetComposition`
    /// を出す(read-modify-write、他チャンネルは今の値のまま)。
    fn commit_background_channel(&mut self, channel: BackgroundChannel) {
        let Some(draft) = self.background_draft.take() else {
            return;
        };
        if draft.channel != channel {
            self.background_draft = Some(draft);
            return;
        }
        let Some(mut composition) = self.doc.view().composition().ok().flatten() else {
            self.status = Some("comp が無い".to_owned());
            return;
        };
        let Some(value_0_255) = settings_pane::parse_channel_u8(&draft.text) else {
            self.status = Some(format!("数値として読めない: {}", draft.text));
            return;
        };
        composition.background[channel.index()] = value_0_255 / 255.0;
        if let Err(error) = self.doc.apply(Intent::SetComposition(composition)) {
            self.status = Some(format!("背景を書けない: {error}"));
        }
    }

    /// ui_scale(%)欄 — 下書きを確定して 50..200 にクランプ、`Tokens`/`Dimensions`
    /// を更新する。**Document を経由しない**(`ui_scale` は既存の置き場どおり
    /// トークン扱い、undo 対象ではない)。debug ビルドでは正本 JSON へも書き戻す —
    /// 失敗しても in-memory の値は既に更新済みなので、画面上は反映される
    /// (書き込み失敗は status 帯へ理由を出すだけで機能は止めない、M16)。
    fn commit_ui_scale(&mut self) {
        let Some(text) = self.ui_scale_draft.take() else {
            return;
        };
        let Some(ui_scale) = settings_pane::parse_ui_scale_percent(&text) else {
            self.status = Some(format!("数値として読めない: {text}"));
            return;
        };
        self.tokens.dims.ui_scale = ui_scale;
        self.tokens.ui_scale = ui_scale;
        if let Err(error) = tokens::save_ui_scale(ui_scale) {
            self.status = Some(format!("ui_scale を保存できない: {error}"));
        }
    }

    // ---- Inspector の drag-to-scrub ----

    /// 値セルの press — click か drag かはまだ未確定
    /// (`FieldDragState::origin_x` が `None` のまま)。選択なし・animated(編集
    /// 不可)・対応する field が投影に無い、のいずれも黙って無視
    /// (`mouse_area` の `on_press` は常にこの Message を出すが、UI がそもそも
    /// animated field には draggable なセルを出していない — `commit_inspector_field`
    /// と同じ二重の柵)。
    fn start_field_drag(&mut self, field: TransformField) {
        if self.inspector_drag.is_some() {
            return; // 既に別の drag が進行中 — 多重起動しない
        }
        let Some(layer) = self.session.selection else {
            return;
        };
        let Some(selection) = self.inspector_selection() else {
            return;
        };
        let Some((start_value, current_vec2)) = inspector_pane::drag_origin(&selection, field)
        else {
            return;
        };
        self.inspector_drag = Some(FieldDragState {
            field,
            layer,
            start_value,
            current_vec2,
            origin_x: None,
            moved: false,
            last_value: None,
        });
    }

    /// window 全体の cursor 移動。drag が armed/dragging でなければ即 no-op。
    /// **1px = 感度表の刻み**(`inspector_pane::dragged_value`)。press 直後の
    /// 最初の move は基準点を確定するだけで値は動かさない(そうしないと press
    /// した瞬間の sub-pixel な揺れで値が動く)。
    ///
    /// **transient overlay(`Document::set_transient`)を毎 move 呼ぶだけ** —
    /// `edit timeline` には一切触れないので、undo/redo の意味論(`revision()`)は
    /// drag 中ずっと不変。Stage・Inspector セルの「ドラッグ中の即応」は
    /// `refresh_frame` が `display_revision()`(履歴 + overlay 世代)を見て
    /// 再描画することで出る。
    fn continue_field_drag(&mut self, point: iced::Point) {
        let Some(drag) = self.inspector_drag.as_mut() else {
            return;
        };
        let Some(origin_x) = drag.origin_x else {
            drag.origin_x = Some(point.x);
            return;
        };

        let delta_px = point.x - origin_x;
        if delta_px == 0.0 && !drag.moved {
            return; // まだ実質的に動いていない — click 候補のまま据え置く
        }

        let field = drag.field;
        let layer = drag.layer;
        let start_value = drag.start_value;
        let current_vec2 = drag.current_vec2;
        let fine = self.keyboard_modifiers.shift();

        let Ok(property) = inspector_pane::property_id(field) else {
            return;
        };
        let new_display = inspector_pane::dragged_value(field, start_value, delta_px, fine);
        let value = inspector_pane::next_value(field, new_display, current_vec2);

        self.doc.set_transient(layer, property, value.clone());
        if let Some(drag) = self.inspector_drag.as_mut() {
            drag.moved = true;
            drag.last_value = Some(value);
        }
    }

    /// 左クリック release(window 全体から — `mouse_area` 自身の `on_release` は
    /// bounds を出た drag を捉えられないので使わない)。**drag が実際に動いて
    /// いたら確定**: 最後の transient 値そのものを1回の本編集 `Intent` として
    /// `apply` してから `clear_transient`(1 gesture = 1 undo、overlay を残さない)。
    /// 動いていなければ click として type 編集へ切り替える。
    fn finish_field_drag(&mut self) -> Task<Message> {
        let Some(drag) = self.inspector_drag.take() else {
            return Task::none();
        };
        if !drag.moved {
            return self.enter_field_editing(drag.field);
        }
        let Ok(property) = inspector_pane::property_id(drag.field) else {
            // 起こらないはず(`moved` は property_id が通った move でしか立たない)
            // だが、安全側で overlay だけは残さず抜ける実害は無い(次の press で
            // 上書きされる)。
            return Task::none();
        };
        if let Some(value) = drag.last_value {
            let track = inspector_pane::single_hold_track(value);
            if let Err(error) = self.doc.apply(Intent::SetTrack {
                layer: drag.layer,
                property: property.clone(),
                track,
            }) {
                self.status = Some(format!("値を書けない: {error}"));
            }
        }
        self.doc.clear_transient(drag.layer, &property);
        Task::none()
    }

    /// click(ドラッグせず release)→ type 編集。下書きを立て、text_input へ
    /// フォーカスを戻す(値セルは編集していない間は `mouse_area` + 静止
    /// `text` なので、click 直後にはまだ text_input が木に無く自動フォーカス
    /// されない — 明示的な focus task が要る)。
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
    /// drag の復元は **`clear_transient` だけ**でよい — overlay は edit timeline に
    /// 一切触れていないので、undo/redo 履歴は最初から無傷(旧実装が抱えていた
    /// 「同じ値で1回上書きしてから undo」という無害化ワークアラウンドは不要になった
    /// — `Document` に「squash」API が無いことが理由で存在した迂回であり、transient
    /// overlay 自体が squash を要らなくする)。
    fn cancel_inspector_interaction(&mut self) {
        if let Some(drag) = self.inspector_drag.take() {
            if drag.moved {
                if let Ok(property) = inspector_pane::property_id(drag.field) {
                    self.doc.clear_transient(drag.layer, &property);
                }
            }
            return;
        }
        if self.inspector_field_draft.take().is_some() {
            return;
        }
        self.inspector_name_draft = None;
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

    pub fn view(&self) -> Element<'_, Message> {
        // pane が受け取るのは不変の投影だけ。
        let store = self.doc.view();
        // `ui_scale` 適用済み(`Shell::dims` — 適用点1箇所)。
        let dims = self.dims();
        let colors = self.tokens.colors;
        let timeline = timeline_pane::TimelinePane::new(
            &store,
            &self.session,
            dims,
            colors,
            self.keyboard_modifiers,
        );
        // Inspector は canvas を使わない標準 widget 構成(inspector_pane.rs 冒頭の
        // doc comment)なので、投影自体が `Element<'static, _>` を返す — Stage の
        // `self.frame` を借りる `stage_pane` と同じ `row!` に同居できる(共変性)。
        let inspector_selection = inspector_pane::project(&store, &self.session)
            .ok()
            .flatten();
        let inspector = inspector_pane::view(
            inspector_selection.as_ref(),
            self.inspector_field_draft.as_ref(),
            self.inspector_name_draft.as_deref(),
            dims,
            colors,
        );

        // Settings パネル(タスク#18)。**表示だけの分岐** — 開いていなければ
        // 木に一切現れない(Q0: 効かない chrome を並べない、閉じている間は
        // 下書き入力欄も存在しないので誤操作の的にならない)。
        let mut layout = column![self.header()];
        if self.settings_panel_open {
            layout = layout.push(settings_pane::view(
                self.composition().as_ref(),
                self.background_draft.as_ref(),
                self.tokens.ui_scale,
                self.ui_scale_draft.as_deref(),
                self.checkerboard,
                dims,
                colors,
            ));
        }

        layout
            .push(
                row![inspector, stage_pane(self.frame.as_ref(), dims, colors)]
                    .spacing(dims.spacing_m)
                    .height(Length::FillPortion(3)),
            )
            .push(timeline.view())
            .push(transport(&self.session, &store, dims, colors))
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
                .on_press(Message::ToggleSettingsPanel),
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
        let colors = self.tokens.colors;

        if let Some(frame) = &self.frame {
            if frame.revision == revision && frame.playhead == playhead {
                if frame.checkerboard == checkerboard {
                    return;
                }
                let width = frame.width;
                let height = frame.height;
                let preview = self.checkerboard_preview_source(checkerboard, playhead);
                let (handle, handle_bytes) = match &preview {
                    Some(preview) => build_stage_handle(width, height, preview, true, colors),
                    None => {
                        let frame = self.frame.as_ref().expect("直前の if let で確認済み");
                        build_stage_handle(width, height, &frame.rgba, false, colors)
                    }
                };
                metrics::record_handle_creation(handle_bytes);
                if let Some(frame) = self.frame.as_mut() {
                    frame.handle = handle;
                    frame.checkerboard = checkerboard;
                    frame.checkerboard_preview_rgba = preview;
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
        let render_result = self.engine.render_frame(&self.doc.view(), t);
        metrics::record_render_frame(render_start.elapsed());
        match render_result {
            Ok(rgba) => {
                let preview = if checkerboard {
                    self.checkerboard_preview_source(true, playhead)
                } else {
                    None
                };
                let (handle, handle_bytes) = match &preview {
                    Some(preview) => build_stage_handle(
                        composition.width,
                        composition.height,
                        preview,
                        true,
                        colors,
                    ),
                    None => build_stage_handle(
                        composition.width,
                        composition.height,
                        &rgba,
                        false,
                        colors,
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
                    checkerboard_preview_rgba: preview,
                    checkerboard,
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
}

/// Stage 表示用の Handle を作る唯一の場所。`stage_handle_rgba` で縮め、
/// **市松が有効なら display 用の複製にだけ**
/// [`settings_pane::composite_checkerboard`] を乗せる — 呼び出し側が渡す
/// `full_rgba` 自体は一切変更しない。
///
/// `full_rgba` は呼び出し側(`refresh_frame`)が選ぶ: 市松 OFF なら
/// `RenderedFrame::rgba`(背景込みの export 真値)、市松 ON なら
/// `Engine::render_frame_without_background` の結果(裁定141、背景を敷かない
/// 可視化専用の合成)— どちらの場合も、export/screenshot が読む生値
/// (`RenderedFrame::rgba`)自体はここでは一切変更しない。
fn build_stage_handle(
    width: u32,
    height: u32,
    full_rgba: &[u8],
    checkerboard: bool,
    colors: Colors,
) -> (image::Handle, usize) {
    let (handle_width, handle_height, mut handle_rgba) = stage_handle_rgba(width, height, full_rgba);
    if checkerboard {
        settings_pane::composite_checkerboard(handle_width, handle_height, &mut handle_rgba, colors);
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
/// `iced::event::listen_with` を選んだ理由: `_status` を見ずに常に拾う。
/// `iced::keyboard::listen()`(Ignored 限定)だと、typing 中の text_input は
/// Escape を自分で `shell.capture_event()` する(`iced_widget::text_input`
/// 実測)ので、typing の Esc-cancel に使いたい場合に届かなくなる。
fn inspector_pointer_event(
    event: iced::Event,
    _status: iced::event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    match event {
        iced::Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::InspectorPointerMoved(position))
        }
        iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::InspectorPointerReleased)
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
        }) => Some(Message::TimelineDeleteSelectedKeys),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// pane — **`StoreView`(不変)・`&Session`・`Tokens`(読み取り専用の意匠値)しか
// 取らない**。書ける物を持たない。`timeline_pane::TimelinePane` も同じ制約。
// ---------------------------------------------------------------------------

fn stage_pane(
    frame: Option<&RenderedFrame>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'_, Message> {
    let body: Element<'_, Message> = match frame {
        Some(frame) => image(frame.handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => text("comp がまだ無い")
            .size(dims.body_text)
            .color(colors.text_muted)
            .into(),
    };
    // letterbox は neutral dark(D8: 装飾 gradient 禁止・余白は neutral)。raw 値ではなく
    // token 経由の面色 + 罫線幅。
    // **高さは `Length::Fill`**(Inspector と並ぶ `row!` の中にいるため、以前の
    // `FillPortion(3)` は `Shell::view` 側のその `row!` 自身が持つ — 2箇所で
    // portion を重ねて割合をずらさない)。
    container(body)
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

fn transport<'a>(
    session: &Session,
    store: &StoreView<'a>,
    dims: Dimensions,
    colors: Colors,
) -> Element<'a, Message> {
    let last = store
        .composition()
        .ok()
        .flatten()
        .map(|c| (c.duration_frames - 1).max(0) as i32)
        .unwrap_or(0);

    row![
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

/// header の3ボタン共通スタイル。**意味色ロール経由**(raw 値の直書き禁止) —
/// hover/pressed/disabled をそれぞれ別ロールで塗り分ける(状態: hover・選択・無効)。
/// `pub(crate)`: `settings_pane` のプリセット/市松トグルボタンも同じ意味色
/// ロールを使う — 状態ごとに専用の色を新設しない。
pub(crate) fn button_style(dims: Dimensions, colors: Colors, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => colors.surface_hover,
        button::Status::Pressed => colors.state_selected,
        button::Status::Disabled => colors.surface_panel,
        button::Status::Active => colors.surface_raised,
    };
    let text_color = if status == button::Status::Disabled {
        colors.state_disabled
    } else {
        colors.text_primary
    };
    button::Style {
        background: Some(iced::Background::Color(background)),
        text_color,
        border: iced::Border {
            color: colors.border_default,
            width: dims.border_width,
            radius: 0.0.into(),
        },
        ..button::Style::default()
    }
}
