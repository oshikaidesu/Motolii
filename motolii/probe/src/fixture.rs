use motolii_store::{property, Document, LayerId, LayerSource, PropertyId, RationalTime, ShapeNode, StoreView, Value};

use motolii_engine::known_effects;

use crate::timeline_widget::CanvasRow;

const FPS: f64 = 30.0;

/// LayerId % 12 → 差し色。長さはmotolii_tokens_rs::LABEL_PALETTE_LENと同じ12。
pub const LABEL_PALETTE: [&str; 12] = [
    "#d96b6b", "#d9985a", "#d9c95a", "#a3d95a", "#5ad98c", "#5ac6c6",
    "#5a96d9", "#7d7dd9", "#a86bd9", "#d96bbc", "#9a9a9a", "#cba97a",
];

/// Documentから行を読む。ドラッグcommit後の再読みにも同じ関数を使う。
pub fn canvas_rows_from_doc(doc: &Document) -> Vec<CanvasRow> {
    let view = doc.view();
    let props: Vec<PropertyId> = [property::OPACITY, property::POSITION]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .collect();

    let mut layers = view.layers();
    layers.sort_by_key(|l| std::cmp::Reverse(view.meta(*l).ok().flatten().map(|m| m.order).unwrap_or(0)));

    layers
        .into_iter()
        .map(|layer| {
            let (start, duration) = view
                .meta(layer)
                .ok()
                .flatten()
                .map(|m| (m.timing.start, m.timing.duration))
                .unwrap_or((0, 0));
            let color_ix = view
                .attrs(layer)
                .ok()
                .flatten()
                .and_then(|a| a.label_color)
                .unwrap_or(10);
            let mut keys = Vec::new();
            for prop in &props {
                if let Ok(Some(track)) = view.track(layer, prop) {
                    keys.extend(track.keys().iter().map(|k| k.t.as_seconds_f64()));
                }
            }
            keys.sort_by(|a, b| a.partial_cmp(b).unwrap());
            CanvasRow {
                is_group: false,
                keys,
                span: Some((start as f64 / FPS, (start + duration) as f64 / FPS)),
                agg: Vec::new(),
                layer: Some(layer),
                color: label_rgb(color_ix),
            }
        })
        .collect()
}

pub fn label_rgb(ix: u8) -> [u8; 3] {
    let hex = LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()];
    let v = u32::from_str_radix(&hex[1..], 16).unwrap_or(0x8c8c8c);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

#[derive(Clone)]
pub struct LayerRow {
    pub layer: LayerId,
    pub name: String,
    pub color: &'static str,
    pub hidden: bool,
    pub solo: bool,
    pub locked: bool,
}

/// Documentから層行を読む。M/S/Lクリック後の再読みにも同じ関数を使う。
pub fn layer_rows_from_doc(doc: &Document) -> Vec<LayerRow> {
    let view = doc.view();
    let mut layers = view.layers();
    layers.sort_by_key(|l| std::cmp::Reverse(view.meta(*l).ok().flatten().map(|m| m.order).unwrap_or(0)));

    layers
        .into_iter()
        .map(|layer| {
            let attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
            let color = attrs
                .label_color
                .map(|ix| LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()])
                .unwrap_or("#8c8c8c");
            LayerRow {
                layer,
                name: attrs.name,
                color,
                hidden: attrs.hidden,
                solo: attrs.solo,
                locked: attrs.locked,
            }
        })
        .collect()
}

