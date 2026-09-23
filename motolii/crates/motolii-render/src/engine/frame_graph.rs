//! Product playback owner: compile once per revision, evaluate the semantic scene once per exact
//! comp time, then prepare it — below the semantic graph — once per evaluation and precision. A view,
//! its window or its zoom never evaluates the scene again.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreView};
use crate::frame_graph::{BlobAnalysisRequestValue, BlobAnalysisValue, CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, MediaExtentValue, NodeExecutor, NodeInputs, NodeKey, NodeKind, NodeValue, OverlayAnalysisValue, OverlaySetValue, SceneProgram, SceneValue, SolverPlanValue};

use super::frame_graph_scene::{GpuSceneValue, LegacyCameraSeam, Precision, Preparation};

pub mod tick;
#[cfg(test)]
mod tick_tests;
use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    program: SceneProgram,
    scene: NodeKey,
    solver: NodeKey,
    overlay: NodeKey,
    prepared: Option<Arc<GpuSceneValue>>,
    /// What `prepared` was made from: the semantic evaluation (its generation) and the precision.
    prepared_for: Option<(u64, Precision)>,
    comp: CompSpec,
    fps: crate::doc::store::Fps,
    background: [f32; 4],
    in_points: HashMap<LayerId, i64>,
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
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
        Ok(Self { graph: CompiledGraph::with_topology(revision, topology), program, scene, solver, overlay, prepared: None, prepared_for: None, comp, fps, background, in_points, frame: None, generation: 0, prepare_us: 0, measured: false })
    }
    /// The semantic scene is a function of the document and the time only.
    fn matches(&self, revision: GraphRevision, time: RationalTime) -> bool { self.graph.revision() == revision && self.frame.as_ref().is_some_and(|frame| frame.time() == time) }

    /// The semantic scene at `time`: the graph's evaluation, with no GPU preparation of its own.
    fn evaluate_scene(
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
        let overlays = evaluated.value(self.overlay).and_then(|value| value.downcast_ref::<OverlaySetValue>())
            .ok_or_else(|| EngineError::Store("FrameGraph overlay set has the wrong type".into()))?;
        engine.install_frame_graph_overlays(overlays);
        self.frame = Some(evaluated);
        Ok(())
    }

    /// The prepared world of the evaluated scene at `precision`: made once per evaluation and
    /// precision, whatever views read it.
    fn prepare(&mut self, engine: &mut Engine, precision: Precision) -> Result<(), EngineError> {
        if self.prepared.is_some() && self.prepared_for == Some((self.generation, precision)) { return Ok(()); }
        let frame = self.frame.as_ref().ok_or_else(|| EngineError::Store("FrameGraph scene is not evaluated".into()))?;
        let value = |key: NodeKey| frame.value(key).ok_or_else(|| EngineError::Store("FrameGraph root is missing".into()));
        let scene = value(self.scene)?.downcast_ref::<SceneValue>().ok_or_else(|| EngineError::Store("FrameGraph scene has the wrong type".into()))?;
        let authored = value(self.program.camera())?.downcast_ref::<ResolvedCamera>().copied().unwrap_or_default();
        let solver = value(self.solver)?.downcast_ref::<SolverPlanValue>().ok_or_else(|| EngineError::Store("FrameGraph solver has the wrong type".into()))?;
        let time = frame.time();
        engine.compositor.clock = Some([
            time.as_seconds_f64() as f32,
            self.fps.den() as f32 / self.fps.num() as f32,
            time.try_to_frame_round(self.fps).unwrap_or(0) as f32,
        ]);
        let mut prep = Preparation::new(self.comp, precision, LegacyCameraSeam::new(authored));
        let prepared = engine.prepare_gpu_scene_with_solver(scene, solver, &mut prep, time, self.fps)?;
        self.prepared = Some(Arc::new(prepared));
        self.prepared_for = Some((self.generation, precision));
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



    /// The members of the plates in the evaluated scene at `time` (for the invariant tests).
    #[cfg(test)]
    pub(crate) fn plate_member_ids(&mut self, doc: &motolii_edit::Document, time: RationalTime) -> Vec<LayerId> {
        let scene = self.frame_graph_editor_scene(&doc.view(), time).unwrap();
        scene.layers.iter().filter_map(|l| match &l.content {
            crate::frame_graph::SceneContentValue::Plate(plate) => Some(plate.members.iter().filter_map(|m| m.layer.as_ref().map(|l| l.layer)).collect::<Vec<_>>()),
            _ => None,
        }).flatten().collect()
    }

    /// The semantic scene at `time` (editor reads, the tick's first step): no GPU preparation.
    fn evaluated_frame_graph(&mut self, view: &StoreView<'_>, time: RationalTime, quality: FrameQuality) -> Result<EngineFrameGraph, EngineError> {
        let revision = GraphRevision::new(view.revision_key());
        let mut state = match self.frame_graph.take() { Some(state) if state.graph.revision() == revision => state, _ => EngineFrameGraph::new(view, revision)? };
        self.compositor.feedback_set_revision(view.revision_key());
        if !state.matches(revision, time) {
            if let Err(error) = state.evaluate_scene(self, time, quality) {
                self.frame_graph = Some(state);
                return Err(error);
            }
        }
        Ok(state)
    }

    /// The prepared world at `time` and `precision`, on the semantic scene of the same time.
    fn prepared_frame_graph(&mut self, view: &StoreView<'_>, time: RationalTime, quality: FrameQuality, precision: Precision) -> Result<EngineFrameGraph, EngineError> {
        let mut state = self.evaluated_frame_graph(view, time, quality)?;
        let started = std::time::Instant::now();
        let before = state.prepared_for;
        if let Err(error) = state.prepare(self, precision) {
            self.frame_graph = Some(state);
            return Err(error);
        }
        if state.prepared_for != before {
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
        }
        Ok(state)
    }

    pub(super) fn semantic_layer_for(&self, view: &StoreView<'_>, time: RationalTime, id: LayerId) -> Option<&crate::frame_graph::SceneLayerValue> {
        self.frame_graph_cached_scene(view, time)?.layer(id)
    }



    /// The earliest frame the histories read this tick must be replayed from (a jump).
    fn frame_graph_feedback_replay_start(
        &mut self,
        state: &EngineFrameGraph,
        now: i64,
    ) -> Option<i64> {
        let seen = std::mem::take(&mut self.compositor.feedback_seen);
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

}



fn direct<'a, T: 'static>(node: &GraphNode, inputs: &'a NodeInputs, index: usize) -> Result<&'a T, EngineError> { inputs.at(index).and_then(|value| value.downcast_ref::<T>()).ok_or_else(|| EngineError::Store(format!("FrameGraph {:?} has invalid input {index}", node.identity().kind))) }
fn store(error: crate::doc::store::StoreError) -> EngineError { EngineError::Store(error.to_string()) }
fn unsupported(kind: NodeKind) -> EngineError { EngineError::Store(format!("FrameGraph program executor does not support {kind:?}")) }
