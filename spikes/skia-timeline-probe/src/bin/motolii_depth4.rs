// Depth Rail — 縦案(Fable)。深度を時間と直交させる。
//
// 診断: v6〜v11は全部、奥行きを時間と同じ横向きに描いていた。Timeline上の横帯は
// この画面の文法では「時間の仲間」としか読めない。利用者自身が「巨大stack」と
// 呼んでいたものは縦に積むものであり、層パネルの文法(上=手前、下=奥)は
// AE / Photoshop / クリスタ / AM の利用者全員が既に手に持っている。
// Adobe Animate Layer Depth も Disney のマルチプレーンカメラも深度は縦である。
//
// この案: Depth Rail = 層リストを実距離つきの縦の柱にしたもの。
//   上にカメラが浮き(移動レールつき)、その下へ板が沈む。
//   z=0 の27枚は文字どおり「積まれたカードの束」。選択6枚は束から覗くタブ。
//   配布は床でなく「棚」— 2本の橙テープの間へカードが層順のまま降りて並ぶ。
//
// 左panel=開いた直後 / 右panel=配布preview中。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const PW: f32 = 190.0; // panel幅
const PH: f32 = 330.0; // panel高 = Timeline本体と同じ高さを借りる想定
const GAP: f32 = 26.0;
const W: f32 = GAP * 3.0 + PW * 2.0;
const H: f32 = PH + 44.0;

const BG: u32 = 0x242424;
const SURFACE: u32 = 0x2a2a2a;
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

// 空間fixture。上=手前(カメラ)、下=奥
const Z_TOP: f32 = 1.05;
const Z_BOT: f32 = -0.55;
const CAM_NOW: f32 = 0.82;
const CAM_FROM: f32 = 0.95;
const CAM_TO: f32 = 0.62;
const FAR_Z: f32 = -0.42;
const NEAR_Z: f32 = -0.06;
const HAZE_Z: f32 = -0.50;
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

