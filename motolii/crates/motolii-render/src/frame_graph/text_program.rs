use std::collections::BTreeMap;

use crate::doc::store::{LayerId, LayerSource, StoreError, StoreView, TextDocument};
use crate::doc::vector::text::ShapedText;
use crate::picture::shapes_ops::Canvas;

use super::{ContentProgram, DynamicInput, EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

#[derive(Clone, Debug, PartialEq)]
pub struct TextShapeValue { pub document: TextDocument, pub shaped: ShapedText }

impl TextShapeValue {
    pub fn shapes(&self) -> Vec<crate::doc::store::ShapeNode> {
        use crate::doc::vector::{Brush, Fill, FillRule, PathSource, Rgb, Shape};
        let mut batches: Vec<(usize, Vec<crate::doc::vector::Contour>)> = Vec::new();
        for (contour, style) in self.shaped.contours.iter().cloned().zip(&self.shaped.contour_styles) {
            if let Some((_, contours)) = batches.last_mut().filter(|(index, _)| index == style) { contours.push(contour); }
            else { batches.push((*style, vec![contour])); }
        }
        batches.into_iter().filter_map(|(index, contours)| {
            let style = self.document.styles.get(index)?;
            Some(crate::doc::store::ShapeNode::Leaf(Shape { source: PathSource::Bezier(contours), ops: Vec::new(), fill: Some(Fill { brush: Brush::Solid(Rgb { r: style.fill[0], g: style.fill[1], b: style.fill[2] }), rule: FillRule::NonZero, opacity: style.fill[3], hidden: false }), stroke: None }))
        }).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextBinding { pub layer: LayerId, pub shape: NodeKey }

#[derive(Clone, Copy, Default)]
struct StyleInputs {
    size: Option<usize>,
    line_height: Option<usize>,
    tracking: Option<usize>,
    fill: Option<usize>,
}

#[derive(Clone)]
struct Recipe {
    flow_input: usize,
    flow_index: usize,
    canvas: Canvas,
    justify: Option<usize>,
    autospace: Option<usize>,
    spacing_trim: Option<usize>,
    hanging: Option<usize>,
    styles: Vec<StyleInputs>,
}

#[derive(Debug)]
pub enum TextProgramError { Store(StoreError), InvalidInput(NodeKind), Shape(String) }
impl std::fmt::Display for TextProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for TextProgramError {}
impl From<StoreError> for TextProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

#[derive(Clone, Copy)]
struct MotionRecipe { base: usize, flow: usize, flow_index: usize }

pub struct TextProgram {
    nodes: BTreeMap<NodeKey, GraphNode>, recipes: BTreeMap<NodeKey, Recipe>, motion: BTreeMap<NodeKey, MotionRecipe>, bindings: BTreeMap<LayerId, TextBinding>,
}

impl TextProgram {
    pub fn compile(view: &StoreView<'_>, content: &ContentProgram, flow: &FlowProgram, properties: &PropertyProgram) -> Result<Self, TextProgramError> {
        let comp = view.composition()?.map(|comp| [comp.width, comp.height]).unwrap_or([1920, 1080]);
        let canvas = Canvas { width: comp[0], height: comp[1], origin_x: 0, origin_y: 0 };
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut motion = BTreeMap::new(); let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            if !view.meta(layer)?.is_some_and(|meta| meta.source == LayerSource::Text) { continue; }
            let Some(text) = content.binding(layer).and_then(|binding| binding.content) else { continue };
            let Some(flow_binding) = flow.binding(layer) else { continue };
            let Some(document) = view.text_document(layer)? else { continue };
            let mut inputs = vec![text, flow.key()];
            let mut input = |property: crate::doc::store::PropertyId| {
                properties.node_for(layer, &property).map(|key| {
                    let at = inputs.len();
                    inputs.push(key);
                    at
                })
            };
            let justify = input(crate::doc::store::PropertyId::text_justify());
            let autospace = input(crate::doc::store::PropertyId::text_autospace());
            let spacing_trim = input(crate::doc::store::PropertyId::text_spacing_trim());
            let hanging = input(crate::doc::store::PropertyId::hanging_punctuation());
            let styles = document.styles.iter().map(|style| StyleInputs {
                size: input(crate::doc::store::PropertyId::text_style_size(style.id)),
                line_height: input(crate::doc::store::PropertyId::text_style_line_height(style.id)),
                tracking: input(crate::doc::store::PropertyId::text_style_tracking(style.id)),
                fill: input(crate::doc::store::PropertyId::text_style_fill_color(style.id)),
            }).collect::<Vec<_>>();
            let mut identity = NodeIdentity::new(NodeKind::TextShape, inputs);
            identity.parameters = (flow_binding.index as u64).to_be_bytes().to_vec();
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let base = node.key();
            recipes.entry(base).or_insert(Recipe {
                flow_input: 1,
                flow_index: flow_binding.index,
                canvas,
                justify,
                autospace,
                spacing_trim,
                hanging,
                styles,
            });
            nodes.entry(base).or_insert(node);

            let mut motion_identity = NodeIdentity::new(NodeKind::TextShape, vec![base, flow.key()]);
            motion_identity.parameters = b"glyph-transition".to_vec();
            motion_identity.parameters.extend_from_slice(&(flow_binding.index as u64).to_be_bytes());
            motion_identity.time_dependency = TimeDependency::Exact;
            let motion_node = GraphNode::new(motion_identity);
            let shape = motion_node.key();
            motion.entry(shape).or_insert(MotionRecipe { base: 0, flow: 1, flow_index: flow_binding.index });
            nodes.entry(shape).or_insert(motion_node);
            bindings.insert(layer, TextBinding { layer, shape });
        }
        Ok(Self { nodes, recipes, motion, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = TextBinding> + '_ { self.bindings.values().copied() }
    pub fn binding(&self, layer: LayerId) -> Option<TextBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<Vec<DynamicInput>, TextProgramError>> {
        let recipe = self.motion.get(&node.key())?;
        Some((|| {
            let flow = inputs.at(recipe.flow).and_then(|value| value.downcast_ref::<FlowFrameValue>()).ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            let samples = flow.transition_samples.get(recipe.flow_index).ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            Ok(samples.iter().map(|(time, _)| DynamicInput { node: node.identity().inputs[recipe.base], time: *time }).collect())
        })())
    }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, TextProgramError>> {
        if let Some(recipe) = self.motion.get(&node.key()) {
            return Some(apply_glyph_transition(node, inputs, *recipe));
        }
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let mut document = inputs.at(0).and_then(|value| value.downcast_ref::<TextDocument>()).cloned().ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            let value = |index: Option<usize>| index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<crate::doc::eval::Value>());
            if let Some(value) = value(recipe.justify) {
                match value {
                    crate::doc::eval::Value::Enum(value) => {
                        document.justify = crate::doc::store::TextJustify::from_enum_value(*value).ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
                    }
                    _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                }
            }
            if let Some(value) = value(recipe.autospace) {
                match value {
                    crate::doc::eval::Value::Enum(value) => {
                        document.alignment.autospace = crate::doc::store::TextAutospace::from_enum_value(*value).unwrap_or_default();
                    }
                    _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                }
            }
            if let Some(value) = value(recipe.spacing_trim) {
                match value {
                    crate::doc::eval::Value::Enum(value) => {
                        document.alignment.spacing_trim = crate::doc::store::TextSpacingTrim::from_enum_value(*value).unwrap_or_default();
                    }
                    _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                }
            }
            if let Some(value) = value(recipe.hanging) {
                match value {
                    crate::doc::eval::Value::Enum(value) => {
                        document.alignment.hanging = crate::doc::store::HangingPunctuation::from_enum_value(*value).unwrap_or_default();
                    }
                    _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                }
            }
            for (style, bindings) in document.styles.iter_mut().zip(&recipe.styles) {
                if let Some(value) = value(bindings.size) {
                    match value {
                        crate::doc::eval::Value::F64(value) => style.size = *value as f32,
                        _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                    }
                }
                if let Some(value) = value(bindings.line_height) {
                    match value {
                        crate::doc::eval::Value::F64(value) => style.line_height = Some(*value as f32),
                        _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                    }
                }
                if let Some(value) = value(bindings.tracking) {
                    match value {
                        crate::doc::eval::Value::F64(value) => style.tracking = *value as f32,
                        _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                    }
                }
                if let Some(value) = value(bindings.fill) {
                    match value {
                        crate::doc::eval::Value::Color(value) => style.fill = *value,
                        _ => return Err(TextProgramError::InvalidInput(node.identity().kind)),
                    }
                }
            }
            let flow = inputs.at(recipe.flow_input).and_then(|value| value.downcast_ref::<FlowFrameValue>()).ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            if let Some(wrap) = flow.slots.get(recipe.flow_index).copied().flatten().and_then(|slot| slot.wrap) { document.wrap_size = Some([wrap.max(1.0), recipe.canvas.height as f32]); }
            let shaped = crate::picture::text_frame::shape_document(&document, context.time, &recipe.canvas).map_err(|error| TextProgramError::Shape(error.to_string()))?.unwrap_or_default();
            Ok(NodeValue::new(TextShapeValue { document, shaped }))
        })())
    }
}


fn apply_glyph_transition(node: &GraphNode, inputs: &NodeInputs, recipe: MotionRecipe) -> Result<NodeValue, TextProgramError> {
    let mut current = inputs.at(recipe.base)
        .and_then(|value| value.downcast_ref::<TextShapeValue>())
        .cloned()
        .ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
    let flow = inputs.at(recipe.flow)
        .and_then(|value| value.downcast_ref::<FlowFrameValue>())
        .ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
    let samples = flow.transition_samples.get(recipe.flow_index)
        .ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
    if samples.is_empty() {
        return Ok(NodeValue::new(current));
    }

    let positions = |shaped: &ShapedText| -> Vec<(usize, [f32; 2])> {
        shaped.lines.iter().flat_map(|line| {
            line.glyph_bytes.iter().zip(&line.glyph_xs).map(move |(byte, x)| (*byte, [*x, line.baseline_y]))
        }).collect()
    };
    let now = positions(&current.shaped);
    if now.is_empty() {
        return Ok(NodeValue::new(current));
    }

    let mut sum = vec![[0.0f32; 2]; now.len()];
    let mut weights = vec![0.0f32; now.len()];
    let dynamic_start = node.identity().inputs.len();
    for (sample_index, (_, weight)) in samples.iter().enumerate() {
        let past = inputs.at(dynamic_start + sample_index)
            .and_then(|value| value.downcast_ref::<TextShapeValue>())
            .ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
        let past: std::collections::HashMap<usize, [f32; 2]> = positions(&past.shaped).into_iter().collect();
        for (glyph, (byte, here)) in now.iter().enumerate() {
            if let Some(previous) = past.get(byte) {
                sum[glyph][0] += (previous[0] - here[0]) * *weight;
                sum[glyph][1] += (previous[1] - here[1]) * *weight;
                weights[glyph] += *weight;
            }
        }
    }
    let offsets: Vec<[f32; 2]> = sum.iter().zip(&weights)
        .map(|(sum, weight)| if *weight > 1e-6 { [sum[0] / weight, sum[1] / weight] } else { [0.0, 0.0] })
        .collect();

    for (contour, glyph) in current.shaped.contours.iter_mut().zip(&current.shaped.contour_glyphs) {
        let Some(offset) = offsets.get(*glyph) else { continue };
        for vertex in &mut contour.vertices {
            vertex.point.x += f64::from(offset[0]);
            vertex.point.y += f64::from(offset[1]);
        }
    }
    Ok(NodeValue::new(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::core::RationalTime;
    use crate::doc::store::{ContentKeyframe, ContentTrack, FontRef, LayerMeta, LayerTiming, TextAlignmentOptions, TextDocumentStyle, TextJustify, TextStyleId};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn text_shape_is_evaluated_once_from_content_and_flow_values() {
        let mut content = ContentTrack::new();
        content.insert(ContentKeyframe { t: RationalTime::ZERO, content: "M".into() });
        let document = TextDocument { content, justify: TextJustify::Center, wrap_size: None, styles: vec![TextDocumentStyle { id: TextStyleId(0), font: FontRef { path: String::new(), fingerprint: None, family: "Helvetica Neue".into(), style: String::new() }, size: 96.0, fill: [1.0; 4], line_height: None, tracking: 0.0, axes: vec![], features: vec![] }], slot_id: None, ranges: vec![], alignment: TextAlignmentOptions::default(), runs: vec![] };
        let mut doc = Document::new();
        let layer = LayerId(1);
        doc.apply_all([
            Intent::AddLayer(layer),
            Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Text, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetTextDocument { layer, document: document.clone() },
        ]).unwrap();
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let root = program.text().binding(layer).unwrap().shape;
        let topology = GraphTopology::try_new(program.nodes(), vec![root]).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let shaped = frame.value(root).and_then(|value| value.downcast_ref::<TextShapeValue>()).unwrap();
        let canvas = Canvas { width: 1920, height: 1080, origin_x: 0, origin_y: 0 };
        let oracle = crate::picture::text_frame::shape_document(&document, RationalTime::ZERO, &canvas).unwrap().unwrap();
        assert_eq!(shaped.shaped, oracle);
    }
}
