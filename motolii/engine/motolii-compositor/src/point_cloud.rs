//! wraps: `re_renderer::PointCloudBuilder`/`PointCloudDrawData` — 点群を1枚のオフスクリーン
//! texture へ焼く(engine 側の `LayerSource::PointCloud` から呼ばれる)。
//!
//! **新しい合成経路は作らない**: 出力は他の layer と同じ [`GpuTexture2D`] 1枚
//! (`Compositor::render`/`render_with_timing` の `TexturedRect` としてそのまま混ざる)。
//! `sequential.rs` の「layer 単体を自分の `ViewBuilder` へ描き、`main_target()` を
//! `texture_manager_2d.import_gpu_premultiplied` で読み戻す」手口をそのまま流用する
//! (同じ device・同じ `RenderContext` — zero-copy、CPU 往復なし)。
//!
//! カメラは **bounding sphere オートフレーミングの固定視点**。手で振れる3Dカメラ・
//! LOD/streaming はこの切片の非目標(`docs/reviews/2026-08-28-seams-remaining.md` S10c)。

use re_renderer::renderer::PointCloudBatchFlags;
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder,
};
use re_renderer::{Color32, PointCloudBuilder, Rgba, Size, ViewBuilderId};

use crate::{Compositor, CompositorError, GpuTexture2D};

/// `motolii_core::camera::CAMERA_BASE_VERTICAL_FOV_DEGREES` と同じ値 — comp の 2.5D
/// カメラと画角の見え方を揃える(独自の画角定数を増やさない)。
const POINT_CLOUD_VERTICAL_FOV_DEGREES: f32 = motolii_core::CAMERA_BASE_VERTICAL_FOV_DEGREES;

impl Compositor {
    /// 点群 (`positions`/`colors`、素材そのままの world 座標系) を `width`×`height`
    /// の texture へ焼く。`colors` は `positions` より短くてよい(足りない分は
    /// `PointCloudBatchBuilder::add_points` の既定 = 白)。
    pub fn render_point_cloud_to_texture(
        &mut self,
        positions: &[[f32; 3]],
        colors: &[[u8; 4]],
        width: u32,
        height: u32,
    ) -> Result<GpuTexture2D, CompositorError> {
        let positions: Vec<glam::Vec3> = positions.iter().copied().map(glam::Vec3::from).collect();
        let colors: Vec<Color32> = colors
            .iter()
            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .collect();

        let (center, radius) = bounding_sphere(&positions);
        let half_fov = (POINT_CLOUD_VERTICAL_FOV_DEGREES * 0.5).to_radians();
        // 半径0(点1つ・全点同座標)でも壊れないよう最小距離を敷く。1.2 は縁に
        // ちょうど触れないための余白(自由なデザイン定数、出典なし)。
        let distance = (radius.max(1e-3) * 1.2) / half_fov.tan();
        let eye = center + glam::Vec3::new(0.0, 0.0, distance);
        let view_from_world = macaw::IsoTransform::look_at_rh(eye, center, glam::Vec3::Y)
            .ok_or_else(|| {
                CompositorError::View("点群カメラを組めない(eye/target が縮退)".into())
            })?;

        self.ctx.begin_frame();

        // 固定 1.5 ui-point 半径(スクリーン空間、scene のスケールに関係なく一定の
        // 見た目のドット)。scene 単位で決めると点群のスケールに応じて毎回別の
        // 値を選ぶ必要が出る——ui-point 固定はそれを避ける自由なデザイン定数
        // (出典なし)。
        let radii = vec![Size::new_ui_points(1.5); positions.len()];
        let picking_ids = vec![Default::default(); positions.len()];
        let mut builder = PointCloudBuilder::new(&self.ctx);
        builder
            .batch("motolii-point-cloud")
            .world_from_obj(glam::Affine3A::IDENTITY)
            .add_points_slow(&positions, &radii, &colors, &picking_ids)
            .flags(PointCloudBatchFlags::FLAG_ENABLE_SHADING);
        let draw_data = builder
            .into_draw_data()
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        let mut view_builder = ViewBuilder::new(
            &self.ctx,
            TargetConfiguration {
                name: "motolii-point-cloud".into(),
                render_mode: RenderMode::Deterministic,
                resolution_in_pixel: [width, height],
                view_from_world,
                projection_from_view: Projection::Perspective {
                    vertical_fov: half_fov * 2.0,
                    near_plane_distance: (distance * 0.01).max(0.001),
                    aspect_ratio: width as f32 / height as f32,
                },
                pixels_per_point: 1.0,
                blend_with_background: BlendWithBackground::Premultiplied,
                ..Default::default()
            },
            ViewBuilderId::new(self.next_readback),
        )
        .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;

        view_builder.queue_draw(&self.ctx, draw_data);
        let command_buffer = view_builder
            .draw(&self.ctx, Rgba::TRANSPARENT)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        self.ctx.before_submit();
        self.ctx.queue.submit([command_buffer]);
        self.sequential_submits += 1;
        self.ctx.begin_frame();
        self.ctx
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| CompositorError::Draw(e.to_string()))?;

        let main_target = view_builder.main_target().clone();
        self.next_effect_key += 1;
        let key = self.next_effect_key;
        self.ctx
            .texture_manager_2d
            .import_gpu_premultiplied(key, &self.ctx, &main_target.texture)
            .map_err(|e| CompositorError::Effect(e.to_string()))
    }
}

/// 中心と半径(全点を内包する最小の球、正確な最小包含球ではなく AABB 対角/2の近似で
/// 十分——カメラの自動フレーミングにミリ精度は要らない)。
fn bounding_sphere(positions: &[glam::Vec3]) -> (glam::Vec3, f32) {
    if positions.is_empty() {
        return (glam::Vec3::ZERO, 1.0);
    }
    let mut min = positions[0];
    let mut max = positions[0];
    for &p in &positions[1..] {
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    let radius = positions
        .iter()
        .map(|&p| (p - center).length())
        .fold(0.0_f32, f32::max);
    (center, radius)
}
