use crate::FrameDesc;

/// 正準空間の点(原点中央・Y-up・高さ=1.0)。パラメータにpx値を持たせない(F-1)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalPoint {
    pub x: f64,
    pub y: f64,
}

impl CanonicalPoint {
    pub const CENTER: Self = Self { x: 0.0, y: 0.0 };
}

/// 正準空間のサイズ(高さ=1.0基準)。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanonicalSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelSize {
    pub width: f64,
    pub height: f64,
}

/// 正準空間(原点中央・Y-up・高さ=1.0)からピクセル空間(Y-down)への変換。
///
/// px変換はレンダ直前のこの型に集約し、ノードパラメータにはpx値を持たせない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportTransform {
    width_px: u32,
    height_px: u32,
}

impl ViewportTransform {
    pub fn new(width_px: u32, height_px: u32) -> Self {
        assert!(width_px > 0 && height_px > 0, "viewport must be non-zero");
        Self {
            width_px,
            height_px,
        }
    }

    pub fn from_desc(desc: &FrameDesc) -> Self {
        Self::new(desc.width, desc.height)
    }

    pub fn point_to_px(self, p: CanonicalPoint) -> PixelPoint {
        let h = self.height_px as f64;
        PixelPoint {
            x: self.width_px as f64 * 0.5 + p.x * h,
            y: self.height_px as f64 * 0.5 - p.y * h,
        }
    }

    pub fn size_to_px(self, s: CanonicalSize) -> PixelSize {
        let h = self.height_px as f64;
        PixelSize {
            width: s.width * h,
            height: s.height * h,
        }
    }

    pub fn height_px(self) -> u32 {
        self.height_px
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_center_maps_to_pixel_center() {
        let tx = ViewportTransform::new(1920, 1080);
        assert_eq!(
            tx.point_to_px(CanonicalPoint::CENTER),
            PixelPoint { x: 960.0, y: 540.0 }
        );
    }

    #[test]
    fn canonical_uses_height_as_unit_and_y_up() {
        let tx = ViewportTransform::new(1920, 1080);
        assert_eq!(
            tx.point_to_px(CanonicalPoint { x: 0.5, y: 0.25 }),
            PixelPoint {
                x: 1500.0,
                y: 270.0
            }
        );
        assert_eq!(
            tx.size_to_px(CanonicalSize {
                width: 0.25,
                height: 0.5
            }),
            PixelSize {
                width: 270.0,
                height: 540.0
            }
        );
    }
}
