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

use motolii_core::{Fps, RationalTime};
use motolii_doc::{Document, LayerId};
use motolii_ui::blitz_shell::{seconds_to_us, UiEditParam, UiIntent, UiItemFlag, UiKeyInterp};
use motolii_ui::timeline_editor::{TimelineView, TrimEdge};
use motolii_ui::timeline_rows::{RowKind, TimelineFoldState};

use super::semantics::{
    clamped_move_delta, clamped_trim, frame_snapped, initial_view, key_times_match, move_targets,
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
    /// Cmd+A。**見えている object 行を全部選ぶ**(egui 版 `TimelineEditor` の
    /// `self.selected = object_layers.clone()` と同じ意味。閉じた Group の
    /// 中は見えていない = 選ばれない — この殻はまだ Group の開閉 UI が無い
    /// ので `TimelineFoldState::default()` が常に「全部閉じ」を意味する)。
    SelectAllPressed,
    /// Cmd+ホイール = 横 zoom。`anchor_seconds` の時刻が動かない。
    ZoomedAt { anchor_seconds: f32, factor: f32 },
    /// Shift+ホイール / 横スワイプ = 横パン(秒)。
    PannedX { delta_seconds: f32 },
    /// 素のホイール = 縦スクロール。`max` は canvas が寸法から出した上限。
    ScrolledY { delta_px: f32, max: f32 },
    /// 修飾キーの状態。`WheelScrolled` が modifiers を運ばないので追いかける
    /// (spike の詰まった箇所その1と同じ回避)。
    ModifiersChanged(iced::keyboard::Modifiers),

    // ---- property 行(`RowKind::Property`)のキー菱形(2026-08-19 M-6
    // キー編集レーン)。凍結名は発注の通り、名前は変えていない。 ----
    /// 菱形を掴んだ = 選択(Cmd で足し引き)+ ドラッグの開始。**intent は無い**
    /// — 選択は pane 側の局所状態(`selected_keys`。egui 版の `selected_keys`
    /// と同じ扱いで、Document/journal には入らない)。
    KeyGrabbed {
        layer: LayerId,
        param: UiEditParam,
        at_seconds: f32,
        additive: bool,
    },
    /// property 行の、菱形の外側のトラック面を押した = playhead へキーを打つ
    /// (`UiIntent::KeyParamAtPlayhead` — Timeline からもこの1本を呼ぶ)。
    KeyTrackClicked { layer: LayerId, param: UiEditParam },
    /// 菱形を右クリック = 補間メニューを開く。`anchor` は窓座標(メニューの
    /// 出る位置)。
    KeyMenuRequested {
        layer: LayerId,
        param: UiEditParam,
        at_seconds: f32,
        anchor: iced::Point,
    },
    /// 補間メニューの外を押した / Esc = 閉じるだけ(何も書かない)。
    KeyMenuDismissed,
    /// 補間メニューで1項目を選んだ → `UiIntent::SetParamKeyInterp` そのもの
    /// (凍結名)。`view.rs` の menu closure が `key_menu` の控え(layer/param/
    /// at_us)と選んだ id からこの形へ組み立てる。
    KeyInterpChosen {
        layer: LayerId,
        param: UiEditParam,
        at_us: i64,
        interp: UiKeyInterp,
    },
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
    /// property 行のキー菱形の移動。`from_seconds` は掴んだ時の実時刻
    /// (egui 版 `Grab::KeyTime.original` と同じ役)、`preview` はフレーム吸着済み
    /// の着地(egui のように clip 端へは吸着しない — v1 の残差、README 参照)。
    Key {
        layer: LayerId,
        param: UiEditParam,
        from_seconds: f32,
        preview: f32,
    },
}

