use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::fixture::{inspector_data_from_doc, InspectorData, PropRow};
use crate::playback::Clock;
use motolii_store::{
    property, ContentKeyframe, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerId,
    PropertyId, RationalTime, Value,
};

/// 全幅を横断するドラッグ距離。300pxは手首を大きく振らずに掃ける実用値。
const RANGE_SPAN_PX: f64 = 300.0;

/// 1pxあたりの値の増分。宣言済みrangeがあれば全幅/300pxへ正規化(Transform行はNoneのまま既存表)。
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

/// axisの成分を`delta`だけ動かした新しい値。vec2でない行はaxisを無視する。rangeがあればclamp。
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

/// 値セルの横drag。押した瞬間の値・位置だけ覚え、離した位置との差でSetTrackを1回書く。
#[derive(Clone)]
struct ValueDrag {
    layer: LayerId,
    property: String,
    vec2: bool,
    axis: usize,
    start_x: f64,
    start_value: Value,
    range: Option<(f64, f64)>,
    last_dx: f64,
}

/// `SetTrack`本体。trackが無ければ`at_zero`(静的値)、あれば`t`(playhead)へ書く。
fn write_key(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &str,
    value: Value,
    t: RationalTime,
    at_zero_if_new: bool,
) -> Result<(), motolii_store::StoreError> {
    let Ok(prop) = PropertyId::new(property) else {
        return Ok(());
    };
    let mut doc = doc.lock().unwrap();
    let existing = doc.view().track(layer, &prop).ok().flatten();
    let is_new = existing.as_ref().map(|tr| tr.keys().is_empty()).unwrap_or(true);
    let mut track = existing.unwrap_or_else(KeyframeTrack::new);
    let key_t = if is_new && at_zero_if_new { RationalTime::ZERO } else { t };
    track.insert(Keyframe { t: key_t, value, interp: Interp::Linear, spatial: None });
    doc.apply(Intent::SetTrack { layer, property: prop, track })
}

/// 離された瞬間に1回だけ呼ぶこと — `apply`が1発なので1ドラッグ=1 undo になる。
fn commit_drag(doc: &Arc<Mutex<Document>>, d: &ValueDrag, t: RationalTime) {
    if let Ok(prop) = PropertyId::new(&d.property) {
        doc.lock().unwrap().clear_transient(d.layer, &prop);
    }
    if d.last_dx == 0.0 {
        return;
    }
    let new_value = nudge(&d.start_value, d.vec2, d.axis, d.last_dx * increment(&d.property, d.range), d.range);
    if let Err(e) = write_key(doc, d.layer, &d.property, new_value, t, true) {
        println!("PROBE room=write verdict=apply-error {e}");
    }
}

/// text-layerのcontentだけを差し替える。トラック無し(新規)なら`RationalTime::ZERO`、
/// あればplayheadの`t`(数値スクラブと同じ書き先の法、`write_key`と対称)。
fn write_content(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    t: RationalTime,
    content: String,
) -> Result<(), motolii_store::StoreError> {
    let mut doc = doc.lock().unwrap();
    let Some(mut document) = doc.view().text_document(layer)? else {
        return Ok(());
    };
    let key_t = if document.content.keys().is_empty() { RationalTime::ZERO } else { t };
    document.content.insert(ContentKeyframe { t: key_t, content });
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
        let editable = p.property.is_some()
            && !c.is_empty()
            && if p.vec2 { i < 2 } else { i == 2 };
        if editable {
            let property = p.property.clone().unwrap();
            let vec2 = p.vec2;
            let start_value = p.value.clone();
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
        move |_| match write_key(&doc, layer, &property, value.clone(), t, false) {
            Ok(_) => {
                println!("PROBE room=write verdict=key-added layer={:?} prop={} t={:?}", layer, property, t);
                *revision.write() += 1;
            }
            Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
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

/// 下書き文字列が`editing: Signal`に居るのは、transient overlayが持てるのが
/// `motolii_eval::Value`だけだから。
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

pub fn inspector_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    clock: &Clock,
    mut revision: Signal<u32>,
    editing: Signal<Option<String>>,
) -> Element {
    let mut drag = use_signal(|| Option::<ValueDrag>::None);
    let _ = revision(); // Document書き換え後の再描画をここで購読する(値そのものは使わない)

    let empty = InspectorData {
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
    let effect_rows = inspector
        .effects
        .iter()
        .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, drag, revision));
    let fx_label = if inspector.has_effects { "" } else { "No shared FX" };

    let doc_move = doc.clone();
    let doc_up = doc.clone();
    rsx!(
        div {
            id: "inspector",
            onmousemove: move |evt| {
                // パネルの外で離したmouseupはここに来ない。
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
            div { class: "ptitle",
                span { class: "way", style: "background:var(--way-inspector);" }
                "Inspector"
                em { "solid" }
            }
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
            div { class: "sec", "TRANSFORM" }
            {transform_rows}
            div { class: "sec", "EFFECTS" }
            if inspector.effects.is_empty() {
                div { class: "prow",
                    span { class: "n empty", "{fx_label}" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { class: "v blank", "" }
                    span { "" }
                }
            } else {
                {effect_rows}
            }
            div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
        }
    )
}
