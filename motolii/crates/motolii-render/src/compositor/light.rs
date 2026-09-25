//! The world's light as Motolii lays it into a view: the sun and what blocks it (a coverage
//! picture), the Views a layer asked for (a view capture), and the constants the standard material
//! reads (`vism/material.wgsl`).

use re_renderer::resource_managers::GpuTexture2D;
use re_renderer::view_builder::TargetConfiguration;

/// The one light that casts shadows: the environment's brightest direction, and the picture of what
/// blocks it (a cookie rendered from the sun; premultiplied tint × coverage). Everything shaded reads it.
#[derive(Clone, Debug)]
pub struct SunLight {
    /// World direction toward the sun.
    pub direction: glam::Vec3,
    /// Share of the diffuse light that comes from the sun, in [0, 1]; what a shadow takes away.
    pub weight: f32,
    pub color: glam::Vec3,
    /// World position → cookie uv.
    pub uv_from_world: glam::Mat4,
    pub cookie: GpuTexture2D,
}

/// The Views one layer asked for (its Vism's `VIEWS`), drawn once for the prepared frame and laid
/// side by side in one picture (premultiplied; alpha is what the View saw).
#[derive(Clone, Debug)]
pub struct LayerViews {
    pub owner: u64,
    pub picture: GpuTexture2D,
    /// Where they were taken from: the centre of the layer (all its copies).
    pub origin: glam::Vec3,
    pub count: u32,
    /// Mip levels the host generated (base included): what `view_sample` may read.
    pub levels: u32,
}

/// Binds a layer's Views to the draw that shades it. The layout is material.wgsl's `view_*`.
pub(crate) fn bind_views(config: &mut TargetConfiguration, views: &LayerViews) {
    config.view_capture = Some(views.picture.clone());
    config.program_constants[6] = glam::vec4(views.count as f32, views.levels.max(1) as f32 - 1.0, 0.0, 0.0);
    config.program_constants[7] = views.origin.extend(0.0);
}

/// Lays the world's light into a view. `capture` marks the view that draws the sun's cookie itself:
/// its surfaces write what they let through instead of shading. The layout of the constants is
/// material.wgsl's accessors.
pub(crate) fn light_view(config: &mut TargetConfiguration, sun: Option<&SunLight>, capture: bool) {
    let c = &mut config.program_constants;
    if let Some(s) = sun {
        config.coverage = Some(s.cookie.clone());
        c[0] = s.direction.extend(s.weight);
    }
    c[1] = sun.map_or(glam::Vec3::ZERO, |s| s.color).extend(if capture { 1.0 } else { 0.0 });
    let uv_from_world = sun.map_or(glam::Mat4::IDENTITY, |s| s.uv_from_world);
    c[2..6].copy_from_slice(&[uv_from_world.x_axis, uv_from_world.y_axis, uv_from_world.z_axis, uv_from_world.w_axis]);
}
