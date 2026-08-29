use makepad_widgets::*;
use motolii_timeline_projection as timeline_pane;
use motolii_store::Fps;

use crate::gesture_input::{GestureDevice, GesturePhase, GestureSample};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.TimelineSurfaceBase = #(TimelineSurface::register_widget(vm))
    mod.widgets.TimelineSurface = set_type_default() do mod.widgets.TimelineSurfaceBase{
        width: Fill
        height: Fill
        draw_bg +: {color: #x2e2e2e}
        draw_item +: {color: #c5c5c5}
        draw_text +: {
            color: #c5c5c5
            // 名前(レーン名)もこの draw_text が描く。等幅は値専用(tokens の規則)なので
            // regular。ルーラーの数字は値だが、Live もルーラーは UI 書体で刻む
            // font_size はここでは死んでいる既定値 — draw_label が呼ぶたびに
            // text_xs/text_sm/text_md で上書きする。tokens.text と揃える。
            text_style: theme.font_regular{font_size: mod.tokens.text.md}
        }
        // 見た目の調整値はここに出しておく — --hot が拾えるのは script_mod!
        // だけで、Rust の const は再ビルドしないと変わらない。
        ruler_height: 22.0
        rail_width: 150.0
        // 行高は**利用者の持ち物**であってペイン高の従属変数ではない(欠陥 B1)。
        // 26.0 は canon のモック実測値(target_cell_ratio 0.52 = 13.5px/26px の分母)。
        // 入り切らない分はレーンの縦スクロールで見る(欠陥 A3)。
        lane_row_height: 26.0
        // クリップ両端のトリム掴み代。この幅の中だけが EwResize を名乗ってよい。
        trim_handle_width: 6.0
        tick_row_floor: 40.0
        band_alpha: 0.030
        tick_fade_from: 9.0
        tick_fade_to: 18.0
        type_ratio: 0.53
        ink_k: 1.1
        // このペインだけ独自の生数値で字を置いていた(利用者指摘「バラバラ」)。
        // 他7パネルと同じく mod.tokens.text.* に揃える
        text_xs: mod.tokens.text.xs
        text_sm: mod.tokens.text.sm
        text_md: mod.tokens.text.md
        // playhead = ACCENT 1.5x(canon: timeline-semantics.html S5b) — グリッド線の
        // 通常太さ(1.0)に対する倍率と、pane 内で唯一許されるヒーローの最大コントラスト色。
        // どちらも --hot で振れる値なので Rust const ではなくここに置く。
        playhead_width_scale: 1.5
        playhead_color: #(vec4(0.85, 0.71, 0.45, 1.0))
        // ロケータ(発注 S5)。色は tokens の accent 系1色 — 新しい色の族を発明しない
        // (`mod.tokens.accent.on`、`stage_error` 等が同じ琥珀を使っている)。
        marker_color: mod.tokens.accent.on
    }
}
#[cfg(test)]
use makepad_widgets::makepad_platform::event::ScrollPhase;

const PROPERTY_ROW_HEIGHT: f64 = 18.0;
const MIN_VISIBLE_SPAN_SECONDS: f64 = 2.0;
// draw_grid_and_ruler の縦線が引いている実太さ(そこは弄らない指示なので定数だけ
// ここへ写して playhead_width_scale の掛け算元にする)。canon: 「playhead = ACCENT 1.5x」
// の 1.5x はこの通常グリッド線太さに対する倍率。
const GRID_LINE_WIDTH: f64 = 1.0;

#[derive(Clone, Debug, Default)]
pub struct TimelineLane {
    pub id: u64,
    pub name: String,
    pub hidden: bool,
    pub solo: bool,
    pub locked: bool,
    pub label_color: usize,
    pub start: i64,
    pub duration: i64,
    pub selected: bool,
    /// この**クリップが見せている区間**の波形 (min, max)。空 = 音が無い(または
    /// まだ読めていない)ので何も描かない。
    ///
    /// **既に切り出された物が来る**(素材の全長ではない)。トリム・タイムストレッチ
    /// の写像は `motolii_timeline_projection::waveform_bucket_range` が持っていて、
    /// ここは「クリップの左端から右端までを、この配列の先頭から末尾までへ均等に
    /// 割り当てる」しか知らない。**同じ写像を2箇所に住まわせない。**
    pub waveform: Vec<(f32, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct TimelinePropertyLane {
    pub layer_id: u64,
    pub name: String,
    pub keys: Vec<i64>,
}

/// ルーラーのロケータ1枚(発注 S5)。**`motolii_store::Marker` の写しではない** —
/// `TimelineLane`/`TimelinePropertyLane` と同じ、widget が要る分だけの投影。
/// 名前・尺(範囲ロケータ)はこの波の非目標なので運ばない — 運ぶ物を増やすと
/// 「まだ無い機能の器だけ先に生える」(Q0 の裏側)。**宣言順が身分**
/// (`marker.rs` の doc: マーカーに安定 id は無い)なので、`main.rs` の
/// `TimelineEditAction::RemoveMarker` はこの Vec の index で名指す。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TimelineMarker {
    pub frame: i64,
}

#[derive(Clone, Debug, Default)]
pub struct TimelineModel {
    /// Front-to-back. The backend derives this from `LayerMeta.order`; the widget
    /// never owns a second persistent ordering model.
    pub lanes: Vec<TimelineLane>,
    pub property_lanes: Vec<TimelinePropertyLane>,
    pub markers: Vec<TimelineMarker>,
    pub duration_frames: i64,
    pub playhead: i64,
    pub fps_num: i64,
    pub fps_den: i64,
}

// M/S/L はレールグリフへの直接操作(canon: timeline-semantics.html
// 「M/S/L | b | rail glyph 直接 | 1:0:0:0」)。3値をどれか1つ選ぶだけで
// 意味が閉じているので bool 3枚より enum の方が取り違えが起きない。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaneFlag {
    Hidden,
    Solo,
    Locked,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TimelineSurfaceAction {
    #[default]
    None,
    Scrub(i64),
    /// Destination is a front-to-back lane index. The backend translates this
    /// once into a Document stacking edit (one gesture = one undo).
    Restack {
        layer_id: u64,
        target_from_front: usize,
    },
    ZoomChanged {
        start_frame: i64,
        visible_frames: i64,
    },
    /// レールグリフ(M/S/L)の直接クリック。値そのものではなく「トグルしろ」
    /// という意図だけを運ぶ — 現在値は Document/TimelineModel 側の真実なので
    /// ここで反転後の値を計算して持たせると二重管理になる。
    ToggleLaneFlag { layer_id: u64, flag: LaneFlag },
}

/// Timeline がもう1本だけ持つ、**編集意図**の口。
///
/// なぜ [`TimelineSurfaceAction`] へ variant を足さないか: あちらは shell
/// (`main.rs` の `apply_timeline_action`)が**網羅 match** で受けている。variant を
/// 足すと shell 側のファイルを同時に書き換えなければ型が通らず、レーン境界
/// (write-set)を跨ぐ。迂回でごまかすより「自分の境界に素直な口を1本足す」方を
/// 採った(裁定: 迂回より wrapper)。`filter_widget_actions_cast` は型が違う
/// action を `Default`(= `None`)へ落とすので、shell は同じ uid をこの型でもう一度
/// 拾うだけでよく、片方の型の `None` は互いに無害に素通りする。
///
/// **shell 側の受け口**(`main.rs` の `BackendBridge::apply_timeline_edit`、2026-08-28 配線):
/// - [`TimelineEditAction::Select`] → `session.selection` / `session.selected_layers`
/// - [`TimelineEditAction::SetClipTiming`] → 現在の `LayerMeta.timing` を読み、
///   `start`/`duration`/`source_in` を差し替えて `Intent::SetTiming { layer, timing }`
///   (`Intent::SetOrder` を使う restack と同じ形。新しい書き込み経路ではない)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TimelineEditAction {
    #[default]
    None,
    /// 行/クリップのクリック選択。`layer_id: None` は空所クリックによる全解除。
    /// `additive` は shift/⌘ 併用(トグル追加)。
    Select {
        layer_id: Option<u64>,
        additive: bool,
    },
    /// トリム/移動の確定。**1ジェスチャ = 1つ**しか出ない(= 1 undo)ので、
    /// restack と同じく `FingerUp` でだけ発火する。
    SetClipTiming {
        layer_id: u64,
        start: i64,
        duration: i64,
        /// 掴んだ端 = **利用者が何をしたつもりか**。`None` は本体を掴んだ移動。
        /// shell はこれを見て素材の頭出し(`LayerTiming.source_in`)を動かすか
        /// 決める — 頭を切っても素材はずれない、丸ごと動かせば素材も一緒に動く。
        edge: Option<ClipEdge>,
    },
    /// ルーラーの空所を右クリック(発注 S5)。**その時刻へ、既定値のロケータを1つ**
    /// 置く(name="" / duration=0)。値を発明しない — 改名 UI は次の波。
    AddMarker { frame: i64 },
    /// 既存ロケータの上を右クリック(発注 S5)。**置けるのに消せないのは Q0 違反**
    /// なので、置く口と同じルーラーに消す口も要る。`index` は `TimelineModel.markers`
    /// の宣言順(マーカーに安定 id は無い、`TimelineMarker` の doc 参照)。
    RemoveMarker { index: usize },
}

type TimelineInputAction = TimelineSurfaceAction;

#[derive(Clone, Copy, Debug, PartialEq)]
struct TimelineViewport {
    rail_width: f64,
    time_width: f64,
    view_start: f64,
    visible_frames: f64,
    duration_frames: i64,
    /// The interaction contract has no vertical zoom. Keeping this explicit
    /// makes accidental Y scaling observable in tests.
    vertical_scale: f64,
}

impl TimelineViewport {
    fn new(
        rail_width: f64,
        time_width: f64,
        view_start: f64,
        visible_frames: f64,
        duration_frames: i64,
    ) -> Self {
        Self {
            rail_width,
            time_width: time_width.max(1.0),
            view_start,
            visible_frames: visible_frames.max(1.0),
            duration_frames: duration_frames.max(1),
            vertical_scale: 1.0,
        }
    }

    fn frame_at_x(&self, x: f64) -> i64 {
        let fraction = ((x - self.rail_width) / self.time_width).clamp(0.0, 1.0);
        (self.view_start + fraction * self.visible_frames)
            .round()
            .clamp(0.0, self.duration_frames.saturating_sub(1) as f64) as i64
    }

    /// トリム/移動の差分に使う**未クランプ**の連続フレーム。掴んだ点から今の点への
    /// 差だけが欲しいので、画面外へ出た瞬間に 0..duration へ丸めると差が消えてしまう。
    fn frames_at_x(&self, x: f64) -> f64 {
        self.view_start + (x - self.rail_width) / self.time_width * self.visible_frames
    }

    fn zoom_at(&self, x: f64, scroll_y: f64, scroll_x: f64) -> Option<Self> {
        let scroll = if scroll_y.abs() > f64::EPSILON {
            scroll_y
        } else {
            scroll_x
        };
        if scroll.abs() <= f64::EPSILON || x < self.rail_width {
            return None;
        }
        let anchor_fraction = ((x - self.rail_width) / self.time_width).clamp(0.0, 1.0);
        let anchor_frame = self.view_start + anchor_fraction * self.visible_frames;
        let zoom_power = (-scroll / 240.0).clamp(-1.0, 1.0);
        let min_span = 10.0_f64.min(self.duration_frames as f64);
        let visible_frames = (self.visible_frames * 2.0_f64.powf(zoom_power))
            .clamp(min_span, self.duration_frames as f64);
        if (visible_frames - self.visible_frames).abs() < 0.01 {
            return None;
        }
        let max_start = (self.duration_frames as f64 - visible_frames).max(0.0);
        let view_start = (anchor_frame - anchor_fraction * visible_frames).clamp(0.0, max_start);
        Some(Self {
            view_start,
            visible_frames,
            ..*self
        })
    }

    // tick_row_floor は TimelineSurface 側の #[live] 値(--hot で調整する対象)。
    // TimelineViewport は TimelineSurface を持たないので、self 経由では読めず
    // 呼び出し側から渡してもらう。
    fn tick_steps(&self, fps_num: i64, fps_den: i64, lane_height: f64, tick_row_floor: f64) -> (i64, i64) {
        let fps = Fps::try_new(fps_num.max(1), fps_den.max(1)).ok();
        // 目標セル比率は JSON 正本(`target_cell_ratio` = 0.52、モック実測
        // 13.5px/26px)。ここで数値を手打ちすると「形は比率で定数化する」
        // 裁定を r7 だけが破る — 5.0 を渡していたのは 10 倍の取り違えだった。
        timeline_pane::tick_steps(
            fps,
            self.visible_frames.round().max(1.0) as i64,
            self.time_width as f32,
            // 行高そのものを床上げする。比率(0.52)は変えずに、比率の分母を
            // 可読な最小値まで持ち上げるだけなので、床より上のズームでは
            // 従来通り比率駆動のまま。
            lane_height.max(tick_row_floor).max(1.0) as f32,
        )
    }
}

/// クリップのどちら端を掴んだか。`None`(= [`PointerTarget::Clip`] の `edge` が
/// `None`)は本体を掴んだ = 尺を変えずに移動。
///
/// **`pub` なのは [`TimelineEditAction::SetClipTiming`] が運ぶから** — shell は
/// `(start, duration)` の差分から「頭を切ったのか丸ごと動かしたのか」を推理できない
/// (どちらも `start` が動く)。推理させると、素材の頭出し(`source_in`)を
/// 動かすべき時とそうでない時を取り違える。**掴んだ端は widget だけが知っている
/// 事実なので、意図として運ぶ**(裁定271: 動詞は意図を語る)。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipEdge {
    Start,
    End,
}

