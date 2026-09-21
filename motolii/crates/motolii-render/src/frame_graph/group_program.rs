use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{layout, LayerId, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};
use crate::doc::vector::{Brush, Fill, FillRule, OpKind, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeOp};

use super::{EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone)] struct Recipe { flow: usize, index: usize, color: Option<usize>, radius: Option<usize> }
#[derive(Debug)] pub enum GroupBackgroundProgramError { Store(StoreError), InvalidInput(NodeKind) }
impl std::fmt::Display for GroupBackgroundProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for GroupBackgroundProgramError {}
impl From<StoreError> for GroupBackgroundProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct GroupBackgroundProgram { nodes: BTreeMap<NodeKey, GraphNode>, recipes: BTreeMap<NodeKey, Recipe>, bindings: BTreeMap<LayerId, NodeKey> }
impl GroupBackgroundProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, flow: &FlowProgram) -> Result<Self, GroupBackgroundProgramError> {
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            if !view.meta(layer)?.is_some_and(|meta| meta.source == LayerSource::Group) { continue; }
            let Some(binding) = flow.binding(layer) else { continue };
            let mut inputs = vec![flow.key()];
            let color = properties.node_for(layer, &PropertyId::new(layout::BACKGROUND)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            let radius = properties.node_for(layer, &PropertyId::new(layout::BORDER_RADIUS)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            if color.is_none() { continue; }
            let mut identity = NodeIdentity::new(NodeKind::GroupBackground, inputs); identity.parameters = (binding.index as u64).to_be_bytes().to_vec(); identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity); recipes.entry(node.key()).or_insert(Recipe { flow: 0, index: binding.index, color, radius }); bindings.insert(layer, node.key()); nodes.entry(node.key()).or_insert(node);
        }
        Ok(Self { nodes, recipes, bindings })
    }
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<NodeKey> { self.bindings.get(&layer).copied() }
    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, GroupBackgroundProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let flow = inputs.at(recipe.flow).and_then(|value| value.downcast_ref::<FlowFrameValue>()).ok_or(GroupBackgroundProgramError::InvalidInput(node.identity().kind))?;
            let Some([width, height]) = flow.sizes.get(recipe.index).copied().flatten() else { return Ok(NodeValue::new(Vec::<ShapeNode>::new())); };
            let color = match recipe.color.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Color(color)) => *color, _ => [0.0; 4] };
            if color[3] <= 0.0 || width <= 0.0 || height <= 0.0 { return Ok(NodeValue::new(Vec::<ShapeNode>::new())); }
            let radius = match recipe.radius.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) => value.max(0.0).min(f64::from(width.min(height)) * 0.5), _ => 0.0 };
            let shape = ShapeNode::Group(ShapeGroup { transform: RepeaterTransform { position: Point { x: f64::from(width) * 0.5, y: f64::from(height) * 0.5 }, ..RepeaterTransform::IDENTITY }, children: vec![ShapeNode::Leaf(Shape { source: PathSource::Rectangle { size: Point { x: f64::from(width), y: f64::from(height) } }, ops: (radius > 0.0).then(|| ShapeOp::new(OpKind::RoundedCorners { radius })).into_iter().collect(), fill: Some(Fill { brush: Brush::Solid(Rgb { r: color[0], g: color[1], b: color[2] }), rule: FillRule::NonZero, opacity: color[3], hidden: false }), stroke: None })] });
            Ok(NodeValue::new(vec![shape]))
        })())
    }
}
