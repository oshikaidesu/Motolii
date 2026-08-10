// timeline 全体
//   Overview / 小節ruler / locator / Inbox / rail / packing面 / 分秒ruler
// 帯は利用者が作った器として残る（空でも消えない）。ただし名前も型も持たない。
// バーは読み取り専用。時間の操作だけがこの面に集まる。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect,
    surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1240.0;
const INBOX_W: f32 = 118.0;
const RAIL_W: f32 = 84.0;
const SURF_X: f32 = INBOX_W + RAIL_W;
const OVER_H: f32 = 22.0;
const RULER_H: f32 = 18.0;
const LOC_H: f32 = 15.0;
const ROW: f32 = 20.0;
const TIME_H: f32 = 16.0;
const SONG_BARS: f32 = 96.0; // 120BPM 4/4 → 1小節2秒 → 3:12
const VIEW_A: f32 = 0.0;
const VIEW_B: f32 = 48.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const SURFACE_HI: u32 = 0x464646;
const SURFACE_LO: u32 = 0x242424;
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
fn dim_bg(c: u32, t: f32) -> u32 {
    let m = |sh: u32| {
        let a = ((c >> sh) & 0xff) as f32;
        let b = ((DESKTOP >> sh) & 0xff) as f32;
        (a + (b - a) * t) as u32
    };
    (m(16) << 16) | (m(8) << 8) | m(0)
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

fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, d: f32, sel: bool) {
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
    if sel {
        // 選択中はさらに外側のoutline
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(1.4);
        p.set_color(Color::WHITE);
        cv.draw_path(&path(7.6), &p);
    }
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

fn glyph(cv: &skia_safe::Canvas, cx: f32, cy: f32, kind: &str, on_dark: bool, quiet: bool) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    let warn = kind == "missing";
    p.set_color(if warn {
        argb(0xcc, 0xc4552e)
    } else {
        argb(if on_dark { 0x40 } else { 0x2e }, 0x000000)
    });
    cv.draw_circle((cx, cy), 5.5, &p);
    let c = if warn {
        gray(0xf4)
    } else if on_dark {
        gray(0xc8)
    } else {
        argb(if quiet { 0x70 } else { 0xd8 }, ON_BAR)
    };
    p.set_color(c);
    match kind {
        "opacity" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.0, cy - 3.0, 6.0, 6.0), &p);
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 3.0, cy - 3.0, 3.0, 6.0), &p);
        }
        "blend" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.4, cy - 3.4, 4.6, 4.6), &p);
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 1.2, cy - 1.2, 4.6, 4.6), &p);
        }
        "retime" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.2);
            for o in [-2.6f32, 0.6] {
                let mut b = PathBuilder::new();
                b.move_to((cx + o, cy - 3.0));
                b.line_to((cx + o + 2.4, cy));
                b.line_to((cx + o, cy + 3.0));
                cv.draw_path(&b.detach(), &p);
            }
        }
        "bypass" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.2);
            cv.draw_circle((cx, cy), 3.2, &p);
            cv.draw_line((cx - 2.6, cy + 2.6), (cx + 2.6, cy - 2.6), &p);
        }
        "muted" => {
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            cv.draw_rect(Rect::from_xywh(cx - 3.2, cy - 3.2, 6.4, 6.4), &p);
            for i in 0..3 {
                let o = -2.4 + i as f32 * 2.4;
                cv.draw_line((cx + o, cy + 3.2), (cx + o + 3.2, cy - 3.2), &p);
            }
        }
        _ => {
            p.set_style(PaintStyle::Fill);
            cv.draw_rect(Rect::from_xywh(cx - 0.8, cy - 3.4, 1.6, 4.4), &p);
            cv.draw_rect(Rect::from_xywh(cx - 0.8, cy + 2.0, 1.6, 1.6), &p);
        }
    }
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