/// ポインタが今どこに居るかの**唯一の答え**。カーソル(A2)と FingerDown(A1/A5)が
/// 同じ関数を読むことで、「伸縮できない所で EwResize」のような嘘が構造的に作れない。
#[derive(Clone, Copy, Debug, PartialEq)]
enum PointerTarget {
    /// ルーラー帯。スクラブできるのはここだけ。
    Ruler,
    /// 既存ロケータの上(発注 S5)。`Ruler` の特殊形 — 左クリックは同じスクラブ
    /// (その時刻へ跳ぶ、`frame` は marker の生の値なので x→frame の丸め誤差が乗らない)、
    /// 右クリックだけ挙動が分かれる(消す)。`index` は `TimelineModel.markers` の
    /// 宣言順(`TimelineMarker` の doc 参照)。
    Marker { index: usize, frame: i64 },
    /// プロパティ行の開閉三角(B2)。
    Fold { layer_id: u64 },
    /// M/S/L グリフ。
    Flag { layer_id: u64, flag: LaneFlag },
    /// レール(名前側)。選択 + 並べ替えドラッグ。
    Rail {
        layer_id: u64,
        from_front: usize,
        /// 行上端から掴んだ点までの距離。掴んだ行を指に付いてこさせる(A4)ため。
        grab_offset: f64,
        locked: bool,
    },
    /// クリップの棒。`edge` が `Some` ならトリム、`None` なら移動。
    Clip {
        layer_id: u64,
        lane_index: usize,
        edge: Option<ClipEdge>,
        start: i64,
        duration: i64,
        locked: bool,
    },
    /// 何も無い所。クリックは選択の解除。
    Empty,
}

