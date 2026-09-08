use std::sync::Arc;

use re_renderer::resource_managers::{GpuTexture2D, ImageDataDesc};

use crate::render::compositor::{Compositor, CompositorError};

/// GPU に上げた環境(空)。放射輝度は背景と映り込み用(mip 付き、粗さで段を読む)、照度は網の拡散照明用
/// (上流 `Environment` の契約)。
pub struct GpuEnvironmentData {
    pub(crate) environment: re_renderer::Environment,
    /// 元の画の寸法。層の枠に使う。
    pub size: [f32; 2],
}

/// 照度図の寸法。拡散照明は低周波なので小さくてよい(cosine lobe は 2 次の球面調和で
/// ほぼ表せる: Ramamoorthi & Hanrahan 2001)。
const IRRADIANCE_SIZE: (usize, usize) = (32, 16);
/// 照度の畳み込みの入力を先にここまで縮める。総当たりなので 64×32 × 32×16 = 100 万回で済ませる。
const CONVOLVE_INPUT_SIZE: (usize, usize) = (64, 32);
/// 背景と映り込みに使う放射輝度の上限幅。4096×2048 の Rgba16Float + mip で約 85MB。
/// 2048 では 55° の視野に約 490 px しか入らず、1920 の画面へ 4 倍に伸びていた(2026-09-08)。
const MAX_RADIANCE_WIDTH: usize = 4096;

impl Compositor {
    /// 線形 RGB の等距円筒図(`width * height * 3`)を環境として上げる。
    pub(crate) fn upload_environment(
        &self,
        label: &str,
        rgb: &[f32],
        width: u32,
        height: u32,
    ) -> Result<Arc<GpuEnvironmentData>, CompositorError> {
        let small = downsample_rgb(rgb, width as usize, height as usize, CONVOLVE_INPUT_SIZE);
        let irradiance = re_renderer::environment::convolve_irradiance(
            &small,
            CONVOLVE_INPUT_SIZE.0,
            CONVOLVE_INPUT_SIZE.1,
            IRRADIANCE_SIZE.0,
            IRRADIANCE_SIZE.1,
        );
        let (radiance_rgb, radiance_w, radiance_h) = if width as usize > MAX_RADIANCE_WIDTH {
            let w = MAX_RADIANCE_WIDTH;
            let h = (height as usize * w / width as usize).max(1);
            (downsample_rgb(rgb, width as usize, height as usize, (w, h)), w as u32, h as u32)
        } else {
            (rgb.to_vec(), width, height)
        };
        let radiance = self.upload_rgb_f16_mipmapped(&format!("{label} radiance"), &radiance_rgb, radiance_w, radiance_h)?;
        let irradiance = self.upload_rgb_f16(
            &format!("{label} irradiance"),
            &irradiance,
            IRRADIANCE_SIZE.0 as u32,
            IRRADIANCE_SIZE.1 as u32,
        )?;
        Ok(Arc::new(GpuEnvironmentData {
            environment: re_renderer::Environment {
                radiance,
                irradiance,
                // 世界は x 右・y 下・z 奥(camera.rs の base 回転)。等距円筒図は y 上・-z 正面なので、
                // X 軸まわり 180° で写す(鏡像にしない)。
                environment_from_world: glam::Mat3::from_diagonal(glam::vec3(1.0, -1.0, -1.0)),
                strength: 1.0,
            },
            size: [width as f32, height as f32],
        }))
    }

    fn upload_rgb_f16(&self, label: &str, rgb: &[f32], width: u32, height: u32) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .create(&self.ctx, rgb_f16_desc(label, rgb, width, height))
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// mip 付き(GPU で生成)。映り込みは粗さでこの段を読む。
    fn upload_rgb_f16_mipmapped(&self, label: &str, rgb: &[f32], width: u32, height: u32) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .create_with_mipmaps(&self.ctx, rgb_f16_desc(label, rgb, width, height))
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }
}

fn rgb_f16_desc(label: &str, rgb: &[f32], width: u32, height: u32) -> ImageDataDesc<'static> {
    let mut bytes = Vec::with_capacity(rgb.len() / 3 * 8);
    for px in rgb.chunks_exact(3) {
        for v in [px[0], px[1], px[2], 1.0] {
            bytes.extend_from_slice(&half::f16::from_f32(v).to_le_bytes());
        }
    }
    ImageDataDesc {
        label: label.to_owned().into(),
        data: bytes.into(),
        format: wgpu::TextureFormat::Rgba16Float.into(),
        width_height: [width, height],
        alpha_channel_usage: re_renderer::AlphaChannelUsage::Opaque,
    }
}

/// 箱平均で縮める。出力より小さい入力はそのまま伸びる(平均する箱が 1 画素)。
fn downsample_rgb(rgb: &[f32], width: usize, height: usize, (out_w, out_h): (usize, usize)) -> Vec<f32> {
    let mut out = Vec::with_capacity(out_w * out_h * 3);
    for oy in 0..out_h {
        let y0 = oy * height / out_h;
        let y1 = ((oy + 1) * height / out_h).max(y0 + 1).min(height);
        for ox in 0..out_w {
            let x0 = ox * width / out_w;
            let x1 = ((ox + 1) * width / out_w).max(x0 + 1).min(width);
            let mut sum = [0.0f32; 3];
            let mut n = 0.0f32;
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * width + x) * 3;
                    sum[0] += rgb[i];
                    sum[1] += rgb[i + 1];
                    sum[2] += rgb[i + 2];
                    n += 1.0;
                }
            }
            out.extend_from_slice(&[sum[0] / n, sum[1] / n, sum[2] / n]);
        }
    }
    out
}
