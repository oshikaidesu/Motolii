use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const H: f32 = 270.0;
const HEAD_H: f32 = 28.0;
const DEPTH_H: f32 = 56.0;
const LABEL_W: f32 = 200.0;
const TOOL_W: f32 = 92.0;
const ROW_H: f32 = 25.0;

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum RailMode {
    Closed,
    Open,
    Distribute,
}

fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn fill(cv: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_color(color);
    cv.draw_rect(rect, &paint);
}

fn stroke(cv: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(1.0);
    paint.set_color(color);
    cv.draw_rect(rect, &paint);
}

fn line(cv: &skia_safe::Canvas, a: (f32, f32), b: (f32, f32), color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_stroke_width(1.0);
    paint.set_color(color);
    cv.draw_line(a, b, &paint);
}

fn text(cv: &skia_safe::Canvas, value: &str, x: f32, y: f32, size: f32, color: Color) {
    let typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("typeface");
    let font = Font::new(typeface, size);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    cv.draw_str(value, (x, y), &font, &paint);
}

fn diamond(cv: &skia_safe::Canvas, x: f32, y: f32, radius: f32, color: Color) {
    let mut builder = PathBuilder::new();
    builder.move_to((x, y - radius));
    builder.line_to((x + radius, y));
    builder.line_to((x, y + radius));
    builder.line_to((x - radius, y));
    builder.close();
    let path = builder.detach();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    cv.draw_path(&path, &paint);
}

fn draw_header(cv: &skia_safe::Canvas, mode: RailMode) {
    let open = mode != RailMode::Closed;
    fill(cv, Rect::from_xywh(0.0, 0.0, W, HEAD_H), rgb(SURFACE));
    line(cv, (0.0, HEAD_H - 1.0), (W, HEAD_H - 1.0), rgb(LINE));

    let toggle = Rect::from_xywh(8.0, 3.0, 28.0, 22.0);
    fill(cv, toggle, rgb(if open { 0x4a4a4a } else { 0x333333 }));
    stroke(cv, toggle, rgb(if open { INK } else { LINE_2 }));
    for offset in [-4.0, 0.0, 4.0] {
        line(cv, (17.0, 14.0 + offset), (27.0, 14.0 + offset), rgb(INK));
    }
    text(cv, "TIMELINE", 45.0, 18.0, 10.0, rgb(INK));
    if open {
        text(cv, "6 SELECTED", W - 76.0, 18.0, 8.0, rgb(SUB));
    }
}

fn depth_x(z: f32) -> f32 {
    let left = LABEL_W + 30.0;
    let right = W - TOOL_W - 30.0;
    left + (z + 0.5) * (right - left)
}

fn tool_button(cv: &skia_safe::Canvas, x: f32, top: f32, label: &str, active: bool) {
    let rect = Rect::from_xywh(x, top + 17.0, 20.0, 22.0);
    fill(cv, rect, rgb(BG));
    stroke(cv, rect, rgb(if active { ACTIVE } else { LINE_2 }));
    text(
        cv,
        label,
        x + 6.0,
        top + 31.0,
        8.0,
        rgb(if active { INK } else { MUTED }),
    );
}

fn marker(cv: &skia_safe::Canvas, z: f32, top: f32, label: &str, selected: bool) {
    let x = depth_x(z);
    let width = if selected { 68.0 } else { 58.0 };
    let rect = Rect::from_xywh(x - width / 2.0, top + 19.0, width, 20.0);
    fill(cv, rect, rgb(BG));
    stroke(cv, rect, rgb(if selected { ACTIVE } else { LINE_2 }));
    diamond(
        cv,
        rect.left + 9.0,
        top + 29.0,
        3.2,
        rgb(if selected { ACTIVE } else { MUTED }),
    );
    text(
        cv,
        label,
        rect.left + 16.0,
        top + 32.0,
        7.5,
        rgb(if selected { INK } else { SUB }),
    );
}

