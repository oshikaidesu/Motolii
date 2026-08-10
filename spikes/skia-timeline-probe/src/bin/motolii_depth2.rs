// Depth Rail — 2枚目。件数を縦量として描き、marker種別を1つへ畳む案。
//
// 前提fixture(root scope):
//   32 object / 27 が z=0 / うち 6 を選択(同一parent) / 表示範囲内の他 3
//   (plain=Z animation済み / Group 1 marker / Emitter 1 marker+粒子range) / 範囲外 左1 右1
//   Camera rankはEdit-Space Zと不一致 / 遮蔽は Layer Order
//
// 現行案との差:
//   - stack は文字pillでなく件数の棒。単独objectは同じ棒の高さ1。marker種別は1つ
//   - 選択は棒の中の塗り分け。「6はこの山の中」「21は動かない」が絵になる
//   - baselineの上=現在、下=提案。distributeは橙が棒から抜けて下段へ出る
//   - Camera rankは軸へ載せず、Z順と食い違う時だけ警告として出す
//   - ボタンclusterを状態で入れ替えない。Apply/Cancelは元から何も無い左列の空行へ出る

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const H: f32 = 256.0;
const HEAD_H: f32 = 28.0;
const RAIL_H: f32 = 46.0;
const LABEL_W: f32 = 200.0;
const TOOL_W: f32 = 116.0;
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
const WARN: u32 = 0xe0705f;
const BAR: u32 = 0x7f7f7f;
const CLIPS: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

const ROW_NAMES: [&str; 6] = [
    "sky_plate.mp4",
    "Pulse rings",
    "NIGHT DRIVE",
    "City grid",
    "reflection.mp4",
    "traffic_pass.mp4",
];

// 配布preview: 奥端 -0.36 / 手前端 +0.06 を利用者が指定した想定
const FAR_Z: f32 = -0.36;
const NEAR_Z: f32 = 0.06;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
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

