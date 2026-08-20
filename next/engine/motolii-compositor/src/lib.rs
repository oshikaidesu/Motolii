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

pub use headless::HeadlessError;

/// 素材ハンドル。上流の型をそのまま通す(包み直さない)。
pub use re_renderer::resource_managers::GpuTexture2D;

/// 合成の器。comp の解像度だけを持つ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompSpec {
    pub width: u32,
    pub height: u32,
}

/// 1枚の layer。**空間に立つ板**であり、2D の完成フレームではない。
#[derive(Clone)]
pub struct Layer {
    pub texture: GpuTexture2D,
    /// comp 座標(ピクセル・左上原点)での左上位置。
    pub top_left: [f32; 2],
    /// comp 座標での大きさ。
    pub size: [f32; 2],
    /// 重ね順。**大きいほど手前**。
    pub order: i32,
    /// 0.0〜1.0。
    pub opacity: f32,
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
            |_caps| re_renderer::RenderConfig::testing(),
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

    /// **唯一の評価経路**。RGBA8(premultiplied)を返す。
    ///
    /// preview はこの結果を窓へ出し、export は同じ結果を mux へ渡す。
    pub fn render(
        &mut self,
        comp: CompSpec,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        let rects: Vec<TexturedRect> = layers
            .iter()
            .map(|layer| TexturedRect {
                top_left_corner_position: glam::vec3(layer.top_left[0], layer.top_left[1], 0.0),
                extent_u: glam::vec3(layer.size[0], 0.0, 0.0),
                extent_v: glam::vec3(0.0, layer.size[1], 0.0),
                colormapped_texture: re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                    layer.texture.clone(),
                ),
                options: RectangleOptions {
                    // premultiplied なので alpha も色も同じ係数で掛ける。
                    multiplicative_tint: Rgba::from_rgba_premultiplied(
                        layer.opacity,
                        layer.opacity,
                        layer.opacity,
                        layer.opacity,
                    ),
                    depth_offset: layer.order as re_renderer::DepthOffset,
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
        self.ctx.begin_frame();

        let mut out: Option<Vec<u8>> = None;
        re_renderer::ScreenshotProcessor::next_readback_result::<()>(
            &self.ctx,
            identifier,
            |data, _extent, ()| {
                out = Some(data.to_vec());
            },
        );

        out.ok_or(CompositorError::ReadbackMissing)
    }
}
