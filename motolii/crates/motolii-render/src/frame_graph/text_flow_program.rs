use std::collections::BTreeMap;

use crate::doc::core::{Fps, LayerTiming};
use crate::doc::eval::Value;
use crate::doc::store::{layout, EffectId, LayerId, LayerSource, PropertyId, ShapeNode, StoreError, StoreView};
use crate::doc::vector::text::ShapedText;
use crate::picture::shapes_ops::Canvas;

use super::{
    AnalysisProgram, BlobAnalysisValue, ContentProgram, DynamicInput, EvaluationContext,
    FlowFrameValue, FlowProgram, GraphNode, MediaExtentValue, NodeIdentity, NodeInputs, NodeKey,
    NodeKind, NodeValue, PropertyProgram, TextProgram, TextShapeValue, TimeDependency,
    TransformProgram, TransformValue,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextFlowBinding {
    pub layer: LayerId,
    pub shape: NodeKey,
}

#[derive(Clone, Debug, PartialEq)]
struct ObstacleValue {
    polygons: Vec<Vec<[f32; 2]>>,
    margin: f32,
}

#[derive(Clone, Copy)]
struct ObstacleInput {
    key: NodeKey,
    static_input: usize,
    flow_index: usize,
}

#[derive(Clone)]
enum Recipe {
    Obstacle {
        layer: LayerId,
        timing: LayerTiming,
        fps: Fps,
        source: LayerSource,
        transform: usize,
        flow: usize,
        flow_index: usize,
        content: Option<usize>,
        mode: usize,
        shape_margin: Option<usize>,
        layout_margin: Option<usize>,
        analysis: Option<usize>,
    },
    Layout {
        base: usize,
        transform: usize,
        flow: usize,
        flow_index: usize,
        canvas: Canvas,
        obstacles: Vec<ObstacleInput>,
    },
    Motion {
        base: usize,
        flow: usize,
        flow_index: usize,
    },
}

#[derive(Debug)]
pub enum TextFlowProgramError {
    Store(StoreError),
    InvalidInput(NodeKind),
    Shape(String),
}

impl std::fmt::Display for TextFlowProgramError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for TextFlowProgramError {}
impl From<StoreError> for TextFlowProgramError {
    fn from(value: StoreError) -> Self { Self::Store(value) }
}

pub struct TextFlowProgram {
    nodes: BTreeMap<NodeKey, GraphNode>,
    recipes: BTreeMap<NodeKey, Recipe>,
    bindings: BTreeMap<LayerId, TextFlowBinding>,
}

impl TextFlowProgram {
    #[allow(clippy::too_many_arguments)]
    pub fn compile(
        view: &StoreView<'_>,
        properties: &PropertyProgram,
        content: &ContentProgram,
        flow: &FlowProgram,
        transforms: &TransformProgram,
        text: &TextProgram,
        analysis: &AnalysisProgram,
    ) -> Result<Self, TextFlowProgramError> {
        let composition = view.composition()?;
        let fps = composition
            .as_ref()
            .map(|composition| composition.fps)
            .unwrap_or(Fps::try_new(30, 1).expect("30fps"));
        let canvas = composition
            .map(|composition| Canvas { width: composition.width, height: composition.height, origin_x: 0, origin_y: 0 })
            .unwrap_or(Canvas { width: 1920, height: 1080, origin_x: 0, origin_y: 0 });

        let mut nodes = BTreeMap::new();
        let mut recipes = BTreeMap::new();
        let mut obstacles = BTreeMap::new();

        for layer in view.layers() {
            let Some(mode_key) = properties.node_for(layer, &PropertyId::new(layout::SHAPE_OUTSIDE)?) else { continue };
            let Some(meta) = view.meta(layer)? else { continue };
            let Some(transform) = transforms.binding(layer).map(|binding| binding.world) else { continue };
            let Some(flow_binding) = flow.binding(layer) else { continue };

            let mut inputs = vec![transform, flow.key()];
            let transform_input = 0;
            let flow_input = 1;
            let content_key = match meta.source {
                LayerSource::Shape => content.binding(layer).and_then(|binding| binding.content),
                LayerSource::Text => text.binding(layer).map(|binding| binding.shape),
                LayerSource::File { .. } => content.binding(layer).and_then(|binding| binding.extent),
                _ => None,
            };
            let content_input = content_key.map(|key| {
                let index = inputs.len();
                inputs.push(key);
                index
            });
            let mode = {
                let index = inputs.len();
                inputs.push(mode_key);
                index
            };
            let shape_margin = properties.node_for(layer, &PropertyId::new(layout::SHAPE_MARGIN)?).map(|key| {
                let index = inputs.len();
                inputs.push(key);
                index
            });
            let layout_margin = properties.node_for(layer, &PropertyId::new(layout::MARGIN)?).map(|key| {
                let index = inputs.len();
                inputs.push(key);
                index
            });
            let analysis_input = analysis.binding(layer).map(|binding| {
                let index = inputs.len();
                inputs.push(binding.result);
                index
            });

            let mut identity = NodeIdentity::new(NodeKind::TextObstacle, inputs);
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&(flow_binding.index as u64).to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let node = GraphNode::new(identity);
            let key = node.key();
            recipes.insert(key, Recipe::Obstacle {
                layer,
                timing: meta.timing,
                fps,
                source: meta.source,
                transform: transform_input,
                flow: flow_input,
                flow_index: flow_binding.index,
                content: content_input,
                mode,
                shape_margin,
                layout_margin,
                analysis: analysis_input,
            });
            nodes.insert(key, node);
            obstacles.insert(layer, (key, flow_binding.index, view.attrs(layer)?.unwrap_or_default().parent));
        }

        let mut bindings = BTreeMap::new();
        for layer in view.layers() {
            let Some(base) = text.binding(layer) else { continue };
            let Some(text_transform) = transforms.binding(layer).map(|binding| binding.world) else {
                bindings.insert(layer, TextFlowBinding { layer, shape: base.shape });
                continue;
            };
            let Some(text_flow) = flow.binding(layer) else {
                bindings.insert(layer, TextFlowBinding { layer, shape: base.shape });
                continue;
            };
            let parent = view.attrs(layer)?.unwrap_or_default().parent;
            let candidates: Vec<_> = obstacles.iter()
                .filter(|(other, (_, _, obstacle_parent))| **other != layer && *obstacle_parent == parent)
                .map(|(_, value)| *value)
                .collect();
            if candidates.is_empty() {
                bindings.insert(layer, TextFlowBinding { layer, shape: base.shape });
                continue;
            }

            let mut inputs = vec![base.shape, text_transform, flow.key()];
            let obstacle_inputs = candidates.into_iter().map(|(key, flow_index, _)| {
                let static_input = inputs.len();
                inputs.push(key);
                ObstacleInput { key, static_input, flow_index }
            }).collect::<Vec<_>>();

            let mut identity = NodeIdentity::new(NodeKind::TextFlow, inputs);
            identity.parameters = b"shape-outside".to_vec();
            identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            identity.parameters.extend_from_slice(&(text_flow.index as u64).to_be_bytes());
            identity.time_dependency = TimeDependency::Exact;
            let layout_node = GraphNode::new(identity);
            let layout_key = layout_node.key();
            recipes.insert(layout_key, Recipe::Layout {
                base: 0,
                transform: 1,
                flow: 2,
                flow_index: text_flow.index,
                canvas,
                obstacles: obstacle_inputs,
            });
            nodes.insert(layout_key, layout_node);

            let mut motion_identity = NodeIdentity::new(NodeKind::TextFlow, vec![layout_key, flow.key()]);
            motion_identity.parameters = b"glyph-transition-after-shape-outside".to_vec();
            motion_identity.parameters.extend_from_slice(&layer.0.to_be_bytes());
            motion_identity.parameters.extend_from_slice(&(text_flow.index as u64).to_be_bytes());
            motion_identity.time_dependency = TimeDependency::Exact;
            let motion_node = GraphNode::new(motion_identity);
            let shape = motion_node.key();
            recipes.insert(shape, Recipe::Motion { base: 0, flow: 1, flow_index: text_flow.index });
            nodes.insert(shape, motion_node);
            bindings.insert(layer, TextFlowBinding { layer, shape });
        }

        Ok(Self { nodes, recipes, bindings })
    }

    pub fn nodes(&self) -> impl ExactSizeIterator<Item = GraphNode> + '_ { self.nodes.values().cloned() }
    pub fn binding(&self, layer: LayerId) -> Option<TextFlowBinding> { self.bindings.get(&layer).copied() }

    pub fn dynamic_inputs(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        _context: &EvaluationContext,
    ) -> Option<Result<Vec<DynamicInput>, TextFlowProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Obstacle { .. } => Ok(Vec::new()),
            Recipe::Layout { flow, obstacles, .. } => (|| {
                let flow = inputs.at(*flow)
                    .and_then(|value| value.downcast_ref::<FlowFrameValue>())
                    .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
                let mut out = Vec::new();
                for obstacle in obstacles {
                    let samples = flow.transition_samples.get(obstacle.flow_index)
                        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
                    for (time, _) in samples {
                        out.push(DynamicInput { node: obstacle.key, time: *time });
                    }
                }
                Ok(out)
            })(),
            Recipe::Motion { base, flow, flow_index } => (|| {
                let flow = inputs.at(*flow)
                    .and_then(|value| value.downcast_ref::<FlowFrameValue>())
                    .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
                let samples = flow.transition_samples.get(*flow_index)
                    .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
                Ok(samples.iter().map(|(time, _)| DynamicInput {
                    node: node.identity().inputs[*base],
                    time: *time,
                }).collect())
            })(),
        })
    }

    pub fn execute(
        &self,
        node: &GraphNode,
        inputs: &NodeInputs,
        context: &EvaluationContext,
    ) -> Option<Result<NodeValue, TextFlowProgramError>> {
        let recipe = self.recipes.get(&node.key())?;
        Some(match recipe {
            Recipe::Obstacle {
                layer,
                timing,
                fps,
                source,
                transform,
                flow,
                flow_index,
                content,
                mode,
                shape_margin,
                layout_margin,
                analysis,
            } => obstacle_value(
                node, inputs, context, *layer, *timing, *fps, source, *transform, *flow,
                *flow_index, *content, *mode, *shape_margin, *layout_margin, *analysis,
            ).map(NodeValue::new),
            Recipe::Layout { base, transform, flow, canvas, obstacles, .. } => {
                layout_text(node, inputs, context, *base, *transform, *flow, *canvas, obstacles).map(NodeValue::new)
            }
            Recipe::Motion { base, flow, flow_index } => {
                apply_glyph_transition(node, inputs, *base, *flow, *flow_index).map(NodeValue::new)
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn obstacle_value(
    node: &GraphNode,
    inputs: &NodeInputs,
    context: &EvaluationContext,
    layer: LayerId,
    timing: LayerTiming,
    fps: Fps,
    source: &LayerSource,
    transform_input: usize,
    flow_input: usize,
    flow_index: usize,
    content_input: Option<usize>,
    mode_input: usize,
    shape_margin_input: Option<usize>,
    layout_margin_input: Option<usize>,
    analysis_input: Option<usize>,
) -> Result<ObstacleValue, TextFlowProgramError> {
    let frame = context.time.try_to_frame_floor(fps).map_err(|error| StoreError::Property(error.to_string()))?;
    if !timing.covers(frame) {
        return Ok(ObstacleValue { polygons: Vec::new(), margin: 0.0 });
    }
    let mode = match inputs.at(mode_input).and_then(|value| value.downcast_ref::<Value>()) {
        Some(Value::Enum(value)) => *value,
        Some(Value::F64(value)) => value.round() as i64,
        None => 0,
        Some(_) => return Err(TextFlowProgramError::InvalidInput(node.identity().kind)),
    };
    if mode == 0 {
        return Ok(ObstacleValue { polygons: Vec::new(), margin: 0.0 });
    }
    let number = |index: Option<usize>, default: f32| -> Result<f32, TextFlowProgramError> {
        Ok(match index.and_then(|index| inputs.at(index)).and_then(|value| value.downcast_ref::<Value>()) {
            Some(Value::F64(value)) if value.is_finite() => *value as f32,
            None => default,
            Some(_) => return Err(TextFlowProgramError::InvalidInput(node.identity().kind)),
        })
    };
    let margin = number(shape_margin_input, 0.0)?.max(0.0);

    if mode == 2 {
        if let Some(analysis) = analysis_input
            .and_then(|index| inputs.at(index))
            .and_then(|value| value.downcast_ref::<BlobAnalysisValue>())
        {
            if let Some(marks) = analysis.inputs.blobs(layer, EffectId(0), context.time) {
                let polygons = marks.iter().map(|mark| {
                    let half = glam::Vec2::from(mark.size) * 0.5;
                    let center = glam::Vec2::from(mark.center);
                    let lo = center - half;
                    let hi = center + half;
                    vec![[lo.x, lo.y], [hi.x, lo.y], [hi.x, hi.y], [lo.x, hi.y]]
                }).collect();
                return Ok(ObstacleValue { polygons, margin });
            }
        }
    }

    let transform = inputs.at(transform_input)
        .and_then(|value| value.downcast_ref::<TransformValue>())
        .copied()
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let flow = inputs.at(flow_input)
        .and_then(|value| value.downcast_ref::<FlowFrameValue>())
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let slot = flow.slots.get(flow_index).copied().flatten();

    let mut local = match source {
        LayerSource::Shape => {
            let Some(shapes) = content_input
                .and_then(|index| inputs.at(index))
                .and_then(|value| value.downcast_ref::<Vec<ShapeNode>>())
            else { Vec::new() };
            shape_polygons(shapes, slot.map_or([1.0, 1.0], |slot| slot.stretch))
        }
        LayerSource::Text => {
            content_input
                .and_then(|index| inputs.at(index))
                .and_then(|value| value.downcast_ref::<TextShapeValue>())
                .and_then(|text| crate::picture::text_frame::line_box(
                    &text.document,
                    &text.shaped,
                    &Canvas { width: 1, height: 1, origin_x: 0, origin_y: 0 },
                ))
                .map(rectangle)
                .into_iter()
                .collect()
        }
        LayerSource::File { .. } => {
            content_input
                .and_then(|index| inputs.at(index))
                .and_then(|value| value.downcast_ref::<MediaExtentValue>())
                .filter(|extent| extent.size[0] > 0.0 && extent.size[1] > 0.0)
                .map(|extent| rectangle([0.0, 0.0, extent.size[0], extent.size[1]]))
                .into_iter()
                .collect()
        }
        LayerSource::Group => {
            flow.sizes.get(flow_index).copied().flatten()
                .map(|size| rectangle([
                    crate::picture::CANVAS_MARGIN,
                    crate::picture::CANVAS_MARGIN,
                    crate::picture::CANVAS_MARGIN + size[0],
                    crate::picture::CANVAS_MARGIN + size[1],
                ]))
                .into_iter()
                .collect()
        }
        _ => Vec::new(),
    };

    if mode == 1 {
        let grow = number(layout_margin_input, 0.0)?;
        let (lo, hi) = local.iter().flatten().fold(
            (glam::Vec2::splat(f32::MAX), glam::Vec2::splat(f32::MIN)),
            |(lo, hi), point| {
                let point = glam::Vec2::from(*point);
                (lo.min(point), hi.max(point))
            },
        );
        local = if lo.x <= hi.x {
            vec![rectangle([lo.x - grow, lo.y - grow, hi.x + grow, hi.y + grow])]
        } else {
            Vec::new()
        };
    }

    let polygons = local.into_iter().map(|polygon| {
        polygon.into_iter().map(|point| {
            transform.affine.transform_point2(glam::Vec2::from(point)).to_array()
        }).collect()
    }).collect();
    Ok(ObstacleValue { polygons, margin })
}

fn layout_text(
    node: &GraphNode,
    inputs: &NodeInputs,
    context: &EvaluationContext,
    base_input: usize,
    transform_input: usize,
    flow_input: usize,
    canvas: Canvas,
    obstacles: &[ObstacleInput],
) -> Result<TextShapeValue, TextFlowProgramError> {
    let base = inputs.at(base_input)
        .and_then(|value| value.downcast_ref::<TextShapeValue>())
        .cloned()
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let transform = inputs.at(transform_input)
        .and_then(|value| value.downcast_ref::<TransformValue>())
        .copied()
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let flow = inputs.at(flow_input)
        .and_then(|value| value.downcast_ref::<FlowFrameValue>())
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;

    let to_text = transform.affine.inverse();
    let mut around = Vec::new();
    let mut dynamic = node.identity().inputs.len();
    for obstacle in obstacles {
        let samples = flow.transition_samples.get(obstacle.flow_index)
            .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
        if samples.is_empty() {
            if let Some(value) = inputs.at(obstacle.static_input).and_then(|value| value.downcast_ref::<ObstacleValue>()) {
                extend_obstacles(&mut around, value, 1.0, to_text);
            }
            continue;
        }
        for (_, weight) in samples {
            let value = inputs.at(dynamic)
                .and_then(|value| value.downcast_ref::<ObstacleValue>())
                .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
            dynamic += 1;
            extend_obstacles(&mut around, value, *weight, to_text);
        }
    }

    let shaped = crate::picture::text_frame::shape_document_around(
        &base.document,
        context.time,
        &canvas,
        &around,
    ).map_err(|error| TextFlowProgramError::Shape(error.to_string()))?.unwrap_or_default();
    Ok(TextShapeValue { document: base.document, shaped })
}

fn extend_obstacles(
    out: &mut Vec<crate::picture::text_frame::Obstacle>,
    value: &ObstacleValue,
    weight: f32,
    to_text: glam::Affine2,
) {
    for polygon in &value.polygons {
        if polygon.len() < 3 { continue; }
        out.push(crate::picture::text_frame::Obstacle {
            margin: value.margin,
            weight,
            points: polygon.iter().map(|point| {
                to_text.transform_point2(glam::Vec2::from(*point)).to_array()
            }).collect(),
        });
    }
}

fn apply_glyph_transition(
    node: &GraphNode,
    inputs: &NodeInputs,
    base_input: usize,
    flow_input: usize,
    flow_index: usize,
) -> Result<TextShapeValue, TextFlowProgramError> {
    let mut current = inputs.at(base_input)
        .and_then(|value| value.downcast_ref::<TextShapeValue>())
        .cloned()
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let flow = inputs.at(flow_input)
        .and_then(|value| value.downcast_ref::<FlowFrameValue>())
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    let samples = flow.transition_samples.get(flow_index)
        .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
    if samples.is_empty() { return Ok(current); }

    let now = glyph_positions(&current.shaped);
    let mut sum = vec![[0.0f32; 2]; now.len()];
    let mut weights = vec![0.0f32; now.len()];
    let dynamic_start = node.identity().inputs.len();
    for (sample_index, (_, weight)) in samples.iter().enumerate() {
        let past = inputs.at(dynamic_start + sample_index)
            .and_then(|value| value.downcast_ref::<TextShapeValue>())
            .ok_or(TextFlowProgramError::InvalidInput(node.identity().kind))?;
        let past: std::collections::HashMap<usize, [f32; 2]> = glyph_positions(&past.shaped).into_iter().collect();
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
    Ok(current)
}

fn glyph_positions(shaped: &ShapedText) -> Vec<(usize, [f32; 2])> {
    shaped.lines.iter().flat_map(|line| {
        line.glyph_bytes.iter().zip(&line.glyph_xs).map(move |(byte, x)| (*byte, [*x, line.baseline_y]))
    }).collect()
}

fn rectangle(bounds: [f32; 4]) -> Vec<[f32; 2]> {
    vec![
        [bounds[0], bounds[1]],
        [bounds[2], bounds[1]],
        [bounds[2], bounds[3]],
        [bounds[0], bounds[3]],
    ]
}

fn shape_polygons(shapes: &[ShapeNode], stretch: [f32; 2]) -> Vec<Vec<[f32; 2]>> {
    let stretched;
    let shapes = if stretch != [1.0, 1.0] {
        stretched = crate::picture::shapes_ops::stretch_outline(shapes, stretch);
        stretched.as_slice()
    } else {
        shapes
    };
    let Some(canvas) = crate::picture::shapes_ops::content_canvas(shapes).ok().flatten() else { return Vec::new() };
    let origin = glam::vec2(canvas.origin_x as f32, canvas.origin_y as f32);
    let mut out = Vec::new();
    for shape in crate::picture::shapes_ops::flatten(shapes).unwrap_or_default() {
        for instance in crate::picture::shapes_ops::resolve(&shape).unwrap_or_default() {
            for contour in &instance.path {
                let n = contour.vertices.len();
                if n < 2 { continue; }
                let point = |value: crate::doc::vector::Point| glam::vec2(value.x as f32, value.y as f32);
                let mut polygon = Vec::new();
                for index in 0..if contour.closed { n } else { n.saturating_sub(1) } {
                    let a = &contour.vertices[index];
                    let b = &contour.vertices[(index + 1) % n];
                    let p0 = point(a.point);
                    let p3 = point(b.point);
                    let p1 = p0 + point(a.out_tangent);
                    let p2 = p3 + point(b.in_tangent);
                    for step in 0..8 {
                        let u = step as f32 / 8.0;
                        let w = 1.0 - u;
                        polygon.push((p0 * w * w * w + p1 * 3.0 * w * w * u + p2 * 3.0 * w * u * u + p3 * u * u * u + origin).to_array());
                    }
                }
                if polygon.len() >= 3 { out.push(polygon); }
            }
        }
    }
    out
}
