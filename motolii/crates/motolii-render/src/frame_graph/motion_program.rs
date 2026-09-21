use std::collections::BTreeMap;

use crate::doc::core::{Fps, RationalTime};
use crate::doc::eval::Value;
use crate::doc::store::{kind::SamplingProgram as SamplingEvaluator, property, LayerId, LayerSource, PropertyId, StoreError, StoreView};

use super::transform::compose_transform_value;
use super::{DynamicInput, EffectProgram, EffectValue, EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency, TransformProgram, TransformValue};

const ROWS: [&str; 10] = [
    property::POSITION,
    property::ANCHOR,
    property::SCALE,
    property::ROTATION,
    property::SKEW,
    property::SKEW_AXIS,
    property::POSITION_Z,
    property::ROTATION_X,
    property::ROTATION_Y,
    property::SCALE_Z,
];

#[derive(Clone, Debug, PartialEq)]
pub struct MotionPlanValue {
    pub selected_effect: Option<usize>,
    pub channels: [bool; 3],
    pub sample_times: Vec<RationalTime>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionSamplesValue {
    pub selected_effect: Option<usize>,
    pub transforms: Vec<TransformValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MotionBinding {
    pub layer: LayerId,
    pub measure: NodeKey,
    pub samples: NodeKey,
}

#[derive(Clone)]
struct SampleRecipe {
    world: usize,
    parent_world: Option<usize>,
    rows: [Option<usize>; 10],
    flow: Option<(usize, usize)>,
}

#[derive(Clone)]
enum Recipe {
    Measure {
        effect_count: usize,
        programs: Vec<Option<SamplingEvaluator>>,
        sample: SampleRecipe,
        fps: Fps,
        size: [f32; 2],
    },
    Samples {
        plan: usize,
        sample: SampleRecipe,
    },
}

#[derive(Debug)]
pub enum MotionProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for MotionProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for MotionProgramError {}
impl From<StoreError> for MotionProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct MotionProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, MotionBinding>,
}

impl MotionProgram {
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
        effects: &EffectProgram,
        transforms: &TransformProgram,
        flow: &FlowProgram,
    ) -> Result<Self, MotionProgramError> {
        let Some(comp) = view.composition()? else {
            return Ok(Self { nodes: BTreeMap::new(), recipes: BTreeMap::new(), bindings: BTreeMap::new() });
        };

        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if meta.source == LayerSource::Group { continue; }
            let Some(effect_binding) = effects.binding(layer) else { continue };
            let programs: Vec<_> = effect_binding.effects.iter().map(|key| effects.sampling_program(*key)).collect();
            if programs.iter().all(Option::is_none) { continue; }

            let Some(transform_binding) = transforms.binding(layer) else { continue };

            let mut measure_inputs = effect_binding.effects.clone();
            let effect_count = measure_inputs.len();
            let measure_sample = append_sample_inputs(
                view,
                properties,
                transforms,
                flow,
                layer,
                transform_binding.world,
                &mut measure_inputs,
            )?;
            let mut measure_identity = NodeIdentity::new(NodeKind::MotionMeasure, measure_inputs);
            measure_identity.parameters.extend_from_slice(&comp.fps.num().to_be_bytes());
            measure_identity.parameters.extend_from_slice(&comp.fps.den().to_be_bytes());
            let size = meta.source.declared_size().unwrap_or([0.0, 0.0]);
            for value in size {
                measure_identity.parameters.extend_from_slice(&value.to_bits().to_be_bytes());
            }
            measure_identity.time_dependency = TimeDependency::Exact;
            let measure_node = GraphNode::new(measure_identity);
            let measure_key = measure_node.key();
            nodes.entry(measure_key).or_insert(measure_node);
            recipes.entry(measure_key).or_insert(Recipe::Measure {
                effect_count,
                programs: programs.clone(),
                sample: measure_sample,
                fps: comp.fps,
                size,
            });

            let mut sample_inputs = vec![measure_key];
            let sample_recipe = append_sample_inputs(
                view,
                properties,
                transforms,
                flow,
                layer,
                transform_binding.world,
                &mut sample_inputs,
            )?;
            let mut sample_identity = NodeIdentity::new(NodeKind::MotionSamples, sample_inputs);
            sample_identity.time_dependency = TimeDependency::Exact;
            let sample_node = GraphNode::new(sample_identity);
            let sample_key = sample_node.key();
            nodes.entry(sample_key).or_insert(sample_node);
            recipes.entry(sample_key).or_insert(Recipe::Samples { plan: 0, sample: sample_recipe });

            bindings.insert(layer, MotionBinding { layer, measure: measure_key, samples: sample_key });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<MotionBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, MotionProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Measure { effect_count, programs, sample, fps, .. } => {
                let Some((_, program, effect)) = selected_effect(*effect_count, programs, inputs) else {
                    return Some(Ok(Vec::new()));
                };
                let Some(shutter) = (program.shutter)(&effect.params) else {
                    return Some(Ok(Vec::new()));
                };
                let edge_times = shutter_edges(context.time, *fps, shutter.width_frames);
                let sources = sample_sources(sample, shutter.channels);
                Ok(edge_times.into_iter().flat_map(|time| {
                    sources.iter().map(move |index| DynamicInput { node: node.identity().inputs[*index], time })
                }).collect())
            }
            Recipe::Samples { plan, sample } => {
                let Some(plan) = inputs.at(*plan).and_then(|value| value.downcast_ref::<MotionPlanValue>()) else {
                    return Some(Err(MotionProgramError::InvalidInput(node.identity().kind)));
                };
                let sources = sample_sources(sample, plan.channels);
                Ok(plan.sample_times.iter().flat_map(|time| {
                    sources.iter().map(move |index| DynamicInput { node: node.identity().inputs[*index], time: *time })
                }).collect())
            }
        })
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, MotionProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Measure { effect_count, programs, sample, fps, size } => (|| {
                let Some((selected, program, effect)) = selected_effect(*effect_count, programs, inputs) else {
                    return Ok(NodeValue::new(MotionPlanValue { selected_effect: None, channels: [false; 3], sample_times: Vec::new() }));
                };
                let Some(shutter) = (program.shutter)(&effect.params) else {
                    return Ok(NodeValue::new(MotionPlanValue { selected_effect: Some(selected), channels: [false; 3], sample_times: Vec::new() }));
                };
                let sources = sample_sources(sample, shutter.channels);
                let static_len = node.identity().inputs.len();
                let current = mixed_local(node, inputs, sample, shutter.channels, None, &sources)?;
                let world = read_transform(node, inputs, sample.world)?;
                let parent = sample.parent_world
                    .map(|index| read_transform(node, inputs, index))
                    .transpose()?
                    .unwrap_or(identity_transform());
                let edges = shutter_edges(context.time, *fps, shutter.width_frames);
                if edges.len() != 2 || sources.is_empty() {
                    return Ok(NodeValue::new(MotionPlanValue { selected_effect: Some(selected), channels: shutter.channels, sample_times: Vec::new() }));
                }
                let early = mixed_local(node, inputs, sample, shutter.channels, Some(static_len), &sources)?;
                let late = mixed_local(node, inputs, sample, shutter.channels, Some(static_len + sources.len()), &sources)?;
                let early_world = sampled_world(current, early, world, parent);
                let late_world = sampled_world(current, late, world, parent);
                let [w, h] = *size;
                let (lo, hi) = if w > 0.0 && h > 0.0 {
                    ([0.0, 0.0], [w, h])
                } else {
                    ([-100.0, -100.0], [100.0, 100.0])
                };
                let travel = [[lo[0], lo[1]], [hi[0], lo[1]], [lo[0], hi[1]], [hi[0], hi[1]]]
                    .map(|corner| {
                        let p = glam::Vec2::from(corner);
                        early_world.affine.transform_point2(p).distance(late_world.affine.transform_point2(p))
                    })
                    .into_iter()
                    .fold(0.0f32, f32::max);
                let count = (shutter.count)(travel);
                let sample_times = if count <= 1 {
                    Vec::new()
                } else {
                    shutter_times(context.time, *fps, &shutter, count)
                };
                Ok(NodeValue::new(MotionPlanValue {
                    selected_effect: Some(selected),
                    channels: shutter.channels,
                    sample_times,
                }))
            })(),
            Recipe::Samples { plan, sample } => (|| {
                let plan = inputs.at(*plan)
                    .and_then(|value| value.downcast_ref::<MotionPlanValue>())
                    .cloned()
                    .ok_or(MotionProgramError::InvalidInput(node.identity().kind))?;
                let Some(selected) = plan.selected_effect else {
                    return Ok(NodeValue::new(MotionSamplesValue { selected_effect: None, transforms: Vec::new() }));
                };
                if plan.sample_times.is_empty() {
                    return Ok(NodeValue::new(MotionSamplesValue { selected_effect: Some(selected), transforms: Vec::new() }));
                }

                let sources = sample_sources(sample, plan.channels);
                let static_len = node.identity().inputs.len();
                let current = mixed_local(node, inputs, sample, plan.channels, None, &sources)?;
                let world = read_transform(node, inputs, sample.world)?;
                let parent = sample.parent_world
                    .map(|index| read_transform(node, inputs, index))
                    .transpose()?
                    .unwrap_or(identity_transform());
                let mut transforms = Vec::with_capacity(plan.sample_times.len());
                for index in 0..plan.sample_times.len() {
                    let sampled = mixed_local(
                        node,
                        inputs,
                        sample,
                        plan.channels,
                        Some(static_len + index * sources.len()),
                        &sources,
                    )?;
                    transforms.push(sampled_world(current, sampled, world, parent));
                }
                Ok(NodeValue::new(MotionSamplesValue { selected_effect: Some(selected), transforms }))
            })(),
        })
    }
}

