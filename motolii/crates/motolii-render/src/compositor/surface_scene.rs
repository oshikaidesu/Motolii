use re_renderer::renderer::{
    GpuMeshInstance, MeshDrawData, PointCloudDrawData, RectangleDrawData, RectangleOptions,
    TexturedRect,
};
use re_renderer::view_builder::{
    BlendWithBackground, Projection, RenderMode, TargetConfiguration, ViewBuilder, ViewBuilderId,
};
use re_renderer::{ClipPlane, Rgba};

use super::*;

/// 太陽から見た型紙(light cookie)の置き場。1 枚を frame ごとに描き直す。
pub(crate) struct LightCookieResources {
    texture: re_renderer::GpuTexture,
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
    ) -> Result<Option<crate::render::compositor::light::SunLight>, CompositorError> {
        if !inputs.iter().any(|i| i.shadow > 0.0) {
            return Ok(None);
        }
        // 嘘(2026-09-15): 2.5D の影は既定カメラで置いた形から落とす。見えている形はカメラごとに違うが、
        // 影までカメラで揺れると「カメラを動かしたら影が動いた」になる。Stage(既定カメラ)と出力で影は同じ。
        let flatten = inputs.iter().any(|i| i.shadow > 0.0 && i.projection == crate::doc::store::LayerProjection::TwoPointFiveD && ResolvedCamera { near_fade: 0.0, ..i.projection_camera } != ResolvedCamera::default());
        // Cast Shadow の Strength は型紙の濃さ: 遮る層をその分だけ薄く描く。
        let faded = inputs.iter().any(|i| i.shadow > 0.0 && i.shadow < 1.0);
        let placed: Vec<SequentialInput<'_>>;
        let (inputs, shared) = if flatten || faded {
            placed = inputs.iter().map(|i| SequentialInput {
                projection_camera: if flatten && i.projection == crate::doc::store::LayerProjection::TwoPointFiveD { ResolvedCamera::default() } else { i.projection_camera },
                opacity: if i.shadow > 0.0 { i.opacity * i.shadow.clamp(0.0, 1.0) } else { i.opacity },
                ..i.clone()
            }).collect();
            (&placed[..], None)
        } else {
            (inputs, shared)
        };
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
                let texture = self.picture_texture(LIGHT_COOKIE_SIZE, LIGHT_COOKIE_SIZE);
                let imported = self.import_premultiplied(&texture)?;
                LightCookieResources { texture, imported }
            }
        };
        let occludes = |layer: usize| inputs[layer].shadow > 0.0;
        let draws = self.surface_scene_draws(comp, inputs, Vec::new(), true, &|layer| !occludes(layer), shared.map(|scene| SharedSelection { scene, keep: &occludes }))?;
        let mut config = TargetConfiguration {
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
            motion: self.motion.clone(),
            ..Default::default()
        };
        super::light::light_view(&mut config, None, true);
        let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, config, ViewBuilderId::new(self.next_readback), &resources.texture.texture)
            .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        draws.queue(&self.ctx, &mut builder)?;
        self.ctx.queue_commands([builder.draw(&self.ctx, Rgba::TRANSPARENT).map_err(|e| CompositorError::Draw(e.to_string()))?]);
        self.surface_work.light_captures += 1;
        let cookie = resources.imported.clone();
        self.light_cookie = Some(resources);
        Ok(Some(crate::render::compositor::light::SunLight { direction, weight: sun.weight, color: sun.color, uv_from_world, cookie }))
    }
}

#[derive(Clone)]
pub(crate) struct SharedMeshScene {
    draw: MeshDrawData,
    source_layers: Vec<usize>,
}

impl SharedMeshScene {
    /// The instances of the layers `keep` names (by their index in the list the scene was built
    /// from), as one draw over the shared upload.
    pub(crate) fn select(&self, keep: &dyn Fn(usize) -> bool) -> MeshDrawData {
        if self.source_layers.iter().all(|&l| keep(l)) {
            return self.draw.clone();
        }
        self.draw.select_source_instances(|source| keep(self.source_layers[source]))
    }
}

/// The shared scene a drawing selects from: the scene, and which of its layers this drawing draws.
pub(crate) struct SharedSelection<'a> {
    pub scene: &'a SharedMeshScene,
    pub keep: &'a dyn Fn(usize) -> bool,
}

