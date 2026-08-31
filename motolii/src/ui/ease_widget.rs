
use std::sync::{Arc, Mutex};

use anyrender::{PaintRef, PaintScene};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::UiEvent;
use peniko::kurbo::{Affine, BezPath, Circle, Point, Rect, Stroke};
use peniko::{Color, Fill};

use crate::ui::session::{KeySel, Session};
use crate::ui::tokens;

/// 区間のイージング。CSS の cubic-bezier と同じ4つ(両端は (0,0) と (1,1) で固定)。
pub(super) type Curve = [f64; 4];

pub(super) const LINEAR: Curve = [0.0, 0.0, 1.0, 1.0];

const PAD: f64 = 24.0;
const GRAB: f64 = 14.0;

/// 曲線を掴んで曲げる盤。離した時にその形が区間へ乗る(押す手数を作らない)。
pub(super) struct EaseWidget {
    curve: Arc<Mutex<Curve>>,
    session: Session,
    size: (f64, f64),
    holding: Option<usize>,
    /// 前に見ていた区間。変わったらその区間が持っている形を読み直す。
    showing: Option<Vec<KeySel>>,
}

impl EaseWidget {
    pub(super) fn new(curve: Arc<Mutex<Curve>>, session: Session) -> Self {
        Self { curve, session, size: (0.0, 0.0), holding: None, showing: None }
    }

    fn starts(&self) -> Vec<KeySel> {
        crate::ui::ease::segments(&self.session.selected_keys.lock().unwrap())
    }

    /// 掴んでいる区間が変わったら、その区間が今持っている形を盤へ映す。
    fn follow_selection(&mut self) {
        let starts = self.starts();
        if self.showing.as_ref() == Some(&starts) {
            return;
        }
        if let Some(first) = starts.first() {
            *self.curve.lock().unwrap() = crate::ui::ease::curve_of(&self.session, first);
        }
        self.showing = Some(starts);
    }

    fn commit(&self) {
        let starts = self.starts();
        if starts.is_empty() {
            return;
        }
        let curve = *self.curve.lock().unwrap();
        match crate::ui::ease::apply(&self.session, &starts, curve) {
            Ok(n) => println!("PROBE room=write verdict=applied Ease tracks={n}"),
            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
        }
    }

    /// 盤の中の位置(px)を曲線の値(0..1)へ。y は上が 1。
    fn to_curve(&self, x: f64, y: f64) -> (f64, f64) {
        let (w, h) = (self.size.0 - PAD * 2.0, self.size.1 - PAD * 2.0);
        if w <= 0.0 || h <= 0.0 {
            return (0.0, 0.0);
        }
        (((x - PAD) / w).clamp(0.0, 1.0), 1.0 - (y - PAD) / h)
    }

    fn to_px(&self, cx: f64, cy: f64) -> Point {
        let (w, h) = (self.size.0 - PAD * 2.0, self.size.1 - PAD * 2.0);
        Point::new(PAD + cx * w, PAD + (1.0 - cy) * h)
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
        let point = |p: &blitz_traits::events::BlitzPointerEvent| {
            (p.element.x as f64, p.element.y as f64)
        };
        match event {
            UiEvent::PointerDown(p) => {
                let (x, y) = point(p);
                let c = *self.curve.lock().unwrap();
                let a = self.to_px(c[0], c[1]);
                let b = self.to_px(c[2], c[3]);
                self.holding = if a.distance(Point::new(x, y)) <= GRAB {
                    Some(0)
                } else if b.distance(Point::new(x, y)) <= GRAB {
                    Some(1)
                } else {
                    None
                };
            }
            UiEvent::PointerMove(p) => {
                let Some(which) = self.holding else { return };
                let (x, y) = point(p);
                let (cx, cy) = self.to_curve(x, y);
                let mut c = self.curve.lock().unwrap();
                c[which * 2] = cx;
                c[which * 2 + 1] = cy;
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

        let zero = self.to_px(0.0, 0.0);
        let one = self.to_px(1.0, 1.0);
        let frame = Rect::from_points(zero, one);
        s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(dim), None, &frame);

        let mut diagonal = BezPath::new();
        diagonal.move_to(zero);
        diagonal.line_to(one);
        s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(dim), None, &diagonal);

        let c = *self.curve.lock().unwrap();
        let a = self.to_px(c[0], c[1]);
        let b = self.to_px(c[2], c[3]);

        for handle in [(zero, a), (one, b)] {
            let mut line = BezPath::new();
            line.move_to(handle.0);
            line.line_to(handle.1);
            s.stroke(&Stroke::new(1.0 / k), at, PaintRef::Solid(dim), None, &line);
        }

        let mut curve = BezPath::new();
        curve.move_to(zero);
        curve.curve_to(a, b, one);
        s.stroke(&Stroke::new(2.0 / k), at, PaintRef::Solid(ink), None, &curve);

        for handle in [a, b] {
            s.fill(
                Fill::NonZero,
                at,
                PaintRef::Solid(accent),
                None,
                &Circle::new(handle, 5.0),
            );
        }
        s
    }
}

