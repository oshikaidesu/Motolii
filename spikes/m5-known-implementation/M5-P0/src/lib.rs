use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Draft,
    Final,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region {
    Finite {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RgbaImage {
    pub width: usize,
    pub height: usize,
    pub pixels: &'static [[f32; 4]],
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PostError {
    #[error("image dimensions do not match pixel storage")]
    InvalidImage,
    #[error("blur radius is too large for this fixture")]
    RadiusTooLarge,
    #[error("finite region exceeds image bounds")]
    InvalidRegion,
}

pub fn blur_linear(
    pixels: &[[f32; 4]],
    width: usize,
    height: usize,
    region: Region,
    radius: usize,
) -> Result<Vec<[f32; 4]>, PostError> {
    validate_image(pixels, width, height)?;
    if radius > 8 {
        return Err(PostError::RadiusTooLarge);
    }
    if radius == 0 {
        return Ok(pixels.to_vec());
    }
    let (x0, y0, x1, y1) = region_bounds(region, width, height)?;
    let mut output = pixels.to_vec();
    for y in y0..y1 {
        for x in x0..x1 {
            let mut sum = [0.0; 4];
            let mut count = 0.0;
            for sy in y.saturating_sub(radius)..=(y + radius).min(height - 1) {
                for sx in x.saturating_sub(radius)..=(x + radius).min(width - 1) {
                    let sample = pixels[sy * width + sx];
                    sum[0] += srgb_to_linear(sample[0]);
                    sum[1] += srgb_to_linear(sample[1]);
                    sum[2] += srgb_to_linear(sample[2]);
                    sum[3] += sample[3];
                    count += 1.0;
                }
            }
            output[y * width + x] = [
                linear_to_srgb(sum[0] / count),
                linear_to_srgb(sum[1] / count),
                linear_to_srgb(sum[2] / count),
                sum[3] / count,
            ];
        }
    }
    Ok(output)
}

pub fn evaluate_post(
    input: &[[f32; 4]],
    width: usize,
    height: usize,
    region: Region,
    quality: Quality,
    seed: u64,
) -> Result<Vec<[f32; 4]>, PostError> {
    let blurred = blur_linear(
        input,
        width,
        height,
        region,
        if quality == Quality::Draft { 1 } else { 2 },
    )?;
    let (x0, y0, x1, y1) = region_bounds(region, width, height)?;
    let mut output = blurred;
    for y in y0..y1 {
        for x in x0..x1 {
            let index = y * width + x;
            let mut color = output[index];
            color[0] = (color[0] * 1.04 + 0.01).clamp(0.0, 1.0);
            color[1] = (color[1] * 1.04 + 0.01).clamp(0.0, 1.0);
            color[2] = (color[2] * 1.04 + 0.01).clamp(0.0, 1.0);
            let grain = grain(seed, index as u64) - 0.5;
            let amplitude = if quality == Quality::Draft {
                0.01
            } else {
                0.02
            };
            color[0] = (color[0] + grain * amplitude).clamp(0.0, 1.0);
            color[1] = (color[1] + grain * amplitude).clamp(0.0, 1.0);
            color[2] = (color[2] + grain * amplitude).clamp(0.0, 1.0);
            output[index] = color;
        }
    }
    Ok(output)
}

fn validate_image(pixels: &[[f32; 4]], width: usize, height: usize) -> Result<(), PostError> {
    if width == 0 || height == 0 || pixels.len() != width * height {
        return Err(PostError::InvalidImage);
    }
    Ok(())
}

fn region_bounds(
    region: Region,
    width: usize,
    height: usize,
) -> Result<(usize, usize, usize, usize), PostError> {
    match region {
        Region::Unknown => Ok((0, 0, width, height)),
        Region::Finite {
            x,
            y,
            width: rw,
            height: rh,
        } if x <= width
            && y <= height
            && rw <= width.saturating_sub(x)
            && rh <= height.saturating_sub(y) =>
        {
            Ok((x, y, x + rw, y + rh))
        }
        Region::Finite { .. } => Err(PostError::InvalidRegion),
    }
}

fn srgb_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f32) -> f32 {
    let value = value.max(0.0);
    if value <= 0.0031308 {
        value * 12.92
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

fn grain(seed: u64, index: u64) -> f32 {
    let mut value = seed ^ index.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    let mixed = value ^ (value >> 31);
    (mixed as f32) / (u64::MAX as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Vec<[f32; 4]>, usize, usize) {
        let width = 5;
        let height = 5;
        let pixels = (0..width * height)
            .map(|index| {
                let value = index as f32 / 24.0;
                [value, 1.0 - value, value * 0.5, 1.0]
            })
            .collect();
        (pixels, width, height)
    }

    #[test]
    fn blur_roi_padding_matches_full_frame_inside_region() {
        let (pixels, width, height) = fixture();
        let full = blur_linear(&pixels, width, height, Region::Unknown, 1).unwrap();
        let roi = blur_linear(
            &pixels,
            width,
            height,
            Region::Finite {
                x: 1,
                y: 1,
                width: 3,
                height: 3,
            },
            1,
        )
        .unwrap();
        for y in 1..4 {
            for x in 1..4 {
                assert_eq!(full[y * width + x], roi[y * width + x]);
            }
        }
    }

    #[test]
    fn unknown_region_is_explicitly_full_frame_and_invalid_is_rejected() {
        let (pixels, width, height) = fixture();
        let unknown = blur_linear(&pixels, width, height, Region::Unknown, 0).unwrap();
        assert_eq!(unknown, pixels);
        assert_eq!(
            blur_linear(
                &pixels,
                width,
                height,
                Region::Finite {
                    x: 4,
                    y: 4,
                    width: 2,
                    height: 1
                },
                1
            ),
            Err(PostError::InvalidRegion)
        );
    }

    #[test]
    fn lgg_and_blur_use_linear_light_not_srgb_average() {
        let pixels = vec![[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]];
        let result = blur_linear(&pixels, 2, 1, Region::Unknown, 1).unwrap();
        assert!((result[0][0] - 0.735_356_9).abs() < 0.000_01);
        assert!((result[1][0] - 0.735_356_9).abs() < 0.000_01);
    }

    #[test]
    fn preview_and_export_share_evaluator_and_seed_is_stable() {
        let (pixels, width, height) = fixture();
        let preview =
            evaluate_post(&pixels, width, height, Region::Unknown, Quality::Draft, 17).unwrap();
        let preview_again =
            evaluate_post(&pixels, width, height, Region::Unknown, Quality::Draft, 17).unwrap();
        let final_result =
            evaluate_post(&pixels, width, height, Region::Unknown, Quality::Final, 17).unwrap();
        assert_eq!(preview, preview_again);
        assert_ne!(preview, final_result);
        assert_eq!(
            evaluate_post(&pixels, width, height, Region::Unknown, Quality::Draft, 18).unwrap(),
            evaluate_post(&pixels, width, height, Region::Unknown, Quality::Draft, 18).unwrap()
        );
    }
}
