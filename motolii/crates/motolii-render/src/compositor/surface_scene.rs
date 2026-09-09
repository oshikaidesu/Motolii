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

pub(super) struct SharedMeshScene {
    draw: MeshDrawData,
    source_layers: Vec<usize>,
}

impl SharedMeshScene {
    fn select(&self, range: std::ops::Range<usize>, skip: Option<usize>) -> MeshDrawData {
        if skip.is_none()
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
            range.contains(&layer) && skip != Some(layer)
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
        SequentialContent::Rect(_) => None,
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
        skip: Option<usize>,
        shared: Option<&SharedMeshScene>,
        input_offset: usize,
    ) -> Result<SceneDraws, CompositorError> {
        let started = std::time::Instant::now();
        let mut clouds = Vec::new();
        let mut mesh_groups: Vec<(ClipPlane, Vec<GpuMeshInstance>)> = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            if skip == Some(index)
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
                    )?);
                }
                SequentialContent::Model(model) => {
                    let (instances, clip) = self.model_instances(
                        model,
                        input.placement,
                        input.opacity,
                        comp,
                        input.projection_camera,
                        input.projection,
                        &shading,
                        input.clip,
                    );
                    if let Some((_, group)) = mesh_groups.iter_mut().find(|(c, _)| *c == clip) {
                        group.extend(instances);
                    } else {
                        mesh_groups.push((clip, instances));
                    }
                }
                SequentialContent::Rect(texture) => {
                    let (corner, u, v) = projected_placement_corners(
                        comp,
                        input.projection_camera,
                        input.projection,
                        input.placement,
                        input.local_min,
                        input.local_size,
                    );
                    let alpha = if input.blend_mode == BlendMode::Add {
                        0.0
                    } else {
                        input.opacity
                    };
                    rects.push(TexturedRect {
                        top_left_corner_position: corner,
                        extent_u: u,
                        extent_v: v,
                        colormapped_texture: premultiplied_texture(texture.clone()),
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
                            surface: shading.program,
                            surface_params: shading.params,
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
            meshes.push(shared.select(
                input_offset..input_offset + inputs.len(),
                skip.map(|i| input_offset + i),
            ));
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
        let (receivers, origins) = if first.0 == last.0 {
            (vec![receiver], vec![first_origin])
        } else {
            (vec![receiver, last.0], vec![first_origin, last_origin])
        };
        let mut lo = receiver_min;
        let mut hi = receiver_max;
        let mut has_other = false;
        for (i, input) in inputs.iter().enumerate() {
            if i == receiver {
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
        let near = ((hi - lo).length() * 1e-5).clamp(0.001, 0.01);
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
            let draws = self.surface_scene_draws(
                comp,
                inputs,
                Vec::new(),
                true,
                Some(receiver),
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
        self.reflection_resources = Some(resources);
        Ok(Some(result))
    }
}
