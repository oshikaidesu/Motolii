use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::doc::store::{
    property, ContentKeyframe, ContentTrack, Document, EffectId, EffectInstance, FontRef, Intent,
    Interp, Keyframe, KeyframeTrack, LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming,
    Mask, MaskId, MaskMode, Path, PathSource, PathVertex, PropertyId, RationalTime, Shape,
    ShapeNode, TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify, TextStyleId,
    Value, VectorPoint,
};
use crate::doc::vector::{Brush, Contour, Fill, FillRule, Rgb, Vertex};

use crate::ui::fixture::ColorSwatch;

use crate::ui::dock::Panel;
use crate::ui::fixture::{self, LayerRow};
use crate::ui::playback::Clock;
use crate::ui::color::{wheel_slot, ColorWheel};
use crate::ui::semantic_menu::SemanticButton;
use crate::ui::session::Session;
use crate::ui::timeline_widget::TimelineMsg;

#[derive(Clone)]
enum NewKind {
    Text,
    Rectangle,
    Bezier,
    Media { path: String, name: String },
}

/// 空間素材のファイル座標を comp のピクセルへ橋渡しする初期値。囲む球が画角の
/// 6割に収まる倍率と、正規化した箱の左上が中央配置になる位置を**一度だけ**書く。
/// 以後は利用者の物(キーフレームも打てる)。
/// 3D の素材を画角へ収める。中心を comp の真ん中へ、大きさを画面の6割へ。
fn spatial_fit_intents(layer: LayerId, path: &str, comp: (f64, f64)) -> Vec<Intent> {
    let bounds = if crate::render::media::is_mesh_path(path) {
        crate::render::media::load_mesh_bounds(path).ok()
    } else {
        crate::render::media::load_point_cloud(std::path::Path::new(path))
            .ok()
            .map(|data| data.bounds())
    };
    let Some(bounds) = bounds else {
        return Vec::new();
    };
    let radius = bounds.radius();
    if radius <= 0.0 {
        return Vec::new();
    }
    let fit = (comp.1 * 0.6) / (radius as f64 * 2.0);
    let size = bounds.size_xy();
    // 置いた時の初期値は**素の値**。0秒のキーにすると、利用者が ◇ を
    // 押していないのに時間の世界が開いてしまう。
    let put = |name: &str, value: Value| Intent::SetConstant {
        layer,
        property: PropertyId::new(name).expect("既知の属性"),
        value,
    };
    vec![
        put(property::SCALE, Value::Vec2([fit, fit])),
        put(
            property::POSITION,
            Value::Vec2([
                (comp.0 - size[0] as f64 * fit) * 0.5,
                (comp.1 - size[1] as f64 * fit) * 0.5,
            ]),
        ),
    ]
}

/// 位置を指定せずに生まれた層を、枠の真ん中へ置く。
///
/// 隅(0,0)に置くと、素材が小さいほど画面の角の点になって見つからない。
/// 大きさが分かる物は中心を合わせ、分からない物(文字は組んでみるまで
/// 大きさが決まらない)は左上を真ん中へ置く。
fn center_intents(layer: LayerId, natural: (f64, f64), comp: (f64, f64)) -> Vec<Intent> {
    let Ok(property) = crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION)
    else {
        return Vec::new();
    };
    let value = crate::doc::store::Value::Vec2([
        (comp.0 - natural.0) * 0.5,
        (comp.1 - natural.1) * 0.5,
    ]);
    vec![Intent::SetConstant { layer, property, value }]
}

/// 形の実寸。焼く前でも輪郭から測れる。
fn shape_natural(shapes: &[ShapeNode]) -> (f64, f64) {
    crate::doc::vector::content_bounds(shapes)
        .ok()
        .flatten()
        .map(|b| (b[2] - b[0], b[3] - b[1]))
        .unwrap_or((0.0, 0.0))
}

