use std::collections::{BTreeMap, HashMap};

use taffy::prelude::*;

use crate::doc::eval::Value;
use crate::doc::store::{layout, LayerId, LayerSource, PropertyId, ShapeNode, StoreError, StoreView, TextDocument};
use crate::picture::shapes_ops::Canvas;

use super::{ContentProgram, EvaluationContext, GraphNode, NodeIdentity, NodeInputs, NodeKey, NodeKind, NodeValue, PropertyProgram, TimeDependency};

const ROWS: [&str; 22] = [layout::DISPLAY, layout::FLEX_DIRECTION, layout::FLEX_WRAP, layout::JUSTIFY_CONTENT, layout::ALIGN_ITEMS, layout::GAP, layout::PADDING, layout::GRID_COLUMNS, layout::GRID_ROWS, layout::HORIZONTAL_SIZING, layout::VERTICAL_SIZING, layout::WIDTH, layout::HEIGHT, layout::MARGIN, layout::FLEX_SHRINK, layout::ALIGN_SELF, layout::COLUMN_START, layout::COLUMN_SPAN, layout::ROW_START, layout::ROW_SPAN, layout::OBJECT_FIT, crate::doc::store::property::SCALE];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FlowSlot { pub position: [f32; 2], pub scale: [f32; 2], pub stretch: [f32; 2], pub wrap: Option<f32> }

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlowFrameValue { pub slots: Vec<Option<FlowSlot>>, pub sizes: Vec<Option<[f32; 2]>> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowBinding { pub layer: LayerId, pub index: usize }

#[derive(Clone)]
struct LayerPlan {
    parent: Option<usize>, order: i16, source: LayerSource,
    props: [Option<usize>; 22], content: Option<usize>,
}

#[derive(Clone)]
struct Recipe { layers: Vec<LayerPlan>, comp: [u32; 2] }

#[derive(Debug)]
pub enum FlowProgramError { Store(StoreError), Taffy(taffy::TaffyError), InvalidInput(NodeKind) }
impl std::fmt::Display for FlowProgramError { fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") } }
impl std::error::Error for FlowProgramError {}
impl From<StoreError> for FlowProgramError { fn from(value: StoreError) -> Self { Self::Store(value) } }
impl From<taffy::TaffyError> for FlowProgramError { fn from(value: taffy::TaffyError) -> Self { Self::Taffy(value) } }

pub struct FlowProgram {
    node: GraphNode,
    recipe: Recipe,
    bindings: BTreeMap<LayerId, FlowBinding>,
}

impl FlowProgram {
    pub fn compile(view: &StoreView<'_>, properties: &PropertyProgram, content: &ContentProgram) -> Result<Self, FlowProgramError> {
        let ids = view.layers();
        let index: BTreeMap<_, _> = ids.iter().enumerate().map(|(index, layer)| (*layer, index)).collect();
        let mut inputs = Vec::new();
        let mut input_index = BTreeMap::new();
        let mut input = |key: NodeKey| *input_index.entry(key).or_insert_with(|| { let at = inputs.len(); inputs.push(key); at });
        let mut layers = Vec::with_capacity(ids.len());
        let mut parameters = Vec::new();
        for layer in ids.iter().copied() {
            let meta = view.meta(layer)?.unwrap_or_else(|| crate::doc::store::LayerMeta { source: LayerSource::Null, order: 0, timing: crate::doc::store::LayerTiming::place(0, None, 0) });
            let parent = view.attrs(layer)?.unwrap_or_default().parent.and_then(|parent| index.get(&parent).copied());
            let mut props = [None; 22];
            for (row, name) in ROWS.iter().enumerate() {
                props[row] = properties.node_for(layer, &PropertyId::new(name).expect("known layout property")).map(&mut input);
            }
            let content = content.binding(layer).and_then(|binding| binding.content.or(binding.extent)).map(&mut input);
            parameters.extend_from_slice(&(parent.map(|value| value as u32).unwrap_or(u32::MAX)).to_be_bytes());
            parameters.extend_from_slice(&meta.order.to_be_bytes());
            parameters.push(source_tag(&meta.source));
            layers.push(LayerPlan { parent, order: meta.order, source: meta.source, props, content });
        }
        let comp = view.composition()?.map(|comp| [comp.width, comp.height]).unwrap_or([1920, 1080]);
        parameters.extend_from_slice(&comp[0].to_be_bytes()); parameters.extend_from_slice(&comp[1].to_be_bytes());
        let mut identity = NodeIdentity::new(NodeKind::Layout, inputs);
        identity.parameters = parameters;
        identity.time_dependency = TimeDependency::Exact;
        let node = GraphNode::new(identity);
        let bindings = ids.into_iter().enumerate().map(|(index, layer)| (layer, FlowBinding { layer, index })).collect();
        Ok(Self { node, recipe: Recipe { layers, comp }, bindings })
    }

    pub fn node(&self) -> GraphNode { self.node.clone() }
    pub fn key(&self) -> NodeKey { self.node.key() }
    pub fn bindings(&self) -> impl ExactSizeIterator<Item = FlowBinding> + '_ { self.bindings.values().copied() }
    pub fn binding(&self, layer: LayerId) -> Option<FlowBinding> { self.bindings.get(&layer).copied() }

    pub fn execute(&self, node: &GraphNode, inputs: &NodeInputs, _context: &EvaluationContext) -> Option<Result<NodeValue, FlowProgramError>> {
        (node.key() == self.node.key()).then(|| evaluate(&self.recipe, inputs).map(NodeValue::new))
    }
}

fn source_tag(source: &LayerSource) -> u8 { match source { LayerSource::File { .. } => 0, LayerSource::Null => 1, LayerSource::Camera => 2, LayerSource::Stage => 3, LayerSource::Shape => 4, LayerSource::Text => 5, LayerSource::Group => 6, LayerSource::Particles => 7 } }

fn value<'a>(plan: &LayerPlan, row: usize, inputs: &'a NodeInputs) -> Option<&'a Value> { plan.props[row].and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref()) }
fn number(plan: &LayerPlan, row: usize, inputs: &NodeInputs, default: f32) -> f32 { match value(plan, row, inputs) { Some(Value::F64(value)) => *value as f32, _ => default } }
fn choice(plan: &LayerPlan, row: usize, inputs: &NodeInputs, default: i64) -> i64 { match value(plan, row, inputs) { Some(Value::Enum(value)) => *value, Some(Value::F64(value)) => value.round() as i64, _ => default } }
fn pair(plan: &LayerPlan, row: usize, inputs: &NodeInputs, default: [f32; 2]) -> [f32; 2] { match value(plan, row, inputs) { Some(Value::Vec2(value)) => [value[0] as f32, value[1] as f32], _ => default } }

#[derive(Clone, Copy, PartialEq, Eq)] enum Sizing { Hug, Fill, Fixed }
fn sizing(plan: &LayerPlan, inputs: &NodeInputs) -> [Sizing; 2] {
    let map = |value| match value { 1 => Sizing::Fill, 2 => Sizing::Fixed, _ => Sizing::Hug };
    [map(choice(plan, 9, inputs, 0)), map(choice(plan, 10, inputs, 0))]
}

fn natural_bounds(plan: &LayerPlan, inputs: &NodeInputs, canvas: &Canvas) -> [f32; 4] {
    let Some(index) = plan.content else { return [0.0; 4] };
    let Some(content) = inputs.at(index) else { return [0.0; 4] };
    if let Some(document) = content.downcast_ref::<TextDocument>() {
        return crate::picture::text_frame::shape_document(document, crate::doc::core::RationalTime::ZERO, canvas).ok().flatten()
            .and_then(|shaped| crate::picture::text_frame::line_box(document, &shaped, canvas))
            .unwrap_or([0.0; 4]);
    }
    if let Some(shapes) = content.downcast_ref::<Vec<ShapeNode>>() {
        return crate::picture::shapes_ops::content_bounds(shapes).ok().flatten().map(|bounds| bounds.map(|value| value as f32)).unwrap_or([0.0; 4]);
    }
    [0.0; 4]
}

fn natural(plan: &LayerPlan, inputs: &NodeInputs, canvas: &Canvas) -> [f32; 2] {
    let bounds = natural_bounds(plan, inputs, canvas);
    let scale = pair(plan, 21, inputs, [1.0, 1.0]);
    [(bounds[2] - bounds[0]) * scale[0].abs(), (bounds[3] - bounds[1]) * scale[1].abs()]
}

fn item_style(plan: &LayerPlan, parent: Option<&LayerPlan>, inputs: &NodeInputs, natural: [f32; 2], mut style: Style) -> Style {
    let sizing = sizing(plan, inputs);
    let fixed = [number(plan, 11, inputs, 100.0), number(plan, 12, inputs, 100.0)];
    let dim = |axis| match sizing[axis] { Sizing::Hug => Dimension::length(natural[axis]), Sizing::Fixed => Dimension::length(fixed[axis]), Sizing::Fill => Dimension::auto() };
    style.size = Size { width: dim(0), height: dim(1) };
    style.min_size = Size { width: Dimension::length(0.0), height: Dimension::length(0.0) };
    style.flex_shrink = number(plan, 14, inputs, 0.0).max(0.0);
    let margin = number(plan, 13, inputs, 0.0).max(0.0);
    let margin = LengthPercentageAuto::length(margin);
    style.margin = Rect { left: margin, right: margin, top: margin, bottom: margin };
    let row = parent.is_none_or(|parent| matches!(choice(parent, 1, inputs, 0), 0 | 2));
    let main = if row { 0 } else { 1 };
    if sizing[main] == Sizing::Fill { style.flex_grow = 1.0; style.flex_basis = Dimension::length(0.0); }
    if sizing[1 - main] == Sizing::Fill { style.align_self = Some(AlignSelf::STRETCH); }
    style.align_self = match choice(plan, 15, inputs, 0) { 1 => Some(AlignSelf::STRETCH), 2 => Some(AlignSelf::FLEX_START), 3 => Some(AlignSelf::FLEX_END), 4 => Some(AlignSelf::CENTER), _ => style.align_self };
    let line = |start: f32, span: f32| Line { start: if start >= 1.0 { GridPlacement::from_line_index(start.round() as i16) } else { GridPlacement::Auto }, end: GridPlacement::from_span(span.round().max(1.0) as u16) };
    style.grid_column = line(number(plan, 16, inputs, 0.0), number(plan, 17, inputs, 1.0));
    style.grid_row = line(number(plan, 18, inputs, 0.0), number(plan, 19, inputs, 1.0));
    style
}

fn group_style(plan: &LayerPlan, parent: Option<&LayerPlan>, inputs: &NodeInputs, root: bool) -> Style {
    let display = choice(plan, 0, inputs, 0);
    let padding = pair(plan, 6, inputs, [0.0, 0.0]);
    let gap = number(plan, 5, inputs, 0.0);
    let mut style = Style { display: if display == 2 { Display::Grid } else { Display::Flex }, flex_direction: match choice(plan, 1, inputs, 0) { 1 => FlexDirection::Column, 2 => FlexDirection::RowReverse, 3 => FlexDirection::ColumnReverse, _ => FlexDirection::Row }, flex_wrap: match choice(plan, 2, inputs, 0) { 1 => FlexWrap::Wrap, 2 => FlexWrap::WrapReverse, _ => FlexWrap::NoWrap }, justify_content: Some(match choice(plan, 3, inputs, 0) { 1 => JustifyContent::FLEX_END, 2 => JustifyContent::CENTER, 3 => JustifyContent::SPACE_BETWEEN, 4 => JustifyContent::SPACE_AROUND, 5 => JustifyContent::SPACE_EVENLY, _ => JustifyContent::FLEX_START }), align_items: Some(match choice(plan, 4, inputs, 0) { 1 => AlignItems::FLEX_START, 2 => AlignItems::FLEX_END, 3 => AlignItems::CENTER, _ => AlignItems::STRETCH }), gap: Size { width: LengthPercentage::length(gap), height: LengthPercentage::length(gap) }, padding: Rect { left: LengthPercentage::length(padding[0]), right: LengthPercentage::length(padding[0]), top: LengthPercentage::length(padding[1]), bottom: LengthPercentage::length(padding[1]) }, ..Style::default() };
    if display == 2 {
        let columns = number(plan, 7, inputs, 2.0).round().clamp(1.0, 64.0) as usize;
        let rows = number(plan, 8, inputs, 0.0).round().clamp(0.0, 64.0) as usize;
        style.grid_template_columns = vec![fr(1.0); columns];
        if rows > 0 { style.grid_template_rows = vec![fr(1.0); rows]; }
    }
    if root {
        let size = sizing(plan, inputs);
        style.size = Size { width: if size[0] == Sizing::Fixed { Dimension::length(number(plan, 11, inputs, 0.0)) } else { Dimension::auto() }, height: if size[1] == Sizing::Fixed { Dimension::length(number(plan, 12, inputs, 0.0)) } else { Dimension::auto() } };
    } else { style = item_style(plan, parent, inputs, [0.0, 0.0], style); }
    style
}

fn evaluate(recipe: &Recipe, inputs: &NodeInputs) -> Result<FlowFrameValue, FlowProgramError> {
    let mut children: HashMap<usize, Vec<usize>> = HashMap::new();
    for (index, layer) in recipe.layers.iter().enumerate() { if let Some(parent) = layer.parent { children.entry(parent).or_default().push(index); } }
    for list in children.values_mut() { list.sort_by_key(|index| recipe.layers[*index].order); }
    let displayed: Vec<_> = recipe.layers.iter().enumerate().filter(|(_, layer)| layer.source == LayerSource::Group && choice(layer, 0, inputs, 0) != 0).map(|(index, _)| index).collect();
    let canvas = Canvas { width: recipe.comp[0], height: recipe.comp[1], origin_x: 0, origin_y: 0 };
    let mut out = FlowFrameValue { slots: vec![None; recipe.layers.len()], sizes: vec![None; recipe.layers.len()] };
    for root in displayed.iter().copied().filter(|index| recipe.layers[*index].parent.is_none_or(|parent| !displayed.contains(&parent))) {
        let mut tree: TaffyTree<()> = TaffyTree::new(); tree.disable_rounding();
        let mut nodes = BTreeMap::new();
        fn build(index: usize, root: usize, recipe: &Recipe, inputs: &NodeInputs, children: &HashMap<usize, Vec<usize>>, displayed: &[usize], canvas: &Canvas, tree: &mut TaffyTree<()>, nodes: &mut BTreeMap<usize, NodeId>) -> Result<NodeId, FlowProgramError> {
            let layer = &recipe.layers[index];
            let node = if displayed.contains(&index) {
                let kids: Result<Vec<_>, _> = children.get(&index).map(Vec::as_slice).unwrap_or(&[]).iter().map(|child| build(*child, root, recipe, inputs, children, displayed, canvas, tree, nodes)).collect();
                tree.new_with_children(group_style(layer, layer.parent.map(|parent| &recipe.layers[parent]), inputs, index == root), &kids?)?
            } else {
                let style = item_style(layer, layer.parent.map(|parent| &recipe.layers[parent]), inputs, natural(layer, inputs, canvas), Style::default());
                tree.new_leaf(style)?
            };
            nodes.insert(index, node); Ok(node)
        }
        let root_node = build(root, root, recipe, inputs, &children, &displayed, &canvas, &mut tree, &mut nodes)?;
        let root_plan = &recipe.layers[root];
        let root_sizing = sizing(root_plan, inputs);
        let available = Size { width: if root_sizing[0] == Sizing::Fixed { AvailableSpace::Definite(number(root_plan, 11, inputs, 0.0)) } else { AvailableSpace::MaxContent }, height: if root_sizing[1] == Sizing::Fixed { AvailableSpace::Definite(number(root_plan, 12, inputs, 0.0)) } else { AvailableSpace::MaxContent } };
        tree.compute_layout(root_node, available)?;
        for (index, node) in nodes {
            let placed = tree.layout(node)?;
            out.sizes[index] = Some([placed.size.width, placed.size.height]);
            if index == root { continue; }
            let plan = &recipe.layers[index];
            let bounds = natural_bounds(plan, inputs, &canvas);
            let authored_scale = pair(plan, 21, inputs, [1.0, 1.0]);
            let natural = [(bounds[2] - bounds[0]) * authored_scale[0].abs(), (bounds[3] - bounds[1]) * authored_scale[1].abs()];
            let size = sizing(plan, inputs);
            let factor = [0, 1].map(|axis| if size[axis] == Sizing::Hug || natural[axis] <= 1e-6 { 1.0 } else { [placed.size.width, placed.size.height][axis] / natural[axis] });
            let cell = [placed.size.width, placed.size.height];
            let shown = [natural[0] * factor[0], natural[1] * factor[1]];
            let lo = [bounds[0] * authored_scale[0] * factor[0], bounds[1] * authored_scale[1] * factor[1]];
            let origin = [placed.location.x, placed.location.y];
            let position = [0, 1].map(|axis| origin[axis] + (cell[axis] - shown[axis]) * 0.5 - lo[axis] + crate::picture::CANVAS_MARGIN);
            out.slots[index] = Some(FlowSlot { position, scale: factor, stretch: if plan.source == LayerSource::Shape { factor } else { [1.0, 1.0] }, wrap: (plan.source == LayerSource::Text && size[0] == Sizing::Fill).then_some(placed.size.width / authored_scale[0].abs().max(1e-3)) });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, LayerAttrsPatch, LayerMeta, LayerTiming};
    use crate::frame_graph::{CompiledGraph, FrameQuality, Generation, GraphRevision, GraphTopology, NodeExecutor, SceneProgram, SceneProgramError};
    use motolii_edit::{Document, Intent};

    struct Executor<'a>(&'a SceneProgram);
    impl NodeExecutor for Executor<'_> {
        type Error = SceneProgramError;
        fn execute(&mut self, node: &GraphNode, inputs: NodeInputs, context: EvaluationContext) -> Result<NodeValue, Self::Error> { self.0.execute(node, &inputs, &context) }
    }

    #[test]
    fn grid_slots_are_evaluated_from_graph_values_without_store_reads() {
        let mut doc = Document::new();
        let root = LayerId(1);
        doc.apply_all([
            Intent::SetComposition(Composition { width: 640, height: 480, fps: Fps::try_new(30, 1).unwrap(), duration_frames: 90, background: [0.0; 4] }),
            Intent::AddLayer(root),
            Intent::SetMeta { layer: root, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::DISPLAY).unwrap(), value: Value::Enum(2) },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::GRID_COLUMNS).unwrap(), value: Value::F64(2.0) },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::HORIZONTAL_SIZING).unwrap(), value: Value::Enum(2) },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::VERTICAL_SIZING).unwrap(), value: Value::Enum(2) },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::WIDTH).unwrap(), value: Value::F64(600.0) },
            Intent::SetConstant { layer: root, property: PropertyId::new(layout::HEIGHT).unwrap(), value: Value::F64(400.0) },
        ]).unwrap();
        for n in 0..4 {
            let layer = LayerId(2 + n);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta { layer, meta: LayerMeta { source: LayerSource::Null, order: n as i16, timing: LayerTiming::place(0, None, 90) } },
                Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(Some(root)), ..Default::default() } },
                Intent::SetConstant { layer, property: PropertyId::new(layout::HORIZONTAL_SIZING).unwrap(), value: Value::Enum(1) },
                Intent::SetConstant { layer, property: PropertyId::new(layout::VERTICAL_SIZING).unwrap(), value: Value::Enum(1) },
            ]).unwrap();
        }
        let program = SceneProgram::compile(&doc.view()).unwrap();
        let key = program.flow().key();
        let child_world = program.transforms().binding(LayerId(2)).unwrap().world;
        let roots = std::iter::once(key).chain((2..=5).map(|id| program.transforms().binding(LayerId(id)).unwrap().world)).collect();
        let topology = GraphTopology::try_new(program.nodes(), roots).unwrap();
        let mut graph = CompiledGraph::with_topology(GraphRevision::new(1), topology);
        let mut executor = Executor(&program);
        let frame = graph.evaluate(&mut executor, crate::doc::core::RationalTime::ZERO, FrameQuality::Export, Generation::new(1)).unwrap();
        let flow = frame.value(key).and_then(|value| value.downcast_ref::<FlowFrameValue>()).unwrap();
        let slots: Vec<_> = (2..=5).map(|id| flow.slots[program.flow().binding(LayerId(id)).unwrap().index].unwrap()).collect();
        let positions: std::collections::BTreeSet<_> = slots.iter().map(|slot| (slot.position[0] as i32, slot.position[1] as i32)).collect();
        assert_eq!(positions.len(), 4);
        assert!(slots.iter().all(|slot| slot.scale[0] > 0.0 && slot.scale[1] > 0.0));
        let world = frame.value(child_world).and_then(|value| value.downcast_ref::<crate::frame_graph::TransformValue>()).unwrap();
        assert_eq!(world.affine.translation.to_array(), slots[0].position);
        let oracle = crate::picture::resolve::resolved_layers(&doc.view(), crate::doc::core::RationalTime::ZERO).unwrap();
        for id in 2..=5 {
            let root = program.transforms().binding(LayerId(id)).unwrap().world;
            let actual = frame.value(root).and_then(|value| value.downcast_ref::<crate::frame_graph::TransformValue>()).unwrap().affine.translation;
            let expected = oracle.iter().find(|layer| layer.id == LayerId(id) && !layer.ghost).unwrap().placement.transform.translation;
            assert!((actual - expected).length() < 1e-3, "layer {id}: graph={actual:?} oracle={expected:?}");
        }
    }
}
