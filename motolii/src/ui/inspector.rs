use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::ui::fixture::{inspector_data_from_doc, InspectorData, PropRow};
use crate::ui::playback::Clock;
use crate::doc::store::{
    property, BlendMode, ContentKeyframe, Document, Intent, Interp, Keyframe,
    LayerAttrsPatch, LayerId, PropertyId, RationalTime, Value,
};

/// 窓に並べる合成モード。**W3C Compositing の16 mix + `plus`(Add)**で、
/// 順番は語彙の並び(reference/compositing-coverage.tsv と同じ)。
const BLEND_MODES: &[(BlendMode, &str)] = &[
    (BlendMode::Normal, "Normal"),
    (BlendMode::Add, "Add"),
    (BlendMode::Multiply, "Multiply"),
    (BlendMode::Screen, "Screen"),
    (BlendMode::Overlay, "Overlay"),
    (BlendMode::Darken, "Darken"),
    (BlendMode::Lighten, "Lighten"),
    (BlendMode::ColorDodge, "Color Dodge"),
    (BlendMode::ColorBurn, "Color Burn"),
    (BlendMode::HardLight, "Hard Light"),
    (BlendMode::SoftLight, "Soft Light"),
    (BlendMode::Difference, "Difference"),
    (BlendMode::Exclusion, "Exclusion"),
    (BlendMode::Hue, "Hue"),
    (BlendMode::Saturation, "Saturation"),
    (BlendMode::Color, "Color"),
    (BlendMode::Luminosity, "Luminosity"),
];

fn blend_label(mode: BlendMode) -> &'static str {
    BLEND_MODES
        .iter()
        .find(|(m, _)| *m == mode)
        .map(|(_, label)| *label)
        .unwrap_or("Normal")
}

fn write_blend(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    mode: BlendMode,
) -> Result<(), crate::doc::store::StoreError> {
    let patch = LayerAttrsPatch {
        blend_mode: Some(mode),
        ..Default::default()
    };
    doc.lock().unwrap().apply(Intent::SetAttrs { layer, patch })
}

const RANGE_SPAN_PX: f64 = 300.0;

fn increment(property: &str, range: Option<(f64, f64)>) -> f64 {
    if let Some((min, max)) = range {
        return (max - min) / RANGE_SPAN_PX;
    }
    match property {
        p if p == property::SCALE => 0.005,
        p if p == property::ROTATION => 0.5,
        p if p == property::OPACITY => 0.005,
        _ => 1.0, // position / anchor
    }
}

fn nudge(value: &Value, vec2: bool, axis: usize, delta: f64, range: Option<(f64, f64)>) -> Value {
    match (vec2, value) {
        (true, Value::Vec2([x, y])) => {
            let mut v = [*x, *y];
            v[axis] += delta;
            Value::Vec2(v)
        }
        (false, Value::F64(v)) => {
            let v = v + delta;
            Value::F64(match range {
                Some((min, max)) => v.clamp(min, max),
                None => v,
            })
        }
        _ => value.clone(),
    }
}

#[derive(Clone)]
pub(super) struct ValueDrag {
    layer: LayerId,
    property: String,
    vec2: bool,
    axis: usize,
    start_x: f64,
    start_value: Value,
    range: Option<(f64, f64)>,
    last_dx: f64,
}

fn write_key(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut doc = doc.lock().unwrap();
    // **キーが無いなら、値を置くだけ。** 利用者が ◇ を押すまで時間の世界へ
    // 入れない(根底3)。キーが在るなら、いま居る時刻に打つ。
    let Some(mut track) = doc.view().track(layer, &prop).ok().flatten() else {
        return doc.apply(Intent::SetConstant { layer, property: prop, value });
    };
    track.insert(Keyframe { t, value, interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer, property: prop, track })
}

/// 時間の世界を開ける。**今の時刻に1つだけ**キーを立てる。
/// `write_key` は「既に開いている物へ打つ」道なので、開ける時はこちら。
fn open_time(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut track = crate::doc::store::KeyframeTrack::new();
    track.insert(Keyframe { t, value, interp: Interp::Linear, spatial: None });
    doc.lock()
        .unwrap()
        .apply(Intent::SetTrack { layer, property: prop, track })
}