#[derive(Clone, Copy, Debug, Default)]
enum TimelineGesture {
    #[default]
    None,
    Playhead,
    Lane {
        layer_id: u64,
        from_front: usize,
        target_from_front: usize,
        grab_offset: f64,
        pointer_y: f64,
    },
    Clip {
        layer_id: u64,
        lane_index: usize,
        edge: Option<ClipEdge>,
        origin_frame: f64,
        origin_start: i64,
        origin_duration: i64,
        changed: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScrollMode {
    Pan,
    Zoom,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TimelineScrollAction {
    PanPixels(f64),
    /// 無修飾の縦入力(A3)。時間軸ではなくレーンの縦スクロール。
    ScrollLanes(f64),
    Zoom { delta: f64, precise: bool },
}

/// Owns one phased trackpad stream. Axis and verb are selected once, then kept
/// through OS momentum so a diagonal gesture cannot alternate between pan and
/// zoom as individual deltas fluctuate.
#[derive(Clone, Copy, Debug, Default)]
struct TimelineScrollGesture {
    axis: Option<ScrollAxis>,
    mode: Option<ScrollMode>,
    owns_momentum: bool,
}

impl TimelineScrollGesture {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn reject_owner_if_unapplied(&mut self, phase: GesturePhase, applied: bool) {
        if !applied
            && matches!(
                phase,
                GesturePhase::Begin | GesturePhase::Update | GesturePhase::End
            )
        {
            self.owns_momentum = false;
        }
    }

    fn dominant_axis(scroll: [f64; 2]) -> Option<ScrollAxis> {
        const AXIS_THRESHOLD: f64 = 0.5;
        if scroll[0].abs().max(scroll[1].abs()) < AXIS_THRESHOLD {
            None
        } else if scroll[0].abs() >= scroll[1].abs() {
            Some(ScrollAxis::Horizontal)
        } else {
            Some(ScrollAxis::Vertical)
        }
    }

    fn update_sample(&mut self, sample: GestureSample) -> Option<TimelineScrollAction> {
        match sample.phase {
            GesturePhase::Catch | GesturePhase::MomentumEnd | GesturePhase::Cancel => {
                self.reset();
                return None;
            }
            GesturePhase::Begin => {
                self.reset();
            }
            GesturePhase::Momentum if !self.owns_momentum => return None,
            GesturePhase::Instant => self.reset(),
            GesturePhase::Update | GesturePhase::End | GesturePhase::Momentum => {}
        }

        let native_scale = (sample.scale_ratio - 1.0).abs() > f64::EPSILON;
        let mode = *self
            .mode
            .get_or_insert(if native_scale || sample.modifiers.alt {
                ScrollMode::Zoom
            } else {
                ScrollMode::Pan
            });
        let axis = match self.axis {
            Some(axis) => axis,
            None => {
                let axis = Self::dominant_axis(sample.translation)
                    .or_else(|| native_scale.then_some(ScrollAxis::Horizontal))?;
                self.axis = Some(axis);
                axis
            }
        };

        let action = match mode {
            ScrollMode::Zoom => {
                // Option-scroll is the converged AE/Resolve timeline gesture.
                // Use the locked dominant component and preserve the platform's
                // direction rather than applying an app-specific inversion.
                let delta = if native_scale {
                    240.0 * sample.scale_ratio.max(0.01).log2()
                } else {
                    match axis {
                        ScrollAxis::Horizontal => sample.translation[0],
                        ScrollAxis::Vertical => sample.translation[1],
                    }
                };
                (delta.abs() > f64::EPSILON).then_some(TimelineScrollAction::Zoom {
                    delta,
                    precise: sample.device != GestureDevice::Wheel,
                })
            }
            ScrollMode::Pan => {
                // A horizontal trackpad gesture pans time. Shift converts a
                // vertical wheel into the same horizontal operation. Unmodified
                // vertical input scrolls the lane column — 行高が固定になった
                // (B1)ので入り切らない行が出る、その分をここで見る(A3)。
                match axis {
                    ScrollAxis::Horizontal => (sample.translation[0].abs() > f64::EPSILON)
                        .then_some(TimelineScrollAction::PanPixels(sample.translation[0])),
                    ScrollAxis::Vertical if sample.modifiers.shift => {
                        (sample.translation[1].abs() > f64::EPSILON)
                            .then_some(TimelineScrollAction::PanPixels(sample.translation[1]))
                    }
                    ScrollAxis::Vertical => (sample.translation[1].abs() > f64::EPSILON)
                        .then_some(TimelineScrollAction::ScrollLanes(sample.translation[1])),
                }
            }
        };

        if matches!(
            sample.phase,
            GesturePhase::Begin | GesturePhase::Update | GesturePhase::End
        ) {
            self.owns_momentum |= action.is_some();
        }
        if sample.phase == GesturePhase::Instant {
            self.reset();
        }
        action
    }

    #[cfg(test)]
    fn update(
        &mut self,
        scroll: DVec2,
        phase: ScrollPhase,
        modifiers: KeyModifiers,
    ) -> Option<TimelineScrollAction> {
        self.update_sample(GestureSample {
            phase: match phase {
                ScrollPhase::None => GesturePhase::Instant,
                ScrollPhase::Began => GesturePhase::Begin,
                ScrollPhase::Touched => GesturePhase::Catch,
                ScrollPhase::Changed => GesturePhase::Update,
                ScrollPhase::Ended => GesturePhase::End,
                ScrollPhase::Momentum => GesturePhase::Momentum,
                ScrollPhase::MomentumEnded => GesturePhase::MomentumEnd,
            },
            device: if phase == ScrollPhase::None {
                GestureDevice::Wheel
            } else {
                GestureDevice::Trackpad
            },
            centroid: [0.0, 0.0],
            translation: [scroll.x, scroll.y],
            scale_ratio: 1.0,
            rotation_radians: 0.0,
            modifiers: modifiers.into(),
        })
    }
}

impl TimelineGesture {
    /// 掴む。**どこを掴んだかの判定は [`TimelineSurface::pointer_target`] が済ませて
    /// いる** — ここは「その場所ならどのジェスチャか」だけを決める純関数。
    fn begin(
        &mut self,
        viewport: &TimelineViewport,
        position: DVec2,
        target: PointerTarget,
    ) -> Option<TimelineInputAction> {
        match target {
            PointerTarget::Ruler => {
                *self = Self::Playhead;
                Some(TimelineInputAction::Scrub(viewport.frame_at_x(position.x)))
            }
            // ロケータの左クリック = playhead がその時刻へ跳ぶ(発注 S5)。`frame` は
            // marker が持っている生の値をそのまま使う — x から引き直すと丸め誤差が
            // 乗る余地がある。
            PointerTarget::Marker { frame, .. } => {
                *self = Self::Playhead;
                Some(TimelineInputAction::Scrub(frame))
            }
            // LOCKED は掴めない。store が `SetTiming`/`SetOrder` を拒む
            // (`check_not_locked`)ので、ここで掴ませると「動いて見えてから戻る」
            // になる。選択は上の `select_lane` が先に済ませているので、
            // 錠のかかった行もクリックで選べる(L を押して外せる)。
            PointerTarget::Rail { locked: true, .. } | PointerTarget::Clip { locked: true, .. } => {
                *self = Self::None;
                None
            }
            PointerTarget::Rail {
                layer_id,
                from_front,
                grab_offset,
                ..
            } => {
                *self = Self::Lane {
                    layer_id,
                    from_front,
                    target_from_front: from_front,
                    grab_offset,
                    pointer_y: position.y,
                };
                None
            }
            PointerTarget::Clip {
                layer_id,
                lane_index,
                edge,
                start,
                duration,
                ..
            } => {
                *self = Self::Clip {
                    layer_id,
                    lane_index,
                    edge,
                    origin_frame: viewport.frames_at_x(position.x),
                    origin_start: start,
                    origin_duration: duration,
                    changed: false,
                };
                None
            }
            PointerTarget::Fold { .. } | PointerTarget::Flag { .. } | PointerTarget::Empty => {
                *self = Self::None;
                None
            }
        }
    }

    fn pointer_move(
        &self,
        viewport: &TimelineViewport,
        position: DVec2,
    ) -> Option<TimelineInputAction> {
        match self {
            Self::Playhead => Some(TimelineInputAction::Scrub(viewport.frame_at_x(position.x))),
            _ => None,
        }
    }

    fn move_lane_target(&mut self, target_from_front: usize) -> Option<TimelineInputAction> {
        if let Self::Lane {
            layer_id,
            from_front,
            grab_offset,
            pointer_y,
            ..
        } = *self
        {
            *self = Self::Lane {
                layer_id,
                from_front,
                target_from_front,
                grab_offset,
                pointer_y,
            };
        }
        None
    }

    /// 掴んだ行を指に付いてこさせる(A4)ための現在位置。
    fn track_pointer_y(&mut self, y: f64) {
        if let Self::Lane { pointer_y, .. } = self {
            *pointer_y = y;
        }
    }

    /// ドラッグ中のクリップの新しい `(start, duration)`。**純関数** — 掴んだ時の
    /// 値と、掴んだ点からのフレーム差だけで決まる(累積誤差が入らない)。
    fn clip_timing_at(
        &self,
        viewport: &TimelineViewport,
        x: f64,
        duration_frames: i64,
    ) -> Option<(usize, u64, i64, i64)> {
        let Self::Clip {
            layer_id,
            lane_index,
            edge,
            origin_frame,
            origin_start,
            origin_duration,
            ..
        } = *self
        else {
            return None;
        };
        let delta = (viewport.frames_at_x(x) - origin_frame).round() as i64;
        let total = duration_frames.max(1);
        let (start, duration) = match edge {
            // 本体を掴んだ = 尺は変えずに comp 上を移動する。
            None => {
                let max_start = (total - origin_duration).max(0);
                ((origin_start + delta).clamp(0, max_start), origin_duration)
            }
            // 頭を掴んだ = 終端(start + duration)を固定したまま頭を動かす。
            Some(ClipEdge::Start) => {
                let end = origin_start + origin_duration;
                let start = (origin_start + delta).clamp(0, end - 1);
                (start, end - start)
            }
            // 尻を掴んだ = 頭を固定したまま尺を伸縮する。
            Some(ClipEdge::End) => {
                let duration = (origin_duration + delta).clamp(1, (total - origin_start).max(1));
                (origin_start, duration)
            }
        };
        Some((lane_index, layer_id, start, duration))
    }

    fn mark_clip_changed(&mut self) {
        if let Self::Clip { changed, .. } = self {
            *changed = true;
        }
    }

    fn pointer_up(&mut self) -> Option<TimelineInputAction> {
        let action = match *self {
            Self::Lane {
                layer_id,
                from_front,
                target_from_front,
                ..
            } if from_front != target_from_front => Some(TimelineInputAction::Restack {
                layer_id,
                target_from_front,
            }),
            _ => None,
        };
        action
    }

    /// トリム/移動の確定に要る「どのレーンを、どう掴んで変えたか」。`FingerUp` で
    /// 1回だけ読む。`edge` まで返すのは、shell が `(start, duration)` の差分から
    /// 掴んだ端を推理できないから([`ClipEdge`] の doc)。
    fn committed_clip(&self) -> Option<(usize, u64, Option<ClipEdge>)> {
        match *self {
            Self::Clip {
                layer_id,
                lane_index,
                edge,
                changed: true,
                ..
            } => Some((lane_index, layer_id, edge)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum VisualRowKind {
    Lane(usize),
    Property(usize),
}

#[derive(Clone, Copy, Debug)]
struct VisualRow {
    kind: VisualRowKind,
    y: f64,
    height: f64,
}

#[derive(Script, ScriptHook, Widget)]
pub struct TimelineSurface {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,

    /// This draw object owns the full hit area. All later rectangles use a
    /// separate draw object so their smaller areas cannot replace it.
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_item: DrawColor,
    #[live]
    draw_text: DrawText,

    // 見た目のチューニング値。const だと --hot で拾えず再ビルドが要るので、
    // script_mod! の type-default から埋まる #[live] フィールドとして持つ。
    #[live(22.0)]
    ruler_height: f64,
    #[live(150.0)]
    rail_width: f64,
    /// 行高(欠陥 B1)。**ペイン高から独立した固定値** — 以前は
    /// `body_height / lane_count` で、ペインのリサイズがタイムライン全体の
    /// 画像ズームになっていた。入り切らない分は縦スクロール(A3)で見る。
    #[live(26.0)]
    lane_row_height: f64,
    /// クリップ両端のトリム掴み代(A1)。カーソルが EwResize を名乗ってよい
    /// 範囲もこの幅と同じ(A2 — 伸縮できる所だけ)。
    #[live(6.0)]
    trim_handle_width: f64,
    // レーン高が実測 ~15pt まで潰れると比率(0.52)通りの間隔でも ~7pt になり、
    // 実機Ableton(68pt行→21pt間隔)が読める密度から外れて単なるノイズになる。
    // 比率そのものはスケール不変で正しいので変えず、tick_steps へ渡す行高だけ
    // この物理下限で持ち上げる — ここが唯一 px 絶対値を許す場所(可読性の床)。
    #[live(40.0)]
    tick_row_floor: f64,
    #[live(0.030)]
    band_alpha: f64,
    #[live(9.0)]
    tick_fade_from: f64,
    /// 字は比率で導出する(裁定271)。行の中身の帯/行高 = 0.53、書体係数 1.1。
    ///
    /// **行高が固定になっても比率は残す**(B1 の直しで消さない): 裁定271 が禁じたのは
    /// 「字を絶対 pt で置くこと」であって、比率そのものではない。変わったのは分母で、
    /// `lane_row_height` は**利用者が選ぶ持ち物**になった — 行高を上げれば字も一緒に
    /// 上がるという関係は保ったまま、ペイン高という他人の都合からは切れている。
    #[live(0.53)]
    type_ratio: f64,
    #[live(1.1)]
    ink_k: f64,
    // mod.tokens.text.* の写し(裸の生数値をこのペインから追放 — 利用者指摘「バラバラ」)。
    #[live(7.5)]
    text_xs: f64,
    #[live(7.23)]
    text_sm: f64,
    #[live(6.87)]
    text_md: f64,
    #[live(18.0)]
    tick_fade_to: f64,
    // playhead = ACCENT 1.5x(canon: timeline-semantics.html S5b)。通常のグリッド線
    // 太さ(GRID_LINE_WIDTH)に掛ける倍率と、pane 内で唯一許されるヒーローの最大
    // コントラスト色。dragging 中のレーンバーもこの色を借りる(同じ ACCENT なので)。
    #[live(1.5)]
    playhead_width_scale: f64,
    #[live(vec4(0.85, 0.71, 0.45, 1.0))]
    playhead_color: Vec4f,
    /// ロケータの色(発注 S5)。script_mod 側は `mod.tokens.accent.on` を渡す
    /// (琥珀1色、新しい色の族を発明しない) — この Rust リテラルはその16進の
    /// 素直な写しで、DSL が載る前の一瞬だけ使われる fallback(`playhead_color` と
    /// 同じ扱い)。
    #[live(vec4(1.0, 0.678, 0.337, 1.0))]
    marker_color: Vec4f,

    #[rust]
    rect: Rect,
    #[rust]
    lanes: Vec<TimelineLane>,
    #[rust]
    property_lanes: Vec<TimelinePropertyLane>,
    /// ルーラーのロケータ(発注 S5)。`lanes`/`property_lanes` と同じ身分 — 正本は
    /// Document で、ここは `set_model` が押し込む投影。
    #[rust]
    markers: Vec<TimelineMarker>,
    #[rust]
    duration_frames: i64,
    #[rust]
    playhead: i64,
    #[rust]
    fps_num: i64,
    #[rust]
    fps_den: i64,
    /// Horizontal viewport only. There is intentionally no vertical scale.
    #[rust]
    view_start: f64,
    #[rust]
    view_span: f64,
    /// レーン列の縦スクロール量(px, 下向き正)。**倍率ではない** — 行高は
    /// `lane_row_height` 固定で、ここは見る窓の位置だけを動かす(A3)。
    #[rust]
    scroll_y: f64,
    /// プロパティ行を畳んでいるレーン(B2)。Document には無い**見え方だけの状態**
    /// なので widget が持って良い(選択やトリムのように store の真実ではない)。
    #[rust]
    collapsed_lanes: Vec<u64>,
    #[rust]
    drag: TimelineGesture,
    #[rust]
    scroll_gesture: TimelineScrollGesture,
}

impl TimelineSurface {
    // A1/A5(WIRE-1、2026-08-28 着地): ここには `TRIM_HANDLE_WIRED` /
    // `SELECTION_WIRED` という2つの塞ぎ栓があった。トリムも選択も widget の中では
    // 完成していたのに store へ届かず、次の `install_timeline_model` で黙って元へ
    // 戻る「消える虚報」だったので、出口(当たり判定・カーソル・帯・ハイライト)
    // だけを閉じてあった。`main.rs` の `apply_timeline_edit` が両方を受けるように
    // なったので栓を抜いた。**塞ぎ栓を残さない** — `= true` の定数は、次に読む者に
    // 「まだ半分嘘かもしれない」と思わせる。

    pub fn set_model(&mut self, cx: &mut Cx, model: TimelineModel) {
        let first_model = self.duration_frames <= 0 || self.view_span <= 0.0;
        self.lanes = model.lanes;
        self.property_lanes = model.property_lanes;
        self.markers = model.markers;
        self.duration_frames = model.duration_frames.max(1);
        self.playhead = model
            .playhead
            .clamp(0, self.duration_frames.saturating_sub(1));
        self.fps_num = model.fps_num.max(1);
        self.fps_den = model.fps_den.max(1);

        if first_model {
            self.view_start = 0.0;
            self.view_span = self.duration_frames as f64;
        } else {
            self.view_span = self
                .view_span
                .clamp(self.min_view_span(), self.duration_frames as f64);
            self.clamp_view_start();
        }
        self.clamp_scroll_y();
        self.redraw(cx);
    }

    /// 見た目チューニング値の読み書き(Settings の TIMELINE 帯から)。
    /// 名は `SettingsSurfaceAction::SetField` の `field` とそのまま対応する。
    pub fn tuning_value(&self, field: &str) -> Option<f64> {
        Some(match field {
            "timeline_row_height" => self.lane_row_height,
            "timeline_rail_width" => self.rail_width,
            "timeline_ruler_height" => self.ruler_height,
            "timeline_trim_handle_width" => self.trim_handle_width,
            "timeline_tick_row_floor" => self.tick_row_floor,
            "timeline_band_alpha" => self.band_alpha,
            "timeline_tick_fade_from" => self.tick_fade_from,
            "timeline_tick_fade_to" => self.tick_fade_to,
            "timeline_playhead_scale" => self.playhead_width_scale,
            _ => return None,
        })
    }

    pub fn set_tuning_value(&mut self, cx: &mut Cx, field: &str, value: f64) -> bool {
        match field {
            "timeline_row_height" => self.lane_row_height = value,
            "timeline_rail_width" => self.rail_width = value,
            "timeline_ruler_height" => self.ruler_height = value,
            "timeline_trim_handle_width" => self.trim_handle_width = value,
            "timeline_tick_row_floor" => self.tick_row_floor = value,
            "timeline_band_alpha" => self.band_alpha = value,
            "timeline_tick_fade_from" => self.tick_fade_from = value,
            "timeline_tick_fade_to" => self.tick_fade_to = value,
            "timeline_playhead_scale" => self.playhead_width_scale = value,
            _ => return false,
        }
        self.redraw(cx);
        true
    }

    fn fps(&self) -> f64 {
        self.fps_num.max(1) as f64 / self.fps_den.max(1) as f64
    }

    fn min_view_span(&self) -> f64 {
        (self.fps() * MIN_VISIBLE_SPAN_SECONDS)
            .max(10.0)
            .min(self.duration_frames.max(1) as f64)
    }

    fn clamp_view_start(&mut self) {
        let max_start = (self.duration_frames as f64 - self.view_span).max(0.0);
        self.view_start = self.view_start.clamp(0.0, max_start);
    }

    fn time_rect(&self) -> Rect {
        Rect {
            pos: dvec2(self.rect.pos.x + self.rail_width, self.rect.pos.y),
            size: dvec2((self.rect.size.x - self.rail_width).max(1.0), self.rect.size.y),
        }
    }

    /// ルーラーより下、行が住む帯。縦スクロールの窓でもある。
    fn body_top(&self) -> f64 {
        self.rect.pos.y + self.ruler_height
    }

    fn body_bottom(&self) -> f64 {
        self.rect.pos.y + self.rect.size.y
    }

    fn body_height(&self) -> f64 {
        (self.body_bottom() - self.body_top()).max(0.0)
    }

    fn viewport(&self) -> TimelineViewport {
        let time = self.time_rect();
        TimelineViewport::new(
            time.pos.x,
            time.size.x,
            self.view_start,
            self.view_span,
            self.duration_frames,
        )
    }

    fn is_collapsed(&self, layer_id: u64) -> bool {
        self.collapsed_lanes.contains(&layer_id)
    }

    fn property_count_for_lane(&self, layer_id: u64) -> usize {
        self.property_lanes
            .iter()
            .filter(|property| property.layer_id == layer_id)
            .count()
    }

    fn lane_height(&self) -> f64 {
        // 行高はペイン高の従属変数ではない(B1)。下限だけは、字も M/S/L も
        // 潰れて意味を失う手前で止める。
        self.lane_row_height.max(10.0)
    }

    /// 全行を積み上げた高さ。ペインより高ければその差が縦スクロールの幅になる。
    fn content_height(&self) -> f64 {
        let lane_height = self.lane_height();
        self.lanes
            .iter()
            .map(|lane| {
                lane_height
                    + if self.is_collapsed(lane.id) {
                        0.0
                    } else {
                        self.property_count_for_lane(lane.id) as f64 * PROPERTY_ROW_HEIGHT
                    }
            })
            .sum()
    }

    fn max_scroll_y(&self) -> f64 {
        (self.content_height() - self.body_height()).max(0.0)
    }

    fn clamp_scroll_y(&mut self) {
        let max = self.max_scroll_y();
        self.scroll_y = self.scroll_y.clamp(0.0, max);
    }

    fn visual_rows(&self) -> Vec<VisualRow> {
        let mut rows = Vec::with_capacity(self.lanes.len() + self.property_lanes.len());
        let lane_height = self.lane_height();
        let mut y = self.body_top() - self.scroll_y;
        for (lane_index, lane) in self.lanes.iter().enumerate() {
            rows.push(VisualRow {
                kind: VisualRowKind::Lane(lane_index),
                y,
                height: lane_height,
            });
            y += lane_height;
            if self.is_collapsed(lane.id) {
                continue;
            }
            for (property_index, property) in self.property_lanes.iter().enumerate() {
                if property.layer_id == lane.id {
                    rows.push(VisualRow {
                        kind: VisualRowKind::Property(property_index),
                        y,
                        height: PROPERTY_ROW_HEIGHT,
                    });
                    y += PROPERTY_ROW_HEIGHT;
                }
            }
        }
        rows
    }

    fn row_is_visible(&self, row: VisualRow) -> bool {
        row.y + row.height > self.body_top() && row.y < self.body_bottom()
    }

    /// 上下どちらにも欠けていない行だけ。字はここが真の時だけ描く — 半分だけ
    /// 見えている行に字を出すと、ルーラーの下から切れた字が生えて読めない。
    fn row_is_whole(&self, row: VisualRow) -> bool {
        row.y >= self.body_top() - 0.5 && row.y + row.height <= self.body_bottom() + 0.5
    }

    fn drop_index_at_y(&self, abs_y: f64) -> usize {
        let lane_rows: Vec<VisualRow> = self
            .visual_rows()
            .into_iter()
            .filter(|row| matches!(row.kind, VisualRowKind::Lane(_)))
            .collect();
        if lane_rows.is_empty() {
            return 0;
        }
        for (index, row) in lane_rows.iter().enumerate() {
            if abs_y < row.y + row.height * 0.5 {
                return index;
            }
        }
        lane_rows.len() - 1
    }

    fn x_at_frame(&self, frame: f64) -> f64 {
        let time = self.time_rect();
        time.pos.x + (frame - self.view_start) / self.view_span.max(1.0) * time.size.x
    }

    fn emit_input_action(&mut self, cx: &mut Cx, action: TimelineInputAction) {
        let TimelineInputAction::Scrub(frame) = action else {
            cx.widget_action(self.uid, action);
            return;
        };
        if frame != self.playhead {
            self.playhead = frame;
            self.redraw(cx);
        }
        cx.widget_action(self.uid, TimelineSurfaceAction::Scrub(frame));
    }

    fn emit_edit_action(&mut self, cx: &mut Cx, action: TimelineEditAction) {
        cx.widget_action(self.uid, action);
    }

    fn zoom_at(&mut self, cx: &mut Cx, scroll: f64, abs_x: f64) -> bool {
        let Some(next) = self.viewport().zoom_at(abs_x, scroll, 0.0) else {
            return false;
        };
        self.view_span = next.visible_frames.max(self.min_view_span());
        self.view_start = next.view_start;
        self.clamp_view_start();
        self.redraw(cx);
        cx.widget_action(
            self.uid,
            TimelineSurfaceAction::ZoomChanged {
                start_frame: self.view_start.round() as i64,
                visible_frames: self.view_span.round() as i64,
            },
        );
        true
    }

    fn normalized_zoom_delta(delta: f64, precise: bool) -> f64 {
        if precise {
            delta
        } else {
            // Classic wheels report coarse, platform-dependent step sizes.
            // Preserve step count while keeping one notch comparable across OSes.
            (delta / 120.0).round().clamp(-4.0, 4.0) * 120.0
        }
    }

    fn pan_time_by_pixels(&mut self, cx: &mut Cx, pixels: f64) -> bool {
        let time_width = self.time_rect().size.x.max(1.0);
        let old_start = self.view_start;
        self.view_start += pixels / time_width * self.view_span;
        self.clamp_view_start();
        if (self.view_start - old_start).abs() <= f64::EPSILON {
            return false;
        }
        self.redraw(cx);
        cx.widget_action(
            self.uid,
            TimelineSurfaceAction::ZoomChanged {
                start_frame: self.view_start.round() as i64,
                visible_frames: self.view_span.round() as i64,
            },
        );
        true
    }

    /// レーン列の縦スクロール(A3)。**方向は横パンと同じ約束**にしてある —
    /// `pan_time_by_pixels` が正の translation で「見る窓を先へ」動かすので、
    /// 縦も正の translation で「見る窓を下へ」動かす。
    fn scroll_lanes_by_pixels(&mut self, cx: &mut Cx, pixels: f64) -> bool {
        if self.max_scroll_y() <= 0.0 {
            return false;
        }
        let old = self.scroll_y;
        self.scroll_y = self.scroll_y + pixels;
        self.clamp_scroll_y();
        if (self.scroll_y - old).abs() <= f64::EPSILON {
            return false;
        }
        self.redraw(cx);
        true
    }

    fn apply_gesture_sample(&mut self, cx: &mut Cx, sample: GestureSample) {
        let applied = match self.scroll_gesture.update_sample(sample) {
            Some(TimelineScrollAction::PanPixels(pixels)) => self.pan_time_by_pixels(cx, pixels),
            Some(TimelineScrollAction::ScrollLanes(pixels)) => {
                self.scroll_lanes_by_pixels(cx, pixels)
            }
            Some(TimelineScrollAction::Zoom { delta, precise }) => self.zoom_at(
                cx,
                Self::normalized_zoom_delta(delta, precise),
                sample.centroid[0],
            ),
            None => false,
        };
        self.scroll_gesture
            .reject_owner_if_unapplied(sample.phase, applied);
    }

    fn point_in(rect: Rect, abs: DVec2) -> bool {
        abs.x >= rect.pos.x
            && abs.x < rect.pos.x + rect.size.x
            && abs.y >= rect.pos.y
            && abs.y < rect.pos.y + rect.size.y
    }

    /// プロパティ行の開閉三角(B2)。プロパティを持たないレーンには出さない —
    /// 押しても何も起きない三角は、押せない物を押せるように見せる嘘になる。
    fn fold_rect(&self, row: VisualRow, layer_id: u64) -> Option<Rect> {
        if self.property_count_for_lane(layer_id) == 0 {
            return None;
        }
        let size = (row.height * 0.42).clamp(7.0, 11.0);
        Some(Rect {
            pos: dvec2(
                self.rect.pos.x + 6.0,
                row.y + ((row.height - size) * 0.5).max(0.0),
            ),
            size: dvec2(size, size),
        })
    }

    /// レーン名の左端。三角がある行では三角の分だけ右へ寄せる。
    fn name_x(&self, layer_id: u64) -> f64 {
        self.rect.pos.x
            + if self.property_count_for_lane(layer_id) == 0 {
                9.0
            } else {
                20.0
            }
    }

    /// クリップの棒に乗っているか、乗っているならどこか。**掴み代は棒幅の 1/3 を
    /// 超えない** — 短いクリップで両端の掴み代が中央で出会うと、本体が掴めなくなる。
    fn clip_target(&self, lane_index: usize, abs: DVec2) -> Option<PointerTarget> {
        let lane = self.lanes.get(lane_index)?;
        if lane.duration <= 0 {
            return None;
        }
        let x0 = self.x_at_frame(lane.start as f64);
        let x1 = self.x_at_frame((lane.start + lane.duration) as f64);
        if x1 <= x0 || abs.x < x0 || abs.x >= x1 {
            return None;
        }
        let handle = self.trim_handle_width.min((x1 - x0) / 3.0).max(1.0);
        let edge = if abs.x < x0 + handle {
            Some(ClipEdge::Start)
        } else if abs.x >= x1 - handle {
            Some(ClipEdge::End)
        } else {
            None
        };
        Some(PointerTarget::Clip {
            layer_id: lane.id,
            lane_index,
            edge,
            start: lane.start,
            duration: lane.duration,
            locked: lane.locked,
        })
    }

    /// ロケータの当たり判定(発注 S5)。**形は統一**(裁定2026-08-08)なので掴み代も
    /// 1つ — 種類ごとに広さを変えない。`x_at_frame` の逆(`frame_at_x`)を使わないのは
    /// `PointerTarget::Marker` が生の `frame` を運びたいから(`begin` の doc 参照)。
    fn marker_at(&self, abs_x: f64) -> Option<(usize, i64)> {
        const HIT_HALF_WIDTH: f64 = 4.0;
        self.markers.iter().enumerate().find_map(|(index, marker)| {
            let x = self.x_at_frame(marker.frame as f64);
            ((abs_x - x).abs() <= HIT_HALF_WIDTH).then_some((index, marker.frame))
        })
    }

    /// **カーソルもクリックもここだけを読む**。1つの関数が答えを持つので、
    /// 「伸縮できない所で EwResize が出る」(A2)ような食い違いが起こせない。
    fn pointer_target(&self, abs: DVec2) -> PointerTarget {
        let rail_right = self.rect.pos.x + self.rail_width;
        if abs.y < self.body_top() {
            // ルーラー帯。スクラブできるのは時間側だけで、レール側の見出しは何でもない。
            if abs.x >= rail_right {
                if let Some((index, frame)) = self.marker_at(abs.x) {
                    return PointerTarget::Marker { index, frame };
                }
                return PointerTarget::Ruler;
            }
            return PointerTarget::Empty;
        }
        for row in self.visual_rows() {
            if abs.y < row.y || abs.y >= row.y + row.height {
                continue;
            }
            let VisualRowKind::Lane(lane_index) = row.kind else {
                // プロパティ行はまだ掴む物が無い(キー選択は別の欠陥)。
                return PointerTarget::Empty;
            };
            let Some(lane) = self.lanes.get(lane_index) else {
                return PointerTarget::Empty;
            };
            if abs.x < rail_right {
                if let Some(fold) = self.fold_rect(row, lane.id) {
                    if Self::point_in(fold, abs) {
                        return PointerTarget::Fold { layer_id: lane.id };
                    }
                }
                let rects = Self::lane_toggle_rects(self.rail_width, self.rect.pos.x, row);
                if let Some(flag) = Self::flag_at_point(&rects, abs) {
                    return PointerTarget::Flag {
                        layer_id: lane.id,
                        flag,
                    };
                }
                return PointerTarget::Rail {
                    layer_id: lane.id,
                    from_front: lane_index,
                    grab_offset: abs.y - row.y,
                    locked: lane.locked,
                };
            }
            return self
                .clip_target(lane_index, abs)
                .unwrap_or(PointerTarget::Empty);
        }
        PointerTarget::Empty
    }

    /// **伸縮できる所だけが EwResize**(A2)。それ以外は掴める物なら Grab、
    /// 押せる物なら Hand、何も無ければ既定。
    ///
    /// **LOCKED の行はここで `NotAllowed`**。store は locked な layer への
    /// `SetTiming`/`SetOrder` を拒む(`check_not_locked`)ので、掴ませてしまうと
    /// 伸びて見えてから次の投影で戻る = 一度嘘をつくことになる。壁は掴む前、
    /// 指がそこへ乗った瞬間に出す(Q0)。錠は M/S/L の L で外せるので行き止まりではない。
    fn set_hover_cursor(&self, cx: &mut Cx, abs: DVec2) {
        let cursor = match self.pointer_target(abs) {
            PointerTarget::Ruler | PointerTarget::Marker { .. } => MouseCursor::EwResize,
            PointerTarget::Clip { locked: true, .. } | PointerTarget::Rail { locked: true, .. } => {
                MouseCursor::NotAllowed
            }
            PointerTarget::Clip { edge: Some(_), .. } => MouseCursor::EwResize,
            PointerTarget::Clip { edge: None, .. } => MouseCursor::Grab,
            PointerTarget::Rail { .. } => MouseCursor::Grab,
            PointerTarget::Fold { .. } | PointerTarget::Flag { .. } => MouseCursor::Hand,
            PointerTarget::Empty => MouseCursor::Default,
        };
        cx.set_cursor(cursor);
    }

    /// クリック選択(A5)。`TimelineLane.selected` は描画側が既に読んでいるので、
    /// ここで倒せば窓の色がその場で変わる。同時に意図を外へも出す — 選択の正本は
    /// shell の `Session` なので、widget 側は先に見せているだけ。
    fn select_lane(&mut self, cx: &mut Cx, layer_id: Option<u64>, additive: bool) {
        // 先に見せてから意図を出す。正本は `session.selection` で、そこから
        // `timeline_pane::rows` → `TimelineLane.selected` として同じフレームのうちに
        // 戻ってくる(shell が `TimelineUpdate::Model` で `set_model` を呼ぶ)。
        // ここで倒すのは「戻ってくるまでの1フレームを詰める」ためだけで、
        // 食い違ったら次の `set_model` が store の答えで上書きする。
        let mut changed = false;
        for lane in self.lanes.iter_mut() {
            let next = match layer_id {
                Some(id) if lane.id == id => {
                    if additive {
                        !lane.selected
                    } else {
                        true
                    }
                }
                _ => {
                    if additive {
                        lane.selected
                    } else {
                        false
                    }
                }
            };
            if lane.selected != next {
                lane.selected = next;
                changed = true;
            }
        }
        if changed {
            self.redraw(cx);
        }
        self.emit_edit_action(cx, TimelineEditAction::Select { layer_id, additive });
    }

    fn toggle_fold(&mut self, cx: &mut Cx, layer_id: u64) {
        if let Some(index) = self
            .collapsed_lanes
            .iter()
            .position(|collapsed| *collapsed == layer_id)
        {
            self.collapsed_lanes.remove(index);
        } else {
            self.collapsed_lanes.push(layer_id);
        }
        self.clamp_scroll_y();
        self.redraw(cx);
    }

    fn draw_rect(&mut self, cx: &mut Cx2d, rect: Rect, color: Vec4f) {
        if rect.size.x <= 0.0 || rect.size.y <= 0.0 {
            return;
        }
        self.draw_item.color = color;
        self.draw_item.draw_abs(cx, rect);
    }

    /// 行の物はルーラーとペイン下端で切る。縦スクロールが入った以上、行は窓から
    /// はみ出しうる — はみ出た分をそのまま描くと隣のペインへ漏れる。
    fn draw_body_rect(&mut self, cx: &mut Cx2d, rect: Rect, color: Vec4f) {
        let top = self.body_top();
        let bottom = self.body_bottom();
        let y0 = rect.pos.y.max(top);
        let y1 = (rect.pos.y + rect.size.y).min(bottom);
        if y1 <= y0 {
            return;
        }
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(rect.pos.x, y0),
                size: dvec2(rect.size.x, y1 - y0),
            },
            color,
        );
    }

    fn draw_label(&mut self, cx: &mut Cx2d, pos: DVec2, text: &str, color: Vec4f, size: f32) {
        self.draw_text.color = color;
        self.draw_text.text_style.font_size = size;
        self.draw_text.draw_abs(cx, pos, text);
    }

    fn lane_color(index: usize) -> Vec4f {
        const COLORS: [[f32; 4]; 12] = [
            [0.92, 0.92, 0.88, 1.0],
            [0.71, 0.55, 0.47, 1.0],
            [0.47, 0.59, 0.67, 1.0],
            [0.82, 0.78, 0.92, 1.0],
            [0.63, 0.47, 0.59, 1.0],
            [0.92, 0.78, 0.59, 1.0],
            [0.86, 0.35, 0.35, 1.0],
            [0.35, 0.71, 0.67, 1.0],
            [0.55, 0.43, 0.67, 1.0],
            [0.29, 0.35, 0.51, 1.0],
            [0.78, 0.78, 0.78, 1.0],
            [0.55, 0.55, 0.51, 1.0],
        ];
        let color = COLORS[index % COLORS.len()];
        vec4(color[0], color[1], color[2], color[3])
    }

    // bar の差し色は3段の優先順位(canon: timeline-semantics.html)。
    // hidden → muted / dragging → ACCENT が label_color 自体より優先される
    // ("dragging=ACCENT・hidden=muted が優先" の順に読む: hidden が最優先)。
    // muted は tokens.rs の ink.muted(#x757575)を焼き直した値 — この描画関数
    // 一式はまだトークン参照ではなく Vec4 定数で組んであるので、既存の並びに合わせる。
    const MUTED_LABEL_COLOR: Vec4f = vec4(0.459, 0.459, 0.459, 1.0);

    fn is_dragging_lane(&self, lane_id: u64) -> bool {
        matches!(self.drag, TimelineGesture::Lane { layer_id, .. } if layer_id == lane_id)
    }

    // M/S/L の当たり判定は描画と同じ数字でなければクリックが少しずつズレていく。
    // 描画側(draw_lane)とヒットテスト側(pointer_target)の両方がここを呼ぶことで、
    // control_h/control_y/control_x/15.0 ストライド/12.0 幅を手で二重に書かない。
    // self を取らない associated fn にしてあるのは、Widget インスタンスを
    // 起こさずにテストできるようにするため(このファイルの他のテストも
    // TimelineGesture/TimelineViewport という「純関数の側」だけを見ている)。
    fn lane_toggle_rects(rail_width: f64, origin_x: f64, row: VisualRow) -> [Rect; 3] {
        let control_h = (row.height - 4.0).clamp(8.0, 13.0);
        let control_y = row.y + (row.height - control_h) * 0.5;
        let control_x = origin_x + rail_width - 45.0;
        let mut rects = [Rect::default(); 3];
        for (index, rect) in rects.iter_mut().enumerate() {
            *rect = Rect {
                pos: dvec2(control_x + index as f64 * 15.0, control_y),
                size: dvec2(12.0, control_h),
            };
        }
        rects
    }

    // [`Self::lane_toggle_rects`] のどれかに abs が乗っているかだけを見る、
    // これも純関数(index 0..3 と LaneFlag の対応が生まれる唯一の場所)。
    fn flag_at_point(rects: &[Rect; 3], abs: DVec2) -> Option<LaneFlag> {
        for (index, rect) in rects.iter().enumerate() {
            if Self::point_in(*rect, abs) {
                return Some(match index {
                    0 => LaneFlag::Hidden,
                    1 => LaneFlag::Solo,
                    _ => LaneFlag::Locked,
                });
            }
        }
        None
    }

    fn draw_lane(&mut self, cx: &mut Cx2d, lane: &TimelineLane, row: VisualRow, zebra: bool) {
        let bg = if lane.selected {
            vec4(0.34, 0.31, 0.28, 1.0)
        } else if zebra {
            vec4(0.205, 0.205, 0.205, 1.0)
        } else {
            vec4(0.225, 0.225, 0.225, 1.0)
        };
        self.draw_body_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(self.rect.size.x, row.height),
            },
            bg,
        );

        // 優先順位は hidden が最優先、次に dragging、既定が label_color。
        // ("dragging=ACCENT・hidden=muted が優先" — 両方立つことは無いが、
        // 順番が hidden を先に書いている通りに倒す)
        let color = if lane.hidden {
            Self::MUTED_LABEL_COLOR
        } else if self.is_dragging_lane(lane.id) {
            self.playhead_color
        } else {
            Self::lane_color(lane.label_color)
        };
        // Sticky-note tab: full lane height, left aligned. It labels the row
        // without adding a second, misleading 8x8 "content height" signal.
        self.draw_body_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(4.0, row.height),
            },
            color,
        );

        let whole = self.row_is_whole(row);

        // 開閉三角(B2)。プロパティを持つ行にだけ出る。
        if let Some(fold) = self.fold_rect(row, lane.id) {
            if whole {
                let collapsed = self.is_collapsed(lane.id);
                self.draw_label(
                    cx,
                    dvec2(fold.pos.x, fold.pos.y - 1.0),
                    if collapsed { "▶" } else { "▼" },
                    vec4(0.62, 0.62, 0.62, 1.0),
                    (fold.size.y * 0.8) as f32,
                );
            }
        }

        // レーン名の大きさは行高から導出(比率の原則を字にも適用 — 裁定271)。
        // 行高は固定になったが、比率の関係は残す(`type_ratio` の doc を参照)。
        let name_size = (row.height * self.type_ratio / self.ink_k).clamp(6.0, 11.0);
        let text_y = row.y + ((row.height - name_size) * 0.5).max(0.0);
        // 名前は M/S/L の手前で止める。長い名は切って「…」— 走らせて衝突させない
        let name_x = self.name_x(lane.id);
        let controls_left = Self::lane_toggle_rects(self.rail_width, self.rect.pos.x, row)[0].pos.x;
        let name_budget = ((controls_left - 4.0 - name_x) / (name_size * 0.58)).max(1.0) as usize;
        let display_name: String = if lane.name.chars().count() > name_budget {
            let mut cut: String = lane.name.chars().take(name_budget.saturating_sub(1)).collect();
            cut.push('…');
            cut
        } else {
            lane.name.clone()
        };
        if whole {
            self.draw_label(
                cx,
                dvec2(name_x, text_y),
                &display_name,
                if lane.selected {
                    vec4(0.93, 0.91, 0.84, 1.0)
                } else {
                    vec4(0.72, 0.72, 0.72, 1.0)
                },
                name_size as f32,
            );
        }

        // Live の文法: on のトグルは意味色のベタ + 暗インク(極性反転)。
        // activator=琥珀 #ffad56 / solo=シアン #03c3d5 (.ask ChosenDefault/ChosenAlternative)。
        // lock は Live に無い操作なので無彩の明面で「掴めない」を言う。
        const TOGGLE_ON: [Vec4; 3] = [
            vec4(1.0, 0.678, 0.337, 1.0),
            vec4(0.012, 0.765, 0.835, 1.0),
            vec4(0.569, 0.569, 0.569, 1.0),
        ];
        // 描画とヒットテストが同じ数字を見るように lane_toggle_rects を経由する
        // (このコメント直上のトグル配色以外、幾何は一切ここに手で書かない)。
        let toggle_rects = Self::lane_toggle_rects(self.rail_width, self.rect.pos.x, row);
        for (index, (label, active)) in [("M", lane.hidden), ("S", lane.solo), ("L", lane.locked)]
            .into_iter()
            .enumerate()
        {
            let rect = toggle_rects[index];
            self.draw_body_rect(
                cx,
                rect,
                if active {
                    TOGGLE_ON[index]
                } else {
                    vec4(0.118, 0.118, 0.118, 1.0)
                },
            );
            if whole {
                self.draw_label(
                    cx,
                    dvec2(
                        rect.pos.x + 3.2,
                        rect.pos.y + ((rect.size.y - self.text_xs) * 0.5).max(0.0),
                    ),
                    label,
                    if active {
                        vec4(0.027, 0.027, 0.027, 1.0)
                    } else {
                        vec4(0.55, 0.55, 0.55, 1.0)
                    },
                    self.text_xs as f32,
                );
            }
        }
    }

    /// クリップの棒だけ。**格子より後**に描く(Live の層: 地 → 格子 → クリップ)。
    fn draw_lane_clip(&mut self, cx: &mut Cx2d, lane: &TimelineLane, row: VisualRow) {
        let color = if lane.hidden {
            Self::MUTED_LABEL_COLOR
        } else if self.is_dragging_lane(lane.id) {
            self.playhead_color
        } else {
            Self::lane_color(lane.label_color)
        };
        let visible_start = self.view_start;
        let visible_end = self.view_start + self.view_span;
        let clip_start = lane.start as f64;
        let clip_end = (lane.start + lane.duration) as f64;
        let left = clip_start.max(visible_start);
        let right = clip_end.min(visible_end);
        if right > left {
            let x0 = self.x_at_frame(left);
            let x1 = self.x_at_frame(right);
            self.draw_body_rect(
                cx,
                Rect {
                    pos: dvec2(x0, row.y),
                    // One separator pixel is the only vertical gap. The clip
                    // otherwise fits the lane instead of floating inside it.
                    size: dvec2((x1 - x0).max(1.0), (row.height - 1.0).max(1.0)),
                },
                color,
            );
            self.draw_lane_waveform(cx, lane, row, color, x0, x1, left, right);
            // トリム掴み代を「見える物」にする(A1/A2)。掴める幅と同じ幅を
            // 少し明るく置くだけ — 別部品ではないので色相は増やさない。
            let handle = self.trim_handle_width.min((x1 - x0) / 3.0).max(1.0);
            let grip = vec4(
                (color.x * 0.55 + 0.45).min(1.0),
                (color.y * 0.55 + 0.45).min(1.0),
                (color.z * 0.55 + 0.45).min(1.0),
                1.0,
            );
            if clip_start >= visible_start {
                self.draw_body_rect(
                    cx,
                    Rect {
                        pos: dvec2(x0, row.y),
                        size: dvec2(handle, (row.height - 1.0).max(1.0)),
                    },
                    grip,
                );
            }
            if clip_end <= visible_end {
                self.draw_body_rect(
                    cx,
                    Rect {
                        pos: dvec2(x1 - handle, row.y),
                        size: dvec2(handle, (row.height - 1.0).max(1.0)),
                    },
                    grip,
                );
            }
        }
    }

    /// クリップの中に波形を描く。**別部品ではない** — 色相を増やさず、クリップの
    /// 色をそのまま暗くしただけの物を重ねる(Ableton も波形はクリップ色の濃淡)。
    ///
    /// hero が MV である以上、BPM グリッドへ手で合わせる作業は「聞きながら置く」
    /// 作業で、その前段として**まず波の形が見えている**ことが要る。
    ///
    /// `visible_left`/`visible_right` は画面に出ているフレーム範囲(クリップの
    /// 一部が視野の外にある時に効く)。`lane.waveform` はクリップ全体ぶんなので、
    /// 「クリップ内の相対位置」で添字を引く。
    #[allow(clippy::too_many_arguments)]
    fn draw_lane_waveform(
        &mut self,
        cx: &mut Cx2d,
        lane: &TimelineLane,
        row: VisualRow,
        color: Vec4f,
        x0: f64,
        x1: f64,
        visible_left: f64,
        visible_right: f64,
    ) {
        let peaks = &lane.waveform;
        if peaks.is_empty() || lane.duration <= 0 || x1 - x0 < 2.0 {
            return;
        }
        let body_height = (row.height - 1.0).max(1.0);
        // 上下2pxは空ける — 波形がレーンの境界線に触れると、隣の行と繋がって見える。
        let half = (body_height * 0.5 - 2.0).max(1.0);
        let mid = row.y + body_height * 0.5;
        let wave = vec4(color.x * 0.30, color.y * 0.30, color.z * 0.30, 1.0);

        let duration = lane.duration as f64;
        let clip_start = lane.start as f64;
        let left_fraction = ((visible_left - clip_start) / duration).clamp(0.0, 1.0);
        let right_fraction = ((visible_right - clip_start) / duration).clamp(0.0, 1.0);
        let span = right_fraction - left_fraction;
        if span <= 0.0 {
            return;
        }
        // 2px 刻み。1px にしても目には同じで、描画インスタンスだけ倍になる。
        const COLUMN_WIDTH: f64 = 2.0;
        let columns = (((x1 - x0) / COLUMN_WIDTH).floor() as usize).max(1);
        for column in 0..columns {
            let fraction = left_fraction + span * (column as f64 + 0.5) / columns as f64;
            let index = ((fraction * peaks.len() as f64) as usize).min(peaks.len() - 1);
            let (min, max) = peaks[index];
            let top = mid - f64::from(max).clamp(0.0, 1.0) * half;
            let bottom = mid - f64::from(min).clamp(-1.0, 0.0) * half;
            self.draw_body_rect(
                cx,
                Rect {
                    pos: dvec2(x0 + column as f64 * COLUMN_WIDTH, top),
                    // 無音でも1pxの中心線は残す — 「音は在るが今は静か」と
                    // 「そもそも波形が無い」を見分けられなくなるため。
                    size: dvec2(COLUMN_WIDTH - 1.0, (bottom - top).max(1.0)),
                },
                wave,
            );
        }
    }

    fn draw_property(&mut self, cx: &mut Cx2d, property: &TimelinePropertyLane, row: VisualRow) {
        self.draw_body_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x, row.y),
                size: dvec2(self.rect.size.x, row.height),
            },
            vec4(0.18, 0.18, 0.18, 1.0),
        );
        if self.row_is_whole(row) {
            self.draw_label(
                cx,
                dvec2(self.rect.pos.x + 30.0, row.y + (row.height - 8.0) * 0.5),
                &property.name,
                vec4(0.57, 0.57, 0.57, 1.0),
                self.text_sm as f32,
            );
        }
    }

    /// キーの菱形だけ。クリップと同じ層(格子より上)。
    fn draw_property_keys(&mut self, cx: &mut Cx2d, property: &TimelinePropertyLane, row: VisualRow) {
        let key_color = self
            .lanes
            .iter()
            .find(|lane| lane.id == property.layer_id)
            .map(|lane| Self::lane_color(lane.label_color))
            .unwrap_or_else(|| vec4(0.92, 0.78, 0.59, 1.0));
        let key_size = 8.0;
        let key_y = row.y + (row.height - key_size) * 0.5;
        for &frame in &property.keys {
            let frame = frame as f64;
            if frame < self.view_start || frame > self.view_start + self.view_span {
                continue;
            }
            let x = self.x_at_frame(frame) - key_size * 0.5;
            self.draw_body_rect(
                cx,
                Rect {
                    pos: dvec2(x, key_y),
                    size: dvec2(key_size, key_size),
                },
                key_color,
            );
        }
    }

    // 面(band)と線(line)は同じ時間軸を「頻度」で分業する。線は全ティックに立つ細かい
    // リズムで、面はメジャー刻みだけを周期にした粗いリズム。粗いリズムを先に敷いておかないと
    // 線が透けて見えるべき下地がなくなるので、この帯は線より先に描く。
    fn draw_time_bands(&mut self, cx: &mut Cx2d, minor: i64, major: i64) {
        // 帯の周期はメジャー刻みだが、メジャーがマイナーを下回る異常値では周期が潰れて
        // 無限ループになりかねないので、マイナーを下限として使う。
        let step = major.max(minor).max(1);
        let rail_x = self.rect.pos.x + self.rail_width;
        let right_x = self.rect.pos.x + self.rect.size.x;
        let top_y = self.body_top();
        let bottom_y = self.body_bottom();
        let body_height = (bottom_y - top_y).max(0.0);
        if right_x <= rail_x || body_height <= 0.0 {
            return;
        }
        let first_major = (self.view_start / step as f64).floor() as i64 * step;
        let last = (self.view_start + self.view_span).ceil() as i64;
        let mut frame = first_major;
        while frame <= last {
            // 周期の偶奇だけで「点灯」区間を決める。view_start に依存させないことで、
            // スクロールしても同じメジャー区間が同じ位相のまま光り続ける。
            if frame.div_euclid(step).rem_euclid(2) == 0 {
                let x0 = self.x_at_frame(frame as f64).max(rail_x);
                let x1 = self.x_at_frame((frame + step) as f64).min(right_x);
                if x1 > x0 {
                    self.draw_rect(
                        cx,
                        Rect {
                            pos: dvec2(x0, top_y),
                            size: dvec2(x1 - x0, body_height),
                        },
                        vec4(1.0, 1.0, 1.0, self.band_alpha as f32),
                    );
                }
            }
            frame = frame.saturating_add(step);
        }
    }

    /// 時間場の背景(帯 + 縦線)。**クリップより先**に描く — Live は格子を
    /// クリップの下に敷く。上に乗せると全部が網をかけたように濁る。
    fn draw_time_field_background(&mut self, cx: &mut Cx2d, lane_height: f64) {
        let (minor, major) = self
            .viewport()
            .tick_steps(self.fps_num, self.fps_den, lane_height, self.tick_row_floor);
        self.draw_time_bands(cx, minor, major);
        // マイナー刻みの実ピクセル間隔。tick_steps は行高の床上げで「選ぶ刻み」を
        // 間引くだけなので、ズームでその刻みの画面幅がまだ狭い一瞬が残る —
        // そこをハードカットではなくフェードで埋めて「ポップ」させない。
        let minor_px = minor as f64 * (self.time_rect().size.x / self.view_span.max(1.0));
        let minor_fade = ((minor_px - self.tick_fade_from) / (self.tick_fade_to - self.tick_fade_from))
            .clamp(0.0, 1.0);
        let first_minor = (self.view_start / minor as f64).ceil() as i64 * minor;
        let last = (self.view_start + self.view_span).ceil() as i64;
        let mut frame = first_minor;
        while frame <= last {
            let is_major = frame.rem_euclid(major) == 0;
            let x = self.x_at_frame(frame as f64);
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, self.body_top()),
                    size: dvec2(1.0, self.body_height().max(1.0)),
                },
                if is_major {
                    vec4(0.05, 0.05, 0.05, 0.38)
                } else {
                    vec4(0.02, 0.02, 0.02, (0.20 * minor_fade) as f32)
                },
            );
            frame = frame.saturating_add(minor.max(1));
        }
    }

    fn draw_grid_and_ruler(&mut self, cx: &mut Cx2d, lane_height: f64) {
        let (minor, major) = self
            .viewport()
            .tick_steps(self.fps_num, self.fps_den, lane_height, self.tick_row_floor);
        let minor_px = minor as f64 * (self.time_rect().size.x / self.view_span.max(1.0));
        let minor_fade = ((minor_px - self.tick_fade_from) / (self.tick_fade_to - self.tick_fade_from))
            .clamp(0.0, 1.0);
        let last = (self.view_start + self.view_span).ceil() as i64;

        // Ruler is deliberately emitted in a fresh foreground draw call after
        // clips and body grid, so bars can never overwrite its ticks again.
        self.draw_item.new_draw_call(cx);
        self.draw_rect(
            cx,
            Rect {
                pos: self.rect.pos,
                size: dvec2(self.rect.size.x, self.ruler_height),
            },
            vec4(0.245, 0.245, 0.245, 1.0),
        );
        self.draw_rect(
            cx,
            Rect {
                pos: dvec2(self.rect.pos.x + self.rail_width - 1.0, self.rect.pos.y),
                size: dvec2(1.0, self.rect.size.y),
            },
            vec4(0.10, 0.10, 0.10, 1.0),
        );

        let zoom_percent = (self.duration_frames as f64 / self.view_span.max(1.0) * 100.0).round();
        self.draw_label(
            cx,
            dvec2(self.rect.pos.x + 9.0, self.rect.pos.y + 5.0),
            &format!("TIME  {zoom_percent:.0}%"),
            vec4(0.50, 0.50, 0.50, 1.0),
            self.text_sm as f32,
        );

        let first_minor = (self.view_start / minor as f64).ceil() as i64 * minor;
        let mut frame = first_minor;
        while frame <= last {
            let is_major = frame.rem_euclid(major) == 0;
            let x = self.x_at_frame(frame as f64);
            let tick_height = if is_major { 11.0 } else { 5.0 };
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x, self.rect.pos.y + self.ruler_height - tick_height),
                    size: dvec2(1.0, tick_height),
                },
                if is_major {
                    vec4(0.47, 0.47, 0.47, 1.0)
                } else {
                    vec4(0.12, 0.12, 0.12, (0.55 * minor_fade) as f32)
                },
            );
            if is_major {
                let seconds = frame as f64 / self.fps();
                let label = if major as f64 >= self.fps() {
                    format!("{seconds:.0}")
                } else {
                    format!("{seconds:.1}")
                };
                self.draw_label(
                    cx,
                    dvec2(x + 2.0, self.rect.pos.y + 1.0),
                    &label,
                    vec4(0.55, 0.55, 0.55, 1.0),
                    self.text_md as f32,
                );
            }
            frame = frame.saturating_add(minor.max(1));
        }
    }

    /// ロケータの目盛り(発注 S5)。**形は統一**(裁定2026-08-08) — 全マーカーが
    /// 同じ旗を使う、種類ごとの形は無い(範囲ロケータは次の波、非目標)。
    /// 色は `marker_color`(= `mod.tokens.accent.on`、script_mod 側の doc 参照)。
    fn draw_markers(&mut self, cx: &mut Cx2d) {
        if self.markers.is_empty() {
            return;
        }
        let time = self.time_rect();
        let markers = self.markers.clone();
        self.draw_item.new_draw_call(cx);
        for marker in &markers {
            let x = self.x_at_frame(marker.frame as f64);
            if x < time.pos.x - 1.5 || x > time.pos.x + time.size.x + 1.5 {
                continue;
            }
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(x - 1.5, self.rect.pos.y + self.ruler_height - 6.0),
                    size: dvec2(3.0, 6.0),
                },
                self.marker_color,
            );
        }
    }

    /// 掴んだ行が指に付いてくる(A4)。挿入先の線だけでは「掴んだ物が静止したまま
    /// ドラッグする」ことになり、掴んだ実感が無い。
    fn floating_lane_row(&self) -> Option<(usize, f64)> {
        let TimelineGesture::Lane {
            layer_id,
            grab_offset,
            pointer_y,
            ..
        } = self.drag
        else {
            return None;
        };
        let lane_index = self.lanes.iter().position(|lane| lane.id == layer_id)?;
        let height = self.lane_height();
        let y = (pointer_y - grab_offset).clamp(self.body_top(), (self.body_bottom() - height).max(self.body_top()));
        Some((lane_index, y))
    }

    fn draw_playhead_and_drop_target(&mut self, cx: &mut Cx2d) {
        let time = self.time_rect();
        let playhead_x = self.x_at_frame(self.playhead as f64);
        if playhead_x >= time.pos.x && playhead_x <= time.pos.x + time.size.x {
            self.draw_item.new_draw_call(cx);
            // playhead = ACCENT 1.5x(canon: timeline-semantics.html S5b)。
            // 通常グリッド線太さ(GRID_LINE_WIDTH)の倍率も色も --hot で振れる。
            let playhead_width = GRID_LINE_WIDTH * self.playhead_width_scale;
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(playhead_x, self.rect.pos.y),
                    size: dvec2(playhead_width, self.rect.size.y),
                },
                self.playhead_color,
            );
            self.draw_rect(
                cx,
                Rect {
                    pos: dvec2(playhead_x - 3.0, self.rect.pos.y),
                    size: dvec2(7.0, 7.0),
                },
                self.playhead_color,
            );
        }

        if let TimelineGesture::Lane {
            target_from_front, ..
        } = self.drag
        {
            if let Some(row) = self
                .visual_rows()
                .into_iter()
                .filter(|row| matches!(row.kind, VisualRowKind::Lane(_)))
                .nth(target_from_front)
            {
                self.draw_item.new_draw_call(cx);
                self.draw_body_rect(
                    cx,
                    Rect {
                        pos: dvec2(self.rect.pos.x, row.y),
                        size: dvec2(self.rect.size.x, 2.0),
                    },
                    vec4(0.85, 0.71, 0.45, 1.0),
                );
            }
        }
    }
}

