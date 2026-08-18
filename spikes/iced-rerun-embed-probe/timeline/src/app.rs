//! アプリの状態と `update()`。**モデルを書き換えるコードはこのファイルにしかない**。
//!
//! これは我慢して守った制約であって、iced が自動で与えてくれる物ではない。
//! ただし iced は破りにくくしてはいる: `canvas::Program::draw` は `&self` で
//! モデルを借りるので、描画中にモデルを書く道が**型として無い**。
//! egui 側は `&mut` を描画関数に通すので、同じ場所に書ける。

use std::sync::atomic::{AtomicU64, Ordering};

use iced::keyboard::Modifiers;
use iced::time::Instant;

use crate::message::TimelineMsg;
use crate::model::{
    hit_test, Clip, ClipId, Grab, Hit, Selection, Timeline, Viewport, PX_PER_FRAME_MAX,
};

/// 進行中のジェスチャ。**確定前の状態だけ**を持つ。
///
/// 置き場所の話が spike の観測点そのものなので、選択と理由を書いておく:
/// iced の canvas は `Program::State` という widget ローカルな可変状態を用意していて、
/// ここに置くこともできた。置かなかったのは、そうすると
/// 「進行中のジェスチャ」だけが message を通らない裏道になり、
/// 比較したかった「全部 Message にしたらどうなるか」が測れなくなるから。
/// 代償は下の `Drag::Move` が持つ `origins` で、これはモデルの複製である。
#[derive(Debug, Clone)]
pub enum Drag {
    /// 選択中の clip をまとめて移動。
    /// `origins` は押した瞬間の位置。差分をここから足すので、途中でスナップしても
    /// 丸め誤差が累積しない(相対移動を積むと必ず流れる)。
    Move {
        anchor_frame: f64,
        origins: Vec<(ClipId, i64)>,
    },
    /// 端の trim。1本だけ。
    Trim {
        id: ClipId,
        edge: Grab,
        anchor_frame: f64,
        origin_start: i64,
        origin_len: i64,
    },
    /// playhead のスクラブ。
    Scrub,
}

pub struct App {
    pub timeline: Timeline,
    pub viewport: Viewport,
    pub selection: Selection,
    pub playhead: i64,

    /// 修飾キーの現在値。ホイールイベントが運んでくれないので自前で保持する。
    pub modifiers: Modifiers,

    /// 進行中ジェスチャ。`None` なら何も掴んでいない。
    pub drag: Option<Drag>,

    // ── 計測 ───────────────────────────────────────────────────────────
    /// `draw()` の中で幾何を組み立てるのに掛かった時間(µs)。
    /// `draw` は `&self` なので、書き戻す口はこの atomic しかない。
    pub draw_us: AtomicU64,
    /// 直近フレームで実際に描いた clip 数。
    pub drawn_clips: AtomicU64,
    /// フレーム間隔の指数移動平均(ms)。
    pub frame_ms: f64,
    last_frame: Option<Instant>,
}

impl App {
    pub fn new(clip_count: usize) -> Self {
        let timeline = Timeline::demo(clip_count);
        // 24本のときは既定 zoom、たくさんあるときは全部入る zoom で始める。
        let viewport = if clip_count > 64 {
            Viewport::fit(timeline.extent(), crate::WINDOW_W)
        } else {
            Viewport::default()
        };
        Self {
            timeline,
            viewport,
            selection: Selection::new(),
            // 0 だと playhead が左端に張り付いて見えないので、最初から少し中に置く。
            playhead: 90,
            modifiers: Modifiers::empty(),
            drag: None,
            draw_us: AtomicU64::new(0),
            drawn_clips: AtomicU64::new(0),
            frame_ms: 0.0,
            last_frame: None,
        }
    }

    pub fn title(&self) -> String {
        format!(
            "iced timeline probe — {} clips / draw {:.2} ms / frame {:.2} ms",
            self.timeline.clips.len(),
            self.draw_us.load(Ordering::Relaxed) as f64 / 1000.0,
            self.frame_ms,
        )
    }

