use std::collections::BTreeMap;

use crate::doc::core::RationalTime;
use crate::doc::eval::Value;
use crate::doc::store::{property, kind::PlacementProgram as PlacementEvaluator, LayerId, LayerSource, PropertyId, StoreError, StoreView};

use super::{AnalysisProgram, BlobAnalysisValue, DynamicInput, EffectProgram, EffectValue, EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency, TransformProgram, TransformValue};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacementCopyValue {
    pub index: u32,
    pub transform: Option<TransformValue>,
    pub opacity: f32,
    pub time_offset: RationalTime,
    pub outline_stretch: [f32; 2],
    /// A text unit's box (material coordinates): the member is the whole layer cut to it.
    pub cut: Option<[f32; 4]>,
}

/// The selected effect of a set whose members are a split text's units.
pub const TEXT_UNITS: usize = usize::MAX;

#[derive(Clone, Debug, PartialEq)]
pub struct PlacementSetValue {
    /// The placement effect in force, or `TEXT_UNITS` when the members are a split text's units (every effect stays on
    /// each unit, none is after).
    pub selected_effect: Option<usize>,
    pub copies: Vec<PlacementCopyValue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlacementBinding {
    pub layer: LayerId,
    pub node: NodeKey,
}

#[derive(Clone)]
struct Recipe {
    layer: LayerId,
    effect_count: usize,
    programs: Vec<Option<PlacementEvaluator>>,
    world: usize,
    parent_world: Option<usize>,
    position: Option<usize>,
    position_x: Option<usize>,
    position_y: Option<usize>,
    position_z: Option<usize>,
    stretch_outline: bool,
    analysis: Option<usize>,
    /// The layer's order rows (Stagger, Stagger From, From End): copies are members in order and take the shared law.
    schedule: [Option<usize>; 3],
    /// A text's split: its shaped document, its Split row, its Anchor, and the canvas (the members are its units).
    units: Option<Units>,
}

#[derive(Clone, Copy)]
struct Units {
    shape: usize,
    split: Option<usize>,
    anchor: Option<usize>,
    canvas: (u32, u32),
}

#[derive(Debug)]
pub enum PlacementProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
}

impl std::fmt::Display for PlacementProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for PlacementProgramError {}
impl From<StoreError> for PlacementProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

/// Non-Group placement effects. The output is still semantic: zero-offset
/// copies carry a resolved current-time transform, while shifted copies carry
/// only their requested time offset until the temporal-sampling lane owns them.
pub struct PlacementProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, PlacementBinding>,
}

