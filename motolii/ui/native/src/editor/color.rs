use crate::doc::store::{Document,Intent,LayerId,ShapeNode,StoreError};
use crate::doc::vector::{Brush,Fill,Gradient,GradientStop,GradientType,PathSource,Point,Rgb};
use crate::editor::session::ColorSlot;
pub(super) fn leaf_mut<'a>(
    nodes: &'a mut [ShapeNode],
    path: &[usize],
) -> Option<&'a mut crate::doc::vector::Shape> {
    let (first, rest) = path.split_first()?;
    match nodes.get_mut(*first)? {
        ShapeNode::Leaf(shape) if rest.is_empty() => Some(shape),
        ShapeNode::Group(group) => leaf_mut(&mut group.children, rest),
        ShapeNode::Leaf(_) => None,
    }
}
pub(super) fn shape_location(slot: &ColorSlot) -> Option<(LayerId, &[usize])> {
    match slot {
        ColorSlot::ShapeFill { layer, path } | ColorSlot::ShapeGradientPoint { layer, path, .. } | ColorSlot::ShapeGradientStop { layer, path, .. } => {
            Some((*layer, path))
        }
        _ => None,
    }
}
pub(super) fn gradient_axis(source: &PathSource) -> (Point, Point) {
    let b = crate::doc::store::shape_props::source_bounds(source);
    (Point { x: b[0], y: (b[1] + b[3]) * 0.5 }, Point { x: b[2], y: (b[1] + b[3]) * 0.5 })
}
fn endpoint_color(gradient: &Gradient, end: bool) -> Option<Rgb> {
    let choose = if end {
        gradient
            .stops
            .iter()
            .max_by(|a, b| a.offset.total_cmp(&b.offset))
    } else {
        gradient
            .stops
            .iter()
            .min_by(|a, b| a.offset.total_cmp(&b.offset))
    };
    choose.map(|stop| stop.color)
}
fn write_endpoint(gradient: &mut Gradient, end: bool, color: Rgb) {
    match gradient.stops.len() {
        0 => gradient.stops.extend([
            GradientStop { offset: 0.0, color },
            GradientStop { offset: 1.0, color },
        ]),
        1 => {
            let first = gradient.stops[0].color;
            gradient.stops[0].offset = 0.0;
            gradient.stops.push(GradientStop {
                offset: 1.0,
                color: first,
            });
        }
        _ => {}
    }
    let index = if end {
        gradient
            .stops
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.offset.total_cmp(&b.offset))
    } else {
        gradient
            .stops
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.offset.total_cmp(&b.offset))
    }
    .map(|(index, _)| index);
    if let Some(index) = index {
        gradient.stops[index].color = color;
    }
}
pub(crate) fn set_shape_gradient(
    doc: &mut Document,
    slot: &ColorSlot,
    enabled: bool,
) -> Result<(), StoreError> {
    let Some((layer, path)) = shape_location(slot) else {
        return Ok(());
    };
    let d = doc;
    let mut shapes = d.view().shapes(layer)?;
    let Some(shape) = leaf_mut(&mut shapes, path) else {
        return Ok(());
    };
    let mut fill = shape.fill.take().unwrap_or_default();
    fill.brush = match (enabled, fill.brush) {
        (true, Brush::Solid(color)) => {
            let (start, end) = gradient_axis(&shape.source);
            Brush::Gradient(Gradient {
                kind: GradientType::Linear,
                start,
                end,
                stops: vec![
                    GradientStop { offset: 0.0, color },
                    GradientStop { offset: 1.0, color },
                ],
            })
        }
        (true, brush @ Brush::Gradient(_)) => brush,
        (false, Brush::Gradient(gradient)) => {
            Brush::Solid(endpoint_color(&gradient, false).unwrap_or(Rgb::BLACK))
        }
        (false, brush @ Brush::Solid(_)) => brush,
    };
    shape.fill = Some(fill);
    d.apply(Intent::SetShapes { layer, shapes }).map(|_| ())
}
pub(crate) fn read_color(doc: &Document, slot: &ColorSlot) -> Option<[f64; 4]> {
    let d = doc;
    let view = d.view();
    match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => {
            let text = view.text_document(*layer).ok()??;
            let found = text.styles.iter().find(|s| s.id == *style)?;
            match slot {
                ColorSlot::TextFill { .. } => Some(found.fill),
                _ => found.stroke_color,
            }
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            match shape.fill.as_ref().map(|f| &f.brush) {
                Some(Brush::Solid(rgb)) => Some([rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
        ColorSlot::ShapeStroke { layer, path } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            match shape.stroke.as_ref().map(|s| &s.brush) {
                Some(Brush::Solid(rgb)) => Some([rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
        ColorSlot::ShapeGradientPoint { layer, path, index } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            let Brush::Gradient(g) = &shape.fill.as_ref()?.brush else { return None };
            let c = g.stops.get(*index)?.color;
            Some([c.r, c.g, c.b, 1.0])
        }
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            match shape.fill.as_ref().map(|fill| &fill.brush) {
                Some(Brush::Gradient(gradient)) => {
                    endpoint_color(gradient, *end).map(|rgb| [rgb.r, rgb.g, rgb.b, 1.0])
                }
                _ => None,
            }
        }
    }
}
pub(crate) fn write_color(
    doc: &Document,
    slot: &ColorSlot,
    [r, g, b]: [f64; 3],
) -> Result<Intent, StoreError> {
    let d = doc;
    let intent = match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => {
            let Some(mut text) = d.view().text_document(*layer)? else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            let Some(found) = text.styles.iter_mut().find(|s| s.id == *style) else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            match slot {
                ColorSlot::TextFill { .. } => found.fill = [r, g, b, found.fill[3]],
                _ => {
                    let a = found.stroke_color.map_or(1.0, |c| c[3]);
                    found.stroke_color = Some([r, g, b, a]);
                    // 幅 0 の縁取りは描かれない。色を付けた時点で見える幅を入れる(級数の 5%)。
                    if found.stroke_width <= 0.0 {
                        found.stroke_width = (found.size * 0.05).max(1.0);
                    }
                }
            }
            Intent::SetTextDocument {
                layer: *layer,
                document: text,
            }
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            // gradient の塗りは単色で潰さない(輪の相手は read_color が Solid の時だけ)。
            if matches!(
                shape.fill.as_ref().map(|f| &f.brush),
                Some(Brush::Gradient(_))
            ) {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            }
            let mut fill = shape.fill.take().unwrap_or_default();
            fill.brush = Brush::Solid(Rgb { r, g, b });
            shape.fill = Some(Fill { ..fill });
            Intent::SetShapes {
                layer: *layer,
                shapes,
            }
        }
        ColorSlot::ShapeStroke { layer, path } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            // 線が無い形に色を付けると線が生える(文字の縁取りと同じ流儀)。
            let mut stroke = shape.stroke.take().unwrap_or_default();
            stroke.brush = Brush::Solid(Rgb { r, g, b });
            if stroke.width <= 0.0 { stroke.width = crate::doc::store::shape_props::DEFAULT_STROKE_WIDTH; }
            shape.stroke = Some(stroke);
            Intent::SetShapes { layer: *layer, shapes }
        }
        ColorSlot::ShapeGradientPoint { layer, path, index } => {
            let mut shapes = d.view().without_transients().shapes(*layer)?;
            let shape = leaf_mut(&mut shapes, path).ok_or_else(||StoreError::Property("Gradient no longer exists".into()))?;
            let Some(Fill { brush: Brush::Gradient(gradient), .. }) = &mut shape.fill else { return Err(StoreError::Property("Gradient no longer exists".into())) };
            let stop = gradient.stops.get_mut(*index).ok_or_else(||StoreError::Property("Stop no longer exists".into()))?;
            stop.color = Rgb { r, g, b };
            Intent::SetShapes { layer: *layer, shapes }
        }
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = d.view().shapes(*layer)?;
            let Some(shape) = leaf_mut(&mut shapes, path) else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            let Some(fill) = shape.fill.as_mut() else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            let Brush::Gradient(gradient) = &mut fill.brush else {
                return Err(StoreError::Property("The color target is no longer editable".into()));
            };
            write_endpoint(gradient, *end, Rgb { r, g, b });
            Intent::SetShapes {
                layer: *layer,
                shapes,
            }
        }
    };
    Ok(intent)
}
pub(crate) fn write_alpha(
    doc: &mut Document,
    slot: &ColorSlot,
    alpha: f64,
) -> Result<(), StoreError> {
    let d = doc;
    let (layer, style) = match slot {
        ColorSlot::TextFill { layer, style } | ColorSlot::TextStroke { layer, style } => {
            (*layer, *style)
        }
        ColorSlot::ShapeFill { .. } | ColorSlot::ShapeStroke { .. } | ColorSlot::ShapeGradientStop { .. } | ColorSlot::ShapeGradientPoint { .. } => return Ok(()),
    };
    let Some(mut text) = d.view().text_document(layer)? else {
        return Ok(());
    };
    let Some(found) = text.styles.iter_mut().find(|s| s.id == style) else {
        return Ok(());
    };
    let a = alpha.clamp(0.0, 1.0);
    match slot {
        ColorSlot::TextFill { .. } => found.fill[3] = a,
        _ => {
            if let Some(c) = found.stroke_color.as_mut() {
                c[3] = a;
            }
        }
    }
    d.apply(Intent::SetTextDocument {
        layer,
        document: text,
    })
    .map(|_| ())
}
