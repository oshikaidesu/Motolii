//! Product playback owner: compile once per revision, evaluate once per exact
//! comp time, then lower the evaluated scene below the semantic graph.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreView};
use crate::frame_graph::{BlobAnalysisRequestValue, BlobAnalysisValue, CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, MediaExtentValue, NodeExecutor, NodeInputs, NodeKey, NodeKind, NodeValue, OverlayAnalysisValue, OverlaySetValue, SceneProgram, SceneValue, SolverPlanValue};

use super::frame_graph_scene::GpuSceneValue;
use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    program: SceneProgram,
    scene: NodeKey,
    solver: NodeKey,
    overlay: NodeKey,
    prepared: Option<Arc<GpuSceneValue>>,
    comp: CompSpec,
    fps: crate::doc::store::Fps,
    background: [f32; 4],
    in_points: HashMap<LayerId, i64>,
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
    /// The picture density the prepared frame was baked at.
    density: f32,
}

impl EngineFrameGraph {
    fn new(view: &StoreView<'_>, revision: GraphRevision) -> Result<Self, EngineError> {
        let composition = view.composition().map_err(store)?.ok_or(EngineError::NoComposition)?;
        let (comp, fps, background) = (composition.spec(), composition.fps, composition.background);
        let in_points = view.layers().into_iter().filter_map(|layer| {
            view.meta(layer).ok().flatten().map(|meta| (layer, meta.timing.start))
        }).collect();
        let program = SceneProgram::compile(view).map_err(|error| EngineError::Store(error.to_string()))?;
        let scene = program.scene().scene;
        let document_camera = program.camera();
        let solver = program.solver().key();
        let overlay = program.overlay().output();
        let topology = GraphTopology::try_new(program.nodes().collect::<Vec<_>>(), vec![scene, document_camera, solver, overlay]).map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(Self { graph: CompiledGraph::with_topology(revision, topology), program, scene, solver, overlay, prepared: None, comp, fps, background, in_points, frame: None, generation: 0, prepare_us: 0, measured: false, density: 1.0 })
    }
    fn matches(&self, revision: GraphRevision, time: RationalTime, density: f32) -> bool { self.graph.revision() == revision && self.density == density && self.frame.as_ref().is_some_and(|frame| frame.time() == time) }

    fn evaluate_at(
        &mut self,
        engine: &mut Engine,
        time: RationalTime,
        quality: FrameQuality,
    ) -> Result<(), EngineError> {
        self.generation += 1;
        let mut executor = ProgramExecutor {
            engine,
            program: &self.program,
            comp: self.comp,
            fps: self.fps,
        };
        let evaluated = self.graph.evaluate(
            &mut executor,
            time,
            quality,
            Generation::new(self.generation),
        )?;
        let value = |key: NodeKey| evaluated.value(key).ok_or_else(|| EngineError::Store("FrameGraph root is missing".into()));
        let scene = value(self.scene)?.downcast_ref::<SceneValue>().ok_or_else(|| EngineError::Store("FrameGraph scene has the wrong type".into()))?;
        let camera = value(self.program.camera())?.downcast_ref::<ResolvedCamera>().copied().unwrap_or_default();
        let solver = value(self.solver)?.downcast_ref::<SolverPlanValue>().ok_or_else(|| EngineError::Store("FrameGraph solver has the wrong type".into()))?;
        let overlays = value(self.overlay)?.downcast_ref::<OverlaySetValue>().ok_or_else(|| EngineError::Store("FrameGraph overlay set has the wrong type".into()))?;
        engine.install_frame_graph_overlays(overlays);
        let frame = time.try_to_frame_round(self.fps).unwrap_or(0) as f32;
        engine.compositor.clock = Some([
            time.as_seconds_f64() as f32,
            self.fps.den() as f32 / self.fps.num() as f32,
            frame,
        ]);
        self.density = engine.picture_density;
        self.prepared = Some(Arc::new(engine.prepare_gpu_scene_with_solver(scene, solver, self.comp, camera, time, self.fps)?));
        self.frame = Some(evaluated);
        Ok(())
    }
}


