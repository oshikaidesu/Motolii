pub(super) use crate::ui::functions::read::{
    asset_family, first_shape_fill, fmt_timecode, human_size, label_rgb,
};
use crate::ui::functions::read::hex_of;

use crate::doc::store::{
    property, Document, LayerId, PropertyId, ShapeNode, StoreView, Value,
};

use crate::ui::session::ColorSlot;

use crate::ui::timeline_widget::CanvasRow;

const FPS: f64 = 30.0;

pub(super) const LABEL_PALETTE: [&str; 12] = [
    "#d96b6b", "#d9985a", "#d9c95a", "#a3d95a", "#5ad98c", "#5ac6c6", "#5a96d9", "#7d7dd9",
    "#a86bd9", "#d96bbc", "#9a9a9a", "#cba97a",
];

#[derive(Clone)]
pub(super) struct LayerRow {
    /// カメラの行は層ではないので None。**単一のカメラを層と同じ形で触らせる**
    /// (裁定217: AE のカメラレイヤー相当は document 所有のカメラ)。
    pub layer: Option<LayerId>,
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

/// カメラが持つ値。`center`/`zoom`/`roll` は普通のトランスフォームの語彙で足りる。
const CAMERA_PROPS: &[&str] = &[
    property::CAMERA_CENTER,
    property::CAMERA_ZOOM,
    property::CAMERA_ROLL,
];

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
    /// カメラの行が開いているか。カメラは層ではないので別に持つ。
    camera_open: bool,
    /// カメラに錠。**掛けると破線の枠を掴めなくなる**(誤って掴むのを止める)。
    camera_locked: bool,
    keyed_only: bool,
    /// AE の P / S / R / T / A: 展開した層にその属性の行だけを出す。もう一度押せば戻る。
    reveal: Option<&'static str>,
}

fn timeline_view() -> &'static std::sync::Mutex<TimelineView> {
    static V: std::sync::OnceLock<std::sync::Mutex<TimelineView>> = std::sync::OnceLock::new();
    V.get_or_init(|| std::sync::Mutex::new(TimelineView::default()))
}

/// カメラの値も**普通のトランスフォームの語彙**で呼ぶ。中心はカメラの居場所、
/// 拡大は倍率、傾きは回転。層と別の言葉を作らない。
fn camera_word(name: &str) -> &str {
    match name {
        property::CAMERA_CENTER => "position",
        property::CAMERA_ZOOM => "scale",
        property::CAMERA_ROLL => "rotation",
        other => other,
    }
}

pub(crate) fn camera_locked() -> bool {
    timeline_view().lock().unwrap().camera_locked
}

pub(super) fn toggle_camera_locked() {
    let mut v = timeline_view().lock().unwrap();
    v.camera_locked = !v.camera_locked;
}

/// カメラの行の開閉。カメラは層ではないので `expanded` の集合に入らない。
pub(super) fn toggle_camera_open() {
    let mut v = timeline_view().lock().unwrap();
    v.camera_open = !v.camera_open;
}

pub(super) fn toggle_expanded(layer: LayerId) {
    bump_view_stamp();
    let mut v = timeline_view().lock().unwrap();
    if !v.expanded.remove(&layer) {
        v.expanded.insert(layer);
    }
}

pub(super) fn expand(layer: LayerId) {
    bump_view_stamp();
    timeline_view().lock().unwrap().expanded.insert(layer);
}

/// 行の並びを変える窓側の状態(展開・絞り・reveal)の世代。行の memo の鍵に入れる。
static VIEW_STAMP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(super) fn view_stamp() -> u64 {
    VIEW_STAMP.load(std::sync::atomic::Ordering::Relaxed)
}

fn bump_view_stamp() {
    VIEW_STAMP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub(super) fn toggle_keyed_only() {
    bump_view_stamp();
    let mut v = timeline_view().lock().unwrap();
    v.keyed_only = !v.keyed_only;
}

pub(super) fn keyed_only() -> bool {
    timeline_view().lock().unwrap().keyed_only
}

/// 属性を 1 つだけ出す(同じ鍵でもう一度押せば全部に戻る)。
pub(super) fn toggle_reveal(property: &'static str) {
    bump_view_stamp();
    let mut v = timeline_view().lock().unwrap();
    v.reveal = if v.reveal == Some(property) { None } else { Some(property) };
}

pub(super) fn reveal() -> Option<&'static str> {
    timeline_view().lock().unwrap().reveal
}

