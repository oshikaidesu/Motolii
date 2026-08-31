use crate::doc::store::{property, Document, LayerId, LayerSource, PropertyId, RationalTime, ShapeNode, StoreView, Value};

use crate::render::engine::known_effects;

use crate::ui::timeline_widget::CanvasRow;

const FPS: f64 = 30.0;

pub(super) const LABEL_PALETTE: [&str; 12] = [
    "#d96b6b", "#d9985a", "#d9c95a", "#a3d95a", "#5ad98c", "#5ac6c6",
    "#5a96d9", "#7d7dd9", "#a86bd9", "#d96bbc", "#9a9a9a", "#cba97a",
];

pub(super) fn label_rgb(ix: u8) -> [u8; 3] {
    let hex = LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()];
    let v = u32::from_str_radix(&hex[1..], 16).unwrap_or(0x8c8c8c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

#[derive(Clone)]
pub(super) struct LayerRow {
    pub layer: LayerId,
    pub name: String,
    pub color: &'static str,
    pub hidden: bool,
    pub solo: bool,
    pub locked: bool,
    /// 属性の行なら属性名。層そのものの行なら None。
    pub prop: Option<String>,
    pub expanded: bool,
    /// 入れ子の深さ。0 が親を持たない層。
    pub depth: u16,
    /// 直接の子の数。畳んだ時に「中に何枚居るか」を出すため。
    pub children: u16,
}

const TRANSFORM_PROPS: &[&str] = &[
    property::ANCHOR,
    property::POSITION,
    property::SCALE,
    property::ROTATION,
    property::OPACITY,
];

#[derive(Default)]
struct TimelineView {
    expanded: std::collections::BTreeSet<LayerId>,
    keyed_only: bool,
}

fn timeline_view() -> &'static std::sync::Mutex<TimelineView> {
    static V: std::sync::OnceLock<std::sync::Mutex<TimelineView>> = std::sync::OnceLock::new();
    V.get_or_init(|| std::sync::Mutex::new(TimelineView::default()))
}

pub(super) fn toggle_expanded(layer: LayerId) {
    let mut v = timeline_view().lock().unwrap();
    if !v.expanded.remove(&layer) {
        v.expanded.insert(layer);
    }
}

pub(super) fn expand(layer: LayerId) {
    timeline_view().lock().unwrap().expanded.insert(layer);
}

pub(super) fn toggle_keyed_only() {
    let mut v = timeline_view().lock().unwrap();
    v.keyed_only = !v.keyed_only;
}

pub(super) fn keyed_only() -> bool {
    timeline_view().lock().unwrap().keyed_only
}

/// 画面に出る行の並び。左の名前列と右の帯は必ずこれを通す(ずれると別物になる)。
struct Row {
    layer: LayerId,
    prop: Option<PropertyId>,
    depth: u16,
    children: u16,
}

fn rows_of(doc: &Document) -> Vec<Row> {
    let (expanded, keyed_only) = {
        let v = timeline_view().lock().unwrap();
        (v.expanded.clone(), v.keyed_only)
    };
    rows_nested(doc, &expanded, keyed_only)
}

fn rows_nested(
    doc: &Document,
    expanded: &std::collections::BTreeSet<LayerId>,
    keyed_only: bool,
) -> Vec<Row> {
    let view = doc.view();
    let mut layers = view.layers();
    layers.sort_by_key(|l| std::cmp::Reverse(view.meta(*l).ok().flatten().map(|m| m.order).unwrap_or(0)));

    let present: std::collections::HashSet<LayerId> = layers.iter().copied().collect();
    let parent_of = |l: LayerId| {
        view.attrs(l)
            .ok()
            .flatten()
            .and_then(|a| a.parent)
            .filter(|p| present.contains(p) && *p != l)
    };

    let mut out = Vec::new();
    let mut stack: Vec<(LayerId, u16)> = layers
        .iter()
        .rev()
        .filter(|l| parent_of(**l).is_none())
        .map(|l| (*l, 0))
        .collect();

    while let Some((layer, depth)) = stack.pop() {
        let children = layers.iter().filter(|c| parent_of(**c) == Some(layer)).count() as u16;
        out.push(Row { layer, prop: None, depth, children });
        if !expanded.contains(&layer) {
            continue;
        }
        for name in TRANSFORM_PROPS {
            let Ok(property) = PropertyId::new(name) else {
                continue;
            };
            let keyed = matches!(view.track(layer, &property), Ok(Some(t)) if !t.keys().is_empty());
            if keyed_only && !keyed {
                continue;
            }
            out.push(Row { layer, prop: Some(property), depth, children });
        }
        stack.extend(
            layers
                .iter()
                .rev()
                .filter(|c| parent_of(**c) == Some(layer))
                .map(|c| (*c, depth + 1)),
        );
    }
    out
}

