use std::rc::Rc;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use crate::ui::playback::Clock;
use crate::ui::session::Selection;
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
const DOC_FPS: f64 = 30.0;
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

fn keyframe_shift_intents(
    doc: &Document,
    layer: LayerId,
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view();
    let fps = Fps::try_new(DOC_FPS as i64, 1).map_err(|e| StoreError::Property(e.to_string()))?;
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
    Ok(intents)
}

/// `at_frame` にあるキーだけを `delta_frames` ずらす。層の行に見えている菱形は
/// 複数のトラックの同じ時刻を束ねているので、束ごと動かす(AE と同じ)。
fn keyframe_move_intents(
    doc: &Document,
    layer: LayerId,
    only: Option<&crate::doc::store::PropertyId>,
    at_frame: i64,
    delta_frames: i64,
) -> Result<Vec<Intent>, StoreError> {
    let view = doc.view();
    let fps = Fps::try_new(DOC_FPS as i64, 1).map_err(|e| StoreError::Property(e.to_string()))?;
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
            if frame == at_frame {
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
}

pub(super) struct TimelineWidget {
    rx: Rc<Receiver<TimelineMsg>>,
    rows: Vec<CanvasRow>,
    markers: Vec<f64>,
    pps: f64,
    scroll_sec: f64,
    scroll_y: f64,
    viewport_h: f64,
    cursor: Option<(f64, f64)>,
    hovered: Option<(usize, usize)>,
    selected: Vec<(usize, usize)>,
    /// 空きからのドラッグで囲む。掴み始めと今の場所(秒, 行)。
    marquee: Option<((f64, f64), (f64, f64))>,
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
    cancel: Option<Arc<std::sync::atomic::AtomicU32>>,
    gesture_active: Option<Arc<std::sync::atomic::AtomicBool>>,
    seen_cancel: u32,
    selected_key: Option<Arc<Mutex<Vec<crate::ui::session::KeySel>>>>,
}

impl TimelineWidget {
    pub(super) fn new(rows: Vec<CanvasRow>, rx: Rc<Receiver<TimelineMsg>>) -> Self {
        Self {
            rx,
            rows,
            markers: Vec::new(),
            pps: PX_PER_SEC,
            scroll_sec: 0.0,
            scroll_y: 0.0,
            viewport_h: 0.0,
            cursor: None,
            hovered: None,
            selected: Vec::new(),
            marquee: None,
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
            cancel: None,
            gesture_active: None,
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

    pub(super) fn with_cancel(
        mut self,
        cancel: Arc<std::sync::atomic::AtomicU32>,
        active: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        self.cancel = Some(cancel);
        self.gesture_active = Some(active);
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
        if let Some(a) = &self.gesture_active {
            a.store(false, std::sync::atomic::Ordering::Relaxed);
        }
                self.scrubbing = false;
                if let Some((from, to)) = self.marquee.take() {
                    self.select_inside(from, to);
                }
                if let Some(drag) = self.drag.take() {
                    let (Some(doc), Some(extractor)) = (self.doc.as_ref(), self.extractor) else {
                        return;
                    };
                    let mut doc = doc.lock().unwrap();
                    let raw_delta = (drag.delta_sec * DOC_FPS).round() as i64;
                    if let DragMode::Key { at_sec } = drag.mode {
                        let at_frame = (at_sec * DOC_FPS).round() as i64;
                        if raw_delta != 0 {
                            // 掴んだ物だけでなく、選んでいるキーを全部同じだけ動かす。
                            let mut moving: Vec<(LayerId, Option<crate::doc::store::PropertyId>, i64)> = self
                                .selected
                                .iter()
                                .filter_map(|(row_ix, key_ix)| {
                                    let row = self.rows.get(*row_ix)?;
                                    let t = row.keys.get(*key_ix).copied()?;
                                    Some((row.layer?, row.prop.clone(), (t * DOC_FPS).round() as i64))
                                })
                                .collect();
                            if !moving.iter().any(|(l, p, f)| {
                                *l == drag.layer && *p == drag.prop && *f == at_frame
                            }) {
                                moving.push((drag.layer, drag.prop.clone(), at_frame));
                            }
                            let mut all = Vec::new();
                            let mut failed = None;
                            for (layer, prop, frame) in moving {
                                match keyframe_move_intents(&doc, layer, prop.as_ref(), frame, raw_delta) {
                                    Ok(intents) => all.extend(intents),
                                    Err(e) => failed = Some(e),
                                }
                            }
                            match failed.map_or(Ok(all), Err) {
                                Ok(intents) => match doc.apply_all(intents) {
                                    Ok(_) => {
                                        println!(
                                            "PROBE room=write verdict=applied MoveKey layer={:?} frame {}->{}",
                                            drag.layer, at_frame, at_frame + raw_delta
                                        );
                                        self.rows = extractor(&doc);
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
                        }
                    }
                    match doc.apply_all(intents) {
                        Ok(_) => {
                            println!(
                                "PROBE room=write verdict=applied SetTiming mode={} start {}->{} dur {}->{}",
                                match drag.mode {
                                    DragMode::Move => "move",
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
        if (y - mid_y).abs() > 8.0 {
            return None;
        }
        let mut best: Option<(usize, f64)> = None;
        for (ki, t) in row.keys.iter().enumerate() {
            let dx = (x - (t - self.scroll_sec) * self.pps).abs();
            if dx <= 6.0 && best.map_or(true, |(_, d)| dx < d) {
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
        for (row_ix, row) in self.rows.iter().enumerate() {
            let mid = rh + row_ix as f64 * rowh + rowh * 0.5 - self.scroll_y;
            if mid < y0 || mid > y1 {
                continue;
            }
            for (key_ix, t) in row.keys.iter().enumerate() {
                let x = (t - self.scroll_sec) * self.pps;
                if x >= x0 && x <= x1 && !self.selected.contains(&(row_ix, key_ix)) {
                    self.selected.push((row_ix, key_ix));
                }
            }
        }
        self.publish_keys();
    }

    /// 掴んでいるキーを窓の側へ出す。イージングのパネルがこれを読む。
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
                TimelineMsg::SetRows(rows) => self.rows = rows,
                TimelineMsg::ScrollBy(dy) => self.set_scroll_y(self.scroll_y + dy),
                TimelineMsg::SetMarkers(markers) => self.markers = markers,
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

    let mut intents = vec![
        Intent::AddLayer(copy),
        Intent::SetMeta {
            layer: copy,
            meta: LayerMeta { order: meta.order.saturating_add(1), ..meta.clone() },
        },
        Intent::SetAttrs { layer: copy, patch: attrs_to_patch(&attrs) },
    ];
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
    for (property, track) in tracks {
        intents.push(Intent::SetTrack { layer: tail, property, track });
    }

    doc.apply_all(intents).ok()?;
    Some(tail)
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
        match event {
            UiEvent::Wheel(wheel) => {
                let (dx, dy) = match wheel.delta {
                    BlitzWheelDelta::Pixels(x, y) => (x, y),
                    BlitzWheelDelta::Lines(x, y) => (x * 20.0, y * 20.0),
                };
                if wheel.mods.contains(Modifiers::CONTROL) {
                    let cursor_x = wheel.element.x as f64;
                    let cursor_sec = self.scroll_sec + cursor_x / self.pps;
                    let new_pps = (self.pps * (1.0 - dy * 0.002)).clamp(MIN_PPS, MAX_PPS);
                    self.pps = new_pps;
                    self.scroll_sec = (cursor_sec - cursor_x / new_pps).max(0.0);
                } else {
                    self.set_scroll_y(self.scroll_y + dy);
                }
                self.scroll_sec = (self.scroll_sec - dx / self.pps).max(0.0);
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
                if let Some(c) = &self.cancel {
                    let now = c.load(std::sync::atomic::Ordering::Relaxed);
                    if now != self.seen_cancel {
                        self.seen_cancel = now;
                        // **書かずに**手放す。
                        self.drag = None;
                        self.scrubbing = false;
                        self.marquee = None;
                        if let Some(a) = &self.gesture_active {
                            a.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        return;
                    }
                }
                self.cursor = Some((x, y));
                // 帯の外で離すと、離した事がここへ届かない。掴んだままの絵が残り、
                // **見えている物が作品と食い違う**。指が上がっていたら掴みを解く。
                if p.buttons.is_empty() && (self.drag.is_some() || self.scrubbing || self.marquee.is_some()) {
                    println!("PROBE room=input verdict=drag-finished reason=release-not-seen");
                    self.finish_drag();
                    return;
                }
                if self.scrubbing {
                    if let Some(clock) = &self.clock {
                        clock.seek(self.scroll_sec + x / self.pps);
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
                            (d.orig.start + d.orig.duration) as f64 / DOC_FPS
                        }
                        _ => self.drag.as_ref().expect("同上").orig.start as f64 / DOC_FPS,
                    };
                    let snapped = self.snapped_delta(moving, raw);
                    if let Some(drag) = &mut self.drag {
                        drag.delta_sec = snapped;
                    }
                } else {
                    self.hovered = self.hit_test(x, y);
                }
            }
            UiEvent::PointerDown(p) => {
                let (x, y) = (p.element.x as f64, p.element.y as f64);
                let t = self.scroll_sec + x / self.pps;
                if y < RULER_H * self.sfac() {
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
                    let add = p.mods.contains(Modifiers::META) || p.mods.contains(Modifiers::SHIFT);
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
                        if p.mods.contains(Modifiers::META) {
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
                        let mode = match self.rows[row_ix].span {
                            Some((a, b)) => {
                                let xa = (a - self.scroll_sec) * self.pps;
                                let xb = (b - self.scroll_sec) * self.pps;
                                if (x - xa).abs() <= EDGE_GRAB_PX {
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
                    if !p.mods.contains(Modifiers::META) && !p.mods.contains(Modifiers::SHIFT) {
                        self.selected.clear();
                        self.publish_keys();
                    }
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
        self.process_messages();
        // 誰かが書いたら行を作り直す。Stage で打ったキーが出ない、を塞ぐ。
        if let Some(rev) = &self.revision {
            let now = (*rev)();
            if now != self.seen_revision {
                self.seen_revision = now;
                if let (Some(doc), Some(extractor)) = (self.doc.as_ref(), self.extractor) {
                    let doc = doc.lock().unwrap();
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
        self.set_scroll_y(self.scroll_y);
        let scroll_y = self.scroll_y * k;
        let pps = self.pps * k;
        let scroll = self.scroll_sec;
        let x_of = |t: f64| (t - scroll) * pps;
        let hairline = k.max(1.0);

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

        let t_first = scroll.floor() as i64;
        let t_last = (scroll + w / pps).ceil() as i64;
        for t in t_first..=t_last {
            if t >= 0 && t % 2 == 1 {
                let x0 = x_of(t as f64).max(0.0);
                let x1 = x_of(t as f64 + 1.0).min(w);
                fill_rect(&mut s, Rect::new(x0, ruler_h, x1, h), c_zebra);
            }
        }

        fill_rect(&mut s, Rect::new(0.0, 0.0, w, ruler_h), c_panel);
        for t in t_first..=t_last {
            if t >= 0 {
                let x = x_of(t as f64);
                if (0.0..=w).contains(&x) {
                    fill_rect(&mut s, Rect::new(x, ruler_h * 0.5, x + hairline, ruler_h), c_bd);
                }
            }
        }
        for &sec in &self.markers {
            let x = x_of(sec);
            if (0.0..=w).contains(&x) {
                fill_rect(
                    &mut s,
                    Rect::new(x - 3.0, ruler_h * 0.15, x + 3.0, ruler_h * 0.55),
                    c_marker,
                );
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
                    DragMode::Key { .. } => (0.0, 0.0),
                },
                _ => (0.0, 0.0),
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
                    .filter(|(at, _)| (at - *kf).abs() < 0.5 / DOC_FPS)
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
            let r = Rect::new(
                from.0.min(to.0),
                from.1.min(to.1),
                from.0.max(to.0),
                from.1.max(to.1),
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
        // 再生位置が視界から出たら追いかける。掴んでいる間は動かさない。
        if self.drag.is_none() && !self.scrubbing {
            let visible = w / pps;
            let left = scroll;
            let right = scroll + visible;
            if playhead_sec < left || playhead_sec > right - visible * 0.1 {
                self.scroll_sec = (playhead_sec - visible * 0.1).max(0.0);
            }
        }
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
