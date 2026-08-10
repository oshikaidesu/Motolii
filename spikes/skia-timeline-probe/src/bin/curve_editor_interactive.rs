use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use skia_safe::{
    AlphaType, Color, ColorType, Contains, EncodedImageFormat, Font, FontMgr, FontStyle, ImageInfo,
    Paint, PaintStyle, PathBuilder, Point, Rect, surfaces,
};
use std::time::{Duration, Instant};

const LEFT: f32 = 190.0;
const TOP: f32 = 38.0;
const STATUS: f32 = 28.0;

#[derive(Clone)]
struct Keyframe {
    time: f32,
    value: f32,
    incoming: [f32; 2],
    outgoing: [f32; 2],
    selected: bool,
}

struct Curve {
    name: &'static str,
    color: Color,
    keys: Vec<Keyframe>,
}

#[derive(Clone, Copy)]
enum Target {
    Key(usize, usize),
    Incoming(usize, usize),
    Outgoing(usize, usize),
}

enum Drag {
    Item {
        target: Target,
        world_start: [f32; 2],
        key_start: [f32; 2],
        handle_start: [f32; 2],
    },
    Marquee {
        start: [f32; 2],
        current: [f32; 2],
        additive: bool,
    },
    Pan {
        mouse: [f32; 2],
        scroll_time: f32,
        center_value: f32,
    },
}

struct App {
    curves: Vec<Curve>,
    px_time: f32,
    px_value: f32,
    scroll_time: f32,
    center_value: f32,
    drag: Option<Drag>,
    left_was_down: bool,
    middle_was_down: bool,
    font11: Font,
    font12: Font,
    font13: Font,
}

fn font_set() -> (Font, Font, Font) {
    let face = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system font");
    (
        Font::new(face.clone(), 11.0),
        Font::new(face.clone(), 12.0),
        Font::new(face, 13.0),
    )
}

fn fill(c: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    c.draw_rect(rect, &p);
}

fn line(c: &skia_safe::Canvas, a: Point, b: Point, color: Color, width: f32) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    p.set_stroke_width(width);
    c.draw_line(a, b, &p);
}

fn label(c: &skia_safe::Canvas, s: &str, x: f32, y: f32, font: &Font, color: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    c.draw_str(s, (x, y), font, &p);
}

fn circle(c: &skia_safe::Canvas, at: Point, radius: f32, color: Color, stroke: Option<Color>) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    c.draw_circle(at, radius, &p);
    if let Some(outline) = stroke {
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(1.5);
        p.set_color(outline);
        c.draw_circle(at, radius, &p);
    }
}

impl App {
    fn new() -> Self {
        let colors = [
            Color::from_rgb(86, 180, 244),
            Color::from_rgb(240, 103, 166),
            Color::from_rgb(249, 183, 52),
            Color::from_rgb(79, 210, 157),
        ];
        let names = ["Position X", "Position Y", "Rotation", "Opacity"];
        let mut curves = Vec::new();
        for ci in 0..4 {
            let mut keys = Vec::new();
            for i in 0..6 {
                let time = i as f32 * 1.55 + ci as f32 * 0.18;
                let value = ((i as f32 * 1.2 + ci as f32).sin() * 0.72) + ci as f32 * 0.12 - 0.2;
                keys.push(Keyframe {
                    time,
                    value,
                    incoming: [-0.48, -0.12 + ci as f32 * 0.025],
                    outgoing: [0.48, 0.12 - ci as f32 * 0.025],
                    selected: ci == 0 && i == 2,
                });
            }
            curves.push(Curve {
                name: names[ci],
                color: colors[ci],
                keys,
            });
        }
        let (font11, font12, font13) = font_set();
        Self {
            curves,
            px_time: 125.0,
            px_value: 205.0,
            scroll_time: -0.35,
            center_value: 0.15,
            drag: None,
            left_was_down: false,
            middle_was_down: false,
            font11,
            font12,
            font13,
        }
    }

    fn graph_rect(width: usize, height: usize) -> Rect {
        Rect::from_xywh(LEFT, TOP, width as f32 - LEFT, height as f32 - TOP - STATUS)
    }