/// 時間の世界を閉じる。キーを捨てて、**今見えている値だけ**を残す。
fn close_time(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
) -> Result<(), crate::doc::store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    doc.lock()
        .unwrap()
        .apply(Intent::SetConstant { layer, property: prop, value })
}

/// エフェクトを層から外す。**param は触れるのに本体を外せない**という
/// Q0b 違反(触れる物は全て編集可能)を閉じる。
fn remove_effect(doc: &Arc<Mutex<Document>>, layer: LayerId, id: u32) {
    let mut d = doc.lock().unwrap();
    let mut effects = d.view().effects(layer).unwrap_or_default();
    effects.retain(|e| e.id.0 != id);
    match d.apply(Intent::SetEffects { layer, effects }) {
        Ok(_) => println!("PROBE room=write verdict=applied RemoveEffect layer={layer:?} id={id}"),
        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
    }
}

fn commit_drag(doc: &Arc<Mutex<Document>>, d: &ValueDrag, t: RationalTime) {
    if let Ok(prop) = PropertyId::new(&d.property) {
        doc.lock().unwrap().clear_transient(d.layer, &prop);
    }
    if d.last_dx == 0.0 {
        return;
    }
    let new_value = nudge(&d.start_value, d.vec2, d.axis, d.last_dx * increment(&d.property, d.range), d.range);
    if let Err(e) = write_key(doc, d.layer, &d.property, new_value, t) {
        println!("PROBE room=write verdict=apply-error {e}");
    }
}

fn write_content(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    t: RationalTime,
    content: String,
) -> Result<(), crate::doc::store::StoreError> {
    let mut doc = doc.lock().unwrap();
    let Some(mut document) = doc.view().text_document(layer)? else {
        return Ok(());
    };
    document.content.insert(ContentKeyframe { t, content });
    doc.apply(Intent::SetTextDocument { layer, document })
}

fn prop_row(
    p: &PropRow,
    layer: LayerId,
    t: RationalTime,
    doc: &Arc<Mutex<Document>>,
    mut drag: Signal<Option<ValueDrag>>,
    mut revision: Signal<u32>,
) -> Element {
    let cells = p.cells.iter().zip(p.dims).enumerate().map(|(i, (c, dim))| {
        let class = if c.is_empty() {
            "v blank"
        } else if dim {
            "v z"
        } else {
            "v"
        };
        let target = match &p.axis[i] {
            Some((property, value)) => Some((property.clone(), value.clone(), false)),
            None if !c.is_empty() && (if p.vec2 { i < 2 } else { i == 2 }) => p
                .property
                .clone()
                .map(|property| (property, p.value.clone(), p.vec2)),
            None => None,
        };
        if let Some((property, start_value, vec2)) = target {
            let range = p.range;
            rsx!(span {
                class: "{class}",
                onmousedown: move |evt| {
                    let x = evt.data().client_coordinates().x;
                    *drag.write() = Some(ValueDrag {
                        layer,
                        property: property.clone(),
                        vec2,
                        axis: i,
                        start_x: x,
                        start_value: start_value.clone(),
                        range,
                        last_dx: 0.0,
                    });
                },
                "{c}"
            })
        } else {
            rsx!(span { class: "{class}", "{c}" })
        }
    });
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    let key_glyph = if p.keyed { "◆" } else { "◇" };
    let key_click = p.property.clone().map(|property| {
        let value = p.value.clone();
        let doc = doc.clone();
        let keyed = p.keyed;
        // ◇ は**時間の世界を開け閉めする一手**。開ける時は今の時刻に1つ立て、
        // 閉じる時はキーを全部捨てて、今の値だけを残す(AE と同じ形)。
        move |_| {
            let done = if keyed {
                close_time(&doc, layer, &property, value.clone())
            } else {
                open_time(&doc, layer, &property, value.clone(), t)
            };
            match done {
                Ok(_) => {
                    println!(
                        "PROBE room=write verdict=key-{} layer={:?} prop={} t={:?}",
                        if keyed { "closed" } else { "opened" },
                        layer,
                        property,
                        t
                    );
                    *revision.write() += 1;
                }
                Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
            }
        }
    });
    rsx!(
        div { class: "prow",
            span { class: "n", "{p.label}" }
            {cells}
            if let Some(on_click) = key_click {
                span { class: "{key_class}", onclick: on_click, "{key_glyph}" }
            } else {
                span { class: "{key_class}", "{key_glyph}" }
            }
        }
    )
}

