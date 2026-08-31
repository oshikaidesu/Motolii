
use std::sync::{Arc, Mutex};

use anyrender::{PaintRef, PaintScene};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::UiEvent;
use peniko::kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Stroke};
use peniko::{Color, Fill};

use crate::doc::store::Interp;
use crate::ui::ease_model::{handles, overshoots, KINDS};
use crate::ui::session::{KeySel, Session};
use crate::ui::tokens;

/// 盤の縦の見え幅。**曲線に合わせて動かさない**(掴んでいる間に写像が変わると
/// 手が滑るため。2026-07-19 台帳の採用差分)。
const STANDARD_VIEW: (f64, f64) = (-0.35, 1.35);
const OVERSHOOT_VIEW: (f64, f64) = (-0.5, 2.2);

const PAD: f64 = 18.0;
const GRAB: f64 = 12.0;

pub(super) const DEFAULT: Interp = Interp::Linear;

/// 曲線を掴んで曲げる盤。離した時にその形が区間へ乗る。
pub(super) struct EaseWidget {
    shape: Arc<Mutex<Interp>>,
    session: Session,
    size: (f64, f64),
    holding: Option<usize>,
    showing: Option<Vec<KeySel>>,
}

impl EaseWidget {
    pub(super) fn new(shape: Arc<Mutex<Interp>>, session: Session) -> Self {
        Self { shape, session, size: (0.0, 0.0), holding: None, showing: None }
    }

    fn starts(&self) -> Vec<KeySel> {
        crate::ui::ease::segments(&self.session.selected_keys.lock().unwrap())
    }

    fn follow_selection(&mut self) {
        let starts = self.starts();
        if self.showing.as_ref() == Some(&starts) {
            return;
        }
        if let Some(first) = starts.first() {
            *self.shape.lock().unwrap() = crate::ui::ease::shape_of(&self.session, first);
        }
        self.showing = Some(starts);
    }

    fn commit(&self) {
        let starts = self.starts();
        if starts.is_empty() {
            return;
        }
        let shape = *self.shape.lock().unwrap();
        match crate::ui::ease::apply(&self.session, &starts, shape) {
            Ok(n) => println!("PROBE room=write verdict=applied Ease kind={} tracks={n}", shape.kind()),
            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
        }
    }

    /// 再生位置が区間の中のどこか。外に居るなら None。
    fn playhead_u(&self) -> Option<f64> {
        let starts = self.starts();
        let (a, b) = crate::ui::ease::span_of(&self.session, starts.first()?)?;
        if b <= a {
            return None;
        }
        let now = self.session.clock.now_sec();
        let u = (now - a) / (b - a);
        (0.0..=1.0).contains(&u).then_some(u)
    }

    fn view(&self) -> (f64, f64) {
        if overshoots(*self.shape.lock().unwrap()) {
            OVERSHOOT_VIEW
        } else {
            STANDARD_VIEW
        }
    }

    /// 盤は**正方**にする。横長の箱に引き伸ばすと、同じ傾きが型ごとに
    /// 違って見えて読めない(実機の plot も正方に近い)。
    fn plot(&self) -> Rect {
        let side = (self.size.0.min(self.size.1) - PAD * 2.0).max(1.0);
        let x0 = (self.size.0 - side) / 2.0;
        let y0 = (self.size.1 - side) / 2.0;
        Rect::new(x0, y0, x0 + side, y0 + side)
    }

    fn to_curve(&self, x: f64, y: f64) -> (f64, f64) {
        let plot = self.plot();
        if plot.width() <= 0.0 || plot.height() <= 0.0 {
            return (0.0, 0.0);
        }
        let (lo, hi) = self.view();
        (
            ((x - plot.x0) / plot.width()).clamp(-0.2, 1.2),
            hi - (y - plot.y0) / plot.height() * (hi - lo),
        )
    }

    fn to_px(&self, cu: f64, cv: f64) -> Point {
        let plot = self.plot();
        let (lo, hi) = self.view();
        Point::new(
            plot.x0 + cu * plot.width(),
            plot.y0 + (hi - cv) / (hi - lo) * plot.height(),
        )
    }
}

