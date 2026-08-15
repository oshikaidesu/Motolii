//! Timeline 1枚のraster。primitiveはdrawへ委譲し、ここは組み立てだけ。

use skia_safe::{surfaces, AlphaType, ColorType, ImageInfo, Paint, PaintStyle, Rect};

use super::draw::{
    argb, clip_count, diamond, dim_bg, fill, first_tick_secs, format_ruler_time, glyph, gray,
    measure, rgb, ruler_label_step_secs, text, tog,
};
use super::geometry::logical_height;
use super::layout::{
    scale_for, ACCENT, CONTRAST, DESKTOP, DIM, FILL_HANDLE, INBOX_W, LOC_H, ON_BAR, OVER_H, P,
    RAIL_W, ROW, RULER_H, RULER_MARK, SURFACE_BG, SURFACE_HI, SURFACE_LO, SURF_X, TIME_H, W,
};
use super::scene::TimelineScene;

/// timeline 1枚をRGBA8888 premulのbytesへ描く。
///
/// `playhead`は0..1で曲全体(0..song_bars)を走る。
/// `selected < 0`なら選択ringなし。非負は平坦化clip列のindex。
pub(super) fn draw_timeline(
    scene: &TimelineScene,
    bytes: &mut [u8],
    width: u32,
    height: u32,
    playhead: f64,
    selected: i32,
) {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let Some(mut surface) = surfaces::wrap_pixels(&info, bytes, Some(width as usize * 4), None)
    else {
        return;
    };
    let cv = surface.canvas();
    cv.clear(rgb(DESKTOP));

    let scale = scale_for(width);
    cv.save();
    cv.scale((scale, scale));

    // 論理座標系。以下は motolii_full.rs と同一。
    let h = logical_height(scene).max(height as f32 / scale);
    let sw = W - SURF_X - 6.0;
    let view_a = scene.view_a;
    let view_b = scene.view_b;
    let song_bars = scene.song_bars;
    let bx = |b: f32| SURF_X + (b - view_a) / (view_b - view_a) * sw;
    let ox = |b: f32| SURF_X + b / song_bars * sw;

    // 0 clip時に rem_euclid(·,1) で幽霊番号を作らない。
    let count = clip_count(scene);
    let selected = if selected < 0 || count == 0 {
        None
    } else {
        Some(selected.rem_euclid(count as i32) as usize)
    };

    fill(cv, Rect::from_ltrb(0.0, 0.0, SURF_X, h), rgb(SURFACE_BG));

    // ── Overview ──
    fill(cv, Rect::from_ltrb(SURF_X, 0.0, W, OVER_H), rgb(SURFACE_LO));
    text(cv, "overview", 8.0, 14.0, 9.0, rgb(DIM));
    for (i, band) in scene.bands.iter().enumerate() {
        let yy = 3.0 + i as f32 * 2.8;
        for c in &band.clips {
            fill(
                cv,
                Rect::from_ltrb(ox(c.a), yy, ox(c.b), yy + 2.2),
                argb(0xcc, P[c.slot]),
            );
        }
    }
    let (va, vb) = (ox(view_a), ox(view_b));
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

    // ── frame 格子 / timecode ruler ──
    let ry = OVER_H + 1.0;
    fill(
        cv,
        Rect::from_ltrb(0.0, ry, W, ry + RULER_H),
        rgb(SURFACE_HI),
    );
    let label_step = ruler_label_step_secs(scene, sw);
    let with_frames = label_step + 1e-6 < 1.0;
    let mut tick = first_tick_secs(view_a, label_step);
    for _ in 0..512 {
        if tick > view_b + 1e-3 {
            break;
        }
        let x = bx(tick);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H - 6.0, x + 1.0, ry + RULER_H),
            gray(0x6a),
        );
        text(
            cv,
            &format_ruler_time(tick, scene, with_frames),
            x + 3.0,
            ry + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
        tick += label_step;
        if !tick.is_finite() {
            break;
        }
    }
    // probe fixture の verse1–chorus 帯。Document 投影へは載せない。
    if !scene.real {
        fill(
            cv,
            Rect::from_ltrb(bx(8.0), ry, bx(24.0), ry + 3.0),
            rgb(RULER_MARK),
        );
    }

    // ── locator lane ──
    let ly = ry + RULER_H;
    fill(cv, Rect::from_ltrb(0.0, ly, W, ly + LOC_H), rgb(SURFACE_BG));
    for &(b, name) in &scene.locators {
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
    // probe dummyはfixture専用。realはhost layer以外をInboxへ描かない。
    if !scene.real {
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
    }
    let by0 = ly + LOC_H + 1.0;

    // ── bands ──
    let mut y = by0;
    let mut flat = 0usize;
    for (band_index, band) in scene.bands.iter().enumerate() {
        if scene.lane_preview_band == Some(band_index) {
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y, W, y + ROW - 1.0),
                argb(0x24, ACCENT),
            );
        }
        for b in view_a as i32..=view_b as i32 {
            let x = bx(b as f32);
            // realは秒格子。fixtureだけ旧4拍強調を残す。
            let major = scene.real || b % 4 == 0;
            fill(
                cv,
                Rect::from_ltrb(x, y, x + 1.0, y + ROW - 1.0),
                argb(if major { 0x54 } else { 0x14 }, 0x060606),
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
        for c in &band.clips {
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

        cv.save();
        cv.clip_rect(Rect::from_ltrb(SURF_X, y, W, y + ROW), None, false);
        for c in &band.clips {
            let is_selected = selected == Some(flat);
            flat += 1;
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
            let place = |wd: f32, x: &mut f32| -> Option<f32> {
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
            let nw = measure(&c.name, 8.5);
            let mut nx = r.left + 5.0;
            for (b, _, _, _) in &c.keys {
                if *b < c.a || *b > c.b {
                    continue;
                }
                let kx = bx(*b);
                if kx > nx - 8.0 && kx < nx + nw + 6.0 {
                    nx = kx + 9.0;
                }
            }
            if x - nx > nw * 0.55 {
                text(cv, &c.name, nx, cy + 3.2, 8.5, ink);
            }
            for (b, d, s, _) in &c.keys {
                if *b < c.a || *b > c.b {
                    continue;
                }
                diamond(cv, bx(*b), cy, *d, *s);
            }
            if is_selected {
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
        cv.restore();
        if scene.lane_preview_band == Some(band_index) {
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y, W, y + 1.0),
                argb(0xd0, ACCENT),
            );
            fill(
                cv,
                Rect::from_ltrb(INBOX_W, y + ROW - 2.0, W, y + ROW - 1.0),
                argb(0xd0, ACCENT),
            );
        }
        y += ROW;
    }

    // 空real: 一行ガイド(fixtureはbands非空のため描画不変)。
    if scene.real && scene.bands.is_empty() {
        let guide = "Create の □ Rectangle をダブルクリックで配置";
        let text_w = measure(guide, 9.0);
        let tx = SURF_X + ((sw - text_w) * 0.5).max(0.0);
        text(cv, guide, tx, y + ROW * 0.5 + 3.0, 9.0, rgb(DIM));
        y += ROW;
    }

    // ── 下段 timecode（上段の2倍粗さ、同じ frame 格子）──
    fill(cv, Rect::from_ltrb(0.0, y, W, y + TIME_H), rgb(SURFACE_BG));
    let coarse_step = label_step * 2.0;
    let mut tick = first_tick_secs(view_a, coarse_step);
    for _ in 0..512 {
        if tick > view_b + 1e-3 {
            break;
        }
        let x = bx(tick);
        fill(cv, Rect::from_ltrb(x, y, x + 1.0, y + 5.0), gray(0x6a));
        text(
            cv,
            &format_ruler_time(tick, scene, false),
            x + 3.0,
            y + 11.0,
            8.5,
            rgb(RULER_MARK),
        );
        tick += coarse_step;
        if !tick.is_finite() {
            break;
        }
    }
    if scene.real {
        text(
            cv,
            &format_ruler_time(song_bars, scene, false),
            W - 64.0,
            y + 11.0,
            8.5,
            rgb(DIM),
        );
    } else {
        text(cv, "3:12 total", W - 64.0, y + 11.0, 8.5, rgb(DIM));
    }

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

    // gesture中のsnap位置だけを表示し、release後のDocument投影には残さない。
    if let Some(guide) = scene
        .snap_guide
        .filter(|guide| *guide >= view_a && *guide <= view_b)
    {
        let x = bx(guide);
        fill(
            cv,
            Rect::from_ltrb(x, ry + RULER_H, x + 1.0, y),
            argb(0xd8, ACCENT),
        );
    }

    // ── playhead。曲基準0..1。表示範囲外は描かない ──
    let bar = (playhead.clamp(0.0, 1.0) as f32) * song_bars;
    if bar >= view_a && bar <= view_b {
        let px = bx(bar);
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
    }

    cv.restore();
}
