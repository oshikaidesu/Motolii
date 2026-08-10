// Depth Rail — ジオラマ案(Fable)。チャートをやめ、横から見た小さな舞台として描く。
//
// 診断: v6〜v10が外れたのは全部「チャート」だったから。位置・棒の高さ・係数という
// 記法は読んで翻訳する必要がある。奥行きを翻訳なしで見せる絵はジオラマ(側面図)だけ。
// Harmony Side View / ディズニーのマルチプレーン図解と同じ構図。
//
// 語彙は3つだけ:
//   板   = object。床に立つ。同一Zはデッキ(密着した束)になり、束の厚みが件数
//   床   = 奥行き。目盛り・数字・BACK/FRONTを置かない。前後はカメラが決める
//   カメラ = カメラの形をしたもの。視錐で「どちらを向いているか」、
//            床下のレールで「どこからどこまで動くか」を言う
//
// fixture(root scope):
//   32 object / 27がz=0のデッキ / うち6選択(同一parent、デッキ内で橙・背高)
//   haze(Z animation済み・床に可動域) / Skyline Group(1枚扱い) / Sparks Emitter(粒子帯)
//   範囲外 左1 / カメラの後ろ1 / カメラは移動する(レール+進行方向)
//   遮蔽はLayer OrderでZ順と食い違う → 警告
//
// distribute: デッキから橙6枚が層順を保って左(奥)へ滑り出る。行のzが追従する。
// Preserve Appearance(keep look)は既定ON。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const H: f32 = 268.0;
const HEAD_H: f32 = 28.0;
const RAIL_H: f32 = 60.0;
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
const SLAT: u32 = 0x8a8a8a;
const CAM: u32 = 0x9db0c6;
const CLIPS: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

const ROW_NAMES: [&str; 6] = [
    "sky_plate.mp4",
    "Pulse rings",
    "NIGHT DRIVE",
    "City grid",
    "reflection.mp4",
    "traffic_pass.mp4",
];

// 空間fixture。カメラは+0.78にいて、0.95→0.62へ寄っていく
const Z_MIN: f32 = -0.5;
const Z_MAX: f32 = 1.0;
const CAM_NOW: f32 = 0.78;
const CAM_FROM: f32 = 0.95;
const CAM_TO: f32 = 0.62;
// 配布区間: 奥端-0.36、手前端-0.05(最前列はほぼ元の平面に残す)
const FAR_Z: f32 = -0.36;
const NEAR_Z: f32 = -0.05;
const HAZE_Z: f32 = -0.44;
const GROUP_Z: f32 = 0.17;
const EMIT_Z: f32 = 0.40;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Open,
    Distribute,
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

fn line_a(cv: &skia_safe::Canvas, a: (f32, f32), b: (f32, f32), color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_stroke_width(1.0);
    paint.set_color(color);
    cv.draw_line(a, b, &paint);
}

