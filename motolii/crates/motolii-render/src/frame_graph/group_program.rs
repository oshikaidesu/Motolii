use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{layout, LayerId, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};
use crate::doc::vector::{Brush, Fill, FillRule, OpKind, PathSource, Point, RepeaterTransform, Rgb, Shape, ShapeGroup, ShapeOp};

use super::{EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone)] struct Recipe { flow: usize, index: usize, color: Option<usize>, radius: Option<usize>, shadow_color: Option<usize>, shadow_offset: Option<usize>, shadow_blur: Option<usize>, shadow_spread: Option<usize> }
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
            let shadow_color = properties.node_for(layer, &PropertyId::new(layout::SHADOW_COLOR)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            let shadow_offset = properties.node_for(layer, &PropertyId::new(layout::SHADOW_OFFSET)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            let shadow_blur = properties.node_for(layer, &PropertyId::new(layout::SHADOW_BLUR)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            let shadow_spread = properties.node_for(layer, &PropertyId::new(layout::SHADOW_SPREAD)?).map(|key| { let at = inputs.len(); inputs.push(key); at });
            if color.is_none() && shadow_color.is_none() { continue; }
            let mut identity = NodeIdentity::new(NodeKind::GroupBackground, inputs); identity.parameters = (binding.index as u64).to_be_bytes().to_vec(); identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity); recipes.entry(node.key()).or_insert(Recipe { flow: 0, index: binding.index, color, radius, shadow_color, shadow_offset, shadow_blur, shadow_spread }); bindings.insert(layer, node.key()); nodes.entry(node.key()).or_insert(node);
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
            let shadow = match recipe.shadow_color.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Color(color)) => *color, _ => [0.0; 4] };
            if (color[3] <= 0.0 && shadow[3] <= 0.0) || width <= 0.0 || height <= 0.0 { return Ok(NodeValue::new(Vec::<ShapeNode>::new())); }
            let radius = match recipe.radius.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) => value.max(0.0), _ => 0.0 };
            let offset = match recipe.shadow_offset.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::Vec2(value)) => *value, _ => [0.0, 12.0] };
            let blur = match recipe.shadow_blur.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) => value.max(0.0), _ => 24.0 };
            let spread = match recipe.shadow_spread.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) { Some(Value::F64(value)) => *value, _ => 0.0 };
            let (w, h) = (f64::from(width), f64::from(height));
            let rounded = |grow: f64, offset: [f64; 2], rgba: [f64; 4]| -> Option<ShapeNode> {
                let (sw, sh) = (w + 2.0 * grow, h + 2.0 * grow);
                if sw <= 0.0 || sh <= 0.0 || rgba[3] <= 0.0 { return None; }
                let r = (radius + grow).max(0.0).min(sw.min(sh) * 0.5);
                Some(ShapeNode::Group(ShapeGroup {
                    transform: RepeaterTransform { position: Point { x: w * 0.5 + offset[0], y: h * 0.5 + offset[1] }, ..RepeaterTransform::IDENTITY },
                    children: vec![ShapeNode::Leaf(Shape {
                        source: PathSource::Rectangle { size: Point { x: sw, y: sh } },
                        ops: (r > 0.0).then(|| ShapeOp::new(OpKind::RoundedCorners { radius: r })).into_iter().collect(),
                        fill: Some(Fill { brush: Brush::Solid(Rgb { r: rgba[0], g: rgba[1], b: rgba[2] }), rule: FillRule::NonZero, opacity: rgba[3], hidden: false }),
                        stroke: None,
                    })],
                }))
            };
            let mut shapes = Vec::new();
            if shadow[3] > 0.0 {
                if blur <= 0.5 {
                    shapes.extend(rounded(spread, offset, shadow));
                } else {
                    const RINGS: usize = 32;
                    let ramp = |n: usize| { let u = n as f64 / RINGS as f64; shadow[3] * u * u * (3.0 - 2.0 * u) };
                    for k in 0..RINGS {
                        let grow = spread + blur * (1.0 - 2.0 * (k as f64 + 0.5) / RINGS as f64);
                        let (before, after) = (ramp(k), ramp(k + 1));
                        let alpha = if before >= 1.0 { 0.0 } else { 1.0 - (1.0 - after) / (1.0 - before) };
                        shapes.extend(rounded(grow, offset, [shadow[0], shadow[1], shadow[2], alpha.clamp(0.0, 1.0)]));
                    }
                }
            }
            shapes.extend(rounded(0.0, [0.0, 0.0], color));
            Ok(NodeValue::new(shapes))
        })())
    }
}
