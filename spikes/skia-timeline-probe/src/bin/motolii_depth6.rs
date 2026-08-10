// Depth Rail — 選択フォーカス案(利用者設計)。
//
// 利用者の設計:
//   - z=0 の既定シェイプは最初から灰色1個に統合され、個別に描かない
//   - タイムラインと一致し、現在時刻にあるシェイプのみ反映される
//   - ユーザが選択したシェイプだけが z 軸レーンでフォーカスされ、移動可能になる
//   - そこで初めて視差が出る
//
// AnimateParallax が使いやすかった構造(パネルへ入れたレイヤーだけが視覚viewに出る)を
// 「選択」に置き換える。レーンは全объектの地図ではなく、選択のための舞台。
//
// 上段: 開いた直後(選択6がz=0の灰色塊の上にフォーカス)
// 下段: drag後(6枚が塊から出て視差が生まれた。行に逸脱chipが生えた)

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const LABEL_W: f32 = 200.0;
const TOOL_W: f32 = 64.0;
const RAIL_H: f32 = 46.0;
const RULER_H: f32 = 18.0;
const ROW_H: f32 = 20.0;
const CAP_H: f32 = 16.0;
const SECTION_H: f32 = RAIL_H + RULER_H + 7.0 * ROW_H + 4.0;
const GAP: f32 = 16.0;
const H: f32 = 10.0 + (CAP_H + SECTION_H) * 2.0 + GAP + 8.0;

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

const ROW_NAMES: [&str; 7] = [
    "sky_plate.mp4",
    "Pulse rings",
    "NIGHT DRIVE",
    "City grid",
    "reflection.mp4",
    "traffic_pass.mp4",
    "intro_card.png",
];
const ROW7: u32 = 0x8b8b7d;

// drag後の配置
const SPREAD_Z: [f32; 6] = [-0.36, -0.298, -0.236, -0.174, -0.112, -0.05];

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Open,
    Moved,
}

fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn with_alpha(v: u32, a: u8) -> Color {
    Color::from_argb(a, (v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
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

fn font_of(size: f32) -> Font {
    let typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("typeface");
    Font::new(typeface, size)
}

fn text(cv: &skia_safe::Canvas, value: &str, x: f32, y: f32, size: f32, color: Color) {
    let font = font_of(size);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    cv.draw_str(value, (x, y), &font, &paint);
}

fn text_w(value: &str, size: f32) -> f32 {
    let font = font_of(size);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    font.measure_str(value, Some(&paint)).0
}

fn text_c(cv: &skia_safe::Canvas, value: &str, cx: f32, y: f32, size: f32, color: Color) {
    text(cv, value, cx - text_w(value, size) / 2.0, y, size, color);
}

fn text_r(cv: &skia_safe::Canvas, value: &str, right: f32, y: f32, size: f32, color: Color) -> f32 {
    let w = text_w(value, size);
    text(cv, value, right - w, y, size, color);
    right - w
}

fn tri(cv: &skia_safe::Canvas, pts: [(f32, f32); 3], color: Color) {
    let mut b = PathBuilder::new();
    b.move_to(pts[0]);
    b.line_to(pts[1]);
    b.line_to(pts[2]);
    b.close();
    let path = b.detach();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    cv.draw_path(&path, &paint);
}

fn diamond(cv: &skia_safe::Canvas, x: f32, y: f32, r: f32, color: Color) {
    tri(cv, [(x, y - r), (x + r, y), (x, y + r)], color);
    tri(cv, [(x, y - r), (x - r, y), (x, y + r)], color);
}

fn axis_x(z: f32) -> f32 {
    let left = LABEL_W + 28.0;
    let right = W - TOOL_W - 28.0;
    left + (z + 0.5) * (right - left)
}

fn draw_rail(cv: &skia_safe::Canvas, top: f32, state: State) {
    let ay = top + 26.0;
    let axis_l = axis_x(-0.5);
    let axis_r = axis_x(0.5);

    fill(cv, Rect::from_xywh(0.0, top, W, RAIL_H), rgb(SURFACE_LO));
    fill(cv, Rect::from_xywh(0.0, top, LABEL_W, RAIL_H), rgb(SURFACE));
    fill(
        cv,
        Rect::from_xywh(W - TOOL_W, top, TOOL_W, RAIL_H),
        rgb(SURFACE),
    );
    line(cv, (LABEL_W, top), (LABEL_W, top + RAIL_H), rgb(LINE));
    line(cv, (W - TOOL_W, top), (W - TOOL_W, top + RAIL_H), rgb(LINE));
    line(
        cv,
        (0.0, top + RAIL_H - 1.0),
        (W, top + RAIL_H - 1.0),
        rgb(LINE),
    );

    // 左列
    text(cv, "DEPTH", 10.0, top + 17.0, 9.5, rgb(INK));
    let scope = Rect::from_xywh(56.0, top + 6.0, 44.0, 15.0);
    fill(cv, scope, rgb(BG));
    stroke(cv, scope, rgb(SUB));
    text(cv, "ROOT", 61.0, top + 17.0, 7.0, rgb(INK));
    tri(
        cv,
        [(104.0, top + 11.0), (110.0, top + 11.0), (107.0, top + 15.0)],
        rgb(SUB),
    );
    text(cv, "6 selected", 10.0, top + 37.0, 8.5, rgb(ACTIVE));
    text(cv, "· same parent", 62.0, top + 37.0, 7.5, rgb(SUB));

    // 軸
    line(cv, (axis_l, ay), (axis_r, ay), rgb(LINE_2));
    text(cv, "BACK", axis_l, ay + 13.0, 6.5, rgb(MUTED));
    text_r(cv, "FRONT", axis_r, ay + 13.0, 6.5, rgb(MUTED));

    // 選択外は描き込まない:
    //   z=0 の既定シェイプ群 → 灰色1個に統合
    //   z を持ってしまったものだけが個別の小さな灰tickになる(逸脱時のみ個別化)
    for z in [-0.44f32, 0.17, 0.40] {
        let x = axis_x(z);
        line(cv, (x - 0.5, ay - 4.0), (x - 0.5, ay + 4.0), rgb(0x565656));
        line(cv, (x + 0.5, ay - 4.0), (x + 0.5, ay + 4.0), rgb(0x565656));
    }

    let sx = axis_x(0.0);
    let n = if state == State::Moved { 21 } else { 27 };
    let mass = Rect::from_xywh(sx - 14.0, ay - 8.0, 28.0, 16.0);
    fill(cv, mass, rgb(0x3a3a3a));
    stroke(cv, mass, rgb(0x4d4d4d));
    text_c(cv, &format!("{n}"), sx, ay + 3.5, 7.5, rgb(0x8a8a8a));

    // 選択がフォーカスを与える。ここだけが動かせる
    match state {
        State::Open => {
            // 6枚は今z=0: 塊の上に、行と同じ色のタブとして覗く
            let total = 6.0 * 6.0 - 1.0;
            let x0 = sx - total / 2.0;
            for (i, color) in CLIPS.iter().enumerate() {
                let x = x0 + i as f32 * 6.0;
                let r = Rect::from_xywh(x, ay - 20.0, 5.0, 11.0);
                fill(cv, r, rgb(*color));
                stroke(cv, r, rgb(ACTIVE));
            }
        }
        State::Moved => {
            fill(
                cv,
                Rect::from_ltrb(axis_x(SPREAD_Z[0]), ay - 1.0, axis_x(SPREAD_Z[5]), ay + 1.0),
                with_alpha(ACTIVE, 70),
            );
            for (i, color) in CLIPS.iter().enumerate() {
                let x = axis_x(SPREAD_Z[i]);
                let r = Rect::from_xywh(x - 3.0, ay - 9.0, 6.0, 18.0);
                fill(cv, r, rgb(*color));
                stroke(cv, r, rgb(ACTIVE));
            }
        }
    }

    // 右列: 常設の入口だけ
    let tx = W - TOOL_W + 8.0;
    let dr = Rect::from_xywh(tx, top + 8.0, 22.0, 16.0);
    fill(cv, dr, rgb(BG));
    stroke(cv, dr, rgb(LINE_2));
    for dx in [-4.0f32, 0.0, 4.0] {
        line(
            cv,
            (tx + 11.0 + dx * 0.4, top + 11.0),
            (tx + 11.0 + dx * 1.8, top + 21.0),
            rgb(SUB),
        );
    }
    let fr = Rect::from_xywh(tx + 26.0, top + 8.0, 22.0, 16.0);
    fill(cv, fr, rgb(BG));
    stroke(cv, fr, rgb(LINE_2));
    line(cv, (tx + 32.0, top + 12.0), (tx + 32.0, top + 20.0), rgb(SUB));
    line(cv, (tx + 42.0, top + 12.0), (tx + 42.0, top + 20.0), rgb(SUB));
}

fn draw_timeline(cv: &skia_safe::Canvas, top: f32, state: State) {
    let body_h = RULER_H + 7.0 * ROW_H + 4.0;
    fill(cv, Rect::from_xywh(0.0, top, W, body_h), rgb(BG));
    fill(
        cv,
        Rect::from_xywh(0.0, top, LABEL_W, body_h),
        rgb(SURFACE_HI),
    );
    line(cv, (LABEL_W, top), (LABEL_W, top + body_h), rgb(LINE));

    fill(cv, Rect::from_xywh(0.0, top, W, RULER_H), rgb(SURFACE));
    text(cv, "TIME / BEAT", LABEL_W + 10.0, top + 13.0, 7.0, rgb(MUTED));
    for i in 0..9 {
        let x = LABEL_W + i as f32 * (W - LABEL_W) / 8.0;
        line(cv, (x, top), (x, top + body_h), rgb(0x2e2e2e));
        if i > 0 && i % 2 == 0 {
            text(
                cv,
                &format!("{}", 52 + i / 2),
                x + 5.0,
                top + 13.0,
                7.0,
                rgb(MUTED),
            );
        }
    }
    line(cv, (0.0, top + RULER_H), (W, top + RULER_H), rgb(LINE));

    let spans: [(f32, f32); 7] = [
        (0.03, 0.55),
        (0.10, 0.60),
        (0.17, 0.65),
        (0.25, 0.73),
        (0.32, 0.79),
        (0.40, 0.85),
        (0.02, 0.32),
    ];
    for row in 0..7 {
        let y = top + RULER_H + row as f32 * ROW_H;
        line(cv, (0.0, y), (W, y), rgb(0x2c2c2c));
        let color = if row < 6 { CLIPS[row] } else { ROW7 };
        fill(
            cv,
            Rect::from_xywh(10.0, y + 5.0, 3.0, ROW_H - 10.0),
            rgb(color),
        );
        text(cv, ROW_NAMES[row], 20.0, y + 14.0, 8.0, rgb(INK));

        // 逸脱chip: zを持った時だけ生まれる
        if state == State::Moved && row < 6 {
            text_r(
                cv,
                &format!("z {:+.2}", SPREAD_Z[row]),
                LABEL_W - 8.0,
                y + 14.0,
                7.0,
                rgb(SUB),
            );
        }

        let start = LABEL_W + spans[row].0 * (W - LABEL_W);
        let end = LABEL_W + spans[row].1 * (W - LABEL_W);
        let r = Rect::from_ltrb(start, y + 3.0, end, y + ROW_H - 3.0);
        fill(cv, r, rgb(color));
        if row < 6 {
            stroke(
                cv,
                Rect::from_ltrb(start - 1.0, y + 2.0, end + 1.0, y + ROW_H - 2.0),
                rgb(INK),
            );
        }
        text(cv, ROW_NAMES[row], start + 8.0, y + 14.0, 7.5, rgb(0x191919));
    }

    let playhead = LABEL_W + 0.46 * (W - LABEL_W);
    line(cv, (playhead, top + RULER_H), (playhead, top + body_h), rgb(INK));
}

fn main() {
    let prefix = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-depth-rail-v14".into());
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    fill(cv, Rect::from_xywh(0.0, 0.0, W, H), rgb(BG));

    let mut y = 10.0;
    for (caption, state) in [
        (
            "open - the z=0 default mass is ONE gray block. only the selected 6 gain focus (color tabs, movable). intro_card is not under the playhead so it never enters the lane",
            State::Open,
        ),
        (
            "after drag - the 6 left the mass; parallax is born here. rows grow z chips (deviation only). the 21 in the mass never moved",
            State::Moved,
        ),
    ] {
        text(cv, caption, 10.0, y + 10.0, 8.0, rgb(MUTED));
        draw_rail(cv, y + CAP_H, state);
        draw_timeline(cv, y + CAP_H + RAIL_H, state);
        y += CAP_H + SECTION_H + GAP;
    }

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("png");
    let out = format!("{prefix}.png");
    std::fs::write(&out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