fn content_row(
    p: &PropRow,
    layer: LayerId,
    t: RationalTime,
    doc: &Arc<Mutex<Document>>,
    mut editing: Signal<Option<String>>,
    mut revision: Signal<u32>,
) -> Element {
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    let key_glyph = if p.keyed { "◆" } else { "◇" };
    if let Some(draft) = editing.read().clone() {
        let doc_commit = doc.clone();
        return rsx!(
            div { class: "prow content-row",
                span { class: "n", "{p.label}" }
                input {
                    class: "v content",
                    value: "{draft}",
                    autofocus: "true",
                    oninput: move |evt| *editing.write() = Some(evt.value()),
                    onkeydown: move |evt| match evt.key() {
                        Key::Enter => {
                            evt.prevent_default();
                            if let Some(text) = editing.write().take() {
                                match write_content(&doc_commit, layer, t, text) {
                                    Ok(_) => {
                                        println!("PROBE room=write verdict=content-commit layer={:?} t={:?}", layer, t);
                                        *revision.write() += 1;
                                    }
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                }
                            }
                        }
                        Key::Escape => {
                            evt.prevent_default();
                            *editing.write() = None;
                        }
                        _ => {}
                    },
                }
                span { class: "{key_class}", "{key_glyph}" }
            }
        );
    }
    let current = p.cells[2].clone();
    rsx!(
        div { class: "prow content-row",
            span { class: "n", "{p.label}" }
            span {
                class: "v content",
                ondoubleclick: move |_| *editing.write() = Some(current.clone()),
                "{p.cells[2]}"
            }
            span { class: "{key_class}", "{key_glyph}" }
        }
    )
}

fn descends_from(view: &crate::doc::store::StoreView<'_>, layer: LayerId, ancestor: LayerId) -> bool {
    let mut cur = Some(layer);
    for _ in 0..64 {
        let Some(l) = cur else { return false };
        if l == ancestor {
            return true;
        }
        cur = view.attrs(l).ok().flatten().and_then(|a| a.parent);
    }
    false
}

fn set_parent(doc: &Arc<Mutex<Document>>, layer: LayerId, parent: Option<LayerId>) {
    let patch = LayerAttrsPatch { parent: Some(parent), ..Default::default() };
    if let Err(e) = doc.lock().unwrap().apply(Intent::SetAttrs { layer, patch }) {
        println!("PROBE room=write verdict=apply-error {e}");
    }
}

