// group layer — 行は自分自身を語り、rail は集合を語る。
// 畳みで変わるのは「子の区間がその行へ投影されるか」だけ。
// グループ自身のエフェクトは畳んでも展開しても同じ位置（行の右クラスタ）に出る。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1180.0;
const RAIL_W: f32 = 96.0;
const RULER_H: f32 = 20.0;
const ROW: f32 = 20.0;
const BARS: f32 = 64.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const SURFACE_HI: u32 = 0x464646;
const CONTRAST: u32 = 0x111111;
const DIM: u32 = 0x757575;
const RULER_MARK: u32 = 0x919191;
const FILL_HANDLE: u32 = 0x5d5d5d;
const ON_BAR: u32 = 0x141414;
const ACCENT: u32 = 0xffad56;
const P: [u32; 6] = [0x96aadb, 0x6fb9c1, 0xbfa973, 0x89b992, 0xd69a8b, 0xc39bc5];

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
fn tf() -> skia_safe::Typeface {
    FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .unwrap()
}
fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, sz: f32, c: Color) {
    let f = Font::new(tf(), sz);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &f, &p);
}
fn measure(s: &str, sz: f32) -> f32 {
    Font::new(tf(), sz).measure_str(s, None).0
}

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, d: f32) {
    let f = gray(if d.abs() < 0.01 {
        0x2a
    } else if d.abs() < 0.2 {
        0x8c
    } else {
        0xf2
    });
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

fn bypass_glyph(cv: &skia_safe::Canvas, cx: f32, cy: f32, on_dark: bool) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(argb(if on_dark { 0x40 } else { 0x2e }, 0x000000));
    cv.draw_circle((cx, cy), 5.5, &p);
    p.set_color(if on_dark {
        gray(0xc8)
    } else {
        argb(0xd8, ON_BAR)
    });
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.2);
    cv.draw_circle((cx, cy), 3.2, &p);
    cv.draw_line((cx - 2.6, cy + 2.6), (cx + 2.6, cy - 2.6), &p);
}

fn tog(cv: &skia_safe::Canvas, x: f32, cy: f32, l: &str, on: bool, mixed: bool, acc: u32) {
    let r = Rect::from_xywh(x, cy - 6.5, 14.0, 13.0);
    fill(cv, r, if on { rgb(acc) } else { rgb(FILL_HANDLE) });
    if mixed {
        fill(
            cv,
            Rect::from_ltrb(r.left, r.top, r.left + 7.0, r.bottom),
            rgb(acc),
        );
    }
    fill(
        cv,
        Rect::from_ltrb(r.left, r.top, r.right, r.top + 1.0),
        gray(0x6e),
    );
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(rgb(0x3a3a3a));
    cv.draw_rect(r, &p);
    let w = measure(l, 8.5);
    text(
        cv,
        l,
        r.left + (14.0 - w) / 2.0,
        cy + 3.2,
        8.5,
        if on || mixed {
            rgb(0x0d0d0d)
        } else {
            gray(0xc4)
        },
    );
}

#[derive(Clone, Copy)]
struct Piece {
    a: f32,
    b: f32,
}

struct Node {
    name: &'static str,
    depth: usize,
    group: bool,
    folded: bool,
    slot: usize,
    own_fx: u8, // その行のオブジェクト自身に掛かっているエフェクト
    own_keys: &'static [(f32, f32)],
    pieces: &'static [Piece], // leaf のclip / group なら子孫の全piece
    inner_bypass: bool,       // 子孫に無効化されたものがある
    count: usize,             // 畳んだ時に隠れる個数（再帰）
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-group.png".into());

