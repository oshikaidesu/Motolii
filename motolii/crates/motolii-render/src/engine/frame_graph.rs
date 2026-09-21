//! Product playback owner: compile once per revision, evaluate once per exact
//! comp time, prepare one GPU scene, then branch only for final projection.

use std::sync::Arc;

use crate::doc::core::{CompSpec, RationalTime, ResolvedCamera};
use crate::doc::store::{LayerId, StoreView};
use crate::frame_graph::{CompiledGraph, EvaluatedFrame, EvaluationContext, FrameQuality, Generation, GraphNode, GraphRevision, GraphTopology, NodeExecutor, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, SceneProgram, SceneValue, TimeDependency};
use crate::picture::resolved::ResolvedLayer;

use super::frame_graph_scene::GpuSceneValue;
use super::{Engine, EngineError};

pub(super) struct EngineFrameGraph {
    graph: CompiledGraph,
    program: SceneProgram,
    scene: NodeKey,
    camera: NodeKey,
    stage: NodeKey,
    comp: CompSpec,
    fps: crate::doc::store::Fps,
    background: [f32; 4],
    frame: Option<EvaluatedFrame>,
    generation: u64,
    prepare_us: u64,
    measured: bool,
}

impl EngineFrameGraph {
    fn new(view: &StoreView<'_>, revision: GraphRevision) -> Result<Self, EngineError> {
        let composition = view.composition().map_err(store)?.ok_or(EngineError::NoComposition)?;
        let (comp, fps, background) = (composition.spec(), composition.fps, composition.background);
        let program = SceneProgram::compile(view).map_err(|error| EngineError::Store(error.to_string()))?;
        let scene = program.scene().scene;
        let document_camera = program.camera();
        let mut nodes: Vec<_> = program.nodes().collect();
        let mut gpu_identity = NodeIdentity::new(NodeKind::GpuScene, vec![scene, document_camera]);
        gpu_identity.time_dependency = TimeDependency::Exact;
        let gpu = GraphNode::new(gpu_identity);
        let projection = |role| {
            let mut identity = NodeIdentity::new(NodeKind::CameraProjection, vec![gpu.key()]);
            identity.parameters = vec![role]; identity.time_dependency = TimeDependency::Exact;
            GraphNode::new(identity)
        };
        let camera = projection(0); let stage = projection(1);
        nodes.extend([gpu, camera.clone(), stage.clone()]);
        let topology = GraphTopology::try_new(nodes, vec![camera.key(), stage.key()]).map_err(|error| EngineError::Store(error.to_string()))?;
        Ok(Self { graph: CompiledGraph::with_topology(revision, topology), program, scene, camera: camera.key(), stage: stage.key(), comp, fps, background, frame: None, generation: 0, prepare_us: 0, measured: false })
    }
    fn matches(&self, revision: GraphRevision, time: RationalTime) -> bool { self.graph.revision() == revision && self.frame.as_ref().is_some_and(|frame| frame.time() == time) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum ProjectionRole { Camera, Stage }
#[derive(Clone)] struct Projection { scene: Arc<GpuSceneValue>, role: ProjectionRole }

struct ProgramExecutor<'a> { engine: &'a mut Engine, program: &'a SceneProgram, comp: CompSpec }
impl NodeExecutor for ProgramExecutor<'_> {
    type Error = EngineError;
    fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
        match self.program.execute(node, &inputs, &context) {
            Ok(value) => return Ok(value),
            Err(crate::frame_graph::SceneProgramError::Unsupported(_)) => {}
            Err(error) => return Err(EngineError::Store(error.to_string())),
        }
        match node.identity().kind {
            NodeKind::GpuScene => {
                let scene = direct::<SceneValue>(node, &inputs, 0)?;
                let camera = *direct::<ResolvedCamera>(node, &inputs, 1)?;
                let prepared = self.engine.prepare_gpu_scene(scene, self.comp, camera)?;
                Ok(NodeValue::new(Arc::new(prepared)))
            }
            NodeKind::CameraProjection => {
                let scene = inputs.at(0).and_then(|value| value.downcast_ref::<Arc<GpuSceneValue>>()).cloned().ok_or_else(|| unsupported(node.identity().kind))?;
                let role = match node.identity().parameters.as_slice() { [0] => ProjectionRole::Camera, [1] => ProjectionRole::Stage, _ => return Err(unsupported(node.identity().kind)) };
                Ok(NodeValue::new(Arc::new(Projection { scene, role })))
            }
            kind => Err(unsupported(kind)),
        }
    }
}

