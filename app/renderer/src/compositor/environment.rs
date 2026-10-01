use std::sync::Arc;

use re_renderer::resource_managers::{GpuTexture2D, ImageDataDesc};

use crate::render::compositor::{Compositor, CompositorError};

/// GPU に上げた環境(空)。放射輝度は背景と映り込み用(mip 付き、粗さで段を読む)、照度は網の拡散照明用
/// (上流 `Environment` の契約)。
pub struct GpuEnvironmentData {
    pub(crate) environment: re_renderer::Environment,
    /// 元の画の寸法。層の枠に使う。
    pub size: [f32; 2],
    /// この空の太陽。光源は置かない — 影が奪うのはこの分だけ(裁定 2026-09-10)。
    pub sun: SunSpec,
}

/// 太陽 = 環境の一番明るい方向。`weight` は拡散光のうち太陽の錐(峰の半分以上の明るさ)から来る割合。
#[derive(Clone, Copy, Debug)]
pub struct SunSpec {
    /// 世界の向き(太陽の方へ)。
    pub direction: glam::Vec3,
    pub weight: f32,
    /// 太陽の色(最大成分で正規化)。
    pub color: glam::Vec3,
}

impl SunSpec {
    /// 環境の無い作品: fork の固定 2 灯の主灯(`simple_lighting`)。
    pub fn fixed_lights() -> Self {
        Self { direction: glam::vec3(1.0, 2.0, 3.0).normalize(), weight: 1.0 / 1.7, color: glam::Vec3::ONE }
    }
}

/// 等距円筒図から太陽を読む: 峰の texel の向きと、峰の半分以上の texel が担う cosine 加重エネルギーの割合。
fn sun_from_equirect(rgb: &[f32], width: usize, height: usize, environment_from_world: glam::Mat3) -> SunSpec {
    let lum = |p: &[f32]| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
    let (mut peak, mut peak_index) = (0.0f32, 0usize);
    for (i, px) in rgb.chunks_exact(3).enumerate() {
        let l = lum(px);
        if l > peak {
            peak = l;
            peak_index = i;
        }
    }
    let direction_of = |i: usize| {
        re_renderer::environment::direction_from_equirect_uv(glam::vec2(
            ((i % width) as f32 + 0.5) / width as f32,
            ((i / width) as f32 + 0.5) / height as f32,
        ))
    };
    if peak <= 0.0 {
        return SunSpec { direction: glam::Vec3::ZERO, weight: 0.0, color: glam::Vec3::ONE };
    }
    let sun = direction_of(peak_index);
    let (mut total, mut from_sun, mut color) = (0.0f32, 0.0f32, glam::Vec3::ZERO);
    for (i, px) in rgb.chunks_exact(3).enumerate() {
        let theta = ((i / width) as f32 + 0.5) / height as f32 * std::f32::consts::PI;
        let solid_angle = theta.sin() * (std::f32::consts::TAU / width as f32) * (std::f32::consts::PI / height as f32);
        let cosine = direction_of(i).dot(sun).max(0.0);
        let energy = lum(px) * cosine * solid_angle;
        total += energy;
        if lum(px) >= 0.5 * peak {
            from_sun += energy;
            color += glam::Vec3::from_slice(px) * cosine * solid_angle;
        }
    }
    SunSpec {
        direction: (environment_from_world.inverse() * sun).normalize_or_zero(),
        weight: if total > 0.0 { (from_sun / total).clamp(0.0, 1.0) } else { 0.0 },
        color: color / color.max_element().max(1e-6),
    }
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
        // 世界は x 右・y 下・z 奥(camera.rs の base 回転)。等距円筒図は y 上・-z 正面なので、
        // X 軸まわり 180° で写す(鏡像にしない)。
        let environment_from_world = glam::Mat3::from_diagonal(glam::vec3(1.0, -1.0, -1.0));
        let sun = sun_from_equirect(&small, CONVOLVE_INPUT_SIZE.0, CONVOLVE_INPUT_SIZE.1, environment_from_world);
        Ok(Arc::new(GpuEnvironmentData {
            environment: re_renderer::Environment {
                radiance,
                irradiance,
                environment_from_world,
                strength: 1.0,
            },
            size: [width as f32, height as f32],
            sun,
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