fn new_layer_intents(
    layer: LayerId,
    order: i16,
    playhead: i64,
    duration_frames: i64,
    fps: crate::doc::store::Fps,
    comp: (f64, f64),
    kind: NewKind,
) -> Vec<Intent> {
    let label_color = Some(Some((layer.0 % fixture::LABEL_PALETTE.len() as u64) as u8));
    match kind {
        NewKind::Media { path, name } => {
            let fit = if crate::render::media::is_point_cloud_path(&path)
                || crate::render::media::is_mesh_path(&path)
            {
                spatial_fit_intents(layer, &path, comp)
            } else {
                let natural = crate::render::media::probe(&path)
                    .map(|i| (i.width as f64, i.height as f64))
                    .unwrap_or((0.0, 0.0));
                center_intents(layer, natural, comp)
            };
            let mut out = vec![
                Intent::AddLayer(layer),
                Intent::SetMeta {
                    layer,
                    meta: LayerMeta {
                        source: LayerSource::File { path, fingerprint: None },
                        order,
                        timing: LayerTiming::place(playhead, None, duration_frames),
                    },
                },
                Intent::SetAttrs {
                    layer,
                    patch: LayerAttrsPatch { name: Some(name), label_color, ..Default::default() },
                },
            ];
            out.extend(fit);
            out
        }
        NewKind::Rectangle => {
            let mut out = vec![
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order,
                    timing: LayerTiming::place(playhead, None, duration_frames),
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { name: Some("Rectangle".to_owned()), label_color, ..Default::default() },
            },
            Intent::SetShapes {
                layer,
                shapes: vec![ShapeNode::Leaf(Shape {
                    source: PathSource::Rectangle { size: VectorPoint { x: 200.0, y: 200.0 } },
                    ops: Vec::new(),
                    fill: Some(Fill {
                        brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }),
                        rule: FillRule::NonZero,
                        opacity: 1.0,
                        hidden: false,
                    }),
                    stroke: None,
                })],
            },
        ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Bezier => {
            let mut out = vec![
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Shape,
                    order,
                    timing: LayerTiming::place(playhead, None, duration_frames),
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { name: Some("Bezier".to_owned()), label_color, ..Default::default() },
            },
            Intent::SetShapes {
                layer,
                shapes: vec![ShapeNode::Leaf(Shape {
                    source: PathSource::Bezier(vec![Contour {
                        closed: false,
                        vertices: vec![
                            Vertex {
                                point: VectorPoint { x: -150.0, y: 0.0 },
                                in_tangent: VectorPoint { x: 0.0, y: 0.0 },
                                out_tangent: VectorPoint { x: 100.0, y: -150.0 },
                            },
                            Vertex {
                                point: VectorPoint { x: 150.0, y: 0.0 },
                                in_tangent: VectorPoint { x: -100.0, y: 150.0 },
                                out_tangent: VectorPoint { x: 0.0, y: 0.0 },
                            },
                        ],
                    }]),
                    ops: Vec::new(),
                    fill: None,
                    stroke: Some(crate::doc::vector::Stroke {
                        brush: Brush::Solid(Rgb { r: 1.0, g: 1.0, b: 1.0 }),
                        width: 6.0,
                        cap: crate::doc::vector::LineCap::Round,
                        join: crate::doc::vector::LineJoin::Round,
                        miter_limit: 4.0,
                        opacity: 1.0,
                        hidden: false,
                        dash: None,
                    }),
                })],
            },
        ];
            let shapes = match out.last() {
                Some(Intent::SetShapes { shapes, .. }) => shapes.clone(),
                _ => Vec::new(),
            };
            out.extend(center_intents(layer, shape_natural(&shapes), comp));
            out
        }
        NewKind::Text => {
            let mut out = vec![
            Intent::AddLayer(layer),
            Intent::SetMeta {
                layer,
                meta: LayerMeta {
                    source: LayerSource::Text,
                    order,
                    timing: LayerTiming::place(playhead, None, duration_frames),
                },
            },
            Intent::SetAttrs {
                layer,
                patch: LayerAttrsPatch { name: Some("Text".to_owned()), label_color, ..Default::default() },
            },
            Intent::SetTextDocument {
                layer,
                document: TextDocument {
                    content: {
                        let mut track = ContentTrack::new();
                        track.insert(ContentKeyframe {
                            t: RationalTime::try_from_frame(playhead, fps)
                                .unwrap_or(RationalTime::ZERO),
                            content: "テキスト".to_owned(),
                        });
                        track
                    },
                    justify: TextJustify::Center,
                    wrap_size: None,
                    styles: vec![TextDocumentStyle {
                        id: TextStyleId(0),
                        font: FontRef {
                            path: "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc".to_owned(),
                            fingerprint: None,
                            family: "Hiragino Sans".to_owned(),
                            style: "W3".to_owned(),
                        },
                        size: 96.0,
                        fill: [1.0, 1.0, 1.0, 1.0],
                        line_height: None,
                        tracking: 0.0,
                        stroke_color: None,
                        stroke_width: 0.0,
                        stroke_over_fill: false,
                        axes: Vec::new(),
                        features: Vec::new(),
                    }],
                    slot_id: None,
                    ranges: Vec::new(),
                    alignment: TextAlignmentOptions::default(),
                    runs: Vec::new(),
                },
            },
        ];
            // 文字は組んでみるまで大きさが決まらない。実寸が要らない形で
            // 真ん中へ置く —— 左上を枠の中心に合わせる。
            out.extend(center_intents(layer, (0.0, 0.0), comp));
            out
        }
    }
}

