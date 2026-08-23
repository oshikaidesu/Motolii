use re_renderer::renderer::{
    RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::Rgba;

use crate::*;

impl Compositor {
    /// **唯一の評価経路**。RGBA8(premultiplied)を返す。
    ///
    /// preview はこの結果を窓へ出し、export は同じ結果を mux へ渡す。
    pub fn render(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<Vec<u8>, CompositorError> {
        self.render_with_timing(comp, camera, layers)
            .map(|(frame, _)| frame)
    }

    /// 内訳つき。**どこが遅いかを隠さない**ための口で、製品経路は [`Self::render`]。
    pub fn render_with_timing(
        &mut self,
        comp: CompSpec,
        camera: ResolvedCamera,
        layers: &[Layer],
    ) -> Result<(Vec<u8>, RenderTiming), CompositorError> {
        let mut timing = RenderTiming::default();
        let build_start = std::time::Instant::now();

        // **投影の正本は `motolii-core::camera`**。ここでは組み立てず、そこが返す
        // 値をそのまま `macaw`/`re_renderer` の型へ詰め替えるだけ。
        let projection = motolii_core::camera_projection(comp, camera);
        // pinned layer(裁定113)用: z=0 平面でのカメラの写像の逆行列。層の transform に
        // 前もって掛けておけば、この後カメラを通しても打ち消し合って画面上不動になる。
        let pinned_cancel = motolii_core::camera_screen_from_world_z0(comp, camera).inverse();

        let rects: Vec<TexturedRect> = layers
            .iter()
            .map(|layer| {
                let (transform, z) = if layer.pinned {
                    (pinned_cancel * layer.placement.transform, 0.0)
                } else {
                    (layer.placement.transform, layer.placement.z)
                };
                // **この入口は分離可能 blend を実装しない**(モジュール doc「分離可能
                // blend」節、`fixed_function_tint_alpha` 参照)——`Normal`/`Add` 以外は
                // `Err(CompositorError::UnsupportedBlendMode)` で明示的に拒む。
                let a = fixed_function_tint_alpha(layer.blend_mode, layer.placement.opacity)?;
                Ok(TexturedRect {
                    // **affine のまま板にする**。`TexturedRect` は左上と2本の辺ベクトルで
                    // 四角形を表すので、変換後の基底ベクトルをそのまま渡せば
                    // 回転も拡大も skew も**シェーダを1行も変えずに**通る。
                    top_left_corner_position: to_point3(
                        transform.transform_point2(glam::Vec2::ZERO),
                        z,
                    ),
                    extent_u: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(layer.size[0], 0.0)),
                    ),
                    extent_v: to_vector3(
                        transform.transform_vector2(glam::Vec2::new(0.0, layer.size[1])),
                    ),
                    colormapped_texture: re_renderer::renderer::ColormappedTexture::from_unorm_rgba(
                        layer.texture.clone(),
                    ),
                    options: RectangleOptions {
                        // premultiplied なので rgb は opacity で揃えて掛ける。alpha だけ
                        // blend mode で分かれる: Normal は opacity と同じ(通常の
                        // premultiplied alpha-over)。Add は 0(module doc の式変形どおり、
                        // `out = 1×src + (1-src.a)×dst` の `src.a` を 0 にすると
                        // `out = src + dst` になる — 加算合成そのもの、alpha は不変)。
                        multiplicative_tint: Rgba::from_rgba_premultiplied(
                            layer.placement.opacity,
                            layer.placement.opacity,
                            layer.placement.opacity,
                            a,
                        ),
                        depth_offset: layer.placement.order,
                        ..Default::default()
                    },
                })
            })
            .collect::<Result<Vec<_>, CompositorError>>()?;

        self.ctx.begin_frame();

        let draw_data = RectangleDrawData::new(&self.ctx, &rects)
            .map_err(|e| CompositorError::Rectangles(e.to_string()))?;

        // `rotation`/`eye` は `motolii-core::CameraProjection` が返す形(world → view の
        // 回転 + カメラ位置)。`macaw::IsoTransform::transform_point3` は
        // `rotation*p + translation` を計算するので、`translation = -(rotation*eye)`
        // にすれば `view = rotation*(p - eye)` になる。
        let view_from_world = macaw::IsoTransform::from_rotation_translation(
            projection.rotation,
            -(projection.rotation * projection.eye),
        );

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-comp".into(),
                // 「同じ絵が出る」ことが preview=export の前提なので beauty より決定性。
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [comp.width, comp.height],
                view_from_world,
                projection_from_view: Projection::Perspective {
                    vertical_fov: projection.vertical_fov_radians,
                    near_plane_distance: projection.near_plane_distance,
                    aspect_ratio: projection.aspect_ratio,
                },
                pixels_per_point: 1.0,
                // 既定 `No` は composite shader が `color = vec4f(color.rgb, 1.0)` へ
                // 強制する(上流 `composite.wgsl` 一次確認)ので、readback の alpha が
                // 常に 255 へ潰れる。`Premultiplied` は `color = vec4f(color.rgb, color.a)`
                // の素通し分岐 — 我々の layer は premultiplied alpha で描いているので
                // 意味が合う。`CompositingScreenshot` フェーズも同じ `CompositorDrawData`
                // (同じ uniform)を使う(`ViewBuilder::new` が一度だけ作って両フェーズへ
                // queue する)ので、screenshot 読み戻しにもこの分岐がそのまま効く
                // — fork 改造なしで alpha が生きることを `alpha_survives_the_composite_step`
                // (tests/compose.rs)で実測済み(2026-08-20)。
                blend_with_background: BlendWithBackground::Premultiplied,
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
