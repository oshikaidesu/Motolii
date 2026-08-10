// group fold — 畳む＝縦方向への射影。同じ規則を clip と key に2回当てる。
//   子clip N本 -> 「中身がある区間」+ 件数
//   子key   N個 -> 「キーがある位置」+ 件数、明度は含まれる中の最大delta
// 行高は固定・最小（keyframe marker基準）。DAWと違い縦が情報を持たないため。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 940.0;
const LEFT: f32 = 104.0;
const RULER_H: f32 = 18.0;
const ROW: f32 = 20.0; // 最小固定。marker 10px + 余白
const BARS: f32 = 48.0;
const H: f32 = RULER_H + 1.0 + 9.0 * ROW + 62.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const CONTRAST: u32 = 0x111111;
const FG: u32 = 0xb5b5b5;
const DIM: u32 = 0x757575;
const RULER_MARK: u32 = 0x919191;

// OKLCH L=0.74 / C=0.075 で知覚明度を揃えた6slot。
// 共通暗色 ON_BAR に対するコントラストは 7.73:1〜8.26:1（振れ幅0.53）。
const C_PLATE_A: u32 = 0x96aadb; // periwinkle
const C_PLATE_B: u32 = 0x6fb9c1; // teal
const C_GRAIN: u32 = 0xbfa973; // sand
const C_GROUP: u32 = 0x89b992; // sage
const ON_BAR: u32 = 0x141414;

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
fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, size: f32, c: Color) {
    let tf = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("tf");
    let f = Font::new(tf, size);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &f, &p);
}

// 明度 = delta の3段階（B案: 明るいほど大きい）
fn key_fill(delta: f32) -> Color {
    let d = delta.abs();
    gray(if d < 0.01 {
        0x2a
    } else if d < 0.2 {
        0x8c
    } else {
        0xf2
    })
}

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, f: Color) {
    let path = |s: f32| {
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
    p.set_color(argb(0x4a, 0xffffff));
    cv.draw_path(&path(5.6), &p);
    p.set_style(PaintStyle::Fill);
    p.set_color(f);
    cv.draw_path(&path(4.2), &p);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.6);
    p.set_color(rgb(0x16181c));
    cv.draw_path(&path(4.9), &p);
}

