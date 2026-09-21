use std::collections::BTreeMap;

use crate::doc::eval::Value;
use crate::doc::store::{layout, LayerId, PropertyId, StoreError, StoreView};

use super::{EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RelationValue {
    pub parent: Option<LayerId>,
    pub anchor: Option<LayerId>,
    pub follow_anchor: bool,
    pub connection: Option<(LayerId, LayerId)>,
    pub trace: Option<(LayerId, i64)>,
    pub rope_slack: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationBinding {
    pub layer: LayerId,
    pub node: NodeKey,
}

#[derive(Clone)]
struct Recipe {
    parent: Option<LayerId>,
    anchor: Option<usize>,
    area: Option<usize>,
    connect_from: Option<usize>,
    connect_to: Option<usize>,
    trace: Option<usize>,
    line_path: Option<usize>,
    slack: Option<usize>,
}

#[derive(Debug)]
pub enum RelationProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for RelationProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for RelationProgramError {}
impl From<StoreError> for RelationProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct RelationProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, RelationBinding>,
}

impl RelationProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram) -> Result<Self, RelationProgramError> {
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let mut inputs = Vec::new();
            let mut bind = |name: &str| -> Result<Option<usize>, RelationProgramError> {
                let property = PropertyId::new(name)?;
                Ok(properties.node_for(layer, &property).map(|key| {
                    let at = inputs.len();
                    inputs.push(key);
                    at
                }))
            };

            let anchor = bind(layout::POSITION_ANCHOR)?;
            let area = bind(layout::POSITION_AREA)?;
            let connect_from = bind(layout::CONNECT_FROM)?;
            let connect_to = bind(layout::CONNECT_TO)?;
            let trace = bind(layout::TRACE)?;
            let line_path = bind(layout::LINE_PATH)?;
            let slack = bind(layout::SLACK)?;

            let mut identity = NodeIdentity::new(NodeKind::Relation, inputs);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&attrs.parent.map_or(0, |id| id.0).to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();

            nodes.entry(key).or_insert(node);
            recipes.entry(key).or_insert(Recipe {
                parent: attrs.parent,
                anchor,
                area,
                connect_from,
                connect_to,
                trace,
                line_path,
                slack,
            });
            bindings.insert(layer, RelationBinding { layer, node: key });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ {
        self.nodes.values().cloned()
    }

    pub fn binding(&self, layer: LayerId) -> Option<RelationBinding> {
        self.bindings.get(&layer).copied()
    }

    pub fn bindings(&self) -> impl ExactSizeIterator<Item = RelationBinding> + '_ {
        self.bindings.values().copied()
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<NodeValue, RelationProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let layer_id = |index: Option<usize>| -> Result<Option<LayerId>, RelationProgramError> {
                Ok(match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
                    Some(Value::LayerId(id)) if *id != 0 => Some(LayerId(*id)),
                    Some(Value::F64(value)) if *value >= 1.0 => Some(LayerId(value.round() as u64)),
                    Some(Value::LayerId(_)) | Some(Value::F64(_)) | None => None,
                    Some(_) => return Err(RelationProgramError::InvalidInput(node.identity().kind)),
                })
            };
            let choice = |index: Option<usize>, default: i64| -> Result<i64, RelationProgramError> {
                Ok(match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
                    Some(Value::Enum(value)) => *value,
                    Some(Value::F64(value)) => value.round() as i64,
                    None => default,
                    Some(_) => return Err(RelationProgramError::InvalidInput(node.identity().kind)),
                })
            };
            let number = |index: Option<usize>, default: f64| -> Result<f64, RelationProgramError> {
                Ok(match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
                    Some(Value::F64(value)) if value.is_finite() => *value,
                    None => default,
                    Some(_) => return Err(RelationProgramError::InvalidInput(node.identity().kind)),
                })
            };

            let anchor = layer_id(recipe.anchor)?;
            let follow_anchor = choice(recipe.area, 0)? > 0 && anchor.is_some();
            let from = layer_id(recipe.connect_from)?;
            let to = layer_id(recipe.connect_to)?;
            let connection = match (from, to) {
                (Some(from), Some(to)) if from != to => Some((from, to)),
                _ => None,
            };
            let trace_kind = choice(recipe.trace, 0)?;
            let trace = if connection.is_none() && trace_kind > 0 {
                from.map(|target| (target, trace_kind))
            } else {
                None
            };
            let rope_slack = (choice(recipe.line_path, 0)? == 4)
                .then(|| number(recipe.slack, 20.0).map(|value| value.max(0.0) as f32))
                .transpose()?;

            Ok(NodeValue::new(RelationValue {
                parent: recipe.parent,
                anchor,
                follow_anchor,
                connection,
                trace,
                rope_slack,
            }))
        })())
    }
}