pub(super) fn canvas_rows_from_doc(doc: &Document) -> Vec<CanvasRow> {
    let view = doc.view();
    let agg_props: Vec<PropertyId> = [property::OPACITY, property::POSITION]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .collect();

    rows_of(doc)
        .into_iter()
        .map(|Row { layer, prop, .. }| {
            let color_ix = view
                .attrs(layer)
                .ok()
                .flatten()
                .and_then(|a| a.label_color)
                .unwrap_or(10);
            match prop {
                None => {
                    let (start, duration) = view
                        .meta(layer)
                        .ok()
                        .flatten()
                        .map(|m| (m.timing.start, m.timing.duration))
                        .unwrap_or((0, 0));
                    let mut keys = Vec::new();
                    for property in &agg_props {
                        if let Ok(Some(track)) = view.track(layer, property) {
                            keys.extend(track.keys().iter().map(|k| k.t.as_seconds_f64()));
                        }
                    }
                    CanvasRow {
                        is_group: false,
                        keys,
                        span: Some((start as f64 / FPS, (start + duration) as f64 / FPS)),
                        agg: Vec::new(),
                        layer: Some(layer),
                        prop: None,
                        color: label_rgb(color_ix),
                    }
                }
                Some(property) => {
                    let keys = view
                        .track(layer, &property)
                        .ok()
                        .flatten()
                        .map(|t| t.keys().iter().map(|k| k.t.as_seconds_f64()).collect())
                        .unwrap_or_default();
                    CanvasRow {
                        is_group: false,
                        keys,
                        span: None,
                        agg: Vec::new(),
                        layer: Some(layer),
                        prop: Some(property),
                        color: label_rgb(color_ix),
                    }
                }
            }
        })
        .collect()
}

pub(super) fn layer_rows_from_doc(doc: &Document) -> Vec<LayerRow> {
    let view = doc.view();
    rows_of(doc)
        .into_iter()
        .map(|Row { layer, prop, depth, children }| {
            let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
            let color = attrs
                .label_color
                .map(|ix| LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()])
                .unwrap_or("#8c8c8c");
            let expanded = timeline_view().lock().unwrap().expanded.contains(&layer);
            LayerRow {
                layer,
                name: match &prop {
                    Some(p) => p.name().to_string(),
                    None => attrs.name,
                },
                color,
                hidden: attrs.hidden,
                solo: attrs.solo,
                locked: attrs.locked,
                prop: prop.map(|p| p.name().to_string()),
                expanded,
                depth,
                children,
            }
        })
        .collect()
}

pub(super) struct ColorSwatch {
    pub hex: String,
    pub rgba: [u8; 4],
}

fn hex_of(rgba: [u8; 4]) -> String {
    format!("#{:02x}{:02x}{:02x}", rgba[0], rgba[1], rgba[2])
}

fn push_swatch(seen: &mut std::collections::BTreeSet<[u8; 4]>, out: &mut Vec<ColorSwatch>, rgba: [u8; 4]) {
    if seen.insert(rgba) {
        out.push(ColorSwatch { hex: hex_of(rgba), rgba });
    }
}

fn shape_fill_colors(node: &ShapeNode, seen: &mut std::collections::BTreeSet<[u8; 4]>, out: &mut Vec<ColorSwatch>) {
    match node {
        ShapeNode::Leaf(shape) => {
            if let Some(fill) = &shape.fill {
                if let crate::render::vector::Brush::Solid(rgb) = &fill.brush {
                    push_swatch(seen, out, [(rgb.r * 255.0) as u8, (rgb.g * 255.0) as u8, (rgb.b * 255.0) as u8, 255]);
                }
            }
        }
        ShapeNode::Group(group) => {
            for child in &group.children {
                shape_fill_colors(child, seen, out);
            }
        }
    }
}

