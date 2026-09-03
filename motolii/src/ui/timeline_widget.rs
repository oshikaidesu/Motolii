use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use crate::ui::playback::Clock;
use crate::ui::session::Selection;
use crate::ui::session::GestureSurface;
use crate::ui::tokens::{self, UiScale};
use anyrender::{PaintRef, PaintScene};
use dioxus_native::prelude::{Signal, WritableExt};
use crate::doc::store::{
    Document, Fps, Intent, KeyframeTrack, LayerAttrs, LayerAttrsPatch, LayerId, LayerMeta,
    LayerTiming, RationalTime, StoreError,
};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::{BlitzWheelDelta, UiEvent};
use keyboard_types::Modifiers;
use peniko::kurbo::{Affine, Point, Rect, Size};
use peniko::{Color, Fill};

const PX_PER_SEC: f64 = 60.0;
/// 再生位置が視界から出たら追いかける。**止まっている間は追いかけない** ——
/// 利用者が自分で右へ動かしたのを、その場で引き戻してしまう。
/// 止まっていても**跳んだ**時(印へ・Home/End)は見せに行く —— 跳んだ先が視界の外では、
/// 押した事が起きていないのと同じに見える。
fn follow_playhead(scroll: f64, visible: f64, playhead: f64, playing: bool, jumped: bool) -> Option<f64> {
    if !(playing || jumped) || visible <= 0.0 {
        return None;
    }
    let right = scroll + visible;
    if playhead < scroll || playhead > right - visible * 0.1 {
        return Some((playhead - visible * 0.1).max(0.0));
    }
    None
}

const MIN_PPS: f64 = 8.0;
const MAX_PPS: f64 = 600.0;
const RULER_H: f64 = crate::ui::tokens::ROW;
const ROW_H: f64 = crate::ui::tokens::ROW;
const PLAYHEAD_SEC: f64 = 4.6;

fn c(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

#[derive(Clone)]
pub(super) struct CanvasRow {
    pub is_group: bool,
    pub keys: Vec<f64>,
    pub span: Option<(f64, f64)>,
    pub agg: Vec<f64>,
    pub layer: Option<LayerId>,
    pub prop: Option<crate::doc::store::PropertyId>,
    pub color: [u8; 3],
}

#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    Move,
    TrimStart,
    TrimEnd,
    /// Alt+drag: 帯は動かさず中身だけずらす(Premiere・Resolve のスリップ)。
    Slip,
    /// 菱形そのものを掴んで時間を動かす。
    Key { at_sec: f64 },
}

struct DragState {
    row: usize,
    layer: LayerId,
    prop: Option<crate::doc::store::PropertyId>,
    orig: LayerTiming,
    grab_sec: f64,
    delta_sec: f64,
    mode: DragMode,
    /// 閾値を超えて、本当に掴んだ事になったか。
    moved: bool,
}

const EDGE_GRAB_PX: f64 = 6.0;
/// タップがドラッグになるまでに指が動ける長さ。
const DRAG_SLOP_PX: f64 = 3.0;
/// 掴んだ物が吸い付く距離。手が止まらない感触はここで決まる。
const SNAP_PX: f64 = 8.0;

pub(super) fn document_fps(doc: &Document) -> Result<Fps, StoreError> {
    doc.view()
        .composition()?
        .map(|composition| composition.fps)
        .ok_or_else(|| StoreError::Property("Composition has no frame rate".to_owned()))
}

/// 目盛の段。1 段は約 80px 以上空く物を 1・2・5・10 … 秒(と 1/fps 未満は frame)から選ぶ。
fn nice_step(pps: f64) -> f64 {
    let min_sec = 80.0 / pps.max(1e-6);
    let steps = [0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0, 600.0];
    steps.into_iter().find(|s| *s >= min_sec).unwrap_or(600.0)
}

fn keyframe_shift_intents(
    doc: &Document,
    layer: LayerId,
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view();
    let fps = document_fps(doc)?;
    let shift =
        RationalTime::try_from_frame(delta_frames, fps).map_err(|e| StoreError::Property(e.to_string()))?;
    let mut intents = Vec::new();
    for property in view.properties(layer) {
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut shifted = KeyframeTrack::new();
        for key in track.keys() {
            let mut key = key.clone();
            key.t = key.t.try_add(shift).map_err(|e| StoreError::Property(e.to_string()))?;
            shifted.insert(key);
        }
        intents.push(Intent::SetTrack { layer, property, track: shifted });
    }
    // 文字の切替(ContentTrack)は property でなく data。層と一緒に動かさないと歌詞の時刻だけ置き去りになる。
    if let Some(mut text) = view.text_document(layer)? {
        if text.content.keys().len() > 1 {
            let mut moved = crate::doc::store::ContentTrack::new();
            for key in text.content.keys() {
                let t = key.t.try_add(shift).map_err(|e| StoreError::Property(e.to_string()))?;
                moved.insert(crate::doc::store::ContentKeyframe { t, content: key.content.clone() });
            }
            text.content = moved;
            intents.push(Intent::SetTextDocument { layer, document: text });
        }
    }
    Ok(intents)
}

/// `at_frame` にあるキーだけを `delta_frames` ずらす。層の行に見えている菱形は
/// 複数のトラックの同じ時刻を束ねているので、束ごと動かす(AE と同じ)。
pub(super) fn keyframe_move_intents(
    doc: &Document,
    layer: LayerId,
    only: Option<&crate::doc::store::PropertyId>,
    frames: &[i64],
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view();
    let fps = document_fps(doc)?;
    let shift =
        RationalTime::try_from_frame(delta_frames, fps).map_err(|e| StoreError::Property(e.to_string()))?;
    let mut intents = Vec::new();
    for property in view.properties(layer) {
        if only.is_some_and(|p| *p != property) {
            continue;
        }
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut touched = false;
        let mut moved = KeyframeTrack::new();
        for key in track.keys() {
            let mut key = key.clone();
            let frame = key
                .t
                .try_to_frame_round(fps)
                .map_err(|e| StoreError::Property(e.to_string()))?;
            if frames.contains(&frame) {
                key.t = key.t.try_add(shift).map_err(|e| StoreError::Property(e.to_string()))?;
                touched = true;
            }
            moved.insert(key);
        }
        if touched {
            intents.push(Intent::SetTrack { layer, property, track: moved });
        }
    }
    Ok(intents)
}

/// その時刻のキーだけ外す。最後の 1 つを外した時は、その値を定数として残す
/// —— 絵が飛ばない(AE の ◆ を消した時と同じ)。
pub(super) fn keyframe_delete_intents(
    doc: &Document,
    layer: LayerId,
    only: Option<&crate::doc::store::PropertyId>,
    at_sec: f64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view();
    let fps = document_fps(doc)?;
    let at_frame = (at_sec * fps.as_f64()).round() as i64;
    let mut intents = Vec::new();
    for property in view.properties(layer) {
        if only.is_some_and(|p| *p != property) {
            continue;
        }
        let Some(track) = view.track(layer, &property)? else {
            continue;
        };
        let mut kept = KeyframeTrack::new();
        let mut removed: Option<RationalTime> = None;
        for key in track.keys() {
            let frame = key
                .t
                .try_to_frame_round(fps)
                .map_err(|e| StoreError::Property(e.to_string()))?;
            if frame == at_frame {
                removed = Some(key.t);
            } else {
                kept.insert(key.clone());
            }
        }
        let Some(at) = removed else { continue };
        if kept.keys().is_empty() {
            intents.push(Intent::SetConstant { layer, property, value: track.eval(at) });
        } else {
            intents.push(Intent::SetTrack { layer, property, track: kept });
        }
    }
    Ok(intents)
}

