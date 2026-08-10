// クリッピング表示 — クリスタ/フォトショ準拠
//   下のものは何も変わらない。乗っている側にだけ印が付く。
//   クリップされている = それ自体が半透明。破線は補助。
//   下の対象の範囲を見て色を変えることはしない（下が動く度に追従が要り、バグの温床）。
//   有効だが効いていない時は印が灰色（クリスタが赤→灰でやっているのと同じ）。
// UI表記は「マスク」と言わない。動作をそのまま名前にする。

use skia_safe::{
    Color, EncodedImageFormat, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder,
    PathEffect, Rect, surfaces,
};

const SCALE: f32 = 2.0;
const W: f32 = 1080.0;
const LEFT: f32 = 12.0;
const SURF_X: f32 = 92.0;
const RULER_H: f32 = 18.0;
const ROW: f32 = 20.0;
const BARS: f32 = 36.0;
const NOTE_X: f32 = 620.0;

const DESKTOP: u32 = 0x2a2a2a;
const SURFACE_BG: u32 = 0x363636;
const CONTRAST: u32 = 0x111111;
const RULER_MARK: u32 = 0x919191;
const ON_BAR: u32 = 0x141414;
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

// 乗っている側だけに付く印。下向き＝直下のものに乗る。
fn clip_mark(cv: &skia_safe::Canvas, x: f32, cy: f32, working: bool) {
    let c = if working { rgb(0xd8734f) } else { gray(0x7a) };
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    let mut b = PathBuilder::new();
    b.move_to((x, cy - 4.6));
    b.line_to((x + 7.0, cy - 4.6));
    b.line_to((x + 3.5, cy + 1.0));
    b.close();
    cv.draw_path(&b.detach(), &p);
    cv.draw_rect(Rect::from_xywh(x, cy + 2.6, 7.0, 1.6), &p);
}

// クリップされている面。半透明が主、破線が補助。下は一切見ない。
fn clipped_face(cv: &skia_safe::Canvas, r: Rect, col: u32) {
    fill(cv, r, argb(0x66, col));
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_style(PaintStyle::Stroke);
    p.set_stroke_width(1.0);
    p.set_color(argb(0xcc, col));
    p.set_path_effect(PathEffect::dash(&[3.0, 3.0], 0.0));
    cv.draw_rect(
        Rect::from_ltrb(r.left + 0.5, r.top + 0.5, r.right - 0.5, r.bottom - 0.5),
        &p,
    );
}

struct Row {
    name: &'static str,
    slot: usize,
    a: f32,
    b: f32,
    clipped: bool,
    base: Option<(f32, f32)>,
    note: &'static str,
}

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "motolii-clip.png".into());
    let rows = [
        Row {
            name: "grunge_tex",
            slot: 2,
            a: 4.0,
            b: 30.0,
            clipped: true,
            base: Some((0.0, 0.0)),
            note: "clipped - translucent. does not stand on its own",
        },
        Row {
            name: "logo_shape",
            slot: 3,
            a: 2.0,
            b: 24.0,
            clipped: false,
            base: None,
            note: "the base. nothing about it changes",
        },
        Row {
            name: "",
            slot: 0,
            a: 0.0,
            b: 0.0,
            clipped: false,
            base: None,
            note: "",
        },
        Row {
            name: "tint",
            slot: 5,
            a: 6.0,
            b: 28.0,
            clipped: true,
            base: None,
            note: "clipping ON but not working - the mark goes grey",
        },
        Row {
            name: "old_take",
            slot: 4,
            a: 8.0,
            b: 26.0,
            clipped: false,
            base: None,
            note: "muted, for contrast - sinks + hatch, not translucent",
        },
        Row {
            name: "bg_plate",
            slot: 0,
            a: 0.0,
            b: 36.0,
            clipped: false,
            base: None,
            note: "",
        },
    ];
    let h = RULER_H + 1.0 + rows.len() as f32 * ROW + 44.0;
    let mut sf = surfaces::raster_n32_premul(((W * SCALE) as i32, (h * SCALE) as i32)).unwrap();
    let cv = sf.canvas();
    cv.scale((SCALE, SCALE));
    cv.clear(rgb(DESKTOP));
    let bx = |b: f32| SURF_X + b / BARS * (NOTE_X - SURF_X - 14.0);

    fill(cv, Rect::from_ltrb(0.0, 0.0, SURF_X, h), rgb(SURFACE_BG));
    fill(cv, Rect::from_ltrb(0.0, 0.0, W, RULER_H), rgb(0x464646));
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
                11.0,
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
    for r in &rows {
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
        if r.name.is_empty() {
            y += ROW;
            continue;
        }

        let bar = Rect::from_ltrb(bx(r.a), y, bx(r.b), y + ROW - 1.0);
        let col = P[r.slot];
        let muted = r.name == "old_take";
        if r.clipped {
            clipped_face(cv, bar, col);
        } else if muted {
            let m = |sh: u32| {
                let a = ((col >> sh) & 0xff) as f32;
                let b = ((DESKTOP >> sh) & 0xff) as f32;
                (a + (b - a) * 0.74) as u32
            };
            fill(cv, bar, rgb((m(16) << 16) | (m(8) << 8) | m(0)));
            let mut p = Paint::default();
            p.set_anti_alias(true);
            p.set_stroke_width(1.0);
            p.set_color(argb(0x46, 0x000000));
            let mut i = bar.left - ROW;
            while i < bar.right {
                cv.draw_line((i, bar.bottom), (i + ROW, bar.top), &p);
                i += 7.0;
            }
        } else {
            fill(cv, bar, rgb(col));
        }
        let working = r.base.is_some();
        let tx = if r.clipped {
            clip_mark(cv, bar.left + 5.0, cy, working);
            bar.left + 17.0
        } else {
            bar.left + 5.0
        };
        text(
            cv,
            r.name,
            tx,
            cy + 3.2,
            8.5,
            if r.clipped {
                argb(0xdd, ON_BAR)
            } else if muted {
                argb(0x82, ON_BAR)
            } else {
                argb(0xff, ON_BAR)
            },
        );
        text(cv, r.note, NOTE_X, cy + 3.2, 8.5, gray(0x82));
        y += ROW;
    }

    text(
        cv,
        "clipped = translucent. it does not stand on its own. the dashed border is only a second cue.",
        LEFT,
        y + 16.0,
        9.5,
        gray(0xa8),
    );
    text(
        cv,
        "nothing is computed from what is below. the look depends only on this item's own clipping flag.",
        LEFT,
        y + 29.0,
        9.0,
        gray(0x60),
    );

    let png = sf
        .image_snapshot()
        .encode(None, EncodedImageFormat::PNG, 100)
        .unwrap();
    std::fs::write(&out, png.as_bytes()).unwrap();
    println!("{out}  {}x{}", (W * SCALE) as i32, (h * SCALE) as i32);
}