impl Widget for TimelineSurface {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event.hits_with_capture_overload(cx, self.draw_bg.area(), true) {
            Hit::FingerDown(fe) if fe.is_primary_hit() => {
                let target = self.pointer_target(fe.abs);
                match target {
                    // M/S/L はレールグリフへの直接操作(canon 1:0:0:0)。ドラッグの
                    // 起点にはしない。
                    PointerTarget::Flag { layer_id, flag } => {
                        self.emit_input_action(
                            cx,
                            TimelineSurfaceAction::ToggleLaneFlag { layer_id, flag },
                        );
                        return;
                    }
                    // 開閉三角も直接操作。畳むのは見え方だけなので Document へは行かない。
                    PointerTarget::Fold { layer_id } => {
                        self.toggle_fold(cx, layer_id);
                        return;
                    }
                    _ => {}
                }

                // 選択(A5)。追加選択は shift/⌘(どちらも「今の選択に足す」の
                // 慣用で、片方だけ効くと迷う)。
                let additive = fe.modifiers.shift || fe.modifiers.logo;
                match target {
                    PointerTarget::Rail { layer_id, .. }
                    | PointerTarget::Clip { layer_id, .. } => {
                        self.select_lane(cx, Some(layer_id), additive)
                    }
                    PointerTarget::Empty => self.select_lane(cx, None, additive),
                    _ => {}
                }

                let viewport = self.viewport();
                let action = self.drag.begin(&viewport, fe.abs, target);
                match self.drag {
                    TimelineGesture::Playhead => cx.set_cursor(MouseCursor::EwResize),
                    TimelineGesture::Lane { .. } => {
                        cx.set_cursor(MouseCursor::Grabbing);
                        self.redraw(cx);
                    }
                    TimelineGesture::Clip { edge, .. } => {
                        cx.set_cursor(if edge.is_some() {
                            MouseCursor::EwResize
                        } else {
                            MouseCursor::Grabbing
                        });
                        self.redraw(cx);
                    }
                    TimelineGesture::None => {}
                }
                if let Some(action) = action {
                    self.emit_input_action(cx, action);
                }
            }
            // ルーラーの右クリック(発注 S5)。空所なら置く、既存ロケータの上なら
            // 消す — 同じ帯の同じボタンが両方の意図を持つ(置ける所でしか消えない、
            // 消せる所でしか置けない、が Q0 の裏側)。ドラッグは起こさない
            // (`self.drag` はここでは触らない、1クリック = 1意図)。
            Hit::FingerDown(fe)
                if fe
                    .mouse_button()
                    .is_some_and(|button| button.is_secondary()) =>
            {
                match self.pointer_target(fe.abs) {
                    PointerTarget::Marker { index, .. } => {
                        self.emit_edit_action(cx, TimelineEditAction::RemoveMarker { index });
                    }
                    PointerTarget::Ruler => {
                        let frame = self.viewport().frame_at_x(fe.abs.x);
                        self.emit_edit_action(cx, TimelineEditAction::AddMarker { frame });
                    }
                    _ => {}
                }
            }
            Hit::FingerMove(fe) => {
                let viewport = self.viewport();
                if let Some(action) = self.drag.pointer_move(&viewport, fe.abs) {
                    self.emit_input_action(cx, action);
                } else if matches!(self.drag, TimelineGesture::Lane { .. }) {
                    let target_from_front = self.drop_index_at_y(fe.abs.y);
                    self.drag.move_lane_target(target_from_front);
                    self.drag.track_pointer_y(fe.abs.y);
                    self.redraw(cx);
                } else if let Some((lane_index, _, start, duration)) =
                    self.drag.clip_timing_at(&viewport, fe.abs.x, self.duration_frames)
                {
                    // 窓に「今の形」を先に見せる。確定(= 意図を外へ出す)は
                    // FingerUp の1回だけなので、途中経過が undo を汚さない。
                    let moved = match self.lanes.get_mut(lane_index) {
                        Some(lane) if lane.start != start || lane.duration != duration => {
                            lane.start = start;
                            lane.duration = duration;
                            true
                        }
                        _ => false,
                    };
                    if moved {
                        self.drag.mark_clip_changed();
                        self.redraw(cx);
                    }
                }
            }
            Hit::FingerUp(_) => {
                if let Some(action) = self.drag.pointer_up() {
                    self.emit_input_action(cx, action);
                }
                // トリム/移動は1ジェスチャ = 1つの意図(= 1 undo)。restack と同じ形。
                if let Some((lane_index, layer_id, edge)) = self.drag.committed_clip() {
                    let timing = self
                        .lanes
                        .get(lane_index)
                        .map(|lane| (lane.start, lane.duration));
                    if let Some((start, duration)) = timing {
                        self.emit_edit_action(
                            cx,
                            TimelineEditAction::SetClipTiming {
                                layer_id,
                                start,
                                duration,
                                edge,
                            },
                        );
                    }
                }
                self.drag = TimelineGesture::None;
                cx.set_cursor(MouseCursor::Default);
                self.redraw(cx);
            }
            Hit::FingerHoverIn(fe) | Hit::FingerHoverOver(fe) => {
                self.set_hover_cursor(cx, fe.abs);
            }
            Hit::FingerHoverOut(_) => cx.set_cursor(MouseCursor::Default),
            Hit::FingerScroll(fs) => {
                self.apply_gesture_sample(cx, GestureSample::from_makepad_scroll(&fs));
            }
            Hit::FingerGesture(fe) => {
                self.apply_gesture_sample(cx, GestureSample::from_makepad_gesture(&fe));
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.rect = cx.walk_turtle(walk);
        self.clamp_scroll_y();
        self.draw_bg.draw_abs(cx, self.rect);
        self.draw_item.new_draw_call(cx);

        // 層(Live の積み): レーン地/rail → 格子(帯+縦線) → クリップ/キー →
        // 掴んで浮いている行 → ルーラー。格子をクリップの上に乗せると全体が
        // 網をかけたように濁る。
        let lanes = self.lanes.clone();
        let properties = self.property_lanes.clone();
        let rows = self.visual_rows();
        let floating = self.floating_lane_row();
        let mut lane_number = 0usize;
        for row in rows.iter().copied() {
            let is_floating = matches!(
                (row.kind, floating),
                (VisualRowKind::Lane(index), Some((floating_index, _))) if index == floating_index
            );
            if matches!(row.kind, VisualRowKind::Lane(_)) {
                lane_number += 1;
            }
            if is_floating || !self.row_is_visible(row) {
                continue;
            }
            match row.kind {
                VisualRowKind::Lane(index) => {
                    self.draw_lane(cx, &lanes[index], row, (lane_number - 1) % 2 == 1);
                }
                VisualRowKind::Property(index) => {
                    self.draw_property(cx, &properties[index], row);
                }
            }
            self.draw_body_rect(
                cx,
                Rect {
                    pos: dvec2(self.rect.pos.x, row.y + row.height - 1.0),
                    size: dvec2(self.rect.size.x, 1.0),
                },
                vec4(0.13, 0.13, 0.13, 1.0),
            );
        }

        self.draw_time_field_background(cx, self.lane_height());
        for row in rows.iter().copied() {
            let is_floating = matches!(
                (row.kind, floating),
                (VisualRowKind::Lane(index), Some((floating_index, _))) if index == floating_index
            );
            if is_floating || !self.row_is_visible(row) {
                continue;
            }
            match row.kind {
                VisualRowKind::Lane(index) => self.draw_lane_clip(cx, &lanes[index], row),
                VisualRowKind::Property(index) => self.draw_property_keys(cx, &properties[index], row),
            }
        }

        // 掴んだ行は最後に、指の位置で描く(A4)。
        if let Some((lane_index, y)) = floating {
            if let Some(lane) = lanes.get(lane_index) {
                let row = VisualRow {
                    kind: VisualRowKind::Lane(lane_index),
                    y,
                    height: self.lane_height(),
                };
                self.draw_item.new_draw_call(cx);
                self.draw_lane(cx, lane, row, false);
                self.draw_lane_clip(cx, lane, row);
            }
        }

        self.draw_grid_and_ruler(cx, self.lane_height());
        self.draw_markers(cx);
        self.draw_playhead_and_drop_target(cx);
        DrawStep::done()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playhead_drag_maps_pointer_motion_to_scrub_actions() {
        let viewport = TimelineViewport::new(100.0, 900.0, 0.0, 1_800.0, 1_800);
        let mut gesture = TimelineGesture::default();

        assert_eq!(
            gesture.begin(&viewport, dvec2(550.0, 10.0), PointerTarget::Ruler),
            Some(TimelineInputAction::Scrub(900))
        );
        assert_eq!(
            gesture.pointer_move(&viewport, dvec2(775.0, 10.0)),
            Some(TimelineInputAction::Scrub(1_350))
        );
    }

    #[test]
    fn lane_drag_commits_one_restack_action_on_pointer_up() {
        let viewport = TimelineViewport::new(150.0, 900.0, 0.0, 1_800.0, 1_800);
        let mut gesture = TimelineGesture::default();

        assert_eq!(
            gesture.begin(
                &viewport,
                dvec2(40.0, 40.0),
                PointerTarget::Rail {
                    layer_id: 7,
                    from_front: 2,
                    grab_offset: 4.0,
                    locked: false,
                }
            ),
            None
        );
        assert_eq!(gesture.move_lane_target(9), None);
        assert_eq!(
            gesture.pointer_up(),
            Some(TimelineInputAction::Restack {
                layer_id: 7,
                target_from_front: 9,
            })
        );
    }

    #[test]
    fn lane_toggle_click_hits_m_and_misses_outside_all_three() {
        let row = VisualRow {
            kind: VisualRowKind::Lane(0),
            y: 100.0,
            height: 20.0,
        };
        let rects = TimelineSurface::lane_toggle_rects(150.0, 0.0, row);

        let inside_m = dvec2(rects[0].pos.x + 1.0, rects[0].pos.y + 1.0);
        assert_eq!(
            TimelineSurface::flag_at_point(&rects, inside_m),
            Some(LaneFlag::Hidden)
        );

        // rail_width=150・origin_x=0 なので M/S/L は x=105..150 に収まる。x=10
        // はどのトグルより手前で、3つとも外している。
        let outside_all = dvec2(10.0, row.y + 1.0);
        assert_eq!(TimelineSurface::flag_at_point(&rects, outside_all), None);
    }

    #[test]
    fn wheel_zoom_is_horizontal_only_and_recomputes_tick_density() {
        let viewport = TimelineViewport::new(150.0, 900.0, 0.0, 1_800.0, 1_800);
        let before = viewport.tick_steps(30, 1, 18.0, 40.0);
        let mut zoomed = viewport;
        for _ in 0..3 {
            zoomed = zoomed.zoom_at(450.0, 240.0, 0.0).expect("wheel zoom");
        }
        let after = zoomed.tick_steps(30, 1, 18.0, 40.0);

        assert!(zoomed.visible_frames < viewport.visible_frames);
        assert_eq!(zoomed.vertical_scale, 1.0);
        assert_ne!(
            after, before,
            "time ruler ticks must follow horizontal zoom"
        );
    }

    #[test]
    fn unmodified_trackpad_scroll_pans_and_never_becomes_zoom() {
        let mut gesture = TimelineScrollGesture::default();
        let modifiers = KeyModifiers::default();

        assert_eq!(
            gesture.update(dvec2(12.0, 2.0), ScrollPhase::Began, modifiers),
            Some(TimelineScrollAction::PanPixels(12.0))
        );
        assert_eq!(
            gesture.update(dvec2(3.0, 20.0), ScrollPhase::Changed, modifiers),
            Some(TimelineScrollAction::PanPixels(3.0)),
            "dominant axis is fixed for the whole gesture"
        );
    }

    #[test]
    fn option_scroll_zooms_and_keeps_the_gesture_verb_fixed() {
        let mut gesture = TimelineScrollGesture::default();
        let mut option = KeyModifiers::default();
        option.alt = true;

        assert_eq!(
            gesture.update(dvec2(0.0, 8.0), ScrollPhase::Began, option),
            Some(TimelineScrollAction::Zoom {
                delta: 8.0,
                precise: true,
            })
        );
        assert_eq!(
            gesture.update(
                dvec2(0.0, 4.0),
                ScrollPhase::Changed,
                KeyModifiers::default()
            ),
            Some(TimelineScrollAction::Zoom {
                delta: 4.0,
                precise: true,
            }),
            "modifier changes cannot reinterpret a live gesture"
        );
    }

    #[test]
    fn momentum_follows_the_owner_until_touch_catches_it() {
        let mut gesture = TimelineScrollGesture::default();
        let modifiers = KeyModifiers::default();

        assert_eq!(
            gesture.update(dvec2(6.0, 0.0), ScrollPhase::Changed, modifiers),
            Some(TimelineScrollAction::PanPixels(6.0))
        );
        assert_eq!(
            gesture.update(dvec2(4.0, 0.0), ScrollPhase::Momentum, modifiers),
            Some(TimelineScrollAction::PanPixels(4.0))
        );
        assert_eq!(
            gesture.update(dvec2(0.0, 0.0), ScrollPhase::Touched, modifiers),
            None
        );
        assert_eq!(
            gesture.update(dvec2(2.0, 0.0), ScrollPhase::Momentum, modifiers),
            None
        );
    }

    #[test]
    fn unapplied_edge_delta_does_not_claim_momentum() {
        let mut gesture = TimelineScrollGesture::default();
        let modifiers = KeyModifiers::default();
        assert_eq!(
            gesture.update(dvec2(6.0, 0.0), ScrollPhase::Changed, modifiers),
            Some(TimelineScrollAction::PanPixels(6.0))
        );
        gesture.reject_owner_if_unapplied(GesturePhase::Update, false);
        assert_eq!(
            gesture.update(dvec2(4.0, 0.0), ScrollPhase::Momentum, modifiers),
            None
        );
    }

    #[test]
    fn vertical_wheel_scrolls_lanes_and_only_pans_time_with_shift() {
        let mut gesture = TimelineScrollGesture::default();
        assert_eq!(
            gesture.update(dvec2(0.0, 12.0), ScrollPhase::None, KeyModifiers::default()),
            Some(TimelineScrollAction::ScrollLanes(12.0))
        );

        let mut shift = KeyModifiers::default();
        shift.shift = true;
        assert_eq!(
            gesture.update(dvec2(0.0, 12.0), ScrollPhase::None, shift),
            Some(TimelineScrollAction::PanPixels(12.0))
        );
    }

    #[test]
    fn wheel_zoom_is_normalized_but_precise_trackpad_delta_is_preserved() {
        assert_eq!(TimelineSurface::normalized_zoom_delta(95.0, false), 120.0);
        assert_eq!(TimelineSurface::normalized_zoom_delta(3.25, true), 3.25);
    }

    #[test]
    fn framework_neutral_scale_sample_maps_to_time_zoom() {
        let mut gesture = TimelineScrollGesture::default();
        let action = gesture.update_sample(GestureSample {
            phase: GesturePhase::Begin,
            device: GestureDevice::Trackpad,
            centroid: [450.0, 200.0],
            translation: [0.0, 0.0],
            scale_ratio: 1.25,
            rotation_radians: 0.0,
            modifiers: Default::default(),
        });
        assert!(matches!(
            action,
            Some(TimelineScrollAction::Zoom { precise: true, .. })
        ));
    }

    #[test]
    fn native_scale_changes_only_time_viewport_and_keeps_centroid_anchor() {
        let viewport = TimelineViewport::new(150.0, 900.0, 300.0, 1_200.0, 2_400);
        let centroid_x = 600.0;
        let anchor_before = viewport.view_start
            + ((centroid_x - viewport.rail_width) / viewport.time_width) * viewport.visible_frames;
        let mut gesture = TimelineScrollGesture::default();
        let action = gesture
            .update_sample(GestureSample {
                phase: GesturePhase::Begin,
                device: GestureDevice::Trackpad,
                centroid: [centroid_x, 200.0],
                translation: [7.0, 3.0],
                scale_ratio: 1.25,
                rotation_radians: 0.2,
                modifiers: Default::default(),
            })
            .expect("native scale must map to timeline zoom");
        let TimelineScrollAction::Zoom { delta, .. } = action else {
            panic!("scale must win over simultaneous translation");
        };
        let zoomed = viewport
            .zoom_at(centroid_x, delta, 0.0)
            .expect("scale must change the visible span");
        let anchor_after = zoomed.view_start
            + ((centroid_x - zoomed.rail_width) / zoomed.time_width) * zoomed.visible_frames;

        assert!((anchor_after - anchor_before).abs() < 0.001);
        assert!(zoomed.visible_frames < viewport.visible_frames);
        assert_eq!(zoomed.vertical_scale, 1.0);
    }

    #[test]
    fn rotation_only_gesture_is_not_a_timeline_operation() {
        let mut gesture = TimelineScrollGesture::default();
        assert_eq!(
            gesture.update_sample(GestureSample {
                phase: GesturePhase::Begin,
                device: GestureDevice::Trackpad,
                centroid: [450.0, 200.0],
                translation: [0.0, 0.0],
                scale_ratio: 1.0,
                rotation_radians: 0.4,
                modifiers: Default::default(),
            }),
            None
        );
    }
}