    // 同じ構造を3状態で見る
    let sets: [(&str, &[Node]); 3] = [
        (
            "unfolded",
            &[
                Node {
                    name: "BG",
                    depth: 0,
                    group: true,
                    folded: false,
                    slot: 3,
                    own_fx: 2,
                    own_keys: &[(6.0, 0.45)],
                    pieces: &[Piece { a: 0.0, b: 58.0 }],
                    inner_bypass: true,
                    count: 5,
                },
                Node {
                    name: "plate_a",
                    depth: 1,
                    group: false,
                    folded: false,
                    slot: 0,
                    own_fx: 0,
                    own_keys: &[],
                    pieces: &[Piece { a: 0.0, b: 18.0 }, Piece { a: 26.0, b: 40.0 }],
                    inner_bypass: false,
                    count: 0,
                },
                Node {
                    name: "plate_b",
                    depth: 1,
                    group: false,
                    folded: false,
                    slot: 1,
                    own_fx: 1,
                    own_keys: &[],
                    pieces: &[Piece { a: 12.0, b: 30.0 }],
                    inner_bypass: false,
                    count: 0,
                },
                Node {
                    name: "fx_stack",
                    depth: 1,
                    group: true,
                    folded: true,
                    slot: 5,
                    own_fx: 1,
                    own_keys: &[(44.0, 0.12)],
                    pieces: &[Piece { a: 36.0, b: 46.0 }, Piece { a: 46.0, b: 58.0 }],
                    inner_bypass: true,
                    count: 2,
                },
            ],
        ),
        (
            "BG folded",
            &[Node {
                name: "BG",
                depth: 0,
                group: true,
                folded: true,
                slot: 3,
                own_fx: 2,
                own_keys: &[(6.0, 0.45)],
                pieces: &[
                    Piece { a: 0.0, b: 18.0 },
                    Piece { a: 12.0, b: 30.0 },
                    Piece { a: 26.0, b: 40.0 },
                    Piece { a: 36.0, b: 46.0 },
                    Piece { a: 46.0, b: 58.0 },
                ],
                inner_bypass: true,
                count: 5,
            }],
        ),
        (
            "group with nothing of its own",
            &[Node {
                name: "SET_B",
                depth: 0,
                group: true,
                folded: true,
                slot: 2,
                own_fx: 0,
                own_keys: &[],
                pieces: &[Piece { a: 4.0, b: 20.0 }, Piece { a: 30.0, b: 52.0 }],
                inner_bypass: false,
                count: 3,
            }],
        ),
    ];

