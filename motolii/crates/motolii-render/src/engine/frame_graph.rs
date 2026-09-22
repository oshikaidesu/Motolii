//! Product playback owner: compile once per revision, evaluate once per exact
//! comp time, prepare one GPU scene, then branch only for final projection.

use std::collections::{HashMap, HashSet};

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreView};
use crate::frame_graph::{BlobAnalysisRequestValue, BlobAnalysisValue, CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, MediaExtentValue, NodeExecutor, NodeInputs, NodeKey, NodeKind, NodeValue, OverlayAnalysisValue, OverlaySetValue, SceneProgram, SceneValue, SolverPlanValue};
use crate::picture::resolved::ResolvedLayer;

use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    program: SceneProgram,
    scene: NodeKey,
    solver: NodeKey,
    overlay: NodeKey,
    comp: CompSpec,
    fps: crate::doc::store::Fps,
    background: [f32; 4],
    in_points: HashMap<LayerId, i64>,
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
    gpu_resources: crate::gpu_exec::GpuResourceGraph,
    gpu_lowerer: crate::gpu_exec::LogicalGpuLowerer,
    gpu_content: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentContent>,
    gpu_placement: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentPlacement>,
    gpu_effects: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentEffectChain>,
    gpu_snapshots: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentSnapshot>,
    gpu_processed: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentProcessedLayer>,
    gpu_composites: crate::gpu_exec::GpuResourceStore<crate::gpu_exec::ResidentCompositeLayer>,
    gpu_scene_root: Option<crate::gpu_exec::GpuSceneRoot>,
    gpu_present_sink: Option<crate::gpu_exec::GpuSinkRoot>,
    gpu_readback_sink: Option<crate::gpu_exec::GpuSinkRoot>,
    gpu_operations: crate::gpu_exec::GpuOperationTable<crate::gpu_exec::EngineGpuOperation>,
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
        let nodes: Vec<_> = program.nodes().collect();
        let topology = GraphTopology::try_new(nodes, vec![scene, document_camera, solver, overlay])
            .map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(Self { graph: CompiledGraph::with_topology(revision, topology), program, scene, solver, overlay, comp, fps, background, in_points, frame: None, generation: 0, prepare_us: 0, measured: false, gpu_resources: Default::default(), gpu_lowerer: Default::default(), gpu_content: Default::default(), gpu_placement: Default::default(), gpu_effects: Default::default(), gpu_snapshots: Default::default(), gpu_processed: Default::default(), gpu_composites: Default::default(), gpu_scene_root: None, gpu_present_sink: None, gpu_readback_sink: None, gpu_operations: Default::default() })
    }
    fn matches(&self, revision: GraphRevision, time: RationalTime) -> bool { self.graph.revision() == revision && self.frame.as_ref().is_some_and(|frame| frame.time() == time) }
    fn plan_sink(&self, kind: crate::gpu_exec::GpuSinkKind) -> Result<crate::gpu_exec::GpuExecutionPlan, EngineError> {
        let sink = match kind {
            crate::gpu_exec::GpuSinkKind::Present => self.gpu_present_sink,
            crate::gpu_exec::GpuSinkKind::Readback => self.gpu_readback_sink,
        }.ok_or_else(|| EngineError::Store("GPU sink root missing".into()))?;
        crate::gpu_exec::GpuPlanner.plan(&self.gpu_resources, [sink.pass])
            .map_err(|error| EngineError::Store(format!("GPU execution plan: {error:?}")))
    }

    fn execute_pre_sink(
        &mut self,
        plan: &crate::gpu_exec::GpuExecutionPlan,
        sink: crate::gpu_exec::GpuSinkKind,
    ) -> Result<crate::gpu_exec::GpuExecutionStats, EngineError> {
        let sink_pass = match sink {
            crate::gpu_exec::GpuSinkKind::Present => self.gpu_present_sink,
            crate::gpu_exec::GpuSinkKind::Readback => self.gpu_readback_sink,
        }.ok_or_else(|| EngineError::Store("GPU sink root missing".into()))?.pass;
        let mut pre = plan.clone();
        pre.passes.retain(|pass| *pass != sink_pass);
        let mut backend = crate::gpu_exec::EngineGpuBackend { operations: &self.gpu_operations };
        crate::gpu_exec::GpuExecutor.execute(
            &mut self.gpu_resources,
            &pre,
            self.generation,
            &mut backend,
        ).map_err(|error| EngineError::Store(format!("GPU executor: {error:?}")))
    }


    fn evaluate_at(
        &mut self,
        engine: &mut Engine,
        time: RationalTime,
        quality: FrameQuality,
        force_gpu: bool,
    ) -> Result<(), EngineError> {
        let _ = force_gpu; // GPU work is no longer represented by a semantic FrameGraph node.
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
        if let Some(scene) = evaluated.value(self.scene).and_then(|value| value.downcast_ref::<SceneValue>()) {
            let adapter = crate::gpu_exec::SemanticGpuAdapter::new(&self.program, &evaluated);
            let inputs = adapter.contributions(scene).map_err(|error| EngineError::Store(format!("GPU semantic adapter: {error:?}")))?;
            // Phase 1: every contribution output identity exists before any
            // matte/clip/cross-layer dependency is connected.
            for input in &inputs {
                self.gpu_lowerer.declare_contribution(&mut self.gpu_resources, input)
                    .map_err(|error| EngineError::Store(format!("GPU contribution declaration: {error:?}")))?;
            }
            // Phase 2: lower producers and cross-contribution edges.
            let mut ordered_outputs = Vec::with_capacity(inputs.len());
            for (layer, input) in scene.layers.iter().zip(inputs.iter()) {
                let resources = self.gpu_lowerer.lower_contribution(&mut self.gpu_resources, input).map_err(|error| EngineError::Store(format!("GPU logical lowering: {error:?}")))?;
                if let Some(output) = resources.final_image_or_geometry {
                    ordered_outputs.push(output);
                }
                let placement_version = self.gpu_resources.version(resources.placement).ok_or_else(|| EngineError::Store("GPU placement resource version missing".into()))?;
                let _ = crate::gpu_exec::resident_placement(
                    &mut self.gpu_placement,
                    resources.placement,
                    placement_version,
                    self.generation,
                    layer,
                );
                self.gpu_resources.mark_resident(resources.placement, placement_version, self.generation);
                if let Some((effect_key, effect_version)) = crate::gpu_exec::effect_chain_key(layer) {
                    let _ = crate::gpu_exec::resident_effect_chain(
                        &mut self.gpu_effects,
                        effect_key,
                        effect_version,
                        self.generation,
                        layer,
                        [self.comp.width, self.comp.height],
                    );
                }
                if let Some(content_key) = resources.content {
                    let version = self.gpu_resources.version(content_key).ok_or_else(|| EngineError::Store("GPU content resource version missing".into()))?;
                    let _ = engine.gpu_resident_content(
                        &mut self.gpu_content,
                        content_key,
                        version,
                        self.generation,
                        layer,
                        self.comp,
                        time,
                    )?;
                    self.gpu_resources.mark_resident(content_key, version, self.generation);
                }
            }
            let scene_root = crate::gpu_exec::lower_scene_root(
                &mut self.gpu_resources,
                self.scene,
                &ordered_outputs,
            ).map_err(|error| EngineError::Store(format!("GPU scene root: {error:?}")))?;
            let present = crate::gpu_exec::lower_sink(
                &mut self.gpu_resources,
                self.scene,
                scene_root,
                crate::gpu_exec::GpuSinkKind::Present,
            ).map_err(|error| EngineError::Store(format!("GPU present sink: {error:?}")))?;
            let readback = crate::gpu_exec::lower_sink(
                &mut self.gpu_resources,
                self.scene,
                scene_root,
                crate::gpu_exec::GpuSinkKind::Readback,
            ).map_err(|error| EngineError::Store(format!("GPU readback sink: {error:?}")))?;
            self.gpu_scene_root = Some(scene_root);
            self.gpu_present_sink = Some(present);
            self.gpu_readback_sink = Some(readback);
            // Concrete resident backends have already materialized ordinary
            // resources during lowering. Register every logical pass so the
            // executor validates the complete plan instead of rediscovering work.
            let pass_ops: Vec<_> = self.gpu_resources.passes().map(|(key, pass)| {
                let op = match pass.identity.kind {
                    crate::gpu_exec::GpuPassKind::Present => crate::gpu_exec::EngineGpuOperation::Present,
                    crate::gpu_exec::GpuPassKind::Readback => crate::gpu_exec::EngineGpuOperation::Readback,
                    _ => crate::gpu_exec::EngineGpuOperation::Resident,
                };
                (key, op)
            }).collect();
            for (key, op) in pass_ops {
                self.gpu_operations.install(key, op);
            }
            self.gpu_resources.retire_temporal(self.generation);
        }
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
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
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
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
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
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
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
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let plan = state.plan_sink(crate::gpu_exec::GpuSinkKind::Readback)?;
        let _stats = state.execute_pre_sink(&plan, crate::gpu_exec::GpuSinkKind::Readback)?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let camera = camera_override.unwrap_or(document_camera);
        let mut layers = self.prepare_frame_graph_layers(&state, time, document_camera)?;
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
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Export)?;
        let _plan = state.plan_sink(crate::gpu_exec::GpuSinkKind::Readback)?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let mut layers = self.prepare_frame_graph_layers(&state, time, document_camera)?;
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
        self.gpu_history.begin_frame();
        if !state.matches(revision, time) {
            let started = std::time::Instant::now();
            if let Err(error) = state.evaluate_at(self, time, quality, false) {
                self.frame_graph = Some(state);
                return Err(error);
            }
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
        }
        Ok(state)
    }

    pub(in crate::engine) fn frame_graph_plate(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        plate: &crate::frame_graph::ScenePlateValue,
        comp: crate::doc::core::CompSpec,
        camera: crate::render::engine::ResolvedCamera,
    ) -> Result<Option<(crate::render::compositor::LayerContent, [f32; 2])>, EngineError> {
        let mut state = self.frame_graph.take().ok_or_else(|| EngineError::Store("frame graph state missing".into()))?;
        let result = self.gpu_plate(&mut state.gpu_composites, source, plate, comp, camera, state.generation);
        self.frame_graph = Some(state);
        result
    }

    pub(in crate::engine) fn frame_graph_matte(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        target: &crate::render::compositor::LayerWithPasses,
        matte_source: &crate::render::compositor::LayerWithPasses,
        comp: crate::doc::core::CompSpec,
        camera: crate::render::engine::ResolvedCamera,
    ) -> Result<crate::render::compositor::LayerWithPasses, EngineError> {
        let mut state = self.frame_graph.take().ok_or_else(|| EngineError::Store("frame graph state missing".into()))?;
        let result = self.gpu_matte(&mut state.gpu_composites, source, target, matte_source, comp, camera, state.generation);
        self.frame_graph = Some(state);
        result
    }

    pub(in crate::engine) fn frame_graph_process_mask_flatten(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        layer: crate::render::compositor::Layer,
        natural: [f32; 2],
        frame: Option<crate::render::compositor::effects::vism::ImageFrame>,
        comp: crate::doc::core::CompSpec,
        camera: crate::render::engine::ResolvedCamera,
    ) -> Result<crate::render::compositor::Layer, EngineError> {
        let mut state = self.frame_graph.take().ok_or_else(|| EngineError::Store("frame graph state missing".into()))?;
        let result = self.gpu_process_mask_flatten(
            &mut state.gpu_processed,
            source,
            layer,
            natural,
            frame,
            comp,
            camera,
            state.generation,
        );
        self.frame_graph = Some(state);
        result
    }

    pub(in crate::engine) fn frame_graph_snapshot_rows(
        &mut self,
        source: &crate::frame_graph::SceneLayerValue,
        comp: crate::doc::core::CompSpec,
        camera: crate::render::engine::ResolvedCamera,
    ) -> Result<Vec<Vec<crate::render::compositor::GpuTexture2D>>, EngineError> {
        let Some(state) = self.frame_graph.as_mut() else { return Ok(Vec::new()) };
        let mut rows = Vec::with_capacity(source.image_sources.len());
        for (pass_slot, row) in source.image_sources.iter().enumerate() {
            let Some(effect) = source.effect_keys.get(pass_slot).copied() else {
                rows.push(Vec::new());
                continue;
            };
            let mut textures = Vec::with_capacity(row.len());
            for (image_slot, image) in row.iter().enumerate() {
                let identity = crate::gpu_exec::snapshot_identity(effect, pass_slot as u32, image_slot as u32);
                let key = identity.key();
                let version = crate::gpu_exec::snapshot_version(image);
                let Some(texture) = self.gpu_snapshot_source(
                    &mut state.gpu_snapshots,
                    key,
                    version,
                    state.generation,
                    image,
                    comp,
                    camera,
                )? else {
                    textures.clear();
                    break;
                };
                state.gpu_resources.mark_resident(key, version, state.generation);
                textures.push(texture);
            }
            rows.push(textures);
        }
        Ok(rows)
    }

    pub(in crate::engine) fn frame_graph_resident_effects(
        &self,
        source: &crate::frame_graph::SceneLayerValue,
    ) -> Option<crate::gpu_exec::ResidentEffectChain> {
        let state = self.frame_graph.as_ref()?;
        let (key, version) = crate::gpu_exec::effect_chain_key(source)?;
        state.gpu_effects.current(key, version).cloned()
    }

    pub(in crate::engine) fn frame_graph_resident_placement(
        &self,
        source: &crate::frame_graph::SceneLayerValue,
    ) -> Option<crate::gpu_exec::ResidentPlacement> {
        let state = self.frame_graph.as_ref()?;
        let node = state.program.transforms().binding(source.layer)?.world;
        let identity = crate::gpu_exec::GpuResourceIdentity::semantic(
            node,
            crate::gpu_exec::GpuResourceClass::Placement,
            source.instance,
        );
        state.gpu_placement.get_any(identity.key()).map(|(_, value)| *value)
    }

    pub(in crate::engine) fn frame_graph_resident_content(
        &self,
        source: &crate::frame_graph::SceneLayerValue,
    ) -> Option<crate::gpu_exec::ResidentContent> {
        let state = self.frame_graph.as_ref()?;
        let node = state.program.content().binding(source.layer)?.content.or(source.content_key)?;
        let identity = crate::gpu_exec::GpuResourceIdentity::semantic(
            node,
            crate::gpu_exec::GpuResourceClass::Content,
            source.instance,
        );
        state.gpu_content.get_any(identity.key()).map(|(_, value)| value.clone())
    }

    pub(in crate::engine) fn frame_graph_gpu_stats(&self) -> Option<crate::gpu_exec::GpuGraphStats> {
        self.frame_graph.as_ref().map(|state| state.gpu_resources.stats())
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
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let frame_start = std::time::Instant::now();
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
            self.gpu_history.begin_frame();
            state.evaluate_at(self, time, FrameQuality::Preview { scale: 1 }, true)?;
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
        self.frame_graph = Some(state);
        Ok(())
    }

    fn prepare_frame_graph_layers(
        &mut self,
        state: &EngineFrameGraph,
        time: RationalTime,
        document_camera: ResolvedCamera,
    ) -> Result<Vec<crate::render::compositor::LayerWithPasses>, EngineError> {
        let frame = state.frame.as_ref().ok_or_else(|| EngineError::Store("FrameGraph evaluated frame is missing".into()))?;
        let scene = frame.value(state.scene)
            .and_then(|value| value.downcast_ref::<SceneValue>())
            .ok_or_else(|| EngineError::Store("FrameGraph semantic scene is missing".into()))?;
        let solver = frame.value(state.solver)
            .and_then(|value| value.downcast_ref::<SolverPlanValue>())
            .ok_or_else(|| EngineError::Store("FrameGraph solver value is missing".into()))?;
        let overlays = frame.value(state.overlay)
            .and_then(|value| value.downcast_ref::<OverlaySetValue>())
            .cloned()
            .ok_or_else(|| EngineError::Store("FrameGraph overlay value is missing".into()))?;
        self.install_frame_graph_overlays(&overlays);
        let frame_number = time.try_to_frame_round(state.fps).unwrap_or(0) as f32;
        self.compositor.clock = Some([
            time.as_seconds_f64() as f32,
            state.fps.den() as f32 / state.fps.num() as f32,
            frame_number,
        ]);
        Ok(self.gpu_executable_scene_with_solver(
            scene,
            solver,
            state.comp,
            document_camera,
            time,
            state.fps,
        )?.layers)
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
        let plan = state.plan_sink(crate::gpu_exec::GpuSinkKind::Present)?;
        let _stats = state.execute_pre_sink(&plan, crate::gpu_exec::GpuSinkKind::Present)?;
        let document_camera = state.frame.as_ref()
            .and_then(|frame| frame.value(state.program.camera()))
            .and_then(|value| value.downcast_ref::<ResolvedCamera>())
            .copied()
            .unwrap_or_default();
        let projection_camera = window.projection_camera.unwrap_or(document_camera);
        let mut layers = self.prepare_frame_graph_layers(state, state.frame.as_ref().map(|f| f.time()).unwrap_or(RationalTime::ZERO), document_camera)?;
        for layer in &mut layers {
            layer.layer.projection_camera =
                if layer.layer.projection == crate::doc::store::LayerProjection::TwoD {
                    document_camera
                } else {
                    projection_camera
                };
        }
        self.stamp_frame_graph_window_feedback(&mut layers, window);
        let background = if include_background {
            state.background
        } else {
            crate::render::compositor::NO_BACKGROUND
        };
        self.compositor.render_into_window(target, state.comp, camera, &layers, background, window)?;
        Ok(())
    }

    fn frame_graph_feedback_replay_start(
        &mut self,
        state: &EngineFrameGraph,
        now: i64,
    ) -> Option<i64> {
        let seen = self.gpu_history.seen().collect::<Vec<_>>();
        self.gpu_history.begin_frame();
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
            self.gpu_history.begin_frame();
            let at = RationalTime::try_from_frame(frame, state.fps)
                .map_err(|error| EngineError::Time(error.to_string()))?;
            state.evaluate_at(self, at, FrameQuality::Preview { scale: 1 }, true)?;
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
        self.gpu_history.begin_frame();
        Ok(())
    }
}