pub(crate) struct SceneDraws {
    rects: RectangleDrawData,
    clouds: Vec<PointCloudDrawData>,
    lines: Vec<re_renderer::renderer::LineDrawData>,
    meshes: Vec<MeshDrawData>,
}

impl SceneDraws {
    pub(crate) fn queue(&self, ctx: &re_renderer::RenderContext, view: &mut ViewBuilder) -> Result<(), CompositorError> {
        let draw = |e: re_renderer::RendererRegistrationError| CompositorError::Draw(e.to_string());
        view.queue_draw(ctx, self.rects.clone()).map_err(draw)?;
        for cloud in &self.clouds {
            view.queue_draw(ctx, cloud.clone()).map_err(draw)?;
        }
        for line in &self.lines {
            view.queue_draw(ctx, line.clone()).map_err(draw)?;
        }
        for mesh in &self.meshes {
            view.queue_draw(ctx, mesh.clone()).map_err(draw)?;
        }
        Ok(())
    }
}

pub(super) fn bounds(comp: CompSpec, input: &SequentialInput<'_>) -> Option<(glam::Vec3, glam::Vec3)> {
    let spatial = match input.content {
        // A plate's members stand for it wherever the world is looked at.
        SequentialContent::Environment(_) | SequentialContent::Plate(_) => return None,
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
    pub(crate) fn shared_mesh_scene(
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
                // Geometry, materials and instance data are shared however the surface is drawn
                // (ruling 2026-09-24): a transparent instance's order is the view's, taken from its
                // camera when the draw is queued, not from this upload. A backdrop reader stays in
                // the scene: its own run selects just its instances.
                // Opacity is instance data (the tint) and shared with it.
                if input.blend_mode != BlendMode::Normal
                    || input.clip.is_some()
                {
                    return Ok(None);
                }
                count += 1;
                instance_count = instance_count.saturating_add(model.instances.len());
            }
        }
        // One layer's copies are as shared as many layers' (every stack that draws the plate reuses
        // the upload).
        if count == 0
            || instance_count > u32::MAX as usize
            || instance_count.saturating_mul(MeshDrawData::gpu_instance_size_bytes()) as u64
                > self.ctx.device.limits().max_buffer_size
        {
            return Ok(None);
        }
        let started = std::time::Instant::now();
        let mut instances = Vec::new();
        let mut layers = Vec::new();
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
                layers.extend(std::iter::repeat_n(draw_order(input), made.len()));
                source_layers.extend(std::iter::repeat_n(index, made.len()));
                instances.extend(made);
            }
        }
        let draw = MeshDrawData::new_ordered(&self.ctx, &instances, ClipPlane::NONE, &layers)
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
    pub(crate) fn surface_scene_draws(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        mut rects: Vec<TexturedRect>,
        capture: bool,
        // Layers (index in `inputs`) left out of these draws.
        skip: &dyn Fn(usize) -> bool,
        // The frame's shared mesh upload, when these inputs' meshes are in it: they are selected
        // from it instead of being made again.
        shared: Option<SharedSelection<'_>>,
    ) -> Result<SceneDraws, CompositorError> {
        let started = std::time::Instant::now();
        let mut clouds = Vec::new();
        let mut lines = Vec::new();
        let mut mesh_groups: Vec<(ClipPlane, Vec<GpuMeshInstance>, Vec<re_renderer::renderer::DrawOrder>)> = Vec::new();
        let mut rect_layers = vec![re_renderer::renderer::DrawOrder::default(); rects.len()];
        for (index, input) in inputs.iter().enumerate() {
            if skip(index)
                || (shared.is_some() && matches!(input.content, SequentialContent::Model(_)))
            {
                continue;
            }
            let shading = input.shading.clone();
            match input.content {
                // A plate is drawn by the view's stack, at its place, never inside a run.
                SequentialContent::Environment(_) | SequentialContent::Plate(_) => {}
                SequentialContent::Cloud {
                    positions,
                    colors,
                    bounds,
                    point_size,
                    sizes,
                    sprites,
                    links,
                } => {
                    if let Some(links) = links {
                        lines.push(self.cloud_links_draw_data(links, bounds, input.placement, comp, input.projection_camera, input.projection, input.opacity)?);
                    }
                    clouds.push(self.point_cloud_draw_data(
                        positions,
                        colors,
                        bounds,
                        point_size,
                        sizes,
                        sprites,
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
                        if input.projection == crate::doc::store::LayerProjection::TwoD && !capture {
                            // 輪郭のままの文字・図形(planar な網)も同じ法: 面の法線に沿ってカメラ側へ積み順ぶん。
                            // 奥行きがずらされた変形では z 列は面の法線ではない。矩形と同じく面の 2 辺の外積。
                            let m = instance.world_from_mesh.matrix3;
                            let normal = glam::Vec3::from(m.x_axis).cross(glam::Vec3::from(m.y_axis)).normalize_or_zero();
                            instance.world_from_mesh = glam::Affine3A::from_translation(-normal * two_d_stack_bias(input.depth_offset)) * instance.world_from_mesh;
                        }
                    }
                    let layers = std::iter::repeat_n(if capture { Default::default() } else { draw_order(input) }, instances.len());
                    if let Some((_, group, keys)) = mesh_groups.iter_mut().find(|(c, _, _)| *c == clip) {
                        group.extend(instances);
                        keys.extend(layers);
                    } else {
                        mesh_groups.push((clip, instances, layers.collect()));
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
                    rect_layers.push(if capture { Default::default() } else { draw_order(input) });
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
                            depth_offset: if capture { 0 } else { input.depth_offset.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16 },
                            clip: input
                                .clip
                                .map_or(ClipPlane::NONE, |c| c.world_for_rect(corner, u, v)),
                            subdivisions: shading.field_grid(),
                            surface: shading.program.or_else(|| self.standard_surface_program()),
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
            .map(|(_, instances, _)| instances.len())
            .sum();
        self.surface_work.mesh_instances_uploaded += uploaded as u64;
        self.surface_work.mesh_instance_upload_bytes +=
            (uploaded * MeshDrawData::gpu_instance_size_bytes()) as u64;
        let mut meshes: Vec<MeshDrawData> = mesh_groups
            .into_iter()
            .map(|(clip, instances, layers)| {
                MeshDrawData::new_ordered(&self.ctx, &instances, clip, &layers)
                    .map_err(|e| CompositorError::Draw(e.to_string()))
            })
            .collect::<Result<_, _>>()?;
        if let Some(shared) = shared {
            meshes.push(shared.scene.select(shared.keep));
        }
        self.surface_work.draw_data_prepare_us += started.elapsed().as_micros() as u64;
        Ok(SceneDraws {
            rects: RectangleDrawData::new_ordered(&self.ctx, &rects, &rect_layers)
                .map_err(|e| CompositorError::Rectangles(e.to_string()))?,
            clouds,
            lines,
            meshes,
        })
    }
}

impl Compositor {
}

/// 2D は積み順だけで重なる(法 2026-09-12)。同じ面に重なった物の描き順を sort key の同点に
/// 委ねると process ごとに揺れた(網は mesh ごとの束ね順が不定)ので、積み順ぶんだけ面に垂直に
/// カメラ側へずらして「遠 → 近 = 下 → 上」を確定させる。1 段 0.02 px、視線に平行なので絵は動かない。
/// 積み順を描き順へ。透明相の並べ替えは camera からの距離が先なので、面をずらすだけでは中心が横にずれた物同士
/// (帯と端の丸)の上下が距離で決まった。2D は積み順を距離より先の鍵に、並べる Group の面に乗る物は面の基準点で
/// 距離を測り積み順で重なる(別の面・浮いた物とは距離で並ぶ)。
fn draw_order(input: &SequentialInput<'_>) -> re_renderer::renderer::DrawOrder {
    use re_renderer::renderer::DrawOrder;
    if input.projection == crate::doc::store::LayerProjection::TwoD {
        return DrawOrder { layer: i32::from(input.depth_offset), ..Default::default() };
    }
    match input.placement.plane {
        Some(point) => DrawOrder { position: Some(glam::Vec3A::from(point)), secondary: Some(input.depth_offset as f32), ..Default::default() },
        None => DrawOrder::default(),
    }
}

fn two_d_stack_bias(order: i32) -> f32 {
    order.max(0) as f32 * 0.02
}
