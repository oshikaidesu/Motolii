// Motolii timeline — skia直描き。色は Ableton Live 12
// "Default Dark Neutral Medium.ask" から抽出した実値を使う（推測しない）。
// 構成はMotolii規範（固定名列なし / Inbox / 無名action rail / packing面）。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Point,
    Rect, surfaces,
};

const SCALE: f32 = 2.0; // Retina実機と同じ物理解像度で出す
const W: f32 = 940.0;
const H: f32 = RULER_H + 26.0 * 8.0 + 62.0;

fn band_y(b: usize) -> f32 {
    RULER_H + BAND_HEIGHTS[..b].iter().sum::<f32>()
}

const INBOX_W: f32 = 132.0;
const RAIL_W: f32 = 30.0;
const SURF_X: f32 = INBOX_W + RAIL_W;
const RULER_H: f32 = 23.0;
// 実測(Live 12 manual ArrangementViewTop/DemoArrangement, 1880px幅):
// 行高 約24px / 小節幅 約14.5px。行は薄く、数が多い。
const BANDS: usize = 9;
const BAND_HEIGHTS: [f32; BANDS] = [26.0, 26.0, 26.0, 26.0, 26.0, 26.0, 26.0, 26.0, 62.0];
const WAVE_MIN_H: f32 = 34.0; // これ未満の帯は名前バーのみ（Liveと同じ）
const BARS: i32 = 64;

// ── Ableton 実値 ───────────────────────────────────────────
const CONTROL_BACKGROUND: u32 = 0x1e1e1e;
const SURFACE_AREA: u32 = 0x242424;
const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BACKGROUND: u32 = 0x363636;
const DETAIL_VIEW_BACKGROUND: u32 = 0x3e3e3e;
const SURFACE_HIGHLIGHT: u32 = 0x464646;
const CONTROL_FILL_HANDLE: u32 = 0x5d5d5d;
const TEXT_DISABLED: u32 = 0x757575;
const RULER_MARKINGS: u32 = 0x919191;
const CONTROL_FOREGROUND: u32 = 0xb5b5b5;
const CONTROL_CONTRAST_FRAME: u32 = 0x111111;
const CHOSEN_DEFAULT: u32 = 0xffad56;
const CONTROL_ON_FOREGROUND: u32 = 0x070707;
// GridLineBase #06060654 / ArrangerGridTiles #0a0a0a19
const GRID_BASE: (u8, u32) = (0x54, 0x060606);
const GRID_TILE: (u8, u32) = (0x19, 0x0a0a0a);

// Live swatch全16色は利用者が選ぶ範囲で知覚明度が揃わない。
// 規範(同じObject=同色, 近い明度, 低chroma)に合う subset だけ使う。
const CLIP: [u32; 6] = [0x6baace, 0x999565, 0x8b7936, 0x4881aa, 0x954eb2, 0xa0a0a0];

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
fn shade(v: u32, f: f32) -> Color {
    let c = |s: u32| (((s & 0xff) as f32 * f).clamp(0.0, 255.0)) as u8;
    Color::from_rgb(c(v >> 16), c(v >> 8), c(v))
}

fn fill(cv: &skia_safe::Canvas, r: Rect, c: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_color(c);
    cv.draw_rect(r, &p);
}

