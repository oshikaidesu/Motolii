use re_renderer::environment::SceneReflection;
use re_renderer::renderer::{
    GpuMeshInstance, MeshDrawData, PointCloudDrawData, RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder, ViewBuilderId,
};
use re_renderer::{ClipPlane, Rgba};

use super::*;

pub(crate) struct ReflectionResources {
    face_size: u32,
    faces: [wgpu::Texture; 6],
    atlas: wgpu::Texture,
    imported: GpuTexture2D,
}

/// 太陽から見た型紙(light cookie)の置き場。1 枚を frame ごとに描き直す。
pub(crate) struct LightCookieResources {
    texture: wgpu::Texture,
    imported: GpuTexture2D,
}

/// 型紙の一辺。影の縁の粗さはこれで決まる(スマホの shadow map と同じ嘘)。
const LIGHT_COOKIE_SIZE: u32 = 512;

impl Compositor {
    /// 「光を遮る」層があれば、太陽から正射影で 1 枚描く: 遮る層だけ、透過の色 × coverage。
    /// 光は環境の太陽(無ければ固定灯)。深度は持たない — 遮る物は光線上の全てに影を落とす。
    pub(super) fn capture_light_cookie(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
        shared: Option<&SharedMeshScene>,
    ) -> Result<Option<re_renderer::environment::SunLight>, CompositorError> {
        if !inputs.iter().any(|i| i.blocks_light) {
            return Ok(None);
        }
        let sun = environment.map_or(super::environment::SunSpec::fixed_lights(), |e| e.sun);
        let direction = sun.direction.normalize_or_zero();
        if sun.weight <= 0.0 || direction == glam::Vec3::ZERO {
            return Ok(None);
        }
        let (mut lo, mut hi, mut any) = (glam::Vec3::INFINITY, glam::Vec3::NEG_INFINITY, false);
        for input in inputs {
            if let Some((a, b)) = bounds(comp, input) {
                lo = lo.min(a);
                hi = hi.max(b);
                any = true;
            }
        }
        if !any {
            return Ok(None);
        }
        let center = (lo + hi) * 0.5;
        let radius = ((hi - lo).length() * 0.5).max(1e-3);
        let up = if direction.y.abs() > 0.9 { glam::Vec3::X } else { glam::Vec3::Y };
        let view = glam::Mat4::look_at_rh(center + direction * radius * 2.0, center, up);
        let rotation = glam::Quat::from_mat3(&glam::Mat3::from_mat4(view));
        let view_from_world = macaw::IsoTransform::from_rotation_translation(rotation, view.w_axis.truncate());
        let ortho = glam::Mat4::orthographic_rh(-radius, radius, -radius, radius, 0.0, radius * 4.0);
        let uv_from_world = glam::Mat4::from_translation(glam::vec3(0.5, 0.5, 0.0))
            * glam::Mat4::from_scale(glam::vec3(0.5, -0.5, 1.0))
            * ortho
            * view;
        let resources = match self.light_cookie.take() {
            Some(r) => r,
            None => {
                let texture = self.create_blend_scratch_texture(LIGHT_COOKIE_SIZE, LIGHT_COOKIE_SIZE);
                let imported = self.import_premultiplied(&texture)?;
                LightCookieResources { texture, imported }
            }
        };
        let draws = self.surface_scene_draws(comp, inputs, Vec::new(), true, &|layer| !inputs[layer].blocks_light, shared, 0)?;
        let config = TargetConfiguration {
            name: "light-cookie".into(),
            render_mode: RenderMode::Deterministic,
            resolution_in_pixel: [LIGHT_COOKIE_SIZE, LIGHT_COOKIE_SIZE],
            view_from_world,
            projection_from_view: Projection::Orthographic {
                camera_mode: re_renderer::view_builder::OrthographicCameraMode::NearPlaneCenter,
                vertical_world_size: radius * 2.0,
                far_plane_distance: radius * 4.0,
            },
            pixels_per_point: 1.0,
            blend_with_background: BlendWithBackground::Premultiplied,
            environment: environment.map(|e| e.environment.clone()),
            light_capture: true,
            ..Default::default()
        };
        let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, config, ViewBuilderId::new(self.next_readback), &resources.texture)
            .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        builder.queue_draw(&self.ctx, draws.rects.clone());
        for cloud in &draws.clouds {
            builder.queue_draw(&self.ctx, cloud.clone());
        }
        for mesh in &draws.meshes {
            builder.queue_draw(&self.ctx, mesh.clone());
        }
        self.pending.push(builder.draw(&self.ctx, Rgba::TRANSPARENT).map_err(|e| CompositorError::Draw(e.to_string()))?);
        self.surface_work.light_captures += 1;
        let cookie = resources.imported.clone();
        self.light_cookie = Some(resources);
        Ok(Some(re_renderer::environment::SunLight { direction, weight: sun.weight, color: sun.color, uv_from_world, cookie }))
    }
}