struct ProgramExecutor<'a> { engine: &'a mut Engine, program: &'a SceneProgram, comp: CompSpec, fps: crate::doc::store::Fps }
impl NodeExecutor for ProgramExecutor<'_> {
    type Error = EngineError;

    fn dynamic_inputs(&mut self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Result<Vec<crate::frame_graph::DynamicInput>, Self::Error> {
        self.program.dynamic_inputs(node, inputs, context).map_err(|error| EngineError::Store(error.to_string()))
    }

    fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
        // Only nodes that are not reused reach here: each claims its recomputation.
        let started = std::time::Instant::now();
        let value = self.execute_node(node, inputs, context);
        self.engine.ledger.claim("evaluate", format!("{:?}", node.identity().kind), "recomputed", started.elapsed());
        value
    }
}

impl ProgramExecutor<'_> {
    fn execute_node(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, EngineError> {
        if node.identity().kind == NodeKind::MediaExtent {
            let source = self.program.content().extent_source(node.key())
                .cloned()
                .ok_or_else(|| unsupported(node.identity().kind))?;
            let size = self.engine.material_extent(&source.path, self.comp).unwrap_or([0.0; 3]);
            return Ok(NodeValue::new(MediaExtentValue { source, size }));
        }
        match self.program.execute(node, &inputs, &context) {
            Ok(value) => return Ok(value),
            Err(crate::frame_graph::SceneProgramError::Unsupported(_)) => {}
            Err(error) => return Err(EngineError::Store(error.to_string())),
        }
        match node.identity().kind {
            NodeKind::AnalysisBlob => {
                let request = direct::<BlobAnalysisRequestValue>(node, &inputs, 0)?;
                let previous = inputs.at(1).and_then(|value| value.downcast_ref::<BlobAnalysisValue>());
                let value = self.engine.frame_graph_blob_analysis(request, previous, context.time, self.comp, self.fps)?;
                Ok(NodeValue::new(value))
            }
            NodeKind::AnalysisOverlay => {
                let scene = direct::<SceneValue>(node, &inputs, 0)?;
                let effect = inputs.at(1)
                    .and_then(|value| value.downcast_ref::<crate::frame_graph::EffectValue>())
                    .and_then(|value| value.0.as_ref())
                    .ok_or_else(|| unsupported(node.identity().kind))?;
                let solver = direct::<SolverPlanValue>(node, &inputs, 2)?;
                let camera = *direct::<ResolvedCamera>(node, &inputs, 3)?;
                let previous = inputs.at(4).and_then(|value| value.downcast_ref::<OverlayAnalysisValue>());
                let (layer, _order, parent) = self.program.overlay().recipe(node.key())
                    .ok_or_else(|| unsupported(node.identity().kind))?;
                let value = self.engine.frame_graph_overlay_analysis(
                    layer, parent, effect, scene, solver, camera, previous, context.time, self.comp,
                )?;
                Ok(NodeValue::new(value))
            }
            kind => Err(unsupported(kind)),
        }
    }
}

