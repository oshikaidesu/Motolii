use crate::{CanonicalPoint, FrameDesc, PixelPoint};

/// コンポ全体で共有される planar orthographic ランタイムカメラ（D1k）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompCamera {
    center: CanonicalPoint,
    roll_radians: f64,
    height: f64,
    aspect_num: i64,
    aspect_den: i64,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum CompCameraError {
    #[error("camera center must be finite, got ({x}, {y})")]
    NonFiniteCenter { x: f64, y: f64 },
    #[error("camera roll must be finite, got {roll_radians}")]
    NonFiniteRoll { roll_radians: f64 },
    #[error("camera height must be finite, got {height}")]
    NonFiniteHeight { height: f64 },
    #[error("camera height must be positive, got {height}")]
    NonPositiveHeight { height: f64 },
    #[error("camera aspect numerator must be positive, got {aspect_num}")]
    NonPositiveAspectNum { aspect_num: i64 },
    #[error("camera aspect denominator must be positive, got {aspect_den}")]
    NonPositiveAspectDen { aspect_den: i64 },
    #[error("world point must be finite, got ({x}, {y})")]
    NonFiniteWorldPoint { x: f64, y: f64 },
    #[error("NDC point must be finite, got ({x}, {y})")]
    NonFiniteNdc { x: f64, y: f64 },
    #[error("pixel point must be finite, got ({x}, {y})")]
    NonFinitePixel { x: f64, y: f64 },
    #[error("frame width must be non-zero")]
    ZeroFrameWidth,
    #[error("frame height must be non-zero")]
    ZeroFrameHeight,
    #[error("frame {width}x{height} does not match camera aspect {aspect_num}/{aspect_den}")]
    AspectMismatch {
        width: u32,
        height: u32,
        aspect_num: i64,
        aspect_den: i64,
    },
}

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    a = a.unsigned_abs() as i64;
    b = b.unsigned_abs() as i64;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a.max(1)
}

impl CompCamera {
    pub fn try_new(
        center: CanonicalPoint,
        roll_radians: f64,
        height: f64,
        aspect_num: i64,
        aspect_den: i64,
    ) -> Result<Self, CompCameraError> {
        if !center.x.is_finite() || !center.y.is_finite() {
            return Err(CompCameraError::NonFiniteCenter {
                x: center.x,
                y: center.y,
            });
        }
        if !roll_radians.is_finite() {
            return Err(CompCameraError::NonFiniteRoll { roll_radians });
        }
        if !height.is_finite() {
            return Err(CompCameraError::NonFiniteHeight { height });
        }
        if height <= 0.0 {
            return Err(CompCameraError::NonPositiveHeight { height });
        }
        if aspect_num <= 0 {
            return Err(CompCameraError::NonPositiveAspectNum { aspect_num });
        }
        if aspect_den <= 0 {
            return Err(CompCameraError::NonPositiveAspectDen { aspect_den });
        }
        let g = gcd_i64(aspect_num, aspect_den);
        Ok(Self {
            center,
            roll_radians,
            height,
            aspect_num: aspect_num / g,
            aspect_den: aspect_den / g,
        })
    }

    pub fn center(self) -> CanonicalPoint {
        self.center
    }

    pub fn roll_radians(self) -> f64 {
        self.roll_radians
    }

    pub fn height(self) -> f64 {
        self.height
    }

    pub fn aspect_num(self) -> i64 {
        self.aspect_num
    }

    pub fn aspect_den(self) -> i64 {
        self.aspect_den
    }

