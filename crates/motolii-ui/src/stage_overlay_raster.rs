use motolii_core::{CanonicalPoint, PixelSize, ViewportTransform};
use motolii_doc::LayerId;
use skia_safe::{surfaces, AlphaType, Color, ColorType, ImageInfo, Paint, PaintStyle, PathBuilder};

use crate::stage_geometry_projection::{StageGeometryProjection, StageLayerProjection};

const OUTLINE_LOGICAL_WIDTH: f64 = 1.5;
// 暫定色。theme token の正本が定まるまで、この module の定数に閉じる。
const OUTLINE_COLOR: Color = Color::from_argb(255, 0, 200, 255);

pub struct StageOverlayRaster {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub fn raster_selection_outline(
    projection: &StageGeometryProjection,
    selected: Option<LayerId>,
    logical: PixelSize,
    scale_factor: f64,
) -> StageOverlayRaster {
    let width = (logical.width * scale_factor).round() as u32;
    let height = (logical.height * scale_factor).round() as u32;
    let Ok(viewport) = ViewportTransform::new(width, height) else {
        return transparent(0, 0);
    };
    let Some(selected) = selected else {
        return transparent(width, height);
    };

    let Some(StageLayerProjection::Available(geometry)) = projection
        .layers()
        .iter()
        .find(|(layer, _)| *layer == selected)
        .map(|(_, projection)| projection)
    else {
        return transparent(width, height);
    };

    let transform = geometry.camera_view * geometry.world;
    let center = geometry.local_rect.center;
    let size = geometry.local_rect.size;
    let corners = [
        (center.x - size.width * 0.5, center.y - size.height * 0.5),
        (center.x + size.width * 0.5, center.y - size.height * 0.5),
        (center.x + size.width * 0.5, center.y + size.height * 0.5),
        (center.x - size.width * 0.5, center.y + size.height * 0.5),
    ];
    let pixels = corners.map(|(x, y)| {
        let [x, y] = transform.transform_point(x, y);
        viewport.point_to_px(CanonicalPoint { x, y })
    });
    if pixels
        .iter()
        .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return transparent(width, height);
    }

    let image_info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Premul,
        None,
    );
    let Some(mut surface) = surfaces::raster(&image_info, None, None) else {
        return transparent(width, height);
    };
    surface.canvas().clear(Color::TRANSPARENT);

    let mut path = PathBuilder::new();
    path.move_to((pixels[0].x as f32, pixels[0].y as f32));
    for point in &pixels[1..] {
        path.line_to((point.x as f32, point.y as f32));
    }
    path.close();
    let path = path.detach();

    let mut paint = Paint::default();
    paint.set_style(PaintStyle::Stroke);
    paint.set_color(OUTLINE_COLOR);
    paint.set_stroke_width((OUTLINE_LOGICAL_WIDTH * scale_factor) as f32);
    surface.canvas().draw_path(&path, &paint);

    let Some(pixmap) = surface.peek_pixels() else {
        return transparent(width, height);
    };
    let Some(bytes) = pixmap.bytes() else {
        return transparent(width, height);
    };
    StageOverlayRaster {
        pixels: bytes.to_vec(),
        width,
        height,
    }
}

fn transparent(width: u32, height: u32) -> StageOverlayRaster {
    StageOverlayRaster {
        pixels: vec![0; width.saturating_mul(height).saturating_mul(4) as usize],
        width,
        height,
    }
}
