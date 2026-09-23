//! The world's light as Motolii lays it into a view: the scene's reflection probes (a view capture),
//! the sun and what blocks it (a coverage picture), and the constants the standard material reads
//! (`effects/program/material.wgsl`).

use re_renderer::resource_managers::GpuTexture2D;
use re_renderer::view_builder::TargetConfiguration;

/// Shared local reflection capture: up to two sets of six faces (+X, -X, +Y, -Y, +Z, -Z) in a 3x4
/// atlas. RGB is linear radiance and alpha is coverage; uncovered directions use the environment.
#[derive(Clone, Debug)]
pub struct SceneReflection {
    pub atlas: GpuTexture2D,
    pub origins: [glam::Vec3; 2],
    pub count: u32,
    pub bounds_min: glam::Vec3,
    pub bounds_max: glam::Vec3,
    /// Fully trusted radius around each capture; fades to zero at twice this radius.
    /// Zero retains unbounded legacy blending.
    pub influence_radii: [f32; 2],
}

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

/// Lays the world's light into a view. `capture` marks the view that draws the sun's cookie itself:
/// its surfaces write what they let through instead of shading. The layout of the constants is
/// material.wgsl's accessors.
pub(crate) fn light_view(config: &mut TargetConfiguration, reflection: Option<&SceneReflection>, sun: Option<&SunLight>, capture: bool) {
    let c = &mut config.program_constants;
    if let Some(r) = reflection {
        config.view_capture = Some(r.atlas.clone());
        c[0] = r.origins[0].extend(r.count as f32);
        c[1] = r.origins[1].extend(r.influence_radii[0]);
        c[2] = r.bounds_min.extend(r.influence_radii[1]);
        c[3] = r.bounds_max.extend(0.0);
    }
    if let Some(s) = sun {
        config.coverage = Some(s.cookie.clone());
        c[4] = s.direction.extend(s.weight);
    }
    c[5] = sun.map_or(glam::Vec3::ZERO, |s| s.color).extend(if capture { 1.0 } else { 0.0 });
    let uv_from_world = sun.map_or(glam::Mat4::IDENTITY, |s| s.uv_from_world);
    c[6..10].copy_from_slice(&[uv_from_world.x_axis, uv_from_world.y_axis, uv_from_world.z_axis, uv_from_world.w_axis]);
}
