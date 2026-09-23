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

/// The Normal blend (Porter-Duff source-over) in `vism/blend.wgsl`'s numbering.
const SRC_OVER: u32 = 3;
/// What is below stays on top (the sky under what is drawn).
const DEST_OVER: u32 = 4;

/// What a view reads from the prepared world.
pub(crate) struct ViewWorld<'a> {
    pub environment: Option<&'a GpuEnvironmentData>,
    pub motion: Option<&'a re_renderer::MotionBuffer>,
    pub reflection: Option<&'a re_renderer::environment::SceneReflection>,
    pub light: Option<&'a re_renderer::environment::SunLight>,
}

impl Compositor {
    /// Records one view: every layer in order, stacked into one picture, shown by the returned
    /// `ViewBuilder` (already drawn into `commands`; the caller composites it into its surface).
    pub(crate) fn record_view(
        &mut self,
        comp: CompSpec,
        window: Window,
        camera: ResolvedCamera,
        inputs: &[SequentialInput<'_>],
        background_color: [f32; 4],
        world: &ViewWorld<'_>,
        commands: &mut Vec<wgpu::CommandBuffer>,
    ) -> Result<ViewBuilder, CompositorError> {
        // The light is the composition's: the top environment layer, else the world's.
        let environment = inputs.iter().rev().find_map(|input| match input.content {
            SequentialContent::Environment(e) => Some(e),
            _ => None,
        }).or(world.environment);
        let projection = crate::doc::core::camera_projection(comp, camera);
        let view_from_world = macaw::IsoTransform::from_rotation_translation(projection.rotation, -(projection.rotation * projection.eye));
        let flat = |input: &SequentialInput<'_>| input.projection == crate::doc::store::LayerProjection::TwoD;
        let alone = |input: &SequentialInput<'_>| {
            matches!(input.content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_)) && vello_blend_mode(input.blend_mode).is_some()
        };

        let mut stack: Option<re_renderer::GpuTexture> = None;
        // Backdrops a recorded draw still reads; kept until the view is recorded.
        let mut backdrops = Vec::new();
        let mut index = 0;
        while index < inputs.len() {
            // A mix blend reads what is below: the picture is drawn alone, then mixed onto the stack.
            let (start, mode) = if let Some(mode) = vello_blend_mode(inputs[index].blend_mode).filter(|_| alone(&inputs[index])) {
                index += 1;
                (index - 1, mode)
            } else {
                // 2D is not in the world (2026-09-12): a run is all 2D or all not, and runs stack in order.
                let start = index;
                let mut has_rect = false;
                while index < inputs.len() && !alone(&inputs[index]) {
                    if index > start && flat(&inputs[index]) != flat(&inputs[start]) {
                        break;
                    }
                    // A surface reading what is behind it starts a run: everything before is its backdrop.
                    // A mesh after a picture starts one too.
                    if index > start && (inputs[index].shading.reads_backdrop || (has_rect && matches!(inputs[index].content, SequentialContent::Model(_)))) {
                        break;
                    }
                    // A layer whose effects run on the view's picture is a run of its own: its neighbours
                    // must not be in the picture its effects see.
                    if index > start && !inputs[index].screen_passes.is_empty() {
                        break;
                    }
                    has_rect |= matches!(inputs[index].content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_));
                    index += 1;
                    if !inputs[index - 1].screen_passes.is_empty() {
                        break;
                    }
                }
                (start, SRC_OVER)
            };
            let run = &inputs[start..index];

