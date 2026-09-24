//! One application tick, run the way Rerun's embedders run one (`re_viewer/src/app/ui.rs`): Motolii is
//! the embedder that opens and ends the renderer frame (`begin_frame` / `before_submit`):
//!
//! 1. Prepare the document frame — once, shared by every view (Rerun's once-per-frame context
//!    systems and caches): FrameGraph evaluation, lowering, the prepared GPU scene.
//! 2. Each view records its commands against that prepared frame (Rerun's `ViewBuilder::draw` per
//!    view). A view reads the prepared frame; it never makes, completes or changes it.
//! 3. The frame ends — one submission of everything recorded into it — and the next begins
//!    (`Compositor::next_frame`). Work between ticks (an edit) records into that open frame.
//!
//! What a piece of work belongs to is decided by its inputs: the document frame (prepare), a
//! view's projection / visibility / density (the view), or a view's own image or history (the
//! view's render and history).

use std::sync::Arc;

use crate::doc::core::{RationalTime, ResolvedCamera};
use crate::doc::store::StoreView;
use crate::frame_graph::{FrameQuality, ViewProjection};
use crate::render::compositor::Window;
use crate::render::engine::frame_graph_scene::{GpuSceneValue, Precision};
use crate::render::engine::{Engine, EngineError};

/// One view to draw this tick: where, what part of the composition, from which camera.
pub struct ViewRequest<'a> {
    pub target: &'a wgpu::Texture,
    pub window: Window,
    /// The view's camera; `None` is the work's own camera (the Camera view, export).
    pub camera: Option<ResolvedCamera>,
    pub projection: ViewProjection,
    pub include_background: bool,
    /// Read this view's picture back (an offline route: the readback arrives after the frame).
    pub read_back: bool,
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
        self.compositor.feedback_seen.clear();
        self.collect_analysis_pictures();
        self.tick_inside(doc, time, views, &mut stats)?;
        self.compositor.next_frame();
        stats.submits += 1;
        stats.begin_frames += 1;
        self.tick_stats = stats;
        Ok(stats)
    }

    fn tick_inside(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>], stats: &mut TickStats) -> Result<(), EngineError> {
        let prepared = self.prepare_document_frame(doc, time, views, stats)?;
        for view in views {
            self.record_view(&prepared, view)?;
            stats.views += 1;
        }

        // Feedback is a recurrence from the in-point: a frame reached by a jump is replayed from the
        // nearest checkpoint (the document's history and each view's), then drawn again. Each past
        // frame is a renderer frame of its own: its uploads (a video frame) must not be overwritten
        // by the next frame's before its draws run.
        let fps = doc.composition().ok().flatten().map(|c| c.fps);
        let now = fps.and_then(|fps| time.try_to_frame_round(fps).ok());
        if let (Some(fps), Some(now)) = (fps, now) {
            let state = self.frame_graph.take();
            let start = state.as_ref().and_then(|state| self.frame_graph_feedback_replay_start(state, now));
            self.frame_graph = state;
            if let Some(start) = start {
                self.compositor.next_frame();
                stats.submits += 1;
                stats.begin_frames += 1;
                for frame in start..now {
                    let at = RationalTime::try_from_frame(frame, fps).map_err(|error| EngineError::Time(error.to_string()))?;
                    self.tick_frame = None;
                    self.compositor.feedback_seen.clear();
                    let past = self.prepare_document_frame(doc, at, views, stats)?;
                    for view in views {
                        let scratch = self.compositor.view_canvas_for(view.window, crate::render::compositor::PRESENTABLE_FORMAT);
                        self.record_view(&past, &ViewRequest { target: &scratch.texture, read_back: false, ..*view })?;
                    }
                    self.compositor.next_frame();
                    stats.submits += 1;
                    stats.begin_frames += 1;
                    self.collect_analysis_pictures();
                }
                // The frame itself, on the history just rebuilt.
                self.compositor.baked_effects.clear();
                self.compositor.feedback_seen.clear();
                self.tick_frame = None;
                let prepared = self.prepare_document_frame(doc, time, views, stats)?;
                for view in views {
                    self.record_view(&prepared, view)?;
                }
            }
        }
        Ok(())
    }

    /// The document frame every view of this tick reads. The views ask, before it is prepared, how
    /// precisely (the densest one's device pixels per composition pixel); nothing else of theirs —
    /// camera, window, selection — reaches the preparation.
    fn prepare_document_frame(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>], stats: &mut TickStats) -> Result<Arc<PreparedFrame>, EngineError> {
        let densest = views.iter().map(|v| v.window.width as f32 / v.window.roi[2].max(1.0)).fold(0.0_f32, f32::max);
        let precision = Precision::for_density(densest);
        // A tick whose only views are the output is an export: full quality.
        let quality = if !views.is_empty() && views.iter().all(|v| v.projection == ViewProjection::Export) { FrameQuality::Export } else { FrameQuality::Preview { scale: 1 } };
        let state = self.prepared_frame_graph(doc, time, quality, precision)?;
        let scene = state.prepared.clone().ok_or_else(|| EngineError::Store("Lowered scene is missing".into()))?;
        if let Some(frame) = self.tick_frame.clone().filter(|frame| Arc::ptr_eq(&frame.scene, &scene)) {
            self.frame_graph = Some(state);
            return Ok(frame);
        }
        stats.preparations += 1;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        // Each layer's own effect chain reads only the document frame: run once, here, for every view.
        let (pictures, paddings, spills) = self.compositor.effective_layer_textures(&scene.layers)?;

        // The world the preparation left: its environment, the blocks' motion, and its one light
        // (captured in the preparation, where it saw every plate's members).
        let environment = self.compositor.world_environment.clone();
        let motion = self.compositor.motion.clone();
        let crate::render::compositor::WorldLight { light, meshes, .. } = scene.light.clone();
        let inputs = crate::render::compositor::sequential_inputs(&scene.layers, &pictures, &paddings, &spills, document_camera, document_camera);
        // The mesh instances a view placing the layers as the output does can draw as they are.
        let meshes = match meshes {
            Some(meshes) => Some(meshes),
            None => self.compositor.shared_mesh_scene(state.comp, &inputs)?,
        };
        // The Views the layers asked for see the frame every view shows: drawn once, here.
        let seen = crate::render::compositor::ViewWorld { environment: environment.as_deref(), motion: motion.as_ref(), light: light.as_ref(), views: &[], meshes: None };
        let layer_views = self.compositor.draw_layer_views(state.comp, &inputs, &seen)?;
        drop(inputs);
        let frame = Arc::new(PreparedFrame { scene, pictures, paddings, spills, environment, motion, light, meshes, views: layer_views, comp: state.comp, background: state.background, document_camera });
        self.frame_graph = Some(state);
        self.tick_frame = Some(frame.clone());
        Ok(frame)
    }

    /// One view's commands, as Rerun records a view: the view's pictures are drawn by its
    /// `ViewBuilder`s (`draw`, egui-wgpu's `prepare`), then the view composites into the surface
    /// (`composite`, `paint`).
    fn record_view(&mut self, frame: &PreparedFrame, view: &ViewRequest<'_>) -> Result<(), EngineError> {
        // The view draws the prepared layers where the work places them — every layer relative to
        // the work's camera (2.5D included: an observer looks at that relation from outside) — as a
        // re_renderer visualizer builds a view's draw data from shared resources.
        let inputs = crate::render::compositor::sequential_inputs(&frame.scene.layers, &frame.pictures, &frame.paddings, &frame.spills, frame.document_camera, frame.document_camera);
        let background = if view.include_background { frame.background } else { crate::render::compositor::NO_BACKGROUND };
        let mut encoder = self.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-tick-view") });
        let world = crate::render::compositor::ViewWorld { environment: frame.environment.as_deref(), motion: frame.motion.as_ref(), light: frame.light.as_ref(), views: &frame.views, meshes: frame.meshes.as_ref() };
        let camera = view.camera.unwrap_or(frame.document_camera);
        // A history on the view's own picture is the view's: keyed by which view it is.
        let shown = self.compositor.record_view(frame.comp, view.window, camera, &inputs, background, &world, 1 + view.projection as u32, &mut encoder)?;

        let ctx = &self.compositor.ctx;
        let surface = view.target.create_view(&wgpu::TextureViewDescriptor { format: Some(ctx.output_format_color()), ..Default::default() });
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
            shown.composite(ctx, &mut pass);
        }
        ctx.queue_commands([encoder.finish()]);
        if view.read_back {
            let id = crate::render::compositor::readback::ask_texture(ctx, view.target)
                .map_err(|error| EngineError::Store(error.to_string()))?;
            self.view_readbacks.push(id);
        }
        Ok(())
    }

    /// The output at `time`, read back: one tick with one Export view. An offline route: it waits
    /// for what the drawing route gets a frame later (the pictures its analyses ask for, the output).
    pub fn export_frame(&mut self, doc: &StoreView<'_>, time: RationalTime, include_background: bool, camera: Option<ResolvedCamera>) -> Result<Vec<u8>, EngineError> {
        /// How many times an export draws a frame again for analyses that ask for pictures.
        const SETTLE: usize = 4;
        let comp = doc.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        let window = Window::output(comp);
        let target = self.compositor.view_canvas_for(window, crate::render::compositor::PRESENTABLE_FORMAT);
        for _ in 0..SETTLE {
            self.view_readbacks.clear();
            self.tick(doc, time, &[ViewRequest { target: &target.texture, window, camera, projection: ViewProjection::Export, include_background, read_back: true }])?;
            if !self.analysis_waiting() {
                break;
            }
            self.compositor.wait_offline().map_err(|error| EngineError::Store(error.to_string()))?;
            self.collect_analysis_pictures();
        }
        let id = self.view_readbacks.pop().ok_or_else(|| EngineError::Store("the export view asked for no readback".into()))?;
        Ok(self.wait_for_readback(id)?.data)
    }

    /// Offline routes and tests only: `texture` as this frame leaves it, now (the frame ends and
    /// the GPU is waited for).
    #[cfg(test)]
    pub(crate) fn read_texture_offline(&mut self, texture: &wgpu::Texture) -> Result<Vec<u8>, EngineError> {
        let id = crate::render::compositor::readback::ask_texture(&self.compositor.ctx, texture)
            .map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(self.wait_for_readback(id)?.data)
    }

    /// The last tick's counts.
    pub fn tick_stats(&self) -> TickStats { self.tick_stats }
}

/// The document frame as every view of a tick reads it.
pub(in crate::engine) struct PreparedFrame {
    pub(in crate::engine) scene: Arc<GpuSceneValue>,
    /// Each layer's picture after its own effect chain, with its padding and spill.
    pictures: Vec<crate::render::compositor::LayerContent>,
    paddings: Vec<u32>,
    spills: Vec<crate::render::compositor::LayerSpill>,
    environment: Option<Arc<crate::render::compositor::GpuEnvironmentData>>,
    motion: Option<re_renderer::DataTexture>,
    views: Vec<crate::render::compositor::light::LayerViews>,
    light: Option<crate::render::compositor::light::SunLight>,
    meshes: Option<crate::render::compositor::SharedMeshScene>,
    comp: crate::doc::core::CompSpec,
    background: [f32; 4],
    document_camera: ResolvedCamera,
}