/// 層の端を `frame` へ。`tail` で頭/尻、`trim` で「切り落とす」か「丸ごと動かす」か。
/// 動かす場合は素材の中身がずれないよう source_in も一緒に動く。
pub(super) fn edge_to_frame(
    orig: LayerTiming,
    frame: i64,
    tail: bool,
    trim: bool,
) -> LayerTiming {
    match (tail, trim) {
        // 尻を切る: 現在時刻までの長さにする
        (true, true) => LayerTiming {
            duration: (frame - orig.start).max(1),
            ..orig
        },
        // 頭を切る: 中身を保ったまま入り口を移す
        (false, true) => {
            let delta = (frame - orig.start).clamp(
                -(orig.start.min(orig.source_in)),
                orig.duration - 1,
            );
            LayerTiming {
                start: orig.start + delta,
                duration: orig.duration - delta,
                source_in: orig.source_in + delta,
                ..orig
            }
        }
        // 尻を現在時刻へ: 長さを保ったまま丸ごと動かす
        (true, false) => LayerTiming {
            start: (frame - orig.duration).max(0),
            ..orig
        },
        // 頭を現在時刻へ: 同上
        (false, false) => LayerTiming {
            start: frame.max(0),
            ..orig
        },
    }
}

pub(super) enum TimelineMsg {
    SetRows(Vec<CanvasRow>),
    ScrollBy(f64),
    SetMarkers(Vec<f64>),
    /// Escape: キーの選択を落とす。
    DeselectKeys,
}

pub(super) struct TimelineWidget {
    rx: Rc<Receiver<TimelineMsg>>,
    rows: Vec<CanvasRow>,
    markers: Vec<f64>,
    /// 目盛の帯で印を掴んでいる: (index, 元の秒, 掴んだ秒, 今の差分)。
    marker_drag: Option<(usize, f64, f64, f64)>,
    fps: f64,
    /// 前の描画の再生位置。跳んだかどうかはこれと比べる。
    last_playhead: f64,
    pps: f64,
    scroll_sec: f64,
    scroll_y: f64,
    viewport_h: f64,
    viewport_w: f64,
    cursor: Option<(f64, f64)>,
    hovered: Option<(usize, usize)>,
    selected: Vec<(usize, usize)>,
    /// 空きからのドラッグで囲む。掴み始めと今の場所(秒, 行)。
    marquee: Option<((f64, f64), (f64, f64))>,
    /// 囲いが ⌘ / ⇧ 付きで始まった(足す。置き換えない)。
    marquee_add: bool,
    clock: Option<Arc<Clock>>,
    scale: Option<Arc<UiScale>>,
    doc: Option<Arc<Mutex<Document>>>,
    extractor: Option<fn(&Document) -> Vec<CanvasRow>>,
    drag: Option<DragState>,
    scrubbing: bool,
    selection: Option<Selection>,
    selected_mirror: Option<Signal<Option<LayerId>>>,
    scroll_y_mirror: Option<Signal<f64>>,
    /// 再生位置の鏡。`clock` は signal ではないので、これが無いと
    /// 位置を動かしても値の欄が描き直されない。
    playhead_mirror: Option<Signal<f64>>,
    /// 誰かが Document を書いたら上がる。**書いた者が知らせる**形だと
    /// 書く場所の数だけ配線が要るので、こちらから見に行く。
    revision: Option<Signal<u32>>,
    seen_revision: u32,
    gesture: Option<GestureSurface>,
    seen_cancel: u32,
    selected_key: Option<Arc<Mutex<Vec<crate::ui::session::KeySel>>>>,
}

impl TimelineWidget {
    pub(super) fn new(rows: Vec<CanvasRow>, rx: Rc<Receiver<TimelineMsg>>) -> Self {
        Self {
            rx,
            rows,
            markers: Vec::new(),
            marker_drag: None,
            fps: 30.0,
            last_playhead: 0.0,
            pps: PX_PER_SEC,
            scroll_sec: 0.0,
            scroll_y: 0.0,
            viewport_h: 0.0,
            viewport_w: 0.0,
            cursor: None,
            hovered: None,
            selected: Vec::new(),
            marquee: None,
            marquee_add: false,
            clock: None,
            scale: None,
            doc: None,
            extractor: None,
            drag: None,
            scrubbing: false,
            selection: None,
            selected_mirror: None,
            scroll_y_mirror: None,
            playhead_mirror: None,
            revision: None,
            seen_revision: 0,
            gesture: None,
            seen_cancel: 0,
            selected_key: None,
        }
    }

    pub(super) fn with_selection(mut self, selection: Selection, mirror: Signal<Option<LayerId>>) -> Self {
        self.selection = Some(selection);
        self.selected_mirror = Some(mirror);
        self
    }

    pub(super) fn with_key_mirror(
        mut self,
        slot: Arc<Mutex<Vec<crate::ui::session::KeySel>>>,
    ) -> Self {
        self.selected_key = Some(slot);
        self
    }

    pub(super) fn with_scroll_mirror(mut self, mirror: Signal<f64>) -> Self {
        self.scroll_y_mirror = Some(mirror);
        self
    }

    pub(super) fn with_playhead_mirror(mut self, mirror: Signal<f64>) -> Self {
        self.playhead_mirror = Some(mirror);
        self
    }

    pub(super) fn with_gesture(mut self, gesture: GestureSurface) -> Self {
        self.gesture = Some(gesture);
        self
    }

    pub(super) fn with_revision(mut self, revision: Signal<u32>) -> Self {
        self.revision = Some(revision);
        self
    }

    pub(super) fn with_clock(mut self, clock: Arc<Clock>) -> Self {
        self.clock = Some(clock);
        self.pps = 20.0;
        self
    }

    pub(super) fn with_scale(mut self, scale: Arc<UiScale>) -> Self {
        self.scale = Some(scale);
        self
    }

    fn sfac(&self) -> f64 {
        self.scale.as_ref().map(|s| s.factor()).unwrap_or(1.0)
    }

    pub(super) fn with_document(
        mut self,
        doc: Arc<Mutex<Document>>,
        extractor: fn(&Document) -> Vec<CanvasRow>,
    ) -> Self {
        self.fps = doc
            .lock()
            .ok()
            .and_then(|doc| document_fps(&doc).ok())
            .map(|fps| fps.as_f64())
            .unwrap_or(30.0);
        self.doc = Some(doc);
        self.extractor = Some(extractor);
        self
    }

    fn max_scroll_y(&self) -> f64 {
        let rowh = ROW_H * self.sfac();
        let content_h = self.rows.len() as f64 * rowh;
        let avail_h = (self.viewport_h - RULER_H * self.sfac()).max(0.0);
        (content_h - avail_h).max(0.0)
    }

