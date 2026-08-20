//! wraps: re_renderer — 合成器。
//!
//! layer は **板**(`TexturedRect`)、重ね順は `depth_offset`、不透明度は
//! `multiplicative_tint` の alpha。カメラは正射影の `TopLeftCornerAndExtendZ` で、
//! world 単位 = ピクセル・原点左上 = **AE の comp 座標そのもの**。
//!
//! 背骨2: **評価経路は1本**。preview も export も [`Compositor::render`] を呼び、
//! 違いは窓の有無だけである。第二経路を作れる公開 API をここに置かない。
//!
//! 2026-08-11「direct `re_renderer` scene を禁止」はリセット裁定3(viewer 層を引かない)で
//! 撤回済み。禁止の趣旨(第二 runtime を作らない)は上の背骨2で維持する。
//!
//! GPU の起こし方も上流のものをそのまま使う — instance descriptor・adapter 選択・
//! device limits はいずれも `re_renderer::device_caps` が持っている(自前で書かない)。

use re_renderer::renderer::{RectangleDrawData, RectangleOptions, TexturedRect};
use re_renderer::resource_managers::ImageDataDesc;
use re_renderer::view_builder::{
    self, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::{RenderContext, Rgba};

mod headless;

/// comp は 2D なので z は常に 0。板は同一平面に並び、前後は `depth_offset` が決める。
fn to_vec3(v: glam::Vec2) -> glam::Vec3 {
    glam::vec3(v.x, v.y, 0.0)
}

pub use headless::HeadlessError;

/// 素材ハンドル。上流の型をそのまま通す(包み直さない)。
pub use re_renderer::resource_managers::GpuTexture2D;

/// 合成の器。**定義は `motolii-core`** にある(背骨2を依存グラフで守るため)。
pub use motolii_core::{CompSpec, LayerPlacement};

/// 1枚の layer。**空間に立つ板**であり、2D の完成フレームではない。
///
/// 置き方は `motolii-core::LayerPlacement` をそのまま持つ。store 側の
/// `ResolvedLayer` と**同じ型**を共有しているので、置き方の property が増えても
/// ここで並べ直さない。
#[derive(Clone)]
pub struct Layer {
    pub texture: GpuTexture2D,
    /// 素材の実寸(ピクセル)。板のローカル矩形は `(0,0)-(size)` で、
    /// そこへ `placement.transform` を掛けて comp 座標の四角形にする。
    pub size: [f32; 2],
    pub placement: LayerPlacement,
}

#[derive(Debug, thiserror::Error)]
pub enum CompositorError {
    #[error("GPU の用意に失敗した: {0}")]
    Headless(#[from] HeadlessError),
    #[error("re_renderer の context を作れない: {0}")]
    Context(String),
    #[error("板の組み立てに失敗した: {0}")]
    Rectangles(String),
    #[error("view の組み立てに失敗した: {0}")]
    View(String),
    #[error("描画に失敗した: {0}")]
    Draw(String),
    #[error("読み戻しが返ってこなかった")]
    ReadbackMissing,
}

/// 1フレームの内訳。どこで時間を使っているかを隠さない。
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderTiming {
    /// 板の組み立てと view の用意。
    pub build_us: u128,
    /// GPU への提出と完了待ち。
    pub gpu_us: u128,
    /// GPU → CPU の読み戻し。**preview には本来要らない**。
    pub readback_us: u128,
}

impl RenderTiming {
    pub fn total_us(&self) -> u128 {
        self.build_us + self.gpu_us + self.readback_us
    }
}

pub struct Compositor {
    ctx: RenderContext,
    /// 読み戻しの識別子。1つの Compositor で連番にする。
    next_readback: u64,
}

impl Compositor {
    /// 窓を持たない GPU の上に合成器を建てる。**export も preview もこれ1つ**。
    pub fn headless() -> Result<Self, CompositorError> {
        let gpu = headless::HeadlessGpu::new()?;
        let ctx = RenderContext::new(
            &gpu.adapter,
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
        .map_err(|e| CompositorError::Context(e.to_string()))?;

        Ok(Self {
            ctx,
            next_readback: 1,
        })
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
        use re_renderer::resource_managers::{SourceImageDataFormat, YuvMatrixCoefficients, YuvPixelLayout, YuvRange};

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

    /// **唯一の評価経路**。RGBA8(premultiplied)を返す。
    ///
    /// preview はこの結果を窓へ出し、export は同じ結果を mux へ渡す。
    pub fn render(
        &mut self,
        comp: CompSpec,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        self.render_with_timing(comp, layers).map(|(frame, _)| frame)
    }

    /// 内訳つき。**どこが遅いかを隠さない**ための口で、製品経路は [`Self::render`]。
    pub fn render_with_timing(
        &mut self,
        comp: CompSpec,
        layers: &[Layer],
    ) -> Result<(Vec<u8>, RenderTiming), CompositorError> {
        let mut timing = RenderTiming::default();
        let build_start = std::time::Instant::now();
        let rects: Vec<TexturedRect> = layers
            .iter()
            .map(|layer| TexturedRect {
                // **affine のまま板にする**。`TexturedRect` は左上と2本の辺ベクトルで
                // 四角形を表すので、変換後の基底ベクトルをそのまま渡せば
                // 回転も拡大も skew も**シェーダを1行も変えずに**通る。
                top_left_corner_position: to_vec3(
                    layer.placement.transform.transform_point2(glam::Vec2::ZERO),
                ),
                extent_u: to_vec3(
                    layer
                        .placement
                        .transform
                        .transform_vector2(glam::Vec2::new(layer.size[0], 0.0)),
                ),
                extent_v: to_vec3(
                    layer
                        .placement
                        .transform
                        .transform_vector2(glam::Vec2::new(0.0, layer.size[1])),
                ),
                colormapped_texture: re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                    layer.texture.clone(),
                ),
                options: RectangleOptions {
                    // premultiplied なので alpha も色も同じ係数で掛ける。
                    multiplicative_tint: Rgba::from_rgba_premultiplied(
                        layer.placement.opacity,
                        layer.placement.opacity,
                        layer.placement.opacity,
                        layer.placement.opacity,
                    ),
                    depth_offset: layer.placement.order,
                    ..Default::default()
                },
            })
            .collect();

        self.ctx.begin_frame();

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp".into(),
                // 「同じ絵が出る」ことが preview=export の前提なので beauty より決定性。
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [comp.width, comp.height],
                view_from_world: macaw::IsoTransform::IDENTITY,
                projection_from_view: Projection::Orthographic {
                    camera_mode: view_builder::OrthographicCameraMode::TopLeftCornerAndExtendZ,
                    vertical_world_size: comp.height as f32,
                    far_plane_distance: 1000.0,
                },
                pixels_per_point: 1.0,
                ..Default::default()
            },
            re_renderer::ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;

        view_builder.queue_draw(&self.ctx, draw_data);

        let identifier = self.next_readback;
        self.next_readback += 1;
        view_builder
            .schedule_screenshot(&self.ctx, identifier, ())
            .map_err(|e| CompositorError::View(e.to_string()))?;

        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.build_us = build_start.elapsed().as_micros();

        let gpu_start = std::time::Instant::now();

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);

        // 読み戻しの段取りは上流の frame 進行に埋まっている(`RenderContext::begin_frame`
        // が `GpuReadbackBelt::after_queue_submit`(map 開始)→ `begin_frame`(受け取り)
        // の順に呼ぶ)。窓のある側は次フレームで受け取るが、ここは窓が無いので
        // **同じ呼び出しの中でフレームを2回進めて**受け取る。
        //   1回目: map_async を開始する
        //   poll : map の完了と提出済み作業の完了を待つ
        //   2回目: receive_chunks が届いた chunk を拾う
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        timing.gpu_us = gpu_start.elapsed().as_micros();

        let readback_start = std::time::Instant::now();
        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        re_renderer::ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        let frame = out.ok_or(CompositorError::ReadbackMissing)?;
        timing.readback_us = readback_start.elapsed().as_micros();
        Ok((frame, timing))
    }
}
