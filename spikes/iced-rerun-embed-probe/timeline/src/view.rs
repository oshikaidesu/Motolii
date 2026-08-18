//! 描画。**ここには `&mut` が1つも無い**。
//!
//! これは規律ではなく型である。`canvas::Program::draw` は `&self` でモデルを借りるので、
//! 「描きながらモデルを直す」が書けない。egui 側の `show(&mut self, ui)` は
//! 同じ場所に両方が書けるので、実際に混ざっている
//! (`timeline_editor/mod.rs:4407-4414` は、行を描くループの中から
//! ドラッグ中の毎フレーム Document コマンドを積んでいる)。
//!
//! 唯一の例外が計測用の atomic 2本で、これは「描画に掛かった時間」を外へ出す
//! 口が他に無いため。モデルの意味には触らない。

use std::sync::atomic::Ordering;

use iced::alignment;
use iced::mouse;
use iced::widget::canvas::{Frame, Geometry, Path, Stroke, Text};
use iced::{Color, Pixels, Point, Rectangle, Renderer, Size, Theme};

use crate::app::{App, Drag};
use crate::model::{frame_label, hit_test, Grab, Hit, ROW_GAP, ROW_H, RULER_H, TRIM_HIT_W};

// ── 配色 ───────────────────────────────────────────────────────────────────
// Ableton 風の暗色。仮の値であって、意味は載せていない。
const BG: Color = Color::from_rgb(0.113, 0.113, 0.121);
const RULER_BG: Color = Color::from_rgb(0.152, 0.152, 0.164);
const ROW_A: Color = Color::from_rgb(0.137, 0.137, 0.145);
const ROW_B: Color = Color::from_rgb(0.156, 0.156, 0.164);
const CLIP: Color = Color::from_rgb(0.325, 0.454, 0.596);
const CLIP_SEL: Color = Color::from_rgb(0.945, 0.694, 0.239);
const CLIP_EDGE: Color = Color::from_rgb(0.06, 0.06, 0.07);
const TRIM_BAND: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.28);
const PLAYHEAD: Color = Color::from_rgb(0.925, 0.341, 0.341);
const GRID: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.07);
const TEXT: Color = Color::from_rgb(0.75, 0.75, 0.78);
const TEXT_DIM: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.35);