    let rows: usize = sets.iter().map(|s| s.1.len()).sum();
    let h = RULER_H + 1.0 + rows as f32 * ROW + sets.len() as f32 * 16.0 + 26.0;
    let mut sf = surfaces::raster_n32_premul(((W * SCALE) as i32, (h * SCALE) as i32)).unwrap();
    let cv = sf.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));
    let bx = |b: f32| RAIL_W + b / BARS * (W - RAIL_W - 6.0);

    fill(cv, Rect::from_ltrb(0.0, 0.0, RAIL_W, h), rgb(SURFACE_BG));
    fill(cv, Rect::from_ltrb(0.0, 0.0, W, RULER_H), rgb(SURFACE_HI));
    for b in (0..=BARS as i32).step_by(4) {
        let x = bx(b as f32);
        fill(
            cv,
            Rect::from_ltrb(x, RULER_H - 7.0, x + 1.0, RULER_H),
            gray(0x6a),
        );
        if b < BARS as i32 {
            text(
                cv,
                &format!("{}", b + 1),
                x + 3.0,
                12.0,
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

    let mut y = RULER_H + 1.0;
    for (label, nodes) in sets {
        text(cv, label, 6.0, y + 11.0, 9.0, rgb(DIM));
        y += 16.0;
        for n in nodes.iter() {
            for b in 0..=BARS as i32 {
                let x = bx(b as f32);
                fill(
                    cv,
                    Rect::from_ltrb(x, y, x + 1.0, y + ROW - 1.0),
                    argb(if b % 4 == 0 { 0x54 } else { 0x14 }, 0x060606),
                );
            }
            fill(
                cv,
                Rect::from_ltrb(0.0, y + ROW - 1.0, W, y + ROW),
                rgb(CONTRAST),
            );
            let cy = y + (ROW - 1.0) / 2.0;
            let ind = n.depth as f32 * 10.0;

            // ── rail: 集合を語る ──
            tog(cv, 4.0 + ind, cy, "M", false, false, 0xd8d8d8);
            tog(cv, 20.0 + ind, cy, "S", false, false, ACCENT);
            if n.group {
                let tri = if n.folded { ">" } else { "v" };
                text(cv, tri, 38.0 + ind, cy + 3.2, 9.0, gray(0xc0));
                if n.folded {
                    text(
                        cv,
                        &format!("{}", n.count),
                        47.0 + ind,
                        cy + 3.2,
                        8.5,
                        gray(0x8e),
                    );
                }
            }
            if n.inner_bypass && n.folded {
                bypass_glyph(cv, RAIL_W - 12.0, cy, true);
            }

            let col = P[n.slot];
            if n.group && !n.folded {
                // 展開中の親: clipを持たないので面を敷かない。範囲だけを細い帯で示す。
                let (a, b) = (n.pieces[0].a, n.pieces[n.pieces.len() - 1].b);
                fill(
                    cv,
                    Rect::from_ltrb(bx(a), cy - 1.0, bx(b), cy + 1.0),
                    argb(0x66, col),
                );
                fill(
                    cv,
                    Rect::from_ltrb(bx(a), cy - 4.0, bx(a) + 1.0, cy + 4.0),
                    argb(0x99, col),
                );
                fill(
                    cv,
                    Rect::from_ltrb(bx(b) - 1.0, cy - 4.0, bx(b), cy + 4.0),
                    argb(0x99, col),
                );
            } else {
                // 区間を結合。内部の切れ目は1pxで残す（「どの子か」は捨てるが「何個で出来ているか」は残す）
                let mut ps: Vec<Piece> = n.pieces.to_vec();
                ps.sort_by(|x, z| x.a.partial_cmp(&z.a).unwrap());
                let mut merged: Vec<(f32, f32, Vec<f32>)> = vec![];
                for p in ps {
                    match merged.last_mut() {
                        Some(l) if p.a <= l.1 => {
                            l.2.push(p.a);
                            l.1 = l.1.max(p.b);
                        }
                        _ => merged.push((p.a, p.b, vec![])),
                    }
                }
                for (a, b, cuts) in &merged {
                    let r = Rect::from_ltrb(bx(*a), y, bx(*b), y + ROW - 1.0);
                    fill(cv, r, rgb(col));
                    for c in cuts {
                        fill(
                            cv,
                            Rect::from_ltrb(bx(*c), r.top, bx(*c) + 1.0, r.bottom),
                            argb(0x50, 0x000000),
                        );
                    }
                }
            }

            // ── 自分自身のこと。面がある時は右端、面が無い時は名前の直後 ──
            let (ba, bb) = (n.pieces[0].a, n.pieces[n.pieces.len() - 1].b);
            let has_face = !(n.group && !n.folded);
            let nw = measure(n.name, 8.5);
            let mut nx = bx(ba) + 5.0;
            for (b, _) in n.own_keys {
                let kx = bx(*b);
                if kx > nx - 8.0 && kx < nx + nw + 6.0 {
                    nx = kx + 9.0;
                }
            }
            let ink = if has_face {
                argb(0xff, ON_BAR)
            } else {
                gray(0xcc)
            };
            text(cv, n.name, nx, cy + 3.2, 8.5, ink);

            if n.own_fx > 0 {
                let lbl = format!("fx {}", n.own_fx);
                let wd = measure(&lbl, 8.5) + 8.0;
                if has_face {
                    let x = bx(bb) - 4.0 - wd;
                    text(cv, &lbl, x + 4.0, cy + 3.2, 8.5, argb(0xc4, ON_BAR));
                } else {
                    // 面が無いので右端に貼れない。名前に続けて置く。
                    let x = nx + nw + 6.0;
                    let mut p = Paint::default();
                    p.set_anti_alias(true);
                    p.set_color(argb(0x5a, 0x000000));
                    cv.draw_rect(Rect::from_xywh(x, cy - 6.5, wd, 13.0), &p);
                    text(cv, &lbl, x + 4.0, cy + 3.2, 8.5, gray(0xd4));
                }
            }
            for (b, d) in n.own_keys {
                diamond(cv, bx(*b), cy, *d);
            }
            y += ROW;
        }
    }

    text(
        cv,
        "the row always speaks for its own object. the rail always speaks for the set.",
        6.0,
        h - 14.0,
        9.0,
        rgb(DIM),
    );
    text(
        cv,
        "folding only adds the descendants' spans to the row. group's own fx / keys never move.",
        6.0,
        h - 3.0,
        9.0,
        gray(0x5f),
    );

    let png = sf
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .unwrap();
    std::fs::write(&out, png.as_bytes()).unwrap();
    println!("{out}  {}x{}", (W * SCALE) as i32, (h * SCALE) as i32);
}
