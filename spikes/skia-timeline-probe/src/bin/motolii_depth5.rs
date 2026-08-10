// Depth — 段差ゼロ案(Fable)。深度の面を作らない。
//
// 原則(利用者): 「zはユーザにとって避けたいもの。画面は平面なのに奥行きを
// 求められるから。情報量を増やすのではなく、いかに段差をなくせるか」
//
// 適用: 規範「状態語は逸脱時のみ表示し、正常・不変の既定表示は沈黙とする」を
// 深度そのものへ適用する。
//   1. 平らなprojectに深度UIは存在しない
//   2. 操作は 選択 → Spread → 量を1つドラッグ。層順が順番を、keep lookが構図を吸収する
//   3. 操作後は行に小さな逸脱chipが残るだけ。深度が在って初めて表示が生まれる
//
// 3段で1枚: flat(沈黙) / Spread gesture中 / 確定後。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const LABEL_W: f32 = 200.0;
const ROW_H: f32 = 20.0;
const RULER_H: f32 = 18.0;
const CAP_H: f32 = 16.0;
const STRIP_H: f32 = 148.0; // ruler + 6行 + 余白
const GAP: f32 = 14.0;
const H: f32 = 10.0 + (CAP_H + STRIP_H) * 3.0 + GAP * 2.0 + 8.0;

const BG: u32 = 0x242424;
const SURFACE: u32 = 0x2a2a2a;
const SURFACE_HI: u32 = 0x363636;
const LINE: u32 = 0x111111;
const LINE_2: u32 = 0x5d5d5d;
const INK: u32 = 0xd6d6d6;
const SUB: u32 = 0xb5b5b5;
const MUTED: u32 = 0x757575;
const ACTIVE: u32 = 0xffad56;
const WARN: u32 = 0xe0705f;
const CLIPS: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

const ROW_NAMES: [&str; 6] = [
    "sky_plate.mp4",
    "Pulse rings",
    "NIGHT DRIVE",
    "City grid",
    "reflection.mp4",
    "traffic_pass.mp4",
];

// Spread量。奥へだけ広げ、最前列は平面に残す。層順: 先頭行=最背面が最も奥
const SPAN: f32 = 0.36;

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Flat,
    Gesture,
    After,
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

fn row_z(row: usize) -> f32 {
    // 先頭行=最背面が最も奥へ。最前列は平面に残る
    -SPAN * (5 - row) as f32 / 5.0
}