pub(super) fn used_colors_from_doc(doc: &Document) -> Vec<ColorSwatch> {
    let view = doc.view();
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for layer in view.layers() {
        if let Ok(Some(text)) = view.text_document(layer) {
            for style in &text.styles {
                let f = style.fill;
                push_swatch(&mut seen, &mut out, [(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8, (f[3] * 255.0) as u8]);
                if let Some(s) = style.stroke_color {
                    push_swatch(&mut seen, &mut out, [(s[0] * 255.0) as u8, (s[1] * 255.0) as u8, (s[2] * 255.0) as u8, (s[3] * 255.0) as u8]);
                }
            }
        }
        if let Ok(shapes) = view.shapes(layer) {
            for node in &shapes {
                shape_fill_colors(node, &mut seen, &mut out);
            }
        }
    }
    out
}

pub(super) struct AssetRow {
    pub name: String,
    pub kind: String,
    pub thumb: &'static str,
    pub family: AssetFamily,
    pub path: Option<String>,
    pub preview: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(super) enum AssetFamily {
    Video,
    TwoD,
    ThreeD,
    Audio,
    Data,
    Other,
}

impl AssetFamily {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::TwoD => "2D",
            Self::ThreeD => "3D",
            Self::Audio => "Audio",
            Self::Data => "Data",
            Self::Other => "Other",
        }
    }
}

