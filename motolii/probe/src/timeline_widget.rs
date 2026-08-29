use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::playback::Clock;
use crate::tokens::{self, UiScale};
use anyrender::{PaintRef, PaintScene};
use motolii_store::{Document, Intent, LayerId, LayerTiming};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::{BlitzWheelDelta, UiEvent};
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

struct DragState {
    row: usize,
    layer: LayerId,
    orig: LayerTiming,
    grab_sec: f64,
    delta_sec: f64,
}

pub enum TimelineMsg {
    SetRows(Vec<CanvasRow>),
}

pub struct TimelineWidget {
    tx: Sender<TimelineMsg>,
    rx: Receiver<TimelineMsg>,
    rows: Vec<CanvasRow>,
    /// 論理px/秒。wheelでカーソルの時刻を固定点にzoomする。
    pps: f64,
    /// 表示窓の左端が指す秒。
    scroll_sec: f64,
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
            cursor: None,
            hovered: None,
            selected: None,
            clock: None,
            scale: None,
            doc: None,
            extractor: None,
            drag: None,
            scrubbing: false,
        }
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

    fn band_hit(&self, x: f64, y: f64) -> Option<usize> {
        let row_ix = ((y - RULER_H * self.sfac()) / (ROW_H * self.sfac())).floor();
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
        let row_ix = ((y - rh) / rowh).floor();
        if row_ix < 0.0 {
            return None;
        }
        let row_ix = row_ix as usize;
        let row = self.rows.get(row_ix)?;
        let mid_y = rh + row_ix as f64 * rowh + rowh * 0.5;
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
            }
        }
    }
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
                let cursor_x = wheel.element.x as f64;
                let cursor_sec = self.scroll_sec + cursor_x / self.pps;
                let new_pps = (self.pps * (1.0 - dy * 0.002)).clamp(MIN_PPS, MAX_PPS);
                self.pps = new_pps;
                // カーソル下の時刻を動かさない: scroll = t_cursor - x/pps
                self.scroll_sec = (cursor_sec - cursor_x / new_pps).max(0.0);
                self.scroll_sec = (self.scroll_sec - dx / new_pps).max(0.0);
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
                    let orig = layer.and_then(|l| {
                        self.doc
                            .as_ref()
                            .and_then(|d| d.lock().unwrap().view().meta(l).ok().flatten())
                            .map(|m| m.timing)
                    });
                    if let (Some(layer), Some(orig)) = (layer, orig) {
                        println!("PROBE room=write drag-start row={row_ix} start={}", orig.start);
                        self.drag = Some(DragState {
                            row: row_ix,
                            layer,
                            orig,
                            grab_sec: t,
                            delta_sec: 0.0,
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
                    let new_start =
                        ((drag.orig.start as f64 + drag.delta_sec * DOC_FPS).round() as i64).max(0);
                    let timing = LayerTiming { start: new_start, ..drag.orig };
                    match doc.apply(Intent::SetTiming { layer: drag.layer, timing }) {
                        Ok(_) => {
                            println!(
                                "PROBE room=write verdict=applied SetTiming start {}->{}",
                                drag.orig.start, new_start
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

        let hover_row = self
            .cursor
            .map(|(_, cy)| ((cy - RULER_H * self.sfac()) / (ROW_H * self.sfac())).floor())
            .filter(|r| *r >= 0.0)
            .map(|r| r as usize);

        for (i, row) in self.rows.iter().enumerate() {
            let y = ruler_h + i as f64 * row_h;
            if y >= h {
                break;
            }
            let mid = y + row_h * 0.5;
            // ドラッグ中の行の帯だけtransientにずらす。Documentはreleaseまで触らない。
            // キーはずらさない — キーフレームの時刻はcomp絶対で、timingに従わない。
            let shift = match &self.drag {
                Some(d) if d.row == i => d.delta_sec,
                _ => 0.0,
            };

            if row.is_group {
                fill_rect(&mut s, Rect::new(0.0, y, w, y + row_h), c_panel);
            }
            if hover_row == Some(i) {
                fill_rect(
                    &mut s,
                    Rect::new(0.0, y, w, y + row_h),
                    Color::from_rgba8(0xff, 0xff, 0xff, 0x0a),
                );
            }
            fill_rect(&mut s, Rect::new(0.0, y + row_h - hairline, w, y + row_h), c_rowline);

            // sceneはelement境界でクリップされない — 全図形をローカル0..w/0..hへ自前で抑える。
            if let Some((a, b)) = row.span {
                let x0 = x_of(a + shift).max(0.0);
                let x1 = x_of(b + shift).min(w);
                if x1 > x0 {
                    fill_rect(&mut s, Rect::new(x0, y, x1, y + row_h - hairline), c_hair);
                    fill_rect(
                        &mut s,
                        Rect::new(
                            x0 + hairline,
                            y + hairline,
                            x1 - hairline,
                            y + row_h - 2.0 * hairline,
                        ),
                        c(row.color[0], row.color[1], row.color[2]),
                    );
                }
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