/// 画面に出る行の並び。左の名前列と右の帯は必ずこれを通す(ずれると別物になる)。
struct Row {
    layer: Option<LayerId>,
    prop: Option<PropertyId>,
    depth: u16,
    children: u16,
}

fn rows_of(doc: &Document) -> Vec<Row> {
    let (expanded, keyed_only, reveal) = {
        let v = timeline_view().lock().unwrap();
        (v.expanded.clone(), v.keyed_only, v.reveal)
    };
    rows_nested(doc, &expanded, keyed_only, reveal)
}

fn rows_nested(
    doc: &Document,
    expanded: &std::collections::BTreeSet<LayerId>,
    keyed_only: bool,
    reveal: Option<&str>,
) -> Vec<Row> {
    let view = doc.view();
    let mut layers = view.layers();
    layers.sort_by_key(|l| {
        std::cmp::Reverse(view.meta(*l).ok().flatten().map(|m| m.order).unwrap_or(0))
    });

    let present: std::collections::HashSet<LayerId> = layers.iter().copied().collect();
    let parent_of = |l: LayerId| {
        view.attrs(l)
            .ok()
            .flatten()
            .and_then(|a| a.parent)
            .filter(|p| present.contains(p) && *p != l)
    };

    let mut out = Vec::new();

    // **カメラは1台きり**(裁定217)。層の一番上に、層と同じ形で置く。
    let camera_open = {
        let v = timeline_view().lock().unwrap();
        v.camera_open
    };
    out.push(Row {
        layer: None,
        prop: None,
        depth: 0,
        children: 0,
    });
    if camera_open {
        for name in CAMERA_PROPS {
            if let Ok(property) = PropertyId::camera(name) {
                out.push(Row {
                    layer: None,
                    prop: Some(property),
                    depth: 0,
                    children: 0,
                });
            }
        }
    }

    let mut stack: Vec<(LayerId, u16)> = layers
        .iter()
        .rev()
        .filter(|l| parent_of(**l).is_none())
        .map(|l| (*l, 0))
        .collect();

    while let Some((layer, depth)) = stack.pop() {
        let children = layers
            .iter()
            .filter(|c| parent_of(**c) == Some(layer))
            .count() as u16;
        out.push(Row {
            layer: Some(layer),
            prop: None,
            depth,
            children,
        });
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
            // P / S / R / T / A: その属性の行だけ。
            if reveal.is_some_and(|r| r != *name) {
                continue;
            }
            out.push(Row {
                layer: Some(layer),
                prop: Some(property),
                depth,
                children,
            });
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
    // 帯の秒は作品の fps で割る。30 決め打ちだと 24fps の作品で帯だけ時間軸がずれる。
    let fps = doc
        .view()
        .composition()
        .ok()
        .flatten()
        .map(|c| c.fps.as_f64())
        .unwrap_or(FPS);
    let view = doc.view();
    let agg_props: Vec<PropertyId> = [property::OPACITY, property::POSITION]
        .iter()
        .filter_map(|p| PropertyId::new(p).ok())
        .collect();

    rows_of(doc)
        .into_iter()
        .map(|Row { layer, prop, .. }| {
            // カメラの行(層ではない)。キーは camera_track から引く。
            let Some(layer) = layer else {
                let keys = prop
                    .as_ref()
                    .and_then(|p| view.camera_track(p).ok().flatten())
                    .map(|t| t.keys().iter().map(|k| k.t.as_seconds_f64()).collect())
                    .unwrap_or_default();
                return CanvasRow {
                    is_group: false,
                    keys,
                    span: None,
                    agg: Vec::new(),
                    layer: None,
                    prop,
                    color: crate::ui::tokens::ACCENT,
                };
            };
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
                    // 歌詞の切替(content のキー)も層の行に菱形で出す。2 つ以上ある時だけ(1 つは「本文」)。
                    if let Ok(Some(text)) = view.text_document(layer) {
                        if text.content.keys().len() > 1 {
                            keys.extend(text.content.keys().iter().map(|k| k.t.as_seconds_f64()));
                        }
                    }
                    keys.sort_by(|a, b| a.total_cmp(b));
                    keys.dedup();
                    CanvasRow {
                        is_group: false,
                        keys,
                        span: Some((start as f64 / fps, (start + duration) as f64 / fps)),
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
        .map(
            |Row {
                 layer,
                 prop,
                 depth,
                 children,
             }| {
                let Some(id) = layer else {
                    return LayerRow {
                        layer: None,
                        name: match &prop {
                            Some(p) => camera_word(p.name()).to_string(),
                            None => "Camera".to_string(),
                        },
                        color: "#d8b574",
                        hidden: false,
                        solo: false,
                        locked: camera_locked(),
                        prop: prop.map(|p| camera_word(p.name()).to_string()),
                        expanded: timeline_view().lock().unwrap().camera_open,
                        depth,
                        children,
                    };
                };
                let attrs = view.attrs(id).ok().flatten().unwrap_or_default();
                let color = attrs
                    .label_color
                    .map(|ix| LABEL_PALETTE[ix as usize % LABEL_PALETTE.len()])
                    .unwrap_or("#8c8c8c");
                let expanded = timeline_view().lock().unwrap().expanded.contains(&id);
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
            },
        )
        .collect()
}

pub(super) struct ColorSwatch {
    pub hex: String,
    pub rgba: [u8; 4],
}

/// Inspector の COLOR の行。押すと焦点になり、机の色の引き出しが指す。
pub(super) struct ColorRow {
    pub label: &'static str,
    pub hex: String,
    pub slot: ColorSlot,
}

fn push_swatch(
    seen: &mut std::collections::BTreeSet<[u8; 4]>,
    out: &mut Vec<ColorSwatch>,
    rgba: [u8; 4],
) {
    if seen.insert(rgba) {
        out.push(ColorSwatch {
            hex: hex_of(rgba),
            rgba,
        });
    }
}

fn shape_fill_colors(
    node: &ShapeNode,
    seen: &mut std::collections::BTreeSet<[u8; 4]>,
    out: &mut Vec<ColorSwatch>,
) {
    match node {
        ShapeNode::Leaf(shape) => {
            if let Some(fill) = &shape.fill {
                let colors: Vec<_> = match &fill.brush {
                    crate::doc::vector::Brush::Solid(rgb) => vec![*rgb],
                    crate::doc::vector::Brush::Gradient(gradient) => {
                        gradient.stops.iter().map(|stop| stop.color).collect()
                    }
                };
                for rgb in colors {
                    push_swatch(seen, out, [
                        (rgb.r * 255.0).round() as u8,
                        (rgb.g * 255.0).round() as u8,
                        (rgb.b * 255.0).round() as u8,
                        255,
                    ]);
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

/// 白紙の時のパレット。使われた色が出来たらそちらに譲る。
pub(super) fn default_palette() -> Vec<ColorSwatch> {
    ["#ffffff", "#000000", "#f2f2f2", "#d8b574", "#e35b5b", "#f29b3c", "#f2d43c", "#5ab34a", "#3cb5b5", "#4a7fe3", "#8c6eaa", "#e37fb8"]
        .into_iter()
        .map(|hex| {
            let b = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            ColorSwatch { hex: hex.to_owned(), rgba: [b(1), b(3), b(5), 255] }
        })
        .collect()
}

pub(super) fn used_colors_from_doc(doc: &Document) -> Vec<ColorSwatch> {
    let view = doc.view();
    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for layer in view.layers() {
        if let Ok(Some(text)) = view.text_document(layer) {
            for style in &text.styles {
                let f = style.fill;
                push_swatch(
                    &mut seen,
                    &mut out,
                    [
                        (f[0] * 255.0).round() as u8,
                        (f[1] * 255.0).round() as u8,
                        (f[2] * 255.0).round() as u8,
                        (f[3] * 255.0).round() as u8,
                    ],
                );
                if let Some(s) = style.stroke_color {
                    push_swatch(
                        &mut seen,
                        &mut out,
                        [
                            (s[0] * 255.0).round() as u8,
                            (s[1] * 255.0).round() as u8,
                            (s[2] * 255.0).round() as u8,
                            (s[3] * 255.0).round() as u8,
                        ],
                    );
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
    pub id: crate::doc::store::AssetId,
    pub name: String,
    pub kind: String,
    pub size: Option<u64>,
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

pub(super) fn inspector_data_from_doc(view: &StoreView<'_>, layer: LayerId, at: crate::doc::store::RationalTime) -> InspectorData {
    let catalog = crate::render::engine::known_effects();
    crate::ui::functions::read::inspector_data_from_doc(view, layer, at, &catalog)
}

pub(super) struct PropRow {
    pub label: String,
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
    pub colors: Vec<ColorRow>,
}

pub(super) struct UiData {
    pub layer_rows: Vec<LayerRow>,
}

pub(super) struct Loaded {
    pub doc: Document,
    pub ui: UiData,
    pub duration_sec: f64,
}

/// 机の顔に貼る参考画像。素材と同じ口で入るが、Browser にも Timeline にも出ない。
pub(super) fn reference_images_from_view(
    view: &StoreView,
) -> Vec<(crate::doc::store::AssetId, String, Option<String>)> {
    view.assets()
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.role == crate::doc::store::AssetRole::Reference)
        .map(|a| {
            let uri = a.path_absolute.as_deref().and_then(crate::ui::thumbnail::image_data_uri);
            (a.id, a.name, uri)
        })
        .collect()
}

pub(super) fn asset_rows_from_view(view: &StoreView) -> Vec<AssetRow> {
    view.assets()
        .unwrap_or_default()
        .into_iter()
        .filter(|a| a.role == crate::doc::store::AssetRole::Material)
        .map(|a| AssetRow {
            id: a.id,
            family: asset_family(&a.asset_type),
            preview: a.path_absolute.as_deref().and_then(|path| {
                match asset_family(&a.asset_type) {
                    AssetFamily::TwoD => crate::ui::thumbnail::image_data_uri(path),
                    AssetFamily::Video => crate::ui::thumbnail::video_data_uri(path),
                    _ => None,
                }
            }),
            path: a.path_absolute,
            size: a.size_bytes,
            // 下地の色は中身から引く。並び順で引くと 1 本消した時に全部回る。
            thumb: ["#6f8fb5", "#8f7fb8", "#6fb58a", "#b59a6f"][hash_ix(&a.content_hash)],
            name: a.name,
            kind: a.asset_type,
        })
        .collect()
}

fn hash_ix(hash: &str) -> usize {
    hash.get(..2)
        .and_then(|h| u8::from_str_radix(h, 16).ok())
        .unwrap_or(0) as usize
        % 4
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ImportSummary {
    pub admitted: usize,
    pub total: usize,
    /// 中身が同じ物が既に棚に在った数。黙って消すと「入らない」に見える。
    pub duplicates: usize,
    pub first_failure: Option<String>,
}

impl ImportSummary {
    pub(super) fn notice(&self) -> String {
        let base = self.base_notice();
        match (self.duplicates, base.is_empty()) {
            (0, _) => base,
            (1, true) => "1 file is already in the library".to_owned(),
            (n, true) => format!("{n} files are already in the library"),
            (1, false) => format!("{base} · 1 already in library"),
            (n, false) => format!("{base} · {n} already in library"),
        }
    }

    fn base_notice(&self) -> String {
        // 棚に既に在った物は「入らなかった」ではない。全部で数える時は成功側に置く。
        let total = self.total.saturating_sub(self.duplicates);
        match (self.admitted, total, self.first_failure.as_deref()) {
            (0, 0, _) => String::new(),
            (1, 1, None) => "Imported 1 file".to_owned(),
            (admitted, total, None) if admitted == total => format!("Imported {admitted} files"),
            (0, _, Some(reason)) => format!("Import failed: {reason}"),
            (admitted, total, Some(reason)) => {
                format!("Imported {admitted} of {total} · {reason}")
            }
            (admitted, total, None) => format!("Imported {admitted} of {total}"),
        }
    }
}

/// 読んで指紋を取った後の 1 本。Document を触らないので、別の糸で作れる。
pub(super) enum Prepared {
    Draft(crate::doc::store::AssetDraft),
    Failed(String),
}

/// 指紋(SHA-256)を取る。**Document も窓も要らない** —— 10GB の素材で窓を止めない為に、
/// 取り込みはここを別の糸で回し、棚へ入れる所だけ窓の糸で行う。
pub(super) fn prepare_paths(paths: &[std::path::PathBuf], role: crate::doc::store::AssetRole) -> Vec<Prepared> {
    // フォルダは中身を全部(Finder・Bridge の bin)。隠しファイルは見ない。
    expand_folders(paths)
        .iter()
        .map(|path| match prepare_path(path, role) {
            Ok(draft) => {
                // 札の絵もここ(別の糸)で作る。描画の糸で ffmpeg を待たせない。
                match asset_family(&draft.asset_type) {
                    AssetFamily::TwoD => crate::ui::thumbnail::warm(&path.to_string_lossy(), false),
                    AssetFamily::Video => crate::ui::thumbnail::warm(&path.to_string_lossy(), true),
                    _ => {}
                }
                Prepared::Draft(draft)
            }
            Err(reason) => {
                let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("file");
                Prepared::Failed(format!("{name} — {reason}"))
            }
        })
        .collect()
}

/// 棚へ入れる。中身が同じ物は数えるだけで増やさない。
pub(super) fn admit_prepared(doc: &mut crate::doc::store::Document, prepared: Vec<Prepared>) -> ImportSummary {
    let mut summary = ImportSummary {
        admitted: 0,
        total: prepared.len(),
        duplicates: 0,
        first_failure: None,
    };
    for item in prepared {
        match item {
            Prepared::Draft(draft) => match admit_draft(doc, draft) {
                Ok(true) => summary.admitted += 1,
                Ok(false) => summary.duplicates += 1,
                Err(reason) if summary.first_failure.is_none() => summary.first_failure = Some(reason),
                Err(_) => {}
            },
            Prepared::Failed(reason) if summary.first_failure.is_none() => summary.first_failure = Some(reason),
            Prepared::Failed(_) => {}
        }
    }
    summary
}

pub(super) fn admit_paths(
    doc: &mut crate::doc::store::Document,
    paths: &[std::path::PathBuf],
    role: crate::doc::store::AssetRole,
) -> ImportSummary {
    admit_prepared(doc, prepare_paths(paths, role))
}

fn expand_folders(paths: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let hidden = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('.'));
            if hidden {
                continue;
            }
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    for path in paths {
        if path.is_dir() {
            walk(path, &mut out);
        } else {
            out.push(path.clone());
        }
    }
    out
}

fn prepare_path(
    path: &std::path::Path,
    role: crate::doc::store::AssetRole,
) -> Result<crate::doc::store::AssetDraft, String> {
    let Some(asset_type) = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(crate::render::media::asset_type_for_extension)
    else {
        return Err("unsupported file type".to_owned());
    };
    let reader = std::fs::File::open(path).map_err(|error| format!("cannot read: {error}"))?;
    let fingerprint = crate::doc::store::SourceFingerprintV1::from_reader(reader)
        .map_err(|error| format!("cannot fingerprint: {error}"))?;
    let mut draft =
        crate::doc::store::AssetDraft::from_probed_source(asset_type, &fingerprint, path, None);
    // 参考になれるのは画だけ。机に落とした動画や音は素材として棚へ行く(消えない)。
    draft.role = match (role, asset_family(&draft.asset_type)) {
        (crate::doc::store::AssetRole::Reference, AssetFamily::TwoD) => role,
        _ => crate::doc::store::AssetRole::Material,
    };
    Ok(draft)
}

/// `Ok(true)` で棚に増えた、`Ok(false)` で中身が同じ物が既に在った。
fn admit_draft(
    doc: &mut crate::doc::store::Document,
    draft: crate::doc::store::AssetDraft,
) -> Result<bool, String> {
    let known = doc
        .view()
        .assets()
        .unwrap_or_default()
        .iter()
        .any(|a| a.content_hash == draft.content_hash);
    if known {
        return Ok(false);
    }
    doc.apply(crate::doc::store::Intent::AdmitAsset { draft })
        .map(|_| true)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod library {
    use super::*;

    /// 29.97 は "29.970fps"、秒は as_f64 で割る(num() の 30000 を出していた)。
    #[test]
    fn comp_line_shows_fractional_fps_and_real_seconds() {
        let mut doc = load_fixture().doc;
        let comp = doc.view().composition().unwrap().unwrap();
        let fps = crate::doc::store::Fps::try_new(30000, 1001).unwrap();
        doc.apply(crate::doc::store::Intent::SetComposition(crate::doc::store::Composition { fps, duration_frames: 2997, ..comp })).unwrap();
        let line = comp_line(&doc.view());
        assert!(line.ends_with("29.970fps · 1:40"), "{line}");
    }

    #[test]
    fn folders_expand_and_duplicates_are_counted_not_dropped() {
        let dir = std::env::temp_dir().join(format!("motolii-lib-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("inner")).unwrap();
        let png = image::RgbaImage::from_pixel(2, 2, image::Rgba([1, 2, 3, 255]));
        png.save(dir.join("a.png")).unwrap();
        png.save(dir.join("inner").join("b.png")).unwrap();
        std::fs::write(dir.join(".hidden.png"), b"x").unwrap();
        let mut doc = crate::doc::store::Document::new();
        let summary = admit_paths(&mut doc, &[dir.clone()], crate::doc::store::AssetRole::Material);
        assert_eq!((summary.admitted, summary.duplicates, summary.total), (1, 1, 2), "{summary:?}");
        assert_eq!(summary.notice(), "Imported 1 file · 1 already in library");
        assert_eq!(human_size(12_345_678), "12.3 MB");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
#[test]
fn rejected_imports_keep_a_user_visible_reason() {
    let mut doc = crate::doc::store::Document::new();
    let summary = admit_paths(
        &mut doc,
        &[std::path::PathBuf::from("notes.unsupported")],
        crate::doc::store::AssetRole::Material,
    );
    assert_eq!(summary.admitted, 0);
    assert_eq!(summary.total, 1);
    assert!(summary.notice().contains("unsupported file type"));
    assert!(summary.notice().contains("notes.unsupported"));
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
    if let Some(deg) = std::env::var("MOTOLII_TILT")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
    {
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
                    .map(|property| crate::doc::store::Intent::SetTrack {
                        layer: *l,
                        property,
                        track,
                    })
            })
            .collect();
        match fx.doc.apply_all(intents) {
            Ok(_) => println!(
                "PROBE room=stage verdict=tilt-seeded deg={deg} layers={}",
                layers.len()
            ),
            Err(e) => println!("PROBE room=stage verdict=tilt-seed-error {e}"),
        }
    }
    let view = fx.doc.view();

    let layer_rows = layer_rows_from_doc(&fx.doc);


    drop(view);

    Loaded {
        doc: fx.doc,
        ui: UiData { layer_rows },
        duration_sec: 60.0,
    }
}

/// #stagefoot の 1 行(1920×1080 · 30fps · 1:00)。枠を変えたら描き直すので毎 render 引く。
pub(super) fn comp_line(view: &StoreView) -> String {
    view.composition()
        .ok()
        .flatten()
        .map(|c| {
            let secs = (c.duration_frames as f64 / c.fps.as_f64().max(1e-9)).round() as i64;
            format!("{}×{} · {}fps · {}:{:02}", c.width, c.height, crate::ui::export_sheet::fps_label(c.fps), secs / 60, secs % 60)
        })
        .unwrap_or_default()
}

/// 行の memo 鍵。作品の revision と表示の状態(畳み・鍵だけ)を 1 つの文字に。
pub(super) fn memo_stamp(revision: crate::doc::store::Revision) -> String {
    format!("{:?}/{}", revision, view_stamp())
}
