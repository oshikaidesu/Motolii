use serde::{Deserialize, Serialize};

use crate::doc::core::RationalTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PixelFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Bgra8Unorm,
    Rgba16Float,
    Rgba32Float,
    Yuv420p,
    Nv12,
}

impl PixelFormat {
    pub fn bytes_per_pixel(&self) -> Option<u32> {
        match self {
            PixelFormat::Rgba8Unorm | PixelFormat::Rgba8UnormSrgb | PixelFormat::Bgra8Unorm => {
                Some(4)
            }
            PixelFormat::Rgba16Float => Some(8),
            PixelFormat::Rgba32Float => Some(16),
            PixelFormat::Yuv420p | PixelFormat::Nv12 => None,
        }
    }

    pub fn is_yuv(&self) -> bool {
        matches!(self, PixelFormat::Yuv420p | PixelFormat::Nv12)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ColorSpace {
    LinearRgb,
    Srgb,
    Rec709Limited,
    Rec709Full,
    Rec601Limited,
}

/// A frame's six meanings (frozen: freeze gate #1). Built by [`FrameDesc::try_packed`] / [`FrameDesc::try_yuv`];
/// a deserialized one is checked by [`FrameDesc::validate`] too, so no descriptor skips the invariants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "FrameDescFields")]
pub struct FrameDesc {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
    pub color_space: ColorSpace,
    pub premultiplied: bool,
}

/// The same six fields, unchecked: only what serde reads before [`FrameDesc::validate`] accepts it.
#[derive(Deserialize)]
struct FrameDescFields {
    width: u32,
    height: u32,
    stride: u32,
    format: PixelFormat,
    color_space: ColorSpace,
    premultiplied: bool,
}

impl TryFrom<FrameDescFields> for FrameDesc {
    type Error = FrameDescError;
    fn try_from(f: FrameDescFields) -> Result<Self, FrameDescError> {
        let desc = FrameDesc {
            width: f.width,
            height: f.height,
            stride: f.stride,
            format: f.format,
            color_space: f.color_space,
            premultiplied: f.premultiplied,
        };
        desc.validate()?;
        Ok(desc)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FrameDescError {
    #[error("FrameDesc: packed format required")]
    PackedFormatRequired,
    #[error("FrameDesc: yuv format required")]
    YuvFormatRequired,
    #[error("FrameDesc: 4:2:0 requires even dimensions")]
    OddDimensions,
    #[error("FrameDesc: zero dimension")]
    ZeroDimension,
    #[error("FrameDesc: width {width} * {bytes_per_pixel} bytes overflows the stride")]
    StrideOverflow { width: u32, bytes_per_pixel: u32 },
    #[error("FrameDesc: stride {stride} < width {width} * {bytes_per_pixel} bytes")]
    StrideTooSmall {
        stride: u32,
        width: u32,
        bytes_per_pixel: u32,
    },
    #[error("FrameDesc: 4:2:0 stride {stride} < width {width}")]
    YuvStrideTooSmall { stride: u32, width: u32 },
}

impl FrameDesc {
    /// A packed frame with the tightest stride. Rejects a YUV format, a zero dimension, and a row that overflows.
    pub fn try_packed(
        width: u32,
        height: u32,
        format: PixelFormat,
        color_space: ColorSpace,
        premultiplied: bool,
    ) -> Result<Self, FrameDescError> {
        let bpp = format
            .bytes_per_pixel()
            .ok_or(FrameDescError::PackedFormatRequired)?;
        if width == 0 || height == 0 {
            return Err(FrameDescError::ZeroDimension);
        }
        let stride = width
            .checked_mul(bpp)
            .ok_or(FrameDescError::StrideOverflow {
                width,
                bytes_per_pixel: bpp,
            })?;
        Ok(Self {
            width,
            height,
            stride,
            format,
            color_space,
            premultiplied,
        })
    }

    /// A 4:2:0 frame (luma stride = width). Rejects a packed format, a zero or odd dimension.
    pub fn try_yuv(
        width: u32,
        height: u32,
        format: PixelFormat,
        color_space: ColorSpace,
    ) -> Result<Self, FrameDescError> {
        if !format.is_yuv() {
            return Err(FrameDescError::YuvFormatRequired);
        }
        if width == 0 || height == 0 {
            return Err(FrameDescError::ZeroDimension);
        }
        if !width.is_multiple_of(2) || !height.is_multiple_of(2) {
            return Err(FrameDescError::OddDimensions);
        }
        Ok(Self {
            width,
            height,
            stride: width,
            format,
            color_space,
            premultiplied: false,
        })
    }

    pub fn data_size(&self) -> usize {
        let w = self.width as usize;
        let h = self.height as usize;
        match self.format {
            PixelFormat::Yuv420p | PixelFormat::Nv12 => w * h + 2 * (w / 2) * (h / 2),
            _ => self.stride as usize * h,
        }
    }

    /// The invariants every descriptor keeps, however it was made (built, deserialized, or edited field by field).
    pub fn validate(&self) -> Result<(), FrameDescError> {
        if self.width == 0 || self.height == 0 {
            return Err(FrameDescError::ZeroDimension);
        }
        if let Some(bpp) = self.format.bytes_per_pixel() {
            let row = self
                .width
                .checked_mul(bpp)
                .ok_or(FrameDescError::StrideOverflow {
                    width: self.width,
                    bytes_per_pixel: bpp,
                })?;
            if self.stride < row {
                return Err(FrameDescError::StrideTooSmall {
                    stride: self.stride,
                    width: self.width,
                    bytes_per_pixel: bpp,
                });
            }
        } else {
            if !self.width.is_multiple_of(2) || !self.height.is_multiple_of(2) {
                return Err(FrameDescError::OddDimensions);
            }
            if self.stride < self.width {
                return Err(FrameDescError::YuvStrideTooSmall {
                    stride: self.stride,
                    width: self.width,
                });
            }
        }
        Ok(())
    }

    pub fn same_aspect_integer_scale(self, other: Self) -> bool {
        if self.width == 0 || self.height == 0 || other.width == 0 || other.height == 0 {
            return false;
        }
        if u64::from(self.width) * u64::from(other.height)
            != u64::from(self.height) * u64::from(other.width)
        {
            return false;
        }
        let (large_w, large_h, small_w, small_h) = if self.width >= other.width {
            (self.width, self.height, other.width, other.height)
        } else {
            (other.width, other.height, self.width, self.height)
        };
        if large_w % small_w != 0 || large_h % small_h != 0 {
            return false;
        }
        large_w / small_w == large_h / small_h
    }
}

#[derive(Debug, Clone)]
pub struct CpuFrame {
    pub desc: FrameDesc,
    pub pts: RationalTime,
    pub data: Vec<u8>,
}

impl CpuFrame {
    pub fn new(desc: FrameDesc, pts: RationalTime, data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), desc.data_size());
        Self { desc, pts, data }
    }
}

pub fn premultiply_rgba_f32(mut rgba: [f32; 4]) -> [f32; 4] {
    rgba[0] *= rgba[3];
    rgba[1] *= rgba[3];
    rgba[2] *= rgba[3];
    rgba
}

pub fn premultiply_rgba_u8(rgba: [u8; 4]) -> [u8; 4] {
    let a = rgba[3] as u16;
    [
        ((rgba[0] as u16 * a + 127) / 255) as u8,
        ((rgba[1] as u16 * a + 127) / 255) as u8,
        ((rgba[2] as u16 * a + 127) / 255) as u8,
        rgba[3],
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompSpec {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LayerPlacement {
    pub transform: glam::Affine2,
    pub world_transform: Option<glam::Affine3A>,
    pub order: i32,
    pub opacity: f32,
    pub z: f32,
    pub rotation_x: f32,
    pub rotation_y: f32,
    /// 並べる Group の面に乗っているなら、その面の基準点(world)。同じ面の物は描き順の距離をここで測り、
    /// 積み順で重なる(3D でも深度で奪い合わない、箱の奥行きの法 3)。
    pub plane: Option<[f32; 3]>,
}

impl Default for LayerPlacement {
    fn default() -> Self {
        Self {
            transform: glam::Affine2::IDENTITY,
            world_transform: None,
            order: 0,
            opacity: 1.0,
            z: 0.0,
            rotation_x: 0.0,
            rotation_y: 0.0,
            plane: None,
        }
    }
}

/// 大きさは 2 成分(x, y)しか持たない。奥行きを持つ素材(網・点群)にはその平均を掛け、
/// 球は球のまま拡縮する。板は奥行きが無いので掛からない(Lottie/web の定規と同じ変換のまま)。
pub fn depth_scale(x_axis: glam::Vec3, y_axis: glam::Vec3) -> f32 {
    (x_axis.length() + y_axis.length()) * 0.5
}

/// A layer's world transform as its volume is drawn: depth follows the mean of the x and y
/// scales. The cage and the drag map take their corners through this same transform.
pub fn depth_scaled(world: glam::Affine3A) -> glam::Affine3A {
    let depth = depth_scale(world.matrix3.x_axis.into(), world.matrix3.y_axis.into());
    world * glam::Affine3A::from_scale(glam::vec3(1.0, 1.0, depth))
}

impl LayerPlacement {
    pub fn from_transform(
        anchor: [f32; 2],
        position: [f32; 2],
        scale: [f32; 2],
        rotation_degrees: f32,
        skew_degrees: f32,
        skew_axis_degrees: f32,
    ) -> glam::Affine2 {
        use glam::{Affine2, Mat2, Vec2};

        let skew_matrix = if skew_degrees == 0.0 {
            Affine2::IDENTITY
        } else {
            let skew = skew_degrees.to_radians();
            let axis = skew_axis_degrees.to_radians();
            let shear = Affine2::from_mat2(Mat2::from_cols(
                Vec2::new(1.0, 0.0),
                Vec2::new(-skew.tan(), 1.0),
            ));
            Affine2::from_angle(-axis) * shear * Affine2::from_angle(axis)
        };

        Affine2::from_translation(Vec2::new(position[0], position[1]))
            * Affine2::from_angle(rotation_degrees.to_radians())
            * skew_matrix
            * Affine2::from_scale(Vec2::new(scale[0], scale[1]))
            * Affine2::from_translation(Vec2::new(-anchor[0], -anchor[1]))
    }

    /// The layer's local 3D transform: tilt about Position, then the planar transform.
    pub fn spatial_from_transform(
        xy: glam::Affine2,
        position: [f32; 2],
        z: f32,
        rotation_x_degrees: f32,
        rotation_y_degrees: f32,
        scale_z: f32,
    ) -> glam::Affine3A {
        use glam::{Affine3A, Mat3, Quat, Vec3};
        let linear = Mat3::from_cols(
            xy.matrix2.x_axis.extend(0.0),
            xy.matrix2.y_axis.extend(0.0),
            Vec3::Z * scale_z,
        );
        let anchored = Affine3A::from_mat3_translation(
            linear,
            xy.translation.extend(0.0) - Vec3::new(position[0], position[1], 0.0),
        );
        Affine3A::from_translation(Vec3::new(position[0], position[1], z))
            * Affine3A::from_quat(
                Quat::from_rotation_x(rotation_x_degrees.to_radians())
                    * Quat::from_rotation_y(rotation_y_degrees.to_radians()),
            )
            * anchored
    }
}

#[cfg(test)]
mod frame_desc_tests {
    use super::*;

    const RGBA: PixelFormat = PixelFormat::Rgba8Unorm;
    const SRGB: ColorSpace = ColorSpace::Srgb;

    #[test]
    fn packed_rejects_what_the_invariants_forbid() {
        assert_eq!(FrameDesc::try_packed(0, 4, RGBA, SRGB, false), Err(FrameDescError::ZeroDimension));
        assert_eq!(
            FrameDesc::try_packed(u32::MAX / 2, 4, RGBA, SRGB, false),
            Err(FrameDescError::StrideOverflow { width: u32::MAX / 2, bytes_per_pixel: 4 })
        );
        assert_eq!(
            FrameDesc::try_packed(4, 4, PixelFormat::Nv12, SRGB, false),
            Err(FrameDescError::PackedFormatRequired)
        );
        let d = FrameDesc::try_packed(3, 2, PixelFormat::Rgba16Float, SRGB, true).unwrap();
        assert_eq!((d.stride, d.data_size()), (24, 48));
    }

    #[test]
    fn yuv_rejects_odd_zero_and_packed() {
        assert_eq!(FrameDesc::try_yuv(3, 4, PixelFormat::Yuv420p, SRGB), Err(FrameDescError::OddDimensions));
        assert_eq!(FrameDesc::try_yuv(0, 4, PixelFormat::Yuv420p, SRGB), Err(FrameDescError::ZeroDimension));
        assert_eq!(FrameDesc::try_yuv(4, 4, RGBA, SRGB), Err(FrameDescError::YuvFormatRequired));
        assert_eq!(FrameDesc::try_yuv(4, 2, PixelFormat::Nv12, SRGB).unwrap().data_size(), 12);
    }

    #[test]
    fn a_field_edited_descriptor_is_caught_by_validate() {
        let mut d = FrameDesc::try_packed(8, 8, RGBA, SRGB, false).unwrap();
        d.stride = 16;
        assert_eq!(
            d.validate(),
            Err(FrameDescError::StrideTooSmall { stride: 16, width: 8, bytes_per_pixel: 4 })
        );
        d.width = u32::MAX;
        assert!(matches!(d.validate(), Err(FrameDescError::StrideOverflow { .. })));
        let mut y = FrameDesc::try_yuv(4, 4, PixelFormat::Yuv420p, SRGB).unwrap();
        y.stride = 2;
        assert_eq!(y.validate(), Err(FrameDescError::YuvStrideTooSmall { stride: 2, width: 4 }));
    }

    #[test]
    fn serde_roundtrips_a_valid_descriptor_and_refuses_a_bypass() {
        let d = FrameDesc::try_packed(1920, 1080, RGBA, SRGB, true).unwrap();
        let text = serde_json::to_string(&d).unwrap();
        assert_eq!(serde_json::from_str::<FrameDesc>(&text).unwrap(), d);
        for bad in [
            r#"{"width":0,"height":2,"stride":0,"format":"Rgba8Unorm","color_space":"Srgb","premultiplied":false}"#,
            r#"{"width":8,"height":2,"stride":4,"format":"Rgba8Unorm","color_space":"Srgb","premultiplied":false}"#,
            r#"{"width":3,"height":2,"stride":3,"format":"Yuv420p","color_space":"Srgb","premultiplied":false}"#,
        ] {
            assert!(serde_json::from_str::<FrameDesc>(bad).is_err(), "{bad}");
        }
    }
}
