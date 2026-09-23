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
    /// The view's camera; `None` is the work's own camera (the Camera view, export).
    pub camera: Option<ResolvedCamera>,
    pub projection: ViewProjection,
    pub include_background: bool,
    /// The layers selected in this view (editor state: the view's, never the prepared frame's).
    pub outline: &'a [crate::doc::store::LayerId],
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
        // Work recorded outside a tick finishes outside it; nothing crosses the frame boundary.
        debug_assert!(self.compositor.pending.is_empty(), "GPU work was recorded outside a tick and left for it");
        let submitted_before = self.compositor.sequential_submits();
        self.in_tick = true;
        let ticked = self.tick_inside(doc, time, views, &mut stats);
        self.in_tick = false;
        ticked?;
        stats.submits += (self.compositor.sequential_submits() - submitted_before) as u32;
        self.compositor.ctx.before_submit();
        let commands = std::mem::take(&mut self.tick_commands);
        self.compositor.last_submission = Some(self.compositor.ctx.queue.submit(commands));
        stats.submits += 1;
        // Staging goes back to the belts; work between ticks (an edit, a query) records into a fresh
        // frame-global encoder and submits itself.
        self.compositor.ctx.after_submit_within_frame();
        if std::mem::take(&mut self.tick_outlined) {
            if let Some(bounds) = self.compositor.selection_bounds.as_mut() { bounds.schedule_map(); }
        }
        self.tick_stats = stats;
        Ok(stats)
    }

    fn tick_inside(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>], stats: &mut TickStats) -> Result<(), EngineError> {
        self.compositor.ctx.begin_frame();
        stats.begin_frames += 1;

        let prepared = self.prepare_document_frame(doc, time, views, stats)?;

        let mut commands = std::mem::take(&mut self.compositor.pending);
        for view in views {
            commands.extend(self.record_view(&prepared, view)?);
            stats.views += 1;
        }

        // Feedback is a recurrence from the in-point: a frame reached by a jump is replayed from the
        // nearest checkpoint (the document's history and each view's), then drawn again.
        let fps = doc.composition().ok().flatten().map(|c| c.fps);
        let now = fps.and_then(|fps| time.try_to_frame_round(fps).ok());
        if let (Some(fps), Some(now)) = (fps, now) {
            let state = self.frame_graph.take();
            let start = state.as_ref().and_then(|state| self.frame_graph_feedback_replay_start(state, now));
            self.frame_graph = state;
            if let Some(start) = start {
                for frame in start..now {
                    let at = RationalTime::try_from_frame(frame, fps).map_err(|error| EngineError::Time(error.to_string()))?;
                    self.tick_frame = None;
                    let past = self.prepare_document_frame(doc, at, views, stats)?;
                    commands.append(&mut self.compositor.pending);
                    for view in views {
                        let scratch = self.compositor.ctx.device.create_texture(&wgpu::TextureDescriptor {
                            label: Some("motolii-feedback-replay"),
                            size: wgpu::Extent3d { width: view.window.width, height: view.window.height, depth_or_array_layers: 1 },
                            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
                            format: crate::render::compositor::PRESENTABLE_FORMAT,
                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        });
                        commands.extend(self.record_view(&past, &ViewRequest { target: &scratch, ..*view })?);
                    }
                    // Each past frame is a frame of its own: its uploads (a video frame) must not be
                    // overwritten by the next frame's before its draws run.
                    self.compositor.ctx.before_submit();
                    self.compositor.ctx.queue.submit(std::mem::take(&mut commands));
                    self.compositor.ctx.after_submit_within_frame();
                    stats.submits += 1;
                }
                // The frame itself, on the history just rebuilt.
                let c = &mut self.compositor;
                c.baked_effects.clear(&mut c.effect_scratch);
                self.feedback_keys_seen.clear();
                self.tick_frame = None;
                let prepared = self.prepare_document_frame(doc, time, views, stats)?;
                commands.append(&mut self.compositor.pending);
                for view in views {
                    commands.extend(self.record_view(&prepared, view)?);
                }
            }
        }
        self.tick_commands = commands;
        Ok(())
    }

    /// The document frame every view of this tick reads, prepared as densely as the densest view
    /// shows it (one device pixel after projection, a power of two, never above the composition's
    /// own pixels) — decided here, before preparing, from the views the tick was given.
    fn prepare_document_frame(&mut self, doc: &StoreView<'_>, time: RationalTime, views: &[ViewRequest<'_>], stats: &mut TickStats) -> Result<Arc<PreparedFrame>, EngineError> {
        let densest = views.iter().map(|v| v.window.width as f32 / v.window.roi[2].max(1.0)).fold(0.0_f32, f32::max);
        let density = if densest <= 0.0 { 1.0 } else { (2.0_f32).powf(densest.max(1.0 / 16.0).log2().ceil()).min(1.0) };
        self.picture_density = density;
        // A tick whose only views are the output is an export: full quality.
        let quality = if !views.is_empty() && views.iter().all(|v| v.projection == ViewProjection::Export) { FrameQuality::Export } else { FrameQuality::Preview { scale: 1 } };
        let state = self.evaluated_frame_graph(doc, time, quality)?;
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
        let (pictures, paddings, spills, _always_empty) = self.compositor.effective_layer_textures(&scene.layers)?;

        // The world the preparation left: its environment and the blocks' motion.
        let environment = self.compositor.world_environment.clone();
        let motion = self.compositor.motion.clone();
        // The world's light, once: every layer placed as the output places it.
        let mut placed = scene.layers.clone();
        for layer in &mut placed { layer.layer.projection_camera = document_camera; }
        let world_inputs = crate::render::compositor::sequential_inputs(&placed, &pictures, &paddings, &spills);
        // The frame's light was captured once in its preparation (it saw every plate's members).
        let crate::render::compositor::WorldLight { reflection, light, meshes, .. } = self.frame_light.clone();
        let meshes = match meshes { Some(meshes) => Some(meshes), None => self.compositor.shared_mesh_scene(state.comp, &world_inputs)? };
        drop(world_inputs);
        let frame = Arc::new(PreparedFrame { scene, pictures, paddings, spills, environment, motion, reflection, light, meshes, comp: state.comp, background: state.background, document_camera });
        self.frame_graph = Some(state);
        self.tick_frame = Some(frame.clone());
        Ok(frame)
    }

    /// One view's commands, as Rerun records a view: the view's pictures are drawn by its
    /// `ViewBuilder`s (`draw`, egui-wgpu's `prepare`), then the view composites into the surface
    /// (`composite`, `paint`).
    fn record_view(&mut self, frame: &PreparedFrame, view: &ViewRequest<'_>) -> Result<Vec<wgpu::CommandBuffer>, EngineError> {
        // The view places the prepared layers: 2D by the document's camera (the output's frame),
        // the rest by the view's own (a Stage's default camera).
        let placing = view.window.projection_camera.unwrap_or(frame.document_camera);
        let mut layers = frame.scene.layers.clone();
        let outline: Vec<_> = view.outline.iter().copied().take(255).collect();
        for (layer, id) in layers.iter_mut().zip(&frame.scene.layer_ids) {
            layer.layer.outline = outline.iter().position(|l| l == id).map_or(0, |i| i as u8 + 1);
            if let Some(feature) = not_ported(layer) {
                return Err(EngineError::Store(format!("tick: {feature} is not ported to the tick yet")));
            }
            layer.layer.projection_camera = if layer.layer.projection == crate::doc::store::LayerProjection::TwoD { frame.document_camera } else { placing };
            // Effects on the view's picture keep the view's own history (its picture over time).
            let on_the_view = layer.layer.content.texture().is_none() || layer.passes.iter().any(|pass| pass.reads_backdrop || pass.reads_composite());
            if on_the_view {
                for pass in &mut layer.passes {
                    if let Some(key) = pass.feedback.as_mut() {
                        key.screen = Some([1 + view.projection as u32, view.window.width, view.window.height]);
                        if key.namespace == 0 { self.feedback_keys_seen.push(*key); }
                    }
                }
            }
        }
        let inputs = crate::render::compositor::sequential_inputs(&layers, &frame.pictures, &frame.paddings, &frame.spills);
        let background = if view.include_background { frame.background } else { crate::render::compositor::NO_BACKGROUND };
        // One encoder per view, as Rerun records a view: every pass of the view's composition, then
        // its composite into the surface.
        let mut encoder = self.compositor.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("motolii-tick-view") });
        // A view placing the layers as the world does (the output's camera) draws the world's mesh
        // instances; a Stage places its own.
        let meshes = frame.meshes.as_ref().filter(|_| view.window.projection_camera.is_none() && view.camera.is_none() && outline.is_empty());
        let world = crate::render::compositor::ViewWorld { environment: frame.environment.as_deref(), motion: frame.motion.as_ref(), reflection: frame.reflection.as_ref(), light: frame.light.as_ref(), meshes };
        let camera = view.camera.unwrap_or(frame.document_camera);
        let shown = self.compositor.record_view(frame.comp, view.window, camera, &inputs, background, &world, &mut encoder)?;
        if self.compositor.record_outline(frame.comp, view.window, camera, &inputs, &mut encoder)? {
            self.outline_order = outline;
            self.tick_outlined = true;
        }

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
        Ok(vec![encoder.finish()])
    }

    /// The output at `time`, read back: one tick with one Export view.
    pub fn export_frame(&mut self, doc: &StoreView<'_>, time: RationalTime, include_background: bool, camera: Option<ResolvedCamera>) -> Result<Vec<u8>, EngineError> {
        let comp = doc.composition().map_err(|e| EngineError::Store(e.to_string()))?.ok_or(EngineError::NoComposition)?.spec();
        let window = Window::output(comp);
        let target = self.compositor.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-export"),
            size: wgpu::Extent3d { width: window.width, height: window.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        self.tick(doc, time, &[ViewRequest { target: &target, window, camera, projection: ViewProjection::Export, include_background, outline: &[] }])?;
        Ok(self.compositor.read_texture_bytes(&target)?)
    }

    /// The last tick's counts.
    pub fn tick_stats(&self) -> TickStats { self.tick_stats }
}