pub(super) struct SharedMeshScene {
    draw: MeshDrawData,
    source_layers: Vec<usize>,
}

impl SharedMeshScene {
    fn select(&self, range: std::ops::Range<usize>, skip: &dyn Fn(usize) -> bool) -> MeshDrawData {
        if !self.source_layers.iter().any(|&l| skip(l))
            && self
                .source_layers
                .first()
                .zip(self.source_layers.last())
                .is_some_and(|(&first, &last)| range.contains(&first) && range.contains(&last))
        {
            return self.draw.clone();
        }
        self.draw.select_source_instances(|source| {
            let layer = self.source_layers[source];
            range.contains(&layer) && !skip(layer)
        })
    }
}

pub(crate) struct SceneDraws {
    rects: RectangleDrawData,
    clouds: Vec<PointCloudDrawData>,
    meshes: Vec<MeshDrawData>,
}

impl SceneDraws {
    pub(super) fn queue(self, ctx: &re_renderer::RenderContext, view: &mut ViewBuilder) {
        view.queue_draw(ctx, self.rects);
        for cloud in self.clouds {
            view.queue_draw(ctx, cloud);
        }
        for mesh in self.meshes {
            view.queue_draw(ctx, mesh);
        }
    }
}

fn bounds(comp: CompSpec, input: &SequentialInput<'_>) -> Option<(glam::Vec3, glam::Vec3)> {
    let spatial = match input.content {
        SequentialContent::Environment(_) => return None,
        SequentialContent::Model(m) => Some(m.bounds),
        SequentialContent::Cloud { bounds, .. } => Some(bounds),
        SequentialContent::Rect(_) | SequentialContent::LinearRect(_) => None,
    };
    let points: Vec<glam::Vec3> = if let Some(b) = spatial {
        let world = projected_spatial_placement(
            comp,
            input.projection_camera,
            input.projection,
            input.placement,
            b,
        );
        [b.min[0], b.max[0]]
            .into_iter()
            .flat_map(|x| {
                [b.min[1], b.max[1]].into_iter().flat_map(move |y| {
                    [b.min[2], b.max[2]]
                        .into_iter()
                        .map(move |z| world.transform_point3(glam::vec3(x, y, z)))
                })
            })
            .collect()
    } else {
        let (p, u, v) = projected_placement_corners(
            comp,
            input.projection_camera,
            input.projection,
            input.placement,
            input.local_min,
            input.local_size,
        );
        vec![p, p + u, p + v, p + u + v]
    };
    if points.iter().any(|p| !p.is_finite()) {
        return None;
    }
    Some(points.into_iter().fold(
        (
            glam::Vec3::splat(f32::INFINITY),
            glam::Vec3::splat(f32::NEG_INFINITY),
        ),
        |(lo, hi), p| (lo.min(p), hi.max(p)),
    ))
}

