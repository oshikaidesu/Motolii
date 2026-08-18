//! Timeline pane の状態機械 — **ジェスチャは release までが preview、
//! release の1件だけが intent**。
//!
//! spike(`spikes/iced-rerun-embed-probe/timeline`)の「intent まで解決してから
//! Message にする」方針を製品へ持ち込んだ層である。canvas(`canvas.rs`)が
//! 生イベントを [`TimelineMsg`] に翻訳し、ここが2段で受ける:
//!
//! 1. [`TimelinePane::plan`] … この Message で **dispatch すべき `UiIntent` の列**を
//!    返す(自分は何も書かない)。dispatch するのは `Shell::update` だけである
//!    (フェンス: `tests/intent_gateway_fence.rs`)
//! 2. [`TimelinePane::note`] … dispatch が済んだ後の座席の状態(`TimelineCtx`)を
//!    見て、自分の view 状態(zoom / pan / scroll / 進行中ジェスチャ)を進める
//!
//! ## なぜ release までは intent にしないのか
//!
//! ドラッグ中の絵は pane の preview で持ち、Document は release まで触らない。
//! こうすると **Esc = 復元** が「まだ何も起きていない」の同義になり、
//! journal には利用者が確定した1手だけが載る(1 gesture = 1 intent = 1 Undo)。
//! これは `motolii_ui::timeline_move_gesture` / `timeline_trim_gesture` が
//! Skia 側で実証した transient lifecycle と同じ判断で、egui pane の
//! live-commit(毎フレーム D2 へ書き、Esc で undo する)とはここが違う。
//! 動きの意味(クランプ・吸着・キー追従)は egui 版の意味関数
//! (`semantics.rs` に移植)と、intent の後ろの `commit_drag`(共用)が守る。

use std::sync::Arc;

use motolii_core::Fps;
use motolii_doc::{Document, LayerId};
use motolii_ui::blitz_shell::{seconds_to_us, UiIntent};
use motolii_ui::timeline_editor::{TimelineView, TrimEdge};

use super::semantics::{
    clamped_move_delta, clamped_trim, frame_snapped, initial_view, move_targets,
};

/// canvas が出す **意図(intent)まで解決済み**の Message。生イベント中継は無い —
/// hit test も時間への換算も吸着も canvas 側(イベントパス)で済んでいる。
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineMsg {
    /// bar を掴んだ。`zone` で移動か trim かが既に決まっている。
    /// `at_seconds` は**スナップ前**の位置(掴んだ点と clip 頭のズレを保つのに要る)。
    BarGrabbed {
        layer: LayerId,
        zone: GrabZone,
        at_seconds: f32,
        /// Cmd 併用 = 選択に足す / 外す。
        additive: bool,
    },
    /// 左レールの行を押した = 選択(クリックの意味そのもの)。
    RowPicked { layer: LayerId, additive: bool },
    /// 何も無い所を押した = 選択解除(Cmd 併用なら何もしない)。
    EmptyPressed { additive: bool },
    /// ルーラを押した = スクラブ開始。
    ScrubStarted { at_seconds: f32 },
    /// ドラッグ中の移動。canvas は**ジェスチャ中しかこれを出さない**。
    /// Move / Trim では canvas が吸着済みの時刻を運ぶ。
    PointerMoved { at_seconds: f32 },
    /// ボタンを離した = 進行中ジェスチャの確定(ここで初めて intent が出る)。
    PointerReleased,
    /// Esc = 進行中ジェスチャの破棄。**Document は最初から触っていない**ので
    /// 復元は preview を捨てるだけで成立する。
    GestureCancelled,
    /// Delete / Backspace。
    DeletePressed,
    /// Cmd+Z / 帯の Undo ボタン。
    UndoPressed,
    /// Shift+Cmd+Z / 帯の Redo ボタン。
    RedoPressed,
    /// ← / →(Shift で ±10 コマ)。
    PlayheadStepped { frames: i32 },
    /// Cmd+ホイール = 横 zoom。`anchor_seconds` の時刻が動かない。
    ZoomedAt { anchor_seconds: f32, factor: f32 },
    /// Shift+ホイール / 横スワイプ = 横パン(秒)。
    PannedX { delta_seconds: f32 },
    /// 素のホイール = 縦スクロール。`max` は canvas が寸法から出した上限。
    ScrolledY { delta_px: f32, max: f32 },
    /// 修飾キーの状態。`WheelScrolled` が modifiers を運ばないので追いかける
    /// (spike の詰まった箇所その1と同じ回避)。
    ModifiersChanged(iced::keyboard::Modifiers),
}

/// bar のどこを掴んだか(Message が運ぶ側)。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrabZone {
    Body,
    Edge(TrimEdge),
}