impl Widget for EaseWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}
    fn can_create_surfaces(&mut self, _ctx: &mut dyn anyrender::RenderContext) {}
    fn destroy_surfaces(&mut self) {}

    fn requires_redraw(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent) {
        match event {
            UiEvent::PointerDown(p) => {
                let point = Point::new(p.element.x as f64, p.element.y as f64);
                let shape = *self.shape.lock().unwrap();
                self.holding = handles(shape)
                    .iter()
                    .position(|h| self.to_px(h.at.0, h.at.1).distance(point) <= GRAB);
            }
            UiEvent::PointerMove(p) => {
                let Some(which) = self.holding else { return };
                let (cu, cv) = self.to_curve(p.element.x as f64, p.element.y as f64);
                let mut shape = self.shape.lock().unwrap();
                let Some(handle) = handles(*shape).into_iter().nth(which) else {
                    return;
                };
                *shape = (handle.moved)(*shape, (cu, cv));
            }
            UiEvent::PointerUp(_) => {
                if self.holding.take().is_some() {
                    self.commit();
                    self.showing = None;
                }
            }
            _ => {}
        }
    }

    fn paint(
        &mut self,
        _ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> anyrender::Scene {
        let mut s = anyrender::Scene::new();
        if width == 0 || height == 0 {
            return s;
        }
        // paint は物理 px、イベントは論理 px。盤の寸法は論理で持ち、描く時に掛ける。
        let k = scale.max(0.001);
        self.size = (width as f64 / k, height as f64 / k);
        let at = Affine::scale(k);

        if self.holding.is_none() {
            self.follow_selection();
        }
        let live = !self.starts().is_empty();
        let shape = *self.shape.lock().unwrap();

        let t3 = |c: [u8; 3]| Color::from_rgb8(c[0], c[1], c[2]);
        let ink = if live { t3(tokens::INK) } else { t3(tokens::INK3) };
        let dim = t3(tokens::INK3);
        let accent = if live { t3(tokens::ACCENT) } else { t3(tokens::INK3) };

        s.fill(
            Fill::NonZero,
            at,
            PaintRef::Solid(t3(tokens::SURFACE_APP)),
            None,
            &Rect::new(0.0, 0.0, self.size.0, self.size.1),
        );

        // 案内は v=0 と v=1 の2本だけ(実機にも縦線は無い)。
        // 目盛りは薄く4分割。読む線(v=0 / v=1)はその上に強く引く。
        let grid = Color::from_rgba8(0x75, 0x75, 0x75, 0x44);
        for i in 1..4 {
            let f = i as f64 / 4.0;
            let (a, b) = (self.to_px(f, 0.0), self.to_px(f, 1.0));
            s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(grid), None, &Line::new(a, b));
            let (a, b) = (self.to_px(0.0, f), self.to_px(1.0, f));
            s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(grid), None, &Line::new(a, b));
        }
        for v in [0.0, 1.0] {
            let a = self.to_px(0.0, v);
            let b = self.to_px(1.0, v);
            s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(dim), None, &Line::new(a, b));
        }
        for u in [0.0, 1.0] {
            let a = self.to_px(u, 0.0);
            let b = self.to_px(u, 1.0);
            s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(dim), None, &Line::new(a, b));
        }

        let mut curve = BezPath::new();
        const SAMPLES: usize = 240;
        for i in 0..=SAMPLES {
            let u = i as f64 / SAMPLES as f64;
            let p = self.to_px(u, shape.ease(u));
            if i == 0 {
                curve.move_to(p);
            } else {
                curve.line_to(p);
            }
        }
        s.stroke(&Stroke::new(2.0 / k), at, PaintRef::Solid(ink), None, &curve);

        for (zero_or_one, v) in [(0.0, 0.0), (1.0, 1.0)] {
            s.fill(
                Fill::NonZero,
                at,
                PaintRef::Solid(dim),
                None,
                &Circle::new(self.to_px(zero_or_one, v), 3.0),
            );
        }

        // 今どこを再生しているか。区間の中に居る時だけ立てる。
        if let Some(u) = self.playhead_u() {
            let top = self.to_px(u, self.view().1);
            let bottom = self.to_px(u, self.view().0);
            let mark = t3(tokens::WAY_TIMELINE);
            s.stroke(
                &Stroke::new(1.0 / k),
                at,
                PaintRef::Solid(mark),
                None,
                &Line::new(top, bottom),
            );
            s.fill(
                Fill::NonZero,
                at,
                PaintRef::Solid(mark),
                None,
                &Circle::new(self.to_px(u, shape.ease(u)), 4.0),
            );
        }

        for handle in handles(shape) {
            let p = self.to_px(handle.at.0, handle.at.1);
            s.fill(Fill::NonZero, at, PaintRef::Solid(accent), None, &Circle::new(p, 5.0));
        }
        s
    }
}

