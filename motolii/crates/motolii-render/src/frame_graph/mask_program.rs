use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{LayerId, MaskFrame, MaskMode, PropertyId, StoreError, StoreView};
use crate::picture::resolved::ResolvedMask;

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Debug, PartialEq)] pub struct MaskValue(pub ResolvedMask);
#[derive(Clone, Debug, PartialEq, Eq)] pub struct MaskBinding { pub layer: LayerId, pub masks: Vec<NodeKey> }
#[derive(Clone)] struct Recipe { mode: MaskMode, inverted: bool, mode_input: Option<usize>, inverted_input: Option<usize>, shape: usize, opacity: Option<usize>, expansion: Option<usize> }

#[derive(Debug)] pub enum MaskProgramError { Store(StoreError), InvalidInput(NodeKind) }
impl std::fmt::Display for MaskProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for MaskProgramError {}
impl From<StoreError> for MaskProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct MaskProgram { nodes: BTreeMap<NodeKey, GraphNode>, recipes: BTreeMap<NodeKey, Recipe>, bindings: BTreeMap<LayerId, MaskBinding> }
impl MaskProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram) -> Result<Self, MaskProgramError> {
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            let mut keys = Vec::new();
            for mask in view.masks(layer)? {
                let mut inputs = Vec::new();
                let mut add = |property: PropertyId| properties.node_for(layer, &property).map(|key| { let at = inputs.len(); inputs.push(key); at });
                let mode_input = add(PropertyId::mask_mode(mask.id));
                let inverted_input = add(PropertyId::mask_inverted(mask.id));
                let Some(shape) = add(PropertyId::mask_shape(mask.id)) else { continue };
                let opacity = add(PropertyId::mask_opacity(mask.id));
                let expansion = add(PropertyId::mask_expansion(mask.id));
                let mut identity = NodeIdentity::new(NodeKind::Mask, inputs);
                identity.parameters = [mask.mode.to_enum_value().to_be_bytes().as_slice(), &[u8::from(mask.inverted)]].concat();
                identity.time_dependency = TimeDependency::Exact;
                let node = GraphNode::new(identity);
                recipes.entry(node.key()).or_insert(Recipe { mode: mask.mode, inverted: mask.inverted, mode_input, inverted_input, shape, opacity, expansion });
                nodes.entry(node.key()).or_insert(node.clone()); keys.push(node.key());
            }
            if !keys.is_empty() { bindings.insert(layer, MaskBinding { layer, masks: keys }); }
        }
        Ok(Self { nodes, recipes, bindings })
    }
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<&MaskBinding> { self.bindings.get(&layer) }
    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, MaskProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let mode = match recipe.mode_input.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Enum(value)) => MaskMode::from_enum_value(*value).ok_or(MaskProgramError::InvalidInput(node.identity().kind))?, None => recipe.mode, _ => return Err(MaskProgramError::InvalidInput(node.identity().kind)) };
            let inverted = match recipe.inverted_input.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Bool(value)) => *value, None => recipe.inverted, _ => return Err(MaskProgramError::InvalidInput(node.identity().kind)) };
            let shape = match inputs.at(recipe.shape).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Path(path)) => path.clone(), _ => return Err(MaskProgramError::InvalidInput(node.identity().kind)) };
            let opacity = match recipe.opacity.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) => *value as f32, None => 1.0, _ => return Err(MaskProgramError::InvalidInput(node.identity().kind)) }.clamp(0.0, 1.0);
            let expansion = match recipe.expansion.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) if value.is_finite() => *value, None => 0.0, _ => return Err(MaskProgramError::InvalidInput(node.identity().kind)) };
            Ok(NodeValue::new(MaskValue(ResolvedMask { mode, inverted, opacity, expansion, shape, frame: MaskFrame::Layer })))
        })())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::eval::{Interp, Keyframe, KeyframeTrack, Path, PathVertex};
    use crate::doc::store::{LayerMeta, LayerSource, LayerTiming, Mask, MaskId};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError, SceneValue};
    use motolii_edit::{Document, Intent};
    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> { type Error = SceneProgramError; fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) } }

    #[test]
    fn mask_properties_are_graph_inputs_of_the_scene() {
        let mut doc = Document::new(); let layer = LayerId(1); let id = MaskId(4);
        let path = Path { closed: true, vertices: vec![PathVertex { point: [10.0, 20.0], in_tangent: [0.0; 2], out_tangent: [0.0; 2] }] };
        let shape = KeyframeTrack::try_from_keys(vec![Keyframe { t: crate::doc::core::RationalTime::ZERO, value: Value::Path(path.clone()), interp: Interp::Hold, spatial: None }]).unwrap();
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::AddMask { layer, mask: Mask { id, mode: MaskMode::Subtract, inverted: true }, shape },
            Intent::SetConstant { layer, property: PropertyId::mask_opacity(id), value: Value::F64(0.4) },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap(); let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap(); let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology); let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<SceneValue>()).unwrap();
        assert_eq!(scene.layers[0].masks[0].mode, MaskMode::Subtract); assert!(scene.layers[0].masks[0].inverted); assert_eq!(scene.layers[0].masks[0].opacity, 0.4); assert_eq!(scene.layers[0].masks[0].shape, path);
    }
}
