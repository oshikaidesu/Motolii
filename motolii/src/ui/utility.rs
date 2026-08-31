use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::doc::store::{
    property, Document, Intent, Interp, Keyframe, KeyframeTrack, LayerId, PropertyId, RationalTime,
    Value,
};
use crate::ui::playback::Clock;

fn vec2_at(doc: &Document, layer: LayerId, name: &str, t: RationalTime, fallback: (f64, f64)) -> (f64, f64) {
    let view = doc.view();
    let Ok(prop) = PropertyId::new(name) else {
        return fallback;
    };
    match view.value_at(layer, &prop, t) {
        Ok(Some(Value::Vec2([x, y]))) => (x, y),
        _ => fallback,
    }
}

fn write_vec2(doc: &mut Document, layer: LayerId, name: &str, value: (f64, f64)) {
    let Ok(prop) = PropertyId::new(name) else {
        return;
    };
    let mut track = doc.view().track(layer, &prop).ok().flatten().unwrap_or_else(KeyframeTrack::new);
    let t = track.keys().first().map(|k| k.t).unwrap_or(RationalTime::ZERO);
    track.insert(Keyframe {
        t,
        value: Value::Vec2([value.0, value.1]),
        interp: Interp::Linear,
        spatial: None,
    });
    let _ = doc.apply(Intent::SetTrack { layer, property: prop, track });
}

/// アンカーを箱の中の `(fx, fy)`(0..1)へ移し、位置で打ち消して絵を動かさない。
fn move_anchor(
    doc: &Arc<Mutex<Document>>,
    layer: LayerId,
    size: [f32; 2],
    t: RationalTime,
    fx: f64,
    fy: f64,
) {
    let mut d = doc.lock().unwrap();
    let anchor = vec2_at(&d, layer, property::ANCHOR, t, (0.0, 0.0));
    let position = vec2_at(&d, layer, property::POSITION, t, (0.0, 0.0));
    let scale = vec2_at(&d, layer, property::SCALE, t, (1.0, 1.0));
    let next = (size[0] as f64 * fx, size[1] as f64 * fy);
    let moved = (
        position.0 + scale.0 * (next.0 - anchor.0),
        position.1 + scale.1 * (next.1 - anchor.1),
    );
    write_vec2(&mut d, layer, property::ANCHOR, next);
    write_vec2(&mut d, layer, property::POSITION, moved);
    println!(
        "PROBE room=write verdict=anchor-move layer={layer:?} to=({:.1},{:.1})",
        next.0, next.1
    );
}

/// 3D の素材を平面へ収めるか。既定は収めない(AE と逆。裁定 2026-08-30)。
fn set_flatten(doc: &Arc<Mutex<Document>>, layer: LayerId, flatten: bool) {
    let patch = crate::doc::store::LayerAttrsPatch {
        flatten: Some(flatten),
        ..Default::default()
    };
    if let Err(e) = doc.lock().unwrap().apply(Intent::SetAttrs { layer, patch }) {
        println!("PROBE room=write verdict=apply-error {e}");
    }
}

pub(super) fn utility_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    selected_size: &Arc<Mutex<Option<[f32; 2]>>>,
    gizmo_3d: &Arc<std::sync::atomic::AtomicBool>,
    clock: &Clock,
    mut revision: Signal<u32>,
) -> Element {
    let _ = revision();
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000)
        .unwrap_or(RationalTime::ZERO);
    let size = *selected_size.lock().unwrap();
    let three_d = gizmo_3d.load(Ordering::Relaxed);
    let flattened = selection.is_some_and(|layer| {
        doc.lock()
            .unwrap()
            .view()
            .attrs(layer)
            .ok()
            .flatten()
            .is_some_and(|a| a.flatten)
    });

    // 升の並びそのものが意味なので、言葉は置かない(裁定451)。
    let spots: [(f64, f64); 9] = [
        (0.0, 0.0),
        (0.5, 0.0),
        (1.0, 0.0),
        (0.0, 0.5),
        (0.5, 0.5),
        (1.0, 0.5),
        (0.0, 1.0),
        (0.5, 1.0),
        (1.0, 1.0),
    ];

    rsx!(
        div { id: "inspector",
            div { class: "sec", "ANCHOR" }
            if let (Some(layer), Some(size)) = (selection, size) {
                div { class: "anchorgrid",
                    for (fx , fy) in spots.iter().copied() {
                        span {
                            class: "aspot",
                            onclick: {
                                let doc = doc.clone();
                                move |_| {
                                    move_anchor(&doc, layer, size, t, fx, fy);
                                    *revision.write() += 1;
                                }
                            },
                            span { class: "adot" }
                        }
                    }
                }
            } else if selection.is_none() {
                div { class: "prow", span { class: "n empty", "No selection" } }
            } else {
                // 層は選ばれているが、まだ箱を測れていない(Stage が測る)。
                div { class: "prow", span { class: "n empty", "No box yet" } }
            }
            div { class: "sec", "GIZMO" }
            div { class: "prow",
                span { class: "n", "Grab" }
                span {
                    class: if three_d { "v" } else { "v content" },
                    onclick: {
                        let gizmo_3d = gizmo_3d.clone();
                        move |_| {
                            gizmo_3d.store(false, Ordering::Relaxed);
                            *revision.write() += 1;
                        }
                    },
                    "Plane"
                }
                span {
                    class: if three_d { "v content" } else { "v" },
                    onclick: {
                        let gizmo_3d = gizmo_3d.clone();
                        move |_| {
                            gizmo_3d.store(true, Ordering::Relaxed);
                            *revision.write() += 1;
                        }
                    },
                    "Space"
                }
            }
            if three_d {
                div { class: "prow", span { class: "n empty", "drag = orient · ⌥drag = depth" } }
            }
            div { class: "sec", "3D" }
            if let Some(layer) = selection {
                div { class: "prow",
                    span { class: "n", "Fit into" }
                    span {
                        class: if flattened { "v" } else { "v content" },
                        onclick: {
                            let doc = doc.clone();
                            move |_| {
                                set_flatten(&doc, layer, false);
                                *revision.write() += 1;
                            }
                        },
                        "Space"
                    }
                    span {
                        class: if flattened { "v content" } else { "v" },
                        onclick: {
                            let doc = doc.clone();
                            move |_| {
                                set_flatten(&doc, layer, true);
                                *revision.write() += 1;
                            }
                        },
                        "Plane"
                    }
                }
            } else {
                div { class: "prow", span { class: "n empty", "No selection" } }
            }
        }
    )
}
