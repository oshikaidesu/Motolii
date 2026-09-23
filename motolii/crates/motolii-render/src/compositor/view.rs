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
        commands: &mut Vec<wgpu::CommandBuffer>,
    ) -> Result<ViewBuilder, CompositorError> {
        let projection = crate::doc::core::camera_projection(comp, camera);
        let view_from_world = macaw::IsoTransform::from_rotation_translation(projection.rotation, -(projection.rotation * projection.eye));
        let flat = |input: &SequentialInput<'_>| input.projection == crate::doc::store::LayerProjection::TwoD;
        let alone = |input: &SequentialInput<'_>| {
            matches!(input.content, SequentialContent::Rect(_) | SequentialContent::LinearRect(_)) && vello_blend_mode(input.blend_mode).is_some()
        };

        let mut stack: Option<re_renderer::GpuTexture> = None;
        let mut index = 0;
        while index < inputs.len() {
            // A mix blend reads what is below: the picture is drawn alone, then mixed onto the stack.
            let (start, mode) = if let Some(mode) = vello_blend_mode(inputs[index].blend_mode).filter(|_| alone(&inputs[index])) {
                index += 1;
                (index - 1, mode)
            } else {
                // 2D is not in the world (2026-09-12): a run is all 2D or all not, and runs stack in order.
                let start = index;
                while index < inputs.len() && !alone(&inputs[index]) {
                    if index > start && flat(&inputs[index]) != flat(&inputs[start]) {
                        break;
                    }
                    index += 1;
                }
                (start, SRC_OVER)
            };
            let run = &inputs[start..index];

            let mut config = sequential_target_config("motolii-view-run", comp, window, view_from_world, projection, None);
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
        Ok(shown)
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
