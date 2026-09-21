use std::collections::BTreeMap;

use crate::doc::core::RationalTime;
use crate::doc::store::{property, BlendMode, LayerId, LayerProjection, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};

use super::{ContentProgram, DynamicInput, EffectProgram, EffectValue, EvaluationContext, GraphNode, GroupBackgroundProgram, MaskProgram, MaskValue, MaterialValue, MediaFrameValue, MediaSourceValue, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, ParticleProgram, ParticleValue, PlacementProgram, PlacementSetValue, PropertyProgram, TextProgram, TextShapeValue, TimeDependency, TransformProgram, TransformValue, VisibilityProgram, VisibilityValue};
use super::ghost_program::{GhostProgram, GhostProgramError};
use super::group_composite_program::{GroupCompositeProgram, GroupCompositeProgramError, SceneFragmentValue};
use super::scene_policy::ScenePolicy;

#[derive(Clone, Debug, PartialEq)]
pub enum SceneContentValue {
    None,
    Text(TextShapeValue),
    Shape(Vec<ShapeNode>),
    Material(MaterialValue),
    Media { source: MediaSourceValue, time: RationalTime },
    Particles(ParticleValue),
    Plate(ScenePlateValue),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScenePlateValue {
    pub(crate) members: Vec<SceneContributionValue>,
    pub(crate) average: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneLayerValue { pub layer: LayerId, pub source: LayerSource, pub transform: TransformValue, pub content_key: Option<NodeKey>, pub content: SceneContentValue, pub effects: Vec<crate::picture::resolved::ResolvedEffect>, pub after_effects: Vec<crate::picture::resolved::ResolvedEffect>, pub masks: Vec<crate::picture::resolved::ResolvedMask>, pub matte: Option<crate::doc::store::Matte>, pub clip_to_below: bool, pub flatten: bool, pub environment: bool, pub ghost: bool, pub opacity: f32, pub projection: LayerProjection, pub blend: BlendMode, pub order: i16 }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SceneValue { pub layers: Vec<SceneLayerValue> }

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneContributionValue {
    pub(crate) solo: bool,
    pub(crate) layer: Option<SceneLayerValue>,
}

#[derive(Clone)]
enum Recipe { Contribution { layer: LayerId, source: LayerSource, visibility: usize, content: Option<usize>, opacity: Option<usize>, blend_value: Option<usize>, matte_mode: Option<usize>, effects: Vec<usize>, placement_effects: Vec<bool>, placement: Option<usize>, masks: Vec<usize>, matte: Option<crate::doc::store::Matte>, clip_to_below: bool, flatten: bool, environment: bool, projection: LayerProjection, blend: BlendMode, order: i16, kind: u8 }, Composite }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SceneProgramNodes { pub scene: NodeKey }

#[derive(Debug)]
pub enum SceneNodeError { Store(StoreError), Ghost(String), GroupComposite(String), InvalidInput(NodeKind) }
impl std::fmt::Display for SceneNodeError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for SceneNodeError {}
impl From<StoreError> for SceneNodeError { fn from(value: StoreError) -> Self { Self::Store(value) } }
impl From<GhostProgramError> for SceneNodeError { fn from(value: GhostProgramError) -> Self { Self::Ghost(value.to_string()) } }
impl From<GroupCompositeProgramError> for SceneNodeError { fn from(value: GroupCompositeProgramError) -> Self { Self::GroupComposite(value.to_string()) } }

pub struct SceneNodeProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, NodeKey>,
    output: SceneProgramNodes,
    policy: ScenePolicy,
    ghost: GhostProgram,
    group_composite: GroupCompositeProgram,
}

impl SceneNodeProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, content: &ContentProgram, transforms: &TransformProgram, text: &TextProgram, groups: &GroupBackgroundProgram, effect_program: &EffectProgram, mask_program: &MaskProgram, visibility_program: &VisibilityProgram, placement_program: &PlacementProgram, particle_program: &ParticleProgram) -> Result<Self, SceneNodeError> {
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
                LayerSource::Particles => (particle_program.binding(layer).map(|binding| binding.particles), 4),
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
            let mut placement_effects = Vec::new();
            if let Some(binding) = effect_program.binding(layer) {
                let end = if meta.source == LayerSource::Group {
                    binding.effects.iter().position(|key| effect_program.is_placement(*key)).unwrap_or(binding.effects.len())
                } else {
                    binding.effects.len()
                };
                for key in &binding.effects[..end] {
                    let at = inputs.len();
                    inputs.push(*key);
                    effect_inputs.push(at);
                    placement_effects.push(effect_program.is_placement(*key));
                }
            }
            let placement = placement_program.binding(layer).map(|binding| {
                let at = inputs.len();
                inputs.push(binding.node);
                at
            });
            let mut mask_inputs = Vec::new();
            if let Some(binding) = mask_program.binding(layer) { for key in &binding.masks { let at = inputs.len(); inputs.push(*key); mask_inputs.push(at); } }
            let mut identity = NodeIdentity::new(NodeKind::CompositeContribution, inputs);
            identity.parameters = [kind].into_iter().chain(meta.order.to_be_bytes()).chain([projection_tag(attrs.projection)]).collect();
            identity.parameters.extend(serde_json::to_vec(&attrs.blend_mode).unwrap_or_default());
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            recipes.entry(node.key()).or_insert(Recipe::Contribution { layer, source: meta.source, visibility, content: content_index, opacity, blend_value, matte_mode, effects: effect_inputs, placement_effects, placement, masks: mask_inputs, matte, clip_to_below: attrs.clip_to_below, flatten: attrs.flatten, environment: attrs.environment, projection: attrs.projection, blend: attrs.blend_mode, order: meta.order, kind });
            nodes.entry(node.key()).or_insert(node.clone());
            bindings.insert(layer, node.key());
        }
        let ghost = GhostProgram::compile(view, &bindings)?;
        for node in ghost.nodes() {
            nodes.insert(node.key(), node);
        }
        let mut hierarchy_bindings = bindings.clone();
        for layer in view.layers() {
            if let Some(key) = ghost.binding(layer) {
                hierarchy_bindings.insert(layer, key);
            }
        }
        let group_composite = GroupCompositeProgram::compile(view, effect_program, &hierarchy_bindings)?;
        for node in group_composite.nodes() {
            nodes.insert(node.key(), node);
        }
        let inputs = group_composite.roots().to_vec();
        let mut identity = NodeIdentity::new(NodeKind::SceneComposite, inputs);
        identity.time_dependency = TimeDependency::Exact;
        let scene = GraphNode::new(identity);
        recipes.insert(scene.key(), Recipe::Composite);
        nodes.insert(scene.key(), scene.clone());
        Ok(Self { nodes, recipes, bindings, output: SceneProgramNodes { scene: scene.key() }, policy, ghost, group_composite })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn output(&self) -> SceneProgramNodes { self.output }
    pub fn binding(&self, layer: LayerId) -> Option<NodeKey> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<Vec<DynamicInput>, SceneNodeError>> {
        let recipe = self.recipes.get(&node.key())?;
        let Recipe::Contribution { placement: Some(placement), .. } = recipe else {
            return Some(Ok(Vec::new()));
        };
        let Some(set) = inputs.at(*placement).and_then(|value| value.downcast_ref::<PlacementSetValue>()) else {
            return Some(Err(SceneNodeError::InvalidInput(node.identity().kind)));
        };
        let sample_indices = contribution_sample_indices(node, Some(*placement));
        let mut requests = Vec::new();
        for copy in &set.copies {
            if copy.time_offset == RationalTime::ZERO || copy.transform.is_none() { continue; }
            let Ok(at) = context.time.try_sub(copy.time_offset) else { continue };
            for index in &sample_indices {
                requests.push(DynamicInput { node: node.identity().inputs[*index], time: at });
            }
        }
        Some(Ok(requests))
    }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, SceneNodeError>> {
        if let Some(value) = self.ghost.execute(node, inputs, context) {
            return Some(value.map_err(Into::into));
        }
        if let Some(value) = self.group_composite.execute(node, inputs, context) {
            return Some(value.map_err(Into::into));
        }
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Contribution { layer, source, visibility, content, opacity, blend_value, matte_mode, effects, placement_effects, placement, masks, matte, clip_to_below, flatten, environment, projection, blend, order, kind } => (|| {
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
                    (4, Some(index)) => SceneContentValue::Particles(
                        inputs.at(index)
                            .and_then(|value| value.downcast_ref::<ParticleValue>())
                            .cloned()
                            .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
                    ),
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
                let placement_set = placement.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<PlacementSetValue>());
                let selected = placement_set.and_then(|set| set.selected_effect);
                let mut direct = Vec::new();
                let mut after = Vec::new();
                for (effect_index, input_index) in effects.iter().copied().enumerate() {
                    let Some(effect) = inputs.at(input_index).and_then(|value| value.downcast_ref::<EffectValue>()).and_then(|value| value.0.clone()) else { continue };
                    match selected {
                        Some(selected) if effect_index == selected => {}
                        Some(selected) if effect_index > selected => after.push(effect),
                        _ => direct.push(effect),
                    }
                }
                let masks = masks.iter().map(|index| inputs.at(*index).and_then(|value| value.downcast_ref::<MaskValue>()).map(|value| value.0.clone()).ok_or(SceneNodeError::InvalidInput(node.identity().kind))).collect::<Result<_, _>>()?;
                let base = SceneLayerValue { layer: *layer, source: source.clone(), transform, content_key, content, effects: direct, after_effects: Vec::new(), masks, matte, clip_to_below: *clip_to_below, flatten: *flatten, environment: *environment, ghost: false, opacity, projection: *projection, blend, order: *order };

                let Some(set) = placement_set.filter(|set| set.selected_effect.is_some()) else {
                    return Ok(NodeValue::new(SceneContributionValue { solo: visibility.solo, layer: Some(base) }));
                };

                let sample_indices = contribution_sample_indices(node, *placement);
                let mut dynamic_start = node.identity().inputs.len();
                let mut members = Vec::new();
                for copy in &set.copies {
                    let Some(copy_transform) = copy.transform else { continue };
                    if copy.time_offset == RationalTime::ZERO {
                        let mut layer = base.clone();
                        layer.transform = copy_transform;
                        layer.opacity = (layer.opacity * copy.opacity).clamp(0.0, 1.0);
                        members.push(SceneContributionValue { solo: visibility.solo, layer: Some(layer) });
                        continue;
                    }

                    let sample_start = dynamic_start;
                    dynamic_start += sample_indices.len();
                    let sampled_visibility = sampled_value(inputs, *visibility, &sample_indices, sample_start)
                        .and_then(|value| value.downcast_ref::<VisibilityValue>())
                        .copied()
                        .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                    if !sampled_visibility.active { continue; }

                    let mut layer = base.clone();
                    layer.content = sampled_content(node, inputs, *kind, *content, &sample_indices, sample_start)?;
                    layer.opacity = sampled_opacity(inputs, *opacity, &sample_indices, sample_start).unwrap_or(1.0).clamp(0.0, 1.0);
                    layer.blend = sampled_blend(node, inputs, *blend_value, *blend, &sample_indices, sample_start)?;
                    layer.matte = sampled_matte(
                        node,
                        inputs,
                        *matte,
                        *matte_mode,
                        *clip_to_below,
                        &sample_indices,
                        sample_start,
                    )?;
                    layer.masks = sampled_masks(node, inputs, masks, &sample_indices, sample_start)?;

                    let (sampled_direct, sampled_selected) = sampled_effects(
                        node,
                        inputs,
                        effects,
                        placement_effects,
                        &sample_indices,
                        sample_start,
                    )?;
                    layer.effects = sampled_direct;

                    if sampled_selected.is_some() {
                        layer.transform = copy_transform;
                        layer.opacity = (layer.opacity * copy.opacity).clamp(0.0, 1.0);
                    } else {
                        // The placement effect was disabled at the sampled
                        // source time. Legacy push_placements emits that sampled
                        // layer unchanged instead of applying the current copy.
                        layer.transform = sampled_value(inputs, 0, &sample_indices, sample_start)
                            .and_then(|value| value.downcast_ref::<TransformValue>())
                            .copied()
                            .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                    }
                    members.push(SceneContributionValue { solo: visibility.solo, layer: Some(layer) });
                }

                if members.is_empty() {
                    return Ok(NodeValue::new(SceneFragmentValue {
                        contributions: vec![SceneContributionValue { solo: visibility.solo, layer: None }],
                    }));
                }
                if after.is_empty() {
                    return Ok(NodeValue::new(SceneFragmentValue { contributions: members }));
                }

                // Effects below the placement effect see the copies as one picture.
                // Matte/clip remain on the outer plate, matching the old build order.
                for member in &mut members {
                    if let Some(layer) = member.layer.as_mut() {
                        layer.matte = None;
                        layer.clip_to_below = false;
                    }
                }
                let mut plate = base;
                plate.content_key = None;
                plate.content = SceneContentValue::Plate(ScenePlateValue { members, average: false });
                plate.effects = after;
                plate.after_effects.clear();
                plate.masks.clear();
                plate.transform = TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY };
                plate.opacity = 1.0;
                plate.projection = LayerProjection::TwoD;
                plate.flatten = false;
                plate.environment = false;
                Ok(NodeValue::new(SceneFragmentValue {
                    contributions: vec![SceneContributionValue { solo: visibility.solo, layer: Some(plate) }],
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

fn contribution_sample_indices(node: &GraphNode, placement: Option<usize>) -> Vec<usize> {
    (0..node.identity().inputs.len()).filter(|index| Some(*index) != placement).collect()
}

fn sampled_value<'a>(inputs: &'a NodeInputs, original: usize, sample_indices: &[usize], sample_start: usize) -> Option<&'a NodeValue> {
    sample_indices.iter().position(|index| *index == original).and_then(|offset| inputs.at(sample_start + offset))
}

fn sampled_content(
    node: &GraphNode,
    inputs: &NodeInputs,
    kind: u8,
    content: Option<usize>,
    sample_indices: &[usize],
    sample_start: usize,
) -> Result<SceneContentValue, SceneNodeError> {
    Ok(match (kind, content) {
        (1, Some(index)) => SceneContentValue::Text(
            sampled_value(inputs, index, sample_indices, sample_start)
                .and_then(|value| value.downcast_ref::<TextShapeValue>())
                .cloned()
                .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
        ),
        (2, Some(index)) => SceneContentValue::Shape(
            sampled_value(inputs, index, sample_indices, sample_start)
                .and_then(|value| value.downcast_ref::<Vec<ShapeNode>>())
                .cloned()
                .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
        ),
        (3, Some(index)) => {
            let value = sampled_value(inputs, index, sample_indices, sample_start)
                .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
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
        }
        (4, Some(index)) => SceneContentValue::Particles(
            sampled_value(inputs, index, sample_indices, sample_start)
                .and_then(|value| value.downcast_ref::<ParticleValue>())
                .cloned()
                .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
        ),
        _ => SceneContentValue::None,
    })
}

fn sampled_opacity(inputs: &NodeInputs, opacity: Option<usize>, sample_indices: &[usize], sample_start: usize) -> Option<f32> {
    opacity.and_then(|index| sampled_value(inputs, index, sample_indices, sample_start))
        .and_then(|value| value.downcast_ref::<crate::doc::eval::Value>())
        .and_then(|value| match value { crate::doc::eval::Value::F64(value) => Some(*value as f32), _ => None })
}

fn sampled_blend(
    node: &GraphNode,
    inputs: &NodeInputs,
    blend_value: Option<usize>,
    default: BlendMode,
    sample_indices: &[usize],
    sample_start: usize,
) -> Result<BlendMode, SceneNodeError> {
    match blend_value.and_then(|index| sampled_value(inputs, index, sample_indices, sample_start))
        .and_then(|value| value.downcast_ref::<crate::doc::eval::Value>())
    {
        Some(crate::doc::eval::Value::Enum(value)) => BlendMode::from_enum_value(*value).ok_or(SceneNodeError::InvalidInput(node.identity().kind)),
        Some(_) => Err(SceneNodeError::InvalidInput(node.identity().kind)),
        None => Ok(default),
    }
}

fn sampled_matte(
    node: &GraphNode,
    inputs: &NodeInputs,
    mut matte: Option<crate::doc::store::Matte>,
    matte_mode: Option<usize>,
    clip_to_below: bool,
    sample_indices: &[usize],
    sample_start: usize,
) -> Result<Option<crate::doc::store::Matte>, SceneNodeError> {
    if !clip_to_below {
        if let (Some(matte), Some(value)) = (
            matte.as_mut(),
            matte_mode.and_then(|index| sampled_value(inputs, index, sample_indices, sample_start))
                .and_then(|value| value.downcast_ref::<crate::doc::eval::Value>()),
        ) {
            match value {
                crate::doc::eval::Value::Enum(value) => {
                    matte.mode = crate::doc::store::MatteMode::from_enum_value(*value)
                        .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?;
                }
                _ => return Err(SceneNodeError::InvalidInput(node.identity().kind)),
            }
        }
    }
    Ok(matte)
}

fn sampled_masks(
    node: &GraphNode,
    inputs: &NodeInputs,
    masks: &[usize],
    sample_indices: &[usize],
    sample_start: usize,
) -> Result<Vec<crate::picture::resolved::ResolvedMask>, SceneNodeError> {
    masks.iter().map(|index| {
        sampled_value(inputs, *index, sample_indices, sample_start)
            .and_then(|value| value.downcast_ref::<MaskValue>())
            .map(|value| value.0.clone())
            .ok_or(SceneNodeError::InvalidInput(node.identity().kind))
    }).collect()
}

fn sampled_effects(
    node: &GraphNode,
    inputs: &NodeInputs,
    effects: &[usize],
    placement_effects: &[bool],
    sample_indices: &[usize],
    sample_start: usize,
) -> Result<(Vec<crate::picture::resolved::ResolvedEffect>, Option<usize>), SceneNodeError> {
    let mut values = Vec::with_capacity(effects.len());
    for index in effects {
        values.push(
            sampled_value(inputs, *index, sample_indices, sample_start)
                .and_then(|value| value.downcast_ref::<EffectValue>())
                .map(|value| value.0.clone())
                .ok_or(SceneNodeError::InvalidInput(node.identity().kind))?,
        );
    }
    let selected = values.iter().enumerate().find_map(|(index, value)| {
        (placement_effects.get(index).copied().unwrap_or(false) && value.is_some()).then_some(index)
    });
    let direct = values.into_iter().enumerate().filter_map(|(index, value)| {
        let effect = value?;
        match selected {
            Some(selected) if index >= selected => None,
            _ => Some(effect),
        }
    }).collect();
    Ok((direct, selected))
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