    fn to_screen(&self, world: [f32; 2], width: usize, height: usize) -> Point {
        let graph = Self::graph_rect(width, height);
        Point::new(
            graph.left + (world[0] - self.scroll_time) * self.px_time,
            graph.center_y() - (world[1] - self.center_value) * self.px_value,
        )
    }

    fn to_world(&self, screen: [f32; 2], width: usize, height: usize) -> [f32; 2] {
        let graph = Self::graph_rect(width, height);
        [
            self.scroll_time + (screen[0] - graph.left) / self.px_time,
            self.center_value - (screen[1] - graph.center_y()) / self.px_value,
        ]
    }

    fn clear_selection(&mut self) {
        for curve in &mut self.curves {
            for key in &mut curve.keys {
                key.selected = false;
            }
        }
    }

    fn hit(&self, mouse: [f32; 2], width: usize, height: usize) -> Option<Target> {
        let mp = Point::new(mouse[0], mouse[1]);
        for (ci, curve) in self.curves.iter().enumerate().rev() {
            for (ki, key) in curve.keys.iter().enumerate().rev() {
                let kp = self.to_screen([key.time, key.value], width, height);
                if key.selected {
                    let ip = self.to_screen(
                        [key.time + key.incoming[0], key.value + key.incoming[1]],
                        width,
                        height,
                    );
                    let op = self.to_screen(
                        [key.time + key.outgoing[0], key.value + key.outgoing[1]],
                        width,
                        height,
                    );
                    if (ip - mp).length() <= 9.0 {
                        return Some(Target::Incoming(ci, ki));
                    }
                    if (op - mp).length() <= 9.0 {
                        return Some(Target::Outgoing(ci, ki));
                    }
                }
                if (kp - mp).length() <= 9.0 {
                    return Some(Target::Key(ci, ki));
                }
            }
        }
        None
    }

    fn zoom_at(&mut self, mouse: [f32; 2], factor: f32, width: usize, height: usize) {
        let before = self.to_world(mouse, width, height);
        self.px_time = (self.px_time * factor).clamp(35.0, 520.0);
        self.px_value = (self.px_value * factor).clamp(65.0, 700.0);
        let after = self.to_world(mouse, width, height);
        self.scroll_time += before[0] - after[0];
        self.center_value += before[1] - after[1];
    }

    fn input(&mut self, window: &Window, width: usize, height: usize) -> bool {
        let (mx, my) = window
            .get_mouse_pos(MouseMode::Discard)
            .unwrap_or((-1.0, -1.0));
        let mouse = [mx, my];
        let mut dirty = false;
        if let Some((wx, wy)) = window.get_scroll_wheel() {
            dirty = wx != 0.0 || wy != 0.0;
            let command = window.is_key_down(Key::LeftSuper)
                || window.is_key_down(Key::RightSuper)
                || window.is_key_down(Key::LeftCtrl)
                || window.is_key_down(Key::RightCtrl);
            let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            if command {
                self.zoom_at(
                    mouse,
                    if wy > 0.0 { 1.13 } else { 1.0 / 1.13 },
                    width,
                    height,
                );
            } else if shift || wx.abs() > wy.abs() {
                self.scroll_time -= (wx + wy) * 0.28;
            } else {
                self.center_value += wy * 0.16;
            }
        }
        if window.is_key_pressed(Key::Equal, KeyRepeat::Yes) {
            self.zoom_at(mouse, 1.12, width, height);
            dirty = true;
        }
        if window.is_key_pressed(Key::Minus, KeyRepeat::Yes) {
            self.zoom_at(mouse, 1.0 / 1.12, width, height);
            dirty = true;
        }
        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            self.clear_selection();
            dirty = true;
        }