    /// 掴みを終える。**離した事が届かなかった時もここを通す** ——
    /// 捨てると、離した所までの編集が失われる(外の規格の pointer capture が
    /// 本来これを保証している物の、届く範囲での代わり)。
    fn finish_drag(&mut self) {
        if let Some(gesture) = &self.gesture {
            gesture.end();
        }
                self.scrubbing = false;
                if let Some((i, orig, _, delta)) = self.marker_drag.take() {
                    if delta != 0.0 {
                        self.move_marker(i, orig + delta);
                    }
                }
                if let Some((from, to)) = self.marquee.take() {
                    self.select_inside(from, to);
                }
                if let Some(drag) = self.drag.take() {
                    let (Some(doc), Some(extractor)) = (self.doc.as_ref(), self.extractor) else {
                        return;
                    };
                    let mut doc = doc.lock().unwrap();
                    let Ok(fps) = document_fps(&doc) else {
                        return;
                    };
                    let fps_value = fps.as_f64();
                    self.fps = fps_value;
                    let raw_delta = (drag.delta_sec * fps_value).round() as i64;
                    if let DragMode::Key { at_sec } = drag.mode {
                        let at_frame = (at_sec * fps_value).round() as i64;
                        if raw_delta != 0 {
                            // 掴んだ物だけでなく、選んでいるキーを全部同じだけ動かす。
                            let mut moving: Vec<(LayerId, Option<crate::doc::store::PropertyId>, i64)> = self
                                .selected
                                .iter()
                                .filter_map(|(row_ix, key_ix)| {
                                    let row = self.rows.get(*row_ix)?;
                                    let t = row.keys.get(*key_ix).copied()?;
                                    Some((row.layer?, row.prop.clone(), (t * fps_value).round() as i64))
                                })
                                .collect();
                            if !moving.iter().any(|(l, p, f)| {
                                *l == drag.layer && *p == drag.prop && *f == at_frame
                            }) {
                                moving.push((drag.layer, drag.prop.clone(), at_frame));
                            }
                            let mut all = Vec::new();
                            let mut failed = None;
                            // 同じ帯の複数キーは 1 本の SetTrack に束ねる —— 別々に出すと最後の 1 本が勝つ。
                            let mut tracks: Vec<((LayerId, Option<crate::doc::store::PropertyId>), Vec<i64>)> = Vec::new();
                            for (layer, prop, frame) in moving {
                                match tracks.iter_mut().find(|(k, _)| *k == (layer, prop.clone())) {
                                    Some((_, frames)) => frames.push(frame),
                                    None => tracks.push(((layer, prop), vec![frame])),
                                }
                            }
                            for ((layer, prop), frames) in tracks {
                                match keyframe_move_intents(&doc, layer, prop.as_ref(), &frames, raw_delta) {
                                    Ok(intents) => all.extend(intents),
                                    Err(e) => failed = Some(e),
                                }
                            }
                            let landed: Vec<(usize, f64)> = self
                                .selected
                                .iter()
                                .filter_map(|(row_ix, key_ix)| {
                                    let t = self.rows.get(*row_ix)?.keys.get(*key_ix).copied()?;
                                    Some((*row_ix, t + raw_delta as f64 / fps_value))
                                })
                                .collect();
                            match failed.map_or(Ok(all), Err) {
                                Ok(intents) => match doc.apply_all(intents) {
                                    Ok(_) => {
                                        println!(
                                            "PROBE room=write verdict=applied MoveKey layer={:?} frame {}->{}",
                                            drag.layer, at_frame, at_frame + raw_delta
                                        );
                                        self.rows = extractor(&doc);
                                        // 選択は動かした先へ付いていく。時刻で照合し直さないと、
                                        // 直後の F9 が動かす前の秒を見て空振りする。
                                        let half = 0.5 / fps_value;
                                        self.selected = landed
                                            .iter()
                                            .filter_map(|(row_ix, t)| {
                                                let row = self.rows.get(*row_ix)?;
                                                let ki = row.keys.iter().position(|k| (k - t).abs() < half)?;
                                                Some((*row_ix, ki))
                                            })
                                            .collect();
                                        self.publish_keys();
                                    }
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                },
                                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                            }
                        }
                        return;
                    }
                    let timing = match drag.mode {
                        DragMode::Move => {
                            let new_start = (drag.orig.start + raw_delta).max(0);
                            LayerTiming { start: new_start, ..drag.orig }
                        }
                        DragMode::Slip => {
                            // 右へ引くと中身が右へ来る = 頭に映る素材が前へ戻る。
                            let source_in = (drag.orig.source_in - raw_delta).max(0);
                            LayerTiming { source_in, ..drag.orig }
                        }
                        DragMode::TrimStart => {
                            let min_delta = -(drag.orig.start.min(drag.orig.source_in));
                            let max_delta = drag.orig.duration - 1;
                            let delta = raw_delta.clamp(min_delta, max_delta);
                            LayerTiming {
                                start: drag.orig.start + delta,
                                duration: drag.orig.duration - delta,
                                source_in: drag.orig.source_in + delta,
                                ..drag.orig
                            }
                        }
                        DragMode::TrimEnd => {
                            let min_delta = -(drag.orig.duration - 1);
                            let delta = raw_delta.max(min_delta);
                            LayerTiming { duration: drag.orig.duration + delta, ..drag.orig }
                        }
                        DragMode::Key { .. } => unreachable!("上で返している"),
                    };
                    let mut intents = vec![Intent::SetTiming { layer: drag.layer, timing }];
                    if drag.mode == DragMode::Move {
                        let applied_delta = timing.start - drag.orig.start;
                        if applied_delta != 0 {
                            match keyframe_shift_intents(&doc, drag.layer, applied_delta) {
                                Ok(more) => intents.extend(more),
                                Err(e) => {
                                    println!("PROBE room=write verdict=apply-error {e}");
                                    return;
                                }
                            }
                            // 一緒に選んでいる層も同じだけ運ぶ(曲を 1 拍ずらすのに 60 枚を掴ませない)。
                            let others: Vec<LayerId> = self
                                .selection
                                .as_ref()
                                .map(|s| s.all())
                                .unwrap_or_default()
                                .into_iter()
                                .filter(|l| *l != drag.layer)
                                .filter(|l| !doc.view().attrs(*l).ok().flatten().is_some_and(|a| a.locked))
                                .collect();
                            for other in others {
                                let Some(meta) = doc.view().meta(other).ok().flatten() else { continue };
                                let start = (meta.timing.start + applied_delta).max(0);
                                let shift = start - meta.timing.start;
                                if shift == 0 {
                                    continue;
                                }
                                intents.push(Intent::SetTiming { layer: other, timing: LayerTiming { start, ..meta.timing } });
                                match keyframe_shift_intents(&doc, other, shift) {
                                    Ok(more) => intents.extend(more),
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                }
                            }
                        }
                    }
                    match doc.apply_all(intents) {
                        Ok(_) => {
                            println!(
                                "PROBE room=write verdict=applied SetTiming mode={} start {}->{} dur {}->{}",
                                match drag.mode {
                                    DragMode::Move => "move",
                                    DragMode::Slip => "slip",
                                    DragMode::TrimStart => "trim-start",
                                    DragMode::TrimEnd => "trim-end",
                                    DragMode::Key { .. } => unreachable!("上で返している"),
                                },
                                drag.orig.start, timing.start,
                                drag.orig.duration, timing.duration
                            );
                            self.rows = extractor(&doc);
                        }
                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                    }
                }
    }

    fn set_scroll_y(&mut self, y: f64) {
        self.scroll_y = y.clamp(0.0, self.max_scroll_y());
        if let Some(mirror) = &mut self.scroll_y_mirror {
            mirror.set(self.scroll_y);
        }
    }

    /// 掴んでいる時刻の吸い付き先(再生位置・comp の頭・各層の端)。
    /// 印を新しい時刻へ。Document の印を書き直し、名前は残す(時刻名は打った時の物)。
    fn move_marker(&mut self, i: usize, sec: f64) {
        let Some(doc) = self.doc.as_ref() else { return };
        let mut d = doc.lock().unwrap();
        let Ok(fps) = document_fps(&d) else { return };
        let Ok(mut markers) = d.view().markers() else { return };
        let Some(marker) = markers.get_mut(i) else { return };
        let Ok(time) = RationalTime::try_from_frame((sec * fps.as_f64()).round() as i64, fps) else { return };
        marker.time = time;
        markers.sort_by(|a, b| a.time.as_seconds_f64().total_cmp(&b.time.as_seconds_f64()));
        match d.apply(Intent::SetMarkers { markers: markers.clone() }) {
            Ok(_) => {
                self.markers = markers.iter().map(|m| m.time.as_seconds_f64()).collect();
                if let Some(mut revision) = self.revision {
                    *revision.write() += 1;
                }
                println!("PROBE room=write verdict=applied MoveMarker index={i} to={sec:.3}");
            }
            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
        }
    }

    /// 印を動かす時の吸い先。動かしている印自身は外す。
    fn snapped_time_excluding(&self, t: f64, skip: usize) -> f64 {
        let threshold = SNAP_PX / self.pps;
        let mut targets = vec![0.0];
        if let Some(clock) = &self.clock {
            targets.push(clock.now_sec());
        }
        for row in &self.rows {
            if let Some((a, b)) = row.span {
                targets.push(a);
                targets.push(b);
            }
        }
        targets.extend(self.markers.iter().enumerate().filter(|(i, _)| *i != skip).map(|(_, m)| *m));
        targets
            .into_iter()
            .filter(|target| (target - t).abs() <= threshold)
            .min_by(|a, b| (a - t).abs().total_cmp(&(b - t).abs()))
            .unwrap_or(t)
    }

