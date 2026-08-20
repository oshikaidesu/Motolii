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
    Composition, Document, Intent, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming,
    RationalTime, Revision, StoreView,
};

pub mod fixture;
pub mod inspector_pane;
pub mod screenshot;
pub mod timeline_pane;
pub mod tokens;

use inspector_pane::{FieldDraft, TransformField};

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
}

impl Default for Session {
    fn default() -> Self {
        Self {
            playhead: 0,
            selection: None,
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
}

/// 描き上がった1フレーム。**Document の写しではなく、描画の成果物**。
///
/// いつ捨てるかは [`Document::revision`] が決める(store 世代 + edit 位置)。
/// front が「前回の値」を自分で持たないための口がこれ。
struct RenderedFrame {
    revision: Revision,
    playhead: i64,
    width: u32,
    height: u32,
    handle: image::Handle,
    /// `handle` と同じ画素の生 RGBA。**screenshot 器具専用**(`screenshot.rs`)—
    /// 通常描画は `handle` だけで足りる(iced の `image::Handle` から画素を
    /// 取り戻す公開 API が無いため、この用途だけのために複製して持つ)。
    rgba: Vec<u8>,
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
            },
            engine,
            frame: None,
            status: Some(built.status),
            pending_drops: Vec::new(),
            tokens: Tokens::load(),
            inspector_field_draft: None,
            inspector_name_draft: None,
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
        iced::Subscription::batch([window, tokens])
    }

    /// **唯一の書き口**。ここ以外に `doc.apply` を呼ぶ場所を作らない。
    pub fn update(&mut self, message: Message) -> Task<Message> {
        self.status = None;
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
        Task::none()
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

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    /// 描き上がったフレームの識別。同じなら描き直していない。
    pub fn frame_token(&self) -> Option<(Revision, i64)> {
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

    /// 描き上がった Stage フレームの生 RGBA。**screenshot 器具専用**
    /// (`screenshot.rs`)— 通常描画は `image::Handle` を持つ `stage_pane` を通る。
    pub fn frame_rgba(&self) -> Option<(u32, u32, &[u8])> {
        self.frame
            .as_ref()
            .map(|frame| (frame.width, frame.height, frame.rgba.as_slice()))
    }

    /// 今の Timeline の行。運転席が「層3枚の行が立つ」「選択が行と一致する」を
    /// 確かめる口(pane 自身が使う投影と同じ関数を呼ぶ)。
    pub fn timeline_rows(&self) -> Vec<timeline_pane::RowProjection> {
        timeline_pane::rows(&self.doc.view(), &self.session)
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

    /// 今のデザイン値。運転席がトークン再読込を確かめる口。
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn view(&self) -> Element<'_, Message> {
        // pane が受け取るのは不変の投影だけ。
        let store = self.doc.view();
        let dims = self.tokens.dims;
        let colors = self.tokens.colors;
        let timeline = timeline_pane::TimelinePane::new(&store, &self.session, dims, colors);
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

        column![
            self.header(),
            row![inspector, stage_pane(self.frame.as_ref(), dims, colors)]
                .spacing(dims.spacing_m)
                .height(Length::FillPortion(3)),
            timeline.view(),
            transport(&self.session, &store, dims, colors),
            status_band(self.status.as_deref(), &self.doc, dims, colors),
        ]
        .spacing(dims.spacing_m)
        .padding(dims.spacing_l)
        .into()
    }

    fn header(&self) -> Element<'_, Message> {
        let dims = self.tokens.dims;
        let colors = self.tokens.colors;
        row![
            button(text("Undo").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press_maybe(self.doc.can_undo().then_some(Message::Undo)),
            button(text("Redo").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press_maybe(self.doc.can_redo().then_some(Message::Redo)),
            button(text("+ Layer").size(dims.body_text))
                .style(move |_theme, status| button_style(dims, colors, status))
                .on_press(Message::AddLayer),
        ]
        .spacing(dims.spacing_m)
        .height(Length::Fixed(dims.panel_header_height))
        .align_y(iced::alignment::Vertical::Center)
        .into()
    }

    /// 採番の正本は store 側([`StoreView::next_layer_id`])。**墓標を含む最大 id + 1**
    /// を返すので、削除した layer の id が再利用されない(2026-08-20 の敵対的レビュー修正)。
    fn next_layer_id(&self) -> u64 {
        self.doc.view().next_layer_id()
    }

    /// Document か再生位置が変わった時だけ描き直す。
    /// 判定は `revision()` — front が「前回の Document」を自分で持たないため。
    fn refresh_frame(&mut self) {
        let revision = self.doc.revision();
        let playhead = self.session.playhead;
        if let Some(frame) = &self.frame {
            if frame.revision == revision && frame.playhead == playhead {
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
                let (handle_width, handle_height, handle_rgba) =
                    stage_handle_rgba(composition.width, composition.height, &rgba);
                let handle_bytes = handle_rgba.len();
                let handle = image::Handle::from_rgba(handle_width, handle_height, handle_rgba);
                metrics::record_handle_creation(handle_bytes);
                self.frame = Some(RenderedFrame {
                    revision,
                    playhead,
                    width: composition.width,
                    height: composition.height,
                    handle,
                    rgba,
                });
            }
            Err(error) => {
                // 絵が出せなくても**画面は空にしない**(M16)。理由は帯に出す。
                self.status = Some(format!("Stage を描けない: {error}"));
            }
        }
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
    text(message).size(dims.caption_text).color(color).into()
}

/// header の3ボタン共通スタイル。**意味色ロール経由**(raw 値の直書き禁止) —
/// hover/pressed/disabled をそれぞれ別ロールで塗り分ける(状態: hover・選択・無効)。
fn button_style(dims: Dimensions, colors: Colors, status: button::Status) -> button::Style {
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