fn dotted_v(cv: &skia_safe::Canvas, x: f32, y0: f32, y1: f32, color: Color) {
    let mut y = y0;
    while y < y1 {
        line(cv, (x, y), (x, (y + 2.0).min(y1)), color);
        y += 4.0;
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

fn mini_button(cv: &skia_safe::Canvas, x: f32, y: f32, label: &str, primary: bool) -> f32 {
    let w = text_w(label, 7.5) + 12.0;
    let rect = Rect::from_xywh(x, y, w, 15.0);
    fill(cv, rect, rgb(if primary { 0x3a3125 } else { BG }));
    stroke(cv, rect, rgb(if primary { ACTIVE } else { LINE_2 }));
    text(
        cv,
        label,
        x + 6.0,
        y + 11.0,
        7.5,
        rgb(if primary { ACTIVE } else { SUB }),
    );
    x + w + 5.0
}

fn draw_panel(cv: &skia_safe::Canvas, px: f32, py: f32, mode: Mode) {
    let inner_top = py + 46.0; // header下
    let inner_bot = py + PH - 26.0;
    let z_y = |z: f32| inner_top + (Z_TOP - z) / (Z_TOP - Z_BOT) * (inner_bot - inner_top);
    let card_l = px + 26.0;
    let card_r = px + PW - 40.0; // 右はカメラの移動レール用
    let card_c = (card_l + card_r) / 2.0;

    fill(cv, Rect::from_xywh(px, py, PW, PH), rgb(SURFACE_LO));
    stroke(cv, Rect::from_xywh(px, py, PW, PH), rgb(LINE));

    // ── header ──
    fill(cv, Rect::from_xywh(px, py, PW, 24.0), rgb(SURFACE));
    line(cv, (px, py + 24.0), (px + PW, py + 24.0), rgb(LINE));
    text(cv, "DEPTH", px + 8.0, py + 16.0, 9.0, rgb(INK));
    let scope = Rect::from_xywh(px + 50.0, py + 5.0, 40.0, 14.0);
    fill(cv, scope, rgb(BG));
    stroke(cv, scope, rgb(SUB));
    text(cv, "ROOT", px + 55.0, py + 15.0, 6.5, rgb(INK));
    tri(
        cv,
        [
            (px + 94.0, py + 9.0),
            (px + 100.0, py + 9.0),
            (px + 97.0, py + 13.0),
        ],
        rgb(SUB),
    );
    // Distribute入口(常設)。armedで橙
    let armed = mode == Mode::Distribute;
    let dr = Rect::from_xywh(px + PW - 42.0, py + 4.0, 16.0, 16.0);
    fill(cv, dr, rgb(if armed { 0x3a3125 } else { BG }));
    stroke(cv, dr, rgb(if armed { ACTIVE } else { LINE_2 }));
    let dc = rgb(if armed { ACTIVE } else { SUB });
    for dy in [-3.0f32, 0.0, 3.0] {
        line(
            cv,
            (px + PW - 38.0, py + 12.0 + dy * 0.4),
            (px + PW - 30.0, py + 12.0 + dy * 2.0),
            dc,
        );
    }
    let fr = Rect::from_xywh(px + PW - 22.0, py + 4.0, 16.0, 16.0);
    fill(cv, fr, rgb(BG));
    stroke(cv, fr, rgb(LINE_2));
    line(cv, (px + PW - 18.0, py + 8.0), (px + PW - 18.0, py + 16.0), rgb(SUB));
    line(cv, (px + PW - 10.0, py + 8.0), (px + PW - 10.0, py + 16.0), rgb(SUB));

    // ── 選択の要約 ──
    if mode == Mode::Open {
        text(cv, "6 selected", px + 8.0, py + 38.0, 8.0, rgb(ACTIVE));
        text(cv, "· same parent", px + 58.0, py + 38.0, 7.0, rgb(SUB));
    } else {
        let x = mini_button(cv, px + 8.0, py + 28.0, "Rev", false);
        let x = mini_button(cv, x, py + 28.0, "Cancel", false);
        mini_button(cv, x, py + 28.0, "Apply", true);
    }

    // ── カメラの移動レール(右縁)。矢印=進行方向(寄っていく=下へ) ──
    let track_x = px + PW - 16.0;
    line(cv, (track_x, z_y(CAM_FROM)), (track_x, z_y(CAM_TO)), rgb(0x5d6b7a));
    for y in [z_y(CAM_FROM), z_y(CAM_TO)] {
        line(cv, (track_x - 2.0, y), (track_x + 2.0, y), rgb(0x5d6b7a));
    }
    tri(
        cv,
        [
            (track_x, z_y(CAM_TO) + 1.0),
            (track_x - 3.5, z_y(CAM_TO) - 5.0),
            (track_x + 3.5, z_y(CAM_TO) - 5.0),
        ],
        rgb(0x8fa0b5),
    );

    // ── カメラ。上に浮いて下を向いている ──
    let cy = z_y(CAM_NOW);
    let cone_bot = z_y(0.02); // 視錐は束のすぐ上まで
    for end_x in [card_l - 6.0, card_r + 6.0] {
        line_a(cv, (card_c, cy + 8.0), (end_x, cone_bot), with_alpha(CAM, 46));
    }
    circle(cv, card_c - 4.0, cy - 8.0, 3.0, rgb(CAM));
    circle(cv, card_c + 3.0, cy - 8.0, 3.0, rgb(CAM));
    fill(cv, Rect::from_xywh(card_c - 7.0, cy - 6.0, 14.0, 9.0), rgb(CAM));
    tri(
        cv,
        [
            (card_c, cy + 9.0),
            (card_c - 4.5, cy + 3.0),
            (card_c + 4.5, cy + 3.0),
        ],
        rgb(CAM),
    );
    line(cv, (card_c + 7.0, cy - 1.5), (track_x, cy - 1.5), rgb(0x46525f));

    // ── カメラの後ろ(上端): 描画されない1枚 ──
    fill(
        cv,
        Rect::from_ltrb(card_c - 14.0, py + 30.0, card_c + 14.0, py + 32.0),
        rgb(0x4a4a4a),
    );
    text(cv, "1", card_c + 19.0, py + 34.0, 6.5, rgb(MUTED));

    // ── 単独の板: 横のカード ──
    // Sparks: 粒子の評価済み帯 + Emitter 1枚
    fill(
        cv,
        Rect::from_ltrb(card_l + 8.0, z_y(0.47), card_r - 8.0, z_y(0.33)),
        with_alpha(0x919191, 40),
    );
    fill(
        cv,
        Rect::from_ltrb(card_l, z_y(EMIT_Z) - 1.0, card_r, z_y(EMIT_Z) + 1.0),
        rgb(SLAT),
    );
    text(cv, "Sparks ~240", card_l, z_y(EMIT_Z) - 5.0, 6.5, rgb(MUTED));

    // Skyline Group: 親側では1枚(二重線)
    for dy in [-1.5f32, 1.5] {
        fill(
            cv,
            Rect::from_ltrb(card_l, z_y(GROUP_Z) + dy - 0.8, card_r, z_y(GROUP_Z) + dy + 0.8),
            rgb(SLAT),
        );
    }
    text(cv, "Skyline (5)", card_l, z_y(GROUP_Z) - 6.0, 6.5, rgb(MUTED));

    // ── z=0 の束。積まれたカード ──
    let sy = z_y(0.0);
    let deck_n = if mode == Mode::Distribute { 21 } else { 27 };
    let layers = if mode == Mode::Distribute { 4 } else { 5 };
    for i in 0..layers {
        let y = sy - (layers as f32 - 1.0) + i as f32 * 2.0;
        fill(cv, Rect::from_ltrb(card_l, y - 0.8, card_r, y + 0.8), rgb(0xa8a8a8));
    }
    text(
        cv,
        &format!("{deck_n}"),
        card_r + 5.0,
        sy + 3.0,
        8.0,
        rgb(if mode == Mode::Open { INK } else { SUB }),
    );
    if mode == Mode::Open {
        // 選択6枚は束から覗くタブ
        for (i, color) in CLIPS.iter().enumerate() {
            let x = card_l + 14.0 + i as f32 * ((card_r - card_l - 28.0) / 5.0);
            fill(cv, Rect::from_xywh(x - 4.0, sy - 9.0, 8.0, 5.0), rgb(*color));
            stroke(cv, Rect::from_xywh(x - 4.0, sy - 9.0, 8.0, 5.0), rgb(ACTIVE));
        }
        text_c(cv, "6 in stack", card_c, sy + 14.0, 6.5, rgb(ACTIVE));
    }

    // ── 配布の棚: 2本の橙テープの間へ層順で降りる ──
    if mode == Mode::Distribute {
        for z in [NEAR_Z, FAR_Z] {
            let y = z_y(z);
            line(cv, (card_l - 6.0, y), (card_r + 6.0, y), rgb(ACTIVE));
            fill(cv, Rect::from_xywh(card_l - 10.0, y - 2.5, 5.0, 5.0), rgb(ACTIVE));
        }
        // 層順のまま: 上(手前)=traffic_pass(行6) … 下(奥)=sky_plate(行1)
        for (i, color) in CLIPS.iter().enumerate() {
            let z = NEAR_Z + (FAR_Z - NEAR_Z) * i as f32 / 5.0;
            let y = z_y(z);
            let idx = 5 - i; // 手前側から行の逆順
            fill(
                cv,
                Rect::from_ltrb(card_l + 4.0, y - 2.0, card_r - 4.0, y + 2.0),
                rgb(CLIPS[idx]),
            );
            let _ = color;
        }
    }

    // ── haze: Z animation済み。可動域は縦の点線 ──
    dotted_v(cv, card_l - 8.0, z_y(-0.38), z_y(-0.55) - 0.0, rgb(0x6a6a6a));
    fill(
        cv,
        Rect::from_ltrb(card_l, z_y(HAZE_Z) - 1.0, card_r, z_y(HAZE_Z) + 1.0),
        rgb(SLAT),
    );
    text(cv, "haze", card_l, z_y(HAZE_Z) - 5.0, 6.5, rgb(MUTED));

    // ── 下端: 表示範囲より奥の1枚 ──
    fill(
        cv,
        Rect::from_ltrb(card_c - 14.0, py + PH - 8.0, card_c + 14.0, py + PH - 6.0),
        rgb(0x4a4a4a),
    );
    text(cv, "1", card_c + 19.0, py + PH - 5.0, 6.5, rgb(MUTED));

    // ── 遮蔽の決まり方。Z順と食い違う時だけ ──
    let ox = text_r(cv, "layer order", px + PW - 8.0, py + PH - 12.0, 6.5, rgb(WARN));
    tri(
        cv,
        [
            (ox - 5.0, py + PH - 18.0),
            (ox - 9.0, py + PH - 12.0),
            (ox - 1.0, py + PH - 12.0),
        ],
        rgb(WARN),
    );
}

fn main() {
    let prefix = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-depth-rail-v12".into());
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    fill(cv, Rect::from_xywh(0.0, 0.0, W, H), rgb(BG));

    draw_panel(cv, GAP, 30.0, Mode::Open);
    draw_panel(cv, GAP * 2.0 + PW, 30.0, Mode::Distribute);
    text(cv, "open", GAP, 20.0, 8.0, rgb(MUTED));
    text(cv, "distribute preview", GAP * 2.0 + PW, 20.0, 8.0, rgb(MUTED));

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("png");
    let out = format!("{prefix}.png");
    std::fs::write(&out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