pub(super) fn resolved_layers_from_scene(
    scene: &SceneValue,
    time: RationalTime,
    fps: crate::doc::store::Fps,
) -> Vec<ResolvedLayer> {
    scene.layers.iter().map(|layer| {
        let source_time = match &layer.content {
            crate::frame_graph::SceneContentValue::Media { time, .. } => *time,
            _ => time,
        };
        let source_frame = source_time.try_to_frame_floor(fps).unwrap_or(0);
        let (plate, averaged) = match &layer.content {
            crate::frame_graph::SceneContentValue::Plate(plate) => (
                plate.owner,
                if plate.average { plate.members.len().try_into().unwrap_or(u32::MAX) } else { 0 },
            ),
            _ => (None, 0),
        };
        ResolvedLayer {
            id: layer.layer,
            source: layer.source.clone(),
            placement: crate::doc::core::LayerPlacement {
                transform: layer.transform.affine,
                world_transform: Some(layer.transform.spatial),
                order: i32::from(layer.order),
                opacity: layer.opacity,
                z: layer.transform.spatial.translation.z,
                rotation_x: 0.0,
                rotation_y: 0.0,
                plane: None,
            },
            declared_size: [0.0; 2],
            source_frame,
            source_time,
            masks: layer.masks.clone(),
            effects: layer.effects.clone(),
            blend_mode: layer.blend,
            matte: layer.matte,
            clip_to_below: layer.clip_to_below,
            projection: layer.projection,
            flatten: layer.flatten,
            environment: layer.environment,
            depth: layer.depth,
            ghost: layer.ghost,
            copy: layer.instance,
            after_effects: layer.after_effects.clone(),
            plate,
            averaged,
            shape_stretch: layer.shape_stretch,
            // These meanings are already baked into TextFlow/TextShape values.
            // The compatibility ResolvedLayer projection must not re-run them.
            glyph_offsets: None,
            flow_around: None,
        }
    }).collect()
}

fn direct<'a, T: 'static>(node: &GraphNode, inputs: &'a NodeInputs, index: usize) -> Result<&'a T, EngineError> { inputs.at(index).and_then(|value| value.downcast_ref::<T>()).ok_or_else(|| EngineError::Store(format!("FrameGraph {:?} has invalid input {index}", node.identity().kind))) }
fn store(error: crate::doc::store::StoreError) -> EngineError { EngineError::Store(error.to_string()) }
fn unsupported(kind: NodeKind) -> EngineError { EngineError::Store(format!("FrameGraph program executor does not support {kind:?}")) }
