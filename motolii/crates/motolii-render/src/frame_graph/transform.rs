use std::collections::{BTreeMap, BTreeSet};

use crate::doc::core::LayerPlacement;
use crate::doc::eval::Value;
use crate::doc::store::{property, LayerId, PropertyId, StoreError, StoreView};

use super::{EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

const ROWS: [&str; 10] = [property::POSITION, property::ANCHOR, property::SCALE, property::ROTATION, property::SKEW, property::SKEW_AXIS, property::POSITION_Z, property::ROTATION_X, property::ROTATION_Y, property::SCALE_Z];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformValue { pub affine: glam::Affine2, pub spatial: glam::Affine3A }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransformBinding { pub layer: LayerId, pub local: NodeKey, pub world: NodeKey }

#[derive(Clone)]
enum Recipe { Local { slots: [Option<usize>; 10], flow: Option<(usize, usize)> }, World { parent: bool } }

#[derive(Debug)]
pub enum TransformProgramError { Store(StoreError), Cycle(LayerId), InvalidInput(NodeKind) }
impl std::fmt::Display for TransformProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for TransformProgramError {}
impl From<StoreError> for TransformProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct TransformProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, TransformBinding>,
}

impl TransformProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, flow: &FlowProgram) -> Result<Self, TransformProgramError> {
        let layers = view.layers();
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut local = BTreeMap::new();
        let mut parents = BTreeMap::new();
        for layer in layers.iter().copied() {
            let mut inputs = Vec::new();
            let mut slots = [None; 10];
            let mut mask = 0u16;
            for (row, name) in ROWS.iter().enumerate() {
                let property = PropertyId::new(name).expect("known transform property");
                if let Some(key) = properties.node_for(layer, &property) {
                    slots[row] = Some(inputs.len());
                    inputs.push(key);
                    mask |= 1 << row;
                }
            }
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let flow_binding = attrs.parent.and_then(|parent| view.meta(parent).ok().flatten().filter(|meta| meta.source == crate::doc::store::LayerSource::Group))
                .and_then(|_| flow.binding(layer));
            let flow_input = flow_binding.map(|binding| { let at = inputs.len(); inputs.push(flow.key()); (at, binding.index) });
            let mut identity = NodeIdentity::new(NodeKind::Transform, inputs);
            identity.parameters = mask.to_be_bytes().into_iter().chain(flow_binding.map(|binding| binding.index as u32).unwrap_or(u32::MAX).to_be_bytes()).collect();
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            local.insert(layer, node.key());
            recipes.entry(node.key()).or_insert(Recipe::Local { slots, flow: flow_input });
            nodes.entry(node.key()).or_insert(node);
            parents.insert(layer, attrs.parent.filter(|parent| layers.contains(parent)));
        }
        let mut world = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        fn build(layer: LayerId, local: &BTreeMap<LayerId, NodeKey>, parents: &BTreeMap<LayerId, Option<LayerId>>, world: &mut BTreeMap<LayerId, NodeKey>, visiting: &mut BTreeSet<LayerId>, nodes: &mut BTreeMap<NodeKey, GraphNode>, recipes: &mut BTreeMap<NodeKey, Recipe>) -> Result<NodeKey, TransformProgramError> {
            if let Some(key) = world.get(&layer) { return Ok(*key); }
            if !visiting.insert(layer) { return Err(TransformProgramError::Cycle(layer)); }
            let parent = match parents[&layer] { Some(parent) => Some(build(parent, local, parents, world, visiting, nodes, recipes)?), None => None };
            let mut inputs = Vec::new();
            if let Some(parent) = parent { inputs.push(parent); }
            inputs.push(local[&layer]);
            let mut identity = NodeIdentity::new(NodeKind::WorldTransform, inputs);
            identity.parameters = vec![u8::from(parent.is_some())];
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            nodes.entry(key).or_insert(node);
            recipes.entry(key).or_insert(Recipe::World { parent: parent.is_some() });
            world.insert(layer, key);
            visiting.remove(&layer);
            Ok(key)
        }
        for layer in layers.iter().copied() { build(layer, &local, &parents, &mut world, &mut visiting, &mut nodes, &mut recipes)?; }
        let bindings = layers.into_iter().map(|layer| (layer, TransformBinding { layer, local: local[&layer], world: world[&layer] })).collect();
        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = TransformBinding> + '_ { self.bindings.values().copied() }
    pub fn binding(&self, layer: LayerId) -> Option<TransformBinding> { self.bindings.get(&layer).copied() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, TransformProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Local { slots, flow } => local_value(node, inputs, slots, *flow).map(NodeValue::new),
            Recipe::World { parent } => (|| {
                let local_index = usize::from(*parent);
                let local = read_transform(node, inputs, local_index)?;
                if *parent {
                    read_transform(node, inputs, 0).map(|up| NodeValue::new(TransformValue { affine: up.affine * local.affine, spatial: up.spatial * local.spatial }))
                } else { Ok(NodeValue::new(local)) }
            })()
        })
    }
}

