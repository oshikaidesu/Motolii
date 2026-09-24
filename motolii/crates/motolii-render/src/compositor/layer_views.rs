//! The Views layers ask for (a surface Vism's `VIEWS`), realized once per frame by the host: each is
//! a view of the work recorded by the same stack as the Camera's and the Stage's, from the asking
//! layer's centre. The order they are drawn in, and how a cycle of Views is made finite, is the
//! host's execution; the work only says which Views it reads.

use super::view::{visit_inputs, Observer};
use super::*;

/// How the host makes Views that see each other in one frame finite. A policy of the host's
/// execution, not of the work: a Vism only says which Views it reads.
///
/// Adopted: cycles are detected and every request is drawn once, in a fixed order.
/// Typed seam, not decided (2026-09-24): what stands in for a View a cycle has not drawn yet.
/// Today it is simply not bound (that surface reads no Views, `view_count() == 0`).
pub(crate) enum SameFrameCycle {
    /// Requests are drawn so that a View comes after every View it sees; within a cycle, in the
    /// order the layers were asked, each seeing the Views drawn before it in this frame.
    EarlierFirst,
}

impl SameFrameCycle {
    const POLICY: Self = Self::EarlierFirst;

    /// The order to draw `n` requests in, where `sees(a, b)` is true when `a`'s faces see `b`.
    fn order(n: usize, sees: impl Fn(usize, usize) -> bool) -> Vec<usize> {
        match Self::POLICY {
            Self::EarlierFirst => {
                // Depth-first post-order: what a request sees is drawn before it. An edge back into
                // the path is a cycle, and it is not followed.
                fn visit(at: usize, n: usize, sees: &dyn Fn(usize, usize) -> bool, state: &mut [u8], out: &mut Vec<usize>) {
                    state[at] = 1;
                    for next in 0..n {
                        if state[next] == 0 && sees(at, next) {
                            visit(next, n, sees, state, out);
                        }
                    }
                    state[at] = 2;
                    out.push(at);
                }
                let mut state = vec![0u8; n];
                let mut out = Vec::with_capacity(n);
                for at in 0..n {
                    if state[at] == 0 {
                        visit(at, n, &sees, &mut state, &mut out);
                    }
                }
                out
            }
        }
    }
}

