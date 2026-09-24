//! One view's picture, composed in the work's order (Motolii's ordered compositing), recorded into
//! the tick's command list. Everything here reads the view's window and camera as arguments and
//! draws on canvases from re_renderer's texture pool; nothing is submitted here and no state is
//! left behind for the next view.
//!
//! The work is stacked bottom to top. Layers that can share one draw (a run) are drawn together
//! by one `ViewBuilder` on a transparent canvas and laid over what is below; a picture with a mix
//! blend mode is drawn alone and mixed onto what is below. The bottom canvas starts from the
//! composition's background. At the end the stack is one picture, which the view's own
//! `ViewBuilder` shows (and the tick composites into the surface).

use re_renderer::renderer::{RectangleDrawData, TexturedRect};
use re_renderer::view_builder::ViewBuilder;
use re_renderer::{Rgba, ViewBuilderId};

use super::*;

fn view_owner(input: &SequentialInput<'_>) -> Option<u64> {
    input.shading.views.as_ref().map(|v| v.owner)
}

/// How many of the view's pixels one of the layer's (composition) pixels covers where the view
/// sees it: its size in this view over its size in the work's output. A Camera or Stage view gives
/// its window's zoom; another eye, how large the layer looks to it. 1 when it cannot be seen.
fn pass_density(comp: CompSpec, window: Window, observer: Observer, input: &SequentialInput<'_>) -> f32 {
    let (mut lo, mut hi, mut any) = (glam::Vec3::INFINITY, glam::Vec3::NEG_INFINITY, false);
    let mut add = |input: &SequentialInput<'_>| {
        if let Some((a, b)) = super::surface_scene::bounds(comp, input) {
            lo = lo.min(a);
            hi = hi.max(b);
            any = true;
        }
    };
    visit_inputs(std::slice::from_ref(input), &mut add);
    if !any { return 1.0; }
    let corners: Vec<glam::Vec3> = (0..8).map(|i| glam::vec3(
        if i & 1 == 0 { lo.x } else { hi.x }, if i & 2 == 0 { lo.y } else { hi.y }, if i & 4 == 0 { lo.z } else { hi.z },
    )).collect();
    // The extent on screen (comp px scaled to the window) of the corners in front of the eye.
    let extent = |projection: crate::doc::core::CameraProjection, pixels: [f32; 2]| {
        let clip_from_world = projection.projection_matrix() * projection.view_matrix();
        let (mut a, mut b, mut seen) = (glam::Vec2::INFINITY, glam::Vec2::NEG_INFINITY, false);
        for corner in &corners {
            let clip = clip_from_world * corner.extend(1.0);
            if clip.w <= 1e-6 { continue; }
            let p = glam::vec2(clip.x, clip.y) / clip.w * glam::Vec2::from(pixels) * 0.5;
            a = a.min(p);
            b = b.max(p);
            seen = true;
        }
        seen.then(|| (b - a).length())
    };
    let zoom = [window.width as f32 / window.roi[2].max(1.0), window.height as f32 / window.roi[3].max(1.0)];
    let here = extent(observer.projection, [comp.width as f32 * zoom[0], comp.height as f32 * zoom[1]]);
    let output = extent(Observer::camera(comp, input.projection_camera).projection, [comp.width as f32, comp.height as f32]);
    match (here, output) {
        (Some(here), Some(output)) if here > 1e-3 && output > 1e-3 => (here / output).clamp(1.0 / 64.0, 64.0),
        _ => 1.0,
    }
}

/// Every input, and every member of a plate the views materialize, in order.
pub(super) fn visit_inputs(inputs: &[SequentialInput<'_>], f: &mut dyn FnMut(&SequentialInput<'_>)) {
    for input in inputs {
        if let SequentialContent::Plate(plate) = input.content {
            if let Some(members) = plate.prepared.get() {
                let member_inputs = sequential_inputs(&plate.sources, &members.pictures, &members.paddings, &members.spills, plate.camera, plate.camera);
                visit_inputs(&member_inputs, f);
            }
            continue;
        }
        f(input);
    }
}

