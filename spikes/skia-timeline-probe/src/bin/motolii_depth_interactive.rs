// Depth Rail 選択フォーカス案の対話demo。
// docs/reviews/2026-08-08-depth-rail-selection-focus-decision.md の§2を動きで確認する。
//
//   click        clip選択 (Shift+click で追加)
//   drag         フォーカスmarkerをdrag → その場で視差が生まれ、行にz chipが生える
//   D            選択を奥端-0.36〜手前端-0.05へ層順distribute(アニメ)
//   R            選択のzを0へ戻す(塊へ帰るアニメ)
//   Esc          選択解除 / Q 終了
//
// 確認対象:
//   - z=0の既定群は灰色1塊。個別に描かれること自体が逸脱表示
//   - 選択だけがフォーカス(行と同色・可動)を与える
//   - 選択を外した逸脱Objectは灰tickへ降格する(逸脱個別化が生きて見える)
//   - playhead下に居ないclip(intro_card)は選択してもレーンに現れない

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use skia_safe::{
    AlphaType, Color, ColorType, Contains, Font, FontMgr, FontStyle, ImageInfo, Paint, PaintStyle,
    Rect, surfaces,
};
use std::time::Duration;

const LABEL_W: f32 = 240.0;
const RAIL_H: f32 = 78.0;
const RULER_H: f32 = 26.0;
const ROW_H: f32 = 32.0;
const FOOTER_H: f32 = 26.0;
const EPS: f32 = 0.004;

const BG: u32 = 0x242424;
const SURFACE: u32 = 0x2a2a2a;
const SURFACE_HI: u32 = 0x363636;
const SURFACE_LO: u32 = 0x1e1e1e;
const LINE: u32 = 0x111111;
const LINE_2: u32 = 0x5d5d5d;
const INK: u32 = 0xd6d6d6;
const SUB: u32 = 0xb5b5b5;
const MUTED: u32 = 0x757575;
const ACTIVE: u32 = 0xffad56;
const CLIPS: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];
const ROW7: u32 = 0x8b8b7d;

const NAMES: [&str; 7] = [
    "sky_plate.mp4",
    "Pulse rings",
    "NIGHT DRIVE",
    "City grid",
    "reflection.mp4",
    "traffic_pass.mp4",
    "intro_card.png",
];
// (start, end) 0..1。playhead=0.46。intro_cardはplayhead前に終わる
const SPANS: [(f32, f32); 7] = [
    (0.03, 0.55),
    (0.10, 0.60),
    (0.17, 0.65),
    (0.25, 0.73),
    (0.32, 0.79),
    (0.40, 0.85),
    (0.02, 0.32),
];
const PLAYHEAD: f32 = 0.46;
const ANON_AT_ZERO: usize = 21;
const ANON_DEVIATED: [f32; 3] = [-0.44, 0.17, 0.40];
const DIST_FAR: f32 = -0.36;
const DIST_NEAR: f32 = -0.05;

struct Obj {
    z: f32,
    target: f32,
    selected: bool,
}

struct App {
    objs: Vec<Obj>, // 7行ぶん
    drag_row: Option<usize>,
    mouse_was_down: bool,
    font8: Font,
    font9: Font,
    font11: Font,
}

fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn fill(c: &skia_safe::Canvas, r: Rect, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    c.draw_rect(r, &p);
}

fn stroke(c: &skia_safe::Canvas, r: Rect, color: Color) {
    let mut p = Paint::default();
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(color);
    c.draw_rect(r, &p);
}

fn hline(c: &skia_safe::Canvas, y: f32, x0: f32, x1: f32, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    c.draw_line((x0, y), (x1, y), &p);
}

fn vline(c: &skia_safe::Canvas, x: f32, y0: f32, y1: f32, color: Color) {
    let mut p = Paint::default();
    p.set_color(color);
    c.draw_line((x, y0), (x, y1), &p);
}

fn text(c: &skia_safe::Canvas, s: &str, x: f32, y: f32, font: &Font, color: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(color);
    c.draw_str(s, (x, y), font, &p);
}

fn text_w(s: &str, font: &Font) -> f32 {
    let p = Paint::default();
    font.measure_str(s, Some(&p)).0
}

fn at_playhead(row: usize) -> bool {
    SPANS[row].0 <= PLAYHEAD && PLAYHEAD <= SPANS[row].1
}

