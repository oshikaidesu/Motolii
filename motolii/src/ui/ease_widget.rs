
use std::sync::{Arc, Mutex};

use anyrender::{PaintRef, PaintScene};
use dioxus_native::prelude::{Signal, WritableExt};
use blitz_dom::node::ComputedStyles;
use blitz_dom::Widget;
use blitz_traits::events::UiEvent;
use peniko::kurbo::{Affine, BezPath, Circle, Line, Point, Rect, Stroke};
use peniko::{Color, Fill};

use crate::doc::store::Interp;
use crate::ui::ease_model::{handles, hold_in_range, overshoots, KINDS};
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
    seen_cancel: u32,
}

impl EaseWidget {
    pub(super) fn new(shape: Arc<Mutex<Interp>>, session: Session) -> Self {
        Self { shape, session, size: (0.0, 0.0), holding: None, showing: None, seen_cancel: 0 }
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

    fn cancel_hold(&mut self) {
        self.holding = None;
        self.showing = None;
        self.session.gesture.end();
    }

    fn sync_cancel(&mut self) -> bool {
        if !self.session.gesture.cancelled(&mut self.seen_cancel) {
            return false;
        }
        self.cancel_hold();
        true
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

    /// 盤の右上に置く2つの小さな道具。左=写す、右=枠の外へ出す許し。
    fn tools(&self) -> [Rect; 2] {
        let top = PAD * 0.5;
        let size = 14.0;
        let right = self.size.0 - PAD * 0.5;
        [
            Rect::new(right - size * 2.0 - 6.0, top, right - size - 6.0, top + size),
            Rect::new(right - size, top, right, top + size),
        ]
    }

    fn view(&self) -> (f64, f64) {
        let free = self
            .session
            .overshoot
            .load(std::sync::atomic::Ordering::Relaxed);
        if free || overshoots(*self.shape.lock().unwrap()) {
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
        if self.sync_cancel() {
            return;
        }
        match event {
            UiEvent::PointerDown(p) => {
                let point = Point::new(p.element.x as f64, p.element.y as f64);
                let [copy, free] = self.tools();
                if copy.contains(point) {
                    let starts = self.starts();
                    if let Some(first) = starts.first() {
                        let shape = crate::ui::ease::shape_of(&self.session, first);
                        *self.session.curve_clip.lock().unwrap() = Some(shape);
                    }
                    return;
                }
                if free.contains(point) {
                    let now = self
                        .session
                        .overshoot
                        .load(std::sync::atomic::Ordering::Relaxed);
                    self.session
                        .overshoot
                        .store(!now, std::sync::atomic::Ordering::Relaxed);
                    return;
                }
                let shape = *self.shape.lock().unwrap();
                self.holding = handles(shape)
                    .iter()
                    .position(|h| self.to_px(h.at.0, h.at.1).distance(point) <= GRAB);
                if self.holding.is_some() {
                    self.session.gesture.begin();
                }
            }
            UiEvent::PointerMove(p) => {
                let Some(which) = self.holding else { return };
                let (cu, cv) = self.to_curve(p.element.x as f64, p.element.y as f64);
                let mut shape = self.shape.lock().unwrap();
                let Some(handle) = handles(*shape).into_iter().nth(which) else {
                    return;
                };
                let free = self
                    .session
                    .overshoot
                    .load(std::sync::atomic::Ordering::Relaxed);
                *shape = hold_in_range((handle.moved)(*shape, (cu, cv)), free);
            }
            UiEvent::PointerUp(_) => {
                if self.holding.take().is_some() {
                    self.session.gesture.end();
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
        self.sync_cancel();
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

        // 道具。写す = 札が2枚重なった形、外へ出す = 箱の上下へ伸びた形。
        let [copy, free] = self.tools();
        let thin = Stroke::new(1.0 / k);
        let held = self.session.curve_clip.lock().unwrap().is_some();
        let copy_ink = if held { accent } else { dim };
        s.stroke(&thin, at, PaintRef::Solid(copy_ink), None, &copy.inset(-4.0));
        s.stroke(
            &thin,
            at,
            PaintRef::Solid(copy_ink),
            None,
            &copy.inset(-4.0).with_origin((copy.x0 + 4.0, copy.y0 + 4.0)),
        );

        let open = self
            .session
            .overshoot
            .load(std::sync::atomic::Ordering::Relaxed);
        let free_ink = if open { accent } else { dim };
        let mid = Rect::new(free.x0 + 2.0, free.y0 + 5.0, free.x1 - 2.0, free.y1 - 5.0);
        s.stroke(&thin, at, PaintRef::Solid(free_ink), None, &mid);
        if open {
            for y in [free.y0 + 1.0, free.y1 - 1.0] {
                s.stroke(
                    &thin,
                    at,
                    PaintRef::Solid(free_ink),
                    None,
                    &Line::new(Point::new(free.x0, y), Point::new(free.x1, y)),
                );
            }
        }
        s
    }
}

#[cfg(test)]
#[test]
fn focus_loss_cancels_an_ease_handle_without_committing_it() {
    let loaded = crate::ui::fixture::load_fixture();
    let session = Session::new(loaded.doc, loaded.duration_sec, loaded.ui);
    let shape = Arc::new(Mutex::new(DEFAULT));
    let mut widget = EaseWidget::new(shape, session.clone());
    widget.holding = Some(0);
    widget.showing = Some(Vec::new());
    session.gesture.begin();

    assert!(session.gesture.cancel());
    assert!(widget.sync_cancel());
    assert!(widget.holding.is_none());
    assert!(widget.showing.is_none());
}

const COLS: usize = 4;
const CELL_PAD: f64 = 6.0;

/// 型の名前。棚は形で見せるが、**説明書が名前で指す物には名前が要る**。
/// 乗せた時だけ出す(常に置くと8つの字が並んで形が読めなくなる)。
pub(super) fn kind_name(interp: &Interp) -> &'static str {
    match interp {
        Interp::Hold => "Hold",
        Interp::Linear => "Linear",
        Interp::Bezier { .. } => "Ease",
        Interp::Bounce { .. } => "Bounce",
        Interp::Elastic { .. } => "Elastic",
        Interp::Cyclic { .. } => "Cyclic",
        Interp::Random { .. } => "Random",
        Interp::Steps { .. } => "Steps",
        Interp::ElasticSteps { .. } => "Elastic Steps",
    }
}

/// 型の棚。押すとその型がそのまま区間へ乗る。名前は置かず、形で見せる。
pub(super) struct KindsWidget {
    shape: Arc<Mutex<Interp>>,
    session: Session,
    size: (f64, f64),
    hovered: Option<usize>,
    /// 指している型の名前を窓へ返す鏡。窓は字を描けるが、盤は形しか描けない。
    name_mirror: Option<Signal<String>>,
}

impl KindsWidget {
    pub(super) fn new(shape: Arc<Mutex<Interp>>, session: Session) -> Self {
        Self { shape, session, size: (0.0, 0.0), hovered: None, name_mirror: None }
    }

    pub(super) fn with_name_mirror(mut self, mirror: Signal<String>) -> Self {
        self.name_mirror = Some(mirror);
        self
    }

    /// 棚に並ぶ形。最後に「写した曲線」が居ることがある。
    fn shelf(&self) -> Vec<Interp> {
        let mut out = KINDS.to_vec();
        if let Some(clip) = *self.session.curve_clip.lock().unwrap() {
            out.push(clip);
        }
        out
    }

    fn cell(&self, i: usize) -> Rect {
        let rows = self.shelf().len().div_ceil(COLS);
        let w = self.size.0 / COLS as f64;
        let h = self.size.1 / rows.max(1) as f64;
        let (cx, cy) = (i % COLS, i / COLS);
        Rect::new(cx as f64 * w, cy as f64 * h, (cx + 1) as f64 * w, (cy + 1) as f64 * h)
    }

    fn at(&self, x: f64, y: f64) -> Option<usize> {
        (0..self.shelf().len()).find(|i| self.cell(*i).contains(Point::new(x, y)))
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
                let name = self
                    .hovered
                    .and_then(|i| self.shelf().get(i).map(|k| kind_name(k).to_owned()))
                    .unwrap_or_default();
                if let Some(mirror) = &mut self.name_mirror {
                    if (*mirror)() != name {
                        mirror.set(name);
                    }
                }
            }
            UiEvent::PointerDown(p) => {
                let Some(i) = self.at(p.element.x as f64, p.element.y as f64) else {
                    return;
                };
                let Some(chosen) = self.shelf().get(i).copied() else { return };
                *self.shape.lock().unwrap() = chosen;
                let starts =
                    crate::ui::ease::segments(&self.session.selected_keys.lock().unwrap());
                if starts.is_empty() {
                    return;
                }
                match crate::ui::ease::apply(&self.session, &starts, chosen) {
                    Ok(n) => println!(
                        "PROBE room=write verdict=applied Ease kind={} tracks={n}",
                        chosen.kind()
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

        let shelf = self.shelf();
        let copied_at = shelf.len().saturating_sub(1);
        let has_clip = self.session.curve_clip.lock().unwrap().is_some();
        for (i, kind) in shelf.iter().copied().enumerate() {
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

            // 写した曲線の札には角へ印を置く(棚の物と見分けるため)。
            if has_clip && i == copied_at {
                let mark = Rect::from_origin_size(
                    (cell.x0 + 3.0, cell.y0 + 3.0),
                    (4.0, 4.0),
                );
                s.fill(Fill::NonZero, at, PaintRef::Solid(t3(tokens::ACCENT)), None, &mark);
            }
        }
        s
    }
}
