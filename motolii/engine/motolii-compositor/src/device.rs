use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::RenderContext;

use crate::*;

impl Compositor {
    /// 窓を持たない GPU の上に合成器を建てる。**export も preview もこれ1つ**。
    ///
    /// 実体は [`Self::with_device`] の薄いラッパー——adapter/device/queue を
    /// [`headless::HeadlessGpu`] で自前に建てて渡すだけで、渡す format/config は
    /// 従来のまま(挙動は不変、`tests/with_device.rs` がバイト一致で縛る)。
    pub fn headless() -> Result<Self, CompositorError> {
        let gpu = headless::HeadlessGpu::new()?;
        Self::with_device(
            gpu.device,
            gpu.queue,
            // 読み戻し形式に合わせる。窓へ出す時はここが surface の形式になる。
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            // **MSAA は切る**。layer は軸に沿った板であって、アンチエイリアスすべき
            // 幾何エッジを持たない。上流の `MsaaMode::Off` の doc も「device 差が出にくい」
            // と言っているので、決定性の点でもこちらが良い。
            // ただし **速度の理由ではない**: R1 実測で 1080p 40枚は 41.6ms → 37.8ms
            // (9%)しか変わらなかった。律速は fragment の量そのものである。
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
    }

    /// 外部から与えられた GPU device/queue の上に合成器を組む第二コンストラクタ
    /// (裁定170 M2、2026-08-21。adapter 引数は M3 で落とした、2026-08-22)。
    /// **まだ誰も呼ばない** — iced 側の配線はここではやらない(配線ゼロ = 挙動ゼロ
    /// 変更、browser B0 骨格と同じ手口)。
    ///
    /// [`Self::headless`] が「device を建てた後」にやっていた共通部分をここへ
    /// 抽出しただけで、`headless()` 自身の挙動は一切変えていない
    /// (`tests/with_device.rs` の `with_device_matches_headless` が
    /// バイト一致で縛る)。
    ///
    /// `adapter` を要求しないのは、rerun fork(`Cargo.toml` の `[patch]` 参照、
    /// 裁定170 M3)に足した `RenderContext::new_from_device` が adapter なしで
    /// `DeviceCaps`/`AdapterInfo` を device 自身(`device.adapter_info()`)から
    /// 導けるため——ここが「adapter なしで pipeline が実際に建つ」の実体。
    pub fn with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format: wgpu::TextureFormat,
        config_provider: impl FnOnce(&re_renderer::device_caps::DeviceCaps) -> re_renderer::RenderConfig,
    ) -> Result<Self, CompositorError> {
        let ctx = RenderContext::new_from_device(device, queue, output_format, config_provider)
            .map_err(|e| CompositorError::Context(e.to_string()))?;

        let glow_pipelines = effects::GlowPipelines::new(&ctx.device);
        let isf_bloom = effects::IsfProgram::compile(
            &ctx,
            effects::BLOOM_SOURCE,
            effects::ISF_TARGET_FORMAT,
        )
        .map_err(|e| CompositorError::Isf(e.to_string()))?;
        let blend_pipelines = blend::SeparableBlendPipelines::new(&ctx.device);
        let matte_pipelines = matte::MattePipelines::new(&ctx.device);

        Ok(Self {
            ctx,
            next_readback: 1,
            next_effect_key: 1,
            effect_scratch: effects::EffectScratch::default(),
            glow_pipelines,
            isf_bloom,
            blend_pipelines,
            matte_pipelines,
            sequential_submits: 0,
        })
    }

    /// [`Self::with_device`]の既定 config 版(裁定171 v2 M4、**additive** —
    /// `with_device`/[`Self::headless`]自体は無改造)。[`Self::headless`]と
    /// **同じ** `output_format`/`RenderConfig`(`MsaaMode::Off`、`headless()`の
    /// doc 参照)を使う——呼び出し側(`motolii-engine::Engine::with_device`)が
    /// `re_renderer::RenderConfig`/`MsaaMode`/`ScreenshotProcessor` を一切知らずに
    /// 済むための薄いラッパー。`output_format` の実際の効き所は
    /// `composite()`/screenshot 読み戻し側(`RenderContext` の「出力」format)で、
    /// [`Self::render_to_texture`]はそちらを一切通らない(readback しない)ので
    /// ここで固定しても実害が無い——`headless()`と同じ値を選んだのは新しい
    /// 意味を持ち込まないため。
    pub fn with_device_using_headless_defaults(
        device: wgpu::Device,
        queue: wgpu::Queue,
    ) -> Result<Self, CompositorError> {
        Self::with_device(
            device,
            queue,
            re_renderer::ScreenshotProcessor::SCREENSHOT_COLOR_FORMAT,
            |_caps| re_renderer::RenderConfig {
                msaa_mode: re_renderer::MsaaMode::Off,
            },
        )
    }

    /// premultiplied RGBA8 を GPU へ載せる。
    pub fn upload_rgba(
        &self,
        label: &str,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> Result<GpuTexture2D, CompositorError> {
        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: rgba.to_vec().into(),
                    format: wgpu::TextureFormat::Rgba8Unorm.into(),
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::AlphaChannelInUse,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

    /// デコード直後の YUV420p(planar)をそのまま GPU へ載せる。
    ///
    /// **色変換は上流(`re_renderer`)がやる**(裁定23)。ffmpeg の
    /// `-f rawvideo -pix_fmt yuv420p` が吐くバイト列は、上流の `Y_U_V420` が期待する
    /// `width × (height + height/2)` の R8 連番そのものなので、詰め替えが要らない。
    /// 自前で WGSL を書くと「色事故を防ぐために自分で変換する」動機ごと二重になる。
    pub fn upload_yuv420p(
        &self,
        label: &str,
        data: &[u8],
        width: u32,
        height: u32,
        color: motolii_core::ColorSpace,
    ) -> Result<GpuTexture2D, CompositorError> {
        use re_renderer::resource_managers::{
            SourceImageDataFormat, YuvMatrixCoefficients, YuvPixelLayout, YuvRange,
        };

        let (coefficients, range) = match color {
            motolii_core::ColorSpace::Rec709Limited => {
                (YuvMatrixCoefficients::Bt709, YuvRange::Limited)
            }
            motolii_core::ColorSpace::Rec709Full => (YuvMatrixCoefficients::Bt709, YuvRange::Full),
            motolii_core::ColorSpace::Rec601Limited => {
                (YuvMatrixCoefficients::Bt601, YuvRange::Limited)
            }
            // RGB 系の色空間で YUV を載せようとしている = 呼び手の取り違え。
            other => {
                return Err(CompositorError::Rectangles(format!(
                    "YUV420p に RGB 系の色空間が渡された: {other:?}"
                )))
            }
        };

        self.ctx
            .texture_manager_2d
            .create(
                &self.ctx,
                ImageDataDesc {
                    label: label.into(),
                    data: data.to_vec().into(),
                    format: SourceImageDataFormat::Yuv {
                        layout: YuvPixelLayout::Y_U_V420,
                        coefficients,
                        range,
                    },
                    width_height: [width, height],
                    alpha_channel_usage: re_renderer::AlphaChannelUsage::Opaque,
                },
            )
            .map_err(|e| CompositorError::Rectangles(e.to_string()))
    }

}