impl Compositor {
    /// The Views the layers asked for (a surface Vism's `VIEWS`): each layer's once per frame, from
    /// the centre of all its copies, side by side in one picture. A View is a view of the work like
    /// the Camera's and the Stage's — the same layers, runs, plates, effects, glass and Views,
    /// recorded by the same stack — from another eye, without the asking layer's own copies. What
    /// the Views draw is the frame's own inputs, so it is the frame's (prepared once, for every view).
    ///
    /// A View shows the Views of the layers it sees: those are drawn first (the host orders the
    /// requests by what each one's faces can see). Layers that see each other in the same frame are
    /// a cycle the host makes finite ([`SameFrameCycle`]); the work does not say how.
    pub(crate) fn draw_layer_views(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        world: &ViewWorld<'_>,
    ) -> Result<Vec<crate::render::compositor::light::LayerViews>, CompositorError> {
        struct Request {
            needs: std::sync::Arc<super::effects::surface_program::ViewNeeds>,
            lo: glam::Vec3,
            hi: glam::Vec3,
        }
        let mut requests: Vec<Request> = Vec::new();
        visit_inputs(inputs, &mut |input| {
            let Some(needs) = &input.shading.views else { return };
            let Some((a, b)) = super::surface_scene::bounds(comp, input) else { return };
            match requests.iter_mut().find(|r| r.needs.owner == needs.owner) {
                Some(r) => { r.lo = r.lo.min(a); r.hi = r.hi.max(b); }
                None => requests.push(Request { needs: needs.clone(), lo: a, hi: b }),
            }
        });
        // Which requests each one's faces see (a cone per face against the other's bounding sphere).
        let sees = |a: &Request, b: &Request| {
            let origin = (a.lo + a.hi) * 0.5;
            let (centre, radius) = ((b.lo + b.hi) * 0.5, (b.hi - b.lo).length() * 0.5);
            let to = centre - origin;
            let distance = to.length();
            if distance <= radius { return true; }
            a.needs.views.iter().any(|view| {
                let look = glam::Vec3::from(view.look).normalize_or_zero();
                let half = ((view.fov.to_radians() * 0.5).tan() * std::f32::consts::SQRT_2).atan();
                look.angle_between(to) <= half + (radius / distance).asin()
            })
        };
        let order = SameFrameCycle::order(requests.len(), |a, b| a != b && sees(&requests[a], &requests[b]));
        let mut drawn: Vec<crate::render::compositor::light::LayerViews> = Vec::new();
        for at in order {
            let Request { needs, lo, hi } = &requests[at];
            let owner = needs.owner;
            let origin = (*lo + *hi) * 0.5;
            let near = ((*hi - *lo).length() * 1e-4).clamp(0.01, 1.0);
            let size = needs.size;
            let count = needs.views.len() as u32;
            let mip_level_count = re_renderer::resource_managers::MipmapGenerator::mip_level_count(size * count, size);
            let picture = self.ctx.gpu_resources.textures.alloc(&self.ctx.device, &re_renderer::TextureDesc {
                label: "layer-views".into(),
                size: wgpu::Extent3d { width: size * count, height: size, depth_or_array_layers: 1 },
                mip_level_count,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: BLEND_TARGET_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::TEXTURE_BINDING,
            });
            // The whole face is the View (the window's region is the composition's own).
            let window = Window { width: size, height: size, roi: [0.0, 0.0, comp.width as f32, comp.height as f32] };
            // What this View sees shows the Views drawn before it.
            let inside = ViewWorld { environment: world.environment, motion: world.motion, light: world.light, views: &drawn, meshes: None };
            let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("layer-views") });
            for (i, view) in needs.views.iter().enumerate() {
                let look = glam::Mat4::look_at_rh(origin, origin + glam::Vec3::from(view.look), glam::Vec3::from(view.up));
                let observer = Observer {
                    projection: crate::doc::core::CameraProjection {
                        eye: origin,
                        rotation: glam::Quat::from_mat3(&glam::Mat3::from_mat4(look)),
                        vertical_fov_radians: view.fov.to_radians(),
                        aspect_ratio: 1.0,
                        near_plane_distance: near,
                    },
                    near_fade: 0.0,
                    requested: true,
                };
                // Histories of effects seen in a View are that View's own.
                let history = 0x8000_0000 | ((owner as u32 & 0x00ff_ffff) << 4) | i as u32;
                if let Some(face) = self.record_stack(comp, window, observer, inputs, NO_BACKGROUND, &inside, history, None, Some(owner), &mut encoder)? {
                    encoder.copy_texture_to_texture(
                        wgpu::TexelCopyTextureInfo { texture: &face.texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                        wgpu::TexelCopyTextureInfo { texture: &picture.texture, mip_level: 0, origin: wgpu::Origin3d { x: i as u32 * size, y: 0, z: 0 }, aspect: wgpu::TextureAspect::All },
                        wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                    );
                }
                self.surface_work.layer_views += 1;
            }
            // Only the levels the Vism's declared roughness reads (`VIEW_BLUR`), as the backdrop's
            // are; a Vism declaring none may read any level.
            let levels = needs.roughness.map_or(mip_level_count, |roughness| super::view::backdrop_levels_read(roughness, mip_level_count));
            self.surface_work.view_mip_levels += u64::from(levels);
            self.ctx.texture_manager_2d.generate_mipmap_levels(&self.ctx, &mut encoder, &picture.texture, levels);
            self.ctx.queue_commands([encoder.finish()]);
            let picture = self.import_premultiplied(&picture)?;
            drawn.push(crate::render::compositor::light::LayerViews { owner, picture, origin, count, levels });
        }
        Ok(drawn)
    }

}

#[cfg(test)]
mod same_frame_cycle_tests {
    use super::SameFrameCycle;

    #[test]
    fn a_view_is_drawn_after_the_views_it_sees() {
        // 0 sees 1, 1 sees 2: Main → View 0 → View 1 → View 2.
        let order = SameFrameCycle::order(3, |a, b| (a, b) == (0, 1) || (a, b) == (1, 2));
        let at = |i| order.iter().position(|&o| o == i).unwrap();
        assert!(at(2) < at(1) && at(1) < at(0), "{order:?}");
    }

    #[test]
    fn views_that_see_each_other_are_drawn_once_each_in_a_fixed_order() {
        let order = SameFrameCycle::order(3, |a, b| a != b);
        assert_eq!(order.len(), 3);
        assert_eq!(order, SameFrameCycle::order(3, |a, b| a != b));
        let mut seen = order.clone();
        seen.sort();
        assert_eq!(seen, vec![0, 1, 2]);
    }
}