    /// 印と帯の端へ吸った時刻。再生位置そのものは吸い先に入れない(自分に吸う)。
    fn snapped_time(&self, t: f64) -> f64 {
        let threshold = SNAP_PX / self.pps;
        let mut targets = vec![0.0];
        for row in &self.rows {
            if let Some((a, b)) = row.span {
                targets.push(a);
                targets.push(b);
            }
        }
        targets.extend(self.markers.iter().copied());
        targets
            .into_iter()
            .filter(|target| (target - t).abs() <= threshold)
            .min_by(|a, b| (a - t).abs().total_cmp(&(b - t).abs()))
            .unwrap_or(t)
    }

    fn snap_targets(&self) -> Vec<f64> {
        let mut out = vec![0.0];
        if let Some(clock) = &self.clock {
            out.push(clock.now_sec());
        }
        for row in &self.rows {
            if let Some((a, b)) = row.span {
                out.push(a);
                out.push(b);
            }
        }
        // 印(マーカー)にも吸い付く。歌詞を拍へ寄せる中核の手(AE・Premiere と同じ)。
        out.extend(self.markers.iter().copied());
        out
    }

    /// `moving_sec + delta` を近くの吸い付き先へ寄せた delta を返す。
    fn snapped_delta(&self, moving_sec: f64, delta: f64) -> f64 {
        let threshold = SNAP_PX / self.pps;
        let landed = moving_sec + delta;
        let mut best: Option<(f64, f64)> = None;
        for target in self.snap_targets() {
            let d = (target - landed).abs();
            if d <= threshold && best.map(|(bd, _)| d < bd).unwrap_or(true) {
                best = Some((d, target));
            }
        }
        match best {
            Some((_, target)) => target - moving_sec,
            None => delta,
        }
    }

    fn band_hit(&self, x: f64, y: f64) -> Option<usize> {
        let row_ix = ((y - RULER_H * self.sfac() + self.scroll_y) / (ROW_H * self.sfac())).floor();
        if row_ix < 0.0 {
            return None;
        }
        let row_ix = row_ix as usize;
        let row = self.rows.get(row_ix)?;
        let (a, b) = row.span?;
        let t = self.scroll_sec + x / self.pps;
        (t >= a && t <= b).then_some(row_ix)
    }

    fn hit_test(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let (rh, rowh) = (RULER_H * self.sfac(), ROW_H * self.sfac());
        let row_ix = ((y - rh + self.scroll_y) / rowh).floor();
        if row_ix < 0.0 {
            return None;
        }
        let row_ix = row_ix as usize;
        let row = self.rows.get(row_ix)?;
        let mid_y = rh + row_ix as f64 * rowh + rowh * 0.5 - self.scroll_y;
        if (y - mid_y).abs() > 8.0 * self.sfac() {
            return None;
        }
        let mut best: Option<(usize, f64)> = None;
        for (ki, t) in row.keys.iter().enumerate() {
            let dx = (x - (t - self.scroll_sec) * self.pps).abs();
            if dx <= 6.0 * self.sfac() && best.map_or(true, |(_, d)| dx < d) {
                best = Some((ki, dx));
            }
        }
        best.map(|(ki, _)| (row_ix, ki))
    }

    /// 囲んだ中のキーを選ぶ(足す)。
    fn select_inside(&mut self, from: (f64, f64), to: (f64, f64)) {
        let (x0, x1) = (from.0.min(to.0), from.0.max(to.0));
        let (y0, y1) = (from.1.min(to.1), from.1.max(to.1));
        if (x1 - x0) < 3.0 && (y1 - y0) < 3.0 {
            return;
        }
        let (rh, rowh) = (RULER_H * self.sfac(), ROW_H * self.sfac());
        let mut layers = Vec::new();
        // 錠は 1 回の lock で全部引く(行ごとに doc を取り合わない)。
        let locked_set: std::collections::HashSet<LayerId> = self
            .doc
            .as_ref()
            .map(|d| {
                let d = d.lock().unwrap();
                let view = d.view();
                view.layers()
                    .into_iter()
                    .filter(|l| view.attrs(*l).ok().flatten().is_some_and(|a| a.locked))
                    .collect()
            })
            .unwrap_or_default();
        for (row_ix, row) in self.rows.iter().enumerate() {
            let mid = rh + row_ix as f64 * rowh + rowh * 0.5 - self.scroll_y;
            if mid < y0 || mid > y1 {
                continue;
            }
            // 錠の掛かった層のキーは選ばない(選べれば消せてしまう)。層そのものは選べる。
            let locked = row.layer.is_some_and(|l| locked_set.contains(&l));
            for (key_ix, t) in row.keys.iter().enumerate() {
                let x = (t - self.scroll_sec) * self.pps;
                if !locked && x >= x0 && x <= x1 && !self.selected.contains(&(row_ix, key_ix)) {
                    self.selected.push((row_ix, key_ix));
                }
            }
            // 帯が箱に触れていれば層も選ぶ(Premiere・Blender は箱で clip も選ぶ)。
            if let (Some(layer), Some((a, b))) = (row.layer, row.span) {
                let xa = (a - self.scroll_sec) * self.pps;
                let xb = (b - self.scroll_sec) * self.pps;
                if xb >= x0 && xa <= x1 && row.prop.is_none() {
                    layers.push(layer);
                }
            }
        }
        if let (Some(selection), Some(mirror), false) =
            (self.selection.as_ref(), self.selected_mirror.as_mut(), layers.is_empty())
        {
            // ⌘ / ⇧ 付きの囲いは足す(キーと同じ流儀)。素の囲いは置き換える。
            if self.marquee_add {
                for layer in &layers {
                    if !selection.contains(*layer) {
                        selection.toggle(*layer);
                    }
                }
            } else {
                selection.set(Some(layers[0]));
                for layer in &layers[1..] {
                    selection.toggle(*layer);
                }
            }
            mirror.set(selection.get());
        }
        self.publish_keys();
    }

    /// 横スクロールの天井。作品の終わりが左端に来る所より先へは行かない(右が無限にならない)。
    fn scroll_ceiling(&self) -> f64 {
        // 天井は帯の右端・印・作品の尺の一番遠い所(層が前半にしか無い 4 分の曲でも後半へ行ける)。
        let bands = self.rows.iter().filter_map(|r| r.span.map(|(_, b)| b)).fold(0.0_f64, f64::max);
        let marks = self.markers.iter().copied().fold(0.0_f64, f64::max);
        let comp = self.clock.as_ref().map_or(0.0, |c| c.duration());
        let end = bands.max(marks).max(comp);
        (end - self.viewport_w * 0.5 / self.pps).max(0.0)
    }

    /// 掴んでいるキーを窓の側へ出す。イージングのパネルがこれを読む。
    /// 行を差し替える。選んだキーは index でなく(層・属性・秒)で追う —— twirl の開閉や
    /// 親子の入れ子で行が組み替わっても、別のキーを指さない。
    fn replace_rows(&mut self, rows: Vec<CanvasRow>) {
        let half = 0.5 / self.fps.max(1.0);
        let kept: Vec<(Option<LayerId>, Option<crate::doc::store::PropertyId>, f64)> = self
            .selected
            .iter()
            .filter_map(|(row_ix, key_ix)| {
                let row = self.rows.get(*row_ix)?;
                Some((row.layer, row.prop.clone(), *row.keys.get(*key_ix)?))
            })
            .collect();
        self.rows = rows;
        self.selected = kept
            .iter()
            .filter_map(|(layer, prop, sec)| {
                let row_ix = self.rows.iter().position(|r| r.layer == *layer && r.prop == *prop)?;
                let key_ix = self.rows[row_ix].keys.iter().position(|k| (k - sec).abs() < half)?;
                Some((row_ix, key_ix))
            })
            .collect();
        self.hovered = None;
        self.publish_keys();
    }

    /// 錠の掛かった層は掴めない。選ぶ事はできる —— 外す為に。
    fn is_locked(&self, layer: LayerId) -> bool {
        self.doc
            .as_ref()
            .and_then(|d| d.lock().unwrap().view().attrs(layer).ok().flatten())
            .is_some_and(|a| a.locked)
    }

