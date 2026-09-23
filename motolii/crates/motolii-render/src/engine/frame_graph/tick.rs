//! One application tick, run the way Rerun runs one (`re_viewer/src/app/ui.rs`, and the fork's
//! embedder `re_view_spatial/src/spatial_stage.rs` `SpatialStage::show`):
//!
//! 1. `RenderContext::begin_frame` — once, before anything records (pools age, belts recycle).
//! 2. Prepare the document frame — once, shared by every view (Rerun's once-per-frame context
//!    systems and caches): FrameGraph evaluation, lowering, the prepared GPU scene.
//! 3. Each view records its commands against that prepared frame (Rerun's `ViewBuilder::draw` per
//!    view). A view reads the prepared frame; it never makes, completes or changes it.
//! 4. `before_submit`, then one `queue.submit` of everything the tick recorded.
//!
//! What a piece of work belongs to is decided by its inputs: the document frame (prepare), a
//! view's projection / visibility / density (the view), or a view's own image or history (the
//! view's render and history).

use std::sync::Arc;

use crate::doc::core::{RationalTime, ResolvedCamera};
use crate::doc::store::StoreView;
use crate::frame_graph::{FrameQuality, ViewProjection};
use crate::render::compositor::Window;
use crate::render::engine::frame_graph_scene::GpuSceneValue;
use crate::render::engine::{Engine, EngineError};

/// One view to draw this tick: where, what part of the composition, from which camera.
pub struct ViewRequest<'a> {
    pub target: &'a wgpu::Texture,
    pub window: Window,
    pub camera: ResolvedCamera,
    pub projection: ViewProjection,
    pub include_background: bool,
}

/// What one tick did. The architecture's invariants are stated over these counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickStats {
    /// Renderer frames begun (always 1).
    pub begin_frames: u32,
    /// Document frames prepared (0 when the tick shows the frame already prepared, else 1).
    pub preparations: u32,
    /// Views recorded.
    pub views: u32,
    /// Queue submissions (always 1).
    pub submits: u32,
}