/// よく使う形。Flow と同じで、名前ではなく**形そのもの**を並べる。
pub(super) const PRESETS: &[Curve] = &[
    LINEAR,
    [0.42, 0.0, 1.0, 1.0],
    [0.0, 0.0, 0.58, 1.0],
    [0.42, 0.0, 0.58, 1.0],
    [0.77, 0.0, 0.175, 1.0],
    [0.165, 0.84, 0.44, 1.0],
    [0.68, -0.55, 0.265, 1.55],
    [0.175, 0.885, 0.32, 1.275],
];

const COLS: usize = 4;
const CELL_PAD: f64 = 6.0;

/// 形を並べた棚。押すとその形が盤と区間へ乗る。
pub(super) struct PresetsWidget {
    curve: Arc<Mutex<Curve>>,
    session: Session,
    size: (f64, f64),
    hovered: Option<usize>,
}

impl PresetsWidget {
    pub(super) fn new(curve: Arc<Mutex<Curve>>, session: Session) -> Self {
        Self { curve, session, size: (0.0, 0.0), hovered: None }
    }

    fn cell(&self, i: usize) -> Rect {
        let rows = PRESETS.len().div_ceil(COLS);
        let w = self.size.0 / COLS as f64;
        let h = self.size.1 / rows.max(1) as f64;
        let (cx, cy) = (i % COLS, i / COLS);
        Rect::new(cx as f64 * w, cy as f64 * h, (cx + 1) as f64 * w, (cy + 1) as f64 * h)
    }

    fn at(&self, x: f64, y: f64) -> Option<usize> {
        (0..PRESETS.len()).find(|i| self.cell(*i).contains(Point::new(x, y)))
    }
}

impl Widget for PresetsWidget {
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
                *self.curve.lock().unwrap() = PRESETS[i];
                let starts =
                    crate::ui::ease::segments(&self.session.selected_keys.lock().unwrap());
                if starts.is_empty() {
                    return;
                }
                match crate::ui::ease::apply(&self.session, &starts, PRESETS[i]) {
                    Ok(n) => println!("PROBE room=write verdict=applied Ease tracks={n}"),
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
        let current = *self.curve.lock().unwrap();

        for (i, preset) in PRESETS.iter().enumerate() {
            let cell = self.cell(i);
            let chosen = preset
                .iter()
                .zip(current.iter())
                .all(|(a, b)| (a - b).abs() < 1e-6);
            let bg = if chosen {
                t3(tokens::SURFACE_RAISED)
            } else if self.hovered == Some(i) {
                t3(tokens::SURFACE_HOVER)
            } else {
                t3(tokens::SURFACE_APP)
            };
            s.fill(Fill::NonZero, at, PaintRef::Solid(bg), None, &cell.inset(-0.5));

            let box_ = cell.inset(-CELL_PAD);
            let zero = Point::new(box_.x0, box_.y1);
            let one = Point::new(box_.x1, box_.y0);
            let ctrl = |cx: f64, cy: f64| {
                Point::new(box_.x0 + cx * box_.width(), box_.y1 - cy * box_.height())
            };
            let mut curve = BezPath::new();
            curve.move_to(zero);
            curve.curve_to(ctrl(preset[0], preset[1]), ctrl(preset[2], preset[3]), one);
            let ink = if chosen { t3(tokens::ACCENT) } else { t3(tokens::INK2) };
            s.stroke(&Stroke::new(1.5 / k), at, PaintRef::Solid(ink), None, &curve);
        }
        s
    }
}