/// What ended a run (the layers one `ViewBuilder` draws together) and began the next. Each is a
/// reading of the work — which picture a layer sees below it, or in what order it stacks — or a
/// binding a draw can hold only one of; none is a cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub(crate) enum RunBreak {
    /// The last run of the stack.
    End = 0,
    /// A picture with a mix blend is drawn alone: its blend reads what is below it.
    Alone,
    /// A plate the view materializes is a stack of its own, at its place.
    Plate,
    /// 2D is not in the world (2026-09-12): a run is all 2D or all not.
    Flat,
    /// Glass reads the non-glass picture, which the non-glass runs build (2026-09-23).
    Glass,
    /// A mesh after a picture.
    MeshAfterRect,
    /// A layer whose effects run on the view's picture sees only itself.
    ScreenPasses,
    /// A run binds one layer's Views.
    ViewOwner,
}
pub(crate) const RUN_BREAKS: usize = 8;

pub(crate) struct PlannedRun {
    pub range: std::ops::Range<usize>,
    /// The blend onto the stack (`vism/blend.wgsl`'s numbering).
    pub mode: u32,
    pub ended_by: RunBreak,
}

/// The runs of a stack, in order: which layers each `ViewBuilder` draws together and why the
/// next one starts. Pure: the same inputs always plan the same runs.
pub(crate) fn plan_runs(inputs: &[SequentialInput<'_>]) -> Vec<PlannedRun> {
    let flat = |input: &SequentialInput<'_>| input.projection == crate::doc::store::LayerProjection::TwoD;
    let alone = |input: &SequentialInput<'_>| {
        matches!(input.content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_)) && vello_blend_mode(input.blend_mode).is_some()
    };
    let mut out = Vec::new();
    let mut index = 0;
    while index < inputs.len() {
        let start = index;
        if matches!(inputs[start].content, SequentialContent::Plate(_)) {
            out.push(PlannedRun { range: start..start + 1, mode: SRC_OVER, ended_by: RunBreak::Plate });
            index += 1;
            continue;
        }
        if let Some(mode) = vello_blend_mode(inputs[start].blend_mode).filter(|_| alone(&inputs[start])) {
            out.push(PlannedRun { range: start..start + 1, mode, ended_by: RunBreak::Alone });
            index += 1;
            continue;
        }
        let mut has_rect = false;
        let mut ended_by = RunBreak::End;
        while index < inputs.len() {
            let input = &inputs[index];
            let reason = if index == start {
                None
            } else if alone(input) {
                Some(RunBreak::Alone)
            } else if matches!(input.content, SequentialContent::Plate(_)) {
                Some(RunBreak::Plate)
            } else if flat(input) != flat(&inputs[start]) {
                Some(RunBreak::Flat)
            } else if input.shading.reads_backdrop != inputs[start].shading.reads_backdrop {
                Some(RunBreak::Glass)
            } else if has_rect && matches!(input.content, SequentialContent::Model(_)) {
                Some(RunBreak::MeshAfterRect)
            } else if !input.screen_passes.is_empty() {
                Some(RunBreak::ScreenPasses)
            } else if view_owner(input) != view_owner(&inputs[start]) {
                Some(RunBreak::ViewOwner)
            } else {
                None
            };
            if let Some(reason) = reason {
                ended_by = reason;
                break;
            }
            has_rect |= matches!(input.content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_));
            index += 1;
            if !input.screen_passes.is_empty() {
                ended_by = RunBreak::ScreenPasses;
                break;
            }
        }
        out.push(PlannedRun { range: start..index, mode: SRC_OVER, ended_by });
    }
    if let Some(last) = out.last_mut() {
        last.ended_by = RunBreak::End;
    }
    out
}

/// Mip levels a surface reads from a backdrop of `level_count` levels when its roughness is at
/// most `max_roughness`: the twin of `vism/material.wgsl`'s lod (`pow(r, 0.8) * (levels - 1) *
/// 0.55`, and trilinear reads the level above), so no level nobody samples is generated. The
/// curve is Motolii's standard material's, so it lives here and not in the host.
pub(super) fn backdrop_levels_read(max_roughness: f32, level_count: u32) -> u32 {
    let lod = max_roughness.clamp(0.0, 1.0).powf(0.8) * level_count.max(1).saturating_sub(1) as f32 * 0.55;
    (lod.ceil() as u32 + 1).clamp(1, level_count.max(1))
}

/// The Normal blend (Porter-Duff source-over) in `vism/blend.wgsl`'s numbering.
const SRC_OVER: u32 = 3;
/// What is below stays on top (the sky under what is drawn).
const DEST_OVER: u32 = 4;