impl PlacementProgram {
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
        effects: &EffectProgram,
        transforms: &TransformProgram,
        analysis: &AnalysisProgram,
        text: &super::TextFlowProgram,
    ) -> Result<Self, PlacementProgramError> {
        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut bindings = BTreeMap::new();

        for layer in view.layers() {
            let Some(meta) = view.meta(layer)? else { continue };
            if meta.source == LayerSource::Group { continue; }
            let layer_effects = effects.binding(layer).map(|binding| binding.effects.clone()).unwrap_or_default();
            let programs: Vec<_> = layer_effects.iter().map(|key| effects.placement_program(*key)).collect();
            // A split text's units are members too (`push_split`); its set is built even with no placement effect. The
            // shape is the one the scene draws (the text after flowing).
            let text_shape = (meta.source == LayerSource::Text).then(|| text.binding(layer)).flatten();
            if programs.iter().all(Option::is_none) && text_shape.is_none() { continue; }

            let Some(transform) = transforms.binding(layer) else { continue };
            let mut inputs = layer_effects;
            let effect_count = inputs.len();

            let world = inputs.len();
            inputs.push(transform.world);

            let parent_world = view.attrs(layer)?.unwrap_or_default().parent
                .and_then(|parent| transforms.binding(parent))
                .map(|binding| {
                    let index = inputs.len();
                    inputs.push(binding.world);
                    index
                });

            let prop = |name: &str| PropertyId::new(name).expect("known placement property");
            let mut add_property = |property: PropertyId| {
                properties.node_for(layer, &property).map(|key| {
                    let index = inputs.len();
                    inputs.push(key);
                    index
                })
            };
            let position = add_property(prop(property::POSITION));
            let position_x = add_property(prop(property::POSITION_X));
            let position_y = add_property(prop(property::POSITION_Y));
            let position_z = add_property(prop(property::POSITION_Z));

            let analysis = analysis.binding(layer).map(|binding| {
                let index = inputs.len();
                inputs.push(binding.result);
                index
            });
            let comp = view.composition()?.map_or((0, 0), |c| (c.width, c.height));
            let units = text_shape.map(|binding| {
                let shape = inputs.len();
                inputs.push(binding.shape);
                let split = properties.node_for(layer, &prop(crate::doc::store::names::TEXT_SPLIT)).map(|key| { let i = inputs.len(); inputs.push(key); i });
                let anchor = properties.node_for(layer, &prop(property::ANCHOR)).map(|key| { let i = inputs.len(); inputs.push(key); i });
                Units { shape, split, anchor, canvas: comp }
            });
            let schedule = [crate::doc::store::layout::STAGGER, crate::doc::store::layout::STAGGER_FROM, crate::doc::store::layout::FROM_END].map(|name| {
                properties.node_for(layer, &prop(name)).map(|key| {
                    let index = inputs.len();
                    inputs.push(key);
                    index
                })
            });

            let mut identity = NodeIdentity::new(NodeKind::PlacementSet, inputs);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.push(u8::from(meta.source == LayerSource::Shape));
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            nodes.entry(key).or_insert(node);
            recipes.entry(key).or_insert(Recipe {
                layer,
                effect_count,
                programs,
                world,
                parent_world,
                position,
                position_x,
                position_y,
                position_z,
                stretch_outline: meta.source == LayerSource::Shape,
                analysis,
                schedule,
                units,
            });
            bindings.insert(layer, PlacementBinding { layer, node: key });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<PlacementBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, PlacementProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            if let Some(cuts) = unit_cuts(recipe, inputs, context.time) {
                let sample_indices = sample_indices(recipe);
                let mut requests = Vec::new();
                for (_, offset) in cuts {
                    if offset == RationalTime::ZERO { continue; }
                    let Ok(at) = context.time.try_sub(offset) else { continue };
                    for index in &sample_indices {
                        requests.push(DynamicInput { node: node.identity().inputs[*index], time: at });
                    }
                }
                return Ok(requests);
            }
            let Some((_, program, effect)) = selected_effect(recipe, inputs) else { return Ok(Vec::new()); };
            let position = read_position(node, inputs, recipe, None)?;
            let analysis = recipe.analysis
                .and_then(|index| inputs.at(index))
                .and_then(|value| value.downcast_ref::<BlobAnalysisValue>())
                .map(|value| &value.inputs);
            let outputs = (program.evaluate)(&crate::doc::store::kind::PlacementInput {
                params: &effect.params,
                layer: recipe.layer,
                time: context.time,
                position,
                stretch_outline: recipe.stretch_outline,
                analysis,
            });
            let outputs = with_schedule(outputs, inputs, recipe);
            let sample_indices = sample_indices(recipe);
            let mut requests = Vec::new();
            for output in outputs {
                if output.placement.time_offset == RationalTime::ZERO { continue; }
                let Ok(at) = context.time.try_sub(output.placement.time_offset) else { continue };
                for index in &sample_indices {
                    requests.push(DynamicInput { node: node.identity().inputs[*index], time: at });
                }
            }
            Ok(requests)
        })())
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, PlacementProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            if let Some(cuts) = unit_cuts(recipe, inputs, context.time) {
                return unit_set(node, inputs, recipe, cuts).map(NodeValue::new);
            }
            let Some((selected_effect, program, effect)) = selected_effect(recipe, inputs) else {
                return Ok(NodeValue::new(PlacementSetValue { selected_effect: None, copies: Vec::new() }));
            };

            let position = read_position(node, inputs, recipe, None)?;
            let analysis = recipe.analysis
                .and_then(|index| inputs.at(index))
                .and_then(|value| value.downcast_ref::<BlobAnalysisValue>())
                .map(|value| &value.inputs);
            let outputs = (program.evaluate)(&crate::doc::store::kind::PlacementInput {
                params: &effect.params,
                layer: recipe.layer,
                time: context.time,
                position,
                stretch_outline: recipe.stretch_outline,
                analysis,
            });
            let outputs = with_schedule(outputs, inputs, recipe);

            let sample_indices = sample_indices(recipe);
            let static_len = node.identity().inputs.len();
            let mut dynamic_start = static_len;
            let mut copies = Vec::with_capacity(outputs.len());
            for output in outputs {
                let placement = output.placement;
                let sampled_start = if placement.time_offset == RationalTime::ZERO {
                    None
                } else if context.time.try_sub(placement.time_offset).is_ok() {
                    let start = dynamic_start;
                    dynamic_start += sample_indices.len();
                    Some(start)
                } else {
                    None
                };
                let transform = if placement.time_offset == RationalTime::ZERO || sampled_start.is_some() {
                    let world = read_transform_sampled(node, inputs, recipe.world, &sample_indices, sampled_start)?;
                    let parent = recipe.parent_world
                        .map(|index| read_transform_sampled(node, inputs, index, &sample_indices, sampled_start))
                        .transpose()?
                        .unwrap_or(TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY });
                    let sampled_position = read_position(node, inputs, recipe, sampled_start)?;
                    let pivot = glam::Vec2::from(sampled_position);
                    let depth = read_number_sampled(inputs, recipe.position_z, &sample_indices, sampled_start).unwrap_or(0.0) as f32;
                    Some(TransformValue {
                        affine: parent.affine * placement.affine2(pivot) * parent.affine.inverse() * world.affine,
                        spatial: parent.spatial * placement.affine3(pivot.extend(depth)) * parent.spatial.inverse() * world.spatial,
                    })
                } else {
                    None
                };
                copies.push(PlacementCopyValue {
                    index: placement.index,
                    transform,
                    opacity: placement.opacity,
                    time_offset: placement.time_offset,
                    outline_stretch: output.outline_stretch,
                    cut: None,
                });
            }

            Ok(NodeValue::new(PlacementSetValue { selected_effect: Some(selected_effect), copies }))
        })())
    }
}