impl Engine {
    /// Camera value already evaluated by the revision-scoped production graph.
    /// Editor read paths use this after preparing the same frame; no second
    /// SceneProgram/Graph is compiled just to discover the observer.
    pub fn frame_graph_cached_camera(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Option<ResolvedCamera> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time, state.density) { return None; }
        state.frame.as_ref()?
            .value(state.program.camera())?
            .downcast_ref::<ResolvedCamera>()
            .copied()
    }

    /// Local and inherited world transform evaluated by the production
    /// TransformProgram for this exact revision/time.
    pub fn frame_graph_cached_transform(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
        id: LayerId,
    ) -> Option<(crate::frame_graph::TransformValue, crate::frame_graph::TransformValue)> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time, state.density) { return None; }
        let binding = state.program.transforms().binding(id)?;
        let frame = state.frame.as_ref()?;
        let local = frame.value(binding.local)?.downcast_ref::<crate::frame_graph::TransformValue>().copied()?;
        let world = frame.value(binding.world)?.downcast_ref::<crate::frame_graph::TransformValue>().copied()?;
        Some((local, world))
    }

    /// Editor read model. Evaluate the production graph once and hand the
    /// semantic scene to editor geometry directly. Do not project back through
    /// ResolvedLayer: that would recreate the migration bridge we are deleting.
    pub fn frame_graph_editor_scene(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<SceneValue, EngineError> {
        let state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let scene = state.frame.as_ref()
            .and_then(|frame| frame.value(state.scene))
            .and_then(|value| value.downcast_ref::<SceneValue>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph semantic scene is missing".into()))?;
        self.frame_graph = Some(state);
        Ok(scene)
    }

    /// Borrow the semantic scene already evaluated for this exact revision/time.
    /// Read-only editor paths use this so Camera/Stage status never compiles a
    /// second graph or asks the legacy resolver for a parallel scene.
    pub fn frame_graph_cached_scene(
        &self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Option<&SceneValue> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time, state.density) { return None; }
        state.frame.as_ref()?.value(state.scene)?.downcast_ref::<SceneValue>()
    }

    pub(in crate::engine) fn evaluate_frame_graph_semantics(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<(SceneValue, ResolvedCamera, CompSpec, crate::doc::store::Fps), EngineError> {
        let composition = view.composition().map_err(store)?.ok_or(EngineError::NoComposition)?;
        let program = SceneProgram::compile(view).map_err(|error| EngineError::Store(error.to_string()))?;
        let scene_key = program.scene().scene;
        let camera_key = program.camera();
        let topology = GraphTopology::try_new(program.nodes(), vec![scene_key, camera_key])
            .map_err(|error| EngineError::Store(error.to_string()))?;
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(view.revision_key()), topology);
        let mut executor = ProgramExecutor {
            engine: self,
            program: &program,
            comp: composition.spec(),
            fps: composition.fps,
        };
        let frame = graph.evaluate(
            &mut executor,
            time,
            FrameQuality::Export,
            Generation::new(1),
        )?;
        let scene = frame.value(scene_key)
            .and_then(|value| value.downcast_ref::<SceneValue>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph semantic scene is missing".into()))?;
        let camera = frame.value(camera_key)
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        Ok((scene, camera, composition.spec(), composition.fps))
    }

    pub fn frame_graph_document_camera(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
    ) -> Result<ResolvedCamera, EngineError> {
        let state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        self.frame_graph = Some(state);
        Ok(camera)
    }

    pub(in crate::engine) fn render_frame_graph_pixels(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        include_background: bool,
        camera_override: Option<ResolvedCamera>,
    ) -> Result<Vec<u8>, EngineError> {
        // Output is the composition's own pixels.
        self.picture_density = 1.0;
        self.compositor.begin_render_frame();
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let prepared = state.prepared.clone()
            .ok_or_else(|| EngineError::Store("Lowered scene is missing".into()))?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let camera = camera_override.unwrap_or(document_camera);
        let mut layers = prepared.layers.clone();
        for layer in &mut layers {
            layer.layer.projection_camera = document_camera;
        }

        let background = if include_background { state.background } else { crate::render::compositor::NO_BACKGROUND };
        let pixels = self.compositor.render_with_effects(state.comp, camera, &layers, background)?;
        self.frame_graph = Some(state);
        Ok(pixels)
    }

    pub(in crate::engine) fn render_frame_graph_to_texture_output(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        include_background: bool,
    ) -> Result<(wgpu::Texture, wgpu::TextureView), EngineError> {
        // Output is the composition's own pixels.
        self.picture_density = 1.0;
        self.compositor.begin_render_frame();
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let prepared = state.prepared.clone()
            .ok_or_else(|| EngineError::Store("Lowered scene is missing".into()))?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let mut layers = prepared.layers.clone();
        for layer in &mut layers {
            layer.layer.projection_camera = document_camera;
        }
        let background = if include_background { state.background } else { crate::render::compositor::NO_BACKGROUND };
        let texture = self.compositor.render_to_texture(state.comp, document_camera, &layers, background)?;
        self.frame_graph = Some(state);
        Ok(texture)
    }

    fn evaluated_frame_graph(&mut self, view: &StoreView<'_>, time: RationalTime, quality: FrameQuality) -> Result<EngineFrameGraph, EngineError> {
        let revision = GraphRevision::new(view.revision_key());
        let mut state = match self.frame_graph.take() { Some(state) if state.graph.revision() == revision => state, _ => EngineFrameGraph::new(view, revision)? };
        self.compositor.feedback_set_revision(view.revision_key());
        self.feedback_keys_seen.clear();
        if !state.matches(revision, time, self.picture_density) {
            let started = std::time::Instant::now();
            if let Err(error) = state.evaluate_at(self, time, quality) {
                self.frame_graph = Some(state);
                return Err(error);
            }
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
        }
        Ok(state)
    }

    pub(super) fn semantic_layer_for(&self, view: &StoreView<'_>, time: RationalTime, id: LayerId) -> Option<&crate::frame_graph::SceneLayerValue> {
        self.frame_graph_cached_scene(view, time)?.layer(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_frame_graph_into_window(
        &mut self,
        view: &StoreView<'_>,
        time: RationalTime,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        outline: &[LayerId],
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        // The frame starts before evaluation: evaluating is part of what the frame costs.
        let frame_start = std::time::Instant::now();
        self.compositor.begin_render_frame();
        self.ledger.begin_view(time, format!("{projection:?}"));
        // Pictures are baked as densely as the densest view shows them (one device pixel after
        // projection), never above the composition's own pixels.
        let density = window.width as f32 / window.roi[2].max(1.0);
        self.view_densities.insert(format!("{projection:?}"), density);
        let densest = self.view_densities.values().copied().fold(0.0_f32, f32::max);
        self.picture_density = (2.0_f32).powf(densest.max(1.0 / 16.0).log2().ceil()).min(1.0);
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        self.compositor.measurement = Default::default();
        if !state.measured {
            self.compositor.measurement.resolve_us = state.prepare_us;
            state.measured = true;
        }
        self.outline_layers = outline.iter().copied().take(255).collect();
        self.outline_order = self.outline_layers.clone();

        self.render_frame_graph_projection(
            &state,
            target,
            camera,
            include_background,
            window,
            projection,
        )?;

        let now = time.try_to_frame_round(state.fps).unwrap_or(0);
        if let Some(start) = self.frame_graph_feedback_replay_start(&state, now) {
            self.replay_frame_graph_feedback(
                &mut state,
                start,
                now,
                camera,
                include_background,
                window,
                projection,
            )?;

            let c = &mut self.compositor;
            c.baked_effects.clear(&mut c.effect_scratch);
            self.feedback_keys_seen.clear();
            state.evaluate_at(self, time, FrameQuality::Preview { scale: 1 })?;
            self.render_frame_graph_projection(
                &state,
                target,
                camera,
                include_background,
                window,
                projection,
            )?;
        }

        self.outline_layers.clear();
        self.compositor.measurement.total_us = frame_start.elapsed().as_micros() as u64;
        let budget = std::time::Duration::from_secs_f64(1.0 / state.fps.as_f64().max(1.0));
        self.ledger.claim("view", format!("{projection:?}"), "drawn for this tick", frame_start.elapsed());
        let tick = self.ledger.end_view(frame_start.elapsed());
        if let Some(report) = self.ledger.report_if_over(tick, budget) {
            eprintln!("{report}");
        }
        self.frame_graph = Some(state);
        Ok(())
    }

    fn render_frame_graph_projection(
        &mut self,
        state: &EngineFrameGraph,
        target: &wgpu::Texture,
        camera: ResolvedCamera,
        include_background: bool,
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        if !matches!(projection, crate::frame_graph::ViewProjection::Camera | crate::frame_graph::ViewProjection::Stage) {
            return Err(EngineError::Store("Unsupported playback projection".into()));
        }
        let prepared = state.prepared.as_ref()
            .ok_or_else(|| EngineError::Store("Lowered scene is missing".into()))?;

        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let projection_camera = window.projection_camera.unwrap_or(document_camera);
        let mut layers = prepared.layers.clone();
        // Selection is editor state: stamped per view, never into the shared prepared scene.
        for (layer, id) in layers.iter_mut().zip(&prepared.layer_ids) {
            layer.layer.projection_camera =
                if layer.layer.projection == crate::doc::store::LayerProjection::TwoD {
                    document_camera
                } else {
                    projection_camera
                };
            layer.layer.outline = self.outline_id(*id);
        }

        self.stamp_frame_graph_window_feedback(&mut layers, window);
        let background = if include_background {
            state.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        for (layer, id) in layers.iter().zip(&prepared.layer_ids) {
            if layer.passes.is_empty() { continue; }
            let names: Vec<_> = layer.passes.iter().map(|pass| pass.plugin_id.as_str()).collect();
            self.ledger.claim("draw", format!("layer {}", id.0), format!("{} effect pass(es): {}", names.len(), names.join(", ")), std::time::Duration::ZERO);
        }
        let started = std::time::Instant::now();
        let before = self.surface_work();
        self.compositor.render_into_window(target, state.comp, camera, &layers, background, window)?;
        let work = self.surface_work();
        self.ledger.claim("draw", "compositor", format!(
            "{} layers into the window in {} run(s), {} bake(s) ({} reused), {} backdrop copy(ies), {} scene capture(s), {} light capture(s)",
            layers.len(), work.main_runs - before.main_runs, work.bakes - before.bakes,
            work.baked_hits - before.baked_hits, work.backdrop_copies - before.backdrop_copies,
            work.scene_captures - before.scene_captures, work.light_captures - before.light_captures,
        ), started.elapsed());
        Ok(())
    }

    fn frame_graph_feedback_replay_start(
        &mut self,
        state: &EngineFrameGraph,
        now: i64,
    ) -> Option<i64> {
        let seen = std::mem::take(&mut self.feedback_keys_seen);
        let mut unique = HashSet::new();
        let mut start: Option<i64> = None;
        for key in seen.into_iter().filter(|key| unique.insert(*key)) {
            let Some(have) = self.compositor.feedback_frame(key) else { continue };
            if have == now && !self.compositor.feedback_is_fresh(key) {
                continue;
            }
            let in_point = state.in_points.get(&key.layer).copied().unwrap_or(0);
            let from = self.compositor.feedback_restore(key, now - 1)
                .map_or(in_point, |checkpoint| checkpoint + 1);
            if from < now {
                start = Some(start.map_or(from, |current| current.min(from)));
            }
        }
        start
    }

    #[allow(clippy::too_many_arguments)]
    fn replay_frame_graph_feedback(
        &mut self,
        state: &mut EngineFrameGraph,
        start: i64,
        now: i64,
        camera: ResolvedCamera,
        include_background: bool,
        window: crate::render::compositor::Window,
        projection: crate::frame_graph::ViewProjection,
    ) -> Result<(), EngineError> {
        let replay_target = self.compositor.ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("motolii-frame-graph-feedback-replay"),
            size: wgpu::Extent3d {
                width: window.width,
                height: window.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: crate::render::compositor::PRESENTABLE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        for frame in start..now {
            self.pending_frame_copies.clear();
            self.feedback_keys_seen.clear();
            let at = RationalTime::try_from_frame(frame, state.fps)
                .map_err(|error| EngineError::Time(error.to_string()))?;
            state.evaluate_at(self, at, FrameQuality::Preview { scale: 1 })?;
            self.render_frame_graph_projection(
                state,
                &replay_target,
                camera,
                include_background,
                window,
                projection,
            )?;
        }
        self.pending_frame_copies.clear();
        self.feedback_keys_seen.clear();
        Ok(())
    }
}



fn direct<'a, T: 'static>(node: &GraphNode, inputs: &'a NodeInputs, index: usize) -> Result<&'a T, EngineError> { inputs.at(index).and_then(|value| value.downcast_ref::<T>()).ok_or_else(|| EngineError::Store(format!("FrameGraph {:?} has invalid input {index}", node.identity().kind))) }
fn store(error: crate::doc::store::StoreError) -> EngineError { EngineError::Store(error.to_string()) }
fn unsupported(kind: NodeKind) -> EngineError { EngineError::Store(format!("FrameGraph program executor does not support {kind:?}")) }
