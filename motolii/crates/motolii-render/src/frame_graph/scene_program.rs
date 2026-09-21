use std::collections::BTreeMap;

use crate::doc::store::{property, BlendMode, LayerId, LayerProjection, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};

use super::{ContentProgram, EvaluationContext, GraphNode, MaterialValue, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TextProgram, TextShapeValue, TimeDependency, TransformProgram, TransformValue};

#[derive(Clone, Debug, PartialEq)]
pub enum SceneContentValue { None, Text(TextShapeValue), Shape(Vec<ShapeNode>), Material(MaterialValue) }

#[derive(Clone, Debug, PartialEq)]
pub struct SceneLayerValue { pub transform: TransformValue, pub content_key: Option<NodeKey>, pub content: SceneContentValue, pub opacity: f32, pub projection: LayerProjection, pub blend: BlendMode, pub order: i16 }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneValue { pub layers: Vec<SceneLayerValue> }

#[derive(Clone)]
enum Recipe { Contribution { content: Option<usize>, opacity: Option<usize>, projection: LayerProjection, blend: BlendMode, order: i16, kind: u8 }, Composite }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneProgramNodes { pub scene: NodeKey }

#[derive(Debug)]
pub enum SceneNodeError { Store(StoreError), InvalidInput(NodeKind) }
impl std::fmt::Display for SceneNodeError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneNodeError {}
impl From<StoreError> for SceneNodeError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct SceneNodeProgram {
    nodes: BTreeMap<NodeKey, GraphNode>, recipes: BTreeMap<NodeKey, Recipe>, bindings: BTreeMap<LayerId, NodeKey>, output: SceneProgramNodes,
}

impl SceneNodeProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, content: &ContentProgram, transforms: &TransformProgram, text: &TextProgram) -> Result<Self, SceneNodeError> {
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new(); let mut ordered = Vec::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if matches!(meta.source, LayerSource::Camera | LayerSource::Stage) { continue; }
            let Some(transform) = transforms.binding(layer).map(|binding| binding.world) else { continue };
            let mut inputs = vec![transform];
            let (content_key, kind) = match meta.source {
                LayerSource::Text => (text.binding(layer).map(|binding| binding.shape), 1),
                LayerSource::Shape => (content.binding(layer).and_then(|binding| binding.content), 2),
                LayerSource::File { .. } => (content.binding(layer).and_then(|binding| binding.material.or(binding.content)), 3),
                _ => (None, 0),
            };
            let content_index = content_key.map(|key| { let at = inputs.len(); inputs.push(key); at });
            let opacity_key = properties.node_for(layer, &PropertyId::new(property::OPACITY).expect("known opacity"));
            let opacity = opacity_key.map(|key| { let at = inputs.len(); inputs.push(key); at });
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let mut identity = NodeIdentity::new(NodeKind::CompositeContribution, inputs);
            identity.parameters = [kind].into_iter().chain(meta.order.to_be_bytes()).chain([projection_tag(attrs.projection)]).collect();
            identity.parameters.extend(serde_json::to_vec(&attrs.blend_mode).unwrap_or_default());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            recipes.entry(node.key()).or_insert(Recipe::Contribution { content: content_index, opacity, projection: attrs.projection, blend: attrs.blend_mode, order: meta.order, kind });
            nodes.entry(node.key()).or_insert(node.clone());
            bindings.insert(layer, node.key());
            ordered.push((meta.order, layer.0, node.key()));
        }
        ordered.sort_by_key(|(order, id, _)| (*order, *id));
        let inputs: Vec<_> = ordered.into_iter().map(|(_, _, key)| key).collect();
        let mut identity = NodeIdentity::new(NodeKind::SceneComposite, inputs);
        identity.time_dependency = TimeDependency::Exact;
        let scene = GraphNode::new(identity);
        recipes.insert(scene.key(), Recipe::Composite);
        nodes.insert(scene.key(), scene.clone());
        Ok(Self { nodes, recipes, bindings, output: SceneProgramNodes { scene: scene.key() } })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn output(&self) -> SceneProgramNodes { self.output }
    pub fn binding(&self, layer: LayerId) -> Option<NodeKey> { self.bindings.get(&layer).copied() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, SceneNodeError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Contribution { content, opacity, projection, blend, order, kind } => (|| {
                let transform = inputs.at(0).and_then(|value| value.downcast_ref::<TransformValue>()).copied().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                let content_key = content.map(|index| node.identity().inputs[index]);
                let content = match (*kind, *content) {
                    (1, Some(index)) => SceneContentValue::Text(inputs.at(index).and_then(|value| value.downcast_ref::<TextShapeValue>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (2, Some(index)) => SceneContentValue::Shape(inputs.at(index).and_then(|value| value.downcast_ref::<Vec<ShapeNode>>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (3, Some(index)) => SceneContentValue::Material(inputs.at(index).and_then(|value| value.downcast_ref::<MaterialValue>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    _ => SceneContentValue::None,
                };
                let opacity = opacity.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>()).and_then(|value| match value { crate::doc::eval::Value::F64(value) => Some(*value as f32), _ => None }).unwrap_or(1.0).clamp(0.0, 1.0);
                Ok(NodeValue::new(SceneLayerValue { transform, content_key, content, opacity, projection: *projection, blend: *blend, order: *order }))
            })(),
            Recipe::Composite => inputs.iter().map(|(_, value)| value.downcast_ref::<SceneLayerValue>().cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))).collect::<Result<Vec<_>, _>>().map(|layers| NodeValue::new(SceneValue { layers })),
        })
    }
}

fn projection_tag(value: LayerProjection) -> u8 { match value { LayerProjection::TwoD => 0, LayerProjection::TwoPointFiveD => 1, LayerProjection::ThreeD => 2 } }