impl Engine {
    /// One application tick: every shown view of the document at `time`.
    pub fn tick(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>]) -> Result<TickStats, EngineError> {
        let mut stats = TickStats::default();
        self.compositor.ctx.begin_frame();
        stats.begin_frames += 1;

        let prepared = self.prepare_document_frame(doc, time, views, &mut stats)?;

        let mut commands = std::mem::take(&mut self.compositor.pending);
        for view in views {
            commands.extend(self.record_view(&prepared, view)?);
            stats.views += 1;
        }

        self.compositor.ctx.before_submit();
        self.compositor.last_submission = Some(self.compositor.ctx.queue.submit(commands));
        stats.submits += 1;
        self.tick_stats = stats;
        Ok(stats)
    }

    /// The document frame every view of this tick reads, prepared as densely as the densest view
    /// shows it (one device pixel after projection, a power of two, never above the composition's
    /// own pixels) — decided here, before preparing, from the views the tick was given.
    fn prepare_document_frame(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>], stats: &mut TickStats) -> Result<Arc<PreparedFrame>, EngineError> {
        let densest = views.iter().map(|v| v.window.width as f32 / v.window.roi[2].max(1.0)).fold(0.0_f32, f32::max);
        let density = if densest <= 0.0 { 1.0 } else { (2.0_f32).powf(densest.max(1.0 / 16.0).log2().ceil()).min(1.0) };
        self.picture_density = density;
        let generation = self.frame_graph.as_ref().map(|state| state.generation);
        let state = self.evaluated_frame_graph(doc, time, FrameQuality::Preview { scale: 1 })?;
        if generation != Some(state.generation) || self.tick_frame.is_none() {
            stats.preparations += 1;
        }
        let scene = state.prepared.clone().ok_or_else(|| EngineError::Store("Lowered scene is missing".into()))?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let frame = Arc::new(PreparedFrame { scene, comp: state.comp, background: state.background, document_camera });
        self.frame_graph = Some(state);
        self.tick_frame = Some(frame.clone());
        Ok(frame)
    }

    /// One view's commands, as Rerun records a view: the `ViewBuilder` draws into its own target
    /// (`draw`, egui-wgpu's `prepare`), then composites into the surface (`composite`, `paint`).
    fn record_view(&mut self, frame: &PreparedFrame, view: &ViewRequest<'_>) -> Result<Vec<wgpu::CommandBuffer>, EngineError> {
        let ctx = &self.compositor.ctx;
        let projection = crate::doc::core::camera_projection(frame.comp, view.camera);
        let view_from_world = macaw::IsoTransform::from_rotation_translation(projection.rotation, -(projection.rotation * projection.eye));
        let config = crate::render::compositor::sequential_target_config("motolii-tick-view", frame.comp, view.window, view_from_world, projection, None);
        let id = re_renderer::ViewBuilderId::new(self.compositor.next_readback);
        self.compositor.next_readback += 1;
        let mut builder = re_renderer::ViewBuilder::new(ctx, config, id).map_err(|error| EngineError::Store(error.to_string()))?;

        // The view places the prepared layers: 2D by the document's camera (the output's frame),
        // the rest by the view's own (a Stage's default camera).
        let placing = view.window.projection_camera.unwrap_or(frame.document_camera);
        let mut layers = frame.scene.layers.clone();
        for layer in &mut layers {
            if let Some(feature) = not_ported(layer) {
                return Err(EngineError::Store(format!("tick: {feature} is not ported to the tick yet")));
            }
            layer.layer.projection_camera = if layer.layer.projection == crate::doc::store::LayerProjection::TwoD { frame.document_camera } else { placing };
        }
        let contents: Vec<_> = layers.iter().map(|layer| layer.layer.content.clone()).collect();
        let paddings: Vec<_> = layers.iter().map(|layer| layer.padding).collect();
        let spills = vec![None; layers.len()];
        let inputs = crate::render::compositor::sequential_inputs(&layers, &contents, &paddings, &spills);
        let draws = self.compositor.surface_scene_draws(frame.comp, &inputs, Vec::new(), false, &|_| false, None, 0)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        let ctx = &self.compositor.ctx;
        draws.queue(ctx, &mut builder);

        let background = if view.include_background { frame.background } else { crate::render::compositor::NO_BACKGROUND };
        let drawn = builder.draw(ctx, crate::render::compositor::clear_color(background)).map_err(|error| EngineError::Store(error.to_string()))?;

        let surface = view.target.create_view(&wgpu::TextureViewDescriptor { format: Some(ctx.output_format_color()), ..Default::default() });
        let mut encoder = ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-tick-composite") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("motolii-tick-composite"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            builder.composite(ctx, &mut pass);
        }
        Ok(vec![drawn, encoder.finish()])
    }

    /// The last tick's counts.
    pub fn tick_stats(&self) -> TickStats { self.tick_stats }
}

/// What the tick cannot draw yet, named; the old path draws it until it is ported.
fn not_ported(layer: &crate::render::compositor::LayerWithPasses) -> Option<&'static str> {
    use crate::render::compositor::LayerContent;
    if !layer.passes.is_empty() { return Some("an effect chain"); }
    if layer.layer.blend_mode != crate::render::compositor::BlendMode::Normal { return Some("a blend mode"); }
    if layer.layer.clip.is_some() { return Some("a clip"); }
    if layer.layer.shading.reads_backdrop { return Some("a surface reading the backdrop"); }
    match layer.layer.content {
        LayerContent::Texture(_) | LayerContent::LinearTexture(_) => None,
        _ => Some("a mesh, cloud, path or environment layer"),
    }
}

/// The document frame as every view of a tick reads it.
pub(in crate::engine) struct PreparedFrame {
    pub(in crate::engine) scene: Arc<GpuSceneValue>,
    comp: crate::doc::core::CompSpec,
    background: [f32; 4],
    document_camera: ResolvedCamera,
}
