// keyframe marker の情報量 — 同一fixture比較
// 形は全て同一（AM準拠）。色だけがヒントを持つ。
// 符号化するのは「出ていく差」= このkeyから次のkeyまでの移動量。
// 基準は絶対規律5の正準座標（原点中央・高さ1.0）。delta 0.5 = 画面高さの半分。
// seed等の識別だけの値は差が定義できないのでニュートラル1色。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 940.0;
const LEFT: f32 = 118.0;
const RULER_H: f32 = 20.0;
const ROW_H: f32 = 46.0;
const GAP: f32 = 10.0;
const ROWS: usize = 4;
const H: f32 = RULER_H + ROWS as f32 * (ROW_H + GAP) + 34.0;
const BARS: f32 = 48.0;

// Default Dark Neutral Medium 実値
const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const CONTRAST: u32 = 0x111111;
const FG: u32 = 0xb5b5b5;
const RULER_MARK: u32 = 0x919191;
const DISABLED: u32 = 0x757575;
const OBJ: u32 = 0x6baace; // Object identity色（この面の意味は「どのObjectか」）

fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}
fn argb(a: u8, v: u32) -> Color {
    Color::from_argb(
        a,
        (v >> 16) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}
fn gray(v: u8) -> Color {
    Color::from_rgb(v, v, v)
}
fn fill(cv: &skia_safe::Canvas, r: Rect, c: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_color(c);
    cv.draw_rect(r, &p);
}
fn vline(cv: &skia_safe::Canvas, x: f32, y0: f32, y1: f32, c: Color) {
    fill(cv, Rect::from_ltrb(x, y0, x + 1.0, y1), c);
}
fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, size: f32, c: Color) {
    let tf = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("typeface");
    let font = Font::new(tf, size);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &font, &p);
}

// 4〜5pxの内側で読めるのは2〜3段階。連続ramp にしない。
#[derive(Clone, Copy, PartialEq)]
enum Level {
    Hold,  // delta ≈ 0
    Small, // 画面高さの数%
    Medium,
    Large,
    Undefined, // seed 等、差が定義できない
    Terminal,  // 最後のkey。出ていく差が無い
}

fn level(delta: Option<f32>, terminal: bool) -> Level {
    if terminal {
        return Level::Terminal;
    }
    match delta {
        None => Level::Undefined,
        Some(d) => {
            let d = d.abs();
            if d < 0.01 {
                Level::Hold
            } else if d < 0.12 {
                Level::Small
            } else if d < 0.35 {
                Level::Medium
            } else {
                Level::Large
            }
        }
    }
}

// 候補A: 暗いほど大きい / 候補B: 明るいほど大きい
fn level_fill(l: Level, bright_is_large: bool) -> Option<Color> {
    let v = match l {
        Level::Hold => 0,
        Level::Small => 1,
        Level::Medium => 2,
        Level::Large => 3,
        Level::Undefined => return Some(gray(0x8a)),
        Level::Terminal => return None, // 中抜き
    };
    let ramp_dark = [0xe4u8, 0xa8, 0x64, 0x1c]; // 大きいほど暗い
    let ramp_light = [0x24u8, 0x60, 0xa4, 0xf2]; // 大きいほど明るい
    Some(gray(if bright_is_large {
        ramp_light[v]
    } else {
        ramp_dark[v]
    }))
}

// 規範: 10px / 2px dark stroke / 1px light outer ring。形は常に同一。
fn diamond_sz(cv: &skia_safe::Canvas, cx: f32, cy: f32, f: Option<Color>, k: f32) {
    let path = |s: f32| {
        let s = s * k;
        let mut b = PathBuilder::new();
        b.move_to((cx, cy - s));
        b.line_to((cx + s, cy));
        b.line_to((cx, cy + s));
        b.line_to((cx - s, cy));
        b.close();
        b.detach()
    };
    let mut p = Paint::default();
    p.set_anti_alias(true);

    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(argb(0x8c, 0xffffff));
    cv.draw_path(&path(6.0), &p);

    if let Some(c) = f {
        p.set_style(PaintStyle::Fill);
        p.set_color(c);
        cv.draw_path(&path(4.0), &p);
    }
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(2.0);
    p.set_color(rgb(0x1e1e1e));
    cv.draw_path(&path(5.0), &p);
}

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, f: Option<Color>) {
    // 左半分は規範の10px、右半分は13pxで大きさの効きを見る
    let k = if cx > W * 0.52 { 1.3 } else { 1.0 };
    diamond_sz(cv, cx, cy, f, k);
}

