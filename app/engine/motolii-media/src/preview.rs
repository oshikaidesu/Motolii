//! Source Preview 用の一枚デコード投影。
//!
//! `decode` の正本は YUV420p のまま維持する。Iced の画像 widget は RGBA を
//! 要求するため、この module だけが preview 表示境界で YUV を RGBA へ変換する。
//! 合成器・exporterへこの変換を漏らさないのが責任の境界である。

use motolii_core::{ColorSpace, CpuFrame, PixelFormat};

use crate::{read_frame_at, MediaInfo, Result};

/// Source Preview が表示する実フレーム。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewFrame {
    pub width: u32,
    pub height: u32,
    pub frame_index: i64,
    pub rgba: Vec<u8>,
}

/// 既存の `read_frame_at` で一枚を読み、Iced 境界向けに RGBA へ投影する。
pub fn read_preview_frame(
    path: impl AsRef<std::path::Path>,
    info: &MediaInfo,
    frame_index: i64,
) -> Result<PreviewFrame> {
    let frame = read_frame_at(path, info, frame_index)?;
    let rgba = yuv420p_to_rgba(&frame)?;
    Ok(PreviewFrame {
        width: frame.desc.width,
        height: frame.desc.height,
        frame_index,
        rgba,
    })
}

/// YUV420p の CPU frame を packed RGBA8 にする。
///
/// 変換係数は `FrameDesc.color_space` の probe 結果に従う。未知のフォーマットを
/// 黙って近似表示せず、呼び出し側へ明示的に返す。
pub fn yuv420p_to_rgba(frame: &CpuFrame) -> Result<Vec<u8>> {
    if frame.desc.format != PixelFormat::Yuv420p {
        return Err(crate::MediaError::Ffmpeg(format!(
            "source preview expects YUV420p, got {:?}",
            frame.desc.format
        )));
    }

    let width = frame.desc.width as usize;
    let height = frame.desc.height as usize;
    let luma_size = width * height;
    let chroma_size = (width / 2) * (height / 2);
    let expected = luma_size + chroma_size * 2;
    if frame.data.len() != expected {
        return Err(crate::MediaError::FrameSizeMismatch {
            expected,
            got: frame.data.len(),
        });
    }

    let (limited, kr, kb) = matrix_coefficients(frame.desc.color_space);
    let kg = 1.0 - kr - kb;
    let mut rgba = Vec::with_capacity(width * height * 4);
    let u_plane = &frame.data[luma_size..luma_size + chroma_size];
    let v_plane = &frame.data[luma_size + chroma_size..];

    for y in 0..height {
        for x in 0..width {
            let y_sample = frame.data[y * width + x] as f32;
            let u_sample = u_plane[(y / 2) * (width / 2) + x / 2] as f32;
            let v_sample = v_plane[(y / 2) * (width / 2) + x / 2] as f32;

            let (y_value, u_value, v_value) = if limited {
                ((y_sample - 16.0) / 219.0, (u_sample - 128.0) / 224.0, (v_sample - 128.0) / 224.0)
            } else {
                (y_sample / 255.0, (u_sample - 128.0) / 255.0, (v_sample - 128.0) / 255.0)
            };
            let r = y_value + 2.0 * (1.0 - kr) * v_value;
            let b = y_value + 2.0 * (1.0 - kb) * u_value;
            let g = y_value - 2.0 * kb * (1.0 - kb) / kg * u_value
                - 2.0 * kr * (1.0 - kr) / kg * v_value;
            rgba.extend_from_slice(&[
                to_u8(r),
                to_u8(g),
                to_u8(b),
                255,
            ]);
        }
    }
    Ok(rgba)
}

fn matrix_coefficients(color_space: ColorSpace) -> (bool, f32, f32) {
    match color_space {
        ColorSpace::Rec601Limited => (true, 0.299, 0.114),
        ColorSpace::Rec709Limited => (true, 0.2126, 0.0722),
        ColorSpace::Rec709Full => (false, 0.2126, 0.0722),
        ColorSpace::Srgb | ColorSpace::LinearRgb => (false, 0.2126, 0.0722),
    }
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use motolii_core::{FrameDesc, RationalTime};

    fn frame(y: u8, u: u8, v: u8) -> CpuFrame {
        let desc = FrameDesc::yuv(2, 2, PixelFormat::Yuv420p, ColorSpace::Rec709Limited);
        CpuFrame::new(desc, RationalTime::ZERO, vec![y, y, y, y, u, v])
    }

    #[test]
    fn limited_range_neutral_black_and_white_reach_rgba() {
        let black = yuv420p_to_rgba(&frame(16, 128, 128)).unwrap();
        assert_eq!(black, vec![0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255]);

        let white = yuv420p_to_rgba(&frame(235, 128, 128)).unwrap();
        assert!(white.chunks_exact(4).all(|pixel| pixel == [255, 255, 255, 255]));
    }

    #[test]
    fn non_yuv_frame_is_rejected_instead_of_approximated() {
        let desc = FrameDesc::packed(2, 2, PixelFormat::Rgba8Unorm, ColorSpace::Srgb, false);
        let frame = CpuFrame::new(desc, RationalTime::ZERO, vec![0; 16]);
        assert!(yuv420p_to_rgba(&frame).is_err());
    }
}