impl App {
    fn new() -> Self {
        let face = FontMgr::default()
            .legacy_make_typeface(None, FontStyle::normal())
            .expect("font");
        Self {
            objs: (0..7)
                .map(|_| Obj {
                    z: 0.0,
                    target: 0.0,
                    selected: false,
                })
                .collect(),
            drag_row: None,
            mouse_was_down: false,
            font8: Font::new(face.clone(), 9.0),
            font9: Font::new(face.clone(), 10.5),
            font11: Font::new(face, 12.0),
        }
    }

    fn axis(&self, width: f32) -> (f32, f32) {
        (LABEL_W + 44.0, width - 44.0)
    }

    fn axis_x(&self, z: f32, width: f32) -> f32 {
        let (l, r) = self.axis(width);
        l + (z + 0.5) * (r - l)
    }

    fn x_to_z(&self, x: f32, width: f32) -> f32 {
        let (l, r) = self.axis(width);
        ((x - l) / (r - l) - 0.5).clamp(-0.5, 0.5)
    }

    // rail上のフォーカスmarker配置。描画とhit-testで共有
    fn markers(&self, width: f32) -> Vec<(usize, Rect)> {
        let ay = RAIL_H * 0.56;
        let sx = self.axis_x(0.0, width);
        let cluster: Vec<usize> = (0..6)
            .filter(|&i| self.objs[i].selected && at_playhead(i) && self.objs[i].z.abs() < EPS)
            .collect();
        let total = cluster.len() as f32 * 10.0 - 2.0;
        let mut out = Vec::new();
        for (k, &row) in cluster.iter().enumerate() {
            let x = sx - total / 2.0 + k as f32 * 10.0;
            out.push((row, Rect::from_xywh(x, ay - 30.0, 8.0, 17.0)));
        }
        for row in 0..6 {
            let o = &self.objs[row];
            if o.selected && at_playhead(row) && o.z.abs() >= EPS {
                let x = self.axis_x(o.z, width);
                out.push((row, Rect::from_xywh(x - 5.0, ay - 14.0, 10.0, 28.0)));
            }
        }
        out
    }

    fn clip_rect(&self, row: usize, width: f32) -> Rect {
        let y = RAIL_H + RULER_H + row as f32 * ROW_H;
        let start = LABEL_W + SPANS[row].0 * (width - LABEL_W);
        let end = LABEL_W + SPANS[row].1 * (width - LABEL_W);
        Rect::from_ltrb(start, y + 5.0, end, y + ROW_H - 5.0)
    }

