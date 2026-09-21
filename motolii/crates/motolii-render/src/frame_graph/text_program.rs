use std::collections::BTreeMap;

use crate::doc::store::{LayerId, LayerSource, StoreError, StoreView, TextDocument};
use crate::doc::vector::text::ShapedText;
use crate::picture::shapes_ops::Canvas;

use super::{ContentProgram, EvaluationContext, FlowFrameValue, FlowProgram, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, TimeDependency};

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

#[derive(Clone)]
struct Recipe { flow_input: usize, flow_index: usize, canvas: Canvas }

#[derive(Debug)]
pub enum TextProgramError { Store(StoreError), InvalidInput(NodeKind), Shape(String) }
impl std::fmt::Display for TextProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for TextProgramError {}
impl From<StoreError> for TextProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }

pub struct TextProgram {
    nodes: BTreeMap<NodeKey, GraphNode>, recipes: BTreeMap<NodeKey, Recipe>, bindings: BTreeMap<LayerId, TextBinding>,
}

impl TextProgram {
    pub fn compile(view: &StoreView<'_>, content: &ContentProgram, flow: &FlowProgram) -> Result<Self, TextProgramError> {
        let comp = view.composition()?.map(|comp| [comp.width, comp.height]).unwrap_or([1920, 1080]);
        let canvas = Canvas { width: comp[0], height: comp[1], origin_x: 0, origin_y: 0 };
        let mut nodes = BTreeMap::new(); let mut recipes = BTreeMap::new(); let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            if !view.meta(layer)?.is_some_and(|meta| meta.source == LayerSource::Text) { continue; }
            let Some(text) = content.binding(layer).and_then(|binding| binding.content) else { continue };
            let Some(flow_binding) = flow.binding(layer) else { continue };
            let mut identity = NodeIdentity::new(NodeKind::TextShape, vec![text, flow.key()]);
            identity.parameters = (flow_binding.index as u64).to_be_bytes().to_vec();
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            recipes.entry(node.key()).or_insert(Recipe { flow_input: 1, flow_index: flow_binding.index, canvas });
            bindings.insert(layer, TextBinding { layer, shape: node.key() });
            nodes.entry(node.key()).or_insert(node);
        }
        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = TextBinding> + '_ { self.bindings.values().copied() }
    pub fn binding(&self, layer: LayerId) -> Option<TextBinding> { self.bindings.get(&layer).copied() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, context: &EvaluationContext) -> Option<Result<NodeValue, TextProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some((|| {
            let mut document = inputs.at(0).and_then(|value| value.downcast_ref::<TextDocument>()).cloned().ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            let flow = inputs.at(recipe.flow_input).and_then(|value| value.downcast_ref::<FlowFrameValue>()).ok_or(TextProgramError::InvalidInput(node.identity().kind))?;
            if let Some(wrap) = flow.slots.get(recipe.flow_index).copied().flatten().and_then(|slot| slot.wrap) { document.wrap_size = Some([wrap.max(1.0), recipe.canvas.height as f32]); }
            let shaped = crate::picture::text_frame::shape_document(&document, context.time, &recipe.canvas).map_err(|error| TextProgramError::Shape(error.to_string()))?.unwrap_or_default();
            Ok(NodeValue::new(TextShapeValue { document, shaped }))
        })())
    }
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
