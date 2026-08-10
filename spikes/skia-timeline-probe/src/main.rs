use skia_safe::{Color, Font, FontMgr, FontStyle, Paint, PaintStyle, Point, Rect, surfaces};
use std::hint::black_box;
use std::time::{Duration, Instant};

const WIDTH: i32 = 2560;
const HEIGHT: i32 = 1440;
const TRACK_HEIGHT: f32 = 28.0;
const HEADER_WIDTH: f32 = 180.0;

#[derive(Clone, Copy)]
struct Clip {
    rect: Rect,
    color: Color,
    selected: bool,
    phase: f32,
}

fn make_clips(count: usize) -> Vec<Clip> {
    let visible_tracks = ((HEIGHT as f32 / TRACK_HEIGHT).ceil() as usize).max(1);
    let columns = count.div_ceil(visible_tracks);
    let usable_width = WIDTH as f32 - HEADER_WIDTH;
    let clip_width = (usable_width / columns as f32).max(3.0);

    (0..count)
        .map(|i| {
            let track = i % visible_tracks;
            let column = i / visible_tracks;
            let x = HEADER_WIDTH + column as f32 * clip_width;
            let y = track as f32 * TRACK_HEIGHT + 2.0;
            let hue = ((i * 37) % 180) as u8;
            Clip {
                rect: Rect::from_xywh(x, y, (clip_width - 1.0).max(2.0), TRACK_HEIGHT - 4.0),
                color: Color::from_argb(255, 45 + hue / 3, 75 + hue / 2, 105 + hue / 3),
                selected: i % 20 == 0,
                phase: (i % 17) as f32 * 0.37,
            }
        })
        .collect()
}

fn render_frame(canvas: &skia_safe::Canvas, clips: &[Clip], frame: usize, rich: bool, font: &Font) {
    canvas.clear(Color::from_rgb(21, 23, 27));

    let mut paint = Paint::default();
    paint.set_anti_alias(false);

    // Track rows and fixed header.
    for track in 0..=((HEIGHT as f32 / TRACK_HEIGHT) as usize) {
        let y = track as f32 * TRACK_HEIGHT;
        paint.set_color(if track % 2 == 0 {
            Color::from_rgb(30, 33, 38)
        } else {
            Color::from_rgb(34, 37, 43)
        });
        canvas.draw_rect(Rect::from_xywh(0.0, y, WIDTH as f32, TRACK_HEIGHT), &paint);
        paint.set_color(Color::from_rgb(53, 57, 64));
        canvas.draw_line(Point::new(0.0, y), Point::new(WIDTH as f32, y), &paint);
    }

    paint.set_color(Color::from_rgb(26, 28, 33));
    canvas.draw_rect(
        Rect::from_xywh(0.0, 0.0, HEADER_WIDTH, HEIGHT as f32),
        &paint,
    );

    let group_dx = (frame as f32 * 0.8) % 24.0;

    for (i, clip) in clips.iter().enumerate() {
        let dx = if clip.selected { group_dx } else { 0.0 };
        let rect = clip.rect.with_offset((dx, 0.0));

        paint.set_style(PaintStyle::Fill);
        paint.set_color(clip.color);
        canvas.draw_rect(rect, &paint);

        paint.set_style(PaintStyle::Stroke);
        paint.set_stroke_width(if clip.selected { 2.0 } else { 1.0 });
        paint.set_color(if clip.selected {
            Color::from_rgb(245, 198, 70)
        } else {
            Color::from_rgb(87, 104, 124)
        });
        canvas.draw_rect(rect, &paint);

        // Approximate a waveform only when the clip is wide enough to expose it.
        if rich || rect.width() >= 12.0 {
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(1.0);
            paint.set_color(Color::from_argb(190, 210, 222, 232));
            let mid = rect.center_y();
            let step = rect.width() / 7.0;
            let mut previous = Point::new(rect.left + 2.0, mid);
            for sample in 1..7 {
                let x = rect.left + sample as f32 * step;
                let amplitude = ((sample as f32 * 1.7 + clip.phase).sin()) * 7.0;
                let next = Point::new(x, mid + amplitude);
                canvas.draw_line(previous, next, &paint);
                previous = next;
            }
        }

        // Labels are deliberately limited by available pixels, as a real timeline would do.
        if rich || rect.width() >= 48.0 {
            paint.set_style(PaintStyle::Fill);
            paint.set_color(Color::WHITE);
            canvas.draw_str(
                format!("Clip {i}"),
                (rect.left + 4.0, rect.top + 14.0),
                font,
                &paint,
            );
        }
    }

    paint.set_style(PaintStyle::Fill);
    paint.set_color(Color::from_rgb(255, 83, 96));
    let playhead_x = HEADER_WIDTH + ((frame as f32 * 9.0) % (WIDTH as f32 - HEADER_WIDTH));
    canvas.draw_rect(Rect::from_xywh(playhead_x, 0.0, 2.0, HEIGHT as f32), &paint);
}

fn percentile(samples: &[Duration], pct: f64) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len() - 1) as f64 * pct).round() as usize;
    sorted[index]
}

fn run_case(count: usize, frames: usize, rich: bool) {
    let clips = make_clips(count);
    let mut surface = surfaces::raster_n32_premul((WIDTH, HEIGHT)).expect("raster surface");
    let typeface = FontMgr::default()
        .legacy_make_typeface(None, FontStyle::normal())
        .expect("system typeface");
    let font = Font::new(typeface, 12.0);

    for frame in 0..10 {
        render_frame(surface.canvas(), &clips, frame, rich, &font);
    }

    let mut samples = Vec::with_capacity(frames);
    let started = Instant::now();
    for frame in 0..frames {
        let tick = Instant::now();
        render_frame(surface.canvas(), &clips, frame, rich, &font);
        black_box(surface.canvas().peek_pixels());
        samples.push(tick.elapsed());
    }
    let elapsed = started.elapsed();
    let p50 = percentile(&samples, 0.50);
    let p95 = percentile(&samples, 0.95);
    let worst = *samples.iter().max().unwrap();

    println!(
        "mode={:<4} clips={count:>6} frames={frames:>3} p50={:>7.3}ms p95={:>7.3}ms worst={:>7.3}ms throughput={:>7.1}fps",
        if rich { "rich" } else { "LOD" },
        p50.as_secs_f64() * 1000.0,
        p95.as_secs_f64() * 1000.0,
        worst.as_secs_f64() * 1000.0,
        frames as f64 / elapsed.as_secs_f64(),
    );
}

fn main() {
    println!("Skia CPU timeline probe: {WIDTH}x{HEIGHT}, Apple/host raster backend");
    println!(
        "Workload: rows + clip fill/stroke + conditional waveform/text + selection drag + playhead"
    );
    for count in [50, 100, 500, 1_000, 5_000, 20_000] {
        run_case(count, 120, false);
    }
    for count in [50, 100, 500, 1_000, 5_000, 20_000] {
        run_case(count, if count == 20_000 { 40 } else { 120 }, true);
    }
}
