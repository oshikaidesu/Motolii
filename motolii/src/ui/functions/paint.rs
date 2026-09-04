use anyrender::{PaintRef, PaintScene, Scene};
use peniko::kurbo::{Affine, Point, Rect, Size};
use peniko::{Color, Fill};

pub(crate) fn color(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub(crate) fn rgb([r, g, b]: [u8; 3]) -> Color {
    color(r, g, b)
}

pub(crate) fn fill_rect(scene: &mut Scene, rect: Rect, color: Color) {
    scene.fill(Fill::NonZero, Affine::IDENTITY, PaintRef::Solid(color), None, &rect);
}

pub(crate) fn diamond(scene: &mut Scene, center: Point, size: f64, color: Color) {
    let rect = Rect::from_center_size(center, Size::new(size, size));
    scene.fill(Fill::NonZero, Affine::rotate_about(std::f64::consts::FRAC_PI_4, center), PaintRef::Solid(color), None, &rect);
}