    fn input(&mut self, window: &Window, width: f32) -> bool {
        let mut dirty = false;
        let (mx, my) = window
            .get_mouse_pos(MouseMode::Discard)
            .unwrap_or((-1.0, -1.0));
        let shift = window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift);
        let down = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Escape, KeyRepeat::No) {
            for o in &mut self.objs {
                o.selected = false;
            }
            dirty = true;
        }
        if window.is_key_pressed(Key::D, KeyRepeat::No) {
            let sel: Vec<usize> = (0..6)
                .filter(|&i| self.objs[i].selected && at_playhead(i))
                .collect();
            let n = sel.len();
            for (k, &row) in sel.iter().enumerate() {
                let t = if n <= 1 { 0.5 } else { k as f32 / (n - 1) as f32 };
                self.objs[row].target = DIST_FAR + (DIST_NEAR - DIST_FAR) * t;
            }
            dirty = !sel.is_empty();
        }
        if window.is_key_pressed(Key::R, KeyRepeat::No) {
            for o in self.objs.iter_mut().filter(|o| o.selected) {
                o.target = 0.0;
            }
            dirty = true;
        }

        if down && !self.mouse_was_down {
            if my < RAIL_H {
                if let Some((row, _)) = self
                    .markers(width)
                    .iter()
                    .find(|(_, r)| r.contains(skia_safe::Point::new(mx, my)))
                    .copied()
                {
                    self.drag_row = Some(row);
                    dirty = true;
                }
            } else {
                let hit = (0..7).find(|&row| {
                    self.clip_rect(row, width)
                        .contains(skia_safe::Point::new(mx, my))
                        || (mx < LABEL_W
                            && my >= RAIL_H + RULER_H + row as f32 * ROW_H
                            && my < RAIL_H + RULER_H + (row + 1) as f32 * ROW_H)
                });
                match hit {
                    Some(row) => {
                        if shift {
                            self.objs[row].selected = !self.objs[row].selected;
                        } else {
                            for (i, o) in self.objs.iter_mut().enumerate() {
                                o.selected = i == row;
                            }
                        }
                    }
                    None => {
                        if !shift {
                            for o in &mut self.objs {
                                o.selected = false;
                            }
                        }
                    }
                }
                dirty = true;
            }
        }
        if down {
            if let Some(row) = self.drag_row {
                let z = self.x_to_z(mx, width);
                self.objs[row].z = z;
                self.objs[row].target = z;
                dirty = true;
            }
        } else if self.mouse_was_down {
            self.drag_row = None;
            dirty = true;
        }
        self.mouse_was_down = down;

        // アニメ(D/Rのtargetへ寄せる)
        for o in &mut self.objs {
            let d = o.target - o.z;
            if d.abs() > 0.0005 {
                o.z += d * 0.22;
                dirty = true;
            } else if o.z != o.target {
                o.z = o.target;
                dirty = true;
            }
        }
        dirty
    }

    fn draw(&self, c: &skia_safe::Canvas, width: f32, height: f32) {
        c.clear(rgb(BG));
        let ay = RAIL_H * 0.56;
        let (axis_l, axis_r) = self.axis(width);

        // ── rail ──
        fill(c, Rect::from_xywh(0.0, 0.0, width, RAIL_H), rgb(SURFACE_LO));
        fill(c, Rect::from_xywh(0.0, 0.0, LABEL_W, RAIL_H), rgb(SURFACE));
        vline(c, LABEL_W, 0.0, RAIL_H, rgb(LINE));
        hline(c, RAIL_H - 1.0, 0.0, width, rgb(LINE));

        text(c, "DEPTH", 12.0, 24.0, &self.font11, rgb(INK));
        let n_sel = self.objs.iter().filter(|o| o.selected).count();
        let off_playhead = (0..7)
            .filter(|&i| self.objs[i].selected && !at_playhead(i))
            .count();
        if n_sel > 0 {
            text(
                c,
                &format!("{n_sel} selected"),
                12.0,
                46.0,
                &self.font9,
                rgb(ACTIVE),
            );
            if off_playhead > 0 {
                text(
                    c,
                    &format!("{off_playhead} not at playhead"),
                    12.0,
                    62.0,
                    &self.font8,
                    rgb(MUTED),
                );
            }
        }

        hline(c, ay, axis_l, axis_r, rgb(LINE_2));
        text(c, "BACK", axis_l, ay + 18.0, &self.font8, rgb(MUTED));
        let fw = text_w("FRONT", &self.font8);
        text(c, "FRONT", axis_r - fw, ay + 18.0, &self.font8, rgb(MUTED));

        // 逸脱した匿名Object = 個別化された灰tick
        for &z in &ANON_DEVIATED {
            let x = self.axis_x(z, width);
            vline(c, x, ay - 6.0, ay + 6.0, rgb(0x565656));
            vline(c, x + 1.0, ay - 6.0, ay + 6.0, rgb(0x565656));
        }
        // 選択されていない named の逸脱も灰tickへ降格する(規則が生きて見える)
        for row in 0..6 {
            let o = &self.objs[row];
            if !o.selected && at_playhead(row) && o.z.abs() >= EPS {
                let x = self.axis_x(o.z, width);
                vline(c, x, ay - 6.0, ay + 6.0, rgb(0x565656));
                vline(c, x + 1.0, ay - 6.0, ay + 6.0, rgb(0x565656));
            }
        }

        // z=0 の灰色の塊
        let at_zero = ANON_AT_ZERO
            + (0..6)
                .filter(|&i| at_playhead(i) && self.objs[i].z.abs() < EPS)
                .count();
        let sx = self.axis_x(0.0, width);
        let mass = Rect::from_xywh(sx - 18.0, ay - 11.0, 36.0, 22.0);
        fill(c, mass, rgb(0x3a3a3a));
        stroke(c, mass, rgb(0x4d4d4d));
        let label = format!("{at_zero}");
        let lw = text_w(&label, &self.font9);
        text(c, &label, sx - lw / 2.0, ay + 4.5, &self.font9, rgb(0x8a8a8a));

        // フォーカスmarker(選択のみ)
        for (row, r) in self.markers(width) {
            fill(c, r, rgb(CLIPS[row]));
            stroke(c, r, rgb(ACTIVE));
        }

        // ── timeline ──
        let ttop = RAIL_H;
        fill(c, Rect::from_xywh(0.0, ttop, width, RULER_H), rgb(SURFACE));
        text(
            c,
            "TIME / BEAT",
            LABEL_W + 12.0,
            ttop + 17.0,
            &self.font8,
            rgb(MUTED),
        );
        for i in 0..9 {
            let x = LABEL_W + i as f32 * (width - LABEL_W) / 8.0;
            vline(c, x, ttop, height - FOOTER_H, rgb(0x2e2e2e));
            if i > 0 && i % 2 == 0 {
                text(
                    c,
                    &format!("{}", 52 + i / 2),
                    x + 6.0,
                    ttop + 17.0,
                    &self.font8,
                    rgb(MUTED),
                );
            }
        }
        hline(c, ttop + RULER_H, 0.0, width, rgb(LINE));
        fill(
            c,
            Rect::from_xywh(0.0, ttop + RULER_H, LABEL_W, 7.0 * ROW_H),
            rgb(SURFACE_HI),
        );
        vline(c, LABEL_W, ttop, height - FOOTER_H, rgb(LINE));

        for row in 0..7 {
            let y = ttop + RULER_H + row as f32 * ROW_H;
            hline(c, y, 0.0, width, rgb(0x2c2c2c));
            let color = if row < 6 { CLIPS[row] } else { ROW7 };
            fill(c, Rect::from_xywh(12.0, y + 8.0, 4.0, ROW_H - 16.0), rgb(color));
            text(c, NAMES[row], 24.0, y + 21.0, &self.font9, rgb(INK));

            let o = &self.objs[row];
            if o.z.abs() >= EPS {
                let s = format!("z {:+.2}", o.z);
                let w = text_w(&s, &self.font8);
                let col = if self.drag_row == Some(row) || o.selected {
                    ACTIVE
                } else {
                    SUB
                };
                text(c, &s, LABEL_W - 10.0 - w, y + 21.0, &self.font8, rgb(col));
            }

            let r = self.clip_rect(row, width);
            fill(c, r, rgb(color));
            if o.selected {
                stroke(
                    c,
                    Rect::from_ltrb(r.left - 1.5, r.top - 1.5, r.right + 1.5, r.bottom + 1.5),
                    rgb(INK),
                );
            }
            text(c, NAMES[row], r.left + 8.0, y + 21.0, &self.font8, rgb(0x191919));
        }

        let px = LABEL_W + PLAYHEAD * (width - LABEL_W);
        vline(c, px, ttop + RULER_H, height - FOOTER_H, rgb(INK));

        // footer
        fill(
            c,
            Rect::from_xywh(0.0, height - FOOTER_H, width, FOOTER_H),
            rgb(SURFACE),
        );
        hline(c, height - FOOTER_H, 0.0, width, rgb(LINE));
        text(
            c,
            "click: select clip (Shift: add)   drag marker: depth   D: distribute   R: back to 0   Esc: deselect   Q: quit",
            12.0,
            height - 9.0,
            &self.font8,
            rgb(MUTED),
        );
    }
}

