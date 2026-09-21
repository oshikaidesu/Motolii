use std::collections::BTreeMap;

use crate::doc::core::{Fps, RationalTime};
use crate::doc::eval::Value;
use crate::doc::store::analysis::AnalysisInputs;
use crate::doc::store::{property, BlendMode, EffectId, LayerId, LayerProjection, LayerSource, PropertyId, StoreError, StoreView};
use crate::picture::resolved::ResolvedEffect;
use crate::render::media::blob::{BlobSettings, BlobSource, BlobTracker};

use super::{
    ContentProgram, DynamicInput, EffectProgram, EffectValue, EvaluationContext, GraphNode,
    GroupBackgroundProgram, MaskProgram, MaskValue, MaterialValue, MediaFrameValue, NodeIdentity,
    NodeInputs, NodeKey, NodeKind, NodeValue, ParticleProgram, ParticleValue, PropertyProgram,
    SceneContentValue, SceneLayerValue, TextProgram, TextShapeValue, TimeDependency, TransformProgram,
    TransformValue, VisibilityProgram, VisibilityValue,
};

#[derive(Clone)]
struct SourcePlan {
    source: LayerSource,
    transform: NodeKey,
    visibility: NodeKey,
    content: Option<(u8, NodeKey)>,
    opacity: Option<NodeKey>,
    effects: Vec<NodeKey>,
    masks: Vec<NodeKey>,
    projection: LayerProjection,
    flatten: bool,
    environment: bool,
    order: i16,
}

#[derive(Clone)]
enum Recipe {
    Request { target: LayerId, effect: NodeKey },
    Result { target: LayerId, request: NodeKey, start: i64, fps: Fps },
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlobAnalysisRequestValue {
    pub target: LayerId,
    pub effect: ResolvedEffect,
    pub source: Option<SceneLayerValue>,
    pub settings: BlobSettings,
    pub detail: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlobAnalysisValue {
    pub inputs: AnalysisInputs,
    pub tracker: BlobTracker,
    pub previous: Option<(Vec<u8>, u32, u32)>,
    pub mask: Option<(Vec<u8>, u32, u32)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnalysisBinding {
    pub layer: LayerId,
    pub request: NodeKey,
    pub result: NodeKey,
}

#[derive(Debug)]
pub enum AnalysisProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for AnalysisProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for AnalysisProgramError {}
impl From<StoreError> for AnalysisProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct AnalysisProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, AnalysisBinding>,
    sources: BTreeMap<LayerId, SourcePlan>,
}

impl AnalysisProgram {
    #[allow(clippy::too_many_arguments)]
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
        content: &ContentProgram,
        transforms: &TransformProgram,
        visibility: &VisibilityProgram,
        effects: &EffectProgram,
        masks: &MaskProgram,
        text: &TextProgram,
        groups: &GroupBackgroundProgram,
        particles: &ParticleProgram,
    ) -> Result<Self, AnalysisProgramError> {
        let fps = view.composition()?.map(|composition| composition.fps)
            .unwrap_or(Fps::try_new(30, 1).expect("30fps"));
        let mut sources = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let Some(transform) = transforms.binding(layer).map(|binding| binding.world) else { continue };
            let Some(visibility) = visibility.binding(layer).map(|binding| binding.node) else { continue };
            let content = match meta.source {
                LayerSource::Text => text.binding(layer).map(|binding| (1, binding.shape)),
                LayerSource::Shape => content.binding(layer).and_then(|binding| binding.content).map(|key| (2, key)),
                LayerSource::File { .. } => content.binding(layer).and_then(|binding| binding.material.or(binding.content)).map(|key| (3, key)),
                LayerSource::Group => groups.binding(layer).map(|key| (2, key)),
                LayerSource::Particles => particles.binding(layer).map(|binding| (4, binding.particles)),
                _ => None,
            };
            let opacity = properties.node_for(layer, &PropertyId::new(property::OPACITY)?);
            let effect_keys = effects.binding(layer).map(|binding| binding.effects.clone()).unwrap_or_default();
            let mask_keys = masks.binding(layer).map(|binding| binding.masks.clone()).unwrap_or_default();
            let attrs = view.attrs(layer)?.unwrap_or_default();
            sources.insert(layer, SourcePlan {
                source: meta.source,
                transform,
                visibility,
                content,
                opacity,
                effects: effect_keys,
                masks: mask_keys,
                projection: attrs.projection,
                flatten: attrs.flatten,
                environment: attrs.environment,
                order: meta.order,
            });
        }

        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            let Some(effect_key) = effects.binding(layer)
                .and_then(|binding| binding.effects.iter().copied().find(|key| {
                    effects.plugin_id(*key).is_some_and(crate::extensions::blob::is_blob_track)
                })) else { continue };

            let mut request_identity = NodeIdentity::new(NodeKind::AnalysisRequest, vec![effect_key]);
            request_identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            request_identity.time_dependency = TimeDependency::Exact;
            let request_node = GraphNode::new(request_identity);
            let request = request_node.key();
            nodes.insert(request, request_node);
            recipes.insert(request, Recipe::Request { target: layer, effect: effect_key });

            let mut result_identity = NodeIdentity::new(NodeKind::AnalysisBlob, vec![request]);
            result_identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            result_identity.parameters.extend_from_slice(&meta.timing.start.to_be_bytes());
            result_identity.parameters.extend_from_slice(&fps.num().to_be_bytes());
            result_identity.parameters.extend_from_slice(&fps.den().to_be_bytes());
            result_identity.time_dependency = TimeDependency::Exact;
            let result_node = GraphNode::new(result_identity);
            let result = result_node.key();
            nodes.insert(result, result_node);
            recipes.insert(result, Recipe::Result { target: layer, request, start: meta.timing.start, fps });
            bindings.insert(layer, AnalysisBinding { layer, request, result });
        }

