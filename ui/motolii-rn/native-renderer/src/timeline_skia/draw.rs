//! 描画primitiveとruler。draw_timelineだけが製品1枚を組み立てる。

use skia_safe::{Color, Font, FontMgr, FontStyle, Paint, PaintStyle, PathBuilder, Rect};

use super::layout::{DESKTOP, FILL_HANDLE, ON_BAR};
use super::scene::TimelineScene;

pub(super) fn frame_duration_secs(scene: &TimelineScene) -> f32 {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        return 1.0;
    }
    scene.fps_den as f32 / scene.fps_num as f32
}

/// ラベル間隔。frame の整数倍。zoom で 1 frame〜数秒。
pub(super) fn ruler_label_step_secs(scene: &TimelineScene, surface_w: f32) -> f32 {
    let span = (scene.view_b - scene.view_a).max(1e-3);
    let secs_per_px = span / surface_w.max(1.0);
    let min_secs = secs_per_px * 48.0;
    let frame = frame_duration_secs(scene);
    let min_frames = (min_secs / frame).ceil().max(1.0);
    const NICE: [f32; 11] = [
        1.0, 2.0, 5.0, 10.0, 15.0, 30.0, 60.0, 120.0, 300.0, 600.0, 1800.0,
    ];
    let frames = NICE
        .iter()
        .copied()
        .find(|n| *n >= min_frames)
        .unwrap_or(min_frames);
    frames * frame
}

pub(super) fn first_tick_secs(view_a: f32, step: f32) -> f32 {
    let step = step.max(1e-6);
    (view_a / step).ceil() * step
}

/// 旧整数秒目盛。test互換。
pub(super) fn first_absolute_tick(view_a: f32, step: i32) -> i32 {
    first_tick_secs(view_a, step.max(1) as f32).round() as i32
}

pub(super) fn format_ruler_time(secs: f32, scene: &TimelineScene, with_frames: bool) -> String {
    if scene.fps_num <= 0 || scene.fps_den <= 0 {
        let s = secs.max(0.0).round() as i32;
        return format!("{}:{:02}", s / 60, s % 60);
    }
    let frame =
        (f64::from(secs.max(0.0)) * scene.fps_num as f64 / scene.fps_den as f64).round() as i64;
    let fps = (scene.fps_num as f64 / scene.fps_den as f64)
        .round()
        .max(1.0) as i64;
    let ff = frame.rem_euclid(fps);
    let total_s = frame.div_euclid(fps);
    let m = total_s / 60;
    let s = total_s % 60;
    if with_frames {
        format!("{m}:{s:02}:{ff:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// 帯とclipを平坦化した時の総clip数。
pub(super) fn clip_count(scene: &TimelineScene) -> usize {
    scene.bands.iter().map(|band| band.clips.len()).sum()
}

pub(super) fn rgb(v: u32) -> Color {
    Color::from_rgb((v >> 16) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
}

pub(super) fn argb(a: u8, v: u32) -> Color {
    Color::from_argb(
        a,
        (v >> 16) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}

pub(super) fn gray(v: u8) -> Color {
    Color::from_rgb(v, v, v)
}

pub(super) fn dim_bg(c: u32, t: f32) -> u32 {
    let m = |sh: u32| {
        let a = ((c >> sh) & 0xff) as f32;
        let b = ((DESKTOP >> sh) & 0xff) as f32;
        (a + (b - a) * t) as u32
    };
    (m(16) << 16) | (m(8) << 8) | m(0)
}

pub(super) fn fill(cv: &skia_safe::Canvas, r: Rect, c: Color) {
    let mut p = Paint::default();
    p.set_anti_alias(false);
    p.set_color(c);
    cv.draw_rect(r, &p);
}

thread_local! {
    static TIMELINE_TYPEFACE: skia_safe::Typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system typeface");
}

pub(super) fn tf() -> skia_safe::Typeface {
    TIMELINE_TYPEFACE.with(Clone::clone)
}

pub(super) fn text(cv: &skia_safe::Canvas, s: &str, x: f32, y: f32, sz: f32, c: Color) {
    let f = Font::new(tf(), sz);
    let mut p = Paint::default();
    p.set_anti_alias(true);
    p.set_color(c);
    cv.draw_str(s, (x, y), &f, &p);
}

pub(super) fn measure(s: &str, sz: f32) -> f32 {
    Font::new(tf(), sz).measure_str(s, None).0
}

pub(super) fn diamond(cv: &skia_safe::Canvas, cx: f32, cy: f32, d: f32, sel: bool) {
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

pub(super) fn glyph(
    cv: &skia_safe::Canvas,
    cx: f32,
    cy: f32,
    kind: &str,
    on_dark: bool,
    quiet: bool,
) {
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

pub(super) fn tog(
    cv: &skia_safe::Canvas,
    x: f32,
    cy: f32,
    l: &str,
    on: bool,
    mixed: bool,
    acc: u32,
) {
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
