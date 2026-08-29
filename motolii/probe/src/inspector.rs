use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::fixture::{inspector_data_from_doc, InspectorData, PropRow};
use crate::playback::Clock;
use motolii_store::{
    property, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerId, PropertyId,
    RationalTime, Value,
};

/// 1pxあたりの値の増分。仮既定 — 実測ベースの調整は次段。
fn increment(property: &str) -> f64 {
    match property {
        p if p == property::SCALE => 0.005,
        p if p == property::ROTATION => 0.5,
        p if p == property::OPACITY => 0.005,
        _ => 1.0, // position / anchor
    }
}

/// axisの成分を`delta`だけ動かした新しい値。vec2でない行はaxisを無視する。
fn nudge(value: &Value, vec2: bool, axis: usize, delta: f64) -> Value {
    match (vec2, value) {
        (true, Value::Vec2([x, y])) => {
            let mut v = [*x, *y];
            v[axis] += delta;
            Value::Vec2(v)
        }
        (false, Value::F64(v)) => Value::F64(v + delta),
        _ => value.clone(),
    }
}

/// 値セルの横drag。押した瞬間の値・位置だけ覚え、離した位置との差でSetTrackを1回書く。
#[derive(Clone)]
struct ValueDrag {
    layer: LayerId,
    property: &'static str,
    vec2: bool,
    axis: usize,
    start_x: f64,
    start_value: Value,
}

/// `SetTrack`本体。trackが無ければ`at_zero`(静的値)、あれば`t`(playhead)へ書く。
fn write_key(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    property: &'static str,
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

fn prop_row(
    p: &PropRow,
    layer: LayerId,
    t: RationalTime,
    doc: &Arc<Mutex<Document>>,
    mut drag: Signal<Option<ValueDrag>>,
    mut tick: Signal<u32>,
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
            let property = p.property.unwrap();
            let vec2 = p.vec2;
            let start_value = p.value.clone();
            let doc_up = doc.clone();
            rsx!(span {
                class: "{class}",
                onmousedown: move |evt| {
                    let x = evt.data().client_coordinates().x;
                    *drag.write() = Some(ValueDrag {
                        layer,
                        property,
                        vec2,
                        axis: i,
                        start_x: x,
                        start_value: start_value.clone(),
                    });
                },
                onmouseup: move |evt| {
                    let Some(d) = drag.write().take() else { return };
                    if d.layer != layer || d.property != property || d.axis != i {
                        return;
                    }
                    let x = evt.data().client_coordinates().x;
                    let dx = x - d.start_x;
                    let new_value = nudge(&d.start_value, d.vec2, d.axis, dx * increment(d.property));
                    match write_key(&doc_up, layer, property, new_value.clone(), t, true) {
                        Ok(_) => {
                            println!(
                                "PROBE room=write verdict=value-scrub layer={:?} prop={} axis={} dx={:.1} new={:?}",
                                layer, property, i, dx, new_value
                            );
                            *tick.write() += 1;
                        }
                        Err(e) => println!("PROBE room=write verdict=apply-error {e}"),
                    }
                },
                "{c}"
            })
        } else {
            rsx!(span { class: "{class}", "{c}" })
        }
    });
    let key_class = if p.keyed { "glyph on" } else { "glyph" };
    let key_glyph = if p.keyed { "◆" } else { "◇" };
    let key_click = p.property.map(|property| {
        let value = p.value.clone();
        let doc = doc.clone();
        move |_| match write_key(&doc, layer, property, value.clone(), t, false) {
            Ok(_) => {
                println!("PROBE room=write verdict=key-added layer={:?} prop={} t={:?}", layer, property, t);
                *tick.write() += 1;
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

pub fn inspector_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    clock: &Clock,
) -> Element {
    let drag = use_signal(|| Option::<ValueDrag>::None);
    let tick = use_signal(|| 0u32);
    let _ = tick(); // apply()後の再描画をここで購読する(値そのものは使わない)

    let empty = InspectorData {
        ident_name: "No selection".to_string(),
        ident_sub: String::new(),
        transform: Vec::new(),
        appearance: Vec::new(),
        has_effects: false,
    };
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000).unwrap_or(RationalTime::ZERO);
    let data = match selection {
        Some(layer) => inspector_data_from_doc(&doc.lock().unwrap().view(), layer, t),
        None => empty,
    };
    let inspector = &data;

    let transform_rows = inspector
        .transform
        .iter()
        .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, drag, tick));
    let appearance_rows = inspector
        .appearance
        .iter()
        .map(|p| prop_row(p, selection.unwrap_or(LayerId(0)), t, doc, drag, tick));
    let fx_label = if inspector.has_effects { "" } else { "No shared FX" };

    rsx!(
        div { id: "inspector",
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
            div { class: "sec", "TRANSFORM" }
            {transform_rows}
            div { class: "sec", "APPEARANCE" }
            {appearance_rows}
            div { class: "sec", "EFFECTS" }
            div { class: "prow",
                span { class: "n empty", "{fx_label}" }
                span { class: "v blank", "" }
                span { class: "v blank", "" }
                span { class: "v blank", "" }
                span { "" }
            }
            div { class: "hint", "Drag to scrub · double-click to type · Esc to cancel" }
        }
    )
}