/// 右クリックで開いている補間メニューの控え。`view.rs` がこれを読んで
/// `widgets::context_menu` を canvas の上に重ねる。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyMenuState {
    pub layer: LayerId,
    pub param: UiEditParam,
    pub at_seconds: f32,
    pub anchor: iced::Point,
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
    /// 選ばれているキー(`(layer, param, at_seconds)`)。egui 版 `selected_keys`
    /// と同じ扱いで、Document/journal には入らない pane 局所の session 状態
    /// (2026-08-19 M-6 — `SelectLayer` と違い、Inspector など他面から見る
    /// 相手が今のところ無いので intent 化していない)。
    pub selected_keys: Vec<(LayerId, UiEditParam, f32)>,
    /// 右クリックで開いている補間メニュー(無ければ閉じている)。
    pub key_menu: Option<KeyMenuState>,
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
            selected_keys: Vec::new(),
            key_menu: None,
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
                Some(TimelineDrag::Key {
                    layer,
                    param,
                    from_seconds,
                    preview,
                }) if !key_times_match(*preview, *from_seconds) => vec![UiIntent::MoveParamKey {
                    layer: layer.get(),
                    param: *param,
                    from_us: seconds_to_us(*from_seconds),
                    to_us: seconds_to_us(*preview),
                }],
                _ => Vec::new(),
            },
            // **キーが選ばれていればキーが先**(egui 版 `delete_selection_gesture`
            // と同じ横取り — clip 選択の削除は構造レーンの担当なので、キー選択が
            // 空の時だけそちらへ通す)。
            TimelineMsg::DeletePressed => {
                if !self.selected_keys.is_empty() {
                    self.selected_keys
                        .iter()
                        .map(|(layer, param, at_seconds)| UiIntent::RemoveParamKey {
                            layer: layer.get(),
                            param: *param,
                            at_us: seconds_to_us(*at_seconds),
                        })
                        .collect()
                } else if ctx.selected.is_empty() {
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
            // Cmd+A。**足す(`additive: true`)だけを、まだ選ばれていない行にだけ
            // 出す。** `UiIntent::SelectLayer { additive: true }` は
            // `add_to_selection` = Cmd+click と同じトグルなので、既に選ばれて
            // いる行にもう一度出すと外れてしまう(egui に無いバグを iced 側で
            // 作らないための filter)。
            TimelineMsg::SelectAllPressed => {
                motolii_ui::timeline_rows::rows(&ctx.document, &TimelineFoldState::default())
                    .into_iter()
                    .filter(|row| row.kind == RowKind::Object)
                    .map(|row| row.layer)
                    .filter(|layer| !ctx.selected.contains(layer))
                    .map(|layer| UiIntent::SelectLayer {
                        layer: layer.get(),
                        additive: true,
                    })
                    .collect()
            }
            // 菱形を掴んだのは選択 + ドラッグ開始(pane 局所、`note` が持つ)。
            // release までは intent が無い(Move/Trim と同じ設計)。
            TimelineMsg::KeyGrabbed { .. } => Vec::new(),
            // 空のトラック面を押した = playhead へキーを打つ。**Inspector 用に
            // M-4b で入った intent をそのまま呼ぶ**(新しい intent は作らない)。
            // 成分値が引けない(catalog / playhead 不整合)なら黙って何もしない
            // — 理由は gateway 側の reject 経路と同じ帯へ出るのは実際に
            // dispatch した時だけなので、ここで出せない分は素通しする。
            TimelineMsg::KeyTrackClicked { layer, param } => {
                let Some(at) = frame_snapped_time(ctx) else {
                    return Vec::new();
                };
                let Some(components) = crate::inspector_model::transform_component_values(
                    &ctx.document,
                    *layer,
                    *param,
                    at,
                ) else {
                    return Vec::new();
                };
                vec![UiIntent::KeyParamAtPlayhead {
                    layer: layer.get(),
                    param: *param,
                    components,
                }]
            }
            // メニューを開く/閉じるのは pane 局所(`note` が持つ)。
            TimelineMsg::KeyMenuRequested { .. } | TimelineMsg::KeyMenuDismissed => Vec::new(),
            TimelineMsg::KeyInterpChosen {
                layer,
                param,
                at_us,
                interp,
            } => vec![UiIntent::SetParamKeyInterp {
                layer: layer.get(),
                param: *param,
                at_us: *at_us,
                interp: *interp,
            }],
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
                Some(TimelineDrag::Key { preview, .. }) => {
                    // v1: フレーム境界へ吸着するだけ(clip 端 / 他キーへの吸着は
                    // egui 版にはあるが、この版の残差 — README に記載)。
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
            // 菱形を掴んだ = 選択(egui 版 `select_key` と同じ規則: 素で置き換え /
            // Cmd で足し引き)+ ドラッグの開始。preview は掴んだ時刻から始まる
            // ので、掴んだだけ(ドラッグ無し)の release は `key_times_match` が
            // 効いて intent を出さない。
            TimelineMsg::KeyGrabbed {
                layer,
                param,
                at_seconds,
                additive,
            } => {
                let entry = (*layer, *param, *at_seconds);
                if *additive {
                    if let Some(pos) = self
                        .selected_keys
                        .iter()
                        .position(|e| key_entry_matches(e, &entry))
                    {
                        self.selected_keys.remove(pos);
                    } else {
                        self.selected_keys.push(entry);
                    }
                } else if !self
                    .selected_keys
                    .iter()
                    .any(|e| key_entry_matches(e, &entry))
                {
                    self.selected_keys = vec![entry];
                }
                self.drag = Some(TimelineDrag::Key {
                    layer: *layer,
                    param: *param,
                    from_seconds: *at_seconds,
                    preview: *at_seconds,
                });
            }
            TimelineMsg::KeyMenuRequested {
                layer,
                param,
                at_seconds,
                anchor,
            } => {
                self.key_menu = Some(KeyMenuState {
                    layer: *layer,
                    param: *param,
                    at_seconds: *at_seconds,
                    anchor: *anchor,
                });
            }
            TimelineMsg::KeyMenuDismissed | TimelineMsg::KeyInterpChosen { .. } => {
                self.key_menu = None;
            }
            // Delete で消えた分は選択にも残さない(egui 版と同じ — 消した後は
            // 選択も空)。行選択の削除は別レーンの `DeleteSelection` が持つので
            // ここでは触らない。
            TimelineMsg::DeletePressed => {
                self.selected_keys.clear();
            }
            // 選択・M/S・Undo/Redo・コマ送り・空トラック押しは座席側の状態で、
            // pane は何も控えない。
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
            | TimelineMsg::KeyTrackClicked { .. }
            | TimelineMsg::UndoPressed
            | TimelineMsg::RedoPressed
            | TimelineMsg::PlayheadStepped { .. }
            | TimelineMsg::SelectAllPressed => {}
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

/// `selected_keys` の1件が、いま押した/掃いたキーと同じか。**秒は
/// epsilon 比較**(`key_times_match`)— 浮動小数の往復で揺れても選択が
/// ちらつかない。
fn key_entry_matches(
    entry: &(LayerId, UiEditParam, f32),
    other: &(LayerId, UiEditParam, f32),
) -> bool {
    entry.0 == other.0 && entry.1 == other.1 && key_times_match(entry.2, other.2)
}

/// playhead(pane が読める f32 秒)を、キーを打てるフレーム境界の `RationalTime`
/// へ。`TimelineEditor::playhead_time` の pane 側の鏡映(同じ「フレームの
/// 格子に載らない playhead はキーを打てない」判断)。
fn frame_snapped_time(ctx: &TimelineCtx) -> Option<RationalTime> {
    let fps = ctx.fps();
    let raw =
        RationalTime::try_new((f64::from(ctx.playhead) * 1000.0).round() as i64, 1000).ok()?;
    let frame = raw.try_to_frame_round(fps).ok()?;
    RationalTime::try_from_frame(frame, fps).ok()
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
