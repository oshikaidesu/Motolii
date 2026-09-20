use super::*;
use crate::doc::store::{layout, Composition, Fps, Interp, Keyframe, KeyframeTrack, LayerMeta, LayerSource, LayerTiming, PropertyId, Value};
use motolii_edit::{Document, Intent};

/// Width is a layout input, not a paint value: a persistent tree must take the
/// new style, while keeping the same Taffy node alive between frames.
#[test]
fn a_timed_size_updates_the_reused_taffy_node() {
    let mut doc = Document::new();
    let fps = Fps::try_new(30, 1).unwrap();
    let group = LayerId(1);
    doc.apply(Intent::SetComposition(Composition { width: 640, height: 480, fps, duration_frames: 90, background: [0.0; 4] })).unwrap();
    let width = KeyframeTrack::try_from_keys(vec![
        Keyframe { t: RationalTime::ZERO, value: Value::F64(120.0), interp: Interp::Linear, spatial: None },
        Keyframe { t: RationalTime::try_new(1, 1).unwrap(), value: Value::F64(240.0), interp: Interp::Linear, spatial: None },
    ]).unwrap();
    doc.apply_all([
        Intent::AddLayer(group),
        Intent::SetMeta { layer: group, meta: LayerMeta { source: LayerSource::Group, order: 0, timing: LayerTiming::place(0, None, 90) } },
        Intent::SetConstant { layer: group, property: PropertyId::new(layout::DISPLAY).unwrap(), value: Value::Enum(1) },
        Intent::SetConstant { layer: group, property: PropertyId::new(layout::HORIZONTAL_SIZING).unwrap(), value: Value::Enum(2) },
        Intent::SetConstant { layer: group, property: PropertyId::new(layout::VERTICAL_SIZING).unwrap(), value: Value::Enum(2) },
        Intent::SetConstant { layer: group, property: PropertyId::new(layout::HEIGHT).unwrap(), value: Value::F64(70.0) },
        Intent::SetTrack { layer: group, property: PropertyId::new(layout::WIDTH).unwrap(), track: width },
    ]).unwrap();

    let cache = std::rc::Rc::new(FlowCache::default());
    let at0 = RationalTime::ZERO;
    let at1 = RationalTime::try_new(1, 1).unwrap();
    let at2 = RationalTime::try_new(2, 1).unwrap();
    let view0 = doc.view().with_layout_solver(cache.clone());
    crate::picture::resolve::resolved_layers(&view0, at0).unwrap();
    let first = crate::picture::frame::layout_frame(&view0, at0).unwrap();
    let node = cache.roots.borrow()[&group].root.id;
    let view1 = doc.view().with_layout_solver(cache.clone());
    crate::picture::resolve::resolved_layers(&view1, at1).unwrap();
    let second = crate::picture::frame::layout_frame(&view1, at1).unwrap();
    let view2 = doc.view().with_layout_solver(cache.clone());
    crate::picture::resolve::resolved_layers(&view2, at2).unwrap();
    let third = crate::picture::frame::layout_frame(&view2, at2).unwrap();

    assert_eq!(cache.roots.borrow()[&group].root.id, node, "style changes must not rebuild the tree");
    assert_eq!(first.sizes[&group], [120.0, 70.0]);
    assert_eq!(second.sizes[&group], [240.0, 70.0], "a changed width must dirty Taffy rather than freeze the old layout");
    assert_eq!(third.sizes[&group], [240.0, 70.0]);
    assert_eq!(cache.layout_passes.get(), 2, "once the evaluated style is unchanged, do not run Taffy again");
}
