// object bar の情報密度 — 「逸脱時のみ表示、既定は沈黙」
// バーは読み取り専用。状態変更はInspectorとkeymapが持つ（誤爆のコストが非対称なため）。
// 右クラスタ: [逸脱グリフ] [fx N] [M] [S]。既定値のものは何も出さない。
// 幅が足りなければ右から順に落とす（progressive disclosure）。
// 行高は 20px 固定のまま。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1000.0;
const LEFT: f32 = 96.0;
const RULER_H: f32 = 18.0;
const ROW: f32 = 20.0;
const BARS: f32 = 48.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const CONTRAST: u32 = 0x111111;
const DIM: u32 = 0x757575;
const RULER_MARK: u32 = 0x919191;
const ON_BAR: u32 = 0x141414;

// OKLCH L=0.74 C=0.075
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
fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, size: f32, c: Color) -> f32 {
    let f = Font::new(tf(), size);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &f, &p);
    f.measure_str(s, Some(&p)).0
}
fn measure(s: &str, size: f32) -> f32 {
    Font::new(tf(), size).measure_str(s, None).0
}

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, delta: f32) {
    let d = delta.abs();
    let f = gray(if d < 0.01 {
        0x2a
    } else if d < 0.2 {
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

// 面を地へ寄せる。mute は widget ではなく面そのもので示す。
fn dim(c: u32, t: f32) -> u32 {
    let m = |sh: u32| {
        let a = ((c >> sh) & 0xff) as f32;
        let b = ((DESKTOP >> sh) & 0xff) as f32;
        (a + (b - a) * t) as u32
    };
    (m(16) << 16) | (m(8) << 8) | m(0)
}

struct Row {
    name: &'static str,
    slot: usize,
    span: (f32, f32),
    fx: (u8, u8), // (適用数, うち無効)
    mute: bool,
    dimmed_by_solo: bool,
    dev: &'static [&'static str], // 逸脱グリフ。既定なら空
    keys: &'static [(f32, f32)],
    note: &'static str,
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-row.png".into());
    let rows = [
        Row {
            name: "sky_plate",
            slot: 0,
            span: (0.0, 20.0),
            fx: (0, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[],
            note: "default - silent",
        },
        Row {
            name: "city_a",
            slot: 1,
            span: (0.0, 26.0),
            fx: (3, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[(6.0, 0.35), (14.0, 0.02)],
            note: "3 effects",
        },
        Row {
            name: "city_b",
            slot: 1,
            span: (0.0, 22.0),
            fx: (3, 1),
            mute: false,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[],
            note: "1 of 3 bypassed",
        },
        Row {
            name: "glow",
            slot: 2,
            span: (0.0, 18.0),
            fx: (1, 0),
            mute: true,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[],
            note: "muted",
        },
        Row {
            name: "hero",
            slot: 3,
            span: (0.0, 24.0),
            fx: (2, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[(9.0, 0.70)],
            note: "soloed - stays normal",
        },
        Row {
            name: "hero_bg",
            slot: 0,
            span: (0.0, 20.0),
            fx: (1, 0),
            mute: false,
            dimmed_by_solo: true,
            dev: &[],
            keys: &[],
            note: "dimmed because something else is soloed",
        },
        Row {
            name: "titles",
            slot: 5,
            span: (0.0, 21.0),
            fx: (0, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &["S", "%"],
            keys: &[],
            note: "blend != normal, opacity < 100",
        },
        Row {
            name: "b_roll",
            slot: 2,
            span: (0.0, 19.0),
            fx: (1, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &["T"],
            keys: &[],
            note: "retimed (TimeMap != identity)",
        },
        Row {
            name: "old_take",
            slot: 4,
            span: (0.0, 17.0),
            fx: (0, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &["!"],
            keys: &[],
            note: "source missing",
        },
        Row {
            name: "BG",
            slot: 3,
            span: (0.0, 30.0),
            fx: (2, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &[],
            keys: &[(11.0, 0.4)],
            note: "folded group - group effects shown here",
        },
        Row {
            name: "n",
            slot: 0,
            span: (0.0, 4.5),
            fx: (2, 0),
            mute: false,
            dimmed_by_solo: false,
            dev: &["%"],
            keys: &[],
            note: "too narrow - drops from the right",
        },
    ];
    let h = RULER_H + 1.0 + rows.len() as f32 * ROW + 62.0;
    let mut sf = surfaces::raster_n32_premul(((W * SCALE) as i32, (h * SCALE) as i32)).unwrap();
    let cv = sf.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));
    let bx = |b: f32| LEFT + b / BARS * (W - LEFT - 260.0);

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
    for (i, r) in rows.iter().enumerate() {
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

        let folded = i == 8;
        text(
            cv,
            if folded { ">" } else { "" },
            8.0,
            y + 13.5,
            9.0,
            rgb(DIM),
        );
        text(
            cv,
            r.name,
            if folded { 18.0 } else { 12.0 },
            y + 13.5,
            9.0,
            gray(0x9a),
        );

        let bar = Rect::from_ltrb(bx(r.span.0), y, bx(r.span.1), y + ROW - 1.0);
        let quiet = r.mute || r.dimmed_by_solo;
        let face = if r.mute {
            dim(P[r.slot], 0.74)
        } else if r.dimmed_by_solo {
            dim(P[r.slot], 0.5)
        } else {
            P[r.slot]
        };
        fill(cv, bar, rgb(face));
        // mute は面を沈めたうえで、斜線1本だけ引いて「意図的に消してある」を示す
        if r.mute {
            let mut p = Paint::default();
            p.set_anti_alias(true);
            p.set_stroke_width(1.0);
            p.set_color(argb(0x4a, 0x000000));
            let mut i = bar.left - ROW;
            while i < bar.right {
                cv.draw_line((i, bar.bottom), (i + ROW, bar.top), &p);
                i += 7.0;
            }
        }
        let ink = if quiet {
            argb(0x88, ON_BAR)
        } else {
            argb(0xff, ON_BAR)
        };
        let cy = bar.center_y();

        // ── 右クラスタ。既定のものは出さない ──
        let mut x = bar.right - 4.0;
        let mut place = |wd: f32, x: &mut f32| -> Option<f32> {
            if *x - wd < bar.left + 28.0 {
                None
            } else {
                *x -= wd;
                Some(*x)
            }
        };
        if r.fx.0 > 0 {
            let lbl = if r.fx.1 > 0 {
                format!("fx {}/{}", r.fx.0 - r.fx.1, r.fx.0)
            } else {
                format!("fx {}", r.fx.0)
            };
            let wd = measure(&lbl, 8.5) + 8.0;
            if let Some(px) = place(wd, &mut x) {
                let a = if quiet {
                    0x70
                } else if r.fx.1 > 0 {
                    0xff
                } else {
                    0xb4
                };
                text(cv, &lbl, px + 4.0, cy + 3.2, 8.5, argb(a, ON_BAR));
            }
        }
        for g in r.dev.iter().rev() {
            if let Some(px) = place(13.0, &mut x) {
                let mut p = Paint::default();
                p.set_anti_alias(true);
                p.set_color(argb(0x30, 0x000000));
                cv.draw_circle((px + 5.5, cy), 5.5, &p);
                let w = measure(g, 8.0);
                text(
                    cv,
                    g,
                    px + 5.5 - w / 2.0,
                    cy + 3.0,
                    8.0,
                    argb(if quiet { 0x70 } else { 0xd8 }, ON_BAR),
                );
            }
        }

        // 名前は残った幅まで
        let avail = x - bar.left - 9.0;
        if avail > 18.0 {
            text(cv, r.name, bar.left + 5.0, cy + 3.2, 8.5, ink);
        }
        for (b, d) in r.keys {
            diamond(cv, bx(*b), cy, *d);
        }
        text(cv, r.note, W - 250.0, y + 13.5, 8.5, gray(0x6e));
        y += ROW;
    }

    let ly = y + 18.0;
    text(
        cv,
        "the bar is READ-ONLY. no widget to mis-hit. state changes live in the Inspector and the keymap.",
        8.0,
        ly,
        9.0,
        rgb(DIM),
    );
    text(
        cv,
        "mute = the face itself sinks + hatch. solo = everything else sinks. no M/S button on the bar.",
        8.0,
        ly + 13.0,
        9.0,
        gray(0x5f),
    );
    text(
        cv,
        "S = blend mode   % = opacity < 100   T = retimed   ! = source missing",
        8.0,
        ly + 26.0,
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