/// What the tick cannot draw yet, named; the old path draws it until it is ported.
fn not_ported(layer: &crate::render::compositor::LayerWithPasses) -> Option<&'static str> {
    use crate::render::compositor::LayerContent;
    match layer.layer.content {
        LayerContent::Texture(_) | LayerContent::LinearTexture(_) | LayerContent::Model(_) | LayerContent::Environment(_) | LayerContent::Cloud { .. } => None,
    }
}

/// The document frame as every view of a tick reads it.
pub(in crate::engine) struct PreparedFrame {
    pub(in crate::engine) scene: Arc<GpuSceneValue>,
    /// Each layer's picture after its own effect chain, with its padding and spill.
    pictures: Vec<crate::render::compositor::LayerContent>,
    paddings: Vec<u32>,
    spills: Vec<crate::render::compositor::LayerSpill>,
    environment: Option<Arc<crate::render::compositor::GpuEnvironmentData>>,
    motion: Option<re_renderer::MotionBuffer>,
    reflection: Option<re_renderer::environment::SceneReflection>,
    light: Option<re_renderer::environment::SunLight>,
    meshes: Option<crate::render::compositor::SharedMeshScene>,
    comp: crate::doc::core::CompSpec,
    background: [f32; 4],
    document_camera: ResolvedCamera,
}