fn append_sample_inputs(
    view: &StoreView<'_>,
    properties: &PropertyProgram,
    transforms: &TransformProgram,
    flow: &FlowProgram,
    layer: LayerId,
    world: NodeKey,
    inputs: &mut Vec<NodeKey>,
) -> Result<SampleRecipe, MotionProgramError> {
    let world_index = inputs.len();
    inputs.push(world);
    let parent_world = view.attrs(layer)?.unwrap_or_default().parent
        .and_then(|parent| transforms.binding(parent))
        .map(|binding| {
            let index = inputs.len();
            inputs.push(binding.world);
            index
        });
    let rows: [Option<usize>; 10] = std::array::from_fn(|row| {
        properties.node_for(layer, &PropertyId::new(ROWS[row]).expect("known transform property")).map(|key| {
            let index = inputs.len();
            inputs.push(key);
            index
        })
    });
    let flow = flow.binding(layer).map(|binding| {
        let index = inputs.len();
        inputs.push(flow.key());
        (index, binding.index)
    });
    Ok(SampleRecipe { world: world_index, parent_world, rows, flow })
}

fn selected_effect<'a>(
    effect_count: usize,
    programs: &[Option<SamplingEvaluator>],
    inputs: &'a NodeInputs,
) -> Option<(usize, SamplingEvaluator, &'a crate::picture::resolved::ResolvedEffect)> {
    (0..effect_count).find_map(|index| {
        let program = programs.get(index).copied().flatten()?;
        let effect = inputs.at(index)?.downcast_ref::<EffectValue>()?.0.as_ref()?;
        Some((index, program, effect))
    })
}

