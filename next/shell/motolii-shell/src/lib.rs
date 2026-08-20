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
    Composition, Document, Intent, LayerId, LayerMeta, LayerSource, LayerTiming, RationalTime,
    Revision, StoreView,
};

pub mod timeline_pane;
pub mod tokens;

use tokens::Tokens;

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
}

/// 描き上がった1フレーム。**Document の写しではなく、描画の成果物**。
///
/// いつ捨てるかは [`Document::revision`] が決める(store 世代 + edit 位置)。
/// front が「前回の値」を自分で持たないための口がこれ。
struct RenderedFrame {
    revision: Revision,
    playhead: i64,
    handle: image::Handle,
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
            },
            Task::none(),
        )
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
            }
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
            self.status = Some(format!("受け取れない素材 {}件 — {}", rejected.len(), rejected.join(" / ")));
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

    /// 今の Timeline の行。運転席が「層3枚の行が立つ」「選択が行と一致する」を
    /// 確かめる口(pane 自身が使う投影と同じ関数を呼ぶ)。
    pub fn timeline_rows(&self) -> Vec<timeline_pane::RowProjection> {
        timeline_pane::rows(&self.doc.view(), &self.session)
    }

    /// 今のデザイン値。運転席がトークン再読込を確かめる口。
    pub fn tokens(&self) -> &Tokens {
        &self.tokens
    }

    pub fn view(&self) -> Element<'_, Message> {
        // pane が受け取るのは不変の投影だけ。
        let store = self.doc.view();
        let timeline = timeline_pane::TimelinePane::new(
            &store,
            &self.session,
            self.tokens.dims,
            self.tokens.colors,
        );

        column![
            self.header(),
            stage_pane(self.frame.as_ref()),
            timeline.view(),
            transport(&self.session, &store),
            status_band(self.status.as_deref(), &self.doc),
        ]
        .spacing(8)
        .padding(12)
        .into()
    }

    fn header(&self) -> Element<'_, Message> {
        row![
            button("Undo").on_press_maybe(self.doc.can_undo().then_some(Message::Undo)),
            button("Redo").on_press_maybe(self.doc.can_redo().then_some(Message::Redo)),
            button("+ Layer").on_press(Message::AddLayer),
        ]
        .spacing(8)
        .into()
    }

    fn next_layer_id(&self) -> u64 {
        self.doc
            .view()
            .layers()
            .last()
            .map(|last| last.0 + 1)
            .unwrap_or(1)
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

        match self.engine.render_frame(&self.doc.view(), t) {
            Ok(rgba) => {
                self.frame = Some(RenderedFrame {
                    revision,
                    playhead,
                    handle: image::Handle::from_rgba(composition.width, composition.height, rgba),
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

fn stage_pane(frame: Option<&RenderedFrame>) -> Element<'_, Message> {
    let body: Element<'_, Message> = match frame {
        Some(frame) => image(frame.handle.clone())
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => text("comp がまだ無い").into(),
    };
    container(body)
        .width(Length::Fill)
        .height(Length::FillPortion(3))
        .into()
}

fn transport<'a>(session: &Session, store: &StoreView<'a>) -> Element<'a, Message> {
    let last = store
        .composition()
        .ok()
        .flatten()
        .map(|c| (c.duration_frames - 1).max(0) as i32)
        .unwrap_or(0);

    row![
        text(format!("frame {}", session.playhead)),
        slider(0..=last, session.playhead as i32, |frame| {
            Message::ScrubTo(i64::from(frame))
        }),
    ]
    .spacing(8)
    .into()
}

fn status_band<'a>(status: Option<&str>, doc: &Document) -> Element<'a, Message> {
    let layers = doc.view().layers().len();
    let message = match status {
        Some(status) => status.to_owned(),
        None => format!("layer {layers} / edit {}", doc.edit_head()),
    };
    text(message).into()
}
