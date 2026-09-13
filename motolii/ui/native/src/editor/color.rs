use crate::doc::eval::Value;
use crate::doc::store::{property, Animate, Document, Intent, LayerId, PropertyId, RationalTime, ShapeNode, StoreError, TextStyleId};
use crate::editor::functions::read;
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
/// slot が指す色の property。色は property で、書類の brush はその既定。
pub(crate) fn property_of(doc: &Document, slot: &ColorSlot) -> Option<PropertyId> {
    let name = match slot {
        ColorSlot::TextFill { style, .. } => return Some(PropertyId::text_style_fill_color(*style)),
        ColorSlot::ShapeFill { .. } => property::SHAPE_FILL_COLOR.to_owned(),
        // 線は効果の責務。ここからは書けない。
        ColorSlot::ShapeStroke { .. } => return None,
        ColorSlot::Property { property, .. } => property.clone(),
        ColorSlot::ShapeGradientPoint { index, .. } => format!("{}{index}.color", property::FILL_STOP_PREFIX),
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = doc.view().shapes(*layer).ok()?;
            let shape = leaf_mut(&mut shapes, path)?;
            let Brush::Gradient(g) = &shape.fill.as_ref()?.brush else { return None };
            let pick = |a: &(usize, &GradientStop), b: &(usize, &GradientStop)| a.1.offset.total_cmp(&b.1.offset);
            let it = g.stops.iter().enumerate();
            let (index, _) = if *end { it.max_by(pick)? } else { it.min_by(pick)? };
            format!("{}{index}.color", property::FILL_STOP_PREFIX)
        }
    };
    PropertyId::new(&name).ok()
}

/// property の名前から slot へ。Inspector の色の行が Browser の輪へ焦点を渡す時の逆引き。
pub(crate) fn slot_of(doc: &Document, layer: LayerId, name: &str) -> Option<ColorSlot> {
    if let Some(rest) = name.strip_prefix(property::TEXT_STYLE_PREFIX) {
        let style = rest.strip_suffix(".fill_color")?.parse().ok()?;
        return Some(ColorSlot::TextFill { layer, style: TextStyleId(style) });
    }
    if let Some(rest) = name.strip_prefix(property::FILL_STOP_PREFIX) {
        let index = rest.strip_suffix(".color")?.parse().ok()?;
        let (path, _) = read::first_shape_fill(&doc.view().shapes(layer).ok()?, Vec::new())?;
        return Some(ColorSlot::ShapeGradientPoint { layer, path, index });
    }
    let shapes = doc.view().shapes(layer).ok()?;
    match name {
        property::SHAPE_FILL_COLOR => read::first_shape_fill(&shapes, Vec::new()).map(|(path, _)| ColorSlot::ShapeFill { layer, path }),
        _ => Some(ColorSlot::Property { layer, property: name.to_owned() }),
    }
}

/// 時刻 t の色: property があればそれ、無ければ書類の brush。
pub(crate) fn read_color(doc: &Document, slot: &ColorSlot, t: RationalTime) -> Option<[f64; 4]> {
    let view = doc.view();
    if let Some(Value::Color(c)) = property_of(doc, slot).and_then(|p| view.value_at(slot.layer(), &p, t).ok().flatten()) {
        return Some(c);
    }
    match slot {
        ColorSlot::Property { .. } => None,
        ColorSlot::TextFill { layer, style } => {
            let text = view.text_document(*layer).ok()??;
            Some(text.styles.iter().find(|s| s.id == *style)?.fill)
        }
        ColorSlot::ShapeFill { layer, path } => {
            let mut shapes = view.shapes(*layer).ok()?;
            match leaf_mut(&mut shapes, path)?.fill.as_ref().map(|f| &f.brush) {
                Some(Brush::Solid(rgb)) => Some([rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
        ColorSlot::ShapeStroke { layer, path } => {
            let mut shapes = view.shapes(*layer).ok()?;
            match leaf_mut(&mut shapes, path)?.stroke.as_ref().map(|s| &s.brush) {
                Some(Brush::Solid(rgb)) => Some([rgb.r, rgb.g, rgb.b, 1.0]),
                _ => Some([0.0, 0.0, 0.0, 1.0]),
            }
        }
        ColorSlot::ShapeGradientPoint { layer, path, index } => {
            let mut shapes = view.shapes(*layer).ok()?;
            let Brush::Gradient(g) = &leaf_mut(&mut shapes, path)?.fill.as_ref()?.brush else { return None };
            let c = g.stops.get(*index)?.color;
            Some([c.r, c.g, c.b, 1.0])
        }
        ColorSlot::ShapeGradientStop { layer, path, end } => {
            let mut shapes = view.shapes(*layer).ok()?;
            match leaf_mut(&mut shapes, path)?.fill.as_ref().map(|fill| &fill.brush) {
                Some(Brush::Gradient(gradient)) => endpoint_color(gradient, *end).map(|rgb| [rgb.r, rgb.g, rgb.b, 1.0]),
                _ => None,
            }
        }
    }
}

/// 色を書く: 他の property と同じ入口(`place_checked`)。animate が立っていれば鍵になる。
/// 単色で無い塗り(gradient)に単色を書く手は無い(stop ごとの slot で書く)。
pub(crate) fn write_color(
    doc: &Document,
    slot: &ColorSlot,
    rgba: [f64; 4],
    at: RationalTime,
    animate: Animate,
) -> Result<Vec<Intent>, StoreError> {
    if let ColorSlot::ShapeFill { layer, path } = slot {
        let mut shapes = doc.view().shapes(*layer)?;
        if !matches!(leaf_mut(&mut shapes, path).and_then(|s| s.fill.as_ref()).map(|f| &f.brush), Some(Brush::Solid(_)) | None) {
            return Err(StoreError::Property("The color target is no longer editable".into()));
        }
    }
    let property = property_of(doc, slot).ok_or_else(|| StoreError::Property("The color target is no longer editable".into()))?;
    let alpha = match slot { ColorSlot::TextFill { .. } | ColorSlot::Property { .. } => rgba[3], _ => 1.0 };
    Ok(doc.place_checked(slot.layer(), &property, Value::Color([rgba[0], rgba[1], rgba[2], alpha]), at, animate)?.into_iter().collect())
}