fn draw_depth_rail(cv: &skia_safe::Canvas, mode: RailMode) {
    let top = HEAD_H;
    fill(cv, Rect::from_xywh(0.0, top, W, DEPTH_H), rgb(SURFACE_LO));
    fill(
        cv,
        Rect::from_xywh(0.0, top, LABEL_W, DEPTH_H),
        rgb(SURFACE),
    );
    fill(
        cv,
        Rect::from_xywh(W - TOOL_W, top, TOOL_W, DEPTH_H),
        rgb(SURFACE),
    );
    line(cv, (LABEL_W, top), (LABEL_W, top + DEPTH_H), rgb(LINE));
    line(
        cv,
        (W - TOOL_W, top),
        (W - TOOL_W, top + DEPTH_H),
        rgb(LINE),
    );
    line(
        cv,
        (0.0, top + DEPTH_H - 1.0),
        (W, top + DEPTH_H - 1.0),
        rgb(LINE),
    );

    text(cv, "DEPTH", 8.0, top + 18.0, 9.0, rgb(INK));
    let scope = Rect::from_xywh(52.0, top + 5.0, 34.0, 18.0);
    fill(cv, scope, rgb(BG));
    stroke(cv, scope, rgb(LINE_2));
    text(cv, "ROOT", 57.0, top + 17.0, 7.0, rgb(SUB));
    diamond(cv, 13.0, top + 40.0, 3.4, rgb(ACTIVE));
    text(cv, "6 selected", 22.0, top + 43.0, 8.0, rgb(INK));
    text(cv, "27 at z 0", 99.0, top + 43.0, 8.0, rgb(MUTED));

    let axis_y = top + 34.0;
    line(
        cv,
        (depth_x(-0.5), axis_y),
        (depth_x(0.5), axis_y),
        rgb(LINE_2),
    );
    for (z, label) in [
        (-0.5, "−.50"),
        (-0.25, "−.25"),
        (0.0, "0"),
        (0.25, "+.25"),
        (0.5, "+.50"),
    ] {
        let x = depth_x(z);
        line(cv, (x, axis_y - 3.0), (x, axis_y + 3.0), rgb(LINE_2));
        text(cv, label, x - 10.0, top + 10.0, 7.0, rgb(MUTED));
    }
    text(cv, "BACK", depth_x(-0.5), top + 24.0, 7.0, rgb(MUTED));
    text(
        cv,
        "FRONT",
        depth_x(0.5) - 26.0,
        top + 24.0,
        7.0,
        rgb(MUTED),
    );

    if mode == RailMode::Distribute {
        let far = depth_x(-0.25);
        let near = depth_x(0.25);
        fill(
            cv,
            Rect::from_ltrb(far, axis_y - 7.0, near, axis_y + 7.0),
            Color::from_argb(45, 255, 173, 86),
        );
        line(cv, (far, axis_y - 10.0), (far, axis_y + 10.0), rgb(ACTIVE));
        line(
            cv,
            (near, axis_y - 10.0),
            (near, axis_y + 10.0),
            rgb(ACTIVE),
        );
        for (z, label) in [
            (-0.25, "1"),
            (-0.15, "2"),
            (-0.05, "3"),
            (0.05, "4"),
            (0.15, "5"),
            (0.25, "6"),
        ] {
            marker(cv, z, top, label, true);
        }
    } else {
        let stack_x = depth_x(0.0);
        let stack = Rect::from_xywh(stack_x - 70.0, top + 18.0, 140.0, 22.0);
        fill(cv, stack, rgb(BG));
        stroke(cv, stack, rgb(ACTIVE));
        diamond(cv, stack.left + 12.0, top + 29.0, 3.4, rgb(ACTIVE));
        text(
            cv,
            "6 selected",
            stack.left + 21.0,
            top + 32.0,
            8.0,
            rgb(INK),
        );
        text(
            cv,
            "/ 27 at z 0",
            stack.left + 73.0,
            top + 32.0,
            7.5,
            rgb(SUB),
        );
        for z in [-0.32, 0.18, 0.31] {
            diamond(cv, depth_x(z), axis_y, 3.0, rgb(MUTED));
        }
        text(
            cv,
            "< 1 OUT",
            depth_x(-0.5) + 5.0,
            top + 33.0,
            7.0,
            rgb(MUTED),
        );
        text(
            cv,
            "1 OUT >",
            depth_x(0.5) - 34.0,
            top + 33.0,
            7.0,
            rgb(MUTED),
        );
    }

    text(
        cv,
        "CAMERA RANK 9-14 / 32",
        W - TOOL_W - 122.0,
        top + 10.0,
        7.0,
        rgb(0xa4b7b7),
    );

    if mode == RailMode::Distribute {
        tool_button(cv, W - 88.0, top, "R", false);
        tool_button(cv, W - 65.0, top, "X", false);
        tool_button(cv, W - 42.0, top, "OK", true);
    } else {
        text(cv, "OCCLUSION OFF", W - 86.0, top + 11.0, 7.0, rgb(MUTED));
        let spread = Rect::from_xywh(W - 88.0, top + 20.0, 54.0, 22.0);
        fill(cv, spread, rgb(BG));
        stroke(cv, spread, rgb(ACTIVE));
        text(cv, "SPREAD", W - 81.0, top + 34.0, 7.5, rgb(INK));
        let fit = Rect::from_xywh(W - 31.0, top + 20.0, 27.0, 22.0);
        fill(cv, fit, rgb(BG));
        stroke(cv, fit, rgb(LINE_2));
        text(cv, "FIT", W - 26.0, top + 34.0, 7.0, rgb(MUTED));
    }
}