    /// **モデルの唯一の書き込み口**。
    ///
    /// 腕がジェスチャの種類ごとに1本ずつ立っていて、腕の中に「今どのモードか」の
    /// 再分岐が無いことに注目してほしい。それは message が届く前に
    /// `Program::update` の hit test で解決済みだから。
    pub fn update(&mut self, message: TimelineMsg) {
        match message {
            TimelineMsg::ClipGrabbed {
                id,
                grab,
                at_frame,
                additive,
            } => {
                self.apply_selection(id, additive);
                // Cmd+クリックが**選択を外した**場合、掴んだ物はもう選択に居ない。
                // ここでドラッグを始めると「外したはずの clip を掴んだつもりで、
                // 残りの選択が動く」という嘘になるので、始めない。
                if !self.selection.contains(&id) {
                    self.drag = None;
                    return;
                }
                self.drag = Some(match grab {
                    Grab::Body => Drag::Move {
                        anchor_frame: at_frame,
                        origins: self
                            .timeline
                            .clips
                            .iter()
                            .filter(|c| self.selection.contains(&c.id))
                            .map(|c| (c.id, c.start))
                            .collect(),
                    },
                    Grab::LeftEdge | Grab::RightEdge => {
                        let clip = self.timeline.get(id).copied().unwrap_or(Clip {
                            id,
                            track: 0,
                            start: 0,
                            len: 1,
                        });
                        Drag::Trim {
                            id,
                            edge: grab,
                            anchor_frame: at_frame,
                            origin_start: clip.start,
                            origin_len: clip.len,
                        }
                    }
                });
            }

            TimelineMsg::EmptyPressed { additive } => {
                if !additive {
                    self.selection.clear();
                }
            }

            TimelineMsg::ScrubStarted { frame } => {
                self.playhead = self.viewport.snap(frame).max(0);
                self.drag = Some(Drag::Scrub);
            }

            TimelineMsg::PointerMoved { frame } => match self.drag.clone() {
                Some(Drag::Move {
                    anchor_frame,
                    origins,
                }) => {
                    // 掴んだ点からの差分をフレームグリッドへ落とす。
                    let delta = self.viewport.snap(frame - anchor_frame);
                    // どれか1本でも負に突き抜けるなら、集団ごと押し戻す。
                    // 選択の相対配置が崩れないのが AE / Premiere の挙動。
                    let floor = origins
                        .iter()
                        .map(|(_, start)| start + delta)
                        .min()
                        .unwrap_or(0);
                    let delta = if floor < 0 { delta - floor } else { delta };
                    for (id, origin) in &origins {
                        if let Some(clip) = self.timeline.get_mut(*id) {
                            clip.start = origin + delta;
                        }
                    }
                }
                Some(Drag::Trim {
                    id,
                    edge,
                    anchor_frame,
                    origin_start,
                    origin_len,
                }) => {
                    let delta = self.viewport.snap(frame - anchor_frame);
                    if let Some(clip) = self.timeline.get_mut(id) {
                        match edge {
                            Grab::LeftEdge => {
                                // 頭を引くと start と len が逆向きに動く。
                                // 1フレーム未満に潰さない・負に出さない、の2つで挟む。
                                let delta = delta.clamp(-origin_start, origin_len - 1);
                                clip.start = origin_start + delta;
                                clip.len = origin_len - delta;
                            }
                            Grab::RightEdge => {
                                clip.len = (origin_len + delta).max(1);
                            }
                            Grab::Body => {}
                        }
                    }
                }
                Some(Drag::Scrub) => {
                    self.playhead = self.viewport.snap(frame).max(0);
                }
                None => {}
            },

            TimelineMsg::PointerReleased => {
                self.drag = None;
            }

            TimelineMsg::ZoomedAt { anchor_x, notches } => {
                // 1ノッチ = 1.2倍。AE の zoom 段と同じくらいの粗さ。
                self.viewport.zoom_at(anchor_x, 1.2f32.powf(notches));
            }

            TimelineMsg::PannedX { notches } => {
                // 「1ノッチ = 見えている幅の 1/12」ではなく px 固定にする。
                // zoom に依らず指の動きと画面の動きが一致する方が迷子になりにくい。
                let px = notches * 48.0;
                self.viewport.scroll_x =
                    (self.viewport.scroll_x - px as f64 / self.viewport.px_per_frame as f64)
                        .max(0.0);
            }

            TimelineMsg::ScrolledY { notches } => {
                let max = (self.timeline.tracks as f32 * (crate::model::ROW_H + crate::model::ROW_GAP)
                    - crate::model::ROW_H)
                    .max(0.0);
                self.viewport.scroll_y = (self.viewport.scroll_y - notches * 32.0).clamp(0.0, max);
            }

            TimelineMsg::Nudged { frames } => {
                if self.selection.is_empty() {
                    // 選択が無いときは playhead を動かす。製品の F-04 と同じ振り分け。
                    self.playhead = (self.playhead + frames).max(0);
                } else {
                    let floor = self
                        .timeline
                        .clips
                        .iter()
                        .filter(|c| self.selection.contains(&c.id))
                        .map(|c| c.start + frames)
                        .min()
                        .unwrap_or(0);
                    let frames = if floor < 0 { frames - floor } else { frames };
                    for clip in &mut self.timeline.clips {
                        if self.selection.contains(&clip.id) {
                            clip.start += frames;
                        }
                    }
                }
            }

            TimelineMsg::ModifiersChanged(modifiers) => {
                self.modifiers = modifiers;
            }

            TimelineMsg::Rendered(now) => {
                if let Some(prev) = self.last_frame {
                    let dt = (now - prev).as_secs_f64() * 1000.0;
                    // 起動直後の跳ねを均す指数移動平均。
                    self.frame_ms = if self.frame_ms == 0.0 {
                        dt
                    } else {
                        self.frame_ms * 0.9 + dt * 0.1
                    };
                }
                self.last_frame = Some(now);
            }
        }
    }

    /// クリックの選択規則。Cmd 併用でトグル、素のクリックで単独選択。
    /// 既に選択済みの clip を素で押したときに選択を潰さないのは、
    /// 複数選択したまま引きずるため(これを忘れると複数移動が事実上使えない)。
    fn apply_selection(&mut self, id: ClipId, additive: bool) {
        if additive {
            if !self.selection.insert(id) {
                self.selection.remove(&id);
            }
        } else if !self.selection.contains(&id) {
            self.selection.clear();
            self.selection.insert(id);
        }
    }

    /// zoom 上限に張り付いているか(隅の表示に使うだけ)。
    pub fn at_zoom_ceiling(&self) -> bool {
        self.viewport.px_per_frame >= PX_PER_FRAME_MAX - f32::EPSILON
    }
}

/// `Program::update` から呼ぶ、hit test の薄い口。
/// `mouse_interaction` と同じ関数を通すことでカーソルとジェスチャがズレない。
pub fn hit_at(app: &App, x: f32, y: f32) -> Hit {
    hit_test(&app.timeline, &app.viewport, x, y)
}