struct Key {
    bar: f32,
    value: Option<f32>, // None = 識別のみ(seed)
    label: &'static str,
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-kf.png".into());
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));

    let bx = |b: f32| LEFT + b / BARS * (W - LEFT - 12.0);

    // Position Y（正準座標・高さ1.0基準）
    let keys: Vec<Key> = vec![
        Key {
            bar: 1.0,
            value: Some(0.00),
            label: "0.00",
        },
        Key {
            bar: 7.0,
            value: Some(0.02),
            label: "0.02",
        },
        Key {
            bar: 13.0,
            value: Some(0.02),
            label: "0.02",
        },
        Key {
            bar: 19.0,
            value: Some(0.35),
            label: "0.35",
        },
        Key {
            bar: 25.0,
            value: Some(0.28),
            label: "0.28",
        },
        Key {
            bar: 33.0,
            value: Some(-0.45),
            label: "-0.45",
        },
        Key {
            bar: 41.0,
            value: Some(-0.40),
            label: "-0.40",
        },
        Key {
            bar: 46.0,
            value: Some(-0.40),
            label: "-0.40",
        },
    ];
    let seed_keys: Vec<Key> = vec![
        Key {
            bar: 1.0,
            value: None,
            label: "3",
        },
        Key {
            bar: 13.0,
            value: None,
            label: "9041",
        },
        Key {
            bar: 25.0,
            value: None,
            label: "77",
        },
        Key {
            bar: 41.0,
            value: None,
            label: "9042",
        },
    ];

    // ── ruler ────────────────────────────────────────────
    fill(cv, Rect::from_ltrb(0.0, 0.0, W, RULER_H), rgb(SURFACE_BG));
    for b in 0..=(BARS as i32) {
        if b % 4 != 0 {
            continue;
        }
        let x = bx(b as f32);
        vline(cv, x, RULER_H - 7.0, RULER_H, gray(0x6a));
        if b < BARS as i32 {
            text(
                cv,
                &format!("{}", b + 1),
                x + 3.0,
                11.0,
                8.5,
                rgb(RULER_MARK),
            );
        }
    }
    fill(
        cv,
        Rect::from_ltrb(0.0, RULER_H, W, RULER_H + 1.0),
        rgb(CONTRAST),
    );

    let rows: [(&str, &str); ROWS] = [
        ("now", "all identical - position only"),
        ("A", "darker = larger move"),
        ("B", "brighter = larger move"),
        ("seed", "no defined distance - uniform"),
    ];

    for (ri, (name, note)) in rows.iter().enumerate() {
        let y = RULER_H + 1.0 + ri as f32 * (ROW_H + GAP) + GAP;
        text(cv, name, 10.0, y + 16.0, 11.0, rgb(FG));
        text(cv, note, 10.0, y + 30.0, 9.0, rgb(DISABLED));

        // Object bar（面の色は identity。keyframeの色とは別体系）
        let bar = Rect::from_ltrb(bx(0.0), y, W - 12.0, y + ROW_H);
        fill(cv, bar, rgb(OBJ));
        fill(
            cv,
            Rect::from_ltrb(bar.left, bar.top, bar.right, bar.top + 12.0),
            argb(0x2e, 0x000000),
        );
        text(
            cv,
            "skyline_a",
            bar.left + 5.0,
            bar.top + 9.0,
            9.0,
            rgb(0x0a0a0a),
        );

        let ks = if ri == 3 { &seed_keys } else { &keys };
        let cy = y + 12.0 + (ROW_H - 12.0) / 2.0;

        for (i, k) in ks.iter().enumerate() {
            let terminal = i + 1 == ks.len();
            let delta = if terminal {
                None
            } else {
                match (k.value, ks[i + 1].value) {
                    (Some(a), Some(b)) => Some(b - a),
                    _ => None,
                }
            };
            let lv = level(delta, terminal);
            let f = match ri {
                0 => Some(gray(0x1e)), // 現状: 一律の暗色塗り
                1 => level_fill(lv, false),
                _ => level_fill(lv, true),
            };
            diamond(cv, bx(k.bar), cy, f);

            // 値と差を検算できるように併記（fixture用。製品UIではない）
            text(
                cv,
                k.label,
                bx(k.bar) - 9.0,
                bar.bottom - 4.0,
                8.0,
                rgb(0x0d0d0d),
            );
            if let Some(d) = delta {
                let mid = (bx(k.bar) + bx(ks[i + 1].bar)) / 2.0;
                text(
                    cv,
                    &format!("Δ{:.2}", d.abs()),
                    mid - 11.0,
                    bar.top + 22.0,
                    8.0,
                    argb(0xb0, 0x000000),
                );
            }
        }
    }

    let ly = H - 18.0;
    text(
        cv,
        "outgoing delta = move from this key to the next. unit = canonical height 1.0",
        10.0,
        ly,
        9.5,
        rgb(DISABLED),
    );
    text(
        cv,
        "hold <0.01  /  small <0.12  /  medium <0.35  /  large >=0.35  /  last key = hollow",
        10.0,
        ly + 12.0,
        9.0,
        rgb(0x5f5f5f),
    );

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("encode");
    std::fs::write(&out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