        let left = window.get_mouse_down(MouseButton::Left);
        let middle = window.get_mouse_down(MouseButton::Middle);
        if middle && !self.middle_was_down {
            self.drag = Some(Drag::Pan {
                mouse,
                scroll_time: self.scroll_time,
                center_value: self.center_value,
            });
            dirty = true;
        }
        if left
            && !self.left_was_down
            && Self::graph_rect(width, height).contains(Point::new(mx, my))
        {
            let additive =
                window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
            if let Some(target) = self.hit(mouse, width, height) {
                let (ci, ki) = match target {
                    Target::Key(c, k) | Target::Incoming(c, k) | Target::Outgoing(c, k) => (c, k),
                };
                if !additive && !self.curves[ci].keys[ki].selected {
                    self.clear_selection();
                }
                self.curves[ci].keys[ki].selected = true;
                let key = &self.curves[ci].keys[ki];
                self.drag = Some(Drag::Item {
                    target,
                    world_start: self.to_world(mouse, width, height),
                    key_start: [key.time, key.value],
                    handle_start: match target {
                        Target::Incoming(_, _) => key.incoming,
                        Target::Outgoing(_, _) => key.outgoing,
                        Target::Key(_, _) => [0.0, 0.0],
                    },
                });
            } else {
                if !additive {
                    self.clear_selection();
                }
                self.drag = Some(Drag::Marquee {
                    start: mouse,
                    current: mouse,
                    additive,
                });
            }
            dirty = true;
        }

        if left || middle {
            let world_now = self.to_world(mouse, width, height);
            if let Some(drag) = &mut self.drag {
                match drag {
                    Drag::Item {
                        target,
                        world_start,
                        key_start,
                        handle_start,
                    } => {
                        let delta = [world_now[0] - world_start[0], world_now[1] - world_start[1]];
                        match *target {
                            Target::Key(ci, ki) => {
                                self.curves[ci].keys[ki].time = (key_start[0] + delta[0]).max(0.0);
                                self.curves[ci].keys[ki].value = key_start[1] + delta[1];
                            }
                            Target::Incoming(ci, ki) => {
                                self.curves[ci].keys[ki].incoming =
                                    [handle_start[0] + delta[0], handle_start[1] + delta[1]]
                            }
                            Target::Outgoing(ci, ki) => {
                                self.curves[ci].keys[ki].outgoing =
                                    [handle_start[0] + delta[0], handle_start[1] + delta[1]]
                            }
                        }
                    }
                    Drag::Marquee { current, .. } => *current = mouse,
                    Drag::Pan {
                        mouse: start,
                        scroll_time,
                        center_value,
                    } => {
                        self.scroll_time = *scroll_time - (mouse[0] - start[0]) / self.px_time;
                        self.center_value = *center_value + (mouse[1] - start[1]) / self.px_value;
                    }
                }
                dirty = true;
            }
        }

