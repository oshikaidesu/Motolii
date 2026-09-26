//! The layer clock reads the times the document always read: the same order shifts and the same Loop fold, frame by frame,
//! for nested groups, keyed Stagger, every Stagger From, From End and every Loop direction.
use motolii_doc::eval::{Interp, Keyframe, KeyframeTrack};
use motolii_doc::store::layout::*;
use motolii_doc::store::*;
use motolii_edit::{blank_project, Document, Intent};

fn add(doc: &mut Document, id: u64, source: LayerSource, parent: Option<LayerId>) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta { layer, meta: LayerMeta { source, order: id as i16, timing: LayerTiming::place(0, None, 300) } },
        Intent::SetAttrs { layer, patch: LayerAttrsPatch { parent: Some(parent), ..Default::default() } },
    ])
    .unwrap();
    layer
}

fn put(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
    doc.apply(Intent::SetConstant { layer, property: PropertyId::new(name).unwrap(), value }).unwrap();
}

fn keyed(doc: &mut Document, layer: LayerId, name: &str, keys: &[(i64, f64)]) {
    let fps = doc.view().composition().unwrap().unwrap().fps;
    let mut track = KeyframeTrack::new();
    for &(f, v) in keys {
        track.insert(Keyframe { t: RationalTime::try_from_frame(f, fps).unwrap(), value: Value::F64(v), interp: Interp::Linear, spatial: Default::default() });
    }
    doc.apply(Intent::SetTrack { layer, property: PropertyId::new(name).unwrap(), track }).unwrap();
}

fn same_times(doc: &Document, what: &str) {
    let view = doc.view();
    let fps = view.composition().unwrap().unwrap().fps;
    for layer in view.layers() {
        let clock = view.layer_clock(layer).unwrap();
        for f in 0..=150 {
            let t = RationalTime::try_from_frame(f, fps).unwrap();
            let ordered = clock.ordered(t).unwrap();
            assert_eq!(ordered, view.layer_time_by_view(layer, t).unwrap(), "{what}: order of {layer:?} at {f}");
            assert_eq!(clock.local_from_ordered(ordered).unwrap(), view.looped_time_by_view(layer, ordered).unwrap(), "{what}: loop of {layer:?} at {f}");
        }
    }
}

#[test]
fn the_clock_reads_the_times_the_document_always_read() {
    let mut doc = blank_project();
    let outer = add(&mut doc, 1, LayerSource::Group, None);
    let inner = add(&mut doc, 2, LayerSource::Group, Some(outer));
    for id in 3..6 {
        add(&mut doc, id, LayerSource::Shape, Some(outer));
    }
    for id in 6..10 {
        add(&mut doc, id, LayerSource::Shape, Some(inner));
    }
    same_times(&doc, "no rows");
    put(&mut doc, outer, STAGGER, Value::F64(0.5));
    same_times(&doc, "outer stagger");
    keyed(&mut doc, inner, STAGGER, &[(0, 0.0), (60, 1.2)]);
    same_times(&doc, "nested, keyed stagger");
    for from in 0..4 {
        put(&mut doc, inner, STAGGER_FROM, Value::Enum(from));
        put(&mut doc, outer, FROM_END, Value::Enum(from % 2));
        same_times(&doc, &format!("from {from}"));
    }
    for direction in 0..4 {
        put(&mut doc, LayerId(7), LOOP_DURATION, Value::F64(0.7));
        put(&mut doc, LayerId(7), LOOP_DIRECTION, Value::Enum(direction));
        same_times(&doc, &format!("loop {direction}"));
    }
    keyed(&mut doc, LayerId(8), LOOP_DURATION, &[(0, 0.4), (90, 1.5)]);
    same_times(&doc, "keyed loop under a keyed stagger");
    put(&mut doc, outer, LOOP_DURATION, Value::F64(1.0));
    same_times(&doc, "a group's own loop");
}