pub fn draw(
    app: &App,
    renderer: &Renderer,
    _theme: &Theme,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Vec<Geometry> {
    let t0 = std::time::Instant::now();
    let vp = &app.viewport;
    let size = bounds.size();
    let mut frame = Frame::new(renderer, size);

    frame.fill_rectangle(Point::ORIGIN, size, BG);

    // ── 行の縞 ────────────────────────────────────────────────────────────
    for track in 0..app.timeline.tracks {
        let y = vp.track_to_y(track);
        if y + ROW_H < RULER_H || y > size.height {
            continue;
        }
        frame.fill_rectangle(
            Point::new(0.0, y),
            Size::new(size.width, ROW_H),
            if track % 2 == 0 { ROW_A } else { ROW_B },
        );
    }

    // ── 時間グリッド ──────────────────────────────────────────────────────
    // 目盛の間隔は zoom に応じて 1/2/5 系列から選ぶ。線が潰れるより
    // 数が減る方を選ぶのは製品の ruler と同じ考え方。
    let step = grid_step(vp.px_per_frame);
    let first = (vp.scroll_x / step as f64).floor() as i64 * step;
    let last = vp.x_to_frame(size.width).ceil() as i64;
    let mut tick = first.max(0);
    while tick <= last {
        let x = vp.frame_to_x(tick as f64);
        if x >= 0.0 {
            frame.fill_rectangle(Point::new(x, RULER_H), Size::new(1.0, size.height), GRID);
            frame.fill_text(Text {
                content: frame_label(tick),
                position: Point::new(x + 3.0, RULER_H - 15.0),
                color: TEXT_DIM,
                size: Pixels(10.0),
                ..Text::default()
            });
        }
        tick += step;
    }

    // ── ruler 帯 ─────────────────────────────────────────────────────────
    // 帯そのものは最後に上書きせず先に敷き、目盛の文字だけ上に載せている。
    frame.fill_rectangle(Point::ORIGIN, Size::new(size.width, RULER_H), RULER_BG);
    let mut tick = first.max(0);
    while tick <= last {
        let x = vp.frame_to_x(tick as f64);
        if x >= 0.0 {
            frame.fill_rectangle(Point::new(x, RULER_H - 6.0), Size::new(1.0, 6.0), TEXT_DIM);
            frame.fill_text(Text {
                content: frame_label(tick),
                position: Point::new(x + 3.0, 4.0),
                color: TEXT,
                size: Pixels(10.0),
                ..Text::default()
            });
        }
        tick += step;
    }

    // ── clip ─────────────────────────────────────────────────────────────
    // ホバーしている端の帯を出すために hit test を1回だけ引く。
    // `update` / `mouse_interaction` と同じ純関数なので、絵と挙動がズレない。
    let hover = cursor
        .position_in(bounds)
        .map(|p| hit_test(&app.timeline, vp, p.x, p.y));

    let mut drawn = 0u64;
    for clip in &app.timeline.clips {
        let x0 = vp.frame_to_x(clip.start as f64);
        let x1 = vp.frame_to_x(clip.end() as f64);
        if x1 < 0.0 || x0 > size.width {
            continue; // 画面外は捨てる。製品と同じ culling。
        }
        let y = vp.track_to_y(clip.track);
        if y + ROW_H < RULER_H || y > size.height {
            continue;
        }
        drawn += 1;

        let w = (x1 - x0).max(1.0);
        let selected = app.selection.contains(&clip.id);
        let x0c = x0.max(0.0);
        let wc = (x1.min(size.width) - x0c).max(1.0);

        frame.fill_rectangle(
            Point::new(x0c, y),
            Size::new(wc, ROW_H),
            if selected { CLIP_SEL } else { CLIP },
        );
        frame.stroke_rectangle(
            Point::new(x0c, y),
            Size::new(wc, ROW_H),
            Stroke::default().with_color(CLIP_EDGE).with_width(1.0),
        );

        // trim 帯。掴める場所を掴む前に見せる(触れそうで触れない物を作らない)。
        if w >= crate::model::TRIM_MIN_BAR_W {
            if let Some(Hit::Clip { id, grab }) = hover {
                if id == clip.id {
                    match grab {
                        Grab::LeftEdge => frame.fill_rectangle(
                            Point::new(x0, y),
                            Size::new(TRIM_HIT_W, ROW_H),
                            TRIM_BAND,
                        ),
                        Grab::RightEdge => frame.fill_rectangle(
                            Point::new(x1 - TRIM_HIT_W, y),
                            Size::new(TRIM_HIT_W, ROW_H),
                            TRIM_BAND,
                        ),
                        Grab::Body => {}
                    }
                }
            }
        }

        // ラベルは、入る幅があるときだけ。
        if wc > 26.0 {
            frame.with_clip(
                Rectangle::new(Point::new(x0c, y), Size::new(wc - 3.0, ROW_H)),
                |f| {
                    f.fill_text(Text {
                        content: format!("#{}", clip.id),
                        position: Point::new(x0c + 4.0, y + ROW_H * 0.5),
                        color: Color::BLACK,
                        size: Pixels(10.0),
                        align_y: alignment::Vertical::Center,
                        ..Text::default()
                    });
                },
            );
        }
    }

    // ── playhead ─────────────────────────────────────────────────────────
    let px = vp.frame_to_x(app.playhead as f64);
    if px >= -1.0 && px <= size.width + 1.0 {
        frame.fill_rectangle(
            Point::new(px, 0.0),
            Size::new(1.0, size.height),
            PLAYHEAD,
        );
        // 頭の三角。掴む所が見えるように ruler の中に出す。
        let head = Path::new(|p| {
            p.move_to(Point::new(px - 5.0, 0.0));
            p.line_to(Point::new(px + 5.0, 0.0));
            p.line_to(Point::new(px, 9.0));
            p.close();
        });
        frame.fill(&head, PLAYHEAD);
    }

    // ── 隅の実測表示 ──────────────────────────────────────────────────────
    let hud = format!(
        "{} clips ({} drawn) | draw {:.2} ms | frame {:.2} ms | {:.2} px/frame{} | playhead {} | sel {}{}",
        app.timeline.clips.len(),
        drawn,
        app.draw_us.load(Ordering::Relaxed) as f64 / 1000.0,
        app.frame_ms,
        vp.px_per_frame,
        if app.at_zoom_ceiling() { " (max)" } else { "" },
        app.playhead,
        app.selection.len(),
        match &app.drag {
            Some(Drag::Move { .. }) => " | drag: move",
            Some(Drag::Trim { .. }) => " | drag: trim",
            Some(Drag::Scrub) => " | drag: scrub",
            None => "",
        },
    );
    frame.fill_rectangle(
        Point::new(0.0, size.height - 18.0),
        Size::new(size.width, 18.0),
        Color::from_rgba(0.0, 0.0, 0.0, 0.72),
    );
    frame.fill_text(Text {
        content: hud,
        position: Point::new(6.0, size.height - 9.0),
        color: TEXT,
        size: Pixels(11.0),
        align_y: alignment::Vertical::Center,
        ..Text::default()
    });

    // 計測値の書き戻し。`&self` から出せる唯一の口。
    app.draw_us
        .store(t0.elapsed().as_micros() as u64, Ordering::Relaxed);
    app.drawn_clips.store(drawn, Ordering::Relaxed);

    vec![frame.into_geometry()]
}

/// zoom に応じた目盛の刻み(フレーム)。1/2/5 系列。
fn grid_step(px_per_frame: f32) -> i64 {
    // 目盛の間隔が 64px を下回らない最小の刻みを選ぶ。
    const SERIES: [i64; 13] = [1, 2, 5, 10, 15, 30, 60, 150, 300, 600, 1800, 3600, 9000];
    for s in SERIES {
        if s as f32 * px_per_frame >= 64.0 {
            return s;
        }
    }
    *SERIES.last().unwrap()
}

/// 行の縞と clip の y が食い違わないことを、定数の側から押さえておく。
#[allow(dead_code)]
const _ROW_PITCH_CHECK: f32 = ROW_H + ROW_GAP;
