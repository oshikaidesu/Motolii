use std::sync::{Arc, Mutex};

use crate::doc::store::{
    property, Document, LayerId, PropertyId, RationalTime,
    Value,
};

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
pub(super) fn move_anchor(
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
