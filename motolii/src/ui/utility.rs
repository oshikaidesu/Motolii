use std::sync::{Arc, Mutex};

use dioxus_native::prelude::*;

use crate::doc::store::{
    property, Document, LayerId, PropertyId, RationalTime,
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

fn write_vec2(doc: &mut Document, layer: LayerId, name: &str, value: (f64, f64), t: RationalTime) {
    let Ok(prop) = PropertyId::new(name) else {
        return;
    };
    // 打つ時刻は**いま居る時刻**。以前は「最初のキーの時刻」へ打っていて、
    // 触っただけでキーが生え、しかも別の時刻へ落ちていた。
    let intent = doc.place(layer, &prop, Value::Vec2([value.0, value.1]), t);
    let _ = doc.apply(intent);
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
    write_vec2(&mut d, layer, property::ANCHOR, next, t);
    write_vec2(&mut d, layer, property::POSITION, moved, t);
    println!(
        "PROBE room=write verdict=anchor-move layer={layer:?} to=({:.1},{:.1})",
        next.0, next.1
    );
}

pub(super) fn utility_panel(
    doc: &Arc<Mutex<Document>>,
    selection: Option<LayerId>,
    selected_size: &Arc<Mutex<Option<[f32; 2]>>>,
    clock: &Clock,
    mut revision: Signal<u32>,
) -> Element {
    let _ = revision();
    let t = RationalTime::try_new((clock.now_sec() * 3000.0) as i64, 3000)
        .unwrap_or(RationalTime::ZERO);
    let size = *selected_size.lock().unwrap();

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
        }
    )
}