struct Clip {
    a: f32,
    b: f32,
    slot: usize,
    name: &'static str,
    fx: (u8, u8),
    mute: bool,
    sel: bool,
    dev: &'static [&'static str],
    keys: &'static [(f32, f32, bool)],
}
struct Band {
    mute: bool,
    solo: bool,
    mixed: bool,
    clips: &'static [Clip],
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-full.png".into());
    let bands: [Band; 6] = [
        Band {
            mute: false,
            solo: false,
            mixed: false,
            clips: &[
                Clip {
                    a: 0.0,
                    b: 14.0,
                    slot: 0,
                    name: "sky_plate",
                    fx: (0, 0),
                    mute: false,
                    sel: false,
                    dev: &[],
                    keys: &[],
                },
                Clip {
                    a: 14.0,
                    b: 27.0,
                    slot: 0,
                    name: "sky_plate",
                    fx: (2, 0),
                    mute: false,
                    sel: true,
                    dev: &[],
                    keys: &[(14.0, 0.42, false), (20.0, 0.03, true)],
                },
                Clip {
                    a: 30.0,
                    b: 44.0,
                    slot: 1,
                    name: "city_a",
                    fx: (3, 1),
                    mute: false,
                    sel: false,
                    dev: &["retime"],
                    keys: &[],
                },
            ],
        },
        Band {
            mute: false,
            solo: true,
            mixed: false,
            clips: &[
                Clip {
                    a: 4.0,
                    b: 22.0,
                    slot: 3,
                    name: "hero",
                    fx: (2, 0),
                    mute: false,
                    sel: false,
                    dev: &[],
                    keys: &[(8.0, 0.71, false), (13.0, 0.02, false), (18.0, 0.15, false)],
                },
                Clip {
                    a: 26.0,
                    b: 40.0,
                    slot: 3,
                    name: "hero",
                    fx: (2, 0),
                    mute: false,
                    sel: false,
                    dev: &[],
                    keys: &[(31.0, 0.30, false)],
                },
            ],
        },
        Band {
            mute: false,
            solo: false,
            mixed: true,
            clips: &[
                Clip {
                    a: 0.0,
                    b: 18.0,
                    slot: 2,
                    name: "grain",
                    fx: (1, 0),
                    mute: false,
                    sel: false,
                    dev: &[],
                    keys: &[],
                },
                Clip {
                    a: 20.0,
                    b: 35.0,
                    slot: 2,
                    name: "grain",
                    fx: (1, 0),
                    mute: true,
                    sel: false,
                    dev: &[],
                    keys: &[],
                },
                Clip {
                    a: 38.0,
                    b: 48.0,
                    slot: 2,
                    name: "grain",
                    fx: (1, 0),
                    mute: false,
                    sel: false,
                    dev: &[],
                    keys: &[],
                },
            ],
        },
        Band {
            mute: false,
            solo: false,
            mixed: false,
            clips: &[
                Clip {
                    a: 6.0,
                    b: 13.0,
                    slot: 5,
                    name: "title_01",
                    fx: (0, 0),
                    mute: false,
                    sel: false,
                    dev: &["blend", "opacity"],
                    keys: &[],
                },
                Clip {
                    a: 24.0,
                    b: 32.0,
                    slot: 5,
                    name: "title_02",
                    fx: (0, 0),
                    mute: false,
                    sel: false,
                    dev: &["opacity"],
                    keys: &[(24.0, 0.5, false)],
                },
            ],
        },
        // 空の帯。利用者が作った器なので残る
        Band {
            mute: false,
            solo: false,
            mixed: false,
            clips: &[],
        },
        Band {
            mute: false,
            solo: false,
            mixed: false,
            clips: &[Clip {
                a: 0.0,
                b: 48.0,
                slot: 1,
                name: "track_master.wav",
                fx: (0, 0),
                mute: false,
                sel: false,
                dev: &[],
                keys: &[],
            }],
        },
    ];
    let locators: [(f32, &str); 4] = [
        (0.0, "intro"),
        (8.0, "verse 1"),
        (24.0, "chorus"),
        (40.0, "verse 2"),
    ];