const COLS: usize = 4;
const CELL_PAD: f64 = 6.0;

/// 型の棚。押すとその型がそのまま区間へ乗る。名前は置かず、形で見せる。
pub(super) struct KindsWidget {
    shape: Arc<Mutex<Interp>>,
    session: Session,
    size: (f64, f64),
    hovered: Option<usize>,
}

impl KindsWidget {
    pub(super) fn new(shape: Arc<Mutex<Interp>>, session: Session) -> Self {
        Self { shape, session, size: (0.0, 0.0), hovered: None }
    }

    fn cell(&self, i: usize) -> Rect {
        let rows = KINDS.len().div_ceil(COLS);
        let w = self.size.0 / COLS as f64;
        let h = self.size.1 / rows.max(1) as f64;
        let (cx, cy) = (i % COLS, i / COLS);
        Rect::new(cx as f64 * w, cy as f64 * h, (cx + 1) as f64 * w, (cy + 1) as f64 * h)
    }

    fn at(&self, x: f64, y: f64) -> Option<usize> {
        (0..KINDS.len()).find(|i| self.cell(*i).contains(Point::new(x, y)))
    }
}

impl Widget for KindsWidget {
    fn connected(&mut self) {}
    fn disconnected(&mut self) {}
    fn can_create_surfaces(&mut self, _ctx: &mut dyn anyrender::RenderContext) {}
    fn destroy_surfaces(&mut self) {}

    fn requires_redraw(&self) -> bool {
        true
    }

    fn handle_event(&mut self, event: &UiEvent) {
        match event {
            UiEvent::PointerMove(p) => {
                self.hovered = self.at(p.element.x as f64, p.element.y as f64);
            }
            UiEvent::PointerDown(p) => {
                let Some(i) = self.at(p.element.x as f64, p.element.y as f64) else {
                    return;
                };
                *self.shape.lock().unwrap() = KINDS[i];
                let starts =
                    crate::ui::ease::segments(&self.session.selected_keys.lock().unwrap());
                if starts.is_empty() {
                    return;
                }
                match crate::ui::ease::apply(&self.session, &starts, KINDS[i]) {
                    Ok(n) => println!(
                        "PROBE room=write verdict=applied Ease kind={} tracks={n}",
                        KINDS[i].kind()
                    ),
                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                }
            }
            _ => {}
        }
    }

    fn paint(
        &mut self,
        _ctx: &mut dyn anyrender::RenderContext,
        _styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> anyrender::Scene {
        let mut s = anyrender::Scene::new();
        if width == 0 || height == 0 {
            return s;
        }
        let k = scale.max(0.001);
        self.size = (width as f64 / k, height as f64 / k);
        let at = Affine::scale(k);

        let t3 = |c: [u8; 3]| Color::from_rgb8(c[0], c[1], c[2]);
        let current = *self.shape.lock().unwrap();

        for (i, kind) in KINDS.iter().copied().enumerate() {
            let cell = self.cell(i);
            let chosen = kind.kind() == current.kind();
            let bg = if chosen {
                t3(tokens::SURFACE_RAISED)
            } else if self.hovered == Some(i) {
                t3(tokens::SURFACE_HOVER)
            } else {
                t3(tokens::SURFACE_APP)
            };
            s.fill(Fill::NonZero, at, PaintRef::Solid(bg), None, &cell.inset(-0.5));

            // 絵札も盤と同じ見え幅で描く。型ごとに縦の縮尺を変えない。
            let (lo, hi) = if overshoots(kind) { OVERSHOOT_VIEW } else { STANDARD_VIEW };
            let box_ = cell.inset(-CELL_PAD);
            let point = |u: f64, v: f64| {
                Point::new(
                    box_.x0 + u * box_.width(),
                    box_.y0 + (hi - v) / (hi - lo) * box_.height(),
                )
            };
            let mut curve = BezPath::new();
            const SAMPLES: usize = 96;
            for j in 0..=SAMPLES {
                let u = j as f64 / SAMPLES as f64;
                let p = point(u, kind.ease(u));
                if j == 0 {
                    curve.move_to(p);
                } else {
                    curve.line_to(p);
                }
            }
            let ink = if chosen { t3(tokens::ACCENT) } else { t3(tokens::INK2) };
            s.stroke(&Stroke::new(1.5 / k), at, PaintRef::Solid(ink), None, &curve);
        }
        s
    }
}
