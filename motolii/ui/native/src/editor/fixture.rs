use crate::doc::store::{Document,LayerId,PropertyId,ShapeNode,StoreView,Value};
use crate::editor::session::ColorSlot;
use crate::editor::functions::read::{hex_of,asset_family};
pub(crate) const LABEL_PALETTE: [&str; 12] = [
    "#d96b6b", "#d9985a", "#d9c95a", "#a3d95a", "#5ad98c", "#5ac6c6", "#5a96d9", "#7d7dd9",
    "#a86bd9", "#d96bbc", "#9a9a9a", "#cba97a",
];

pub(crate) struct ColorSwatch {
    pub hex: String,
    pub rgba: [u8; 4],
}

/// Inspector の COLOR の行。押すと焦点になり、机の色の引き出しが指す。
pub(crate) struct ColorRow {
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
pub(crate) fn default_palette() -> Vec<ColorSwatch> {
    ["#ffffff", "#000000", "#f2f2f2", "#d8b574", "#e35b5b", "#f29b3c", "#f2d43c", "#5ab34a", "#3cb5b5", "#4a7fe3", "#8c6eaa", "#e37fb8"]
        .into_iter()
        .map(|hex| {
            let b = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0);
            ColorSwatch { hex: hex.to_owned(), rgba: [b(1), b(3), b(5), 255] }
        })
        .collect()
}

pub(crate) fn used_colors_from_doc(doc: &Document) -> Vec<ColorSwatch> {
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum AssetFamily {
    Video,
    TwoD,
    ThreeD,
    Audio,
    Data,
    Other,
}

impl AssetFamily {
    pub(crate) fn label(self) -> &'static str {
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

pub(crate) fn inspector_data_from_doc(view: &StoreView<'_>, layer: LayerId, at: crate::doc::store::RationalTime) -> InspectorData {
    let catalog = crate::render::engine::known_effects();
    crate::editor::functions::read::inspector_data_from_doc(view, layer, at, &catalog)
}

pub(crate) struct PropRow {
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
pub(crate) struct EffectBlock {
    pub id: u32,
    pub plugin_id: String,
    pub params: Vec<PropRow>,
}

pub(crate) struct InspectorData {
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

pub(crate) fn expand_folders(paths: &[std::path::PathBuf]) -> Vec<std::path::PathBuf> {
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
pub(crate) fn prepare_path(
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
pub(crate) fn admit_draft(
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
