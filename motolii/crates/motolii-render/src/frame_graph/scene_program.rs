use std::collections::BTreeMap;

use crate::doc::store::{property, BlendMode, LayerId, LayerProjection, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};

use super::{ContentProgram, EffectProgram, EffectValue, EvaluationContext, GraphNode, GroupBackgroundProgram, MaskProgram, MaskValue, MaterialValue, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TextProgram, TextShapeValue, TimeDependency, TransformProgram, TransformValue};

#[derive(Clone, Debug, PartialEq)]
pub enum SceneContentValue { None, Text(TextShapeValue), Shape(Vec<ShapeNode>), Material(MaterialValue) }

#[derive(Clone, Debug, PartialEq)]
pub struct SceneLayerValue { pub layer: LayerId, pub source: LayerSource, pub transform: TransformValue, pub content_key: Option<NodeKey>, pub content: SceneContentValue, pub effects: Vec<crate::picture::resolved::ResolvedEffect>, pub after_effects: Vec<crate::picture::resolved::ResolvedEffect>, pub masks: Vec<crate::picture::resolved::ResolvedMask>, pub matte: Option<crate::doc::store::Matte>, pub clip_to_below: bool, pub flatten: bool, pub environment: bool, pub opacity: f32, pub projection: LayerProjection, pub blend: BlendMode, pub order: i16 }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneValue { pub layers: Vec<SceneLayerValue> }

#[derive(Clone)]
enum Recipe { Contribution { layer: LayerId, source: LayerSource, content: Option<usize>, opacity: Option<usize>, effects: Vec<(usize, bool)>, masks: Vec<usize>, matte: Option<crate::doc::store::Matte>, clip_to_below: bool, flatten: bool, environment: bool, projection: LayerProjection, blend: BlendMode, order: i16, kind: u8 }, Composite }

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
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, content: &ContentProgram, transforms: &TransformProgram, text: &TextProgram, groups: &GroupBackgroundProgram, effect_program: &EffectProgram, mask_program: &MaskProgram) -> Result<Self, SceneNodeError> {
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
                LayerSource::Group => (groups.binding(layer), 2),
                _ => (None, 0),
            };
            let content_index = content_key.map(|key| { let at = inputs.len(); inputs.push(key); at });
            let opacity_key = properties.node_for(layer, &PropertyId::new(property::OPACITY).expect("known opacity"));
            let opacity = opacity_key.map(|key| { let at = inputs.len(); inputs.push(key); at });
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let matte = if attrs.clip_to_below { view.clipping_base(layer)?.map(|layer| crate::doc::store::Matte { layer, mode: crate::doc::store::MatteMode::Alpha }) } else { attrs.matte };
            let mut effect_inputs = Vec::new();
            let mut current = Some(layer); let mut own = true;
            while let Some(owner) = current {
                if let Some(binding) = effect_program.binding(owner) { for key in &binding.effects { let at = inputs.len(); inputs.push(*key); effect_inputs.push((at, !own)); } }
                current = view.attrs(owner)?.unwrap_or_default().parent; own = false;
            }
            let mut mask_inputs = Vec::new();
            if let Some(binding) = mask_program.binding(layer) { for key in &binding.masks { let at = inputs.len(); inputs.push(*key); mask_inputs.push(at); } }
            let mut identity = NodeIdentity::new(NodeKind::CompositeContribution, inputs);
            identity.parameters = [kind].into_iter().chain(meta.order.to_be_bytes()).chain([projection_tag(attrs.projection)]).collect();
            identity.parameters.extend(serde_json::to_vec(&attrs.blend_mode).unwrap_or_default());
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let non_group = meta.source != LayerSource::Group;
            recipes.entry(node.key()).or_insert(Recipe::Contribution { layer, source: meta.source, content: content_index, opacity, effects: effect_inputs, masks: mask_inputs, matte, clip_to_below: attrs.clip_to_below, flatten: attrs.flatten, environment: attrs.environment, projection: attrs.projection, blend: attrs.blend_mode, order: meta.order, kind });
            nodes.entry(node.key()).or_insert(node.clone());
            bindings.insert(layer, node.key());
            ordered.push((meta.order, non_group, layer.0, node.key()));
        }
        // Preserve the established settle contract: authored order first, and
        // a Group background immediately behind non-Group content at the same
        // order. The composite input order then becomes the final depth rank.
        ordered.sort_by_key(|(order, non_group, id, _)| (*order, *non_group, *id));
        let inputs = ordered.into_iter().map(|(_, _, _, key)| key).collect();
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
            Recipe::Contribution { layer, source, content, opacity, effects, masks, matte, clip_to_below, flatten, environment, projection, blend, order, kind } => (|| {
                let transform = inputs.at(0).and_then(|value| value.downcast_ref::<TransformValue>()).copied().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                let content_key = content.map(|index| node.identity().inputs[index]);
                let content = match (*kind, *content) {
                    (1, Some(index)) => SceneContentValue::Text(inputs.at(index).and_then(|value| value.downcast_ref::<TextShapeValue>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (2, Some(index)) => SceneContentValue::Shape(inputs.at(index).and_then(|value| value.downcast_ref::<Vec<ShapeNode>>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (3, Some(index)) => SceneContentValue::Material(inputs.at(index).and_then(|value| value.downcast_ref::<MaterialValue>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    _ => SceneContentValue::None,
                };
                let opacity = opacity.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>()).and_then(|value| match value { crate::doc::eval::Value::F64(value) => Some(*value as f32), _ => None }).unwrap_or(1.0).clamp(0.0, 1.0);
                let mut direct = Vec::new(); let mut after = Vec::new();
                for (index, inherited) in effects {
                    let Some(effect) = inputs.at(*index).and_then(|value| value.downcast_ref::<EffectValue>()).and_then(|value| value.0.clone()) else { continue };
                    if (*inherited && effect.scope == crate::doc::store::EffectScope::Whole) || (!*inherited && effect.scope == crate::doc::store::EffectScope::Whole) { after.push(effect); } else { direct.push(effect); }
                }
                let masks = masks.iter().map(|index| inputs.at(*index).and_then(|value| value.downcast_ref::<MaskValue>()).map(|value| value.0.clone()).ok_or(SceneNodeError::InvalidInput(node.identity().kind))).collect::<Result<_, _>>()?;
                Ok(NodeValue::new(SceneLayerValue { layer: *layer, source: source.clone(), transform, content_key, content, effects: direct, after_effects: after, masks, matte: *matte, clip_to_below: *clip_to_below, flatten: *flatten, environment: *environment, opacity, projection: *projection, blend: *blend, order: *order }))
            })(),
            Recipe::Composite => inputs.iter().enumerate().map(|(rank, (_, value))| {
                let mut layer = value.downcast_ref::<SceneLayerValue>().cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                // The compositor's depth key must describe the flattened scene,
                // not the sibling-local authored order.  A child commonly has
                // order 0 while its Group was created later; forwarding those
                // raw values lets the Group background cover the child even
                // though the graph inputs are already parent-before-child.
                layer.order = i16::try_from(rank).unwrap_or(i16::MAX);
                Ok(layer)
            }).collect::<Result<Vec<_>, _>>().map(|layers| NodeValue::new(SceneValue { layers })),
        })
    }
}

fn projection_tag(value: LayerProjection) -> u8 { match value { LayerProjection::TwoD => 0, LayerProjection::TwoPointFiveD => 1, LayerProjection::ThreeD => 2 } }