/// A split text's units now, each with its time offset, when they are members: the text's Stagger is above zero and there
/// are at least two units (`push_split`). The boxes are read at the frame time, the time offsets by the shared order law.
fn unit_cuts(recipe: &Recipe, inputs: &NodeInputs, time: RationalTime) -> Option<Vec<([f32; 4], RationalTime)>> {
    let units = recipe.units?;
    let number = |index: Option<usize>| -> f64 {
        match index.and_then(|i| inputs.at(i)).and_then(|v| v.downcast_ref::<Value>()) {
            Some(Value::F64(v)) if v.is_finite() => *v,
            Some(Value::Enum(v)) => *v as f64,
            _ => 0.0,
        }
    };
    let stagger = number(recipe.schedule[0]);
    let split = number(units.split).round() as i64;
    if stagger <= 0.0 || split == 0 {
        return None;
    }
    let text = inputs.at(units.shape)?.downcast_ref::<super::TextShapeValue>()?;
    let canvas = crate::picture::shapes_ops::Canvas { width: units.canvas.0, height: units.canvas.1, origin_x: 0, origin_y: 0 };
    let content = text.document.content.eval(time);
    let boxes = crate::picture::text_frame::split_boxes(&text.document, &text.shaped, &canvas, &content, split);
    if boxes.len() < 2 {
        return None;
    }
    let (from, from_end, n) = (number(recipe.schedule[1]).round() as i64, number(recipe.schedule[2]).round() as i64 == 1, boxes.len());
    Some(boxes.into_iter().enumerate().map(|(k, b)| {
        let delay = crate::doc::store::layout::schedule_delay(stagger, from, from_end, k, n);
        // The document never reads before zero: the unit's time stops at 0 (`schedule_shift`).
        let at = crate::doc::store::layout::shift_by(time, delay).unwrap_or(time);
        (b, time.try_sub(at).unwrap_or(RationalTime::ZERO))
    }).collect())
}