/// Where a view looks from. A Camera or Stage view, and a picture a preparation makes, look from a
/// camera of the work; a View a Vism asks for looks from its layer. What the view draws — the
/// work's layers in order, placed where the work places them — does not depend on the eye.
#[derive(Clone, Copy)]
pub(crate) struct Observer {
    pub projection: crate::doc::core::CameraProjection,
    /// The work's camera fades what is near it; another eye fades nothing.
    pub near_fade: f32,
    /// A View a layer asked for (its draws are named so in GPU traces).
    pub requested: bool,
}

impl Observer {
    pub(crate) fn camera(comp: CompSpec, camera: ResolvedCamera) -> Self {
        Self { projection: crate::doc::core::camera_projection(comp, camera), near_fade: camera.near_fade, requested: false }
    }
}

/// The world's light for one frame: the sun's occluder map.
#[derive(Clone, Default)]
pub(crate) struct WorldLight {
    pub light: Option<crate::render::compositor::light::SunLight>,
    /// The mesh instances the capture uploaded, when its scene is the frame's own layer list (no
    /// plate members added): the views place them the same way and draw them as they are.
    pub meshes: Option<super::surface_scene::SharedMeshScene>,
    /// Which frame-level capture this is (0: none, unlit).
    pub serial: u64,
}

/// What a view reads from the prepared world.
pub(crate) struct ViewWorld<'a> {
    pub environment: Option<&'a GpuEnvironmentData>,
    pub motion: Option<&'a re_renderer::DataTexture>,
    pub light: Option<&'a crate::render::compositor::light::SunLight>,
    pub views: &'a [crate::render::compositor::light::LayerViews],
    /// The world's mesh instances, for a view that places the layers as the world does.
    pub meshes: Option<&'a super::surface_scene::SharedMeshScene>,
}