fn draw_strip(cv: &skia_safe::Canvas, top: f32, state: State) {
    fill(cv, Rect::from_xywh(0.0, top, W, STRIP_H), rgb(BG));
    fill(
        cv,
        Rect::from_xywh(0.0, top, LABEL_W, STRIP_H),
        rgb(SURFACE_HI),
    );
    line(cv, (LABEL_W, top), (LABEL_W, top + STRIP_H), rgb(LINE));
    stroke(cv, Rect::from_xywh(0.0, top, W, STRIP_H), rgb(LINE));

    // ruler。Depth iconは時間面headerの明示入口(既決)。それ以外に深度の面は無い
    fill(cv, Rect::from_xywh(0.0, top, W, RULER_H), rgb(SURFACE));
    text(cv, "TIME / BEAT", LABEL_W + 10.0, top + 13.0, 7.0, rgb(MUTED));
    for i in 0..9 {
        let x = LABEL_W + i as f32 * (W - LABEL_W) / 8.0;
        line(cv, (x, top), (x, top + STRIP_H), rgb(0x2e2e2e));
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
    // Depth icon(入口)。gesture中はarmed
    let armed = state == State::Gesture;
    let di = Rect::from_xywh(W - 24.0, top + 2.0, 18.0, 14.0);
    fill(cv, di, rgb(if armed { 0x3a3125 } else { SURFACE }));
    stroke(cv, di, rgb(if armed { ACTIVE } else { LINE_2 }));
    diamond(
        cv,
        W - 15.0,
        top + 9.0,
        4.0,
        rgb(if armed { ACTIVE } else { SUB }),
    );

    let spans = [
        (0.03f32, 0.55f32),
        (0.10, 0.60),
        (0.17, 0.65),
        (0.25, 0.73),
        (0.32, 0.79),
        (0.40, 0.85),
    ];
    for row in 0..6 {
        let y = top + RULER_H + row as f32 * ROW_H;
        line(cv, (0.0, y), (W, y), rgb(0x2c2c2c));
        fill(
            cv,
            Rect::from_xywh(10.0, y + 5.0, 3.0, ROW_H - 10.0),
            rgb(CLIPS[row]),
        );
        text(cv, ROW_NAMES[row], 20.0, y + 14.0, 8.0, rgb(INK));

        // 逸脱chip: 深度が存在する時だけ生まれる。平時は沈黙
        match state {
            State::Flat => {}
            State::Gesture => {
                let z = row_z(row);
                let label = if z == 0.0 {
                    "z 0".to_string()
                } else {
                    format!("z {z:+.2}")
                };
                text_r(cv, &label, LABEL_W - 8.0, y + 14.0, 7.0, rgb(ACTIVE));
            }
            State::After => {
                let z = row_z(row);
                if z != 0.0 {
                    text_r(
                        cv,
                        &format!("z {z:+.2}"),
                        LABEL_W - 8.0,
                        y + 14.0,
                        7.0,
                        rgb(SUB),
                    );
                }
                if row == 4 {
                    // 遮蔽がZ順と食い違う行にだけ警告が来る。パネルを開いて探さない
                    let ox = text_r(cv, "camera", LABEL_W - 46.0, y + 14.0, 6.5, rgb(WARN));
                    tri(
                        cv,
                        [
                            (ox - 5.0, y + 7.5),
                            (ox - 9.0, y + 14.0),
                            (ox - 1.0, y + 14.0),
                        ],
                        rgb(WARN),
                    );
                }
            }
        }

        let start = LABEL_W + spans[row].0 * (W - LABEL_W);
        let end = LABEL_W + spans[row].1 * (W - LABEL_W);
        let r = Rect::from_ltrb(start, y + 3.0, end, y + ROW_H - 3.0);
        fill(cv, r, rgb(CLIPS[row]));
        if state != State::Flat {
            stroke(
                cv,
                Rect::from_ltrb(start - 1.0, y + 2.0, end + 1.0, y + ROW_H - 2.0),
                rgb(INK),
            ); // 選択は白outline(規範どおり)
        }
        text(cv, ROW_NAMES[row], start + 8.0, y + 14.0, 7.5, rgb(0x191919));
    }

    // gesture中だけ、選択の上に量が1つ浮かぶ
    if state == State::Gesture {
        let gx = LABEL_W + 240.0;
        let gy = top + RULER_H + 6.0;
        let gw = 320.0;
        let g = Rect::from_xywh(gx, gy, gw, 22.0);
        fill(cv, g, rgb(SURFACE));
        stroke(cv, g, rgb(ACTIVE));
        diamond(cv, gx + 12.0, gy + 11.0, 4.5, rgb(ACTIVE));
        text(cv, "SPREAD", gx + 22.0, gy + 15.0, 8.0, rgb(INK));
        // 量。1本のdrag。0が左端=平ら
        let track_l = gx + 75.0;
        let track_r = gx + gw - 88.0;
        line(cv, (track_l, gy + 11.0), (track_r, gy + 11.0), rgb(LINE_2));
        let t = 0.62; // 現在量
        let hx = track_l + (track_r - track_l) * t;
        fill(
            cv,
            Rect::from_ltrb(track_l, gy + 10.0, hx, gy + 12.0),
            rgb(ACTIVE),
        );
        fill(cv, Rect::from_xywh(hx - 2.5, gy + 4.0, 5.0, 14.0), rgb(ACTIVE));
        text(cv, &format!("{SPAN:.2}"), track_r + 8.0, gy + 15.0, 7.5, rgb(ACTIVE));
        // keep look 既定ON
        let kb = Rect::from_xywh(gx + gw - 46.0, gy + 6.0, 9.0, 9.0);
        fill(cv, kb, rgb(ACTIVE));
        line(
            cv,
            (gx + gw - 44.0, gy + 10.5),
            (gx + gw - 42.0, gy + 13.0),
            rgb(0x1a1a1a),
        );
        line(
            cv,
            (gx + gw - 42.0, gy + 13.0),
            (gx + gw - 38.5, gy + 7.5),
            rgb(0x1a1a1a),
        );
        text(cv, "keep look", gx + gw - 34.0, gy + 14.0, 6.5, rgb(SUB));
    }

    let playhead = LABEL_W + 0.46 * (W - LABEL_W);
    line(cv, (playhead, top + RULER_H), (playhead, top + STRIP_H), rgb(INK));
}

fn main() {
    let prefix = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-depth-rail-v13".into());
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    fill(cv, Rect::from_xywh(0.0, 0.0, W, H), rgb(BG));

    let mut y = 10.0;
    for (caption, state) in [
        ("1 - flat project: no depth UI exists. silence is the default", State::Flat),
        (
            "2 - Spread: one amount. layer order decides who goes deeper, keep look holds the picture",
            State::Gesture,
        ),
        (
            "3 - after release (1 undo): small deviation chips remain. depth exists, so now it shows",
            State::After,
        ),
    ] {
        text(cv, caption, 10.0, y + 10.0, 8.0, rgb(MUTED));
        draw_strip(cv, y + CAP_H, state);
        y += CAP_H + STRIP_H + GAP;
    }

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("png");
    let out = format!("{prefix}.png");
    std::fs::write(&out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