fn fill_a(cv: &skia_safe::Canvas, rect: Rect, color: Color) {
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

fn line_a(cv: &skia_safe::Canvas, a: (f32, f32), b: (f32, f32), color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_stroke_width(1.2);
    paint.set_color(color);
    cv.draw_line(a, b, &paint);
}

fn with_alpha(v: u32, a: u8) -> Color {
    Color::from_argb(a, (v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

fn dotted(cv: &skia_safe::Canvas, y: f32, x0: f32, x1: f32, color: Color) {
    let mut x = x0;
    while x < x1 {
        line(cv, (x, y), ((x + 2.0).min(x1), y), color);
        x += 4.0;
    }
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

// カメラは固定でないため軸の上に載る。表示範囲は -0.5 .. +1.0
const Z_MIN: f32 = -0.5;
const Z_MAX: f32 = 1.0;
const CAM_NOW: f32 = 0.78;
const CAM_FROM: f32 = 0.95;
const CAM_TO: f32 = 0.62;

fn axis_x(z: f32) -> f32 {
    let left = LABEL_W + 28.0;
    let right = W - TOOL_W - 20.0;
    left + (z - Z_MIN) / (Z_MAX - Z_MIN) * (right - left)
}

// カメラ移動に対して画面上をどれだけ流れるか。z=0 を 1.00 とする相対量
fn flow(z: f32) -> f32 {
    (CAM_NOW - 0.0) / (CAM_NOW - z).max(0.02)
}

fn dist_z(i: usize) -> f32 {
    FAR_Z + (NEAR_Z - FAR_Z) * i as f32 / 5.0
}

// 棒の高さ: 件数の対数。1件=最小、32件=最大。100 layerでも頭打ちしない
fn count_h(n: f32) -> f32 {
    8.0 + 28.0 * (n.max(1.0).ln() / 32.0f32.ln())
}

fn draw_header(cv: &skia_safe::Canvas) {
    fill(cv, Rect::from_xywh(0.0, 0.0, W, HEAD_H), rgb(SURFACE));
    line(cv, (0.0, HEAD_H - 1.0), (W, HEAD_H - 1.0), rgb(LINE));
    let toggle = Rect::from_xywh(8.0, 3.0, 28.0, 22.0);
    fill(cv, toggle, rgb(0x4a4a4a));
    stroke(cv, toggle, rgb(INK));
    for offset in [-4.0, 0.0, 4.0] {
        line(cv, (17.0, 14.0 + offset), (27.0, 14.0 + offset), rgb(INK));
    }
    text(cv, "TIMELINE", 45.0, 18.0, 10.0, rgb(INK));
}

/// 常設の配布icon: 束ねられた3本が開く形。文字略号を置かない
fn distribute_icon(cv: &skia_safe::Canvas, x: f32, y: f32, armed: bool) {
    let rect = Rect::from_xywh(x, y, 36.0, 30.0);
    fill(cv, rect, rgb(if armed { 0x3a3125 } else { BG }));
    stroke(cv, rect, rgb(if armed { ACTIVE } else { LINE_2 }));
    let ink = rgb(if armed { ACTIVE } else { SUB });
    // 上段: 1点に束ねられた3本
    for dx in [-1.5, 0.0, 1.5] {
        line(cv, (x + 18.0 + dx, y + 7.0), (x + 18.0 + dx, y + 13.0), ink);
    }
    // 下段: 開いた3本
    for dx in [-9.0, 0.0, 9.0] {
        line(
            cv,
            (x + 18.0 + dx, y + 19.0),
            (x + 18.0 + dx, y + 25.0),
            ink,
        );
    }
    tri(
        cv,
        [
            (x + 18.0, y + 17.5),
            (x + 15.0, y + 14.5),
            (x + 21.0, y + 14.5),
        ],
        ink,
    );
}

fn fit_icon(cv: &skia_safe::Canvas, x: f32, y: f32) {
    let rect = Rect::from_xywh(x, y, 36.0, 30.0);
    fill(cv, rect, rgb(BG));
    stroke(cv, rect, rgb(LINE_2));
    let ink = rgb(SUB);
    line(cv, (x + 8.0, y + 10.0), (x + 8.0, y + 20.0), ink);
    line(cv, (x + 28.0, y + 10.0), (x + 28.0, y + 20.0), ink);
    line(cv, (x + 11.0, y + 15.0), (x + 25.0, y + 15.0), ink);
    tri(
        cv,
        [
            (x + 11.0, y + 15.0),
            (x + 15.0, y + 12.0),
            (x + 15.0, y + 18.0),
        ],
        ink,
    );
    tri(
        cv,
        [
            (x + 25.0, y + 15.0),
            (x + 21.0, y + 12.0),
            (x + 21.0, y + 18.0),
        ],
        ink,
    );
}

fn chip(cv: &skia_safe::Canvas, x: f32, y: f32, label: &str, border: u32, ink: u32) -> f32 {
    let w = text_w(label, 7.0) + 12.0;
    let rect = Rect::from_xywh(x, y, w, 15.0);
    fill(cv, rect, rgb(BG));
    stroke(cv, rect, rgb(border));
    text(cv, label, x + 6.0, y + 11.0, 7.0, rgb(ink));
    x + w
}

fn button(cv: &skia_safe::Canvas, x: f32, y: f32, label: &str, primary: bool) -> f32 {
    let w = text_w(label, 8.0) + 16.0;
    let rect = Rect::from_xywh(x, y, w, 17.0);
    fill(cv, rect, rgb(if primary { 0x3a3125 } else { BG }));
    stroke(cv, rect, rgb(if primary { ACTIVE } else { LINE_2 }));
    text(
        cv,
        label,
        x + 8.0,
        y + 12.0,
        8.0,
        rgb(if primary { ACTIVE } else { SUB }),
    );
    x + w + 6.0
}

fn draw_rail(cv: &skia_safe::Canvas, mode: Mode) {
    let top = HEAD_H;
    let ay = top + 22.0;
    let axis_l = axis_x(Z_MIN);
    let axis_r = axis_x(Z_MAX);
    let far = axis_x(FAR_Z);
    let near = axis_x(NEAR_Z);
    let sx = axis_x(0.0);

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

    // ── 左列 ──
    text(cv, "DEPTH", 10.0, top + 16.0, 9.5, rgb(INK));
    let scope = Rect::from_xywh(56.0, top + 5.0, 44.0, 15.0);
    fill(cv, scope, rgb(BG));
    stroke(cv, scope, rgb(SUB));
    text(cv, "ROOT", 61.0, top + 16.0, 7.0, rgb(INK));
    tri(
        cv,
        [(104.0, top + 10.0), (110.0, top + 10.0), (107.0, top + 14.0)],
        rgb(SUB),
    );
    if mode == Mode::Distribute {
        let x = button(cv, 10.0, top + 24.0, "Reverse", false);
        let x = button(cv, x, top + 24.0, "Cancel", false);
        button(cv, x, top + 24.0, "Apply", true);
    } else {
        text(cv, "6 selected", 10.0, top + 37.0, 8.5, rgb(ACTIVE));
        text(cv, "· same parent", 62.0, top + 37.0, 7.5, rgb(SUB));
    }

    // ── 軸 ──
    line(cv, (axis_l, ay), (axis_r, ay), rgb(LINE_2));

    // カメラの移動区間。ここへ素材を置くとカメラが通り抜ける
    let cf = axis_x(CAM_FROM);
    let ct = axis_x(CAM_TO);
    fill_a(
        cv,
        Rect::from_ltrb(ct, ay - 6.0, cf, ay + 6.0),
        Color::from_argb(38, 143, 160, 181),
    );
    line(cv, (ct, ay - 6.0), (ct, ay + 6.0), rgb(0x5d6b7a));
    line(cv, (cf, ay - 6.0), (cf, ay + 6.0), rgb(0x5d6b7a));
    // 進行方向
    let cx = axis_x(CAM_NOW);
    tri(
        cv,
        [(ct + 4.0, ay), (ct + 11.0, ay - 4.0), (ct + 11.0, ay + 4.0)],
        rgb(0x5d6b7a),
    );
    // カメラ本体(現在時刻)
    fill(
        cv,
        Rect::from_xywh(cx - 2.0, ay - 5.0, 10.0, 10.0),
        rgb(0x8fa0b5),
    );
    tri(
        cv,
        [(cx - 11.0, ay), (cx - 2.0, ay - 7.0), (cx - 2.0, ay + 7.0)],
        rgb(0x8fa0b5),
    );

    // ── 軸の下: 周囲。件数は重さ、識別は行が持つ ──
    for z in [-0.30f32, 0.17, 0.40] {
        let x = axis_x(z);
        line(cv, (x, ay + 2.0), (x, ay + 8.0), rgb(0x606060));
    }
    let pile = if mode == Mode::Distribute { 21 } else { 27 };
    fill(
        cv,
        Rect::from_xywh(sx - 2.5, ay + 2.0, 5.0, 10.0),
        rgb(0xc0c0c0),
    );
    text_c(cv, &format!("{pile}"), sx, ay + 21.0, 7.5, rgb(SUB));
    for dir in [-1.0f32, 1.0] {
        let x = if dir < 0.0 { axis_l - 7.0 } else { axis_r + 7.0 };
        if dir > 0.0 {
            continue;
        }
        for k in [0.0f32, 3.5] {
            tri(
                cv,
                [
                    (x + dir * (4.0 + k), ay),
                    (x + dir * k, ay - 4.5),
                    (x + dir * k, ay + 4.5),
                ],
                rgb(0x6e6e6e),
            );
        }
    }

    // ── 軸の上: 選択集合と配布区間 ──
    fill_a(
        cv,
        Rect::from_ltrb(far, ay - 15.0, near, ay - 2.0),
        Color::from_argb(if mode == Mode::Open { 20 } else { 38 }, 255, 173, 86),
    );
    for x in [far, near] {
        fill(cv, Rect::from_xywh(x - 1.5, ay - 18.0, 3.0, 19.0), rgb(ACTIVE));
    }
    if mode == Mode::Open {
        let r = Rect::from_xywh(sx - 9.0, ay - 15.0, 18.0, 13.0);
        fill(cv, r, rgb(ACTIVE));
        text_c(cv, "6", sx, ay - 5.0, 8.5, rgb(0x1a1a1a));
    } else {
        for (i, color) in CLIPS.iter().enumerate() {
            let x = axis_x(dist_z(i));
            let r = Rect::from_xywh(x - 5.0, ay - 15.0, 10.0, 13.0);
            fill(cv, r, rgb(*color));
            stroke(cv, r, rgb(ACTIVE));
        }
    }

    let ox = text_r(cv, "layer order", axis_r - 6.0, ay + 21.0, 6.5, rgb(MUTED));
    tri(
        cv,
        [
            (ox - 11.0, ay + 14.0),
            (ox - 16.0, ay + 22.0),
            (ox - 6.0, ay + 22.0),
        ],
        rgb(WARN),
    );

    // ── 右列 ──
    let bx = W - TOOL_W + 10.0;
    let br = Rect::from_xywh(bx, top + 6.0, 60.0, 18.0);
    fill(cv, br, rgb(if mode == Mode::Distribute { 0x3a3125 } else { BG }));
    stroke(
        cv,
        br,
        rgb(if mode == Mode::Distribute { ACTIVE } else { LINE_2 }),
    );
    for dx in [-4.0f32, 0.0, 4.0] {
        line(
            cv,
            (bx + 9.0 + dx, top + 10.0),
            (bx + 9.0 + dx * 2.2, top + 19.0),
            rgb(if mode == Mode::Distribute { ACTIVE } else { SUB }),
        );
    }
    text(
        cv,
        "Distribute",
        bx + 20.0,
        top + 18.0,
        7.5,
        rgb(if mode == Mode::Distribute { ACTIVE } else { SUB }),
    );
    let fr = Rect::from_xywh(bx + 66.0, top + 6.0, 30.0, 18.0);
    fill(cv, fr, rgb(BG));
    stroke(cv, fr, rgb(LINE_2));
    text(cv, "Fit", bx + 74.0, top + 18.0, 7.5, rgb(SUB));
    // Preserve Appearance は既定ON。切るときだけ意識させる
    let kb = Rect::from_xywh(bx, top + 28.0, 9.0, 9.0);
    fill(cv, kb, rgb(ACTIVE));
    line(cv, (bx + 2.0, top + 32.5), (bx + 4.0, top + 35.0), rgb(0x1a1a1a));
    line(cv, (bx + 4.0, top + 35.0), (bx + 7.5, top + 29.5), rgb(0x1a1a1a));
    text(cv, "keep look", bx + 13.0, top + 36.0, 7.0, rgb(SUB));
}

fn draw_body(cv: &skia_safe::Canvas, top: f32, mode: Mode) {
    let body_h = H - top;
    fill(cv, Rect::from_xywh(0.0, top, W, body_h), rgb(BG));
    fill(
        cv,
        Rect::from_xywh(0.0, top, LABEL_W, body_h),
        rgb(SURFACE_HI),
    );
    line(cv, (LABEL_W, top), (LABEL_W, H), rgb(LINE));

    let ruler_h = 20.0;
    fill(cv, Rect::from_xywh(0.0, top, W, ruler_h), rgb(SURFACE));
    text_r(cv, "depth", 148.0, top + 14.0, 6.5, rgb(MUTED));
    text_r(cv, "parallax", LABEL_W - 10.0, top + 14.0, 6.5, rgb(MUTED));
    text(cv, "TIME / BEAT", LABEL_W + 10.0, top + 14.0, 7.0, rgb(MUTED));
    for i in 0..9 {
        let x = LABEL_W + i as f32 * (W - LABEL_W) / 8.0;
        line(cv, (x, top), (x, H), rgb(0x2e2e2e));
        if i > 0 && i % 2 == 0 {
            text(
                cv,
                &format!("{}", 52 + i / 2),
                x + 5.0,
                top + 14.0,
                7.0,
                rgb(MUTED),
            );
        }
    }
    line(cv, (0.0, top + ruler_h), (W, top + ruler_h), rgb(LINE));

    let spans = [
        (0.03f32, 0.55f32),
        (0.10, 0.60),
        (0.17, 0.65),
        (0.25, 0.73),
        (0.32, 0.79),
        (0.40, 0.85),
    ];
    for row in 0..6 {
        let y = top + ruler_h + row as f32 * ROW_H;
        line(cv, (0.0, y), (W, y), rgb(0x2c2c2c));
        fill(
            cv,
            Rect::from_xywh(10.0, y + 7.0, 3.0, ROW_H - 13.0),
            rgb(CLIPS[row]),
        );
        text(cv, ROW_NAMES[row], 20.0, y + 16.0, 8.5, rgb(INK));

        let z = if mode == Mode::Open { 0.0 } else { dist_z(row) };
        let ink = if mode == Mode::Open { MUTED } else { ACTIVE };
        text_r(cv, &format!("{z:+.2}"), 148.0, y + 16.0, 7.0, rgb(ink));
        // 主役の数値。開いた直後は全行が同じ値で縦に揃う = 視差ゼロ
        text_r(
            cv,
            &format!("{:.2}x", flow(z)),
            LABEL_W - 10.0,
            y + 16.0,
            8.5,
            rgb(if mode == Mode::Open { INK } else { ACTIVE }),
        );

        let start = LABEL_W + spans[row].0 * (W - LABEL_W);
        let end = LABEL_W + spans[row].1 * (W - LABEL_W);
        let r = Rect::from_ltrb(start, y + 3.0, end, y + ROW_H - 3.0);
        fill(cv, r, rgb(CLIPS[row]));
        stroke(cv, r, rgb(INK));
        text(cv, ROW_NAMES[row], start + 8.0, y + 16.0, 8.0, rgb(0x191919));
    }

    let playhead = LABEL_W + 0.46 * (W - LABEL_W);
    line(cv, (playhead, top), (playhead, H), rgb(INK));
}

fn render(mode: Mode, out: &str) {
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    fill(cv, Rect::from_xywh(0.0, 0.0, W, H), rgb(BG));
    draw_header(cv);
    draw_rail(cv, mode);
    draw_body(cv, HEAD_H + RAIL_H, mode);

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
        .unwrap_or_else(|| "motolii-depth-rail-v10".into());
    render(Mode::Open, &format!("{prefix}-open.png"));
    render(Mode::Distribute, &format!("{prefix}-distribute.png"));
}