/// 進行中のジェスチャ。**確定前の preview だけ**を持つ(Document の複製ではない)。
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineDrag {
    /// 選択中の clip 群の移動。`targets` は掴んだ瞬間の (layer, start, end)。
    Move {
        grabbed: LayerId,
        grab_at: f32,
        targets: Vec<(LayerId, f32, f32)>,
        /// クランプ・吸着済みの差分(秒)。絵はこれを足して描く。
        preview_delta: f32,
    },
    /// 端の trim。`preview` はいまの端の時刻。
    Trim {
        layer: LayerId,
        edge: TrimEdge,
        span: (f32, f32),
        preview: f32,
    },
    /// playhead のスクラブ。`preview` はフレーム吸着済み。
    Scrub { preview: f32 },
}

/// plan / note が座席から借りる読み取り一式。dispatch の前(`plan`)と後(`note`)で
/// 別々に採る — 押した瞬間の選択と、選択 intent が効いた後の選択は別物である。
#[derive(Clone)]
pub struct TimelineCtx {
    pub document: Arc<Document>,
    pub selected: Vec<LayerId>,
    pub playhead: f32,
}

impl TimelineCtx {
    pub fn comp_seconds(&self) -> f32 {
        self.document.composition.duration.as_seconds_f64() as f32
    }

    pub fn fps(&self) -> Fps {
        self.document.composition.fps
    }
}

/// Timeline pane の view 状態。**Project session の棚**(Document には入らない)。
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePane {
    pub view: TimelineView,
    pub scroll_y: f32,
    pub modifiers: iced::keyboard::Modifiers,
    pub drag: Option<TimelineDrag>,
    /// 最初の ctx で view を composition に合わせて据えたか。
    initialized: bool,
}

impl Default for TimelinePane {
    fn default() -> Self {
        Self {
            view: initial_view(super::semantics::TIMELINE_SECONDS),
            scroll_y: 0.0,
            modifiers: iced::keyboard::Modifiers::empty(),
            drag: None,
            initialized: false,
        }
    }
}

impl TimelinePane {
    /// 座り直したら view ごと最初から(別 project の zoom を引き継がない)。
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// この Message で dispatch すべき intent の列。**自分は何も書かない。**
    pub fn plan(&self, message: &TimelineMsg, ctx: &TimelineCtx) -> Vec<UiIntent> {
        match message {
            TimelineMsg::BarGrabbed {
                layer,
                zone,
                additive,
                ..
            } => match zone {
                // 端の掴みは選択を変えない(egui と同じ)。
                GrabZone::Edge(_) => Vec::new(),
                GrabZone::Body => press_selection_intents(&ctx.selected, *layer, *additive),
            },
            TimelineMsg::RowPicked { layer, additive } => vec![UiIntent::SelectLayer {
                layer: *layer,
                additive: *additive,
            }],
            TimelineMsg::EmptyPressed { additive } => {
                if !additive && !ctx.selected.is_empty() {
                    vec![UiIntent::ClearSelection]
                } else {
                    Vec::new()
                }
            }
            TimelineMsg::PointerReleased => match &self.drag {
                Some(TimelineDrag::Move {
                    grabbed,
                    grab_at,
                    preview_delta,
                    ..
                }) if *preview_delta != 0.0 => vec![UiIntent::MoveClips {
                    grabbed: *grabbed,
                    grab_at_us: seconds_to_us(*grab_at),
                    drop_at_us: seconds_to_us(*grab_at + *preview_delta),
                }],
                Some(TimelineDrag::Trim {
                    layer,
                    edge,
                    span,
                    preview,
                }) => {
                    let original = match edge {
                        TrimEdge::In => span.0,
                        TrimEdge::Out => span.1,
                    };
                    if *preview != original {
                        vec![UiIntent::TrimClip {
                            layer: *layer,
                            edge: *edge,
                            at_us: seconds_to_us(*preview),
                        }]
                    } else {
                        Vec::new()
                    }
                }
                Some(TimelineDrag::Scrub { preview }) => vec![UiIntent::SetPlayhead {
                    at_us: seconds_to_us(*preview),
                }],
                _ => Vec::new(),
            },
            TimelineMsg::DeletePressed => {
                if ctx.selected.is_empty() {
                    Vec::new()
                } else {
                    vec![UiIntent::DeleteSelection]
                }
            }
            TimelineMsg::UndoPressed => vec![UiIntent::Undo],
            TimelineMsg::RedoPressed => vec![UiIntent::Redo],
            TimelineMsg::PlayheadStepped { frames } => {
                vec![UiIntent::StepPlayhead { frames: *frames }]
            }
            // view の操作・preview の途中経過は intent にならない
            // (再現は Message 列 replay が持つ — `drive_timeline.rs` の oracle)。
            TimelineMsg::ScrubStarted { .. }
            | TimelineMsg::PointerMoved { .. }
            | TimelineMsg::GestureCancelled
            | TimelineMsg::ZoomedAt { .. }
            | TimelineMsg::PannedX { .. }
            | TimelineMsg::ScrolledY { .. }
            | TimelineMsg::ModifiersChanged(_) => Vec::new(),
        }
    }

