use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::playback::Clock;
use crate::session::Selection;
use crate::tokens::{self, UiScale};
use anyrender::{PaintRef, PaintScene};
use dioxus_native::prelude::{Signal, WritableExt};
use motolii_store::{
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
const RULER_H: f64 = crate::tokens::ROW;
const ROW_H: f64 = crate::tokens::ROW;
const PLAYHEAD_SEC: f64 = 4.6;

fn c(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

#[derive(Clone)]
pub struct CanvasRow {
    pub is_group: bool,
    pub keys: Vec<f64>,
    pub span: Option<(f64, f64)>,
    /// 畳んだグループ行にだけ入る、子キーの集約表示。
    pub agg: Vec<f64>,
    /// ドラッグ書き戻し(`Intent::SetTiming`)の宛先。fixture行のみSome。
    pub layer: Option<LayerId>,
    /// レイヤー差し色。バンドの塗り。
    pub color: [u8; 3],
}

#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    Move,
    TrimStart,
    TrimEnd,
}

struct DragState {
    row: usize,
    layer: LayerId,
    orig: LayerTiming,
    grab_sec: f64,
    delta_sec: f64,
    mode: DragMode,
}

/// 表示px基準の端当たり判定幅(hit_testの6px/8pxと同じ生値、sfac倍しない)。
const EDGE_GRAB_PX: f64 = 6.0;

/// Move で帯をずらした量だけ、その層の全 property track のキー時刻を同じ量ずらす
/// `SetTrack` intent 群を作る。キー時刻は comp 絶対(`start` はローカル)なので、
/// 帯だけ動かすとアニメーションが元の時刻に取り残される。
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

pub enum TimelineMsg {
    SetRows(Vec<CanvasRow>),
    ScrollBy(f64),
}

pub struct TimelineWidget {
    tx: Sender<TimelineMsg>,
    rx: Receiver<TimelineMsg>,
    rows: Vec<CanvasRow>,
    /// 論理px/秒。wheelでカーソルの時刻を固定点にzoomする。
    pps: f64,
    /// 表示窓の左端が指す秒。
    scroll_sec: f64,
    /// 表示窓の上端が指すpx(sfacスケール済、rows描画と同じ単位)。
    scroll_y: f64,
    /// 直近paintの表示窓高さ(sfacスケール済)。スクロール上限の計算に使う。
    viewport_h: f64,
    /// 要素ローカルの論理px。
    cursor: Option<(f64, f64)>,
    hovered: Option<(usize, usize)>,
    selected: Option<(usize, usize)>,
    clock: Option<Arc<Clock>>,
    scale: Option<Arc<UiScale>>,
    doc: Option<Arc<Mutex<Document>>>,
    extractor: Option<fn(&Document) -> Vec<CanvasRow>>,
    drag: Option<DragState>,
    scrubbing: bool,
    selection: Option<Selection>,
    selected_mirror: Option<Signal<Option<LayerId>>>,
    scroll_y_mirror: Option<Signal<f64>>,
}

impl TimelineWidget {
    pub fn new(rows: Vec<CanvasRow>) -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            rows,
            pps: PX_PER_SEC,
            scroll_sec: 0.0,
            scroll_y: 0.0,
            viewport_h: 0.0,
            cursor: None,
            hovered: None,
            selected: None,
            clock: None,
            scale: None,
            doc: None,
            extractor: None,
            drag: None,
            scrubbing: false,
            selection: None,
            selected_mirror: None,
            scroll_y_mirror: None,
        }
    }

    /// 層選択の唯一の真実と、chrome側再描画のためのミラーSignal。
    pub fn with_selection(mut self, selection: Selection, mirror: Signal<Option<LayerId>>) -> Self {
        self.selection = Some(selection);
        self.selected_mirror = Some(mirror);
        self
    }

    /// scroll_yの唯一の真実(self)と、layers列再描画のためのミラーSignal。
    pub fn with_scroll_mirror(mut self, mirror: Signal<f64>) -> Self {
        self.scroll_y_mirror = Some(mirror);
        self
    }

    /// 60秒docが初期表示で収まるようzoomも引いておく。
    pub fn with_clock(mut self, clock: Arc<Clock>) -> Self {
        self.clock = Some(clock);
        self.pps = 20.0;
        self
    }

    /// chrome側の--sと同じ値。行高・ルーラー高をこの倍率で描く。
    pub fn with_scale(mut self, scale: Arc<UiScale>) -> Self {
        self.scale = Some(scale);
        self
    }

    fn sfac(&self) -> f64 {
        self.scale.as_ref().map(|s| s.factor()).unwrap_or(1.0)
    }

    /// 書き戻し先のDocumentと、apply後に行を読み直す抽出関数。
    pub fn with_document(
        mut self,
        doc: Arc<Mutex<Document>>,
        extractor: fn(&Document) -> Vec<CanvasRow>,
    ) -> Self {
        self.doc = Some(doc);
        self.extractor = Some(extractor);
        self
    }

    /// rowsの縦スクロールが動ける上限(sfacスケール済px)。0行やviewportが未知なら0。
    fn max_scroll_y(&self) -> f64 {
        let rowh = ROW_H * self.sfac();
        let content_h = self.rows.len() as f64 * rowh;
        let avail_h = (self.viewport_h - RULER_H * self.sfac()).max(0.0);
        (content_h - avail_h).max(0.0)
    }

    fn set_scroll_y(&mut self, y: f64) {
        self.scroll_y = y.clamp(0.0, self.max_scroll_y());
        if let Some(mirror) = &mut self.scroll_y_mirror {
            mirror.set(self.scroll_y);
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

    pub fn sender(&self) -> Sender<TimelineMsg> {
        self.tx.clone()
    }

    fn process_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                TimelineMsg::SetRows(rows) => self.rows = rows,
                TimelineMsg::ScrollBy(dy) => self.set_scroll_y(self.scroll_y + dy),
            }
        }
    }
}