pub(super) fn asset_family(asset_type: &str) -> AssetFamily {
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

pub(super) struct PropRow {
    pub label: &'static str,
    pub cells: [String; 3],
    pub dims: [bool; 3],
    pub keyed: bool,
    pub property: Option<String>,
    pub vec2: bool,
    pub value: Value,
    pub range: Option<(f64, f64)>,
    pub axis: [Option<(String, Value)>; 3],
}

/// 1つのエフェクトと、その param 行。**エフェクト自体を外せる**ように、
/// 名前と id を param と一緒に持つ。
pub(super) struct EffectBlock {
    pub id: u32,
    pub plugin_id: String,
    pub params: Vec<PropRow>,
}

pub(super) struct InspectorData {
    pub ident_name: String,
    /// 層の合成モード。値は Document が持つ(表示用の写しをここへ運ぶだけ)。
    pub blend: crate::doc::store::BlendMode,
    pub ident_sub: String,
    pub text: Vec<PropRow>,
    pub transform: Vec<PropRow>,
    pub effects: Vec<EffectBlock>,
    pub has_effects: bool,
    pub colors: Vec<(&'static str, String)>,
}

pub(super) struct UiData {
    pub layer_rows: Vec<LayerRow>,
    pub assets: Vec<AssetRow>,
    pub comp_line: String,
    pub status: String,
}

pub(super) struct Loaded {
    pub doc: Document,
    pub ui: UiData,
    pub duration_sec: f64,
}

pub(super) fn inspector_data_from_doc(view: &StoreView, layer: LayerId, t: RationalTime) -> InspectorData {
    let value_of = |prop: &str| {
        PropertyId::new(prop)
            .ok()
            .and_then(|p| view.value_at(layer, &p, t).ok().flatten())
    };
    let keyed = |prop: &str| {
        PropertyId::new(prop)
            .ok()
            .and_then(|p| view.track(layer, &p).ok().flatten())
            .is_some()
    };
    let f = |v: f64| format!("{v:.3}");

    let f1 = |v: f64| format!("{v:.1}");
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
            let params = known_effects()
                .iter()
                .find(|d| d.plugin_id == instance.plugin_id)
                .map(|d| d.params)
                .unwrap_or(&[]);
            let id = instance.id;
            let rows = params
                .iter()
                .filter_map(move |param| {
                    let prop = PropertyId::effect_param(id, param.name).ok()?;
                    let keyed = view.track(layer, &prop).ok().flatten().is_some();
                    let v = match view.value_at(layer, &prop, t).ok().flatten() {
                        Some(Value::F64(v)) => v,
                        _ => param.default,
                    };
                    Some(PropRow {
                        label: param.name,
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
            EffectBlock {
                id: instance.id.0,
                plugin_id: instance.plugin_id,
                params: rows,
            }
        })
        .collect();
    let source_name = match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::File { path, .. }) => {
            if crate::render::media::is_point_cloud_path(&path) {
                "point cloud"
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
        Ok(Some(doc)) => vec![PropRow {
            label: "Content",
            cells: [String::new(), String::new(), doc.content.eval(t).to_string()],
            dims: [false, false, false],
            keyed: doc.content.keys().len() > 1,
            property: None,
            vec2: false,
            value: Value::F64(0.0),
            range: None,
            axis: [None, None, None],
        }],
        _ => Vec::new(),
    };

    let mut colors: Vec<(&'static str, String)> = Vec::new();
    match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::Text) => {
            if let Ok(Some(doc)) = view.text_document(layer) {
                if let Some(style) = doc.styles.first() {
                    let f = style.fill;
                    colors.push(("Fill", hex_of([(f[0] * 255.0) as u8, (f[1] * 255.0) as u8, (f[2] * 255.0) as u8, (f[3] * 255.0) as u8])));
                    if let Some(s) = style.stroke_color {
                        colors.push(("Stroke", hex_of([(s[0] * 255.0) as u8, (s[1] * 255.0) as u8, (s[2] * 255.0) as u8, (s[3] * 255.0) as u8])));
                    }
                }
            }
        }
        Some(LayerSource::Shape) => {
            if let Ok(shapes) = view.shapes(layer) {
                let mut seen = std::collections::BTreeSet::new();
                let mut swatches = Vec::new();
                for node in &shapes {
                    shape_fill_colors(node, &mut seen, &mut swatches);
                }
                if let Some(sw) = swatches.into_iter().next() {
                    colors.push(("Fill", sw.hex));
                }
            }
        }
        _ => {}
    }

    let transform = vec![
            PropRow {
                label: "Position",
                cells: [px, py, f1(pos_z)],
                dims: [false, false, false],
                keyed: keyed(property::POSITION),
                property: Some(property::POSITION.to_owned()),
                vec2: true,
                value: Value::Vec2([pos_x, pos_y]),
                range: None,
                axis: [
                    None,
                    None,
                    Some((property::POSITION_Z.to_owned(), Value::F64(pos_z))),
                ],
            },
            PropRow {
                label: "Scale",
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
                label: "Rotation",
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
                label: "Opacity",
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

fn admit_testdata(doc: &mut crate::doc::store::Document) {
    let Some(dir) = std::env::var_os("MOTOLII_TESTDATA") else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(std::path::PathBuf::from(dir)) else {
        return;
    };
    let mut drafts = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(asset_type) = path
            .extension()
            .and_then(|e| e.to_str())
            .and_then(crate::render::media::asset_type_for_extension)
        else {
            continue;
        };
        let Ok(reader) = std::fs::File::open(&path) else {
            continue;
        };
        let Ok(fingerprint) = crate::doc::store::SourceFingerprintV1::from_reader(reader) else {
            continue;
        };
        drafts.push(crate::doc::store::AssetDraft::from_probed_source(
            asset_type,
            &fingerprint,
            &path,
            None,
        ));
    }
    if drafts.is_empty() {
        return;
    }
    let intents: Vec<_> = drafts
        .into_iter()
        .map(|draft| crate::doc::store::Intent::AdmitAsset { draft })
        .collect();
    if let Err(e) = doc.apply_all(intents) {
        println!("PROBE room=browser verdict=admit-error {e}");
    }
}

pub(super) fn load_fixture() -> Loaded {
    let mut fx = crate::doc::fixture::build();
    admit_testdata(&mut fx.doc);
    if let Some(deg) = std::env::var("MOTOLII_TILT").ok().and_then(|v| v.parse::<f64>().ok()) {
        let layers = fx.doc.view().layers();
        let intents: Vec<_> = layers
            .iter()
            .filter_map(|l| {
                let mut track = crate::doc::store::KeyframeTrack::new();
                track.insert(crate::doc::store::Keyframe {
                    t: crate::doc::store::RationalTime::ZERO,
                    value: crate::doc::store::Value::F64(deg),
                    interp: crate::doc::store::Interp::Linear,
                    spatial: None,
                });
                crate::doc::store::PropertyId::new(crate::doc::store::property::ROTATION_X)
                    .ok()
                    .map(|property| crate::doc::store::Intent::SetTrack { layer: *l, property, track })
            })
            .collect();
        match fx.doc.apply_all(intents) {
            Ok(_) => println!("PROBE room=stage verdict=tilt-seeded deg={deg} layers={}", layers.len()),
            Err(e) => println!("PROBE room=stage verdict=tilt-seed-error {e}"),
        }
    }
    let view = fx.doc.view();

    let layer_rows = layer_rows_from_doc(&fx.doc);

    let assets = view
        .assets()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, a)| AssetRow {
            family: asset_family(&a.asset_type),
            preview: a.path_absolute.as_deref().and_then(|path| {
                match asset_family(&a.asset_type) {
                    AssetFamily::TwoD => crate::ui::thumbnail::image_data_uri(path),
                    AssetFamily::Video => crate::ui::thumbnail::video_data_uri(path),
                    _ => None,
                }
            }),
            path: a.path_absolute,
            name: a.name,
            kind: a.asset_type,
            thumb: ["#6f8fb5", "#8f7fb8", "#6fb58a", "#b59a6f"][i % 4],
        })
        .collect();

    let comp_line = view
        .composition()
        .ok()
        .flatten()
        .map(|c| {
            format!(
                "{}×{} · {}fps · {}s",
                c.width,
                c.height,
                c.fps.num(),
                c.duration_frames / c.fps.num()
            )
        })
        .unwrap_or_default();

    drop(view);

    Loaded {
        doc: fx.doc,
        ui: UiData { layer_rows, assets, comp_line, status: fx.status },
        duration_sec: 60.0,
    }
}

pub(super) fn fmt_timecode(sec: f64) -> String {
    let f = (sec * FPS).round() as i64;
    format!("{}:{:02}:{:02}", f / 1800, (f / 30) % 60, f % 30)
}

#[cfg(test)]
mod asset_family_folding {
    use super::{asset_family, AssetFamily};

    #[test]
    fn point_clouds_and_meshes_are_one_family() {
        assert_eq!(asset_family("pointcloud.octree.v1"), AssetFamily::ThreeD);
        assert_eq!(asset_family("model/gltf-binary"), AssetFamily::ThreeD);
    }

    #[test]
    fn images_are_2d_and_video_is_its_own() {
        assert_eq!(asset_family("image/png"), AssetFamily::TwoD);
        assert_eq!(asset_family("image/svg+xml"), AssetFamily::TwoD);
        assert_eq!(asset_family("video/mp4"), AssetFamily::Video);
    }

    #[test]
    fn unknown_types_fall_through_instead_of_getting_their_own_box() {
        assert_eq!(asset_family("wat/unknown"), AssetFamily::Other);
    }
}

#[cfg(test)]
mod row_tests {
    use super::*;
    use crate::doc::store::{Composition, Fps, Intent, LayerMeta, LayerTiming};

    fn doc_with_layers(n: u32) -> Document {
        let mut doc = Document::new();
        doc.apply(Intent::SetComposition(Composition {
            width: 64,
            height: 64,
            fps: Fps::try_new(30, 1).unwrap(),
            duration_frames: 300,
            background: [0.0, 0.0, 0.0, 1.0],
        }))
        .unwrap();
        for i in 0..n {
            let layer = LayerId(i as u64 + 1);
            doc.apply_all([
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::Shape,
                        order: i as i16,
                        timing: LayerTiming::place(0, None, 100),
                    },
                },
            ])
            .unwrap();
        }
        doc
    }

    #[test]
    fn the_name_column_and_the_band_column_describe_the_same_rows() {
        let doc = doc_with_layers(3);
        toggle_expanded(LayerId(2));

        let names = layer_rows_from_doc(&doc);
        let bands = canvas_rows_from_doc(&doc);

        assert_eq!(names.len(), bands.len(), "左右の行数がずれている");
        for (name, band) in names.iter().zip(&bands) {
            assert_eq!(Some(name.layer), band.layer);
            assert_eq!(name.prop.is_some(), band.prop.is_some());
        }
        assert!(
            names.iter().any(|r| r.prop.is_some()),
            "開いた層の属性行が1つも出ていない"
        );

        toggle_expanded(LayerId(2));
        assert!(layer_rows_from_doc(&doc).iter().all(|r| r.prop.is_none()));
    }
}

#[cfg(test)]
mod nest_tests {
    use super::*;

    /// 裁定: グループは層で、Timeline は入れ子で出す(2026-08-31)。
    #[test]
    fn a_child_comes_under_its_parent_one_step_in_and_hides_when_folded() {
        let doc = crate::doc::fixture::build().doc;
        let named = |rows: &[Row], name: &str| {
            let view = doc.view();
            rows.iter().position(|r| {
                r.prop.is_none()
                    && view.attrs(r.layer).ok().flatten().map(|a| a.name).as_deref() == Some(name)
            })
        };

        let folded = rows_nested(&doc, &Default::default(), false);
        let parent_ix = named(&folded, "ダンスカット").expect("親の行がある");
        assert!(named(&folded, "グリッチトランジション").is_none(), "畳んだ親の子が出ている");

        let parent = folded[parent_ix].layer;
        let opened = rows_nested(&doc, &[parent].into_iter().collect(), false);
        let p = named(&opened, "ダンスカット").expect("親の行がある");
        let c = named(&opened, "グリッチトランジション").expect("開いたのに子が出ない");
        assert!(c > p, "子が親より上に居る");
        assert_eq!(opened[c].depth, opened[p].depth + 1, "子が一段内側に居ない");
    }
}

