//! RN Timeline用のread-only Skia raster。Document意味とGPU ownershipは持たない。

use skia_safe::{
    surfaces, AlphaType, Color, ColorType, Font, FontMgr, FontStyle, ImageInfo, Paint, PaintStyle,
    PathBuilder, Rect,
};

use crate::rn_product_host::TimelineFrameBorrow;

const HEADER_HEIGHT: f32 = 22.0;
const RULER_HEIGHT: f32 = 18.0;
const RAIL_WIDTH: f32 = 58.0;
const MIN_ROW_HEIGHT: f32 = 20.0;

const BACKGROUND: Color = Color::from_rgb(42, 42, 42);
const SURFACE: Color = Color::from_rgb(54, 54, 54);
const RAISED: Color = Color::from_rgb(70, 70, 70);
const RULE: Color = Color::from_rgb(98, 98, 98);
const GRID: Color = Color::from_rgb(72, 72, 72);
const TEXT: Color = Color::from_rgb(218, 218, 218);
const SUBTEXT: Color = Color::from_rgb(154, 154, 154);
const BAR: Color = Color::from_rgb(150, 170, 219);
const BAR_SELECTED: Color = Color::from_rgb(255, 173, 86);
const KEY: Color = Color::from_rgb(232, 232, 232);
const PLAYHEAD: Color = Color::from_rgb(242, 77, 87);

pub(crate) struct TimelineSkiaRaster {
    pub(crate) pixels: Vec<u8>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) stats: TimelineSkiaStats,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TimelineSkiaStats {
    pub(crate) rows: usize,
    pub(crate) bars: usize,
    pub(crate) keys: usize,
}

pub(crate) fn raster_timeline(
    frame: &TimelineFrameBorrow,
    width: u32,
    height: u32,
) -> Option<TimelineSkiaRaster> {
    if width == 0 || height == 0 {
        return None;
    }
    let info = ImageInfo::new(
        (i32::try_from(width).ok()?, i32::try_from(height).ok()?),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let mut surface = surfaces::raster(&info, None, None)?;
    let canvas = surface.canvas();
    canvas.clear(BACKGROUND);

    let width = width as f32;
    let height = height as f32;
    let content_top = (HEADER_HEIGHT + RULER_HEIGHT).min(height);
    fill(
        canvas,
        Rect::from_xywh(0.0, 0.0, width, HEADER_HEIGHT.min(height)),
        RAISED,
    );
    fill(
        canvas,
        Rect::from_xywh(
            0.0,
            HEADER_HEIGHT.min(height),
            width,
            RULER_HEIGHT.min((height - HEADER_HEIGHT).max(0.0)),
        ),
        SURFACE,
    );
    line(canvas, 0.0, content_top, width, content_top, RULE, 1.0);
    line(
        canvas,
        RAIL_WIDTH.min(width),
        HEADER_HEIGHT,
        RAIL_WIDTH.min(width),
        height,
        RULE,
        1.0,
    );
    draw_text(canvas, "TIMELINE", 8.0, 15.0, 10.0, TEXT);

    let time_left = RAIL_WIDTH.min(width);
    let time_width = (width - time_left).max(1.0);
    let duration_seconds = frame.document.composition.duration.as_seconds_f64();
    for tick in 0..=8 {
        let x = time_left + time_width * tick as f32 / 8.0;
        line(
            canvas,
            x,
            HEADER_HEIGHT,
            x,
            height,
            if tick % 2 == 0 { RULE } else { GRID },
            1.0,
        );
        if tick % 2 == 0 {
            draw_text(
                canvas,
                &format!("{:.1}", duration_seconds * tick as f64 / 8.0),
                x + 3.0,
                HEADER_HEIGHT + 12.0,
                8.0,
                SUBTEXT,
            );
        }
    }

    let rows = frame
        .projection
        .bars()
        .iter()
        .map(|bar| bar.band as usize + 1)
        .max()
        .unwrap_or(1);
    let available = (height - content_top).max(MIN_ROW_HEIGHT);
    let row_height = (available / rows as f32).max(MIN_ROW_HEIGHT);
    for row in 0..rows {
        let y = content_top + row as f32 * row_height;
        if y >= height {
            break;
        }
        line(
            canvas,
            0.0,
            (y + row_height).min(height),
            width,
            (y + row_height).min(height),
            GRID,
            1.0,
        );
        draw_text(canvas, &(row + 1).to_string(), 8.0, y + 14.0, 8.0, SUBTEXT);
    }

    for bar in frame.projection.bars() {
        let x0 = time_left + time_width * bar.x_start as f32;
        let x1 = time_left + time_width * bar.x_end as f32;
        let y = content_top + bar.band as f32 * row_height + 3.0;
        let bottom = (y + row_height - 6.0).min(height - 1.0);
        if x1 <= x0 || bottom <= y || y >= height {
            continue;
        }
        let selected = frame.primary == Some(bar.layer);
        fill(
            canvas,
            Rect::from_ltrb(x0, y, x1.min(width), bottom),
            if selected { BAR_SELECTED } else { BAR },
        );
        outline(
            canvas,
            Rect::from_ltrb(x0, y, x1.min(width), bottom),
            RULE,
            if selected { 2.0 } else { 1.0 },
        );
        if let Some(name) = frame.document.layers.display_name(bar.layer) {
            draw_text(
                canvas,
                name,
                x0 + 5.0,
                y + 13.0,
                9.0,
                Color::from_rgb(24, 24, 24),
            );
        }
    }

    for key in frame.projection.keys() {
        let x = time_left + time_width * key.center_x as f32;
        let y = content_top + key.band as f32 * row_height + row_height * 0.5;
        if x >= time_left && x <= width && y >= content_top && y <= height {
            diamond(canvas, x, y, frame.primary == Some(key.layer));
        }
    }

    let playhead = if duration_seconds > 0.0 {
        (frame.playhead.as_seconds_f64() / duration_seconds).clamp(0.0, 1.0) as f32
    } else {
        0.0
    };
    let playhead_x = time_left + time_width * playhead;
    line(
        canvas,
        playhead_x,
        HEADER_HEIGHT,
        playhead_x,
        height,
        PLAYHEAD,
        1.5,
    );

    let pixmap = surface.peek_pixels()?;
    let pixels = pixmap.bytes()?.to_vec();
    Some(TimelineSkiaRaster {
        pixels,
        width: width as u32,
        height: height as u32,
        stats: TimelineSkiaStats {
            rows,
            bars: frame.projection.bars().len(),
            keys: frame.projection.keys().len(),
        },
    })
}

fn fill(canvas: &skia_safe::Canvas, rect: Rect, color: Color) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_color(color);
    canvas.draw_rect(rect, &paint);
}