        if (!left && self.left_was_down) || (!middle && self.middle_was_down) {
            if let Some(Drag::Marquee {
                start,
                current,
                additive,
            }) = &self.drag
            {
                let rect = Rect::from_ltrb(
                    start[0].min(current[0]),
                    start[1].min(current[1]),
                    start[0].max(current[0]),
                    start[1].max(current[1]),
                );
                if !*additive {
                    self.clear_selection();
                }
                for ci in 0..self.curves.len() {
                    for ki in 0..self.curves[ci].keys.len() {
                        let key = &self.curves[ci].keys[ki];
                        let p = self.to_screen([key.time, key.value], width, height);
                        if rect.contains(p) {
                            self.curves[ci].keys[ki].selected = true;
                        }
                    }
                }
            }
            self.drag = None;
            dirty = true;
        }
        self.left_was_down = left;
        self.middle_was_down = middle;
        dirty
    }

    fn draw(&self, c: &skia_safe::Canvas, width: usize, height: usize) {
        c.clear(Color::from_rgb(31, 32, 34));
        let graph = Self::graph_rect(width, height);
        fill(c, graph, Color::from_rgb(38, 39, 41));

        let time_step = if self.px_time > 180.0 {
            0.5
        } else if self.px_time > 75.0 {
            1.0
        } else {
            2.0
        };
        let first_t = (self.scroll_time / time_step).floor() as i32 - 1;
        let last_t =
            ((self.scroll_time + graph.width() / self.px_time) / time_step).ceil() as i32 + 1;
        for i in first_t..=last_t {
            let t = i as f32 * time_step;
            let x = self.to_screen([t, self.center_value], width, height).x;
            let major = (t.round() - t).abs() < 0.001;
            line(
                c,
                Point::new(x, TOP),
                Point::new(x, graph.bottom),
                if major {
                    Color::from_rgb(66, 67, 70)
                } else {
                    Color::from_rgb(50, 51, 54)
                },
                1.0,
            );
            if major {
                label(
                    c,
                    &format!("{t:.0}s"),
                    x + 5.0,
                    16.0,
                    &self.font11,
                    Color::from_rgb(164, 166, 170),
                );
            }
        }
        let value_step = if self.px_value > 300.0 {
            0.25
        } else if self.px_value > 130.0 {
            0.5
        } else {
            1.0
        };
        for i in -20..=20 {
            let v = i as f32 * value_step;
            let y = self.to_screen([self.scroll_time, v], width, height).y;
            if y < TOP || y > graph.bottom {
                continue;
            }
            let major = i % 2 == 0;
            line(
                c,
                Point::new(LEFT, y),
                Point::new(width as f32, y),
                if major {
                    Color::from_rgb(63, 64, 67)
                } else {
                    Color::from_rgb(48, 49, 52)
                },
                1.0,
            );
            label(
                c,
                &format!("{v:.2}"),
                LEFT + 5.0,
                y - 4.0,
                &self.font11,
                Color::from_rgb(137, 139, 143),
            );
        }

        for curve in &self.curves {
            let mut builder = PathBuilder::new();
            if let Some(first) = curve.keys.first() {
                builder.move_to(self.to_screen([first.time, first.value], width, height));
                for pair in curve.keys.windows(2) {
                    let a = &pair[0];
                    let b = &pair[1];
                    builder.cubic_to(
                        self.to_screen(
                            [a.time + a.outgoing[0], a.value + a.outgoing[1]],
                            width,
                            height,
                        ),
                        self.to_screen(
                            [b.time + b.incoming[0], b.value + b.incoming[1]],
                            width,
                            height,
                        ),
                        self.to_screen([b.time, b.value], width, height),
                    );
                }
            }
            let path = builder.detach();
            let mut p = Paint::default();
            p.set_anti_alias(true);
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(2.2);
            p.set_color(curve.color);
            c.draw_path(&path, &p);
        }

        for curve in &self.curves {
            for key in &curve.keys {
                let kp = self.to_screen([key.time, key.value], width, height);
                if key.selected {
                    let ip = self.to_screen(
                        [key.time + key.incoming[0], key.value + key.incoming[1]],
                        width,
                        height,
                    );
                    let op = self.to_screen(
                        [key.time + key.outgoing[0], key.value + key.outgoing[1]],
                        width,
                        height,
                    );
                    line(c, ip, kp, Color::from_rgb(185, 187, 191), 1.0);
                    line(c, kp, op, Color::from_rgb(185, 187, 191), 1.0);
                    circle(c, ip, 4.0, Color::from_rgb(38, 39, 41), Some(Color::WHITE));
                    circle(c, op, 4.0, Color::from_rgb(38, 39, 41), Some(Color::WHITE));
                }
                circle(
                    c,
                    kp,
                    if key.selected { 6.0 } else { 4.5 },
                    curve.color,
                    key.selected.then_some(Color::WHITE),
                );
            }
        }

        if let Some(Drag::Marquee { start, current, .. }) = &self.drag {
            let rect = Rect::from_ltrb(
                start[0].min(current[0]),
                start[1].min(current[1]),
                start[0].max(current[0]),
                start[1].max(current[1]),
            );
            fill(c, rect, Color::from_argb(38, 105, 170, 245));
            let mut p = Paint::default();
            p.set_style(PaintStyle::Stroke);
            p.set_color(Color::from_rgb(105, 170, 245));
            c.draw_rect(rect, &p);
        }

        fill(
            c,
            Rect::from_xywh(0.0, 0.0, LEFT, height as f32),
            Color::from_rgb(29, 30, 32),
        );
        fill(
            c,
            Rect::from_xywh(0.0, 0.0, width as f32, TOP),
            Color::from_rgb(25, 26, 28),
        );
        label(
            c,
            "CURVE EDITOR",
            14.0,
            24.0,
            &self.font12,
            Color::from_rgb(215, 217, 220),
        );
        for (i, curve) in self.curves.iter().enumerate() {
            let y = TOP + i as f32 * 34.0;
            circle(c, Point::new(17.0, y + 17.0), 4.5, curve.color, None);
            label(
                c,
                curve.name,
                31.0,
                y + 22.0,
                &self.font13,
                Color::from_rgb(209, 211, 214),
            );
        }
        fill(
            c,
            Rect::from_xywh(0.0, height as f32 - STATUS, width as f32, STATUS),
            Color::from_rgb(25, 26, 28),
        );
        let selected = self
            .curves
            .iter()
            .flat_map(|c| &c.keys)
            .filter(|k| k.selected)
            .count();
        label(
            c,
            &format!(
                "click/drag key or tangent   empty drag: marquee   middle drag: pan   wheel: value pan   shift+wheel: time pan   cmd/ctrl+wheel or +/-: zoom   |   selected {selected}"
            ),
            12.0,
            height as f32 - 9.0,
            &self.font11,
            Color::from_rgb(177, 179, 183),
        );
    }
}