/// The units as members: each one the whole layer at its time, turned about its box's centre, cut to its box.
fn unit_set(node: &GraphNode, inputs: &NodeInputs, recipe: &Recipe, cuts: Vec<([f32; 4], RationalTime)>) -> Result<PlacementSetValue, PlacementProgramError> {
    let units = recipe.units.expect("unit cuts come from units");
    let sample_indices = sample_indices(recipe);
    let mut dynamic_start = node.identity().inputs.len();
    let mut copies = Vec::with_capacity(cuts.len());
    for (index, (b, offset)) in cuts.into_iter().enumerate() {
        let sampled = if offset == RationalTime::ZERO {
            None
        } else {
            let start = dynamic_start;
            dynamic_start += sample_indices.len();
            Some(start)
        };
        let world = read_transform_sampled(node, inputs, recipe.world, &sample_indices, sampled)?;
        let parent = recipe.parent_world
            .map(|i| read_transform_sampled(node, inputs, i, &sample_indices, sampled))
            .transpose()?
            .unwrap_or(TransformValue { affine: glam::Affine2::IDENTITY, spatial: glam::Affine3A::IDENTITY });
        let anchor = units.anchor
            .and_then(|original| actual_index(original, &sample_indices, sampled))
            .and_then(|i| inputs.at(i))
            .and_then(|v| v.downcast_ref::<Value>())
            .and_then(|v| match v { Value::Vec2(a) => Some(glam::vec2(a[0] as f32, a[1] as f32)), _ => None })
            .unwrap_or(glam::Vec2::ZERO);
        let shift = glam::vec2((b[0] + b[2]) * 0.5, (b[1] + b[3]) * 0.5) - anchor;
        let s2 = glam::Affine2::from_translation(shift);
        let s3 = glam::Affine3A::from_translation(shift.extend(0.0));
        copies.push(PlacementCopyValue {
            index: index as u32,
            transform: Some(TransformValue {
                affine: parent.affine * s2 * parent.affine.inverse() * world.affine * s2.inverse(),
                spatial: parent.spatial * s3 * parent.spatial.inverse() * world.spatial * s3.inverse(),
            }),
            opacity: 1.0,
            time_offset: offset,
            outline_stretch: [1.0, 1.0],
            cut: Some(b),
        });
    }
    Ok(PlacementSetValue { selected_effect: Some(TEXT_UNITS), copies })
}

/// Copies are members in order: each one's time offset takes the order delay the layer's order rows give it, by the one
/// law children and split units read (`schedule_delay`), with the rows read at the current time.
fn with_schedule(mut outputs: Vec<crate::doc::store::kind::PlacementOutput>, inputs: &NodeInputs, recipe: &Recipe) -> Vec<crate::doc::store::kind::PlacementOutput> {
    let read = |index: Option<usize>| -> f64 {
        match index.and_then(|i| inputs.at(i)).and_then(|v| v.downcast_ref::<Value>()) {
            Some(Value::F64(v)) if v.is_finite() => *v,
            Some(Value::Enum(v)) => *v as f64,
            Some(Value::Bool(v)) => f64::from(u8::from(*v)),
            _ => 0.0,
        }
    };
    let stagger = read(recipe.schedule[0]);
    if stagger <= 1e-9 {
        return outputs;
    }
    let (from, from_end, n) = (read(recipe.schedule[1]).round() as i64, read(recipe.schedule[2]).round() as i64 == 1, outputs.len());
    for (i, output) in outputs.iter_mut().enumerate() {
        let delay = crate::doc::store::layout::schedule_delay(stagger, from, from_end, i, n);
        output.placement.time_offset = crate::picture::resolve::copies::with_delay(output.placement.time_offset, delay);
    }
    outputs
}

fn selected_effect<'a>(recipe: &Recipe, inputs: &'a NodeInputs) -> Option<(usize, PlacementEvaluator, &'a crate::picture::resolved::ResolvedEffect)> {
    (0..recipe.effect_count).find_map(|index| {
        let program = recipe.programs.get(index).copied().flatten()?;
        let effect = inputs.at(index)?.downcast_ref::<EffectValue>()?.0.as_ref()?;
        Some((index, program, effect))
    })
}

fn sample_indices(recipe: &Recipe) -> Vec<usize> {
    let mut indices = vec![recipe.world];
    for index in [recipe.parent_world, recipe.position, recipe.position_x, recipe.position_y, recipe.position_z, recipe.units.and_then(|u| u.anchor)].into_iter().flatten() {
        if !indices.contains(&index) { indices.push(index); }
    }
    indices
}

fn actual_index(original: usize, sample_indices: &[usize], sampled_start: Option<usize>) -> Option<usize> {
    match sampled_start {
        None => Some(original),
        Some(start) => sample_indices.iter().position(|index| *index == original).map(|offset| start + offset),
    }
}

fn read_transform_sampled(node: &GraphNode, inputs: &NodeInputs, original: usize, sample_indices: &[usize], sampled_start: Option<usize>) -> Result<TransformValue, PlacementProgramError> {
    let index = actual_index(original, sample_indices, sampled_start).ok_or(PlacementProgramError::InvalidInput(node.identity().kind))?;
    inputs.at(index).and_then(|value| value.downcast_ref::<TransformValue>()).copied()
        .ok_or(PlacementProgramError::InvalidInput(node.identity().kind))
}

fn read_number_sampled(inputs: &NodeInputs, original: Option<usize>, sample_indices: &[usize], sampled_start: Option<usize>) -> Option<f64> {
    let original = original?;
    let index = actual_index(original, sample_indices, sampled_start)?;
    inputs.at(index).and_then(|value| value.downcast_ref::<Value>()).and_then(|value| match value {
        Value::F64(value) if value.is_finite() => Some(*value),
        _ => None,
    })
}