fn read_transform(node: &GraphNode, inputs: &NodeInputs, index: usize) -> Result<TransformValue, TransformProgramError> {
    inputs.at(index).and_then(|value| value.downcast_ref::<TransformValue>()).copied().ok_or(TransformProgramError::InvalidInput(node.identity().kind))
}

fn local_value(node: &GraphNode, inputs: &NodeInputs, slots: &[Option<usize>; 10], flow: Option<(usize, usize)>) -> Result<TransformValue, TransformProgramError> {
    let value = |row: usize| slots[row].and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>());
    let vec2 = |row: usize, default: [f32; 2]| match value(row) { Some(Value::Vec2(v)) => [v[0] as f32, v[1] as f32], None => default, _ => default };
    let scalar = |row: usize, default: f32| match value(row) { Some(Value::F64(v)) => *v as f32, None => default, _ => default };
    let mut position = vec2(0, [0.0; 2]);
    let mut anchor = vec2(1, [0.0; 2]);
    let mut scale = vec2(2, [1.0; 2]);
    if let Some((input, index)) = flow {
        if let Some(slot) = inputs.at(input).and_then(|value| value.downcast_ref::<FlowFrameValue>()).and_then(|flow| flow.slots.get(index)).copied().flatten() {
            position = [position[0] + slot.position[0], position[1] + slot.position[1]];
            scale = [scale[0] * slot.scale[0], scale[1] * slot.scale[1]];
            anchor = slot.anchor;
        }
    }
    let affine = LayerPlacement::from_transform(anchor, position, scale, scalar(3, 0.0), scalar(4, 0.0), scalar(5, 0.0));
    let spatial = LayerPlacement::spatial_from_transform(affine, position, scalar(6, 0.0), scalar(7, 0.0), scalar(8, 0.0), scalar(9, 1.0));
    if !affine.matrix2.is_finite() || !spatial.is_finite() { return Err(TransformProgramError::InvalidInput(node.identity().kind)); }
    Ok(TransformValue { affine, spatial })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{LayerAttrsPatch, LayerMeta, LayerSource, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn parent_and_child_world_are_composed_from_property_nodes() {
        let mut doc = Document::new();
        let parent = LayerId(1);
        let child = LayerId(2);
        for (layer, position) in [(parent, [10.0, 20.0]), (child, [5.0, 7.0])] {
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: layer.0 as i16, timing: LayerTiming::place(0, None, 90) } },
                Intent::SetConstant { layer, property: PropertyId::new(property::POSITION).unwrap(), value: Value::Vec2(position) },
            ]).unwrap();
        }
        doc.apply(Intent::SetAttrs { layer: child, patch: LayerAttrsPatch { parent: Some(Some(parent)), ..Default::default() } }).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.transforms().binding(child).unwrap().world;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let world = frame.value(root).and_then(|value| value.downcast_ref::<TransformValue>()).unwrap();
        assert_eq!(world.affine.translation.to_array(), [15.0, 27.0]);
    }
}