// 1論理px = SCALE物理px。境界を0.5ずらして罫線を潰さない。
fn vline(cv: &skia_safe::Canvas, x: f32, y0: f32, y1: f32, c: Color) {
    fill(cv, Rect::from_ltrb(x, y0, x + 1.0, y1), c);
}
fn hline(cv: &skia_safe::Canvas, y: f32, x0: f32, x1: f32, c: Color) {
    fill(cv, Rect::from_ltrb(x0, y, x1, y + 1.0), c);
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

fn waveform(cv: &skia_safe::Canvas, r: Rect, seed: f32, c: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    p.set_style(PaintStyle::Fill);
    let mid = r.center_y();
    let n = ((r.width() / 1.5) as usize).clamp(12, 1200);
    let amp = |i: usize| -> f32 {
        let t = i as f32 / (n - 1) as f32;
        let env = (t * 3.7 + seed).sin() * 0.28 + 0.72;
        (((i as f32 * 0.61 + seed).sin() * 0.46
            + (i as f32 * 1.93 + seed * 1.7).sin() * 0.31
            + (i as f32 * 5.3 + seed * 0.4).sin() * 0.23)
            .abs())
            * r.height()
            * 0.46
            * env
    };
    let mut path = PathBuilder::new();
    path.move_to((r.left, mid));
    for i in 0..n {
        let x = r.left + i as f32 / (n - 1) as f32 * r.width();
        path.line_to((x, mid - amp(i)));
    }
    for i in (0..n).rev() {
        let x = r.left + i as f32 / (n - 1) as f32 * r.width();
        path.line_to((x, mid + amp(i)));
    }
    path.close();
    cv.draw_path(&path.detach(), &p);
}

fn clip(
    cv: &skia_safe::Canvas,
    band: usize,
    x0: f32,
    x1: f32,
    name: &str,
    ci: usize,
    wave: bool,
    seed: f32,
    selected: bool,
) {
    let y = band_y(band);
    let bh = BAND_HEIGHTS[band];
    // Liveのclipは行を丸ごと埋める。隙間は1px。
    let r = Rect::from_ltrb(x0, y + 1.0, x1, y + bh - 1.0);
    let base = CLIP[ci % CLIP.len()];
    let tall = bh >= WAVE_MIN_H;
    fill(cv, r, rgb(base));
    if tall {
        fill(
            cv,
            Rect::from_ltrb(r.left, r.top, r.right, r.top + 13.0),
            shade(base, 0.84),
        );
        if wave {
            waveform(
                cv,
                Rect::from_ltrb(r.left + 1.0, r.top + 13.0, r.right - 1.0, r.bottom),
                seed,
                argb(0x59, 0x000000),
            );
        }
    } else {
        // 薄い行では上辺1pxだけ濃くする（Liveの見え方）
        hline(cv, r.top, r.left, r.right, shade(base, 0.78));
    }
    let ty = if tall {
        r.top + 9.8
    } else {
        r.top + bh / 2.0 + 1.6
    };
    if r.width() > 30.0 {
        text(cv, name, r.left + 4.0, ty, 9.5, rgb(CONTROL_ON_FOREGROUND));
    }
    if selected {
        let mut p = Paint::default();
        p.set_anti_alias(false);
        p.set_style(PaintStyle::Stroke);
        p.set_stroke_width(1.0);
        p.set_color(Color::WHITE);
        cv.draw_rect(
            Rect::from_ltrb(r.left + 0.5, r.top + 0.5, r.right - 0.5, r.bottom - 0.5),
            &p,
        );
    }
}

// 規範: 10px, 2px暗stroke, 1px明outer ring
fn keyframe(cv: &skia_safe::Canvas, cx: f32, cy: f32, filled: bool) {
    let d = |s: f32| {
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
    cv.draw_path(&d(6.0), &p);
    if filled {
        p.set_style(PaintStyle::Fill);
        p.set_color(rgb(0x1e1e1e));
        cv.draw_path(&d(5.0), &p);
    }
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(2.0);
    p.set_color(rgb(0x1e1e1e));
    cv.draw_path(&d(5.0), &p);
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-tl.png".into());
    let mut surface =
        surfaces::raster_n32_premul(((W * SCALE) as i32, (H * SCALE) as i32)).expect("surface");
    let cv = surface.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));

    let surf_w = W - SURF_X;
    let bar_w = surf_w / BARS as f32;
    let bx = |bar: f32| SURF_X + bar * bar_w;

    // ── Inbox / rail の chrome ───────────────────────────
    fill(
        cv,
        Rect::from_ltrb(0.0, 0.0, SURF_X, H),
        rgb(SURFACE_BACKGROUND),
    );
    fill(
        cv,
        Rect::from_ltrb(0.0, 0.0, SURF_X, RULER_H),
        rgb(SURFACE_HIGHLIGHT),
    );
    text(cv, "Inbox", 9.0, 15.0, 10.0, rgb(CONTROL_FOREGROUND));
    text(cv, "3", INBOX_W - 14.0, 15.0, 10.0, rgb(TEXT_DISABLED));

    let items = [
        ("street_loop.mp4", 0),
        ("check cut at 1:04", 1),
        ("proxy · 2 files", 2),
    ];
    for (i, (label, kind)) in items.iter().enumerate() {
        let y = RULER_H + 6.0 + i as f32 * 18.0;
        let mut p = Paint::default();
        p.set_anti_alias(true);
        p.set_color(rgb(CONTROL_FILL_HANDLE));
        match kind {
            0 => {
                cv.draw_rect(Rect::from_xywh(9.0, y - 1.0, 7.0, 7.0), &p);
            }
            1 => {
                let mut d = PathBuilder::new();
                d.move_to((12.5, y - 2.0));
                d.line_to((16.0, y + 2.5));
                d.line_to((12.5, y + 7.0));
                d.line_to((9.0, y + 2.5));
                d.close();
                cv.draw_path(&d.detach(), &p);
            }
            _ => {
                cv.draw_circle((12.5, y + 2.5), 3.6, &p);
            }
        }
        text(cv, label, 22.0, y + 6.0, 9.5, rgb(CONTROL_FOREGROUND));
        hline(cv, y + 12.0, 0.0, INBOX_W, rgb(0x2f2f2f));
    }
    vline(cv, INBOX_W, 0.0, H, rgb(CONTROL_CONTRAST_FRAME));

    // ── action rail (名前を持たない) ──────────────────────
    for b in 0..BANDS {
        let y = band_y(b);
        let bh = BAND_HEIGHTS[b];
        let cx = INBOX_W + RAIL_W / 2.0;
        if bh < 22.0 {
            continue;
        }
        for (n, (lab, on)) in [("M", b == 0), ("S", b == 2)].iter().enumerate() {
            let x = cx - 14.0 + n as f32 * 14.0;
            let r = Rect::from_xywh(x, y + bh / 2.0 - 6.5, 13.0, 13.0);
            fill(
                cv,
                r,
                if *on {
                    rgb(if n == 1 { CHOSEN_DEFAULT } else { 0xd6d6d6 })
                } else {
                    rgb(CONTROL_FILL_HANDLE)
                },
            );
            hline(cv, r.top, r.left, r.right, rgb(0x6e6e6e));
            let mut p = Paint::default();
            p.set_anti_alias(false);
            p.set_style(PaintStyle::Stroke);
            p.set_stroke_width(1.0);
            p.set_color(rgb(0x3a3a3a));
            cv.draw_rect(r, &p);
            text(
                cv,
                lab,
                r.left + 3.6,
                r.bottom - 3.8,
                8.5,
                if *on {
                    rgb(CONTROL_ON_FOREGROUND)
                } else {
                    rgb(0xc4c4c4)
                },
            );
        }
        hline(cv, y + bh, INBOX_W, SURF_X, rgb(CONTROL_CONTRAST_FRAME));
    }
    vline(cv, SURF_X - 1.0, 0.0, H, rgb(CONTROL_CONTRAST_FRAME));

    // ── ruler ────────────────────────────────────────────
    fill(
        cv,
        Rect::from_ltrb(SURF_X, 0.0, W, RULER_H),
        rgb(SURFACE_BACKGROUND),
    );
    fill(
        cv,
        Rect::from_ltrb(bx(8.0), 0.0, bx(22.0), 5.0),
        rgb(RULER_MARKINGS),
    ); // loop brace
    for b in 0..=BARS {
        let x = bx(b as f32);
        let major = b % 4 == 0;
        vline(
            cv,
            x,
            RULER_H - if major { 9.0 } else { 4.0 },
            RULER_H,
            rgb(if major { 0x6a6a6a } else { 0x525252 }),
        );
        if major && b < BARS {
            text(
                cv,
                &format!("{}", b + 1),
                x + 3.0,
                12.0,
                9.0,
                rgb(RULER_MARKINGS),
            );
        }
    }
    hline(cv, RULER_H, SURF_X, W, rgb(CONTROL_CONTRAST_FRAME));

    // ── packing面: 地 + grid ─────────────────────────────
    for b in 0..BANDS {
        let y = band_y(b);
        fill(
            cv,
            Rect::from_ltrb(SURF_X, y, W, y + BAND_HEIGHTS[b]),
            rgb(if b % 2 == 0 { DESKTOP } else { SURFACE_AREA }),
        );
    }
    // 密度に応じて間引く。潰れた線は罫線でなく縞のノイズになる。
    let show_beat = bar_w >= 34.0;
    let bar_step = if bar_w >= 12.0 { 1 } else { 4 };
    for b in (0..=BARS).step_by(bar_step as usize) {
        let x = bx(b as f32);
        let major = b % 4 == 0;
        vline(
            cv,
            x,
            RULER_H,
            H,
            argb(
                if major { GRID_BASE.0 } else { GRID_TILE.0 },
                if major { GRID_BASE.1 } else { GRID_TILE.1 },
            ),
        );
        if show_beat {
            for q in 1..4 {
                let qx = x + bar_w * q as f32 / 4.0;
                if qx < W {
                    vline(cv, qx, RULER_H, H, argb(GRID_TILE.0, GRID_TILE.1));
                }
            }
        }
    }
    for b in 0..BANDS {
        hline(
            cv,
            band_y(b) + BAND_HEIGHTS[b],
            SURF_X,
            W,
            rgb(CONTROL_CONTRAST_FRAME),
        );
    }

    // ── clips（Abletonの密度で埋める） ───────────────────
    let rows: [(usize, usize, &str, &[(f32, f32)]); 8] = [
        (
            0,
            0,
            "skyline_a",
            &[(0.0, 12.0), (12.0, 24.0), (32.0, 44.0), (52.0, 64.0)],
        ),
        (
            1,
            1,
            "skyline_b",
            &[(4.0, 16.0), (16.0, 28.0), (44.0, 56.0)],
        ),
        (
            2,
            2,
            "street_loop",
            &[(8.0, 20.0), (24.0, 32.0), (40.0, 52.0), (56.0, 64.0)],
        ),
        (3, 3, "grain", &[(0.0, 32.0), (36.0, 64.0)]),
        (4, 4, "bloom", &[(16.0, 28.0), (44.0, 60.0)]),
        (
            5,
            5,
            "title_01",
            &[(8.0, 14.0), (20.0, 26.0), (36.0, 42.0), (52.0, 58.0)],
        ),
        (6, 0, "logo", &[(28.0, 34.0), (58.0, 64.0)]),
        (7, 2, "vignette", &[(0.0, 64.0)]),
    ];
    for (band, ci, name, spans) in rows {
        for (a, b) in spans {
            clip(
                cv,
                band,
                bx(*a),
                bx(*b),
                name,
                ci,
                false,
                0.0,
                band == 0 && *a == 12.0,
            );
        }
    }
    clip(
        cv,
        8,
        bx(0.0),
        bx(64.0),
        "track_master.wav",
        3,
        true,
        11.0,
        false,
    );

    // ── keyframe ─────────────────────────────────────────
    for (band, bar, f) in [
        (0usize, 12.0f32, true),
        (0, 18.0, false),
        (0, 24.0, false),
        (4, 16.0, false),
        (4, 28.0, false),
    ] {
        keyframe(cv, bx(bar), band_y(band) + BAND_HEIGHTS[band] / 2.0, f);
    }

    // ── playhead ─────────────────────────────────────────
    let px = bx(16.0);
    vline(cv, px, 0.0, H, rgb(0xe7e7e7));
    let mut tri = PathBuilder::new();
    tri.move_to((px - 4.0, 0.0));
    tri.line_to((px + 5.0, 0.0));
    tri.line_to((px + 0.5, 5.0));
    tri.close();
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(rgb(0xe7e7e7));
    cv.draw_path(&tri.detach(), &p);

    let png = surface
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .expect("encode");
    std::fs::write(&out, png.as_bytes()).expect("write");
    println!("{out}  {}x{}", (W * SCALE) as i32, (H * SCALE) as i32);
}
