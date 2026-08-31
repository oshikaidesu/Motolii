
use std::sync::{Arc, Mutex};

use anyrender::{PaintRef, PaintScene};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::UiEvent;
use peniko::kurbo::{Affine, BezPath, Circle, Point, Rect, Stroke};
use peniko::{Color, Fill};

use crate::ui::tokens;

/// 区間のイージング。CSS の cubic-bezier と同じ4つ(両端は (0,0) と (1,1) で固定)。
pub(super) type Curve = [f64; 4];

pub(super) const LINEAR: Curve = [0.0, 0.0, 1.0, 1.0];

const PAD: f64 = 24.0;
const GRAB: f64 = 14.0;

/// 曲線を掴んで曲げる盤。値は窓と共有(パネル側のボタンも同じ物を書く)。
pub(super) struct EaseWidget {
    curve: Arc<Mutex<Curve>>,
    size: (f64, f64),
    holding: Option<usize>,
}

impl EaseWidget {
    pub(super) fn new(curve: Arc<Mutex<Curve>>) -> Self {
        Self { curve, size: (0.0, 0.0), holding: None }
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
            UiEvent::PointerUp(_) => self.holding = None,
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

        let t3 = |c: [u8; 3]| Color::from_rgb8(c[0], c[1], c[2]);
        let ink = t3(tokens::INK);
        let dim = t3(tokens::INK3);
        let accent = t3(tokens::ACCENT);

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