impl Compositor {
    /// Records one view: every layer in order, stacked into one picture, shown by the returned
    /// `ViewBuilder` (already drawn into `encoder`; the caller composites it into its surface).
    pub(crate) fn record_view(
        &mut self,
        comp: CompSpec,
        window: Window,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
        background_color: [f32; 4],
        world: &ViewWorld<'_>,
        view: u32,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<ViewBuilder, CompositorError> {
        let stack = self.record_stack(comp, window, Observer::camera(comp, camera), inputs, background_color, world, view, None, None, encoder)?;
        // The view shows the stack (or, with nothing drawn, the background).
        let mut shown = ViewBuilder::new(&self.ctx, screen_target_config("motolii-view", window), ViewBuilderId::new(self.next_readback))
            .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        let clear = match &stack {
            Some(picture) => {
                let imported = self.import_premultiplied(&picture)?;
                let rects: Vec<TexturedRect> = vec![screen_rect(window, imported)];
                shown.queue_draw(&self.ctx, RectangleDrawData::new(&self.ctx, &rects).map_err(|e| CompositorError::Rectangles(e.to_string()))?)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;
                Rgba::TRANSPARENT
            }
            None => clear_color(background_color),
        };
        shown.draw_into(&self.ctx, clear, encoder).map_err(|e| CompositorError::Draw(e.to_string()))?;
        Ok(shown)
    }

    /// A picture of `inputs` made inside a preparation (a plate, a matte's source, a clip group, an
    /// image input): the same ordered composition as a view, into a texture of its own that the
    /// prepared frame keeps (not the pool's: it outlives the tick).
    pub(crate) fn record_picture(
        &mut self,
        comp: CompSpec,
        window: Window,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
        background_color: [f32; 4],
        world: &ViewWorld<'_>,
        into: Option<&re_renderer::GpuTexture>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<re_renderer::GpuTexture, CompositorError> {
        let picture = into.cloned().unwrap_or_else(|| self.picture_texture(window.width, window.height));
        // A picture is a drawing of its own (view 0): its histories are not a view's.
        match self.record_stack(comp, window, Observer::camera(comp, camera), inputs, background_color, world, 0, None, None, encoder)? {
            Some(stack) => {
                let size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
                encoder.copy_texture_to_texture(stack.texture.as_image_copy(), picture.texture.as_image_copy(), size);
            }
            None => {
                let view = picture.default_view.clone();
                let c = clear_color(background_color);
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("motolii-picture-background"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view, depth_slice: None, resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: c.r() as f64, g: c.g() as f64, b: c.b() as f64, a: c.a() as f64 }), store: wgpu::StoreOp::Store },
                    })],
                    depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
                });
            }
        }
        Ok(picture)
    }

    /// Every layer in order, stacked bottom to top; `None` when nothing is drawn. `below` is the
    /// view's picture beneath these layers when they are a plate's members: glass among them
    /// refracts it (with the members' own non-glass picture over it). `without` leaves out the
    /// copies of the layer whose View this is, wherever they stand (a plate's members too).
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_stack(
        &mut self,
        comp: CompSpec,
        window: Window,
        observer: Observer,
        inputs: &[SequentialInput<'_>],
        background_color: [f32; 4],
        world: &ViewWorld<'_>,
        view: u32,
        below: Option<&re_renderer::GpuTexture>,
        without: Option<u64>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<Option<re_renderer::GpuTexture>, CompositorError> {
        // The work's views show a plate as its camera took it, when it was taken; another eye draws
        // the members. A View leaves out the copies of the layer that asked for it.
        let pictured = |input: &SequentialInput<'_>| !observer.requested && matches!(input.content, SequentialContent::Plate(plate) if plate.camera_picture.is_some());
        let kept: Vec<SequentialInput<'_>>;
        let inputs = if without.is_some() || inputs.iter().any(pictured) {
            kept = inputs.iter().filter(|input| without.is_none() || view_owner(input) != without).map(|input| match input.content {
                SequentialContent::Plate(plate) if pictured(input) => SequentialInput { content: SequentialContent::Rect(plate.camera_picture.as_ref().expect("pictured")), ..input.clone() },
                _ => input.clone(),
            }).collect();
            kept.as_slice()
        } else {
            inputs
        };
        // The light is the composition's: the top environment layer, else the world's.
        let environment = inputs.iter().rev().find_map(|input| match input.content {
            SequentialContent::Environment(e) => Some(e),
            _ => None,
        }).or(world.environment);
        let projection = observer.projection;
        let view_from_world = macaw::IsoTransform::from_rotation_translation(projection.rotation, -(projection.rotation * projection.eye));
        let flat = |input: &SequentialInput<'_>| input.projection == crate::doc::store::LayerProjection::TwoD;
        let alone = |input: &SequentialInput<'_>| {
            matches!(input.content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_)) && vello_blend_mode(input.blend_mode).is_some()
        };

        let mut stack: Option<re_renderer::GpuTexture> = None;
        // Standard glass refracts the non-glass picture below it and never another glass (2026-09-23;
        // three.js / Godot / Filament: one transmission input from the opaque scene, shared by every
        // transmissive object). Until the first glass the non-glass picture is the stack itself;
        // after it, non-glass runs are laid on this one too.
        let mut unglazed: Option<re_renderer::GpuTexture> = None;
        let mut glazed = false;
        // The transmission input: one copy of the non-glass picture, reused by every glass until
        // something non-glass is added. Its mips reach as far as the roughest glass of the view reads.
        let mut transmission: Option<GpuTexture2D> = None;
        let roughest = inputs.iter().filter(|input| input.shading.reads_backdrop).map(|input| input.shading.backdrop_roughness).fold(0.0f32, f32::max);
        // Backdrops a recorded draw still reads; kept until the view is recorded.
        let mut backdrops = Vec::new();
        let planned_runs = plan_runs(inputs);
        for (at, planned) in planned_runs.iter().enumerate() {
            self.surface_work.run_breaks[planned.ended_by as usize] += 1;
            let start = planned.range.start;
            // Whether a later run reads the non-glass picture below it (glass, or a plate).
            let later_reads_below = planned_runs[at + 1..].iter().any(|later| {
                let first = &inputs[later.range.start];
                first.shading.reads_backdrop || matches!(first.content, SequentialContent::Plate(_))
            });
            // A plate the view materializes: its members drawn here, over the view's picture below
            // the plate (what its glass refracts), then the plate's effects, then onto the stack.
            if let SequentialContent::Plate(plate) = inputs[start].content {
                let input = &inputs[start];
                let Some(members) = plate.prepared.get() else { continue };
                let member_inputs = sequential_inputs(&plate.sources, &members.pictures, &members.paddings, &members.spills, plate.camera, plate.camera);
                let beneath = self.non_glass_below(window, below, if glazed { unglazed.as_ref() } else { stack.as_ref() }, encoder);
                // The world's shared mesh instances are indexed by the top-level list, not the members'.
                let members_world = ViewWorld { environment: world.environment, motion: world.motion, light: world.light, views: world.views, meshes: None };
                let Some(mut canvas) = self.record_stack(comp, window, observer, &member_inputs, NO_BACKGROUND, &members_world, view, beneath.as_ref(), without, encoder)? else { continue };
                if !input.screen_passes.is_empty() {
                    let density = pass_density(comp, window, observer, input);
                    canvas = self.screen_passes(window, view, density, canvas, input.screen_passes, input.screen_sources, stack.as_ref(), encoder)?;
                }
                // The plate holds glass: what is above refracts the picture below it, as for a glass run.
                if !glazed {
                    unglazed = stack.clone();
                    glazed = true;
                }
                let mode = vello_blend_mode(input.blend_mode).unwrap_or(SRC_OVER);
                stack = Some(match stack.take() {
                    None => canvas,
                    Some(beneath) => self.mix_onto(window, &beneath, &canvas, mode, encoder),
                });
                continue;
            }
            let mode = planned.mode;
            let run = &inputs[planned.range.clone()];

            // The sky is the ground: the run holding the top environment lays it under the stack.
            if run.iter().any(|input| matches!(input.content, SequentialContent::Environment(e) if environment.is_some_and(|top| std::ptr::eq(top, e)))) {
                let sky = self.view_canvas(window);
                let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, sequential_target_config(if observer.requested { "layer-view-sky" } else { "motolii-view-sky" }, comp, window, view_from_world, projection, environment), ViewBuilderId::new(self.next_readback), &sky.texture)
                    .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                let sky_data = re_renderer::renderer::GenericSkyboxDrawData::new(&self.ctx, re_renderer::renderer::GenericSkyboxType::Environment)
                    .map_err(|e| CompositorError::Draw(e.to_string()))?;
                builder.queue_draw(&self.ctx, sky_data).map_err(|e| CompositorError::Draw(e.to_string()))?;
                builder.draw_into(&self.ctx, Rgba::TRANSPARENT, encoder).map_err(|e| CompositorError::Draw(e.to_string()))?;
                if glazed {
                    unglazed = unglazed.take().map(|below| self.mix_onto(window, &below, &sky, DEST_OVER, encoder));
                    transmission = None;
                }
                stack = Some(match stack.take() {
                    None => sky,
                    Some(below) => self.mix_onto(window, &below, &sky, DEST_OVER, encoder),
                });
            }
            // An environment alone draws nothing of its own (the sky above is its picture): no
            // builder, no mix.
            if run.iter().all(|input| matches!(input.content, SequentialContent::Environment(_)) && input.screen_passes.is_empty()) {
                continue;
            }
            let glass_run = run[0].shading.reads_backdrop;

            let mut config = sequential_target_config(if observer.requested { "layer-view-run" } else { "motolii-view-run" }, comp, window, view_from_world, projection, environment);
            config.motion = world.motion.cloned();
            super::light::light_view(&mut config, world.light, false);
            if let Some(views) = run[0].shading.views.as_ref().and_then(|needs| world.views.iter().find(|v| v.owner == needs.owner)) {
                super::light::bind_views(&mut config, views);
            }
            if glass_run {
                if transmission.is_none() {
                    let beneath = self.non_glass_below(window, below, if glazed { unglazed.as_ref() } else { stack.as_ref() }, encoder);
                    if let Some(beneath) = beneath {
                        let (texture, imported) = self.backdrop(window, &beneath, roughest, encoder)?;
                        transmission = Some(imported);
                        backdrops.push(texture);
                    }
                }
                config.backdrop = transmission.clone();
            }
            // Near things fade: the work's camera fades (an observer's has no fade), only what is in the world.
            if !flat(&run[0]) {
                config.near_fade_distance = observer.near_fade;
            }
            self.surface_work.main_runs += 1;
            let setup_started = std::time::Instant::now();
            let canvas = self.view_canvas(window);
            let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, config, ViewBuilderId::new(self.next_readback), &canvas.texture)
                .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;
            self.surface_work.run_setup_us += setup_started.elapsed().as_micros() as u64;
            let record_started = std::time::Instant::now();
            // A picture drawn alone for a mix blend is drawn plainly; its blend is the mix onto the stack.
            let plain: Vec<SequentialInput<'_>>;
            let drawn = if mode != SRC_OVER || alone(&run[0]) {
                plain = run.iter().map(|input| SequentialInput { blend_mode: BlendMode::Normal, ..input.clone() }).collect();
                plain.as_slice()
            } else {
                run
            };
            self.surface_scene_draws(comp, drawn, Vec::new(), false, &|_| false, world.meshes, start)?.queue(&self.ctx, &mut builder)?;
            // The bottom of the stack starts from the background; everything above is drawn on clear.
            let clear = if stack.is_none() { clear_color(background_color) } else { Rgba::TRANSPARENT };
            builder.draw_into(&self.ctx, clear, encoder).map_err(|e| CompositorError::Draw(e.to_string()))?;
            self.surface_work.run_record_us += record_started.elapsed().as_micros() as u64;
            let canvas = match run {
                [only] if !only.screen_passes.is_empty() => {
                    let density = pass_density(comp, window, observer, only);
                    self.screen_passes(window, view, density, canvas, only.screen_passes, only.screen_sources, stack.as_ref(), encoder)?
                }
                _ => canvas,
            };
            if glass_run && !glazed {
                // The picture before the first glass is the non-glass picture from here on.
                unglazed = stack.clone();
                glazed = true;
            } else if !glass_run && glazed {
                // The non-glass picture is kept only while something later reads it.
                unglazed = if later_reads_below {
                    Some(match unglazed.take() {
                        None => canvas.clone(),
                        Some(below) => self.mix_onto(window, &below, &canvas, mode, encoder),
                    })
                } else {
                    None
                };
                transmission = None;
            }
            stack = Some(match stack.take() {
                None => canvas,
                Some(below) => self.mix_onto(window, &below, &canvas, mode, encoder),
            });
        }

        drop(backdrops);
        Ok(stack)
    }

    /// A layer's effects run on its drawn canvas (it had no picture of its own to bake them into, or
    /// they read the view's picture below): the view's work, at the view's size.
    #[allow(clippy::too_many_arguments)]
    fn screen_passes(
        &mut self,
        window: Window,
        view: u32,
        density: f32,
        canvas: re_renderer::GpuTexture,
        passes: &[EffectPass],
        sources: &[Vec<GpuTexture2D>],
        below: Option<&re_renderer::GpuTexture>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<re_renderer::GpuTexture, CompositorError> {
        let (width, height) = (window.width, window.height);
        // A pass reading what is below gets the view's stack so far (nothing yet: transparent);
        // another reads the pictures the frame prepared for it (another time's composite).
        let below: Option<re_renderer::GpuTexture> = passes.iter().any(|p| p.reads_backdrop).then(|| match below {
            Some(below) => below.clone(),
            None => self.ctx.texture_manager_2d.zeroed_texture_float().clone(),
        });
        let others: Vec<Vec<re_renderer::GpuTexture>> = passes.iter().enumerate().map(|(i, p)| match (&below, p.reads_backdrop) {
            (Some(b), true) => vec![b.clone()],
            _ => sources.get(i).map(|row| row.iter().filter_map(|t| self.ctx.gpu_resources.textures.get_from_handle(t.handle()).ok()).collect()).unwrap_or_default(),
        }).collect();
        // The effects' lengths are the layer's (2026-09-13): the canvas's pixels per layer px here.
        let frame = effects::vism::ImageFrame { size: [width as f32 / density, height as f32 / density], origin: [0.0; 2], pixels: [width, height] };
        let (mut current, linear, premultiplied) = self.record_pass_chain(
            encoder, canvas.clone(), true, true, passes, &others, Some(frame), [width, height], 0, [width, height], Some([view, width, height]),
        )?;
        if !linear {
            current = self.convert_image_encoding(encoder, &current, true, true, premultiplied);
        }
        // The effects' result replaces the canvas: as it is when it is already in the canvas's
        // format, else copied into one (compose 1 = copy).
        if current.texture.format() == canvas.texture.format() {
            return Ok(current);
        }
        const COPY: u32 = 1;
        let out = self.view_canvas(window);
        {
            let Self { ctx, blend_vism, .. } = self;
            blend_vism.get(ctx).record_over(ctx, encoder, &[&canvas, &current], &out.default_view, &[("mode".to_owned(), COPY as f32)], window.size_f32());
        }
        Ok(out)
    }

    /// The non-glass picture glass refracts: this stack's own (`local`) over the view's picture
    /// beneath it (`below`, when the stack is a plate's members).
    fn non_glass_below(
        &mut self,
        window: Window,
        below: Option<&re_renderer::GpuTexture>,
        local: Option<&re_renderer::GpuTexture>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Option<re_renderer::GpuTexture> {
        match (below, local) {
            (Some(below), Some(local)) => Some(self.mix_onto(window, below, local, SRC_OVER, encoder)),
            (Some(only), None) | (None, Some(only)) => Some(only.clone()),
            (None, None) => None,
        }
    }

    /// What is below, copied with the mip levels the roughest reader needs (glass blurs by reading
    /// coarser levels): the view's own picture, so it is the view's, from the pool.
    fn backdrop(&mut self, window: Window, below: &re_renderer::GpuTexture, roughness: f32, encoder: &mut wgpu::CommandEncoder) -> Result<(re_renderer::GpuTexture, GpuTexture2D), CompositorError> {
        self.surface_work.backdrop_copies += 1;
        let size = wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 };
        let pyramid = self.ctx.gpu_resources.textures.alloc(&self.ctx.device, &re_renderer::TextureDesc {
            label: "motolii-view-backdrop".into(),
            size,
            mip_level_count: re_renderer::resource_managers::MipmapGenerator::mip_level_count(window.width, window.height),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
        });
        let level0 = |texture| wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All };
        encoder.copy_texture_to_texture(level0(&below.texture), level0(&pyramid.texture), size);
        let levels = backdrop_levels_read(roughness, pyramid.texture.mip_level_count());
        self.surface_work.backdrop_mip_levels += u64::from(levels);
        self.ctx.texture_manager_2d.generate_mipmap_levels(&self.ctx, encoder, &pyramid.texture, levels);
        let imported = self.import_premultiplied(&pyramid)?;
        Ok((pyramid, imported))
    }

    /// The world's light, captured once per document frame from the world's layers (placed as the
    /// output places them): the sun's occluder map, and the mesh instances the views share.
    pub(crate) fn capture_world_light(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
    ) -> Result<(Option<crate::render::compositor::light::SunLight>, Option<super::surface_scene::SharedMeshScene>), CompositorError> {
        let environment = inputs.iter().rev().find_map(|input| match input.content {
            SequentialContent::Environment(e) => Some(e),
            _ => None,
        }).or(environment);
        // The meshes' instances, placed as the output places them, uploaded once for the frame.
        let shared = self.shared_mesh_scene(comp, inputs)?;
        let light = self.capture_light_cookie(comp, inputs, environment, shared.as_ref())?;
        Ok((light, shared))
    }

    /// A canvas the size of the view's window, from re_renderer's pool: it returns to the pool when
    /// the tick lets it go, and re_renderer retires it at a frame boundary if nothing reuses it.
    fn view_canvas(&self, window: Window) -> re_renderer::GpuTexture {
        self.view_canvas_for(window, BLEND_TARGET_FORMAT)
    }

    /// [`Self::view_canvas`] in `format`.
    pub(crate) fn view_canvas_for(&self, window: Window, format: wgpu::TextureFormat) -> re_renderer::GpuTexture {
        effects::vism::pass_texture(&self.ctx, window.width, window.height, format)
    }

    /// `above` mixed onto `below` by `mode`, into a new canvas.
    fn mix_onto(
        &mut self,
        window: Window,
        below: &re_renderer::GpuTexture,
        above: &re_renderer::GpuTexture,
        mode: u32,
        encoder: &mut wgpu::CommandEncoder,
    ) -> re_renderer::GpuTexture {
        let started = std::time::Instant::now();
        let out = self.view_canvas(window);
        let Self { ctx, blend_vism, surface_work, .. } = self;
        blend_vism.get(ctx).record_over(ctx, encoder, &[below, above], &out.default_view, &[("mode".to_owned(), mode as f32)], window.size_f32());
        surface_work.run_mix_us += started.elapsed().as_micros() as u64;
        out
    }
}
