use motolii_store::{property, Document, LayerId, LayerSource, PropertyId, RationalTime, StoreView, Value};

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

pub struct AssetRow {
    pub name: String,
    pub kind: String,
    pub thumb: &'static str,
}

/// Inspector 1行。cells[i]が空文字のセルは地なし(モックの空白セルと同じ)。
pub struct PropRow {
    pub label: &'static str,
    pub cells: [String; 3],
    pub dims: [bool; 3],
    pub keyed: bool,
    /// SetTrackの宛先。Noneなら編集不可(EFFECTS等のダミー行)。
    pub property: Option<&'static str>,
    /// trueならcells[0]/[1]がVec2のx/y(cells[2]は飾り)。falseならcells[2]が唯一の値。
    pub vec2: bool,
    /// cellsを組んだ生の値。ドラッグ開始点・キー打刻の両方がここから読む。
    pub value: Value,
}

pub struct InspectorData {
    pub ident_name: String,
    pub ident_sub: String,
    pub text: Vec<PropRow>,
    pub transform: Vec<PropRow>,
    pub has_effects: bool,
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

    let sel_attrs = view.attrs(layer).ok().flatten().unwrap_or_default();
    let key_count: usize = [property::POSITION, property::OPACITY]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .filter_map(|p| view.track(layer, &p).ok().flatten())
        .map(|tr| tr.keys().len())
        .sum();
    let has_effects = !view.effects(layer).unwrap_or_default().is_empty();
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
        }],
        _ => Vec::new(),
    };

    // AEの層Transform並び: Anchor Point → Position → Scale → Rotation → Opacity。
    let transform = vec![
            PropRow {
                label: "Anchor",
                cells: [f(anchor_x), f(anchor_y), f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::ANCHOR),
                property: Some(property::ANCHOR),
                vec2: true,
                value: Value::Vec2([anchor_x, anchor_y]),
            },
            PropRow {
                label: "Position",
                cells: [px, py, f(0.0)],
                dims: [false, false, true],
                keyed: keyed(property::POSITION),
                property: Some(property::POSITION),
                vec2: true,
                value: Value::Vec2([pos_x, pos_y]),
            },
            PropRow {
                label: "Scale",
                cells: [f(scale_x), f(scale_y), f(1.0)],
                dims: [false, false, true],
                keyed: keyed(property::SCALE),
                property: Some(property::SCALE),
                vec2: true,
                value: Value::Vec2([scale_x, scale_y]),
            },
            PropRow {
                label: "Rotation",
                cells: [String::new(), String::new(), f(rotation_v)],
                dims: [false, false, true],
                keyed: keyed(property::ROTATION),
                property: Some(property::ROTATION),
                vec2: false,
                value: Value::F64(rotation_v),
            },
            PropRow {
                label: "Opacity",
                cells: [String::new(), String::new(), opacity],
                dims: [false, false, false],
                keyed: keyed(property::OPACITY),
                property: Some(property::OPACITY),
                vec2: false,
                value: Value::F64(opacity_v),
            },
    ];

    InspectorData {
        ident_name: sel_attrs.name,
        ident_sub: format!("{source_name} · {key_count} keys"),
        text,
        transform,
        has_effects,
    }
}

pub fn load_fixture() -> Loaded {
    let fx = motolii_fixture::build();
    let view = fx.doc.view();

    let layer_rows = layer_rows_from_doc(&fx.doc);

    let assets = view
        .assets()
        .unwrap_or_default()
        .into_iter()
        .enumerate()
        .map(|(i, a)| AssetRow {
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
