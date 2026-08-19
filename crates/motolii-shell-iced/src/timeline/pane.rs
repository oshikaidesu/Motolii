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
use motolii_ui::blitz_shell::{seconds_to_us, UiIntent, UiItemFlag};
use motolii_ui::timeline_editor::{TimelineView, TrimEdge};
use motolii_ui::timeline_rows::TimelineFoldState;

use super::semantics::{
    clamped_move_delta, clamped_trim, frame_snapped, initial_view, move_targets,
    row_effective_lock, row_own_lock,
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
    /// 行の M / S ボタンを押した = `UiIntent::ToggleItemFlag` への結線そのもの
    /// (新しい intent は作らない。M-4b が既に持っている1本を Timeline からも呼ぶ)。
    FlagPressed { layer: LayerId, flag: UiItemFlag },
    /// ARRANGEMENT 俯瞰帯を押した/引きずった = その時刻を中心に view を寄せる。
    /// **意味を持たない view chrome**(zoom/pan と同じ scope — intent にはならない)。
    OverviewSeek { at_seconds: f32 },
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

    // ---- 構造操作(2026-08-19)。畳み開閉は Document に入らない session 状態
    //      なので intent にならない — zoom/pan と同じ scope(M-3 の決定と整合)。
    //      Rename / Lock / Group / Ungroup は最終的に `UiIntent` を1件だけ出す。
    /// 子(`params: false`)/ param(`params: true`)の開閉矢印を押した。
    /// `TimelinePane::fold` を直接動かす — intent は無い。
    FoldToggled { layer: LayerId, params: bool },
    /// 行名をダブルクリック(または Enter・単独選択時)= その場編集を始める。
    RenameStarted { layer: LayerId },
    /// 編集中バッファの1手ぶんの差し替え(1キー = 1つの新しい全文字列)。
    RenameEdited(String),
    /// Enter = 確定。`UiIntent::RenameLayer` が1件飛ぶ。
    RenameCommitted,
    /// Esc = 取消(rename 編集中だけを横取りする — ジェスチャの Esc とは別腕)。
    RenameCancelled,
    /// L ボタンを押した。**送るのは「いまの自分の lock の反対」**(Toggle ではなく
    /// 明示値の `UiIntent::SetLayerLock`)。
    LockPressed { layer: LayerId },
    /// Cmd+G。選択を1つの Group にまとめる。
    GroupPressed,
    /// Cmd+Shift+G。選ばれている Group を解く。
    UngroupPressed,
    /// Cmd+D。D2 に `prepare_duplicate_track_item` の口があるので実装する
    /// (RETURN 参照 — capsule は「口が無ければ見送る」だったが実測で在った)。
    DuplicatePressed,
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
    /// 行の畳み開閉。**Document には入らない**(zoom/pan と同じ scope — M-3 の
    /// 決定と整合)。`canvas.rs` の `scene()` がここを読んで `rows()` を作る。
    pub fold: TimelineFoldState,
    /// rename のその場編集バッファ。`Some((layer, 表示中の文字列))`。
    pub renaming: Option<(LayerId, String)>,
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
            fold: TimelineFoldState::default(),
            renaming: None,
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
                at_seconds,
                additive,
            } => {
                // **ロック中は掴む前に断る。** D2(`prepare_set_clip_start` /
                // `prepare_trim_clip_in/out`)は lock を検査しない — 拒否は
                // `TimelineEditor::begin_selected_clips_move` / `begin_trim` の
                // 層でしか起きないので、実際に1回そこを通してもらわないと
                // 理由が status 帯へ出ない。掴んだ瞬間に「進まない0幅の
                // move/trim」を1件飛ばし、断りをその場で journal ⇄ 帯へ写す
                // (2026-08-19 能力台帳の指摘: 掴めないことが掴む前に分かる
                // ようにする・拒否は無言にしない)。
                if row_effective_lock(&ctx.document, *layer) {
                    return match zone {
                        GrabZone::Body => vec![UiIntent::MoveClips {
                            grabbed: *layer,
                            grab_at_us: seconds_to_us(*at_seconds),
                            drop_at_us: seconds_to_us(*at_seconds),
                        }],
                        GrabZone::Edge(edge) => vec![UiIntent::TrimClip {
                            layer: *layer,
                            edge: *edge,
                            at_us: seconds_to_us(*at_seconds),
                        }],
                    };
                }
                match zone {
                    // 端の掴みは選択を変えない(egui と同じ)。
                    GrabZone::Edge(_) => Vec::new(),
                    GrabZone::Body => press_selection_intents(&ctx.selected, *layer, *additive),
                }
            }
            TimelineMsg::RowPicked { layer, additive } => vec![UiIntent::SelectLayer {
                layer: layer.get(),
                additive: *additive,
            }],
            TimelineMsg::FlagPressed { layer, flag } => vec![UiIntent::ToggleItemFlag {
                layer: layer.get(),
                flag: *flag,
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
            // ---- 構造操作 ----
            // rename の確定だけが intent になる(`UiIntent::RenameLayer`)。
            // 空文字は送らない — `TimelineEditor::rename_layer` も断るが、
            // journal を「入力ミスのたび1件」で汚さないための先回り。
            TimelineMsg::RenameCommitted => match &self.renaming {
                Some((layer, name)) if !name.trim().is_empty() => vec![UiIntent::RenameLayer {
                    layer: layer.get(),
                    name: name.clone(),
                }],
                _ => Vec::new(),
            },
            // L ボタン: **明示値**を送る(Toggle ではなく Set — 押すたびに
            // 「いまの自分の lock の反対」を document から読んで積む)。
            TimelineMsg::LockPressed { layer } => vec![UiIntent::SetLayerLock {
                layer: layer.get(),
                locked: !row_own_lock(&ctx.document, *layer),
            }],
            TimelineMsg::GroupPressed => {
                if ctx.selected.is_empty() {
                    Vec::new()
                } else {
                    vec![UiIntent::GroupLayers {
                        layers: ctx.selected.iter().map(|layer| layer.get()).collect(),
                    }]
                }
            }
            // Ungroup は**単独選択**のときだけ意味を持つ(束ねた複数の Group を
            // 一度に解く操作は無い — egui 側にも前例が無いので発明しない)。
            TimelineMsg::UngroupPressed => match ctx.selected.as_slice() {
                [layer] => vec![UiIntent::UngroupLayer { layer: layer.get() }],
                _ => Vec::new(),
            },
            TimelineMsg::DuplicatePressed => {
                if ctx.selected.is_empty() {
                    Vec::new()
                } else {
                    vec![UiIntent::DuplicateSelection]
                }
            }
            // **ロック中は rename も掴む前に断る。** egui 版 `begin_rename` と同じ
            // 判断(コミット時ではなく、始めようとした時点で断る) — 編集欄を
            // 開いてから Enter で初めて断られるのは「入れたのに後出しで
            // 拒否される」体験になる(2026-08-19 能力台帳の指摘と同じ型)。
            // 名前は空でよい — `rename_layer` は lock を空名チェックより先に見る。
            TimelineMsg::RenameStarted { layer } if row_effective_lock(&ctx.document, *layer) => {
                vec![UiIntent::RenameLayer {
                    layer: layer.get(),
                    name: String::new(),
                }]
            }
            // fold・rename の編集途中(バッファの1手ぶん)は view の preview と
            // 同じ scope — Document には入らない(再現は Message 列 replay)。
            TimelineMsg::FoldToggled { .. }
            | TimelineMsg::RenameStarted { .. }
            | TimelineMsg::RenameEdited(_)
            | TimelineMsg::RenameCancelled => Vec::new(),
            // view の操作・preview の途中経過は intent にならない
            // (再現は Message 列 replay が持つ — `drive_timeline.rs` の oracle)。
            TimelineMsg::ScrubStarted { .. }
            | TimelineMsg::PointerMoved { .. }
            | TimelineMsg::GestureCancelled
            | TimelineMsg::ZoomedAt { .. }
            | TimelineMsg::PannedX { .. }
            | TimelineMsg::ScrolledY { .. }
            | TimelineMsg::OverviewSeek { .. }
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
            } => {
                // **ロック中は preview のジェスチャを始めない。** `plan()` が
                // 断りだけを journal ⇄ 帯へ既に送っている(D2 は lock を見ないので、
                // ここで揃えないと「preview は動くが release で無言で戻る」という
                // 嘘になる — 2026-08-19 能力台帳の指摘)。
                if row_effective_lock(&ctx.document, *layer) {
                    self.drag = None;
                    return;
                }
                match zone {
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
                }
            }
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
            TimelineMsg::OverviewSeek { at_seconds } => {
                let span = self.view.span;
                self.view = TimelineView {
                    start: at_seconds - span * 0.5,
                    span,
                }
                .clamped(comp);
            }
            TimelineMsg::ScrolledY { delta_px, max } => {
                self.scroll_y = (self.scroll_y + delta_px).clamp(0.0, max.max(0.0));
            }
            TimelineMsg::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            }
            // ---- 構造操作 ----
            // fold は Document に入らないのでここが正本(intent 経由の反映ではない)。
            TimelineMsg::FoldToggled { layer, params } => {
                if *params {
                    if self.fold.params_are_open(*layer) {
                        self.fold.close_params(*layer);
                    } else {
                        self.fold.open_params(*layer);
                    }
                } else if self.fold.children_are_open(*layer) {
                    self.fold.close_children(*layer);
                } else {
                    self.fold.open_children(*layer);
                }
            }
            // rename の編集バッファは pane の session 状態(egui 版 `self.renaming`
            // と同じ立ち位置)。**いま見えている名前を初期値にする**。
            //
            // ロック中は編集欄を開かない — `plan()` が既に `RenameLayer` の
            // probe を1件飛ばして断りを帯へ書いた後なので、ここは黙って
            // 何もしない(二重に言わない)。
            TimelineMsg::RenameStarted { layer } => {
                if row_effective_lock(&ctx.document, *layer) {
                    return;
                }
                let name = ctx
                    .document
                    .layers
                    .display_name(*layer)
                    .unwrap_or("?")
                    .to_owned();
                self.renaming = Some((*layer, name));
            }
            TimelineMsg::RenameEdited(text) => {
                if let Some((_, buffer)) = self.renaming.as_mut() {
                    *buffer = text.clone();
                }
            }
            TimelineMsg::RenameCommitted | TimelineMsg::RenameCancelled => {
                self.renaming = None;
            }
            // まとめたら中が見えている状態にする(egui 版 `group_selected` の
            // `self.fold.open_children(group)` と同じ手触り — dispatch 後の
            // `ctx.selected` が新しい Group の singleton になっている)。
            TimelineMsg::GroupPressed => {
                if let [group] = ctx.selected.as_slice() {
                    self.fold.open_children(*group);
                }
            }
            // 選択・M/S/L・Ungroup・Duplicate・削除・Undo/Redo・コマ送りは
            // 座席側の状態で、pane は何も控えない。
            TimelineMsg::RowPicked { .. }
            | TimelineMsg::FlagPressed { .. }
            | TimelineMsg::LockPressed { .. }
            | TimelineMsg::UngroupPressed
            | TimelineMsg::DuplicatePressed
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
            layer: layer.get(),
            additive: true,
        }]
    } else if !selected.contains(&layer) {
        vec![UiIntent::SelectLayer {
            layer: layer.get(),
            additive: false,
        }]
    } else {
        Vec::new()
    }
}