fn draw_timeline_body(cv: &skia_safe::Canvas, top: f32) {
    let body_h = H - top;
    fill(cv, Rect::from_xywh(0.0, top, W, body_h), rgb(BG));
    let controls_w = 200.0;
    fill(
        cv,
        Rect::from_xywh(0.0, top, controls_w, body_h),
        rgb(SURFACE_HI),
    );
    line(cv, (controls_w, top), (controls_w, H), rgb(LINE));

    let ruler_h = 22.0;
    fill(cv, Rect::from_xywh(0.0, top, W, ruler_h), rgb(SURFACE));
    text(cv, "S   M", 8.0, top + 15.0, 7.0, rgb(MUTED));
    text(
        cv,
        "TIME / BEAT",
        controls_w + 10.0,
        top + 15.0,
        7.0,
        rgb(MUTED),
    );
    for i in 0..9 {
        let x = controls_w + i as f32 * (W - controls_w) / 8.0;
        line(
            cv,
            (x, top + 14.0),
            (x, H),
            rgb(if i % 2 == 0 { LINE_2 } else { LINE }),
        );
        text(
            cv,
            &format!("{}", 52 + i / 2),
            x + 4.0,
            top + 15.0,
            7.0,
            rgb(MUTED),
        );
    }

    let available = body_h - ruler_h;
    let rows = (available / ROW_H).floor() as usize;
    for row in 0..rows {
        let y = top + ruler_h + row as f32 * ROW_H;
        line(cv, (0.0, y + ROW_H - 1.0), (W, y + ROW_H - 1.0), rgb(LINE));
        text(
            cv,
            [
                "V  sky_plate.mp4",
                "G  Pulse rings",
                "T  NIGHT DRIVE",
                "S  City grid",
                "V  reflection.mp4",
                "V  traffic_pass.mp4",
            ][row % 6],
            8.0,
            y + 16.0,
            8.0,
            rgb(SUB),
        );
        if row < 6 {
            let start = controls_w + 18.0 + row as f32 * 48.0;
            let width = 410.0 - row as f32 * 22.0;
            fill(
                cv,
                Rect::from_xywh(start, y + 2.0, width, ROW_H - 4.0),
                rgb(CLIPS[row]),
            );
            text(
                cv,
                [
                    "sky_plate.mp4",
                    "Pulse rings",
                    "NIGHT DRIVE",
                    "City grid",
                    "reflection.mp4",
                    "traffic_pass.mp4",
                ][row],
                start + 9.0,
                y + 16.0,
                8.0,
                rgb(0x191919),
            );
            stroke(
                cv,
                Rect::from_xywh(start + 0.5, y + 2.5, width - 1.0, ROW_H - 5.0),
                rgb(INK),
            );
            text(cv, "z 0", 171.0, y + 16.0, 7.0, rgb(INK));
        }
    }

    let playhead_x = 676.0;
    line(cv, (playhead_x, top), (playhead_x, H), rgb(INK));
}

fn render(mode: RailMode, out: &str) {
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    fill(cv, Rect::from_xywh(0.0, 0.0, W, H), rgb(BG));
    draw_header(cv, mode);
    if mode != RailMode::Closed {
        draw_depth_rail(cv, mode);
        draw_timeline_body(cv, HEAD_H + DEPTH_H);
    } else {
        draw_timeline_body(cv, HEAD_H);
    }

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("png");
    std::fs::write(out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}

fn main() {
    let prefix = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-depth-rail".into());
    render(RailMode::Open, &format!("{prefix}-open.png"));
    render(RailMode::Distribute, &format!("{prefix}-distribute.png"));
    render(RailMode::Closed, &format!("{prefix}-closed.png"));
}