    fn publish_keys(&self) {
        let Some(slot) = &self.selected_key else { return };
        let mut out: Vec<crate::ui::session::KeySel> = self
            .selected
            .iter()
            .filter_map(|(row_ix, key_ix)| {
                let row = self.rows.get(*row_ix)?;
                let at_sec = row.keys.get(*key_ix).copied()?;
                Some(crate::ui::session::KeySel {
                    layer: row.layer?,
                    property: row.prop.clone(),
                    at_sec,
                })
            })
            .collect();
        out.sort_by(|a, b| a.at_sec.total_cmp(&b.at_sec));
        *slot.lock().unwrap() = out;
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                TimelineMsg::SetRows(rows) => self.replace_rows(rows),
                TimelineMsg::ScrollBy(dy) => self.set_scroll_y(self.scroll_y + dy),
                TimelineMsg::SetMarkers(markers) => self.markers = markers,
                TimelineMsg::DeselectKeys => {
                    self.selected.clear();
                    self.publish_keys();
                }
            }
        }
    }
}

fn attrs_to_patch(a: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
        flatten: Some(a.flatten),
        hidden: Some(a.hidden),
        parent: Some(a.parent),
        blend_mode: Some(a.blend_mode.clone()),
        matte: Some(a.matte.clone()),
        name: Some(a.name.clone()),
        auto_orient: Some(a.auto_orient),
        pinned: Some(a.pinned),
        solo: Some(a.solo),
        locked: Some(a.locked),
        label_color: Some(a.label_color),
    }
}

/// 層をそのまま増やす。中身(尺・見え方・エフェクト・キー)は全部連れていく。
/// 重ね順だけ1つ上へ置く — AE の Cmd+D と同じで、複製は元の上に出る。
pub(super) fn duplicate_layer(doc: &Arc<Mutex<Document>>, layer: LayerId) -> Option<LayerId> {
    let mut doc = doc.lock().unwrap();
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    let copy = LayerId(view.next_layer_id());
    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let effects = view.effects(layer).unwrap_or_default();
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();
    let shapes = view.shapes(layer).unwrap_or_default();
    let text = view.text_document(layer).ok().flatten();

    let above: Vec<(LayerId, i16)> = view
        .layers()
        .into_iter()
        .filter(|l| *l != layer)
        .filter_map(|l| view.meta(l).ok().flatten().map(|m| (l, m.order)))
        .filter(|(_, order)| *order > meta.order)
        .collect();
    let mut intents = vec![
        Intent::AddLayer(copy),
        Intent::SetMeta {
            layer: copy,
            meta: LayerMeta { order: meta.order.saturating_add(1), ..meta.clone() },
        },
        Intent::SetAttrs { layer: copy, patch: attrs_to_patch(&attrs) },
    ];
    // 複製は元の**すぐ上**に割り込む。上に居た層は 1 つずつ退く。
    for (l, order) in above {
        intents.push(Intent::SetOrder { layer: l, order: order.saturating_add(1) });
    }
    if !effects.is_empty() {
        intents.push(Intent::SetEffects { layer: copy, effects });
    }
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes { layer: copy, shapes });
    }
    if let Some(document) = text {
        intents.push(Intent::SetTextDocument { layer: copy, document });
    }
    for (property, track) in tracks {
        intents.push(Intent::SetTrack { layer: copy, property, track });
    }

    doc.apply_all(intents).ok()?;
    Some(copy)
}

pub(super) fn split_layer(doc: &Arc<Mutex<Document>>, layer: LayerId, comp_frame: i64) -> Option<LayerId> {
    let mut doc = doc.lock().unwrap();
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    if !meta.timing.covers(comp_frame) {
        return None;
    }
    let head_dur = comp_frame - meta.timing.start;
    if head_dur <= 0 {
        return None;
    }

    let tail = LayerId(view.next_layer_id());
    let head_timing = LayerTiming { duration: head_dur, ..meta.timing };
    let tail_timing = LayerTiming {
        start: comp_frame,
        duration: meta.timing.duration - head_dur,
        source_in: meta.timing.source_in + head_dur,
        ..meta.timing
    };

    let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let effects = view.effects(layer).unwrap_or_default();
    let shapes = view.shapes(layer).unwrap_or_default();
    let text = view.text_document(layer).ok().flatten();
    let fps = document_fps(&doc).ok()?;
    let cut = RationalTime::try_from_frame(comp_frame, fps).ok()?;
    let tracks: Vec<_> = view
        .properties(layer)
        .into_iter()
        .filter_map(|p| view.track(layer, &p).ok().flatten().map(|t| (p, t)))
        .collect();

    let mut intents = vec![
        Intent::SetTiming { layer, timing: head_timing },
        Intent::AddLayer(tail),
        Intent::SetMeta { layer: tail, meta: LayerMeta { timing: tail_timing, ..meta } },
        Intent::SetAttrs { layer: tail, patch: attrs_to_patch(&attrs) },
    ];
    if !effects.is_empty() {
        intents.push(Intent::SetEffects { layer: tail, effects });
    }
    // 形と文字も連れていく(複製と同じ)。切らないと歌詞層の尻が空になる。
    if !shapes.is_empty() {
        intents.push(Intent::SetShapes { layer: tail, shapes });
    }
    if let Some(document) = text {
        intents.push(Intent::SetTextDocument { layer: tail, document });
    }
    for (property, track) in tracks {
        let (head, tail_track) = split_track_at(&track, cut);
        if let Some(head) = head {
            intents.push(Intent::SetTrack { layer, property: property.clone(), track: head });
        }
        intents.push(Intent::SetTrack { layer: tail, property, track: tail_track });
    }

    doc.apply_all(intents).ok()?;
    Some(tail)
}

/// 切り口を跨ぐ区間のイージングを両側へ分ける(Premiere・Resolve は切っても見た目が変わらない)。
/// 跨ぐ区間が無ければ頭はそのまま(None)、尻は丸ごと写す。
fn split_track_at(track: &KeyframeTrack, cut: RationalTime) -> (Option<KeyframeTrack>, KeyframeTrack) {
    let keys = track.keys();
    let straddle = keys
        .windows(2)
        .position(|pair| pair[0].t < cut && cut < pair[1].t);
    let Some(i) = straddle else {
        return (None, track.clone());
    };
    let (a, b) = (&keys[i], &keys[i + 1]);
    let progress = (cut.as_seconds_f64() - a.t.as_seconds_f64())
        / (b.t.as_seconds_f64() - a.t.as_seconds_f64()).max(f64::EPSILON);
    let Ok((first, second)) = a.interp.split_at(progress) else {
        return (None, track.clone());
    };
    let at_cut = crate::doc::store::Keyframe {
        t: cut,
        value: track.eval(cut),
        interp: second,
        spatial: None,
    };
    let mut head = KeyframeTrack::new();
    let mut tail = KeyframeTrack::new();
    for (k, key) in keys.iter().enumerate() {
        let mut key = key.clone();
        if k == i {
            key.interp = first;
        }
        if k <= i {
            head.insert(key.clone());
        }
        if k > i {
            tail.insert(key);
        }
    }
    head.insert(at_cut.clone());
    tail.insert(at_cut);
    (Some(head), tail)
}

fn fill_rect(s: &mut anyrender::Scene, r: Rect, color: Color) {
    s.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(color), None, &r);
}

fn diamond(s: &mut anyrender::Scene, center: Point, d: f64, color: Color) {
    let r = Rect::from_center_size(center, Size::new(d, d));
    s.fill(
        Fill::NonZero,
        Affine::rotate_about(std::f64::consts::FRAC_PI_4, center),
        PaintRef::Solid(color),
        None,
        &r,
    );
}