fn attrs_to_patch(a: &LayerAttrs) -> LayerAttrsPatch {
    LayerAttrsPatch {
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

/// 選択層を `comp_frame` で2本へ割る(Split)。頭は `layer` のまま尺が縮み、尻は
/// 新しい layer になる。`layer` の timing が `comp_frame` を覆っていなければ何もしない
/// (押しても割れないと分かる = Q3 の拒否の報酬、呼び手が戻り値で判定する)。
/// `apply_all` 1回 = 1 undo(`Document::group_layers` と同じ形)。
pub fn split_layer(doc: &Arc<Mutex<Document>>, layer: LayerId, comp_frame: i64) -> Option<LayerId> {
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
                    // カーソル下の時刻を動かさない: scroll = t_cursor - x/pps
                    self.scroll_sec = (cursor_sec - cursor_x / new_pps).max(0.0);
                } else {
                    self.set_scroll_y(self.scroll_y + dy);
                }
                self.scroll_sec = (self.scroll_sec - dx / self.pps).max(0.0);
                if let Some((cx, cy)) = self.cursor {
                    self.hovered = self.hit_test(cx, cy);
                }
            }
            UiEvent::PointerMove(p) => {
                let (x, y) = (p.element.x as f64, p.element.y as f64);
                self.cursor = Some((x, y));
                if self.scrubbing {
                    if let Some(clock) = &self.clock {
                        clock.seek(self.scroll_sec + x / self.pps);
                    }
                } else if let Some(drag) = &mut self.drag {
                    drag.delta_sec = (self.scroll_sec + x / self.pps) - drag.grab_sec;
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
                if hit.is_some() {
                    self.selected = if self.selected == hit { None } else { hit };
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
                            orig,
                            grab_sec: t,
                            delta_sec: 0.0,
                            mode,
                        });
                    }
                }
            }
            UiEvent::PointerUp(_) => {
                self.scrubbing = false;
                if let Some(drag) = self.drag.take() {
                    let (Some(doc), Some(extractor)) = (self.doc.as_ref(), self.extractor) else {
                        return;
                    };
                    let mut doc = doc.lock().unwrap();
                    let raw_delta = (drag.delta_sec * DOC_FPS).round() as i64;
                    let timing = match drag.mode {
                        DragMode::Move => {
                            let new_start = (drag.orig.start + raw_delta).max(0);
                            LayerTiming { start: new_start, ..drag.orig }
                        }
                        DragMode::TrimStart => {
                            // 頭を右へ削る(delta>0) = 素材の入りが進む: source_inも同じだけ動く。
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
            // ドラッグ中の行の帯だけtransientにずらす。Documentはreleaseまで触らない。
            // キーはずらさない — キーフレームの時刻はcomp絶対で、timingに従わない。
            let (shift_a, shift_b) = match &self.drag {
                Some(d) if d.row == i => match d.mode {
                    DragMode::Move => (d.delta_sec, d.delta_sec),
                    DragMode::TrimStart => (d.delta_sec, 0.0),
                    DragMode::TrimEnd => (0.0, d.delta_sec),
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

            // sceneはelement境界でクリップされない — 全図形をローカル0..w/0..hへ自前で抑える。
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
            for (ki, kf) in row.keys.iter().enumerate() {
                let center = Point::new(x_of(*kf), mid);
                if center.x < 0.0 || center.x > w {
                    continue;
                }
                if self.selected == Some((i, ki)) {
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

        let playhead_sec = self
            .clock
            .as_ref()
            .map(|c| c.now_sec())
            .unwrap_or(PLAYHEAD_SEC);
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
mod keyframe_shift_tests {
    use super::*;
    use motolii_store::{Composition, Interp, Keyframe, LayerSource, PropertyId, Value};

    /// 帯(Move)を動かした量だけ、そのレイヤーのキーフレーム時刻も追従する不変量。
    /// 動かす前に comp フレーム `t` にあったキーは、動かした後 `t + delta` にある。
    #[test]
    fn move_shifts_keyframes_by_the_same_delta() {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();

        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Solid { rgba: [255, 0, 0, 255], width: 64, height: 64 },
                    order: 0,
                    timing: LayerTiming { start: 10, duration: 50, source_in: 0, ..Default::default() },
                },
            },
        ])
        .unwrap();

        let property = PropertyId::new("opacity").unwrap();
        let mut track = KeyframeTrack::new();
        track.insert(Keyframe {
            t: RationalTime::try_from_frame(15, Fps::try_new(30, 1).unwrap()).unwrap(),
            value: Value::F64(1.0),
            interp: Interp::Linear,
            spatial: None,
        });
        doc.apply(Intent::SetTrack { layer, property: property.clone(), track }).unwrap();

        // 帯を start=10 -> 40 へ動かす(delta = +30 フレーム)。
        let orig = doc.view().track(layer, &property).unwrap().unwrap();
        let orig_key_t = orig.keys()[0].t;
        let delta_frames = 30;

        let mut intents =
            vec![Intent::SetTiming { layer, timing: LayerTiming { start: 40, duration: 50, source_in: 0, ..Default::default() } }];
        intents.extend(keyframe_shift_intents(&doc, layer, delta_frames).unwrap());
        doc.apply_all(intents).unwrap();

        let shifted = doc.view().track(layer, &property).unwrap().unwrap();
        let expect = orig_key_t
            .try_add(RationalTime::try_from_frame(delta_frames, Fps::try_new(30, 1).unwrap()).unwrap())
            .unwrap();
        assert_eq!(
            shifted.keys()[0].t,
            expect,
            "帯を動かした量だけキーフレームが追従していない(親を動かしたら子も追従、が破れている)"
        );
    }
}