fn spawn_layer(
    doc: &Arc<Mutex<Document>>,
    clock: &Clock,
    mut layer_rows: Signal<Vec<LayerRow>>,
    mut attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: &Sender<TimelineMsg>,
    kind: NewKind,
    label: &'static str,
    mut revision: Signal<u32>,
) {
    let mut d = doc.lock().unwrap();
    let layer = LayerId(d.view().next_layer_id());
    let order = d
        .view()
        .layers()
        .iter()
        .filter_map(|l| d.view().meta(*l).ok().flatten().map(|m| m.order))
        .max()
        .map(|m| m.saturating_add(1))
        .unwrap_or(0);
    let composition = d.view().composition().ok().flatten();
    let fps = composition
        .as_ref()
        .map(|composition| composition.fps)
        .unwrap_or_else(|| crate::doc::store::Fps::try_new(30, 1).expect("30fps"));
    let playhead = clock.current_frame();
    let duration_frames = composition
        .as_ref()
        .map(|composition| composition.duration_frames)
        .unwrap_or(1800);
    let comp_size = composition
        .as_ref()
        .map(|composition| (composition.width as f64, composition.height as f64))
        .unwrap_or((1920.0, 1080.0));
    let intents = new_layer_intents(
        layer,
        order,
        playhead,
        duration_frames,
        fps,
        comp_size,
        kind,
    );
    match d.apply_all(intents) {
        Ok(_) => {
            let rows = fixture::layer_rows_from_doc(&d);
            let attrs_vec = rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect::<Vec<_>>();
            let canvas_rows = fixture::canvas_rows_from_doc(&d);
            drop(d);
            *layer_rows.write() = rows;
            *attrs_state.write() = attrs_vec;
            timeline_tx.send(TimelineMsg::SetRows(canvas_rows)).ok();
            *revision.write() += 1;
            println!("PROBE room=write verdict=created kind={label} layer={}", layer.0);
        }
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

fn replace_source(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    path: String,
    mut revision: Signal<u32>,
) {
    let mut d = doc.lock().unwrap();
    let source = crate::doc::store::LayerSource::File { path, fingerprint: None };
    let applied = d.apply(Intent::SetSource { layer, source }).is_ok();
    drop(d);
    if applied {
        *revision.write() += 1;
    }
}

fn add_effect(doc: &Arc<Mutex<Document>>, layer: LayerId, plugin_id: &str, mut revision: Signal<u32>) {
    let mut d = doc.lock().unwrap();
    let mut effects = d.view().effects(layer).unwrap_or_default();
    let next_id = effects.iter().map(|e| e.id.0).max().map(|m| m + 1).unwrap_or(0);
    effects.push(EffectInstance { id: EffectId(next_id), plugin_id: plugin_id.to_owned() });
    match d.apply(Intent::SetEffects { layer, effects }) {
        Ok(_) => {
            drop(d);
            *revision.write() += 1;
            println!("PROBE room=write verdict=effect-added layer={} plugin={plugin_id}", layer.0);
        }
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

fn mask_frame(doc: &Document, layer: LayerId) -> Option<[f64; 2]> {
    let view = doc.view();
    let meta = view.meta(layer).ok().flatten()?;
    let comp = view.composition().ok().flatten()?;
    match meta.source {
        LayerSource::Text => Some([comp.width as f64, comp.height as f64]),
        LayerSource::Shape => {
            let shapes = view.shapes(layer).ok()?;
            let (width, height) = shape_natural(&shapes);
            (width > 0.0 && height > 0.0).then_some([width, height])
        }
        LayerSource::File { path, .. }
            if !crate::render::media::is_mesh_path(&path)
                && !crate::render::media::is_point_cloud_path(&path) =>
        {
            let info = crate::render::media::probe(&path).ok()?;
            Some([info.width as f64, info.height as f64])
        }
        LayerSource::File { .. } | LayerSource::Null | LayerSource::Group => None,
    }
}

fn add_rectangle_mask(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    mut revision: Signal<u32>,
) {
    let mut d = doc.lock().unwrap();
    let Some([width, height]) = mask_frame(&d, layer) else {
        println!("PROBE room=write verdict=mask-skip layer={} reason=no-2d-frame", layer.0);
        return;
    };
    let next_id = d
        .view()
        .masks(layer)
        .unwrap_or_default()
        .iter()
        .map(|mask| mask.id.0)
        .max()
        .map(|id| id + 1)
        .unwrap_or(0);
    let x0 = width * 0.2;
    let x1 = width * 0.8;
    let y0 = height * 0.2;
    let y1 = height * 0.8;
    let mut shape = KeyframeTrack::new();
    shape.insert(Keyframe {
        t: RationalTime::ZERO,
        value: Value::Path(Path {
            vertices: [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
                .into_iter()
                .map(|point| PathVertex {
                    point,
                    in_tangent: [0.0, 0.0],
                    out_tangent: [0.0, 0.0],
                })
                .collect(),
            closed: true,
        }),
        interp: Interp::Hold,
        spatial: None,
    });
    match d.apply(Intent::AddMask {
        layer,
        mask: Mask {
            id: MaskId(next_id),
            mode: MaskMode::Add,
            inverted: false,
        },
        shape,
    }) {
        Ok(_) => {
            drop(d);
            *revision.write() += 1;
            println!("PROBE room=write verdict=mask-added layer={} id={next_id}", layer.0);
        }
        Err(error) => println!("PROBE room=write verdict=apply-error {error}"),
    }
}

fn set_shape_fill_color(node: &mut ShapeNode, brush: Brush) {
    match node {
        ShapeNode::Leaf(shape) => {
            if let Some(fill) = shape.fill.as_mut() {
                fill.brush = brush;
            }
        }
        ShapeNode::Group(group) => {
            for child in group.children.iter_mut() {
                set_shape_fill_color(child, brush.clone());
            }
        }
    }
}

fn apply_layer_color(doc: &Arc<Mutex<Document>>, layer: LayerId, rgba: [u8; 4], mut revision: Signal<u32>) {
    let mut d = doc.lock().unwrap();
    let source = d.view().meta(layer).ok().flatten().map(|m| m.source);
    let intent = match source {
        Some(LayerSource::Text) => d.view().text_document(layer).ok().flatten().map(|mut document| {
            let fill = [rgba[0] as f64 / 255.0, rgba[1] as f64 / 255.0, rgba[2] as f64 / 255.0, rgba[3] as f64 / 255.0];
            for style in document.styles.iter_mut() {
                style.fill = fill;
            }
            Intent::SetTextDocument { layer, document }
        }),
        Some(LayerSource::Shape) => {
            let mut shapes = d.view().shapes(layer).unwrap_or_default();
            if shapes.is_empty() {
                None
            } else {
                let brush = Brush::Solid(Rgb { r: rgba[0] as f64 / 255.0, g: rgba[1] as f64 / 255.0, b: rgba[2] as f64 / 255.0 });
                for node in shapes.iter_mut() {
                    set_shape_fill_color(node, brush.clone());
                }
                Some(Intent::SetShapes { layer, shapes })
            }
        }
        _ => None,
    };
    let Some(intent) = intent else {
        println!("PROBE room=write verdict=color-skip layer={} reason=unsupported-source", layer.0);
        return;
    };
    match d.apply(intent) {
        Ok(_) => {
            drop(d);
            *revision.write() += 1;
            println!("PROBE room=write verdict=color-applied layer={}", layer.0);
        }
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

pub(super) fn browser_panel(
    session: &Session,
    doc: Arc<Mutex<Document>>,
    clock: Arc<Clock>,
    layer_rows: Signal<Vec<LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: Sender<TimelineMsg>,
    selected: Signal<Option<LayerId>>,
    revision: Signal<u32>,
    panel: Panel,
    mut rail: Signal<Option<fixture::AssetFamily>>,
) -> Element {
    let rail_class = move |f: Option<fixture::AssetFamily>| {
        if rail() == f {
            "srow on"
        } else {
            "srow"
        }
    };
    // 棚は Document から引き直す。窓へ落ちてきた素材はここにしか現れない。
    let _ = revision();
    let assets = fixture::asset_rows_from_view(&doc.lock().unwrap().view());
    let families = {
        let mut f: Vec<_> = assets.iter().map(|a| a.family).collect();
        f.sort();
        f.dedup();
        f
    };
    let shown: Vec<_> = assets
        .iter()
        .filter(|a| rail().is_none_or(|f| a.family == f))
        .collect();
    let rail_label = rail().map_or("All media", |f| f.label());

    let asset_cards = shown.iter().map(|a| {
        let preview = a.preview.as_deref();
        let place = a.path.clone().map(|path| {
            let name = a.name.clone();
            let doc = doc.clone();
            let clock = clock.clone();
            let timeline_tx = timeline_tx.clone();
            move |evt: Event<MouseData>| {
                if evt.modifiers().alt() {
                    if let Some(layer) = selected() {
                        replace_source(&doc, layer, path.clone(), revision);
                        return;
                    }
                }
                spawn_layer(
                    &doc,
                    &clock,
                    layer_rows,
                    attrs_state,
                    &timeline_tx,
                    NewKind::Media { path: path.clone(), name: name.clone() },
                    "media",
                    revision,
                );
            }
        });
        let disabled = place.is_none();
        rsx!(
            SemanticButton {
                class: "tcard",
                disabled,
                onclick: move |evt| { if let Some(f) = &place { f(evt) } },
                if let Some(src) = preview {
                    img { class: "thumb", src: "{src}" }
                } else {
                    div { class: "thumb", style: "background:{a.thumb};" }
                }
                span { class: "tname", "{a.name}" }
                span { class: "tmeta", "{a.kind}" }
            }
        )
    });
    let first_asset = shown.first().map(|a| a.name.clone()).unwrap_or_default();
    let asset_count = shown.len();

    rsx!(
        div { id: "browser",
            if panel == Panel::Colors {
                {
                    let layer = selected();
                    let swatches = fixture::used_colors_from_doc(&doc.lock().unwrap());
                    let has_swatches = !swatches.is_empty();
                    let cards = swatches.into_iter().map(|ColorSwatch { hex, rgba }| {
                        let card_class = if layer.is_none() { "tcard disabled" } else { "tcard" };
                        let onclick = layer.map(|l| {
                            let doc = doc.clone();
                            move |_| apply_layer_color(&doc, l, rgba, revision)
                        });
                        rsx!(
                            SemanticButton {
                                class: "{card_class}",
                                disabled: layer.is_none(),
                                onclick: move |evt| { if let Some(f) = &onclick { f(evt) } },
                                div { class: "thumb", style: "background:{hex};" }
                                span { class: "tname", "{hex}" }
                            }
                        )
                    });
                    rsx!(
                        div { class: "bwork",
                            div { class: "bside",
                                div { class: "sh", "COLORS" }
                                div { class: "srow on", "Used in this composition" }
                            }
                            div { class: "bresults",
                                div { class: "rhead",
                                    div {
                                        b { "Colors" }
                                        span { class: "sub",
                                            if layer.is_some() { "Click to apply to the selected layer" } else { "Select a layer first" }
                                        }
                                    }
                                }
                                match wheel_slot(session) {
                                    Some(slot) => rsx!(ColorWheel { session: session.clone(), slot, revision }),
                                    None => rsx!(div { class: "rcount", "No color yet · select a layer" }),
                                }
                                if has_swatches {
                                    div { class: "tgrid", {cards} }
                                } else {
                                    div { class: "rcount", "No colors yet" }
                                }
                            }
                        }
                    )
                }
            } else if panel == Panel::Effects {
                {
                    let layer = selected();
                    let attached: Vec<String> = layer
                        .and_then(|l| doc.lock().unwrap().view().effects(l).ok())
                        .unwrap_or_default()
                        .into_iter()
                        .map(|e| e.plugin_id)
                        .collect();
                    let cards = crate::render::engine::known_effects().iter().map(|desc| {
                        let plugin_id = desc.plugin_id.to_owned();
                        let is_on = attached.contains(&plugin_id);
                        let card_class = if is_on { "tcard on" } else if layer.is_none() { "tcard disabled" } else { "tcard" };
                        let onclick = layer.map(|l| {
                            let doc = doc.clone();
                            let plugin_id = plugin_id.clone();
                            move |_| add_effect(&doc, l, &plugin_id, revision)
                        });
                        rsx!(
                            SemanticButton {
                                class: "{card_class}",
                                disabled: layer.is_none(),
                                selected: is_on,
                                onclick: move |evt| { if let Some(f) = &onclick { f(evt) } },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    span { style: "color:#fff; font-size:20px;", "ƒ" }
                                }
                                span { class: "tname", "{plugin_id}" }
                                span { class: "tmeta", if is_on { "attached" } else { "effect" } }
                            }
                        )
                    });
                    rsx!(
                        div { class: "bwork",
                            div { class: "bside",
                                div { class: "sh", "EFFECTS" }
                                div { class: "srow on", "All" }
                            }
                            div { class: "bresults",
                                div { class: "rhead",
                                    div {
                                        b { "Effects" }
                                        span { class: "sub",
                                            if layer.is_some() { "Click to add to the selected layer" } else { "Select a layer first" }
                                        }
                                    }
                                }
                                div { class: "tgrid", {cards} }
                            }
                        }
                    )
                }
            } else if panel == Panel::Create {
                {
                let mask_layer = selected().filter(|layer| {
                    mask_frame(&doc.lock().unwrap(), *layer).is_some()
                });
                let mask_count = mask_layer
                    .and_then(|layer| doc.lock().unwrap().view().masks(layer).ok())
                    .map(|masks| masks.len())
                    .unwrap_or(0);
                rsx!(div { class: "bwork",
                    div { class: "bside",
                        div { class: "sh", "CREATE" }
                        div { class: "srow on", "All" }
                    }
                    div { class: "bresults",
                        div { class: "rhead",
                            div {
                                b { "Create" }
                                span { class: "sub", "Adds a layer or applies a Mask to the selection" }
                            }
                        }
                        div { class: "tgrid",
                            SemanticButton {
                                class: "tcard",
                                onclick: {
                                    let doc = doc.clone();
                                    let clock = clock.clone();
                                    let timeline_tx = timeline_tx.clone();
                                    move |_| spawn_layer(&doc, &clock, layer_rows, attrs_state, &timeline_tx, NewKind::Text, "text", revision)
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    span { style: "color:#fff; font-size:32px;", "T" }
                                }
                                span { class: "tname", "Text" }
                                span { class: "tmeta", "text layer" }
                            }
                            SemanticButton {
                                class: "tcard",
                                onclick: {
                                    let doc = doc.clone();
                                    let clock = clock.clone();
                                    let timeline_tx = timeline_tx.clone();
                                    move |_| spawn_layer(&doc, &clock, layer_rows, attrs_state, &timeline_tx, NewKind::Rectangle, "rectangle", revision)
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    div { style: "width:40%; height:40%; background:#fff;" }
                                }
                                span { class: "tname", "Rectangle" }
                                span { class: "tmeta", "path shape" }
                            }
                            SemanticButton {
                                class: "tcard",
                                onclick: {
                                    let doc = doc.clone();
                                    let clock = clock.clone();
                                    let timeline_tx = timeline_tx.clone();
                                    move |_| spawn_layer(&doc, &clock, layer_rows, attrs_state, &timeline_tx, NewKind::Bezier, "bezier", revision)
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    span { style: "color:#fff; font-size:32px;", "〜" }
                                }
                                span { class: "tname", "Bezier" }
                                span { class: "tmeta", "path shape" }
                            }
                            SemanticButton {
                                class: if mask_count > 0 { "tcard on" } else if mask_layer.is_some() { "tcard" } else { "tcard disabled" },
                                disabled: mask_layer.is_none(),
                                selected: mask_count > 0,
                                onclick: {
                                    let doc = doc.clone();
                                    move |_| {
                                        if let Some(layer) = mask_layer {
                                            add_rectangle_mask(&doc, layer, revision);
                                        }
                                    }
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    div { style: "width:40%; height:40%; border:3px solid #fff;" }
                                }
                                span { class: "tname", "Mask" }
                                span { class: "tmeta", if mask_count > 0 { "{mask_count} attached" } else { "layer mask" } }
                            }
                        }
                    }
                })
                }
            } else {
                div { class: "bwork",
                    div { class: "bside",
                        div { class: "sh", "LIBRARY" }
                        SemanticButton {
                            class: "{rail_class(None)}",
                            selected: rail().is_none(),
                            onclick: move |_| rail.set(None),
                            "All media"
                        }
                        {families.iter().copied().map(|f| rsx!(
                            SemanticButton {
                                class: "{rail_class(Some(f))}",
                                selected: rail() == Some(f),
                                onclick: move |_| rail.set(Some(f)),
                                "{f.label()}"
                            }
                        ))}
                    }
                    div { class: "bresults",
                        div { class: "rhead",
                            div {
                                b { "{rail_label}" }
                                span { class: "sub", "Library" }
                            }
                        }
                        div { class: "rcount",
                            "Results"
                            em { "{asset_count}" }
                        }
                        div { class: "tgrid", {asset_cards} }
                        div { class: "bfoot",
                            span { class: "dot", style: "background:var(--accent);" }
                            "{first_asset}"
                        }
                    }
                }
            }
        }
    )
}

#[cfg(test)]
mod placement {
    use super::*;

    /// 位置を指定せずに生まれた層は、**枠の真ん中に立つ**。
    ///
    /// 隅(0,0)に置くと、素材が小さいほど画面の角の点になって見つからない。
    /// 説明書も最初から「画面の真ん中に立つ」と書いている。
    fn box_of(kind: NewKind, comp: (f64, f64), natural: (f64, f64)) -> (f64, f64) {
        let layer = LayerId(1);
        let intents = new_layer_intents(
            layer,
            0,
            0,
            30,
            crate::doc::store::Fps::try_new(30, 1).unwrap(),
            comp,
            kind,
        );
        let mut doc = Document::new();
        doc.apply_all(intents).unwrap();
        let view = doc.view();
        let property = crate::doc::store::PropertyId::new(crate::doc::store::property::POSITION).unwrap();
        let position = view
            .value_at(layer, &property, crate::doc::store::RationalTime::ZERO)
            .unwrap()
            .and_then(|v| match v {
                crate::doc::store::Value::Vec2([x, y]) => Some((x, y)),
                _ => None,
            })
            .unwrap_or((0.0, 0.0));
        (position.0 + natural.0 * 0.5, position.1 + natural.1 * 0.5)
    }

    #[test]
    fn a_rectangle_is_born_in_the_middle_of_the_frame() {
        let comp = (640.0, 480.0);
        let center = box_of(NewKind::Rectangle, comp, (200.0, 200.0));
        assert!(
            (center.0 - 320.0).abs() < 2.0 && (center.1 - 240.0).abs() < 2.0,
            "四角の中心が枠の真ん中に無い: {center:?}"
        );
    }

    #[test]
    fn an_image_is_born_in_the_middle_of_the_frame() {
        let dir = std::env::temp_dir().join("motolii-placement");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("logo.png");
        image::RgbaImage::from_pixel(64, 48, image::Rgba([255, 0, 0, 255]))
            .save(&path)
            .unwrap();
        let comp = (640.0, 480.0);
        let center = box_of(
            NewKind::Media { path: path.to_str().unwrap().to_owned(), name: "logo".into() },
            comp,
            (64.0, 48.0),
        );
        assert!(
            (center.0 - 320.0).abs() < 2.0 && (center.1 - 240.0).abs() < 2.0,
            "絵の中心が枠の真ん中に無い: {center:?}"
        );
    }

    #[test]
    fn mesh_and_point_cloud_receive_the_same_spatial_fit_contract() {
        let dir = tempfile::tempdir().unwrap();
        let obj = dir.path().join("triangle.obj");
        let ply = dir.path().join("triangle.ply");
        std::fs::write(
            &obj,
            "v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n",
        )
        .unwrap();
        std::fs::write(
            &ply,
            "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nend_header\n-1 -1 0\n1 -1 0\n0 1 0\n",
        )
        .unwrap();

        let values = |path: &std::path::Path| {
            let layer = LayerId(1);
            let mut doc = Document::new();
            doc.apply_all(spatial_fit_intents(
                layer,
                path.to_str().unwrap(),
                (640.0, 480.0),
            ))
            .unwrap();
            let view = doc.view();
            let read = |name| {
                view.value_at(
                    layer,
                    &PropertyId::new(name).unwrap(),
                    RationalTime::ZERO,
                )
                .unwrap()
                .unwrap()
            };
            (read(property::POSITION), read(property::SCALE))
        };

        let mesh = values(&obj);
        let points = values(&ply);
        assert_eq!(mesh, points);
        let (Value::Vec2(position), Value::Vec2(scale)) = mesh else {
            panic!("spatial fit did not write Position and Scale")
        };
        assert!((position[0] + scale[0] - 320.0).abs() < 0.01);
        assert!((position[1] + scale[1] - 240.0).abs() < 0.01);
    }

    #[test]
    fn text_content_starts_on_the_composition_frame() {
        let fps = crate::doc::store::Fps::try_new(24, 1).unwrap();
        let layer = LayerId(1);
        let mut doc = Document::new();
        doc.apply_all(new_layer_intents(
            layer,
            0,
            37,
            240,
            fps,
            (640.0, 480.0),
            NewKind::Text,
        ))
        .unwrap();

        let text = doc.view().text_document(layer).unwrap().unwrap();
        assert_eq!(
            text.content.keys()[0].t.try_to_frame_round(fps).unwrap(),
            37
        );
    }
}
