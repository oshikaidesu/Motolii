use std::collections::BTreeMap;

use crate::doc::core::RationalTime;
use crate::doc::store::{property, BlendMode, LayerId, LayerProjection, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};

use super::{ContentProgram, EffectProgram, EffectValue, EvaluationContext, GraphNode, GroupBackgroundProgram, MaskProgram, MaskValue, MaterialValue, MediaFrameValue, MediaSourceValue, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TextProgram, TextShapeValue, TimeDependency, TransformProgram, TransformValue, VisibilityProgram, VisibilityValue};
use super::group_composite_program::{GroupCompositeProgram, GroupCompositeProgramError, SceneFragmentValue};
use super::scene_policy::ScenePolicy;

#[derive(Clone, Debug, PartialEq)]
pub enum SceneContentValue {
    None,
    Text(TextShapeValue),
    Shape(Vec<ShapeNode>),
    Material(MaterialValue),
    Media { source: MediaSourceValue, time: RationalTime },
    Plate(ScenePlateValue),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScenePlateValue {
    pub(crate) members: Vec<SceneContributionValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneLayerValue { pub layer: LayerId, pub source: LayerSource, pub transform: TransformValue, pub content_key: Option<NodeKey>, pub content: SceneContentValue, pub effects: Vec<crate::picture::resolved::ResolvedEffect>, pub after_effects: Vec<crate::picture::resolved::ResolvedEffect>, pub masks: Vec<crate::picture::resolved::ResolvedMask>, pub matte: Option<crate::doc::store::Matte>, pub clip_to_below: bool, pub flatten: bool, pub environment: bool, pub opacity: f32, pub projection: LayerProjection, pub blend: BlendMode, pub order: i16 }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneValue { pub layers: Vec<SceneLayerValue> }

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneContributionValue {
    pub(crate) solo: bool,
    pub(crate) layer: Option<SceneLayerValue>,
}

#[derive(Clone)]
enum Recipe { Contribution { layer: LayerId, source: LayerSource, visibility: usize, content: Option<usize>, opacity: Option<usize>, blend_value: Option<usize>, matte_mode: Option<usize>, effects: Vec<(usize, bool)>, masks: Vec<usize>, matte: Option<crate::doc::store::Matte>, clip_to_below: bool, flatten: bool, environment: bool, projection: LayerProjection, blend: BlendMode, order: i16, kind: u8 }, Composite }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneProgramNodes { pub scene: NodeKey }

#[derive(Debug)]
pub enum SceneNodeError { Store(StoreError), GroupComposite(GroupCompositeProgramError), InvalidInput(NodeKind) }
impl std::fmt::Display for SceneNodeError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneNodeError {}
impl From<StoreError> for SceneNodeError { fn from(value: StoreError) -> Self { Self::Store(value) } }
impl From<GroupCompositeProgramError> for SceneNodeError { fn from(value: GroupCompositeProgramError) -> Self { Self::GroupComposite(value) } }

pub struct SceneNodeProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, NodeKey>,
    output: SceneProgramNodes,
    policy: ScenePolicy,
    group_composite: GroupCompositeProgram,
}

impl SceneNodeProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, content: &ContentProgram, transforms: &TransformProgram, text: &TextProgram, groups: &GroupBackgroundProgram, effect_program: &EffectProgram, mask_program: &MaskProgram, visibility_program: &VisibilityProgram) -> Result<Self, SceneNodeError> {
        let policy = ScenePolicy::compile(view)?;
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if matches!(meta.source, LayerSource::Camera | LayerSource::Stage) { continue; }
            let Some(transform) = transforms.binding(layer).map(|binding| binding.world) else { continue };
            let Some(visibility_key) = visibility_program.binding(layer).map(|binding| binding.node) else { continue };
            let mut inputs = vec![transform, visibility_key];
            let visibility = 1usize;
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
            let blend_value = properties.node_for(layer, &PropertyId::blend_mode()).map(|key| {
                let at = inputs.len(); inputs.push(key); at
            });
            let matte_mode = properties.node_for(layer, &PropertyId::matte_mode()).map(|key| {
                let at = inputs.len(); inputs.push(key); at
            });
            let attrs = view.attrs(layer)?.unwrap_or_default();
            let matte = if attrs.clip_to_below { view.clipping_base(layer)?.map(|layer| crate::doc::store::Matte { layer, mode: crate::doc::store::MatteMode::Alpha }) } else { attrs.matte };
            let mut effect_inputs = Vec::new();
            if let Some(binding) = effect_program.binding(layer) {
                for key in &binding.effects {
                    let at = inputs.len();
                    inputs.push(*key);
                    effect_inputs.push((at, false));
                }
            }
            let mut mask_inputs = Vec::new();
            if let Some(binding) = mask_program.binding(layer) { for key in &binding.masks { let at = inputs.len(); inputs.push(*key); mask_inputs.push(at); } }
            let mut identity = NodeIdentity::new(NodeKind::CompositeContribution, inputs);
            identity.parameters = [kind].into_iter().chain(meta.order.to_be_bytes()).chain([projection_tag(attrs.projection)]).collect();
            identity.parameters.extend(serde_json::to_vec(&attrs.blend_mode).unwrap_or_default());
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            recipes.entry(node.key()).or_insert(Recipe::Contribution { layer, source: meta.source, visibility, content: content_index, opacity, blend_value, matte_mode, effects: effect_inputs, masks: mask_inputs, matte, clip_to_below: attrs.clip_to_below, flatten: attrs.flatten, environment: attrs.environment, projection: attrs.projection, blend: attrs.blend_mode, order: meta.order, kind });
            nodes.entry(node.key()).or_insert(node.clone());
            bindings.insert(layer, node.key());
        }
        let group_composite = GroupCompositeProgram::compile(view, effect_program, &bindings)?;
        for node in group_composite.nodes() {
            nodes.insert(node.key(), node);
        }
        let inputs = group_composite.roots().to_vec();
        let mut identity = NodeIdentity::new(NodeKind::SceneComposite, inputs);
        identity.time_dependency = TimeDependency::Exact;
        let scene = GraphNode::new(identity);
        recipes.insert(scene.key(), Recipe::Composite);
        nodes.insert(scene.key(), scene.clone());
        Ok(Self { nodes, recipes, bindings, output: SceneProgramNodes { scene: scene.key() }, policy, group_composite })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn output(&self) -> SceneProgramNodes { self.output }
    pub fn binding(&self, layer: LayerId) -> Option<NodeKey> { self.bindings.get(&layer).copied() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, SceneNodeError>> {
        if let Some(value) = self.group_composite.execute(node, inputs, context) {
            return Some(value.map_err(Into::into));
        }
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Contribution { layer, source, visibility, content, opacity, blend_value, matte_mode, effects, masks, matte, clip_to_below, flatten, environment, projection, blend, order, kind } => (|| {
                let visibility = inputs.at(*visibility).and_then(|value| value.downcast_ref::<VisibilityValue>()).copied().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                if !visibility.active {
                    return Ok(NodeValue::new(SceneContributionValue { solo: visibility.solo, layer: None }));
                }
                let transform = inputs.at(0).and_then(|value| value.downcast_ref::<TransformValue>()).copied().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                let content_key = content.map(|index| node.identity().inputs[index]);
                let content = match (*kind, *content) {
                    (1, Some(index)) => SceneContentValue::Text(inputs.at(index).and_then(|value| value.downcast_ref::<TextShapeValue>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (2, Some(index)) => SceneContentValue::Shape(inputs.at(index).and_then(|value| value.downcast_ref::<Vec<ShapeNode>>()).cloned().ok_or(SceneNodeError::InvalidInput(node.identity().kind))?),
                    (3, Some(index)) => {
                        let value = inputs.at(index).ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                        if let Some(material) = value.downcast_ref::<MaterialValue>() {
                            SceneContentValue::Material(material.clone())
                        } else if let Some(frame) = value.downcast_ref::<MediaFrameValue>() {
                            match frame.time {
                                Some(time) => SceneContentValue::Media { source: frame.source.clone(), time },
                                None => SceneContentValue::None,
                            }
                        } else {
                            return Err(SceneNodeError::InvalidInput(node.identity().kind));
                        }
                    },
                    _ => SceneContentValue::None,
                };
                let opacity = opacity.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>()).and_then(|value| match value { crate::doc::eval::Value::F64(value) => Some(*value as f32), _ => None }).unwrap_or(1.0).clamp(0.0, 1.0);
                let blend = match blend_value.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>()) {
                    Some(crate::doc::eval::Value::Enum(value)) => BlendMode::from_enum_value(*value).ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
                    Some(_) => return Err(SceneNodeError::InvalidInput(node.identity().kind)),
                    None => *blend,
                };
                let mut matte = *matte;
                if !*clip_to_below {
                    if let (Some(matte), Some(value)) = (matte.as_mut(), matte_mode.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>())) {
                        match value {
                            crate::doc::eval::Value::Enum(value) => matte.mode = crate::doc::store::MatteMode::from_enum_value(*value).ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
                            _ => return Err(SceneNodeError::InvalidInput(node.identity().kind)),
                        }
                    }
                }
                let mut direct = Vec::new();
                for (index, _) in effects {
                    let Some(effect) = inputs.at(*index).and_then(|value| value.downcast_ref::<EffectValue>()).and_then(|value| value.0.clone()) else { continue };
                    direct.push(effect);
                }
                let after = Vec::new();
                let masks = masks.iter().map(|index| inputs.at(*index).and_then(|value| value.downcast_ref::<MaskValue>()).map(|value| value.0.clone()).ok_or(SceneNodeError::InvalidInput(node.identity().kind))).collect::<Result<_, _>>()?;
                Ok(NodeValue::new(SceneContributionValue {
                    solo: visibility.solo,
                    layer: Some(SceneLayerValue { layer: *layer, source: source.clone(), transform, content_key, content, effects: direct, after_effects: after, masks, matte, clip_to_below: *clip_to_below, flatten: *flatten, environment: *environment, opacity, projection: *projection, blend, order: *order }),
                }))
            })(),
            Recipe::Composite => (|| {
                let mut contributions = Vec::new();
                for (_, value) in inputs.iter() {
                    if let Some(contribution) = value.downcast_ref::<SceneContributionValue>() {
                        contributions.push(contribution.clone());
                    } else if let Some(fragment) = value.downcast_ref::<SceneFragmentValue>() {
                        contributions.extend(fragment.contributions.iter().cloned());
                    } else {
                        return Err(SceneNodeError::InvalidInput(node.identity().kind));
                    }
                }
                let any_solo = contributions.iter().any(contribution_has_solo);
                let mut layers = Vec::new();
                for contribution in contributions {
                    let Some(mut layer) = filtered_layer(contribution, any_solo) else { continue };
                    normalize_plate_orders(&mut layer);
                    layer.order = i16::try_from(layers.len()).unwrap_or(i16::MAX);
                    layers.push(layer);
                }
                self.policy.hand_out_stencils(&mut layers);
                Ok(NodeValue::new(SceneValue { layers }))
            })(),
        })
    }
}