fn circle(cv: &skia_safe::Canvas, x: f32, y: f32, r: f32, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    cv.draw_circle((x, y), r, &paint);
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

fn axis_x(z: f32) -> f32 {
    let left = LABEL_W + 28.0;
    let right = W - TOOL_W - 20.0;
    left + (z - Z_MIN) / (Z_MAX - Z_MIN) * (right - left)
}

fn dist_z(i: usize) -> f32 {
    FAR_Z + (NEAR_Z - FAR_Z) * i as f32 / 5.0
}

/// 床に立つ板。ジオラマの唯一の登場物
fn slat(cv: &skia_safe::Canvas, x: f32, ground: f32, h: f32, w: f32, color: Color) {
    fill(cv, Rect::from_xywh(x - w / 2.0, ground - h, w, h), color);
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

fn draw_rail(cv: &skia_safe::Canvas, mode: Mode) {
    let top = HEAD_H;
    let ground = top + 42.0; // 床。板はこの上に立つ
    let slat_h = 20.0;
    let strip_l = LABEL_W + 10.0;
    let strip_r = W - TOOL_W - 8.0;

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
        let x = button(cv, 10.0, top + 28.0, "Reverse", false);
        let x = button(cv, x, top + 28.0, "Cancel", false);
        button(cv, x, top + 28.0, "Apply", true);
    } else {
        text(cv, "6 selected", 10.0, top + 39.0, 8.5, rgb(ACTIVE));
        text(cv, "· same parent", 62.0, top + 39.0, 7.5, rgb(SUB));
    }

    let cam_x = axis_x(CAM_NOW);
    let lens_tip = cam_x - 10.0;

    // ── カメラの視錐。床より先に描いて、板の下に敷く ──
    for end_y in [ground - slat_h - 8.0, ground] {
        line_a(cv, (lens_tip, ground - 11.0), (strip_l, end_y), with_alpha(CAM, 46));
    }

    // ── 床。目盛りも数字も持たない ──
    line(cv, (strip_l, ground), (strip_r, ground), rgb(LINE_2));

    // カメラの移動レール(床下)。矢印が進行方向
    let track_l = axis_x(CAM_TO);
    let track_r = axis_x(CAM_FROM);
    line(cv, (track_l, ground + 6.0), (track_r, ground + 6.0), rgb(0x5d6b7a));
    for x in [track_l, track_r] {
        line(cv, (x, ground + 4.0), (x, ground + 8.0), rgb(0x5d6b7a));
    }
    tri(
        cv,
        [
            (track_l - 1.0, ground + 6.0),
            (track_l + 6.0, ground + 2.5),
            (track_l + 6.0, ground + 9.5),
        ],
        rgb(0x8fa0b5),
    );

    // ── 配布区間: 床に貼ったテープと2本の柱。開いた直後は出さない ──
    let far = axis_x(FAR_Z);
    let near = axis_x(NEAR_Z);
    if mode == Mode::Distribute {
        fill(
            cv,
            Rect::from_ltrb(far, ground + 1.0, near, ground + 4.0),
            with_alpha(ACTIVE, 160),
        );
        for x in [far, near] {
            fill(
                cv,
                Rect::from_xywh(x - 1.5, ground - slat_h - 4.0, 3.0, slat_h + 5.0),
                rgb(ACTIVE),
            );
        }
    }

    // ── z=0 のデッキ。板の束そのもの。厚みが件数 ──
    let sx = axis_x(0.0);
    let selected_in_deck = [4usize, 8, 12, 16, 20, 24];
    let n = 27usize;
    let spacing = 1.8f32;
    let x0 = sx - (n as f32 - 1.0) * spacing / 2.0;
    let mut drawn = 0usize;
    for i in 0..n {
        let is_sel = selected_in_deck.contains(&i);
        if mode == Mode::Distribute && is_sel {
            continue; // 抜けた6枚は配布先に立っている
        }
        let x = x0 + i as f32 * spacing;
        if is_sel {
            slat(cv, x, ground, slat_h + 5.0, 1.2, rgb(ACTIVE));
        } else {
            slat(cv, x, ground, slat_h, 1.2, rgb(SLAT));
        }
        drawn += 1;
    }
    text_c(
        cv,
        &format!("{drawn}"),
        sx,
        ground - slat_h - 11.0,
        7.5,
        rgb(if mode == Mode::Open { INK } else { SUB }),
    );

    // ── 配布preview: 橙6枚が層順を保って区間へ滑り出た姿 ──
    if mode == Mode::Distribute {
        for (i, color) in CLIPS.iter().enumerate() {
            let x = axis_x(dist_z(i));
            slat(cv, x, ground, slat_h + 5.0, 5.0, rgb(*color));
            stroke(
                cv,
                Rect::from_xywh(x - 2.5, ground - slat_h - 5.0, 5.0, slat_h + 5.0),
                rgb(ACTIVE),
            );
        }
    }

    // ── 周囲の単独板。名前は板の上に小さく ──
    let hx = axis_x(HAZE_Z);
    dotted(
        cv,
        ground - 2.0,
        axis_x(-0.50),
        axis_x(-0.38),
        rgb(0x6a6a6a),
    ); // hazeのZ可動域は床の上の点線
    slat(cv, hx, ground, slat_h, 2.0, rgb(SLAT));
    text_c(cv, "haze", hx, ground - slat_h - 4.0, 6.5, rgb(MUTED));

    let gx = axis_x(GROUP_Z);
    slat(cv, gx - 1.5, ground, slat_h, 2.0, rgb(SLAT));
    slat(cv, gx + 1.5, ground, slat_h, 2.0, rgb(SLAT));
    text_c(cv, "Skyline (5)", gx, ground - slat_h - 4.0, 6.5, rgb(MUTED));

    let ex = axis_x(EMIT_Z);
    fill(
        cv,
        Rect::from_ltrb(axis_x(0.33), ground - 3.0, axis_x(0.47), ground - 1.0),
        with_alpha(0x919191, 90),
    ); // 粒子の評価済みDepth帯も床の上
    slat(cv, ex, ground, slat_h, 2.0, rgb(SLAT));
    text_c(cv, "Sparks ~240", ex, ground - slat_h - 4.0, 6.5, rgb(MUTED));

    // ── カメラ。カメラの形をしている ──
    circle(cv, cam_x + 3.0, ground - 18.0, 3.0, rgb(CAM));
    circle(cv, cam_x + 10.0, ground - 18.0, 3.0, rgb(CAM));
    fill(cv, Rect::from_xywh(cam_x, ground - 16.0, 13.0, 11.0), rgb(CAM));
    tri(
        cv,
        [
            (lens_tip, ground - 11.0),
            (cam_x + 1.0, ground - 15.0),
            (cam_x + 1.0, ground - 7.0),
        ],
        rgb(CAM),
    );
    line(cv, (cam_x + 6.0, ground - 5.0), (cam_x + 6.0, ground), rgb(CAM));

    // ── 見えていないもの: 左端の奥1、カメラの後ろ1 ──
    slat(cv, strip_l + 2.0, ground, slat_h * 0.7, 2.0, rgb(0x555555));
    text_c(cv, "1", strip_l + 2.0, ground - slat_h * 0.7 - 4.0, 6.5, rgb(MUTED));
    let bx = strip_r - 4.0;
    slat(cv, bx, ground, slat_h * 0.7, 2.0, rgb(0x555555));
    text_c(cv, "1", bx, ground - slat_h * 0.7 - 4.0, 6.5, rgb(MUTED));

    // ── 遮蔽の決まり方。Z順と食い違う時だけ警告 ──
    let ox = text_r(cv, "layer order", strip_r, ground + 16.0, 6.5, rgb(WARN));
    tri(
        cv,
        [
            (ox - 5.0, ground + 9.5),
            (ox - 9.0, ground + 16.0),
            (ox - 1.0, ground + 16.0),
        ],
        rgb(WARN),
    );

    // ── 右列 ──
    let tx = W - TOOL_W + 10.0;
    let br = Rect::from_xywh(tx, top + 6.0, 60.0, 18.0);
    fill(cv, br, rgb(if mode == Mode::Distribute { 0x3a3125 } else { BG }));
    stroke(
        cv,
        br,
        rgb(if mode == Mode::Distribute { ACTIVE } else { LINE_2 }),
    );
    for dx in [-4.0f32, 0.0, 4.0] {
        line(
            cv,
            (tx + 9.0 + dx, top + 10.0),
            (tx + 9.0 + dx * 2.2, top + 19.0),
            rgb(if mode == Mode::Distribute { ACTIVE } else { SUB }),
        );
    }
    text(
        cv,
        "Distribute",
        tx + 20.0,
        top + 18.0,
        7.5,
        rgb(if mode == Mode::Distribute { ACTIVE } else { SUB }),
    );
    let fr = Rect::from_xywh(tx + 66.0, top + 6.0, 30.0, 18.0);
    fill(cv, fr, rgb(BG));
    stroke(cv, fr, rgb(LINE_2));
    text(cv, "Fit", tx + 74.0, top + 18.0, 7.5, rgb(SUB));
    // Preserve Appearance は既定ON
    let kb = Rect::from_xywh(tx, top + 32.0, 9.0, 9.0);
    fill(cv, kb, rgb(ACTIVE));
    line(cv, (tx + 2.0, top + 36.5), (tx + 4.0, top + 39.0), rgb(0x1a1a1a));
    line(cv, (tx + 4.0, top + 39.0), (tx + 7.5, top + 33.5), rgb(0x1a1a1a));
    text(cv, "keep look", tx + 13.0, top + 40.0, 7.0, rgb(SUB));
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

        let (z_label, ink) = match mode {
            Mode::Open => ("z 0".to_string(), MUTED),
            Mode::Distribute => (format!("z {:+.2}", dist_z(row)), ACTIVE),
        };
        text_r(cv, &z_label, LABEL_W - 10.0, y + 16.0, 7.5, rgb(ink));

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
        .unwrap_or_else(|| "motolii-depth-rail-v11".into());
    render(Mode::Open, &format!("{prefix}-open.png"));
    render(Mode::Distribute, &format!("{prefix}-distribute.png"));
}
