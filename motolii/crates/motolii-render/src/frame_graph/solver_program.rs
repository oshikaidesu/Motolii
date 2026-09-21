use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{layout, LayerId, PropertyId, StoreError, StoreView};

use super::{
    EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey,
    NodeKind, NodeValue, PropertyProgram, RelationProgram, RelationSetValue, RelationValue,
    TimeDependency,
};

#[derive(Clone, Debug, PartialEq)]
pub struct SolverLayerValue {
    pub relation: RelationValue,
    pub size: Option<[f32; 2]>,
    pub stretch: [f32; 2],
    pub margin: f32,
    pub weight: f32,
    pub hardness: f32,
    pub border_radius: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SolverPlanValue {
    pub layers: BTreeMap<LayerId, SolverLayerValue>,
}

#[derive(Clone)]
struct LayerRecipe {
    layer: LayerId,
    flow_index: Option<usize>,
    margin: Option<usize>,
    flex_shrink: Option<usize>,
    heaviness: Option<usize>,
    hardness: Option<usize>,
    border_radius: Option<usize>,
}

#[derive(Debug)]
pub enum SolverProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for SolverProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SolverProgramError {}
impl From<StoreError> for SolverProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct SolverProgram {
    node: GraphNode,
    layers: Vec<LayerRecipe>,
}

impl SolverProgram {
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
        relations: &RelationProgram,
        flow: &FlowProgram,
    ) -> Result<Self, SolverProgramError> {
        let mut inputs = vec![relations.output(), flow.key()];
        let mut input_index = BTreeMap::new();
        let mut property_input = |key: NodeKey| {
            *input_index.entry(key).or_insert_with(|| {
                let at = inputs.len();
                inputs.push(key);
                at
            })
        };
        let mut layers = Vec::new();

        for layer in view.layers() {
            let property = |name: &str, property_input: &mut dyn FnMut(NodeKey) -> usize| -> Result<Option<usize>, SolverProgramError> {
                let id = PropertyId::new(name)?;
                Ok(properties.node_for(layer, &id).map(property_input))
            };
            layers.push(LayerRecipe {
                layer,
                flow_index: flow.binding(layer).map(|binding| binding.index),
                margin: property(layout::MARGIN, &mut property_input)?,
                flex_shrink: property(layout::FLEX_SHRINK, &mut property_input)?,
                heaviness: property(layout::HEAVINESS, &mut property_input)?,
                hardness: property(layout::HARDNESS, &mut property_input)?,
                border_radius: property(layout::BORDER_RADIUS, &mut property_input)?,
            });
        }

        let mut identity = NodeIdentity::new(NodeKind::SolverPlan, inputs);
        identity.parameters.extend_from_slice(&(layers.len() as u32).to_be_bytes());
        for layer in &layers {
            identity.parameters.extend_from_slice(&layer.layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&(layer.flow_index.unwrap_or(usize::MAX) as u64).to_be_bytes());
        }
        identity.time_dependency = TimeDependency::Exact;
        Ok(Self { node: GraphNode::new(identity), layers })
    }

    pub fn node(&self) -> GraphNode { self.node.clone() }
    pub fn key(&self) -> NodeKey { self.node.key() }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<NodeValue, SolverProgramError>> {
        if node.key() != self.node.key() { return None; }
        Some((|| {
            let relations = inputs.at(0)
                .and_then(|value| value.downcast_ref::<RelationSetValue>())
                .ok_or(SolverProgramError::InvalidInput(node.identity().kind))?;
            let flow = inputs.at(1)
                .and_then(|value| value.downcast_ref::<FlowFrameValue>())
                .ok_or(SolverProgramError::InvalidInput(node.identity().kind))?;

            let number = |index: Option<usize>, default: f32| -> Result<f32, SolverProgramError> {
                Ok(match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
                    Some(Value::F64(value)) if value.is_finite() => *value as f32,
                    None => default,
                    Some(_) => return Err(SolverProgramError::InvalidInput(node.identity().kind)),
                })
            };

            let mut out = BTreeMap::new();
            for recipe in &self.layers {
                let relation = relations.layers.get(&recipe.layer).cloned().unwrap_or_default();
                let slot = recipe.flow_index
                    .and_then(|index| flow.slots.get(index))
                    .copied()
                    .flatten();
                let size = recipe.flow_index
                    .and_then(|index| flow.sizes.get(index))
                    .copied()
                    .flatten();
                let flex = number(recipe.flex_shrink, 1.0)?.max(0.0);
                let weight = match recipe.heaviness {
                    Some(index) => number(Some(index), flex)?.max(0.0),
                    None => flex,
                };
                out.insert(recipe.layer, SolverLayerValue {
                    relation,
                    size,
                    stretch: slot.map_or([1.0, 1.0], |slot| slot.stretch),
                    margin: number(recipe.margin, 0.0)?.max(0.0),
                    weight,
                    hardness: number(recipe.hardness, 0.5)?.clamp(0.0, 1.0),
                    border_radius: number(recipe.border_radius, 0.0)?.max(0.0),
                });
            }
            Ok(NodeValue::new(SolverPlanValue { layers: out }))
        })())
    }
}