        Ok(Self { nodes, recipes, bindings, sources })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<AnalysisBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, AnalysisProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Request { .. } => {
                let effect = inputs.at(0)
                    .and_then(|value| value.downcast_ref::<EffectValue>())
                    .and_then(|value| value.0.as_ref());
                let source = effect.and_then(|effect| source_layer(&effect.params));
                let Some(plan) = source.and_then(|source| self.sources.get(&source)) else { return Some(Ok(Vec::new())); };
                let mut keys = vec![plan.transform, plan.visibility];
                if let Some((_, content)) = plan.content { keys.push(content); }
                if let Some(opacity) = plan.opacity { keys.push(opacity); }
                keys.extend(plan.effects.iter().copied());
                keys.extend(plan.masks.iter().copied());
                Ok(keys.into_iter().map(|key| DynamicInput { node: key, time: context.time }).collect())
            }
            Recipe::Result { start, fps, .. } => {
                let request = inputs.at(0)
                    .and_then(|value| value.downcast_ref::<BlobAnalysisRequestValue>())
                    .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                let sequential = request.settings.persist || matches!(request.settings.source, BlobSource::Motion { .. });
                if !sequential {
                    Ok(Vec::new())
                } else {
                    let frame = context.time.try_to_frame_round(*fps).unwrap_or(*start);
                    if frame <= *start {
                        Ok(Vec::new())
                    } else {
                        let previous = RationalTime::try_from_frame(frame - 1, *fps)
                            .map_err(|error| StoreError::Property(error.to_string()))?;
                        Ok(vec![DynamicInput { node: node.key(), time: previous }])
                    }
                }
            }
        })
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, AnalysisProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        match recipe {
            Recipe::Result { .. } => None,
            Recipe::Request { target, .. } => Some((|| {
                let effect = inputs.at(0)
                    .and_then(|value| value.downcast_ref::<EffectValue>())
                    .and_then(|value| value.0.clone())
                    .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                let settings = settings_of(&effect.params);
                let detail = crate::extensions::blob::number_of(&effect.params, "detail").round().clamp(120.0, 3840.0) as u32;
                let source_id = source_layer(&effect.params);
                let source = match source_id.and_then(|source| self.sources.get(&source).map(|plan| (source, plan))) {
                    None => None,
                    Some((source_id, plan)) => {
                        let mut cursor = 1usize;
                        let transform = inputs.at(cursor)
                            .and_then(|value| value.downcast_ref::<TransformValue>())
                            .copied()
                            .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                        cursor += 1;
                        let visibility = inputs.at(cursor)
                            .and_then(|value| value.downcast_ref::<VisibilityValue>())
                            .copied()
                            .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                        cursor += 1;
                        if !visibility.active {
                            None
                        } else {
                            let content = match plan.content {
                                Some((1, _)) => {
                                    let value = inputs.at(cursor)
                                        .and_then(|value| value.downcast_ref::<TextShapeValue>())
                                        .cloned()
                                        .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                                    cursor += 1;
                                    SceneContentValue::Text(value)
                                }
                                Some((2, _)) => {
                                    let value = inputs.at(cursor)
                                        .and_then(|value| value.downcast_ref::<Vec<crate::doc::store::ShapeNode>>())
                                        .cloned()
                                        .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                                    cursor += 1;
                                    SceneContentValue::Shape(value)
                                }
                                Some((3, _)) => {
                                    let value = inputs.at(cursor).ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                                    cursor += 1;
                                    if let Some(material) = value.downcast_ref::<MaterialValue>() {
                                        SceneContentValue::Material(material.clone())
                                    } else if let Some(frame) = value.downcast_ref::<MediaFrameValue>() {
                                        match frame.time {
                                            Some(time) => SceneContentValue::Media { source: frame.source.clone(), time },
                                            None => SceneContentValue::None,
                                        }
                                    } else {
                                        return Err(AnalysisProgramError::InvalidInput(node.identity().kind));
                                    }
                                }
                                Some((4, _)) => {
                                    let value = inputs.at(cursor)
                                        .and_then(|value| value.downcast_ref::<ParticleValue>())
                                        .cloned()
                                        .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                                    cursor += 1;
                                    SceneContentValue::Particles(value)
                                }
                                _ => SceneContentValue::None,
                            };
                            let opacity = if plan.opacity.is_some() {
                                let value = inputs.at(cursor)
                                    .and_then(|value| value.downcast_ref::<Value>())
                                    .and_then(|value| match value { Value::F64(value) => Some(*value as f32), _ => None })
                                    .unwrap_or(1.0)
                                    .clamp(0.0, 1.0);
                                cursor += 1;
                                value
                            } else { 1.0 };
                            let mut resolved_effects = Vec::new();
                            for _ in &plan.effects {
                                if let Some(effect) = inputs.at(cursor)
                                    .and_then(|value| value.downcast_ref::<EffectValue>())
                                    .and_then(|value| value.0.clone())
                                {
                                    resolved_effects.push(effect);
                                }
                                cursor += 1;
                            }
                            let mut resolved_masks = Vec::new();
                            for _ in &plan.masks {
                                let mask = inputs.at(cursor)
                                    .and_then(|value| value.downcast_ref::<MaskValue>())
                                    .map(|value| value.0.clone())
                                    .ok_or(AnalysisProgramError::InvalidInput(node.identity().kind))?;
                                resolved_masks.push(mask);
                                cursor += 1;
                            }
                            Some(SceneLayerValue {
                                layer: source_id,
                                instance: 0,
                                source: plan.source.clone(),
                                transform,
                                content_key: None,
                                content,
                                effects: resolved_effects,
                                after_effects: Vec::new(),
                                image_sources: Vec::new(),
                                masks: resolved_masks,
                                matte: None,
                                clip_to_below: false,
                                flatten: plan.flatten,
                                environment: plan.environment,
                                ghost: false,
                                freeze_eligible: false,
                                timing_start: 0,
                                opacity,
                                projection: plan.projection,
                                blend: BlendMode::Normal,
                                order: plan.order,
                                shape_stretch: [1.0, 1.0],
                                depth: 0.0,
                            })
                        }
                    }
                };
                Ok(NodeValue::new(BlobAnalysisRequestValue {
                    target: *target,
                    effect,
                    source,
                    settings,
                    detail,
                }))
            })()),
        }
    }
}

fn source_layer(params: &[(String, Value)]) -> Option<LayerId> {
    let id = crate::extensions::blob::number_of(params, "source").round().max(0.0) as u64;
    (id != 0).then_some(LayerId(id))
}

fn settings_of(params: &[(String, Value)]) -> BlobSettings {
    let number = |name| crate::extensions::blob::number_of(params, name);
    let source = match number("mode").round() as i64 {
        1 => BlobSource::Motion { threshold: number("threshold") as f32 },
        2 => BlobSource::Color {
            target: [number("red") as f32, number("green") as f32, number("blue") as f32],
            tolerance: number("tolerance") as f32,
        },
        _ => BlobSource::Luminance { threshold: number("threshold") as f32, invert: number("invert") >= 0.5 },
    };
    BlobSettings {
        source,
        min_area: number("min_area").max(0.0) as u32,
        max_area: number("max_area").clamp(0.0, u32::MAX as f64) as u32,
        max_blobs: number("max_blobs").max(1.0) as usize,
        persist: number("persist") >= 0.5,
        max_move: number("max_move").max(0.0) as f32,
        revive_frames: number("revive").max(0.0) as u32,
        separation: number("separation").max(0.0).round() as u32,
        blur: 0,
    }
}