fn main() {
    let mut window = Window::new(
        "Motolii Depth Rail - selection focus probe",
        1400,
        (RAIL_H + RULER_H + 7.0 * ROW_H + FOOTER_H) as usize + 2,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("window");
    window.set_target_fps(60);
    let mut app = App::new();
    let mut pixels = Vec::<u32>::new();
    let mut last_size = (0, 0);
    let mut dirty = true;

    while window.is_open() && !window.is_key_down(Key::Q) {
        let (width, height) = window.get_size();
        if width == 0 || height == 0 {
            continue;
        }
        if (width, height) != last_size {
            pixels.resize(width * height, 0);
            last_size = (width, height);
            dirty = true;
        }
        dirty |= app.input(&window, width as f32);
        if !dirty {
            window.update();
            continue;
        }
        let byte_len = pixels.len() * std::mem::size_of::<u32>();
        let bytes =
            unsafe { std::slice::from_raw_parts_mut(pixels.as_mut_ptr().cast::<u8>(), byte_len) };
        let info = ImageInfo::new(
            (width as i32, height as i32),
            ColorType::BGRA8888,
            AlphaType::Opaque,
            None,
        );
        let mut surface =
            surfaces::wrap_pixels(&info, bytes, Some(width * 4), None).expect("surface");
        app.draw(surface.canvas(), width as f32, height as f32);
        window
            .update_with_buffer(&pixels, width, height)
            .expect("present");
        dirty = false;
        std::thread::sleep(Duration::from_millis(1));
    }
}