fn sample_sources(recipe: &SampleRecipe, channels: [bool; 3]) -> Vec<usize> {
    let mut out = Vec::new();
    if channels[0] {
        if let Some(index) = recipe.rows[0] { out.push(index); }
        if let Some((index, _)) = recipe.flow { if !out.contains(&index) { out.push(index); } }
    }
    if channels[1] {
        if let Some(index) = recipe.rows[2] { if !out.contains(&index) { out.push(index); } }
    }
    if channels[2] {
        if let Some(index) = recipe.rows[3] { if !out.contains(&index) { out.push(index); } }
    }
    out
}

fn mixed_local(
    node: &GraphNode,
    inputs: &NodeInputs,
    recipe: &SampleRecipe,
    channels: [bool; 3],
    dynamic_start: Option<usize>,
    sources: &[usize],
) -> Result<TransformValue, MotionProgramError> {
    let dynamic_value = |original: usize| -> Option<&NodeValue> {
        let start = dynamic_start?;
        let offset = sources.iter().position(|index| *index == original)?;
        inputs.at(start + offset)
    };
    let values: [Option<&Value>; 10] = std::array::from_fn(|row| {
        let sampled = match row {
            0 if channels[0] => recipe.rows[row].and_then(dynamic_value),
            2 if channels[1] => recipe.rows[row].and_then(dynamic_value),
            3 if channels[2] => recipe.rows[row].and_then(dynamic_value),
            _ => None,
        };
        sampled.or_else(|| recipe.rows[row].and_then(|index| inputs.at(index)))
            .and_then(|value| value.downcast_ref::<Value>())
    });
    let flow_value = match (channels[0], recipe.flow) {
        (true, Some((index, slot))) => dynamic_value(index)
            .or_else(|| inputs.at(index))
            .and_then(|value| value.downcast_ref::<FlowFrameValue>())
            .and_then(|flow| flow.slots.get(slot))
            .copied()
            .flatten(),
        (_, Some((index, slot))) => inputs.at(index)
            .and_then(|value| value.downcast_ref::<FlowFrameValue>())
            .and_then(|flow| flow.slots.get(slot))
            .copied()
            .flatten(),
        (_, None) => None,
    };
    compose_transform_value(values, flow_value).ok_or(MotionProgramError::InvalidInput(node.identity().kind))
}