            // The sky is the ground: the run holding the top environment lays it under the stack.
            if run.iter().any(|input| matches!(input.content, SequentialContent::Environment(e) if environment.is_some_and(|top| std::ptr::eq(top, e)))) {
                let sky = self.view_canvas(window);
                let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, sequential_target_config("motolii-view-sky", comp, window, view_from_world, projection, environment), ViewBuilderId::new(self.next_readback), &sky.texture)
                    .map_err(|e| CompositorError::View(e.to_string()))?;
                self.next_readback += 1;
                builder.queue_draw(&self.ctx, re_renderer::renderer::GenericSkyboxDrawData::new(&self.ctx, re_renderer::renderer::GenericSkyboxType::Environment));
                commands.push(builder.draw(&self.ctx, Rgba::TRANSPARENT).map_err(|e| CompositorError::Draw(e.to_string()))?);
                stack = Some(match stack.take() {
                    None => sky,
                    Some(below) => self.mix_onto(window, &below, &sky, DEST_OVER, commands),
                });
            }

            let mut config = sequential_target_config("motolii-view-run", comp, window, view_from_world, projection, environment);
            config.motion = world.motion.cloned();
            config.scene_reflection = world.reflection.cloned();
            config.light = world.light.cloned();
            // A surface reading what is behind it reads this view's stack so far (ordered transmission).
            if let Some(below) = stack.as_ref().filter(|_| run.iter().any(|input| input.shading.reads_backdrop)) {
                let roughness = run.iter().filter(|input| input.shading.reads_backdrop).map(|input| input.shading.backdrop_roughness).fold(0.0f32, f32::max);
                let (texture, imported) = self.backdrop(window, below, roughness, commands)?;
                config.backdrop = Some(imported);
                backdrops.push(texture);
            }
            // Near things fade: only in the work's camera, only for what is in the world.
            if !flat(&run[0]) && window.projection_camera.is_none() {
                config.near_fade_distance = camera.near_fade;
            }
            self.surface_work.main_runs += 1;
            let canvas = self.view_canvas(window);
            let mut builder = ViewBuilder::new_with_external_resolved(&self.ctx, config, ViewBuilderId::new(self.next_readback), &canvas.texture)
                .map_err(|e| CompositorError::View(e.to_string()))?;
            self.next_readback += 1;
            self.surface_scene_draws(comp, run, Vec::new(), false, &|_| false, None, start)?.queue(&self.ctx, &mut builder);
            // The bottom of the stack starts from the background; everything above is drawn on clear.
            let clear = if stack.is_none() { clear_color(background_color) } else { Rgba::TRANSPARENT };
            commands.push(builder.draw(&self.ctx, clear).map_err(|e| CompositorError::Draw(e.to_string()))?);
            let canvas = match run {
                [only] if !only.screen_passes.is_empty() => self.screen_passes(window, canvas, only.screen_passes, only.screen_sources, stack.as_ref(), commands)?,
                _ => canvas,
            };
            stack = Some(match stack.take() {
                None => canvas,
                Some(below) => self.mix_onto(window, &below, &canvas, mode, commands),
            });
        }

        // The view shows the stack (or, with nothing drawn, the background).
        let mut shown = ViewBuilder::new(&self.ctx, screen_target_config("motolii-view", window), ViewBuilderId::new(self.next_readback))
            .map_err(|e| CompositorError::View(e.to_string()))?;
        self.next_readback += 1;
        let clear = match &stack {
            Some(picture) => {
                self.next_effect_key += 1;
                let imported = self.ctx.texture_manager_2d
                    .import_gpu_premultiplied(self.next_effect_key, &self.ctx, &picture.texture)
                    .map_err(|e| CompositorError::Effect(e.to_string()))?;
                let rects: Vec<TexturedRect> = vec![screen_rect(window, imported)];
                shown.queue_draw(&self.ctx, RectangleDrawData::new(&self.ctx, &rects).map_err(|e| CompositorError::Rectangles(e.to_string()))?);
                Rgba::TRANSPARENT
            }
            None => clear_color(background_color),
        };
        commands.push(shown.draw(&self.ctx, clear).map_err(|e| CompositorError::Draw(e.to_string()))?);
        drop(backdrops);
        Ok(shown)
    }

    /// A layer's effects run on its drawn canvas (it had no picture of its own to bake them into, or
    /// they read the view's picture below): the view's work, at the view's size.
    fn screen_passes(
        &mut self,
        window: Window,
        canvas: re_renderer::GpuTexture,
        passes: &[EffectPass],
        sources: &[Vec<GpuTexture2D>],
        below: Option<&re_renderer::GpuTexture>,
        commands: &mut Vec<wgpu::CommandBuffer>,
    ) -> Result<re_renderer::GpuTexture, CompositorError> {
        let (width, height) = (window.width, window.height);
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-view-screen-passes") });
        // A pass reading what is below gets the view's stack so far (nothing yet: transparent);
        // another reads the pictures the frame prepared for it (another time's composite).
        let below: Option<wgpu::Texture> = passes.iter().any(|p| p.reads_backdrop).then(|| match below {
            Some(below) => below.texture.clone(),
            None => self.ctx.texture_manager_2d.zeroed_texture_float().texture.clone(),
        });
        let others: Vec<Vec<wgpu::Texture>> = passes.iter().enumerate().map(|(i, p)| match (&below, p.reads_backdrop) {
            (Some(b), true) => vec![b.clone()],
            _ => sources.get(i).map(|row| row.iter().filter_map(|t| self.ctx.gpu_resources.textures.get_from_handle(t.handle()).ok().map(|g| g.texture.clone())).collect()).unwrap_or_default(),
        }).collect();
        let (mut current, linear, premultiplied, mut scratch) = self.record_pass_chain(
            &mut encoder, canvas.texture.clone(), true, true, false, passes, &others, None, [width, height], 0, [width, height],
        )?;
        if !linear {
            let back = self.convert_image_encoding(&mut encoder, &current, true, true, premultiplied);
            if scratch { self.effect_scratch.release(width, height, current.format(), current); }
            current = back;
            scratch = true;
        }
        // The effects' result replaces the canvas (compose 1 = copy).
        const COPY: u32 = 1;
        let out = self.view_canvas(window);
        let canvas_view = canvas.texture.create_view(&Default::default());
        let result_view = current.create_view(&Default::default());
        let out_view = out.texture.create_view(&Default::default());
        {
            let Self { ctx, blend_vism, effect_scratch, .. } = self;
            blend_vism.get(ctx).record_over(ctx, &mut encoder, effect_scratch, &[&canvas_view, &result_view], &out_view, &[("mode".to_owned(), COPY as f32)], window.size_f32());
        }
        if scratch { self.effect_scratch.release(width, height, current.format(), current); }
        commands.push(encoder.finish());
        Ok(out)
    }

    /// What is below, copied with the mip levels the roughest reader needs (glass blurs by reading
    /// coarser levels): the view's own picture, so it is the view's, from the pool.
    fn backdrop(&mut self, window: Window, below: &re_renderer::GpuTexture, roughness: f32, commands: &mut Vec<wgpu::CommandBuffer>) -> Result<(re_renderer::GpuTexture, GpuTexture2D), CompositorError> {
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
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-view-backdrop") });
        let level0 = |texture| wgpu::TexelCopyTextureInfo { texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All };
        encoder.copy_texture_to_texture(level0(&below.texture), level0(&pyramid.texture), size);
        let levels = re_renderer::backdrop_levels_read(roughness, pyramid.texture.mip_level_count());
        self.surface_work.backdrop_mip_levels += u64::from(levels);
        self.ctx.texture_manager_2d.generate_mipmap_levels(&self.ctx, &mut encoder, &pyramid.texture, levels);
        commands.push(encoder.finish());
        let imported = self.import_premultiplied(&pyramid.texture)?;
        Ok((pyramid, imported))
    }

    /// The world's light, captured once per document frame from the world's layers (placed as the
    /// output places them): the scene's reflection and the sun's occluder map.
    pub(crate) fn capture_world_light(
        &mut self,
        comp: CompSpec,
        inputs: &[SequentialInput<'_>],
        environment: Option<&GpuEnvironmentData>,
    ) -> Result<(Option<re_renderer::environment::SceneReflection>, Option<re_renderer::environment::SunLight>), CompositorError> {
        let environment = inputs.iter().rev().find_map(|input| match input.content {
            SequentialContent::Environment(e) => Some(e),
            _ => None,
        }).or(environment);
        let mut shared = None;
        let reflection = self.capture_scene_reflection(comp, inputs, environment, &mut shared)?;
        let light = self.capture_light_cookie(comp, inputs, environment, shared.as_ref())?;
        Ok((reflection, light))
    }

    /// A canvas the size of the view's window, from re_renderer's pool: it returns to the pool when
    /// the tick lets it go, and re_renderer retires it at a frame boundary if nothing reuses it.
    fn view_canvas(&self, window: Window) -> re_renderer::GpuTexture {
        self.ctx.gpu_resources.textures.alloc(&self.ctx.device, &re_renderer::TextureDesc {
            label: "motolii-view-canvas".into(),
            size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: BLEND_TARGET_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        })
    }

    /// `above` mixed onto `below` by `mode`, into a new canvas.
    fn mix_onto(
        &mut self,
        window: Window,
        below: &re_renderer::GpuTexture,
        above: &re_renderer::GpuTexture,
        mode: u32,
        commands: &mut Vec<wgpu::CommandBuffer>,
    ) -> re_renderer::GpuTexture {
        let out = self.view_canvas(window);
        let below_view = below.texture.create_view(&Default::default());
        let above_view = above.texture.create_view(&Default::default());
        let out_view = out.texture.create_view(&Default::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-view-mix") });
        let Self { ctx, blend_vism, effect_scratch, .. } = self;
        blend_vism.get(ctx).record_over(ctx, &mut encoder, effect_scratch, &[&below_view, &above_view], &out_view, &[("mode".to_owned(), mode as f32)], window.size_f32());
        commands.push(encoder.finish());
        out
    }
}