impl Widget for TimelineWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}
    fn can_create_surfaces(&mut self, _render_ctx: &mut dyn anyrender::RenderContext) {}
    fn destroy_surfaces(&mut self) {}

    fn requires_redraw(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent) {
        if self
            .gesture
            .as_ref()
            .is_some_and(|gesture| gesture.cancelled(&mut self.seen_cancel))
        {
            self.drag = None;
            self.marker_drag = None;
            self.scrubbing = false;
            self.marquee = None;
            return;
        }
        match event {
            UiEvent::Wheel(wheel) => {
                let (dx, dy) = match wheel.delta {
                    BlitzWheelDelta::Pixels(x, y) => (x, y),
                    BlitzWheelDelta::Lines(x, y) => (x * 20.0, y * 20.0),
                };
                // 目盛の帯の上では素のホイールでも横に広がる。Ctrl(トラックパッドの
                // 摘まみ)だけだと、**見つけられない手**になる —— 60秒の尺では
                // 0.6秒が5画素で、キーが重なって選び分けられない。
                let on_ruler = (wheel.element.y as f64) < RULER_H * self.sfac();
                // 摘まみ(Ctrl)・主の修飾(⌘)・目盛の帯、どれでも広がる。
                // 1つしか道が無いと**見つけられない手**になる。
                let primary = if cfg!(target_os = "macos") {
                    wheel.mods.intersects(Modifiers::META | Modifiers::SUPER)
                } else {
                    wheel.mods.contains(Modifiers::CONTROL)
                };
                if wheel.mods.contains(Modifiers::CONTROL) || primary || on_ruler {
                    let cursor_x = wheel.element.x as f64;
                    let cursor_sec = self.scroll_sec + cursor_x / self.pps;
                    // 上へ回すと広がる。Stage の拡縮と同じ向き。
                    let new_pps = (self.pps * (1.0 + dy * 0.002)).clamp(MIN_PPS, MAX_PPS);
                    self.pps = new_pps;
                    self.scroll_sec = (cursor_sec - cursor_x / new_pps).clamp(0.0, self.scroll_ceiling().max(cursor_sec));
                } else {
                    self.set_scroll_y(self.scroll_y + dy);
                }
                self.scroll_sec = (self.scroll_sec - dx / self.pps).clamp(0.0, self.scroll_ceiling());
                if let Some((cx, cy)) = self.cursor {
                    self.hovered = self.hit_test(cx, cy);
                }
            }
            UiEvent::PointerMove(p) if self.marquee.is_some() => {
                let (x, y) = (p.element.x as f64, p.element.y as f64);
                if let Some((_, to)) = self.marquee.as_mut() {
                    *to = (x, y);
                }
            }
            UiEvent::PointerMove(p) => {
                let (x, y) = (p.element.x as f64, p.element.y as f64);
                self.cursor = Some((x, y));
                // 帯の外で離すと、離した事がここへ届かない。掴んだままの絵が残り、
                // **見えている物が作品と食い違う**。指が上がっていたら掴みを解く。
                if p.buttons.is_empty()
                    && (self.drag.is_some() || self.scrubbing || self.marquee.is_some() || self.marker_drag.is_some())
                {
                    println!("PROBE room=input verdict=drag-finished reason=release-not-seen");
                    self.finish_drag();
                    return;
                }
                if let Some((i, orig, grab, _)) = self.marker_drag {
                    let raw = (self.scroll_sec + x / self.pps) - grab;
                    let landed = self.snapped_time_excluding(orig + raw, i).max(0.0);
                    let delta = ((landed - orig) * self.fps).round() / self.fps;
                    self.marker_drag = Some((i, orig, grab, delta));
                    return;
                }
                if self.scrubbing {
                    if let Some(clock) = &self.clock {
                        let t = self.scroll_sec + x / self.pps;
                        // 再生位置も印・帯の端へ吸う。⌘ を添えると素通り(吸い付きを切る手)。
                        let free = p.mods.intersects(Modifiers::META | Modifiers::SUPER);
                        clock.seek(if free { t } else { self.snapped_time(t) });
                    }
                } else if self.drag.is_some() {
                    let raw = (self.scroll_sec + x / self.pps)
                        - self.drag.as_ref().expect("直前に確認した").grab_sec;
                    // **押しただけでは動かない。**(外の規格の touch slop)
                    if !self.drag.as_ref().expect("同上").moved
                        && (raw * self.pps).abs() < DRAG_SLOP_PX
                    {
                        return;
                    }
                    if let Some(drag) = &mut self.drag {
                        drag.moved = true;
                    }
                    let moving = match self.drag.as_ref().expect("同上").mode {
                        DragMode::Key { at_sec } => at_sec,
                        DragMode::TrimEnd => {
                            let d = self.drag.as_ref().expect("同上");
                            (d.orig.start + d.orig.duration) as f64 / self.fps
                        }
                        _ => self.drag.as_ref().expect("同上").orig.start as f64 / self.fps,
                    };
                    // ⌘ を添えると吸い付きを切る(再生位置と同じ手)。
                    let free = p.mods.intersects(Modifiers::META | Modifiers::SUPER);
                    let landed = if free { raw } else { self.snapped_delta(moving, raw) };
                    let snapped = (landed * self.fps).round() / self.fps;
                    if let Some(drag) = &mut self.drag {
                        drag.delta_sec = snapped;
                    }
                } else {
                    self.hovered = self.hit_test(x, y);
                }
            }
            UiEvent::PointerDown(p) => {
                if let Some(gesture) = &self.gesture {
                    gesture.begin();
                }
                let (x, y) = (p.element.x as f64, p.element.y as f64);
                let t = self.scroll_sec + x / self.pps;
                if y < RULER_H * self.sfac() {
                    // 印は掴んで動かせる(AE・Premiere)。近ければ seek でなく印を持つ。
                    let reach = 6.0 * self.sfac() / self.pps;
                    let grabbed = self
                        .markers
                        .iter()
                        .enumerate()
                        .map(|(i, m)| (i, *m, (m - t).abs()))
                        .filter(|(_, _, d)| *d <= reach)
                        .min_by(|a, b| a.2.total_cmp(&b.2));
                    if let Some((i, sec, _)) = grabbed {
                        println!("PROBE room=input down t={t:.3}s hit=marker index={i}");
                        self.marker_drag = Some((i, sec, t, 0.0));
                        return;
                    }
                    println!("PROBE room=input down t={:.3}s el=({:.0},{:.0}) hit=ruler-seek", t, x, y);
                    self.scrubbing = true;
                    if let Some(clock) = &self.clock {
                        clock.seek(t);
                    }
                    return;
                }
                let hit = self.hit_test(x, y);
                println!(
                    "PROBE room=input down t={:.3}s el=({:.0},{:.0}) hit={:?}",
                    t, x, y, hit
                );
                if let Some((row_ix, key_ix)) = hit {
                    if self.rows[row_ix].layer.is_some_and(|l| self.is_locked(l)) {
                        println!("PROBE room=input verdict=locked-key row={row_ix}");
                        return;
                    }
                    let add = p.mods.intersects(Modifiers::META | Modifiers::SUPER) || p.mods.contains(Modifiers::SHIFT);
                    match (add, self.selected.iter().position(|k| *k == (row_ix, key_ix))) {
                        (true, Some(at)) => {
                            self.selected.remove(at);
                        }
                        (true, None) => self.selected.push((row_ix, key_ix)),
                        (false, Some(_)) => {}
                        (false, None) => self.selected = vec![(row_ix, key_ix)],
                    }
                    self.publish_keys();
                    if let (Some(layer), Some(at_sec)) = (
                        self.rows[row_ix].layer,
                        self.rows[row_ix].keys.get(key_ix).copied(),
                    ) {
                        if self.is_locked(layer) {
                            println!("PROBE room=input verdict=locked layer={layer:?}");
                            return;
                        }
                        let orig = self
                            .doc
                            .as_ref()
                            .and_then(|d| d.lock().unwrap().view().meta(layer).ok().flatten())
                            .map(|m| m.timing)
                            .unwrap_or_default();
                        self.drag = Some(DragState {
                            row: row_ix,
                            layer,
                            prop: self.rows[row_ix].prop.clone(),
                            orig,
                            grab_sec: t,
                            delta_sec: 0.0,
                            mode: DragMode::Key { at_sec },
                            moved: false,
                        });
                    }
                } else if let Some(row_ix) = self.band_hit(x, y) {
                    let layer = self.rows[row_ix].layer;
                    if let (Some(selection), Some(mirror)) =
                        (self.selection.as_ref(), self.selected_mirror.as_mut())
                    {
                        if p.mods.intersects(Modifiers::META | Modifiers::SUPER) || p.mods.contains(Modifiers::SHIFT) {
                            if let Some(l) = layer {
                                selection.toggle(l);
                            }
                        } else {
                            selection.set(layer);
                        }
                        mirror.set(selection.get());
                    }
                    let orig = layer.and_then(|l| {
                        self.doc
                            .as_ref()
                            .and_then(|d| d.lock().unwrap().view().meta(l).ok().flatten())
                            .map(|m| m.timing)
                    });
                    if let (Some(layer), Some(orig)) = (layer, orig) {
                        if self.is_locked(layer) {
                            println!("PROBE room=input verdict=locked layer={layer:?}");
                            return;
                        }
                        let mode = match self.rows[row_ix].span {
                            Some((a, b)) => {
                                let xa = (a - self.scroll_sec) * self.pps;
                                let xb = (b - self.scroll_sec) * self.pps;
                                // ⌥ は常に Slip の宣言(AE)。端でも trim に食わせない。
                                if p.mods.contains(Modifiers::ALT) {
                                    DragMode::Slip
                                } else if (x - xa).abs() <= EDGE_GRAB_PX {
                                    DragMode::TrimStart
                                } else if (x - xb).abs() <= EDGE_GRAB_PX {
                                    DragMode::TrimEnd
                                } else {
                                    DragMode::Move
                                }
                            }
                            None => DragMode::Move,
                        };
                        println!("PROBE room=write drag-start row={row_ix} start={}", orig.start);
                        self.drag = Some(DragState {
                            row: row_ix,
                            layer,
                            prop: None,
                            orig,
                            grab_sec: t,
                            delta_sec: 0.0,
                            mode,
                            moved: false,
                        });
                    }
                } else {
                    // 何も無い所からのドラッグは囲って選ぶ。
                    let add = p.mods.intersects(Modifiers::META | Modifiers::SUPER) || p.mods.contains(Modifiers::SHIFT);
                    if !add {
                        self.selected.clear();
                        self.publish_keys();
                    }
                    self.marquee_add = add;
                    self.marquee = Some(((x, y), (x, y)));
                }
            }
            UiEvent::PointerUp(_) => {
                self.finish_drag();
            }
            _ => {}
        }
    }

    fn paint(
        &mut self,
        _render_ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> anyrender::Scene {
        if self
            .gesture
            .as_ref()
            .is_some_and(|gesture| gesture.cancelled(&mut self.seen_cancel))
        {
            self.drag = None;
            self.marker_drag = None;
            self.scrubbing = false;
            self.marquee = None;
        }
        self.process_messages();
        // 誰かが書いたら行を作り直す。Stage で打ったキーが出ない、を塞ぐ。
        if let Some(rev) = &self.revision {
            let now = (*rev)();
            if now != self.seen_revision {
                self.seen_revision = now;
                if let (Some(doc), Some(extractor)) = (self.doc.as_ref(), self.extractor) {
                    let doc = doc.lock().unwrap();
                    if let Ok(fps) = document_fps(&doc) {
                        self.fps = fps.as_f64();
                    }
                    self.rows = extractor(&doc);
                }
            }
        }
        // 再生位置が動いたら、値を出している側へ知らせる。
        if let (Some(clock), Some(mirror)) = (&self.clock, &mut self.playhead_mirror) {
            let now = clock.now_sec();
            if ((*mirror)() - now).abs() > 1e-6 {
                mirror.set(now);
            }
        }

        let mut s = anyrender::Scene::new();
        if width == 0 || height == 0 {
            return s;
        }
        let w = width as f64;
        let h = height as f64;
        let k = scale;
        let ruler_h = RULER_H * self.sfac() * k;
        let row_h = ROW_H * self.sfac() * k;
        self.viewport_h = h / k;
        self.viewport_w = w / k;
        self.set_scroll_y(self.scroll_y);
        let scroll_y = self.scroll_y * k;
        let pps = self.pps * k;
        let scroll = self.scroll_sec;
        let x_of = |t: f64| (t - scroll) * pps;
        let hairline = k.max(1.0);
        let waveform_tracks = self
            .clock
            .as_ref()
            .map(|clock| clock.waveform_tracks())
            .unwrap_or_default();

        let t3 = |v: [u8; 3]| c(v[0], v[1], v[2]);
        let c_app = t3(tokens::SURFACE_APP);
        let c_panel = t3(tokens::SURFACE_PANEL);
        let c_hair = t3(tokens::LINE_DARK);
        let c_bd = t3(tokens::BORDER);
        let c_marker = t3(tokens::ACCENT);
        let c_text = t3(tokens::INK);
        let c_dim = t3(tokens::INK2);
        let c_accent = t3(tokens::ACCENT);
        let c_zebra = Color::from_rgba8(0xff, 0xff, 0xff, 0x09);
        let c_rowline = c_hair;

        fill_rect(&mut s, Rect::new(0.0, 0.0, w, h), c_app);

        // 目盛の段は倍率で変わる(NLE と同じ)。主目盛は約 80px 以上空くように選ぶ。
        let step = nice_step(pps);
        let t_first = (scroll / step).floor() as i64;
        let t_last = ((scroll + w / pps) / step).ceil() as i64;
        for t in t_first..=t_last {
            if t >= 0 && t % 2 == 1 {
                let x0 = x_of(t as f64 * step).max(0.0);
                let x1 = x_of((t + 1) as f64 * step).min(w);
                fill_rect(&mut s, Rect::new(x0, ruler_h, x1, h), c_zebra);
            }
        }

        fill_rect(&mut s, Rect::new(0.0, 0.0, w, ruler_h), c_panel);
        for t in t_first..=t_last {
            if t >= 0 {
                let x = x_of(t as f64 * step);
                if (0.0..=w).contains(&x) {
                    fill_rect(&mut s, Rect::new(x, ruler_h * 0.5, x + hairline, ruler_h), c_bd);
                }
                // 副目盛(主の 1/5)。
                for k in 1..5 {
                    let x = x_of((t as f64 + k as f64 / 5.0) * step);
                    if (0.0..=w).contains(&x) {
                        fill_rect(&mut s, Rect::new(x, ruler_h * 0.78, x + hairline, ruler_h), c_hair);
                    }
                }
            }
        }
        for (i, &sec) in self.markers.iter().enumerate() {
            let shift = match self.marker_drag {
                Some((d, _, _, delta)) if d == i => delta,
                _ => 0.0,
            };
            let x = x_of(sec + shift);
            if (0.0..=w).contains(&x) {
                fill_rect(
                    &mut s,
                    Rect::new(x - 3.0, ruler_h * 0.15, x + 3.0, ruler_h * 0.55),
                    c_marker,
                );
                // 印は全 track を貫く(Premiere・Ableton)。歌詞の頭を目で合わせる線。
                let faint = Color::from_rgba8(tokens::ACCENT[0], tokens::ACCENT[1], tokens::ACCENT[2], 0x50);
                fill_rect(&mut s, Rect::new(x, ruler_h, x + hairline, h), faint);
            }
        }
        fill_rect(&mut s, Rect::new(0.0, ruler_h - hairline, w, ruler_h), c_hair);

        let primary_layer = self.selection.as_ref().and_then(|s| s.get());

        let hover_row = self
            .cursor
            .map(|(_, cy)| ((cy - RULER_H * self.sfac() + self.scroll_y) / (ROW_H * self.sfac())).floor())
            .filter(|r| *r >= 0.0)
            .map(|r| r as usize);

        for (i, row) in self.rows.iter().enumerate() {
            let y = ruler_h + i as f64 * row_h - scroll_y;
            if y >= h {
                break;
            }
            if y + row_h <= ruler_h {
                continue;
            }
            let mid = y + row_h * 0.5;
            let (shift_a, shift_b) = match &self.drag {
                Some(d) if d.row == i => match d.mode {
                    DragMode::Move => (d.delta_sec, d.delta_sec),
                    DragMode::TrimStart => (d.delta_sec, 0.0),
                    DragMode::TrimEnd => (0.0, d.delta_sec),
                    DragMode::Slip | DragMode::Key { .. } => (0.0, 0.0),
                },
                _ => (0.0, 0.0),
            };
            let waveform_shift = match &self.drag {
                Some(d) if d.row == i && matches!(d.mode, DragMode::Move | DragMode::Slip) => d.delta_sec,
                _ => 0.0,
            };

            let top = y.max(ruler_h);
            if row.is_group {
                fill_rect(&mut s, Rect::new(0.0, top, w, y + row_h), c_panel);
            }
            if hover_row == Some(i) {
                fill_rect(
                    &mut s,
                    Rect::new(0.0, top, w, y + row_h),
                    Color::from_rgba8(0xff, 0xff, 0xff, 0x0a),
                );
            }
            fill_rect(&mut s, Rect::new(0.0, y + row_h - hairline, w, y + row_h), c_rowline);

            if let Some((a, b)) = row.span {
                let x0 = x_of(a + shift_a).max(0.0);
                let x1 = x_of(b + shift_b).min(w);
                if x1 > x0 {
                    fill_rect(&mut s, Rect::new(x0, top, x1, y + row_h - hairline), c_hair);
                    fill_rect(
                        &mut s,
                        Rect::new(
                            x0 + hairline,
                            top.max(y + hairline),
                            x1 - hairline,
                            y + row_h - 2.0 * hairline,
                        ),
                        c(row.color[0], row.color[1], row.color[2]),
                    );
                    let waveform_columns = row
                        .prop
                        .is_none()
                        .then_some(row.layer)
                        .flatten()
                        .and_then(|layer| {
                            waveform_tracks.iter().find(|track| track.layer == layer)
                        })
                        .and_then(|track| {
                            // 描くのは物理 px。列は物理 px ごとに要る(Retina で半分にしない)。
                            track.columns(
                                scroll - waveform_shift,
                                scroll - waveform_shift + w / pps,
                                pps,
                            )
                        });
                    if let Some(columns) = waveform_columns {
                        let amplitude = (row_h - 4.0 * hairline) * 0.45;
                        let wave_color = Color::from_rgba8(
                            tokens::INK[0],
                            tokens::INK[1],
                            tokens::INK[2],
                            0x90,
                        );
                        for column in columns {
                            let x = x_of(column.at_sec + waveform_shift);
                            if x < x0 || x > x1 {
                                continue;
                            }
                            let y0 = (mid - f64::from(column.max) * amplitude).max(top);
                            let y1 = (mid - f64::from(column.min) * amplitude)
                                .min(y + row_h - 2.0 * hairline);
                            if y1 >= y0 {
                                fill_rect(
                                    &mut s,
                                    Rect::new(x, y0, x + hairline, (y1 + hairline).min(y + row_h)),
                                    wave_color,
                                );
                            }
                        }
                    }
                    let selected = row
                        .layer
                        .map(|l| Some(l) == primary_layer || self.selection.as_ref().is_some_and(|s| s.contains(l)))
                        .unwrap_or(false);
                    if selected {
                        let is_primary = row.layer.map(|l| Some(l) == primary_layer).unwrap_or(false);
                        let bw = if is_primary { 2.0 * hairline } else { hairline };
                        fill_rect(&mut s, Rect::new(x0, top, x1, top + bw), c_accent);
                        fill_rect(&mut s, Rect::new(x0, y + row_h - hairline - bw, x1, y + row_h - hairline), c_accent);
                    }
                }
            }
            if mid < ruler_h {
                continue;
            }
            for kf in &row.agg {
                let x = x_of(*kf);
                if x < 0.0 || x > w {
                    continue;
                }
                diamond(&mut s, Point::new(x, mid), 5.0 * k, c_dim);
            }
            let key_shift = match &self.drag {
                Some(d) if d.row == i => match d.mode {
                    DragMode::Key { at_sec } => Some((at_sec, d.delta_sec)),
                    _ => None,
                },
                _ => None,
            };
            // 選んだキーが隣り合っていたら、その間が区間。帯で示す。
            {
                let mut chosen: Vec<usize> = self
                    .selected
                    .iter()
                    .filter(|(r, _)| *r == i)
                    .map(|(_, k)| *k)
                    .collect();
                chosen.sort_unstable();
                for pair in chosen.windows(2) {
                    if pair[1] != pair[0] + 1 {
                        continue;
                    }
                    let (Some(a), Some(b)) =
                        (row.keys.get(pair[0]).copied(), row.keys.get(pair[1]).copied())
                    else {
                        continue;
                    };
                    let (xa, xb) = (x_of(a).max(0.0), x_of(b).min(w));
                    if xb > xa {
                        fill_rect(
                            &mut s,
                            Rect::new(xa, mid - 1.5 * k, xb, mid + 1.5 * k),
                            c_accent,
                        );
                    }
                }
            }
            for (ki, kf) in row.keys.iter().enumerate() {
                let dragged = key_shift
                    .filter(|(at, _)| (at - *kf).abs() < 0.5 / self.fps)
                    .map(|(_, delta)| delta)
                    .unwrap_or(0.0);
                let center = Point::new(x_of(*kf + dragged), mid);
                if center.x < 0.0 || center.x > w {
                    continue;
                }
                if self.selected.contains(&(i, ki)) {
                    diamond(&mut s, center, 12.0 * k, c_accent);
                    diamond(&mut s, center, 8.0 * k, Color::from_rgb8(0xff, 0xff, 0xff));
                } else if self.hovered == Some((i, ki)) {
                    diamond(&mut s, center, 11.0 * k, c_hair);
                    diamond(&mut s, center, 9.0 * k, Color::from_rgb8(0xf0, 0xf0, 0xf0));
                } else {
                    diamond(&mut s, center, 9.0 * k, c_hair);
                    diamond(&mut s, center, 7.0 * k, c_text);
                }
            }
        }

        if let Some((from, to)) = self.marquee {
            // 囲いは pointer の CSS px で持つ。盤は device px で描くので、ここで倍率を掛ける。
            let r = Rect::new(
                from.0.min(to.0) * k,
                from.1.min(to.1) * k,
                from.0.max(to.0) * k,
                from.1.max(to.1) * k,
            );
            fill_rect(&mut s, r, Color::from_rgba8(0xd8, 0xb5, 0x74, 0x22));
            s.stroke(
                &peniko::kurbo::Stroke::new(1.0),
                Affine::IDENTITY,
                PaintRef::Solid(c_accent),
                None,
                &r,
            );
        }

        let playhead_sec = self
            .clock
            .as_ref()
            .map(|c| c.now_sec())
            .unwrap_or(PLAYHEAD_SEC);
        if self.drag.is_none() && !self.scrubbing {
            let playing = self.clock.as_ref().is_some_and(|c| c.playing());
            let jumped = (playhead_sec - self.last_playhead).abs() > 2.5 / self.fps.max(1.0);
            if let Some(to) = follow_playhead(scroll, w / pps, playhead_sec, playing, jumped) {
                self.scroll_sec = to;
            }
        }
        self.last_playhead = playhead_sec;
        let px = x_of(playhead_sec);
        if (0.0..=w).contains(&px) {
            fill_rect(&mut s, Rect::new(px - hairline * 0.5, 0.0, px + hairline * 0.5, h), c_accent);
            fill_rect(&mut s, Rect::new(px - 3.0 * k, 0.0, px + 3.0 * k, 4.0 * k), c_accent);
        }

        if let Some((cx, _)) = self.cursor {
            let x = cx * k;
            fill_rect(
                &mut s,
                Rect::new(x - hairline * 0.5, 0.0, x + hairline * 0.5, h),
                Color::from_rgba8(0xff, 0xff, 0xff, 0x28),
            );
        }

        s
    }
}

#[cfg(test)]
mod tests;