fn contribution_has_solo(contribution: &SceneContributionValue) -> bool {
    if contribution.solo { return true; }
    contribution.layer.as_ref().is_some_and(|layer| {
        matches!(&layer.content, SceneContentValue::Plate(plate) if plate.members.iter().any(contribution_has_solo))
    })
}

fn filtered_layer(mut contribution: SceneContributionValue, any_solo: bool) -> Option<SceneLayerValue> {
    if any_solo && !contribution_has_solo(&contribution) { return None; }
    let mut layer = contribution.layer.take()?;
    if let SceneContentValue::Plate(plate) = &mut layer.content {
        let members = std::mem::take(&mut plate.members);
        plate.members = members.into_iter().filter_map(|member| {
            let solo = member.solo;
            filtered_layer(member, any_solo).map(|layer| SceneContributionValue { solo, layer: Some(layer) })
        }).collect();
        if plate.members.is_empty() { return None; }
    }
    Some(layer)
}

fn normalize_plate_orders(layer: &mut SceneLayerValue) {
    if let SceneContentValue::Plate(plate) = &mut layer.content {
        for (rank, member) in plate.members.iter_mut().enumerate() {
            if let Some(member_layer) = member.layer.as_mut() {
                normalize_plate_orders(member_layer);
                member_layer.order = i16::try_from(rank).unwrap_or(i16::MAX);
            }
        }
    }
}

fn projection_tag(value: LayerProjection) -> u8 { match value { LayerProjection::TwoD => 0, LayerProjection::TwoPointFiveD => 1, LayerProjection::ThreeD => 2 } }