impl Engine {
    fn evaluated_frame_graph(&mut self, view: &StoreView<'_>, time: RationalTime, quality: FrameQuality) -> Result<EngineFrameGraph, EngineError> {
        let revision = GraphRevision::new(view.revision_key());
        let mut state = match self.frame_graph.take() { Some(state) if state.graph.revision() == revision => state, _ => EngineFrameGraph::new(view, revision)? };
        if !state.matches(revision, time) {
            state.generation += 1;
            let started = std::time::Instant::now();
            let evaluated = { let mut executor = ProgramExecutor { engine: self, program: &state.program, comp: state.comp }; state.graph.evaluate(&mut executor, time, quality, Generation::new(state.generation)) };
            state.prepare_us = started.elapsed().as_micros() as u64;
            state.measured = false;
            match evaluated { Ok(frame) => state.frame = Some(frame), Err(error) => { self.frame_graph = Some(state); return Err(error); } }
        }
        Ok(state)
    }

    pub fn resolved_for(&self, view: &StoreView<'_>, time: RationalTime) -> Option<Vec<ResolvedLayer>> {
        let state = self.frame_graph.as_ref()?;
        if !state.matches(GraphRevision::new(view.revision_key()), time) { return None; }
        let scene = state.frame.as_ref()?.value(state.scene)?.downcast_ref::<SceneValue>()?;
        let source_frame = time.try_to_frame_floor(state.fps).ok()?;
        Some(scene.layers.iter().map(|layer| ResolvedLayer { id: layer.layer, source: layer.source.clone(), placement: crate::doc::core::LayerPlacement { transform: layer.transform.affine, world_transform: Some(layer.transform.spatial), order: i32::from(layer.order), opacity: layer.opacity, z: layer.transform.spatial.translation.z, rotation_x: 0.0, rotation_y: 0.0, plane: None }, declared_size: [0.0; 2], source_frame, source_time: time, masks: layer.masks.clone(), effects: layer.effects.clone(), blend_mode: layer.blend, matte: layer.matte, clip_to_below: layer.clip_to_below, projection: layer.projection, flatten: layer.flatten, environment: layer.environment, depth: 0.0, ghost: false, copy: 0, after_effects: layer.after_effects.clone(), plate: None, averaged: 0, shape_stretch: [1.0, 1.0], glyph_offsets: None, flow_around: None }).collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render_frame_graph_into_window(&mut self, view: &StoreView<'_>, time: RationalTime, target: &wgpu::Texture, camera: ResolvedCamera, include_background: bool, outline: &[LayerId], window: crate::render::compositor::Window, projection: crate::frame_graph::ViewProjection) -> Result<(), EngineError> {
        let mut state = self.evaluated_frame_graph(view, time, FrameQuality::Preview { scale: 1 })?;
        let root = match projection { crate::frame_graph::ViewProjection::Camera => state.camera, crate::frame_graph::ViewProjection::Stage => state.stage, _ => { self.frame_graph = Some(state); return Err(EngineError::Store("Unsupported playback projection".into())); } };
        let projected = state.frame.as_ref().and_then(|frame| frame.value(root)).and_then(|value| value.downcast_ref::<Arc<Projection>>()).cloned().ok_or_else(|| EngineError::Store("FrameGraph projection is missing".into()))?;
        let expected = if projection == crate::frame_graph::ViewProjection::Camera { ProjectionRole::Camera } else { ProjectionRole::Stage };
        if projected.role != expected { self.frame_graph = Some(state); return Err(EngineError::Store("FrameGraph projection role mismatch".into())); }
        let document_camera = state.frame.as_ref().and_then(|frame| frame.value(state.program.camera())).and_then(|value| value.downcast_ref::<ResolvedCamera>()).copied().unwrap_or_default();
        let frame_start = std::time::Instant::now(); self.compositor.measurement = Default::default();
        if !state.measured { self.compositor.measurement.resolve_us = state.prepare_us; state.measured = true; }
        self.outline_layers = outline.iter().copied().take(255).collect(); self.outline_order = self.outline_layers.clone();
        let projection_camera = window.projection_camera.unwrap_or(document_camera);
        let mut layers = projected.scene.layers.clone();
        for layer in &mut layers { layer.layer.projection_camera = if layer.layer.projection == crate::doc::store::LayerProjection::TwoD { document_camera } else { projection_camera }; }
        let background = if include_background { state.background } else { crate::render::compositor::NO_BACKGROUND };
        let drawn = self.compositor.render_into_window(target, state.comp, camera, &layers, background, window);
        self.outline_layers.clear(); self.compositor.measurement.total_us = frame_start.elapsed().as_micros() as u64; self.frame_graph = Some(state); Ok(drawn?)
    }
}

fn direct<'a, T: 'static>(node: &GraphNode, inputs: &'a NodeInputs, index: usize) -> Result<&'a T, EngineError> { inputs.at(index).and_then(|value| value.downcast_ref::<T>()).ok_or_else(|| EngineError::Store(format!("FrameGraph {:?} has invalid input {index}", node.identity().kind))) }
fn store(error: crate::doc::store::StoreError) -> EngineError { EngineError::Store(error.to_string()) }
fn unsupported(kind: NodeKind) -> EngineError { EngineError::Store(format!("FrameGraph program executor does not support {kind:?}")) }
