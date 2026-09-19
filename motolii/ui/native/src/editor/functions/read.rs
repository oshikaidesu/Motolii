use crate::doc::store::{names, property, LayerId, LayerSource, PropertyId, RationalTime, ShapeNode, StoreError, StoreView, Value};
use crate::render::engine::EffectDescriptor;
use crate::editor::fixture::{AssetFamily, EffectBlock, InspectorData, PropRow, LABEL_PALETTE};
use crate::editor::session::ColorSlot;

pub(crate) fn property_value(
    view: &StoreView<'_>,
    layer: LayerId,
    property: &PropertyId,
    at: RationalTime,
) -> Result<Option<Value>, StoreError> {
    Ok(view.value_at(layer, property, at)?.or(view.default_value(layer, property)?))
}

pub(crate) fn label_rgb(ix: u8) -> [u8; 3] {
    let hex = LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()];
    let v = u32::from_str_radix(&hex[1..], 16).unwrap_or(0x8c8c8c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// 木の中で最初に塗りを持つ葉と、そこへの道。Inspector/Deskは同じ塗りを見る。
/// 木の最初の葉(線の色・太さの相手)。
pub(crate) fn first_leaf(nodes: &[ShapeNode], path: Vec<usize>) -> Option<(Vec<usize>, crate::doc::vector::Shape)> {
    nodes.iter().enumerate().find_map(|(i, node)| {
        let mut here = path.clone();
        here.push(i);
        match node {
            ShapeNode::Leaf(shape) => Some((here, shape.clone())),
            ShapeNode::Group(group) => first_leaf(&group.children, here),
        }
    })
}

pub(crate) fn first_shape_fill(nodes: &[ShapeNode], path: Vec<usize>) -> Option<(Vec<usize>, crate::doc::vector::Brush)> {
    nodes.iter().enumerate().find_map(|(i, node)| {
        let mut here = path.clone();
        here.push(i);
        match node {
            ShapeNode::Leaf(shape) => shape.fill.as_ref().map(|fill| (here, fill.brush.clone())),
            ShapeNode::Group(group) => first_shape_fill(&group.children, here),
        }
    })
}

pub(crate) fn hex_of(rgba: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
}

pub(crate) fn asset_family(asset_type: &str) -> AssetFamily {
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

pub(crate) fn inspector_data_from_doc(view: &StoreView, layer: LayerId, t: RationalTime, catalog: &[EffectDescriptor]) -> InspectorData {
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
    let scale_z = match value_of(property::SCALE_Z) {
        Some(Value::F64(v)) => v,
        _ => 1.0,
    };
    let depth_v = match value_of(property::DEPTH) {
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
            let id = instance.id;
            if let Some(kind) = crate::doc::extensions::placement::kind(&instance.plugin_id) {
                // 配置効果は形のトグルで出る欄が変わる。表の順・名前・組をそのまま運ぶ。
                let get = |name: &str| PropertyId::effect_param(id, name).ok().and_then(|p| view.value_at(layer, &p, t).ok().flatten());
                let mode = match get("mode") { Some(Value::F64(m)) => m.round().max(0.0) as u8, _ => 0 };
                let rows = kind.params.iter().filter(|p| p.shown(mode)).filter_map(|param| {
                    let prop = PropertyId::effect_param(id, param.name).ok()?;
                    let keyed = view.track(layer, &prop).ok().flatten().is_some();
                    let value = get(param.name).or_else(|| crate::doc::extensions::placement::default_for(kind, param.name, mode)).unwrap_or_else(|| param.default_value());
                    let (cells, vec2) = match value {
                        Value::Vec2([x, y]) => ([f(x), f(y), String::new()], true),
                        Value::F64(v) => ([String::new(), String::new(), f(v)], false),
                        _ => return None,
                    };
                    Some(PropRow { label: param.label.to_owned(), cells, dims: [false, false, false], keyed, property: Some(prop.name().to_owned()), vec2, value, range: param.range, axis: [None, None, None] })
                }).collect::<Vec<_>>();
                let mut rows = rows;
                // グループなら子が素材の袋。子ごとに重み(share)の行を足す。
                if view.meta(layer).ok().flatten().is_some_and(|m| m.source == crate::doc::store::LayerSource::Group) {
                    let mut kids: Vec<(i16, LayerId, String)> = view.layers().into_iter().filter_map(|c| {
                        let attrs = view.attrs(c).ok().flatten()?;
                        (attrs.parent == Some(layer)).then(|| (view.meta(c).ok().flatten().map_or(0, |m| m.order), c, attrs.name))
                    }).collect();
                    kids.sort_by_key(|k| (k.0, k.1));
                    for (_, child, name) in kids {
                        let Ok(prop) = PropertyId::effect_param(id, &format!("{}{}", crate::doc::extensions::placement::SHARE_PREFIX, child.0)) else { continue };
                        let v = match view.value_at(layer, &prop, t).ok().flatten() { Some(Value::F64(v)) => v, _ => crate::doc::extensions::placement::SHARE_DEFAULT };
                        let keyed = view.track(layer, &prop).ok().flatten().is_some();
                        let label = if name.is_empty() { format!("Layer {}", child.0) } else { name };
                        rows.push(PropRow { label, cells: [String::new(), String::new(), f(v)], dims: [false, false, false], keyed, property: Some(prop.name().to_owned()), vec2: false, value: Value::F64(v), range: Some((0.0, 1000.0)), axis: [None, None, None] });
                    }
                }
                return EffectBlock { id: id.0, plugin_id: instance.plugin_id, params: rows };
            }
            let params = catalog.iter().find(|d| d.plugin_id == instance.plugin_id).map(|d| d.params.as_slice()).unwrap_or(&[]);
            let rows = params
                .iter()
                .filter_map(move |param| {
                    let prop = PropertyId::effect_param(id, &param.name).ok()?;
                    let keyed = view.track(layer, &prop).ok().flatten().is_some();
                    let stored = view.value_at(layer, &prop, t).ok().flatten();
                    // 点の欄(Twist・Bend の Center)は 2 つの枡、数の欄は 1 つ。
                    // 色の欄は値の形(Color)だけで窓が hex の部品を出す。枡は使わない。
                    let (cells, vec2, value) = match (param.point, param.color, stored) {
                        // 層を指す欄: 値は LayerId(0 = 無し)。窓はカメラの target と同じ選択肢で描く。
                        (_, _, Some(Value::LayerId(id))) if param.layer => ([String::new(), String::new(), String::new()], false, Value::LayerId(id)),
                        (_, _, _) if param.layer => ([String::new(), String::new(), String::new()], false, Value::LayerId(0)),
                        (_, Some(_), Some(Value::Color(c))) => ([String::new(), String::new(), String::new()], false, Value::Color(c)),
                        (_, Some(c), _) => ([String::new(), String::new(), String::new()], false, Value::Color(c)),
                        (Some(_), _, Some(Value::Vec2([x, y]))) => ([f(x), f(y), String::new()], true, Value::Vec2([x, y])),
                        (Some([x, y]), _, _) => ([f(x), f(y), String::new()], true, Value::Vec2([x, y])),
                        (None, None, Some(Value::F64(v))) => ([String::new(), String::new(), f(v)], false, Value::F64(v)),
                        (None, None, _) => ([String::new(), String::new(), f(param.default)], false, Value::F64(param.default)),
                    };
                    Some(PropRow {
                        label: param.label.clone(),
                        cells,
                        dims: [false, false, false],
                        keyed,
                        property: Some(prop.name().to_owned()),
                        vec2,
                        value,
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
        Some(LayerSource::Camera) => "camera",
        Some(LayerSource::Stage) => "stage",
        Some(LayerSource::Shape) => "shape",
        Some(LayerSource::Text) => "text",
        Some(LayerSource::Group) => "group",
        Some(LayerSource::Particles) => "particles",
        None => "solid",
    };

    let text = match view.text_document(layer) {
        Ok(Some(doc)) => {
            let mut rows = vec![PropRow {
                label: names::label_or_id(names::TEXT_CONTENT).into_owned(),
                cells: [String::new(), String::new(), doc.content.eval(t).to_string()],
                dims: [false, false, false],
                keyed: doc.content.keys().iter().any(|k| k.t == t),
                property: None,
                vec2: false,
                value: Value::F64(0.0),
                range: None,
                axis: [None, None, None],
            }];
            rows.push(PropRow {
                label: names::label_or_id(names::TEXT_JUSTIFY).into_owned(), cells: Default::default(), dims: [false; 3],
                keyed: keyed(names::TEXT_JUSTIFY), property: Some(names::TEXT_JUSTIFY.into()),
                vec2: false, value: Value::Enum(doc.justify.to_enum_value()),
                range: None, axis: [None, None, None],
            });
            // 級数。style の size を property が上書きする(resolve と同じ順)。数の行なので擦れる。
            if let Some(style) = doc.styles.first() {
                let prop = PropertyId::text_style_size(style.id);
                let size = match view.value_at(layer, &prop, t).ok().flatten() {
                    Some(Value::F64(v)) => v,
                    _ => f64::from(style.size),
                };
                rows.push(PropRow {
                    label: names::label_or_id(prop.name()).into_owned(),
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
                for (prop, value, range) in [
                    (line_height_prop, line_height, (0.0, 1000.0)),
                    (tracking_prop, tracking, (-200.0, 200.0)),
                ] {
                    rows.push(PropRow {
                        label: names::label_or_id(prop.name()).into_owned(),
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
    // 形の元の値(星の頂点数・半径、矩形と楕円の大きさ)。書類の値が既定で、property が上書きする。
    let mut text = text;
    if let Ok(shapes) = view.shapes(layer) {
        for row in crate::doc::store::shape_props::rows(&shapes) {
            let Ok(prop) = PropertyId::new(row.name) else { continue };
            let value = view.value_at(layer, &prop, t).ok().flatten().unwrap_or(row.value);
            let (cells, vec2) = match value {
                Value::Vec2([x, y]) => ([f(x), f(y), String::new()], true),
                Value::F64(v) => ([String::new(), String::new(), f(v)], false),
                Value::Color(_) => (Default::default(), false),
                _ => continue,
            };
            text.push(PropRow { label: names::label_or_id(row.name).into_owned(), cells, dims: [false; 3], keyed: keyed(prop.name()), property: Some(prop.name().to_owned()), vec2, value, range: row.range, axis: [None, None, None] });
        }
    }

    // 文字の色は style の property。書類の style が既定で、property が上書きする(resolve と同じ順)。
    if let Ok(Some(doc)) = view.text_document(layer) {
        if let Some(style) = doc.styles.first() {
            let prop = PropertyId::text_style_fill_color(style.id);
            let value = match view.value_at(layer, &prop, t).ok().flatten() { Some(v @ Value::Color(_)) => v, _ => Value::Color(style.fill) };
            text.push(PropRow { label: names::label_or_id(prop.name()).into_owned(), cells: Default::default(), dims: [false; 3], keyed: keyed(prop.name()), property: Some(prop.name().to_owned()), vec2: false, value, range: None, axis: [None, None, None] });
        }
    }
    // 奥行きは板(形・文字・画・動画)だけ。網・点群は素材が奥行きを持ち、camera 等は絵が無い。
    let flat = matches!(source_name, "shape" | "text" | "media");
    let mut transform = vec![
        PropRow {
            label: names::label_or_id(property::POSITION).into_owned(),
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
            label: names::label_or_id(property::SCALE).into_owned(),
            cells: [f(scale_x), f(scale_y), f(scale_z)],
            dims: [false, false, false],
            keyed: keyed(property::SCALE),
            property: Some(property::SCALE.to_owned()),
            vec2: true,
            value: Value::Vec2([scale_x, scale_y]),
            range: None,
            axis: [None, None, Some((property::SCALE_Z.to_owned(), Value::F64(scale_z)))],
        },
        PropRow {
            label: names::label_or_id(property::ROTATION).into_owned(),
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
            label: names::label_or_id(property::OPACITY).into_owned(),
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
    if flat {
        transform.push(PropRow {
            label: names::label_or_id(property::DEPTH).into_owned(),
            cells: [String::new(), String::new(), f(depth_v)],
            dims: [false, false, false],
            keyed: keyed(property::DEPTH),
            property: Some(property::DEPTH.to_owned()),
            vec2: false,
            value: Value::F64(depth_v),
            range: Some((0.0, 100000.0)),
            axis: [None, None, None],
        });
    }

    InspectorData {
        blend: sel_attrs.blend_mode,
        ident_name: sel_attrs.name,
        ident_sub: format!("{source_name} · {key_count} keys"),
        text,
        transform,
        effects,
        has_effects,
    }
}

/// 12.3 MB の様な人向けの容量。
pub(crate) fn human_size(bytes: u64) -> String {
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

pub(crate) fn fmt_timecode(sec: f64, fps: crate::doc::store::Fps) -> String {
    let at = crate::doc::store::RationalTime::try_new((sec.max(0.0) * 1_000_000.0).round() as i64, 1_000_000)
        .unwrap_or(crate::doc::store::RationalTime::ZERO);
    let frame = at.try_to_frame_round(fps).unwrap_or(0);
    let nominal = fps.as_f64().round().max(1.0) as i64;
    format!("{}:{:02}:{:02}", frame / (nominal * 60), (frame / nominal) % 60, frame % nominal)
}