/// この作品で使われている色1つ。標準パレットではなく、層を走査して集めた実測値。
pub struct ColorSwatch {
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
                if let motolii_vector::Brush::Solid(rgb) = &fill.brush {
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

/// 作品全体を走査して使われている色を集める。固定標準パレットは作らない(裁定済み)。
pub fn used_colors_from_doc(doc: &Document) -> Vec<ColorSwatch> {
    let view = doc.view();
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for layer in view.layers() {
        if let Some(LayerSource::Solid { rgba, .. }) = view.meta(layer).ok().flatten().map(|m| m.source) {
            push_swatch(&mut seen, &mut out, rgba);
        }
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

pub struct AssetRow {
    pub name: String,
    pub kind: String,
    pub thumb: &'static str,
    pub family: AssetFamily,
    pub path: Option<String>,
    /// 素材の実画。ffmpeg起動やデコードを伴うので**台帳を読む時に1回だけ**作る。
    pub preview: Option<String>,
}

/// 拡張子の区分の正本は`re_importer`の`SUPPORTED_*_EXTENSIONS`。
/// coreは`asset_type`を解釈しないのでこの畳み込みはfrontが持つ。
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum AssetFamily {
    Video,
    TwoD,
    ThreeD,
    Audio,
    Data,
    Other,
}

impl AssetFamily {
    pub fn label(self) -> &'static str {
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

pub fn asset_family(asset_type: &str) -> AssetFamily {
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

/// Inspector 1行。cells[i]が空文字のセルは地なし(モックの空白セルと同じ)。
pub struct PropRow {
    pub label: &'static str,
    pub cells: [String; 3],
    pub dims: [bool; 3],
    pub keyed: bool,
    /// SetTrackの宛先。Noneなら編集不可(EFFECTS等のダミー行)。
    pub property: Option<String>,
    /// trueならcells[0]/[1]がVec2のx/y(cells[2]は飾り)。falseならcells[2]が唯一の値。
    pub vec2: bool,
    /// cellsを組んだ生の値。ドラッグ開始点・キー打刻の両方がここから読む。
    pub value: Value,
    /// 宣言された範囲(engine側 EffectParamDescriptor::range)。無ければドラッグは無限。
    pub range: Option<(f64, f64)>,
    /// cells[2]が独立したpropertyを持つ行(Positionのz)。Noneならcells[2]は飾り。
    pub z: Option<(String, Value)>,
}

pub struct InspectorData {
    pub ident_name: String,
    pub ident_sub: String,
    pub text: Vec<PropRow>,
    pub transform: Vec<PropRow>,
    pub effects: Vec<PropRow>,
    pub has_effects: bool,
    /// (ラベル, "#rrggbb")。property=Noneで読み取り専用 — 編集の入口はBrowserのColorsタブ。
    pub colors: Vec<(&'static str, String)>,
}

pub struct UiData {
    pub layer_rows: Vec<LayerRow>,
    pub assets: Vec<AssetRow>,
    pub comp_line: String,
    pub status: String,
}

pub struct Loaded {
    pub doc: Document,
    pub ui: UiData,
    pub duration_sec: f64,
}

/// 選択層+comp時刻の断面をInspector表示用に読む。S2+S4の繋ぎ先(session::Selectionが選ぶlayer)。
pub fn inspector_data_from_doc(view: &StoreView, layer: LayerId, t: RationalTime) -> InspectorData {
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

    // comp px級の値は38px幅セルに3桁小数が収まらないので1桁へ落とす。
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
    let (anchor_x, anchor_y) = match value_of(property::ANCHOR) {
        Some(Value::Vec2([x, y])) => (x, y),
        _ => (0.0, 0.0),
    };
    let rotation_v = match value_of(property::ROTATION) {
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
    // 層に付いている effect ごとに、known_effects() の宣言(名前・既定値・範囲)で
    // Inspector 行を組む。宣言に無い plugin_id(known_effects() の外)は無視——
    // 「無い範囲を発明しない」(Q0)と同じ fail-closed。
    let effects: Vec<PropRow> = attached_effects
        .into_iter()
        .flat_map(|instance| {
            let params = known_effects()
                .iter()
                .find(|d| d.plugin_id == instance.plugin_id)
                .map(|d| d.params)
                .unwrap_or(&[]);
            params
                .iter()
                .filter_map(move |param| {
                    let prop = PropertyId::effect_param(instance.id, param.name).ok()?;
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
                        z: None,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let source_name = match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::Solid { .. }) => "solid",
        Some(LayerSource::Media { .. }) => "media",
        Some(LayerSource::PointCloud { .. }) => "point cloud",
        Some(LayerSource::Null) => "null",
        Some(LayerSource::Shape) => "shape",
        Some(LayerSource::Text) => "text",
        Some(LayerSource::Group) => "group",
        None => "solid",
    };

    // TextDocumentのうちcontentだけが時間変化する(text.rs:462-466)。
    // Ok(None)はまだ一度も書かれていない層 — text層でも起こりうる。
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
            z: None,
        }],
        _ => Vec::new(),
    };

    // 色は素材側の静的値であってproperty(track)ではないので、行はここで別に組む。
    // property=None=読み取り専用 — 編集の入口はBrowserのColorsタブ(裁定済み)。
    let mut colors: Vec<(&'static str, String)> = Vec::new();
    match view.meta(layer).ok().flatten().map(|m| m.source) {
        Some(LayerSource::Solid { rgba, .. }) => colors.push(("Color", hex_of(rgba))),
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

    // AEの層Transform並び: Anchor Point → Position → Scale → Rotation → Opacity。
    let transform = vec![
            PropRow {
                label: "Anchor",
                cells: [f(anchor_x), f(anchor_y), f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::ANCHOR),
                property: Some(property::ANCHOR.to_owned()),
                vec2: true,
                value: Value::Vec2([anchor_x, anchor_y]),
                range: None,
                z: None,
            },
            PropRow {
                label: "Position",
                cells: [px, py, f1(pos_z)],
                dims: [false, false, false],
                keyed: keyed(property::POSITION),
                property: Some(property::POSITION.to_owned()),
                vec2: true,
                value: Value::Vec2([pos_x, pos_y]),
                range: None,
                z: Some((property::POSITION_Z.to_owned(), Value::F64(pos_z))),
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
                z: None,
            },
            PropRow {
                label: "Rotation",
                cells: [String::new(), String::new(), f(rotation_v)],
                dims: [false, false, true],
                keyed: keyed(property::ROTATION),
                property: Some(property::ROTATION.to_owned()),
                vec2: false,
                value: Value::F64(rotation_v),
                range: None,
                z: None,
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
                z: None,
            },
    ];

    InspectorData {
        ident_name: sel_attrs.name,
        ident_sub: format!("{source_name} · {key_count} keys"),
        colors,
        text,
        transform,
        effects,
        has_effects,
    }
}

/// `MOTOLII_TESTDATA` のディレクトリから、rerunが読める物を素材台帳へ入れる。
/// 実素材で面を見るための器具で、fixtureと同じ`Intent::AdmitAsset`経路を通る。
fn admit_testdata(doc: &mut motolii_store::Document) {
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
            .and_then(motolii_media::asset_type_for_extension)
        else {
            continue;
        };
        let Ok(reader) = std::fs::File::open(&path) else {
            continue;
        };
        let Ok(fingerprint) = motolii_store::SourceFingerprintV1::from_reader(reader) else {
            continue;
        };
        drafts.push(motolii_store::AssetDraft::from_probed_source(
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
        .map(|draft| motolii_store::Intent::AdmitAsset { draft })
        .collect();
    if let Err(e) = doc.apply_all(intents) {
        println!("PROBE room=browser verdict=admit-error {e}");
    }
}

pub fn load_fixture() -> Loaded {
    let mut fx = motolii_fixture::build();
    admit_testdata(&mut fx.doc);
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
                    AssetFamily::TwoD => crate::thumbnail::image_data_uri(path),
                    AssetFamily::Video => crate::thumbnail::video_data_uri(path),
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

pub fn fmt_timecode(sec: f64) -> String {
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