fn render(app: &App, pixels: &mut [u32], width: usize, height: usize) {
    let byte_len = pixels.len() * 4;
    let bytes =
        unsafe { std::slice::from_raw_parts_mut(pixels.as_mut_ptr().cast::<u8>(), byte_len) };
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::BGRA8888,
        AlphaType::Opaque,
        None,
    );
    let mut surface = surfaces::wrap_pixels(&info, bytes, Some(width * 4), None).expect("surface");
    app.draw(surface.canvas(), width, height);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut app = App::new();
    if args.get(1).map(String::as_str) == Some("--snapshot") {
        let output = args.get(2).expect("snapshot path");
        let (width, height) = (1400usize, 820usize);
        let mut surface = surfaces::raster_n32_premul((width as i32, height as i32)).unwrap();
        app.draw(surface.canvas(), width, height);
        let png = surface
            .image_snapshot()
            .encode(None, EncodedImageFormat::PNG, 100)
            .unwrap();
        std::fs::write(output, png.as_bytes()).unwrap();
        println!("{output}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--bench") {
        let (width, height) = (1400usize, 820usize);
        let mut pixels = vec![0u32; width * height];
        let mut times = Vec::new();
        for _ in 0..240 {
            let now = Instant::now();
            render(&app, &mut pixels, width, height);
            times.push(now.elapsed());
        }
        times.sort();
        println!(
            "curve-editor CPU raster p50={:.3}ms p95={:.3}ms",
            times[120].as_secs_f64() * 1000.0,
            times[228].as_secs_f64() * 1000.0
        );
        return;
    }
    let mut window = Window::new(
        "Skia Curve Editor Probe",
        1400,
        820,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .unwrap();
    window.set_target_fps(60);
    let mut pixels = Vec::<u32>::new();
    let mut last_size = (0, 0);
    let mut dirty = true;
    while window.is_open() && !window.is_key_down(Key::Q) {
        let size = window.get_size();
        if size != last_size {
            pixels.resize(size.0 * size.1, 0);
            last_size = size;
            dirty = true;
        }
        dirty |= app.input(&window, size.0, size.1);
        if dirty {
            render(&app, &mut pixels, size.0, size.1);
            window.update_with_buffer(&pixels, size.0, size.1).unwrap();
            dirty = false;
        } else {
            window.update();
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_round_trip_and_zoom_anchor_hold() {
        let mut app = App::new();
        let world = [2.75, -0.4];
        let screen = app.to_screen(world, 1400, 820);
        let back = app.to_world([screen.x, screen.y], 1400, 820);
        assert!((world[0] - back[0]).abs() < 0.001 && (world[1] - back[1]).abs() < 0.001);
        let mouse = [700.0, 390.0];
        let before = app.to_world(mouse, 1400, 820);
        app.zoom_at(mouse, 1.5, 1400, 820);
        let after = app.to_world(mouse, 1400, 820);
        assert!((before[0] - after[0]).abs() < 0.001 && (before[1] - after[1]).abs() < 0.001);
    }
}