fn read_transform(node: &GraphNode, inputs: &NodeInputs, index: usize) -> Result<TransformValue, MotionProgramError> {
    inputs.at(index)
        .and_then(|value| value.downcast_ref::<TransformValue>())
        .copied()
        .ok_or(MotionProgramError::InvalidInput(node.identity().kind))
}

fn identity_transform() -> TransformValue {
    TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY }
}

fn sampled_world(current_local: TransformValue, sampled_local: TransformValue, world: TransformValue, parent: TransformValue) -> TransformValue {
    let delta2 = sampled_local.affine * current_local.affine.inverse();
    let delta3 = sampled_local.spatial * current_local.spatial.inverse();
    TransformValue {
        affine: parent.affine * delta2 * parent.affine.inverse() * world.affine,
        spatial: parent.spatial * delta3 * parent.spatial.inverse() * world.spatial,
    }
}

fn shutter_edges(time: RationalTime, fps: Fps, width_frames: f64) -> Vec<RationalTime> {
    let frame_seconds = fps.den() as f64 / fps.num() as f64;
    [-0.5f64, 0.5].into_iter()
        .filter_map(|share| offset_time(time, share * width_frames * frame_seconds))
        .collect()
}

fn shutter_times(time: RationalTime, fps: Fps, shutter: &crate::doc::store::kind::Shutter, count: u32) -> Vec<RationalTime> {
    let frame_seconds = fps.den() as f64 / fps.num() as f64;
    (0..count)
        .filter_map(|index| offset_time(time, (shutter.offset)(index, count) * shutter.width_frames * frame_seconds))
        .collect()
}

fn offset_time(time: RationalTime, seconds: f64) -> Option<RationalTime> {
    const DEN: i64 = 1_000_000;
    let delta = RationalTime::try_new((seconds * DEN as f64).round() as i64, DEN).ok()?;
    time.try_add(delta).ok()
}