fn read_position(node: &GraphNode, inputs: &NodeInputs, recipe: &Recipe, sampled_start: Option<usize>) -> Result<[f32; 2], PlacementProgramError> {
    let indices = sample_indices(recipe);
    if let Some(original) = recipe.position {
        if let Some(index) = actual_index(original, &indices, sampled_start) {
            if let Some(Value::Vec2(value)) = inputs.at(index).and_then(|value| value.downcast_ref::<Value>()) {
                return Ok([value[0] as f32, value[1] as f32]);
            }
        }
    }
    let x = read_number_sampled(inputs, recipe.position_x, &indices, sampled_start).unwrap_or(0.0) as f32;
    let y = read_number_sampled(inputs, recipe.position_y, &indices, sampled_start).unwrap_or(0.0) as f32;
    if x.is_finite() && y.is_finite() { Ok([x, y]) } else { Err(PlacementProgramError::InvalidInput(node.identity().kind)) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, EffectId, EffectInstance, Fps, LayerMeta, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> {
            self.0.execute(node, &inputs, &context)
        }
    }


    #[test]
    fn same_time_repeater_expands_scene_contributions_in_order() {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition {
            width: 640, height: 360, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4],
        })).unwrap();
        let layer = LayerId(2);
        let effect = EffectId(2);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: effect, plugin_id: crate::extensions::placement::REPEAT.into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "count").unwrap(), value: Value::F64(3.0) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
        ]).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>()).unwrap();

        assert_eq!(scene.layers.len(), 3);
        assert_eq!(
            scene.layers.iter().map(|layer| layer.transform.affine.translation.x.round()).collect::<Vec<_>>(),
            [0.0, 10.0, 20.0],
        );
    }

    #[test]
    fn effects_below_repeater_receive_one_plate_not_each_copy() {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition {
            width: 640, height: 360, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4],
        })).unwrap();
        let layer = LayerId(3);
        let repeat = EffectId(3);
        let after = EffectId(4);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetEffects { layer, effects: vec![
                EffectInstance { id: repeat, plugin_id: crate::extensions::placement::REPEAT.into() },
                EffectInstance { id: after, plugin_id: "test.after".into() },
            ] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "count").unwrap(), value: Value::F64(2.0) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(repeat, "position_each").unwrap(), value: Value::Vec2([20.0, 0.0]) },
        ]).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.scene().scene;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(2), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let scene = frame.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::SceneValue>()).unwrap();

        assert_eq!(scene.layers.len(), 1);
        assert_eq!(scene.layers[0].effects.iter().map(|effect| effect.plugin_id.as_str()).collect::<Vec<_>>(), ["test.after"]);
        let crate::frame_graph::SceneContentValue::Plate(plate) = &scene.layers[0].content else { panic!("copies must be one plate before downstream effects"); };
        assert_eq!(plate.members.len(), 2);
    }

    #[test]
    fn repeater_outputs_current_time_copies_and_keeps_delayed_copies_explicit() {
        let mut doc = Document::new().with_programs(crate::extensions::bundled());
        doc.apply(Intent::SetComposition(Composition {
            width: 640, height: 360, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4],
        })).unwrap();
        let layer = LayerId(1);
        let effect = EffectId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Shape, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetEffects { layer, effects: vec![EffectInstance { id: effect, plugin_id: crate::extensions::placement::REPEAT.into() }] },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "count").unwrap(), value: Value::F64(3.0) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "position_each").unwrap(), value: Value::Vec2([10.0, 0.0]) },
            Intent::SetConstant { layer, property: PropertyId::effect_param(effect, "delay_each").unwrap(), value: Value::F64(0.5) },
        ]).unwrap();

        let program = SceneProgram::compile(&doc.view()).unwrap();
        let binding = program.placements().binding(layer).unwrap();
        let topology = GraphTopology::try_new(program.nodes(), vec![binding.node]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let set = frame.value(binding.node).and_then(|value| value.downcast_ref::<PlacementSetValue>()).unwrap();

        assert_eq!(set.selected_effect, Some(0));
        assert_eq!(set.copies.len(), 3);
        assert!(set.copies[0].transform.is_some());
        assert!(set.copies[1].transform.is_none());
        assert_eq!(set.copies[1].time_offset, RationalTime::try_new(1, 2).unwrap());
    }
}