fn line(canvas: &skia_safe::Canvas, x0: f32, y0: f32, x1: f32, y1: f32, color: Color, width: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_color(color);
    canvas.draw_line((x0, y0), (x1, y1), &paint);
}

fn outline(canvas: &skia_safe::Canvas, rect: Rect, color: Color, width: f32) {
    let mut paint = Paint::default();
    paint.set_anti_alias(false);
    paint.set_style(PaintStyle::Stroke);
    paint.set_stroke_width(width);
    paint.set_color(color);
    canvas.draw_rect(rect, &paint);
}

fn draw_text(canvas: &skia_safe::Canvas, text: &str, x: f32, y: f32, size: f32, color: Color) {
    let Some(typeface) = FontMgr::default().legacy_make_typeface(None, FontStyle::normal()) else {
        return;
    };
    let font = Font::new(typeface, size);
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(color);
    canvas.draw_str(text, (x, y), &font, &paint);
}

fn diamond(canvas: &skia_safe::Canvas, x: f32, y: f32, selected: bool) {
    let extent = if selected { 5.0 } else { 4.0 };
    let mut path = PathBuilder::new();
    path.move_to((x, y - extent));
    path.line_to((x + extent, y));
    path.line_to((x, y + extent));
    path.line_to((x - extent, y));
    path.close();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_color(if selected { BAR_SELECTED } else { KEY });
    canvas.draw_path(&path.detach(), &paint);
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use motolii_core::RationalTime;
    use motolii_doc::Document;

    use super::*;
    use crate::timeline_projection::{project_timeline, TimelineMetrics, TimelineViewport};

    #[test]
    fn rasterizes_the_revisioned_projection_without_owning_gpu_state() {
        let document = Arc::new(Document::new_current());
        let duration = document.composition.duration;
        let projection = project_timeline(
            document.as_ref(),
            &TimelineMetrics {
                band_height: 1.0,
                units_per_second: duration.as_seconds_f64().recip(),
                key_half_extent: 1.0,
            },
            &TimelineViewport {
                start: RationalTime::ZERO,
                end: duration,
            },
        )
        .expect("empty document has a valid timeline projection");
        let frame = TimelineFrameBorrow {
            revision: 7,
            projection_generation: 3,
            document,
            projection,
            primary: None,
            playhead: RationalTime::ZERO,
        };

        let raster = raster_timeline(&frame, 320, 120).expect("raster");

        assert_eq!((raster.width, raster.height), (320, 120));
        assert_eq!(raster.pixels.len(), 320 * 120 * 4);
        assert_eq!(
            raster.stats,
            TimelineSkiaStats {
                rows: 1,
                bars: 0,
                keys: 0
            }
        );
        assert!(raster
            .pixels
            .chunks_exact(4)
            .any(|pixel| pixel != [42, 42, 42, 255]));
    }
}