    pub fn world_to_ndc(self, point: CanonicalPoint) -> Result<(f64, f64), CompCameraError> {
        if !point.x.is_finite() || !point.y.is_finite() {
            return Err(CompCameraError::NonFiniteWorldPoint {
                x: point.x,
                y: point.y,
            });
        }

        let dx = point.x - self.center.x;
        let dy = point.y - self.center.y;
        if !dx.is_finite() || !dy.is_finite() {
            return Err(CompCameraError::NonFiniteWorldPoint {
                x: point.x,
                y: point.y,
            });
        }

        let cos_r = self.roll_radians.cos();
        let sin_r = self.roll_radians.sin();
        if !cos_r.is_finite() || !sin_r.is_finite() {
            return Err(CompCameraError::NonFiniteNdc { x: 0.0, y: 0.0 });
        }

        // R(-roll_radians) * (point - center)
        let qx = cos_r * dx + sin_r * dy;
        let qy = -sin_r * dx + cos_r * dy;
        if !qx.is_finite() || !qy.is_finite() {
            return Err(CompCameraError::NonFiniteNdc { x: qx, y: qy });
        }

        let scaled_x = (qx / self.height) * 2.0;
        if !scaled_x.is_finite() {
            return Err(CompCameraError::NonFiniteNdc {
                x: scaled_x,
                y: 0.0,
            });
        }
        let aspect_num_f64 = self.aspect_num as f64;
        let aspect_den_f64 = self.aspect_den as f64;
        let ndc_x = (scaled_x / aspect_num_f64) * aspect_den_f64;
        let ndc_y = (qy / self.height) * 2.0;
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return Err(CompCameraError::NonFiniteNdc { x: ndc_x, y: ndc_y });
        }
        Ok((ndc_x, ndc_y))
    }

    pub fn ensure_matches_frame_desc(self, desc: &FrameDesc) -> Result<(), CompCameraError> {
        if desc.width == 0 {
            return Err(CompCameraError::ZeroFrameWidth);
        }
        if desc.height == 0 {
            return Err(CompCameraError::ZeroFrameHeight);
        }
        let w = desc.width as i128;
        let h = desc.height as i128;
        let an = self.aspect_num as i128;
        let ad = self.aspect_den as i128;
        if w * ad != h * an {
            return Err(CompCameraError::AspectMismatch {
                width: desc.width,
                height: desc.height,
                aspect_num: self.aspect_num,
                aspect_den: self.aspect_den,
            });
        }
        Ok(())
    }

    pub fn ndc_to_pixel(
        self,
        ndc_x: f64,
        ndc_y: f64,
        desc: &FrameDesc,
    ) -> Result<PixelPoint, CompCameraError> {
        if !ndc_x.is_finite() || !ndc_y.is_finite() {
            return Err(CompCameraError::NonFiniteNdc { x: ndc_x, y: ndc_y });
        }
        self.ensure_matches_frame_desc(desc)?;

        let w = desc.width as f64;
        let h = desc.height as f64;
        let pixel_x = (ndc_x + 1.0) * w / 2.0;
        let pixel_y = (1.0 - ndc_y) * h / 2.0;
        if !pixel_x.is_finite() || !pixel_y.is_finite() {
            return Err(CompCameraError::NonFinitePixel {
                x: pixel_x,
                y: pixel_y,
            });
        }
        Ok(PixelPoint {
            x: pixel_x,
            y: pixel_y,
        })
    }

    pub fn world_to_pixel(
        self,
        point: CanonicalPoint,
        desc: &FrameDesc,
    ) -> Result<PixelPoint, CompCameraError> {
        let (ndc_x, ndc_y) = self.world_to_ndc(point)?;
        self.ndc_to_pixel(ndc_x, ndc_y, desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ViewportTransform;

    fn identity_camera(aspect_num: i64, aspect_den: i64) -> CompCamera {
        CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1.0, aspect_num, aspect_den).unwrap()
    }

    #[test]
    fn try_new_reduces_aspect_by_gcd() {
        let camera = CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1.0, 16, 8).unwrap();
        assert_eq!(camera.aspect_num(), 2);
        assert_eq!(camera.aspect_den(), 1);
    }

    #[test]
    fn world_to_ndc_overflow_aware_evaluation_order() {
        let camera =
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, f64::MAX / 2.0, 4, 1).unwrap();
        let point = CanonicalPoint {
            x: f64::MAX / 4.0,
            y: 0.0,
        };
        let (ndc_x, ndc_y) = camera.world_to_ndc(point).unwrap();
        assert!((ndc_x - 0.25).abs() < 1e-10);
        assert_eq!(ndc_y, 0.0);
    }

    #[test]
    fn world_to_ndc_large_aspect_ratio_avoids_intermediate_overflow() {
        let aspect_num = i64::MAX;
        let aspect_den = i64::MAX - 2;
        let camera =
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1.0, aspect_num, aspect_den).unwrap();
        let point = CanonicalPoint {
            x: f64::MAX / 4.0,
            y: 0.0,
        };
        let (ndc_x, ndc_y) = camera.world_to_ndc(point).unwrap();
        assert!(
            ndc_x.is_finite(),
            "mathematically finite ndc_x must not be rejected"
        );
        let scaled_x = f64::MAX / 2.0;
        let expected_ndc_x = (scaled_x / (aspect_num as f64)) * (aspect_den as f64);
        assert!(
            (ndc_x - expected_ndc_x).abs() < expected_ndc_x.abs() * 1e-10,
            "ndc_x {ndc_x} should be approximately {expected_ndc_x}"
        );
        assert!(
            (ndc_x - f64::MAX / 2.0).abs() < f64::MAX / 2.0 * 1e-10,
            "ndc_x should be approximately f64::MAX/2"
        );
        assert_eq!(ndc_y, 0.0);
    }

    #[test]
    fn world_corners_map_to_ndc_and_y_down_raster_pixels() {
        let center = CanonicalPoint { x: 0.3, y: -0.1 };
        let roll = 0.25 * std::f64::consts::PI;
        let height = 2.0;
        let aspect_num = 16i64;
        let aspect_den = 9i64;
        let camera = CompCamera::try_new(center, roll, height, aspect_num, aspect_den).unwrap();
        let width = 16u32;
        let frame_height = 9u32;
        let desc = FrameDesc::packed(
            width,
            frame_height,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );

        let cos_r = roll.cos();
        let sin_r = roll.sin();
        let half_w = height * (aspect_num as f64) / (aspect_den as f64) / 2.0;
        let half_h = height / 2.0;

        let corners = [
            (
                CanonicalPoint {
                    x: center.x + cos_r * (-half_w) - sin_r * (-half_h),
                    y: center.y + sin_r * (-half_w) + cos_r * (-half_h),
                },
                (-1.0, -1.0),
                (0.0, f64::from(frame_height)),
            ),
            (
                CanonicalPoint {
                    x: center.x + cos_r * half_w - sin_r * (-half_h),
                    y: center.y + sin_r * half_w + cos_r * (-half_h),
                },
                (1.0, -1.0),
                (f64::from(width), f64::from(frame_height)),
            ),
            (
                CanonicalPoint {
                    x: center.x + cos_r * half_w - sin_r * half_h,
                    y: center.y + sin_r * half_w + cos_r * half_h,
                },
                (1.0, 1.0),
                (f64::from(width), 0.0),
            ),
            (
                CanonicalPoint {
                    x: center.x + cos_r * (-half_w) - sin_r * half_h,
                    y: center.y + sin_r * (-half_w) + cos_r * half_h,
                },
                (-1.0, 1.0),
                (0.0, 0.0),
            ),
        ];

        for (world, (expected_ndc_x, expected_ndc_y), (expected_px_x, expected_px_y)) in corners {
            let (ndc_x, ndc_y) = camera.world_to_ndc(world).unwrap();
            assert!(
                (ndc_x - expected_ndc_x).abs() < 1e-9 && (ndc_y - expected_ndc_y).abs() < 1e-9,
                "world {:?}: expected NDC ({expected_ndc_x}, {expected_ndc_y}), got ({ndc_x}, {ndc_y})",
                world
            );

            let pixel_from_ndc = camera
                .ndc_to_pixel(expected_ndc_x, expected_ndc_y, &desc)
                .unwrap();
            assert!(
                (pixel_from_ndc.x - expected_px_x).abs() < 1e-9
                    && (pixel_from_ndc.y - expected_px_y).abs() < 1e-9,
                "NDC ({expected_ndc_x}, {expected_ndc_y}): expected pixel ({expected_px_x}, {expected_px_y}), got ({}, {})",
                pixel_from_ndc.x,
                pixel_from_ndc.y
            );

            let pixel_from_world = camera.world_to_pixel(world, &desc).unwrap();
            assert!(
                (pixel_from_world.x - expected_px_x).abs() < 1e-9
                    && (pixel_from_world.y - expected_px_y).abs() < 1e-9,
                "world {:?}: expected pixel ({expected_px_x}, {expected_px_y}), got ({}, {})",
                world,
                pixel_from_world.x,
                pixel_from_world.y
            );
        }
    }

    #[test]
    fn identity_matches_viewport_transform() {
        let width = 16u32;
        let height = 9u32;
        let desc = FrameDesc::packed(
            width,
            height,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        let camera = identity_camera(i64::from(width), i64::from(height));
        let vt = ViewportTransform::from_desc(&desc).unwrap();

        let corners = [
            CanonicalPoint { x: -1.0, y: -0.5 },
            CanonicalPoint { x: 1.0, y: -0.5 },
            CanonicalPoint { x: 1.0, y: 0.5 },
            CanonicalPoint { x: -1.0, y: 0.5 },
            CanonicalPoint::CENTER,
        ];
        for point in corners {
            let expected = vt.point_to_px(point);
            let actual = camera.world_to_pixel(point, &desc).unwrap();
            assert!(
                (expected.x - actual.x).abs() < 1e-9 && (expected.y - actual.y).abs() < 1e-9,
                "point {:?}: expected {:?}, got {:?}",
                point,
                expected,
                actual
            );
        }
    }

    #[test]
    fn ndc_y_up_maps_to_pixel_y_down() {
        let desc = FrameDesc::packed(
            8,
            4,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        let camera = identity_camera(8, 4);
        let top = camera
            .world_to_pixel(CanonicalPoint { x: 0.0, y: 0.5 }, &desc)
            .unwrap();
        let bottom = camera
            .world_to_pixel(CanonicalPoint { x: 0.0, y: -0.5 }, &desc)
            .unwrap();
        assert!(top.y < bottom.y, "Y-up NDC must become Y-down pixels");
    }

    #[test]
    fn roll_rotates_world_to_ndc() {
        let camera = CompCamera::try_new(
            CanonicalPoint::CENTER,
            std::f64::consts::FRAC_PI_2,
            1.0,
            1,
            1,
        )
        .unwrap();
        let (ndc_x, ndc_y) = camera
            .world_to_ndc(CanonicalPoint { x: 1.0, y: 0.0 })
            .unwrap();
        assert!((ndc_x - 0.0).abs() < 1e-9);
        assert!((ndc_y - (-2.0)).abs() < 1e-9);
    }

    #[test]
    fn rejects_non_finite_inputs() {
        let camera = identity_camera(1, 1);
        assert!(matches!(
            CompCamera::try_new(
                CanonicalPoint {
                    x: f64::NAN,
                    y: 0.0
                },
                0.0,
                1.0,
                1,
                1
            ),
            Err(CompCameraError::NonFiniteCenter { .. })
        ));
        assert!(matches!(
            CompCamera::try_new(CanonicalPoint::CENTER, f64::NAN, 1.0, 1, 1),
            Err(CompCameraError::NonFiniteRoll { .. })
        ));
        assert!(matches!(
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, f64::NAN, 1, 1),
            Err(CompCameraError::NonFiniteHeight { .. })
        ));
        assert!(matches!(
            camera.world_to_ndc(CanonicalPoint {
                x: f64::INFINITY,
                y: 0.0
            }),
            Err(CompCameraError::NonFiniteWorldPoint { .. })
        ));
        assert!(matches!(
            camera.ndc_to_pixel(
                f64::NAN,
                0.0,
                &FrameDesc::packed(
                    1,
                    1,
                    crate::PixelFormat::Rgba8Unorm,
                    crate::ColorSpace::Srgb,
                    true,
                )
            ),
            Err(CompCameraError::NonFiniteNdc { .. })
        ));
    }

    #[test]
    fn rejects_mathematically_non_finite_ndc() {
        let camera = CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1e-308, 1, 1).unwrap();
        assert!(matches!(
            camera.world_to_ndc(CanonicalPoint { x: 0.0, y: 1.0 }),
            Err(CompCameraError::NonFiniteNdc { .. })
        ));
    }

    #[test]
    fn rejects_non_finite_pixel() {
        let desc = FrameDesc::packed(
            8,
            4,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        let camera = identity_camera(8, 4);
        assert!(matches!(
            camera.ndc_to_pixel(f64::MAX, 0.0, &desc),
            Err(CompCameraError::NonFinitePixel { .. })
        ));
    }

    #[test]
    fn rejects_non_positive_height_and_aspect() {
        assert!(matches!(
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 0.0, 1, 1),
            Err(CompCameraError::NonPositiveHeight { .. })
        ));
        assert!(matches!(
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1.0, 0, 1),
            Err(CompCameraError::NonPositiveAspectNum { .. })
        ));
        assert!(matches!(
            CompCamera::try_new(CanonicalPoint::CENTER, 0.0, 1.0, 1, 0),
            Err(CompCameraError::NonPositiveAspectDen { .. })
        ));
    }

    #[test]
    fn rejects_zero_frame_and_aspect_mismatch() {
        let camera = identity_camera(16, 9);
        let zero_w = FrameDesc::packed(
            0,
            9,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        let zero_h = FrameDesc::packed(
            16,
            0,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        let mismatch = FrameDesc::packed(
            16,
            8,
            crate::PixelFormat::Rgba8Unorm,
            crate::ColorSpace::Srgb,
            true,
        );
        assert!(matches!(
            camera.ensure_matches_frame_desc(&zero_w),
            Err(CompCameraError::ZeroFrameWidth)
        ));
        assert!(matches!(
            camera.ensure_matches_frame_desc(&zero_h),
            Err(CompCameraError::ZeroFrameHeight)
        ));
        assert!(matches!(
            camera.ensure_matches_frame_desc(&mismatch),
            Err(CompCameraError::AspectMismatch { .. })
        ));
    }
}