    /// dispatch の後の状態遷移。`ctx` は **intent が効いた後**の座席。
    pub fn note(&mut self, message: &TimelineMsg, ctx: &TimelineCtx) {
        if !self.initialized {
            self.view = initial_view(ctx.comp_seconds());
            self.initialized = true;
        }
        let comp = ctx.comp_seconds();
        match message {
            TimelineMsg::BarGrabbed {
                layer,
                zone,
                at_seconds,
                ..
            } => match zone {
                GrabZone::Body => {
                    // Cmd+クリックが**選択を外した**場合、掴んだ物はもう選択に
                    // 居ない。ここでドラッグを始めると「外したはずの clip を
                    // 掴んだつもりで、残りの選択が動く」という嘘になるので始めない
                    // (spike と同じ判断)。
                    if !ctx.selected.contains(layer) {
                        self.drag = None;
                        return;
                    }
                    let targets = move_targets(&ctx.document, &ctx.selected);
                    if targets.is_empty() {
                        self.drag = None;
                        return;
                    }
                    self.drag = Some(TimelineDrag::Move {
                        grabbed: *layer,
                        grab_at: *at_seconds,
                        targets,
                        preview_delta: 0.0,
                    });
                }
                GrabZone::Edge(edge) => {
                    let Some((start, end, is_group)) =
                        super::semantics::bar_span(&ctx.document, *layer)
                    else {
                        return;
                    };
                    if is_group {
                        // hit test は Group に端を出さないが、防波堤をここにも置く。
                        return;
                    }
                    let preview = match edge {
                        TrimEdge::In => start,
                        TrimEdge::Out => end,
                    };
                    self.drag = Some(TimelineDrag::Trim {
                        layer: *layer,
                        edge: *edge,
                        span: (start, end),
                        preview,
                    });
                }
            },
            TimelineMsg::ScrubStarted { at_seconds } => {
                self.drag = Some(TimelineDrag::Scrub {
                    preview: frame_snapped(at_seconds.clamp(0.0, comp), ctx.fps()),
                });
            }
            TimelineMsg::PointerMoved { at_seconds } => match &mut self.drag {
                Some(TimelineDrag::Move {
                    grab_at,
                    targets,
                    preview_delta,
                    ..
                }) => {
                    *preview_delta = clamped_move_delta(targets, comp, *at_seconds - *grab_at);
                }
                Some(TimelineDrag::Trim {
                    edge,
                    span,
                    preview,
                    ..
                }) => {
                    *preview = clamped_trim(*edge, *span, comp, ctx.fps(), *at_seconds);
                }
                Some(TimelineDrag::Scrub { preview }) => {
                    *preview = frame_snapped(at_seconds.clamp(0.0, comp), ctx.fps());
                }
                None => {}
            },
            TimelineMsg::PointerReleased | TimelineMsg::GestureCancelled => {
                self.drag = None;
            }
            TimelineMsg::ZoomedAt {
                anchor_seconds,
                factor,
            } => {
                self.view = self.view.zoom_at(*anchor_seconds, *factor, comp);
            }
            TimelineMsg::PannedX { delta_seconds } => {
                self.view = self.view.pan(*delta_seconds, comp);
            }
            TimelineMsg::ScrolledY { delta_px, max } => {
                self.scroll_y = (self.scroll_y + delta_px).clamp(0.0, max.max(0.0));
            }
            TimelineMsg::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            }
            // 選択・削除・Undo/Redo・コマ送りは座席側の状態で、pane は何も控えない。
            TimelineMsg::RowPicked { .. }
            | TimelineMsg::EmptyPressed { .. }
            | TimelineMsg::DeletePressed
            | TimelineMsg::UndoPressed
            | TimelineMsg::RedoPressed
            | TimelineMsg::PlayheadStepped { .. } => {}
        }
    }

    /// preview 中の playhead(スクラブ中はそれ、でなければ `None`)。絵が読む。
    pub fn scrub_preview(&self) -> Option<f32> {
        match &self.drag {
            Some(TimelineDrag::Scrub { preview }) => Some(*preview),
            _ => None,
        }
    }

    /// ジェスチャ中に**動いている**layer の集合(吸着の除外と preview 描画が読む)。
    pub fn moving_layers(&self) -> Vec<LayerId> {
        match &self.drag {
            Some(TimelineDrag::Move { targets, .. }) => {
                targets.iter().map(|(layer, _, _)| *layer).collect()
            }
            Some(TimelineDrag::Trim { layer, .. }) => vec![*layer],
            _ => Vec::new(),
        }
    }
}

/// bar 本体を押した瞬間の選択の意味(spike の `apply_selection` と同じ):
/// - `Cmd` … 足す / 外すのトグル
/// - 素で、まだ選ばれていない … その1つに置き換える
/// - 素で、**既に選ばれている … 何もしない**(複数選択のまま引きずるため)
pub fn press_selection_intents(
    selected: &[LayerId],
    layer: LayerId,
    additive: bool,
) -> Vec<UiIntent> {
    if additive {
        vec![UiIntent::SelectLayer {
            layer,
            additive: true,
        }]
    } else if !selected.contains(&layer) {
        vec![UiIntent::SelectLayer {
            layer,
            additive: false,
        }]
    } else {
        Vec::new()
    }
}
