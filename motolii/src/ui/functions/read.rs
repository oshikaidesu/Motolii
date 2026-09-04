use crate::doc::store::{property, LayerId, LayerSource, PropertyId, RationalTime, ShapeNode, StoreError, StoreView, Value};
use crate::render::engine::EffectDescriptor;
use crate::ui::fixture::{AssetFamily, ColorRow, EffectBlock, InspectorData, PropRow, LABEL_PALETTE};
use crate::ui::session::ColorSlot;

pub(in crate::ui) fn property_value(
    view: &StoreView<'_>,
    layer: LayerId,
    property: &PropertyId,
    at: RationalTime,
) -> Result<Option<Value>, StoreError> {
    Ok(view.value_at(layer, property, at)?.or(view.default_value(layer, property)?))
}

pub(in crate::ui) fn label_rgb(ix: u8) -> [u8; 3] {
    let hex = LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()];
    let v = u32::from_str_radix(&hex[1..], 16).unwrap_or(0x8c8c8c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// 木の中で最初に塗りを持つ葉と、そこへの道。Inspector/Deskは同じ塗りを見る。
pub(in crate::ui) fn first_shape_fill(nodes: &[ShapeNode], path: Vec<usize>) -> Option<(Vec<usize>, crate::doc::vector::Brush)> {
    nodes.iter().enumerate().find_map(|(i, node)| {
        let mut here = path.clone();
        here.push(i);
        match node {
            ShapeNode::Leaf(shape) => shape.fill.as_ref().map(|fill| (here, fill.brush.clone())),
            ShapeNode::Group(group) => first_shape_fill(&group.children, here),
        }
    })
}

pub(in crate::ui) fn hex_of(rgba: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
}

pub(in crate::ui) fn asset_family(asset_type: &str) -> AssetFamily {
    let t = asset_type.to_ascii_lowercase();
    if t.starts_with("video/") {
        AssetFamily::Video
    } else if t.starts_with("audio/") {
        AssetFamily::Audio
    } else if t.starts_with("image/") {
        AssetFamily::TwoD
    } else if t.starts_with("pointcloud") || t.starts_with("model/") || t.starts_with("mesh") {
        AssetFamily::ThreeD
    } else if t.starts_with("application/") || t.starts_with("text/") || t.starts_with("rerun") {
        AssetFamily::Data
    } else {
        AssetFamily::Other
    }
}

pub(in crate::ui) fn inspector_data_from_doc(view: &StoreView, layer: LayerId, t: RationalTime, catalog: &[EffectDescriptor]) -> InspectorData {
    let value_of = |prop: &str| PropertyId::new(prop).ok().and_then(|p| property_value(view, layer, &p, t).ok().flatten());
    let keyed = |prop: &str| PropertyId::new(prop).ok().and_then(|p| view.track(layer, &p).ok().flatten()).is_some();
    let f = |v: f64| format!("{v:.2}");

    let f1 = |v: f64| format!("{v:.2}");
    let (pos_x, pos_y) = match value_of(property::POSITION) {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => (0.0, 0.0),
    };
    let (px, py) = (f1(pos_x), f1(pos_y));
    let opacity_v = match value_of(property::OPACITY) {
        Some(Value::F64(v)) => v,
        _ => 1.0,
    };
    let opacity = f(opacity_v);
    let (scale_x, scale_y) = match value_of(property::SCALE) {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => (1.0, 1.0),
    };
    let rotation_v = match value_of(property::ROTATION) {
        Some(Value::F64(v)) => v,
        _ => 0.0,
    };
    let rot_x = match value_of(property::ROTATION_X) {
        Some(Value::F64(v)) => v,
        _ => 0.0,
    };
    let rot_y = match value_of(property::ROTATION_Y) {
        Some(Value::F64(v)) => v,
        _ => 0.0,
    };
    let pos_z = match value_of(property::POSITION_Z) {
        Some(Value::F64(v)) => v,
        _ => 0.0,
    };

    let sel_attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let key_count: usize = [property::POSITION, property::OPACITY]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .filter_map(|p| view.track(layer, &p).ok().flatten())
        .map(|tr| tr.keys().len())
        .sum();
    let attached_effects = view.effects(layer).unwrap_or_default();
    let has_effects = !attached_effects.is_empty();
    let effects: Vec<EffectBlock> = attached_effects
        .into_iter()
        .map(|instance| {
            let params = catalog.iter().find(|d| d.plugin_id == instance.plugin_id).map(|d| d.params.as_slice()).unwrap_or(&[]);
            let id = instance.id;
            let rows = params
                .iter()
                .filter_map(move |param| {
                    let prop = PropertyId::effect_param(id, &param.name).ok()?;
                    let keyed = view.track(layer, &prop).ok().flatten().is_some();
                    let v = match view.value_at(layer, &prop, t).ok().flatten() {
                        Some(Value::F64(v)) => v,
                        _ => param.default,
                    };
                    Some(PropRow {
                        label: param.name.clone(),
                        cells: [String::new(), String::new(), f(v)],
                        dims: [false, false, false],
                        keyed,
                        property: Some(prop.name().to_owned()),
                        vec2: false,
                        value: Value::F64(v),
                        range: param.range,
                        axis: [None, None, None],
                    })
                })
                .collect::<Vec<_>>();
            EffectBlock { id: instance.id.0, plugin_id: instance.plugin_id, params: rows }
        })
        .collect();
    let source_name = match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::File { path, .. }) => {
            if crate::render::media::is_point_cloud_path(&path) {
                "point cloud"
            } else if crate::render::media::is_mesh_path(&path) {
                "3D model"
            } else {
                "media"
            }
        }
        Some(LayerSource::Null) => "null",
        Some(LayerSource::Shape) => "shape",
        Some(LayerSource::Text) => "text",
        Some(LayerSource::Group) => "group",
        None => "solid",
    };

    let text = match view.text_document(layer) {
        Ok(Some(doc)) => {
            let mut rows = vec![PropRow {
                label: "Content".into(),
                cells: [String::new(), String::new(), doc.content.eval(t).to_string()],
                dims: [false, false, false],
                keyed: doc.content.keys().iter().any(|k| k.t == t),
                property: None,
                vec2: false,
                value: Value::F64(0.0),
                range: None,
                axis: [None, None, None],
            }];
            // 級数。style の size を property が上書きする(resolve と同じ順)。数の行なので擦れる。
            if let Some(style) = doc.styles.first() {
                let prop = PropertyId::text_style_size(style.id);
                let size = match view.value_at(layer, &prop, t).ok().flatten() {
                    Some(Value::F64(v)) => v,
                    _ => f64::from(style.size),
                };
                rows.push(PropRow {
                    label: "Size".into(),
                    cells: [String::new(), String::new(), f1(size)],
                    dims: [false, false, false],
                    keyed: keyed(prop.name()),
                    property: Some(prop.name().to_owned()),
                    vec2: false,
                    value: Value::F64(size),
                    range: Some((1.0, 1000.0)),
                    axis: [None, None, None],
                });
                // 行間と字送り。model も resolve も持っているのに行が無かった(組版の主要素)。
                let line_height_prop = PropertyId::text_style_line_height(style.id);
                let line_height = match view.value_at(layer, &line_height_prop, t).ok().flatten() {
                    Some(Value::F64(v)) => v,
                    _ => style.line_height.map_or(size * 1.2, f64::from),
                };
                let tracking_prop = PropertyId::text_style_tracking(style.id);
                let tracking = match view.value_at(layer, &tracking_prop, t).ok().flatten() {
                    Some(Value::F64(v)) => v,
                    _ => f64::from(style.tracking),
                };
                for (label, prop, value, range) in [
                    ("Line height", line_height_prop, line_height, (0.0, 1000.0)),
                    ("Tracking", tracking_prop, tracking, (-200.0, 200.0)),
                ] {
                    rows.push(PropRow {
                        label: label.into(),
                        cells: [String::new(), String::new(), f1(value)],
                        dims: [false, false, false],
                        keyed: keyed(prop.name()),
                        property: Some(prop.name().to_owned()),
                        vec2: false,
                        value: Value::F64(value),
                        range: Some(range),
                        axis: [None, None, None],
                    });
                }
            }
            rows
        }
        _ => Vec::new(),
    };

    let mut colors = Vec::new();
    let row = |label, c: [f64; 4], slot| ColorRow {
        label,
        hex: hex_of([
            (c[0] * 255.0).round() as u8,
            (c[1] * 255.0).round() as u8,
            (c[2] * 255.0).round() as u8,
            (c[3] * 255.0).round() as u8,
        ]),
        slot,
    };
    match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::Text) => {
            if let Ok(Some(doc)) = view.text_document(layer) {
                if let Some(style) = doc.styles.first() {
                    colors.push(row("Fill", style.fill, ColorSlot::TextFill { layer, style: style.id }));
                    // 縁取りは無くても行を出す。無い物を「足す口」が無いと、縁取り無しの歌詞しか作れない。
                    let stroke = style.stroke_color.unwrap_or([0.0, 0.0, 0.0, 1.0]);
                    colors.push(row("Stroke", stroke, ColorSlot::TextStroke { layer, style: style.id }));
                }
            }
        }
        Some(LayerSource::Shape) => {
            if let Ok(shapes) = view.shapes(layer) {
                if let Some((path, brush)) = first_shape_fill(&shapes, Vec::new()) {
                    match brush {
                        crate::doc::vector::Brush::Solid(rgb) => {
                            colors.push(row("Color", [rgb.r, rgb.g, rgb.b, 1.0], ColorSlot::ShapeFill { layer, path }));
                        }
                        crate::doc::vector::Brush::Gradient(gradient) => {
                            let start = gradient
                                .stops
                                .iter()
                                .min_by(|a, b| a.offset.total_cmp(&b.offset))
                                .map(|stop| stop.color)
                                .unwrap_or(crate::doc::vector::Rgb::BLACK);
                            let end = gradient
                                .stops
                                .iter()
                                .max_by(|a, b| a.offset.total_cmp(&b.offset))
                                .map(|stop| stop.color)
                                .unwrap_or(start);
                            colors.push(row(
                                "Start",
                                [start.r, start.g, start.b, 1.0],
                                ColorSlot::ShapeGradientStop { layer, path: path.clone(), end: false },
                            ));
                            colors.push(row(
                                "End",
                                [end.r, end.g, end.b, 1.0],
                                ColorSlot::ShapeGradientStop { layer, path, end: true },
                            ));
                        }
                    }
                }
            }
        }
        _ => {}
    }

    let transform = vec![
        PropRow {
            label: "Position".into(),
            cells: [px, py, f1(pos_z)],
            dims: [false, false, false],
            keyed: keyed(property::POSITION),
            property: Some(property::POSITION.to_owned()),
            vec2: true,
            value: Value::Vec2([pos_x, pos_y]),
            range: None,
            axis: [None, None, Some((property::POSITION_Z.to_owned(), Value::F64(pos_z)))],
        },
        PropRow {
            label: "Scale".into(),
            cells: [f(scale_x), f(scale_y), f(1.0)],
            dims: [false, false, true],
            keyed: keyed(property::SCALE),
            property: Some(property::SCALE.to_owned()),
            vec2: true,
            value: Value::Vec2([scale_x, scale_y]),
            range: None,
            axis: [None, None, None],
        },
        PropRow {
            label: "Rotation".into(),
            cells: [f(rot_x), f(rot_y), f(rotation_v)],
            dims: [false, false, false],
            keyed: keyed(property::ROTATION),
            property: Some(property::ROTATION.to_owned()),
            vec2: false,
            value: Value::F64(rotation_v),
            range: None,
            axis: [
                Some((property::ROTATION_X.to_owned(), Value::F64(rot_x))),
                Some((property::ROTATION_Y.to_owned(), Value::F64(rot_y))),
                None,
            ],
        },
        PropRow {
            label: "Opacity".into(),
            cells: [String::new(), String::new(), opacity],
            dims: [false, false, false],
            keyed: keyed(property::OPACITY),
            property: Some(property::OPACITY.to_owned()),
            vec2: false,
            value: Value::F64(opacity_v),
            range: None,
            axis: [None, None, None],
        },
    ];

    InspectorData {
        blend: sel_attrs.blend_mode,
        ident_name: sel_attrs.name,
        ident_sub: format!("{source_name} · {key_count} keys"),
        colors,
        text,
        transform,
        effects,
        has_effects,
    }
}

/// 12.3 MB の様な人向けの容量。
pub(in crate::ui) fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub(in crate::ui) fn fmt_timecode(sec: f64, fps: crate::doc::store::Fps) -> String {
    let at = crate::doc::store::RationalTime::try_new((sec.max(0.0) * 1_000_000.0).round() as i64, 1_000_000)
        .unwrap_or(crate::doc::store::RationalTime::ZERO);
    let frame = at.try_to_frame_round(fps).unwrap_or(0);
    let nominal = fps.as_f64().round().max(1.0) as i64;
    format!("{}:{:02}:{:02}", frame / (nominal * 60), (frame / nominal) % 60, frame % nominal)
}