impl Compositor {
    pub(super) fn shared_mesh_scene(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
    ) -> Result<Option<SharedMeshScene>, CompositorError> {
        if !self.gpu_instance_sharing_enabled {
            return Ok(None);
        }
        let mut count = 0;
        let mut instance_count = 0usize;
        for input in inputs {
            if let SequentialContent::Model(model) = input.content {
                if input.blend_mode != BlendMode::Normal
                    || input.opacity != 1.0
                    || input.shading.reads_backdrop
                    || input.clip.is_some()
                    || model
                        .instances
                        .iter()
                        .any(|i| i.gpu_mesh.materials.iter().any(|m| m.has_transparency))
                {
                    return Ok(None);
                }
                count += 1;
                instance_count = instance_count.saturating_add(model.instances.len());
            }
        }
        if count < 2
            || instance_count > u32::MAX as usize
            || instance_count.saturating_mul(MeshDrawData::gpu_instance_size_bytes()) as u64
                > self.ctx.device.limits().max_buffer_size
        {
            return Ok(None);
        }
        let started = std::time::Instant::now();
        let mut instances = Vec::new();
        let mut source_layers = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            if let SequentialContent::Model(model) = input.content {
                let (made, _) = self.model_instances(
                    model,
                    input.local_size.to_array(),
                    input.placement,
                    input.opacity,
                    comp,
                    input.projection_camera,
                    input.projection,
                    &input.shading,
                    input.clip,
                );
                source_layers.extend(std::iter::repeat_n(index, made.len()));
                instances.extend(made);
            }
        }
        let draw = MeshDrawData::new(&self.ctx, &instances)
            .map_err(|e| CompositorError::Draw(e.to_string()))?;
        self.surface_work.mesh_batches += 1;
        self.surface_work.mesh_instances_uploaded += instances.len() as u64;
        self.surface_work.mesh_instance_upload_bytes +=
            (instances.len() * MeshDrawData::gpu_instance_size_bytes()) as u64;
        self.surface_work.draw_data_prepare_us += started.elapsed().as_micros() as u64;
        Ok(Some(SharedMeshScene {
            draw,
            source_layers,
        }))
    }

    /// One geometry conversion for the main view and auxiliary reflection views.
    pub(super) fn surface_scene_draws(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        mut rects: Vec<TexturedRect>,
        capture: bool,
        // Layers (absolute input index) left out of these draws.
        skip: &dyn Fn(usize) -> bool,
        shared: Option<&SharedMeshScene>,
        input_offset: usize,
    ) -> Result<SceneDraws, CompositorError> {
        let started = std::time::Instant::now();
        let mut clouds = Vec::new();
        let mut mesh_groups: Vec<(ClipPlane, Vec<GpuMeshInstance>)> = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            #[cfg(test)]
            if capture && self.reflection_diagnostic_skip == Some(index) {
                continue;
            }
            if skip(input_offset + index)
                || (shared.is_some() && matches!(input.content, SequentialContent::Model(_)))
            {
                continue;
            }
            let shading = input.shading.clone();
            match input.content {
                SequentialContent::Environment(_) => {}
                SequentialContent::Cloud {
                    positions,
                    colors,
                    bounds,
                    point_size,
                } => {
                    clouds.push(self.point_cloud_draw_data(
                        positions,
                        colors,
                        bounds,
                        point_size,
                        input.placement,
                        input.opacity,
                        comp,
                        input.projection_camera,
                        input.projection,
                        input.displace,
                        input.clip,
                        outline_mask(input.outline),
                    )?);
                }
                SequentialContent::Model(model) => {
                    let (mut instances, clip) = self.model_instances(
                        model,
                        input.local_size.to_array(),
                        input.placement,
                        input.opacity,
                        comp,
                        input.projection_camera,
                        input.projection,
                        &shading,
                        input.clip,
                    );
                    for instance in &mut instances {
                        instance.outline_mask_ids = outline_mask(input.outline);
                        if input.projection == crate::doc::store::LayerProjection::TwoD && !capture {
                            // 輪郭のままの文字・図形(planar な網)も同じ法: 面の法線に沿ってカメラ側へ積み順ぶん。
                            let normal = glam::Vec3::from(instance.world_from_mesh.matrix3.z_axis).normalize_or_zero();
                            instance.world_from_mesh = glam::Affine3A::from_translation(-normal * two_d_stack_bias(input.depth_offset)) * instance.world_from_mesh;
                        }
                    }
                    if let Some((_, group)) = mesh_groups.iter_mut().find(|(c, _)| *c == clip) {
                        group.extend(instances);
                    } else {
                        mesh_groups.push((clip, instances));
                    }
                }
                SequentialContent::Rect(_) | SequentialContent::LinearRect(_) => {
                    let (mut corner, u, v) = projected_placement_corners(
                        comp,
                        input.projection_camera,
                        input.projection,
                        input.placement,
                        input.local_min,
                        input.local_size,
                    );
                    if input.projection == crate::doc::store::LayerProjection::TwoD && !capture {
                        corner += -u.cross(v).normalize_or_zero() * two_d_stack_bias(input.depth_offset);
                    }
                    let alpha = if input.blend_mode == BlendMode::Add {
                        0.0
                    } else {
                        input.opacity
                    };
                    rects.push(TexturedRect {
                        top_left_corner_position: corner,
                        extent_u: u,
                        extent_v: v,
                        colormapped_texture: input.content.image().expect("planar image"),
                        options: RectangleOptions {
                            multiplicative_tint: Rgba::from_rgba_premultiplied(
                                input.opacity,
                                input.opacity,
                                input.opacity,
                                alpha,
                            ),
                            depth_offset: if capture { 0 } else { input.depth_offset },
                            clip: input
                                .clip
                                .map_or(ClipPlane::NONE, |c| c.world_for_rect(corner, u, v)),
                            field_grid: shading.field_grid(),
                            surface: shading.program,
                            surface_params: shading.params,
                            outline_mask: outline_mask(input.outline),
                            ..Default::default()
                        },
                    });
                }
            }
        }
        self.surface_work.mesh_batches += mesh_groups.len() as u64;
        let uploaded: usize = mesh_groups
            .iter()
            .map(|(_, instances)| instances.len())
            .sum();
        self.surface_work.mesh_instances_uploaded += uploaded as u64;
        self.surface_work.mesh_instance_upload_bytes +=
            (uploaded * MeshDrawData::gpu_instance_size_bytes()) as u64;
        let mut meshes: Vec<MeshDrawData> = mesh_groups
            .into_iter()
            .map(|(clip, instances)| {
                MeshDrawData::new_clipped(&self.ctx, &instances, clip)
                    .map_err(|e| CompositorError::Draw(e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        if let Some(shared) = shared {
            meshes.push(shared.select(input_offset..input_offset + inputs.len(), skip));
        }
        self.surface_work.draw_data_prepare_us += started.elapsed().as_micros() as u64;
        Ok(SceneDraws {
            rects: RectangleDrawData::new(&self.ctx, &rects)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?,
            clouds,
            meshes,
        })
    }

    /// At most two shared, single-bounce captures per scene evaluation. No per-copy probes.
    pub(super) fn capture_scene_reflection(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
        shared: &mut Option<SharedMeshScene>,
    ) -> Result<Option<SceneReflection>, CompositorError> {
        let candidates: Vec<_> = inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| {
                if matches!(input.content, SequentialContent::Model(m) if m.planar_size.is_some()) && !input.shading.reads_backdrop {
                    return None;
                }
                if !input
                    .shading
                    .program
                    .as_ref()
                    .is_some_and(|p| p.desc().surface.is_some())
                {
                    return None;
                }
                bounds(comp, input).map(|(lo, hi)| (index, lo, hi))
            })
            .collect();
        let Some(first) = candidates.first() else {
            return Ok(None);
        };
        let last = candidates.last().expect("nonempty candidates");
        #[cfg(test)]
        let (first, last) = if matches!(self.reflection_probe_experiment, 0 | 2) {
            let compare = |a: &&(usize, glam::Vec3, glam::Vec3),
                           b: &&(usize, glam::Vec3, glam::Vec3)| {
                let a = (a.1 + a.2) * 0.5;
                let b = (b.1 + b.2) * 0.5;
                a.x.total_cmp(&b.x)
                    .then(a.y.total_cmp(&b.y))
                    .then(a.z.total_cmp(&b.z))
            };
            (
                candidates.iter().min_by(compare).unwrap(),
                candidates.iter().max_by(compare).unwrap(),
            )
        } else {
            (first, last)
        };
        let (receiver, receiver_min, receiver_max) = *first;
        let first_origin = (first.1 + first.2) * 0.5;
        let last_origin = (last.1 + last.2) * 0.5;
        let is_receiver = |i: usize| candidates.iter().any(|(c, _, _)| *c == i);
        // Arm's local cubemap: the senders are captured once from the middle of their box and every
        // receiver reads it through box projection, so a receiver moving does not retake the scene.
        // Receivers are not in the capture (no self-image, no receiver-to-receiver reflection).
        let scene_probe = self.reflection_scene_probe;
        let mut lo = if scene_probe { glam::Vec3::INFINITY } else { receiver_min };
        let mut hi = if scene_probe { glam::Vec3::NEG_INFINITY } else { receiver_max };
        let mut has_other = false;
        for (i, input) in inputs.iter().enumerate() {
            if i == receiver || (scene_probe && is_receiver(i)) {
                continue;
            }
            if let Some((a, b)) = bounds(comp, input) {
                lo = lo.min(a);
                hi = hi.max(b);
                has_other = true;
            }
        }
        if !has_other {
            return Ok(None);
        }
        let (receivers, origins) = if scene_probe {
            (vec![usize::MAX], vec![(lo + hi) * 0.5])
        } else if first.0 == last.0 {
            (vec![receiver], vec![first_origin])
        } else {
            (vec![receiver, last.0], vec![first_origin, last_origin])
        };
        let near = ((hi - lo).length() * 1e-5).clamp(0.001, 0.01);
        #[cfg(test)]
        let near = self.reflection_diagnostic_near.unwrap_or(near);
        // Tight box projection aligns planar senders between probes. Expand only degenerate axes.
        for axis in 0..3 {
            if hi[axis] - lo[axis] < 1.0 {
                let center = (lo[axis] + hi[axis]) * 0.5;
                lo[axis] = center - 0.5;
                hi[axis] = center + 0.5;
            }
        }
        #[cfg(test)]
        let (receivers, origins) = if self.reflection_probe_experiment == 2 {
            let center = (lo + hi) * 0.5;
            let offset = glam::Vec3::X * (hi.x - lo.x) * 0.25;
            (vec![usize::MAX; 2], vec![center - offset, center + offset])
        } else {
            (receivers, origins)
        };
        let face_size = (comp.width.min(comp.height) / 2)
            .next_power_of_two()
            .clamp(64, 512);
        let resources = match self.reflection_resources.take() {
            Some(r) if r.face_size == face_size => r,
            _ => {
                let atlas = self.ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("shared-scene-reflection"),
                    size: wgpu::Extent3d {
                        width: face_size * 3,
                        height: face_size * 4,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count:
                        re_renderer::resource_managers::MipmapGenerator::mip_level_count(
                            face_size * 3,
                            face_size * 4,
                        ),
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: BLEND_TARGET_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                });
                let imported = self.import_premultiplied(&atlas)?;
                ReflectionResources {
                    face_size,
                    faces: std::array::from_fn(|_| {
                        self.create_blend_scratch_texture(face_size, face_size)
                    }),
                    atlas,
                    imported,
                }
            }
        };
        *shared = self.shared_mesh_scene(comp, inputs)?;
        let directions = [
            glam::Vec3::X,
            glam::Vec3::NEG_X,
            glam::Vec3::Y,
            glam::Vec3::NEG_Y,
            glam::Vec3::Z,
            glam::Vec3::NEG_Z,
        ];
        let ups = [
            glam::Vec3::NEG_Y,
            glam::Vec3::NEG_Y,
            glam::Vec3::Z,
            glam::Vec3::NEG_Z,
            glam::Vec3::NEG_Y,
            glam::Vec3::NEG_Y,
        ];
        for (probe, (&receiver, &origin)) in receivers.iter().zip(&origins).enumerate() {
            // Draw data is reusable across all six camera views.
            let skip = |layer: usize| if scene_probe { is_receiver(layer) } else { layer == receiver };
            let draws = self.surface_scene_draws(
                comp,
                inputs,
                Vec::new(),
                true,
                &skip,
                shared.as_ref(),
                0,
            )?;
            for face in 0..6 {
                let view = glam::Mat4::look_at_rh(origin, origin + directions[face], ups[face]);
                let rotation = glam::Quat::from_mat3(&glam::Mat3::from_mat4(view));
                let config = TargetConfiguration {
                    name: "shared-scene-reflection-face".into(),
                    render_mode: RenderMode::Deterministic,
                    resolution_in_pixel: [face_size, face_size],
                    view_from_world: macaw::IsoTransform::from_rotation_translation(
                        rotation,
                        view.w_axis.truncate(),
                    ),
                    projection_from_view: Projection::Perspective {
                        vertical_fov: std::f32::consts::FRAC_PI_2,
                        near_plane_distance: near,
                        aspect_ratio: 1.0,
                    },
                    pixels_per_point: 1.0,
                    blend_with_background: BlendWithBackground::Premultiplied,
                    environment: environment.map(|e| e.environment.clone()),
                    ..Default::default()
                };
                let mut builder = ViewBuilder::new_with_external_resolved(
                    &self.ctx,
                    config,
                    ViewBuilderId::new(self.next_readback),
                    &resources.faces[face],
                )
                .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                builder.queue_draw(&self.ctx, draws.rects.clone());
                for cloud in &draws.clouds {
                    builder.queue_draw(&self.ctx, cloud.clone());
                }
                for mesh in &draws.meshes {
                    builder.queue_draw(&self.ctx, mesh.clone());
                }
                self.pending.push(
                    builder
                        .draw(&self.ctx, Rgba::TRANSPARENT)
                        .map_err(|e| CompositorError::Draw(e.to_string()))?,
                );
                self.surface_work.scene_captures += 1;
            }
            let mut encoder =
                self.ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("shared-reflection-atlas"),
                    });
            for face in 0..6 {
                encoder.copy_texture_to_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &resources.faces[face],
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::TexelCopyTextureInfo {
                        texture: &resources.atlas,
                        mip_level: 0,
                        origin: wgpu::Origin3d {
                            x: face as u32 % 3 * face_size,
                            y: (face as u32 / 3 + probe as u32 * 2) * face_size,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::Extent3d {
                        width: face_size,
                        height: face_size,
                        depth_or_array_layers: 1,
                    },
                );
            }
            self.pending.push(encoder.finish());
        }
        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("shared-reflection-mips"),
            });
        self.ctx
            .texture_manager_2d
            .generate_mipmaps(&self.ctx, &mut encoder, &resources.atlas);
        self.pending.push(encoder.finish());
        let influence_radii = [origins[0], *origins.last().unwrap()].map(|origin| {
            candidates
                .iter()
                .map(|(_, a, b)| origin.distance((*a + *b) * 0.5) + (*b - *a).length() * 0.5)
                .fold(1.0_f32, f32::max)
        });
        #[cfg(test)]
        let influence_radii = if self.reflection_probe_experiment == 3 {
            influence_radii
        } else {
            [0.0; 2]
        };
        let result = SceneReflection {
            atlas: resources.imported.clone(),
            origins: [origins[0], *origins.last().unwrap()],
            count: origins.len() as u32,
            bounds_min: lo,
            bounds_max: hi,
            influence_radii,
        };
        #[cfg(test)]
        if self.reflection_diagnostic_enabled {
            let input_metadata: Vec<_> = inputs.iter().enumerate().map(|(index, input)| {
                let mut item = serde_json::json!({"index":index,"opacity":input.opacity,
                    "surface":input.shading.program.as_ref().is_some_and(|p|p.desc().surface.is_some()),
                    "params":input.shading.params,"bounds":bounds(comp,input).map(|(a,b)|[a.to_array(),b.to_array()])});
                if let SequentialContent::Model(model) = input.content {
                    let world = projected_spatial_placement(comp,input.projection_camera,input.projection,input.placement,model.bounds);
                    item["model_bounds"] = serde_json::json!([model.bounds.min,model.bounds.max]);
                    item["world_from_object"] = serde_json::json!(glam::Mat4::from(world).to_cols_array());
                }
                item
            }).collect();
            let metadata = serde_json::json!({"near_plane":near,"receivers":receivers,
                "origins":origins.iter().map(|p|p.to_array()).collect::<Vec<_>>(),
                "bounds_min":lo.to_array(),"bounds_max":hi.to_array(),"radii":influence_radii,
                "inputs":input_metadata,"skipped_capture_input":self.reflection_diagnostic_skip});
            self.reflection_diagnostic =
                Some(super::reflection_diagnostic::CaptureDiagnostic::enqueue(
                    &self.ctx,
                    &mut self.pending,
                    &resources.atlas,
                    metadata,
                ));
        }
        self.reflection_resources = Some(resources);
        Ok(Some(result))
    }
}

/// 2D は積み順だけで重なる(法 2026-09-12)。同じ面に重なった物の描き順を sort key の同点に
/// 委ねると process ごとに揺れた(網は mesh ごとの束ね順が不定)ので、積み順ぶんだけ面に垂直に
/// カメラ側へずらして「遠 → 近 = 下 → 上」を確定させる。1 段 0.02 px、視線に平行なので絵は動かない。
fn two_d_stack_bias(order: i16) -> f32 {
    f32::from(order.max(0)) * 0.02
}
