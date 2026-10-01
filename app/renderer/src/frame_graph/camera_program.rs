use std::collections::BTreeMap;

use crate::doc::core::{CompSpec, ResolvedCamera};
use crate::doc::eval::Value;
use crate::doc::store::{property, Fps, LayerSource, LayerTiming, PropertyId, StoreError, StoreView};

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency, TransformProgram, TransformValue};

const ROWS: [&str; 9] = [property::CAMERA_CENTER, property::CAMERA_TARGET_Z, property::CAMERA_TARGET, property::CAMERA_FRAMING, property::CAMERA_ORBIT, property::CAMERA_DISTANCE, property::CAMERA_ZOOM, property::CAMERA_ROLL, property::CAMERA_NEAR_FADE];

#[derive(Clone)]
struct CameraPlan { order: i16, timing: LayerTiming, hidden: bool, solo: bool, props: [Option<usize>; 9] }

#[derive(Clone)]
struct Recipe { cameras: Vec<CameraPlan>, worlds: BTreeMap<u64, usize>, fps: Fps, comp: CompSpec }

#[derive(Debug)]
pub enum CameraProgramError { Store(StoreError), InvalidInput(NodeKind) }
impl std::fmt::Display for CameraProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for CameraProgramError {}
impl From<StoreError> for CameraProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct CameraProgram { node: GraphNode, recipe: Recipe }

impl CameraProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, transforms: &TransformProgram) -> Result<Self, CameraProgramError> {
        let composition = view.composition()?;
        let fps = composition.as_ref().map_or(Fps::try_new(30, 1).expect("30fps"), |comp| comp.fps);
        let comp = composition.map_or(CompSpec { width: 1920, height: 1080 }, |comp| comp.spec());
        let mut inputs = Vec::new(); let mut input_index = BTreeMap::new();
        let mut input = |key: NodeKey| *input_index.entry(key).or_insert_with(|| { let at = inputs.len(); inputs.push(key); at });
        let mut cameras = Vec::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if meta.source != LayerSource::Camera { continue; }
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let mut props = [None; 9];
            for (row, name) in ROWS.iter().enumerate() { props[row] = properties.node_for(layer, &PropertyId::new(name).expect("known camera property")).map(&mut input); }
            cameras.push(CameraPlan { order: meta.order, timing: meta.timing, hidden: attrs.hidden, solo: attrs.solo, props });
        }
        let mut worlds = BTreeMap::new();
        for binding in transforms.bindings() { worlds.insert(binding.layer.0, input(binding.world)); }
        let mut identity = NodeIdentity::new(NodeKind::Camera, inputs);
        identity.parameters = cameras.iter().flat_map(|camera| camera.order.to_be_bytes().into_iter().chain(camera.timing.start.to_be_bytes()).chain(camera.timing.duration.to_be_bytes()).chain([u8::from(camera.hidden), u8::from(camera.solo)])).collect();
        identity.time_dependency = TimeDependency::Exact;
        Ok(Self { node: GraphNode::new(identity), recipe: Recipe { cameras, worlds, fps, comp } })
    }

    pub fn node(&self) -> GraphNode { self.node.clone() }
    pub fn key(&self) -> NodeKey { self.node.key() }
    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, CameraProgramError>> {
        (node.key() == self.node.key()).then(|| evaluate(&self.recipe, inputs, context).map(NodeValue::new))
    }
}

fn evaluated<'a>(camera: &'a CameraPlan, row: usize, inputs: &'a NodeInputs) -> Option<&'a Value> { camera.props[row].and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref()) }
fn scalar(camera: &CameraPlan, row: usize, inputs: &NodeInputs, default: f32) -> f32 { match evaluated(camera, row, inputs) { Some(Value::F64(value)) if value.is_finite() => *value as f32, _ => default } }
fn pair(camera: &CameraPlan, row: usize, inputs: &NodeInputs, default: [f32; 2]) -> [f32; 2] { match evaluated(camera, row, inputs) { Some(Value::Vec2(value)) => [value[0] as f32, value[1] as f32], _ => default } }

fn evaluate(recipe: &Recipe, inputs: &NodeInputs, context: &EvaluationContext) -> Result<ResolvedCamera, CameraProgramError> {
    let frame = context.time.try_to_frame_floor(recipe.fps).unwrap_or(0);
    let any_solo = recipe.cameras.iter().any(|camera| camera.solo && !camera.hidden && camera.timing.covers(frame));
    let Some(camera) = recipe.cameras.iter().filter(|camera| !camera.hidden && camera.timing.covers(frame) && (!any_solo || camera.solo)).max_by_key(|camera| camera.order) else { return Ok(ResolvedCamera::default()); };
    let mut resolved = ResolvedCamera { center: pair(camera, 0, inputs, [0.0, 0.0]), target_z: scalar(camera, 1, inputs, 0.0), orbit_degrees: pair(camera, 4, inputs, [0.0, 0.0]), distance_scale: scalar(camera, 5, inputs, 1.0).max(0.01), zoom: scalar(camera, 6, inputs, 1.0), roll_degrees: scalar(camera, 7, inputs, 0.0), near_fade: scalar(camera, 8, inputs, 0.0).max(0.0) };
    if let Some(Value::LayerId(target)) = evaluated(camera, 2, inputs) {
        if let Some(index) = recipe.worlds.get(target).copied() {
            if let Some(world) = inputs.at(index).and_then(|value| value.downcast_ref::<TransformValue>()) {
                let point = world.spatial.transform_point3(glam::Vec3::ZERO);
                resolved.center = [point.x - recipe.comp.width as f32 * 0.5, point.y - recipe.comp.height as f32 * 0.5]; resolved.target_z = point.z;
            }
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::Value;
    use crate::doc::store::{Composition, LayerMeta, PropertyId};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn camera_properties_and_target_world_are_graph_inputs() {
        let mut doc = Document::new();
        let target = crate::doc::store::LayerId(1); let camera = crate::doc::store::LayerId(2);
        doc.apply_all([
            Intent::SetComposition(Composition { width: 640, height: 480, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] }),
            Intent::AddLayer(target), Intent::SetMeta { layer: target, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetConstant { layer: target, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2([400.0, 300.0]) },
            Intent::AddLayer(camera), Intent::SetMeta { layer: camera, meta: LayerMeta { source: LayerSource::Camera, order: 1, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_TARGET).unwrap(), value: Value::LayerId(target.0) },
            Intent::SetConstant { layer: camera, property: PropertyId::new(property::CAMERA_ZOOM).unwrap(), value: Value::F64(2.0) },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.camera();
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let actual = frame.value(root).and_then(|value| value.downcast_ref::<ResolvedCamera>()).unwrap();
        let expected = crate::picture::resolve::camera::resolve_camera(&doc.view(), crate::doc::core::RationalTime::ZERO).unwrap();
        assert_eq!(*actual, expected);
    }
}