pub(super) fn inspector_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    clock: &Clock,
    mut revision: Signal<u32>,
    editing: Signal<Option<String>>,
    drag: Signal<Option<ValueDrag>>,
    mut blend_open: Signal<bool>,
    mut parent_open: Signal<bool>,
    playhead: Signal<f64>,
) -> Element {
    let mut drag = drag;
    let _ = revision(); // Document書き換え後の再描画をここで購読する(値そのものは使わない)
    // 再生位置が動いた時も描き直す。**値は時刻で決まる**ので、
    // ここを購読しないと絵だけ動いて数字が止まる。
    let _ = playhead();

    let empty = InspectorData {
        blend: BlendMode::Normal,
        ident_name: "No selection".to_string(),
        ident_sub: String::new(),
        text: Vec::new(),
        transform: Vec::new(),
        effects: Vec::new(),
        has_effects: false,
        colors: Vec::new(),
    };
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);
    let data = match selection {
        Some(layer) => inspector_data_from_doc(&doc.lock().unwrap().view(), layer, t),
        None => empty,
    };
    let inspector = &data;

    let text_rows = inspector
        .text
        .iter()
        .map(|p| content_row(p, selection.unwrap_or(LayerId(0)), t, doc, editing, revision));
    let transform_rows = inspector
        .transform
        .iter()
        .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, drag, revision));
    let effect_blocks: Vec<_> = inspector
        .effects
        .iter()
        .map(|block| {
            (
                block.id,
                block.plugin_id.clone(),
                block
                    .params
                    .iter()
                    .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, drag, revision))
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let fx_label = if inspector.has_effects { "" } else { "No shared FX" };

    let candidates: Vec<(LayerId, String)> = match selection {
        Some(layer) => {
            let d = doc.lock().unwrap();
            let view = d.view();
            view.layers()
                .into_iter()
                .filter(|l| *l != layer && !descends_from(&view, *l, layer))
                .map(|l| {
                    let name = view.attrs(l).ok().flatten().map(|a| a.name).unwrap_or_default();
                    (l, name)
                })
                .collect()
        }
        None => Vec::new(),
    };
    let parent_label = match selection {
        Some(layer) => {
            let d = doc.lock().unwrap();
            let view = d.view();
            view.attrs(layer)
                .ok()
                .flatten()
                .and_then(|a| a.parent)
                .and_then(|p| view.attrs(p).ok().flatten().map(|a| a.name))
                .unwrap_or_else(|| "なし".to_string())
        }
        None => "None".to_string(),
    };
    let has_children = match selection {
        Some(layer) => {
            let d = doc.lock().unwrap();
            let view = d.view();
            view.layers().into_iter().any(|l| {
                view.attrs(l).ok().flatten().and_then(|a| a.parent) == Some(layer)
            })
        }
        None => false,
    };
    let frozen = match selection {
        Some(layer) => doc
            .lock()
            .unwrap()
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .is_some_and(|a| a.frozen),
        None => false,
    };

    let doc_move = doc.clone();
    let doc_up = doc.clone();
    rsx!(
        div {
            id: "inspector",
            onmousemove: move |evt| {
                if evt.data().held_buttons().is_empty() {
                    if let Some(d) = drag.write().take() {
                        commit_drag(&doc_move, &d, t);
                        *revision.write() += 1;
                    }
                    return;
                }
                let Some(state) = drag.write().as_mut().map(|d| {
                    let x = evt.data().client_coordinates().x;
                    let dx = x - d.start_x;
                    let changed = dx != d.last_dx;
                    d.last_dx = dx;
                    (changed, d.layer, d.property.clone(), d.vec2, d.axis, d.start_value.clone(), d.range, dx)
                }) else { return };
                let (changed, layer, property, vec2, axis, start_value, range, dx) = state;
                if !changed {
                    return;
                }
                let new_value = nudge(&start_value, vec2, axis, dx * increment(&property, range), range);
                if let Ok(prop) = PropertyId::new(&property) {
                    doc_move.lock().unwrap().set_transient(layer, prop, new_value.clone());
                    println!(
                        "PROBE room=write verdict=value-scrub layer={:?} prop={} axis={} dx={:.1} new={:?}",
                        layer, property, axis, dx, new_value
                    );
                    *revision.write() += 1;
                }
            },
            onmouseup: move |_| {
                if let Some(d) = drag.write().take() {
                    commit_drag(&doc_up, &d, t);
                    *revision.write() += 1;
                }
            },
            div { class: "ident",
                div {
                    b { "{inspector.ident_name}" }
                    span { class: "sub", "{inspector.ident_sub}" }
                }
                div { class: "sp",
                    span { class: "glyph", "M" }
                    span { class: "glyph on", "S" }
                }
            }
            div { class: "cols",
                span { class: "pn", "Property" }
                span { "X" }
                span { "Y" }
                span { "Z" }
                span { class: "k", "Key" }
            }
            div { class: "sec", "TRANSFORM" }
            {transform_rows}
            div { class: "iscroll",
            if let Some(layer) = selection {
                div { class: "sec", "BLEND" }
                div { class: "prow",
                    span { class: "n", "mode" }
                    span {
                        class: "v content",
                        onclick: move |_| {
                            let open = *blend_open.read();
                            *blend_open.write() = !open;
                        },
                        "{blend_label(inspector.blend)}"
                    }
                    span { class: "glyph", "◇" }
                }
                if blend_open() {
                    for (mode , label) in BLEND_MODES.iter().copied() {
                        div {
                            class: if mode == inspector.blend { "prow blend-pick on" } else { "prow blend-pick" },
                            onclick: {
                                let doc = doc.clone();
                                move |_| {
                                    match write_blend(&doc, layer, mode) {
                                        Ok(_) => {
                                            println!(
                                                "PROBE room=write verdict=applied SetAttrs blend={label} layer={layer:?}"
                                            );
                                            *revision.write() += 1;
                                        }
                                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                    }
                                    *blend_open.write() = false;
                                }
                            },
                            span { class: "n", "" }
                            span { class: "v content", "{label}" }
                        }
                    }
                }
            }
            if let Some(layer) = selection {
                div { class: "sec", "PARENT" }
                div { class: "prow",
                    span { class: "n", "Parent" }
                    span {
                        class: "v content",
                        onclick: move |_| {
                            let open = *parent_open.read();
                            *parent_open.write() = !open;
                        },
                        "{parent_label}"
                    }
                    span { class: "glyph", "◇" }
                }
                if parent_open() {
                    div {
                        class: "prow blend-pick",
                        onclick: {
                            let doc = doc.clone();
                            move |_| {
                                set_parent(&doc, layer, None);
                                *parent_open.write() = false;
                                *revision.write() += 1;
                            }
                        },
                        span { class: "n", "" }
                        span { class: "v content", "None" }
                    }
                    for (candidate , name) in candidates.iter().cloned() {
                        div {
                            class: "prow blend-pick",
                            onclick: {
                                let doc = doc.clone();
                                move |_| {
                                    set_parent(&doc, layer, Some(candidate));
                                    *parent_open.write() = false;
                                    *revision.write() += 1;
                                }
                            },
                            span { class: "n", "" }
                            span { class: "v content", "{name}" }
                        }
                    }
                }
                if has_children {
                    div {
                        class: "prow",
                        onclick: {
                            let doc = doc.clone();
                            move |_| {
                                let intent = if frozen {
                                    Intent::Unfreeze { group: layer }
                                } else {
                                    Intent::Freeze { group: layer }
                                };
                                match doc.lock().unwrap().apply(intent) {
                                    Ok(_) => *revision.write() += 1,
                                    Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                                }
                            }
                        },
                        span { class: "n", "Group" }
                        span { class: "v content", if frozen { "Unfreeze" } else { "Freeze" } }
                    }
                }
            }
            if !inspector.text.is_empty() {
                div { class: "sec", "TEXT" }
                {text_rows}
            }
            if !inspector.colors.is_empty() {
                div { class: "sec", "COLOR" }
                for (label , hex) in inspector.colors.iter() {
                    div { class: "prow",
                        span { class: "n", "{label}" }
                        span { class: "v swatch",
                            span { class: "dot", style: "background:{hex};" }
                            "{hex}"
                        }
                        span { class: "v blank", "" }
                        span { class: "v blank", "" }
                        span { class: "glyph", "◇" }
                    }
                }
            }
            div { class: "sec", "EFFECTS" }
            if inspector.effects.is_empty() {
                div { class: "prow",
                    span { class: "n empty", "{fx_label}" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { "" }
                }
            } else if let Some(layer) = selection {
                for (id , plugin_id , rows) in effect_blocks.into_iter() {
                    div { class: "prow fxhead",
                        span { class: "n", "{plugin_id}" }
                        span {
                            class: "v fxdrop",
                            onclick: {
                                let doc = doc.clone();
                                move |_| {
                                    remove_effect(&doc, layer, id);
                                    *revision.write() += 1;
                                }
                            },
                            "×"
                        }
                    }
                    {rows.into_iter()}
                }
            }
            }
            div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
        }
    )
}