    let body_h = bands.len() as f32 * ROW;
    let h = OVER_H + RULER_H + LOC_H + 1.0 + body_h + TIME_H + 2.0;
    let mut sf = surfaces::raster_n32_premul(((W * SCALE) as i32, (h * SCALE) as i32)).unwrap();
    let cv = sf.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));

    let sw = W - SURF_X - 6.0;
    let bx = |b: f32| SURF_X + (b - VIEW_A) / (VIEW_B - VIEW_A) * sw; // 表示範囲
    let ox = |b: f32| SURF_X + b / SONG_BARS * sw; // Overview は全曲

    fill(cv, Rect::from_ltrb(0.0, 0.0, SURF_X, h), rgb(SURFACE_BG));

    // ── Overview: 全曲をその場でミニチュア（畳みと同じ技法の別スケール） ──
    fill(cv, Rect::from_ltrb(SURF_X, 0.0, W, OVER_H), rgb(SURFACE_LO));
    text(cv, "overview", 8.0, 14.0, 9.0, rgb(DIM));
    for (i, band) in bands.iter().enumerate() {
        let yy = 3.0 + i as f32 * 2.8;
        for c in band.clips {
            fill(
                cv,
                Rect::from_ltrb(ox(c.a), yy, ox(c.b), yy + 2.2),
                argb(0xcc, P[c.slot]),
            );
        }
    }
    let (va, vb) = (ox(VIEW_A), ox(VIEW_B));
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(gray(0xd8));
    cv.draw_rect(Rect::from_ltrb(va, 1.0, vb, OVER_H - 1.0), &p);
    fill(
        cv,
        Rect::from_ltrb(0.0, OVER_H, W, OVER_H + 1.0),
        rgb(CONTRAST),
    );

    // ── 小節 ruler ──
    let ry = OVER_H + 1.0;
    fill(
        cv,
        Rect::from_ltrb(0.0, ry, W, ry + RULER_H),
        rgb(SURFACE_HI),
    );
    for b in (VIEW_A as i32..=VIEW_B as i32).step_by(4) {
        let x = bx(b as f32);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H - 6.0, x + 1.0, ry + RULER_H),
            gray(0x6a),
        );
        text(
            cv,
            &format!("{}", b + 1),
            x + 3.0,
            ry + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
    }
    fill(
        cv,
        Rect::from_ltrb(bx(8.0), ry, bx(24.0), ry + 3.0),
        rgb(RULER_MARK),
    ); // loop brace

    // ── locator lane ──
    let ly = ry + RULER_H;
    fill(cv, Rect::from_ltrb(0.0, ly, W, ly + LOC_H), rgb(SURFACE_BG));
    for (b, name) in locators {
        let x = bx(b);
        fill(cv, Rect::from_ltrb(x, ly, x + 1.0, ly + LOC_H), gray(0x8a));
        let mut t = PathBuilder::new();
        t.move_to((x + 1.0, ly + 2.0));
        t.line_to((x + 6.0, ly + 5.0));
        t.line_to((x + 1.0, ly + 8.0));
        t.close();
        let mut pp = Paint::default();
        pp.set_anti_alias(true);
        pp.set_color(gray(0x8a));
        cv.draw_path(&t.detach(), &pp);
        text(cv, name, x + 9.0, ly + 11.0, 8.5, gray(0xa8));
    }
    fill(
        cv,
        Rect::from_ltrb(0.0, ly + LOC_H, W, ly + LOC_H + 1.0),
        rgb(CONTRAST),
    );

    // ── Inbox ──
    text(cv, "Inbox", 9.0, ry + 12.0, 9.5, gray(0xc0));
    text(cv, "3", INBOX_W - 15.0, ry + 12.0, 9.0, rgb(DIM));
    let by0 = ly + LOC_H + 1.0;
    for (i, s) in ["street_loop.mp4", "check cut 1:04", "proxy 2 files"]
        .iter()
        .enumerate()
    {
        let y = by0 + 6.0 + i as f32 * ROW;
        let mut pp = Paint::default();
        pp.set_anti_alias(true);
        pp.set_color(rgb(FILL_HANDLE));
        match i {
            0 => {
                cv.draw_rect(Rect::from_xywh(9.0, y, 7.0, 7.0), &pp);
            }
            1 => {
                let mut d = PathBuilder::new();
                d.move_to((12.5, y - 1.0));
                d.line_to((16.5, y + 3.5));
                d.line_to((12.5, y + 8.0));
                d.line_to((8.5, y + 3.5));
                d.close();
                cv.draw_path(&d.detach(), &pp);
            }
            _ => {
                cv.draw_circle((12.5, y + 3.5), 3.6, &pp);
            }
        }
        text(cv, s, 22.0, y + 7.0, 9.0, gray(0xb0));
        fill(
            cv,
            Rect::from_ltrb(0.0, y + ROW - 6.0, INBOX_W, y + ROW - 5.0),
            argb(0x40, 0x000000),
        );
    }

    // ── bands ──
    let mut y = by0;
    for band in &bands {
        for b in VIEW_A as i32..=VIEW_B as i32 {
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

        tog(cv, INBOX_W + 5.0, cy, "M", band.mute, band.mixed, 0xd8d8d8);
        tog(cv, INBOX_W + 21.0, cy, "S", band.solo, false, ACCENT);
        if !band.clips.is_empty() {
            text(
                cv,
                &format!("{}", band.clips.len()),
                INBOX_W + 41.0,
                cy + 3.2,
                8.5,
                gray(0x8e),
            );
        }
        let mut roll: Vec<&str> = vec![];
        for c in band.clips {
            for d in c.dev {
                if !roll.contains(d) {
                    roll.push(d);
                }
            }
            if c.mute && !roll.contains(&"muted") {
                roll.push("muted");
            }
            if c.fx.1 > 0 && !roll.contains(&"bypass") {
                roll.push("bypass");
            }
        }
        let mut rx = INBOX_W + RAIL_W - 5.0;
        for g in roll.iter().rev() {
            rx -= 13.0;
            glyph(cv, rx + 5.5, cy, g, true, false);
        }

        for c in band.clips {
            let quiet = c.mute || band.mute;
            let r = Rect::from_ltrb(bx(c.a), y, bx(c.b), y + ROW - 1.0);
            fill(
                cv,
                r,
                rgb(if quiet {
                    dim_bg(P[c.slot], 0.74)
                } else {
                    P[c.slot]
                }),
            );
            if quiet {
                let mut pp = Paint::default();
                pp.set_anti_alias(true);
                pp.set_stroke_width(1.0);
                pp.set_color(argb(0x46, 0x000000));
                let mut i = r.left - ROW;
                while i < r.right {
                    cv.draw_line((i, r.bottom), (i + ROW, r.top), &pp);
                    i += 7.0;
                }
            }
            let ink = if quiet {
                argb(0x82, ON_BAR)
            } else {
                argb(0xff, ON_BAR)
            };
            let mut x = r.right - 4.0;
            let mut place = |wd: f32, x: &mut f32| -> Option<f32> {
                if *x - wd < r.left + 26.0 {
                    None
                } else {
                    *x -= wd;
                    Some(*x)
                }
            };
            if c.fx.0 > 0 {
                let lbl = if c.fx.1 > 0 {
                    format!("fx {}/{}", c.fx.0 - c.fx.1, c.fx.0)
                } else {
                    format!("fx {}", c.fx.0)
                };
                let wd = measure(&lbl, 8.5) + 8.0;
                if let Some(px) = place(wd, &mut x) {
                    text(
                        cv,
                        &lbl,
                        px + 4.0,
                        cy + 3.2,
                        8.5,
                        argb(
                            if quiet {
                                0x66
                            } else if c.fx.1 > 0 {
                                0xff
                            } else {
                                0xb0
                            },
                            ON_BAR,
                        ),
                    );
                }
            }
            for g in c.dev.iter().rev() {
                if let Some(px) = place(13.0, &mut x) {
                    glyph(cv, px + 5.5, cy, g, false, quiet);
                }
            }
            let nw = measure(c.name, 8.5);
            let mut nx = r.left + 5.0;
            for (b, _, _) in c.keys {
                let kx = bx(*b);
                if kx > nx - 8.0 && kx < nx + nw + 6.0 {
                    nx = kx + 9.0;
                }
            }
            if x - nx > nw * 0.55 {
                text(cv, c.name, nx, cy + 3.2, 8.5, ink);
            }
            for (b, d, s) in c.keys {
                diamond(cv, bx(*b), cy, *d, *s);
            }
            if c.sel {
                // 選択は白outline
                let mut pp = Paint::default();
                pp.set_anti_alias(false);
                pp.set_style(PaintStyle::Stroke);
                pp.set_stroke_width(1.0);
                pp.set_color(Color::WHITE);
                cv.draw_rect(
                    Rect::from_ltrb(r.left + 0.5, r.top + 0.5, r.right - 0.5, r.bottom - 0.5),
                    &pp,
                );
            }
        }
        y += ROW;
    }

    // ── 分秒 ruler。完成条件が3〜5分なので尺が読めないと困る ──
    fill(cv, Rect::from_ltrb(0.0, y, W, y + TIME_H), rgb(SURFACE_BG));
    for b in (VIEW_A as i32..=VIEW_B as i32).step_by(8) {
        let x = bx(b as f32);
        let sec = b * 2; // 120BPM 4/4
        fill(cv, Rect::from_ltrb(x, y, x + 1.0, y + 5.0), gray(0x6a));
        text(
            cv,
            &format!("{}:{:02}", sec / 60, sec % 60),
            x + 3.0,
            y + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
    }
    text(cv, "3:12 total", W - 64.0, y + 11.0, 8.5, rgb(DIM));

    fill(
        cv,
        Rect::from_ltrb(SURF_X - 1.0, 0.0, SURF_X, h),
        rgb(CONTRAST),
    );
    fill(
        cv,
        Rect::from_ltrb(INBOX_W, OVER_H, INBOX_W + 1.0, h),
        rgb(CONTRAST),
    );

    // playhead: 時間面の中だけ。頭は小節rulerへ
    let px = bx(21.0);
    fill(
        cv,
        Rect::from_ltrb(px, ry + RULER_H, px + 1.0, y),
        gray(0xe7),
    );
    let mut tri = PathBuilder::new();
    tri.move_to((px - 4.0, ry + RULER_H - 6.0));
    tri.line_to((px + 5.0, ry + RULER_H - 6.0));
    tri.line_to((px + 0.5, ry + RULER_H));
    tri.close();
    let mut pp = Paint::default();
    pp.set_anti_alias(true);
    pp.set_color(gray(0xe7));
    cv.draw_path(&tri.detach(), &pp);

    let png = sf
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .unwrap();
    std::fs::write(&out, png.as_bytes()).unwrap();
    println!("{out}  {}x{}", (W * SCALE) as i32, (h * SCALE) as i32);
}
