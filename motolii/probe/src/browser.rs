use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use motolii_store::{
    ContentKeyframe, ContentTrack, Document, EffectId, EffectInstance, FontRef, Intent,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PathSource, RationalTime,
    Shape, ShapeNode, TextAlignmentOptions, TextDocument, TextDocumentStyle, TextJustify,
    TextStyleId, VectorPoint,
};
use motolii_vector::{Brush, Fill, FillRule, Rgb};

use crate::fixture::ColorSwatch;

use crate::fixture::{self, LayerRow, UiData};
use crate::playback::Clock;
use crate::timeline_widget::TimelineMsg;

const FPS: f64 = 30.0;

#[derive(Clone, Copy)]
enum NewKind {
    Text,
    Rectangle,
}

fn new_layer_intents(layer: LayerId, order: i16, playhead: i64, duration_frames: i64, kind: NewKind) -> Vec<Intent> {
    let label_color = Some(Some((layer.0 % fixture::LABEL_PALETTE.len() as u64) as u8));
    match kind {
        NewKind::Rectangle => vec![
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
        ],
        NewKind::Text => vec![
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
                            t: RationalTime::try_new(playhead, 30).unwrap_or(RationalTime::ZERO),
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
        ],
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
    let playhead = (clock.now_sec() * FPS) as i64;
    let duration_frames = d.view().composition().ok().flatten().map(|c| c.duration_frames).unwrap_or(1800);
    let intents = new_layer_intents(layer, order, playhead, duration_frames, kind);
    match d.apply_all(intents) {
        Ok(_) => {
            let rows = fixture::layer_rows_from_doc(&d);
            let attrs_vec = rows.iter().map(|r| (r.hidden, r.solo, r.locked)).collect::<Vec<_>>();
            let canvas_rows = fixture::canvas_rows_from_doc(&d);
            drop(d);
            *layer_rows.write() = rows;
            *attrs_state.write() = attrs_vec;
            timeline_tx.send(TimelineMsg::SetRows(canvas_rows)).ok();
            println!("PROBE room=write verdict=created kind={label} layer={}", layer.0);
        }
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
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

/// 選択層の色を書き換える。書き口は素材ごとに違う(裁定済み) —
/// Solidは`SetSource`、Textは`SetTextDocument`のfill、Shapeは`SetShapes`のfill brush。
fn apply_layer_color(doc: &Arc<Mutex<Document>>, layer: LayerId, rgba: [u8; 4], mut revision: Signal<u32>) {
    let mut d = doc.lock().unwrap();
    let source = d.view().meta(layer).ok().flatten().map(|m| m.source);
    let intent = match source {
        Some(LayerSource::Solid { width, height, .. }) => {
            Some(Intent::SetSource { layer, source: LayerSource::Solid { rgba, width, height } })
        }
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

pub fn browser_panel(
    ui: &UiData,
    doc: Arc<Mutex<Document>>,
    clock: Arc<Clock>,
    layer_rows: Signal<Vec<LayerRow>>,
    attrs_state: Signal<Vec<(bool, bool, bool)>>,
    timeline_tx: Sender<TimelineMsg>,
    selected: Signal<Option<LayerId>>,
    revision: Signal<u32>,
) -> Element {
    let mut tab = use_signal(|| 0u8);
    let tab_class = move |n: u8| if tab() == n { "btab on" } else { "btab" };

    let asset_cards = ui.assets.iter().map(|a| {
        rsx!(
            div { class: "tcard",
                div { class: "thumb", style: "background:{a.thumb};" }
                span { class: "tname", "{a.name}" }
                span { class: "tmeta", "{a.kind}" }
            }
        )
    });
    let first_asset = ui.assets.first().map(|a| a.name.clone()).unwrap_or_default();
    let asset_count = ui.assets.len();

    rsx!(
        div { id: "browser",
            div { class: "ptitle",
                span { class: "way", style: "background:var(--way-browser);" }
                "Browser"
                em { "LOCAL LIBRARY" }
            }
            div { class: "btoolbar",
                span { class: "hbtn", "‹" }
                span { class: "hbtn", "›" }
                span { class: "search", "Search files and tags" }
                span { class: "tbtn", "Filters" }
                span { class: "tbtn", "Tags" }
            }
            div { class: "btabs",
                span { class: "{tab_class(0)}", onclick: move |_| tab.set(0), "Media" }
                span { class: "{tab_class(1)}", onclick: move |_| tab.set(1), "Effects" }
                span { class: "{tab_class(2)}", onclick: move |_| tab.set(2), "Create" }
                span { class: "{tab_class(3)}", onclick: move |_| tab.set(3), "Panels" }
                span { class: "{tab_class(4)}", onclick: move |_| tab.set(4), "Colors" }
            }
            if tab() == 4 {
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
                            div {
                                class: "{card_class}",
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
                                if has_swatches {
                                    div { class: "tgrid", {cards} }
                                } else {
                                    div { class: "rcount", "No colors used yet" }
                                }
                            }
                        }
                    )
                }
            } else if tab() == 1 {
                {
                    let layer = selected();
                    let attached: Vec<String> = layer
                        .and_then(|l| doc.lock().unwrap().view().effects(l).ok())
                        .unwrap_or_default()
                        .into_iter()
                        .map(|e| e.plugin_id)
                        .collect();
                    let cards = motolii_engine::known_effects().iter().map(|desc| {
                        let plugin_id = desc.plugin_id.to_owned();
                        let is_on = attached.contains(&plugin_id);
                        let card_class = if is_on { "tcard on" } else if layer.is_none() { "tcard disabled" } else { "tcard" };
                        let onclick = layer.map(|l| {
                            let doc = doc.clone();
                            let plugin_id = plugin_id.clone();
                            move |_| add_effect(&doc, l, &plugin_id, revision)
                        });
                        rsx!(
                            div {
                                class: "{card_class}",
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
            } else if tab() == 2 {
                div { class: "bwork",
                    div { class: "bside",
                        div { class: "sh", "CREATE" }
                        div { class: "srow on", "All" }
                        div { class: "srow", "Text" }
                        div { class: "srow", "Shape" }
                    }
                    div { class: "bresults",
                        div { class: "rhead",
                            div {
                                b { "Create" }
                                span { class: "sub", "Adds a new layer to the composition" }
                            }
                        }
                        div { class: "tgrid",
                            div {
                                class: "tcard",
                                onclick: {
                                    let doc = doc.clone();
                                    let clock = clock.clone();
                                    let timeline_tx = timeline_tx.clone();
                                    move |_| spawn_layer(&doc, &clock, layer_rows, attrs_state, &timeline_tx, NewKind::Text, "text")
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    span { style: "color:#fff; font-size:32px;", "あ" }
                                }
                                span { class: "tname", "Text" }
                                span { class: "tmeta", "text layer" }
                            }
                            div {
                                class: "tcard",
                                onclick: {
                                    let doc = doc.clone();
                                    let clock = clock.clone();
                                    let timeline_tx = timeline_tx.clone();
                                    move |_| spawn_layer(&doc, &clock, layer_rows, attrs_state, &timeline_tx, NewKind::Rectangle, "rectangle")
                                },
                                div { class: "thumb", style: "background:#222; display:flex; align-items:center; justify-content:center;",
                                    div { style: "width:40%; height:40%; background:#fff;" }
                                }
                                span { class: "tname", "Rectangle" }
                                span { class: "tmeta", "path shape" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bwork",
                    div { class: "bside",
                        div { class: "sh", "LIBRARY" }
                        div { class: "srow on", "All media" }
                        div { class: "srow", "Video" }
                        div { class: "srow", "Images" }
                        div { class: "srow", "Audio" }
                        div { class: "sh", "PLACES" }
                        div { class: "srow", "Comp 1" }
                    }
                    div { class: "bresults",
                        div { class: "rhead",
                            div {
                                b { "All media" }
                                span { class: "sub", "Library · fixture" }
                            }
                            div { class: "sp",
                                span { class: "glyph", "▦" }
                                span { class: "glyph on", "▤" }
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
                            em { "Edit tags" }
                        }
                    }
                }
            }
        }
    )
}