struct Child {
    name: &'static str,
    color: u32,
    clips: &'static [(f32, f32)],
    keys: &'static [(f32, f32)], // (bar, outgoing delta)
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-fold.png".into());
    let mut sf = surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).unwrap();
    let cv = sf.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));
    let bx = |b: f32| LEFT + b / BARS * (W - LEFT - 10.0);

    let children = [
        Child {
            name: "plate_a",
            color: C_PLATE_A,
            clips: &[(1.0, 13.0), (21.0, 31.0)],
            keys: &[(7.0, 0.35), (13.0, 0.02)],
        },
        Child {
            name: "plate_b",
            color: C_PLATE_B,
            clips: &[(5.0, 17.0), (35.0, 45.0)],
            keys: &[(9.0, 0.70), (25.5, 0.05), (26.5, 0.04)],
        },
        Child {
            name: "grain",
            color: C_GRAIN,
            clips: &[(9.0, 15.0), (27.0, 33.0)],
            keys: &[(33.0, 0.10)],
        },
    ];

    // ── ruler ──
    fill(cv, Rect::from_ltrb(0.0, 0.0, W, RULER_H), rgb(SURFACE_BG));
    for b in (0..=BARS as i32).step_by(4) {
        let x = bx(b as f32);
        fill(
            cv,
            Rect::from_ltrb(x, RULER_H - 6.0, x + 1.0, RULER_H),
            gray(0x6a),
        );
        if b < BARS as i32 {
            text(
                cv,
                &format!("{}", b + 1),
                x + 3.0,
                10.0,
                8.0,
                rgb(RULER_MARK),
            );
        }
    }
    fill(
        cv,
        Rect::from_ltrb(0.0, RULER_H, W, RULER_H + 1.0),
        rgb(CONTRAST),
    );

    let mut y = RULER_H + 1.0;
    let grid = |cv: &skia_safe::Canvas, y: f32| {
        for b in 0..=BARS as i32 {
            let x = bx(b as f32);
            let major = b % 4 == 0;
            fill(
                cv,
                Rect::from_ltrb(x, y, x + 1.0, y + ROW - 1.0),
                argb(if major { 0x54 } else { 0x1c }, 0x060606),
            );
        }
        fill(
            cv,
            Rect::from_ltrb(0.0, y + ROW - 1.0, W, y + ROW),
            rgb(CONTRAST),
        );
    };
    let row_bar = |cv: &skia_safe::Canvas, y: f32, a: f32, b: f32, col: u32, name: &str| {
        let r = Rect::from_ltrb(bx(a), y, bx(b), y + ROW - 1.0);
        fill(cv, r, rgb(col));
        if r.width() > 34.0 {
            text(cv, name, r.left + 5.0, r.top + ROW * 0.66, 8.5, rgb(ON_BAR));
        }
    };

    // ── UNFOLDED ──
    grid(cv, y);
    // グループ自体はclipを持たない(Ableton 18.3と同じ)。行は空。
    text(cv, "v", 8.0, y + 13.5, 9.0, rgb(DIM));
    text(cv, "BG", 18.0, y + 13.5, 9.5, rgb(0xd8d8d8));
    fill(
        cv,
        Rect::from_ltrb(bx(0.0), y + ROW - 4.0, bx(48.0), y + ROW - 2.0),
        argb(0x5a, C_GROUP),
    );
    y += ROW;
    for c in &children {
        grid(cv, y);
        text(cv, c.name, 28.0, y + 13.5, 9.0, rgb(0x9a9a9a));
        for (a, b) in c.clips {
            row_bar(cv, y, *a, *b, c.color, c.name);
        }
        for (bar, d) in c.keys {
            diamond(cv, bx(*bar), y + ROW / 2.0, key_fill(*d));
        }
        y += ROW;
    }

    y += 26.0;

    // ── FOLDED: 子clipの和集合を1段へ / keyは近接を束ねる ──
    grid(cv, y);
    let mut spans: Vec<(f32, f32)> = children
        .iter()
        .flat_map(|c| c.clips.iter().copied())
        .collect();
    spans.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut merged: Vec<(f32, f32, usize)> = vec![];
    for (a, b) in spans {
        match merged.last_mut() {
            Some(l) if a <= l.1 => {
                l.1 = l.1.max(b);
                l.2 += 1;
            }
            _ => merged.push((a, b, 1)),
        }
    }
    let total: usize = children.iter().map(|c| c.clips.len()).sum();
    text(cv, ">", 8.0, y + 13.5, 9.0, rgb(DIM));
    text(cv, "BG", 18.0, y + 13.5, 9.5, rgb(0xd8d8d8));
    text(cv, &format!("{}", total), 40.0, y + 13.5, 9.0, rgb(DIM));
    // 空きは地のまま。中身がある区間だけを塗る。
    for (a, b, n) in &merged {
        let r = Rect::from_ltrb(bx(*a), y, bx(*b), y + ROW - 1.0);
        fill(cv, r, rgb(C_GROUP));
        // 件数は区間の左端。キーは中央に来るので当たらない。
        text(
            cv,
            &format!("{}", n),
            r.left + 5.0,
            r.top + ROW * 0.66,
            8.0,
            argb(0xb4, ON_BAR),
        );
    }

    // key: 近接を1マーカーへ束ね、明度は最大delta、件数を添える
    let mut ks: Vec<(f32, f32)> = children
        .iter()
        .flat_map(|c| c.keys.iter().copied())
        .collect();
    ks.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let mut kg: Vec<(f32, f32, usize)> = vec![];
    for (bar, d) in ks {
        match kg.last_mut() {
            Some(l) if bx(bar) - bx(l.0) < 13.0 => {
                l.1 = l.1.max(d);
                l.2 += 1;
            }
            _ => kg.push((bar, d, 1)),
        }
    }
    for (bar, d, n) in &kg {
        diamond(cv, bx(*bar), y + ROW / 2.0, key_fill(*d));
        if *n > 1 {
            text(
                cv,
                &format!("{}", n),
                bx(*bar) + 7.0,
                y + ROW * 0.42,
                8.0,
                gray(0xe0),
            );
        }
    }

    let ly = H - 40.0;
    text(
        cv,
        "empty stays background. only spans with content are filled. count sits at the span head.",
        8.0,
        ly,
        9.0,
        rgb(DIM),
    );
    text(
        cv,
        "merged key brightness = MAX outgoing delta inside (not sum). count already carries density.",
        8.0,
        ly + 12.0,
        9.0,
        gray(0x5f),
    );
    text(
        cv,
        "row height fixed at 20px. vertical axis carries no information in a compositor timeline.",
        8.0,
        ly + 24.0,
        9.0,
        gray(0x5f),
    );

    let png = sf
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .unwrap();
    std::fs::write(&out, png.as_bytes()).unwrap();
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
