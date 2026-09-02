use motolii::doc::store::{
    property, Composition, Document, Fps, Intent, Interp, Keyframe, KeyframeTrack,
    LayerAttrsPatch, LayerId, LayerMeta, LayerSource, LayerTiming, PropertyId, RationalTime, Value,
};

fn document() -> Document {
    let mut doc = Document::new();
    doc.apply(Intent::SetComposition(Composition {
        width: 1920,
        height: 1080,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 300,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    doc
}

fn add_layer(
    doc: &mut Document,
    id: u64,
    source: LayerSource,
    order: i16,
    parent: Option<LayerId>,
) -> LayerId {
    let layer = LayerId(id);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source,
                order,
                timing: LayerTiming::place(0, None, 300),
            },
        },
        Intent::SetAttrs {
            layer,
            patch: LayerAttrsPatch {
                name: Some(format!("Layer {id}")),
                parent: Some(parent),
                ..Default::default()
            },
        },
    ])
    .unwrap();
    layer
}

fn set_position(doc: &mut Document, layer: LayerId, value: [f64; 2]) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::new(property::POSITION).unwrap(),
        value: Value::Vec2(value),
    })
    .unwrap();
}

fn animated_position() -> KeyframeTrack {
    let mut track = KeyframeTrack::new();
    for (frame, value) in [(0, [0.0, 0.0]), (30, [20.0, 10.0])] {
        track.insert(Keyframe {
            t: RationalTime::try_new(frame, 30).unwrap(),
            value: Value::Vec2(value),
            interp: Interp::Linear,
            spatial: None,
        });
    }
    track
}

fn assert_affine_close(a: glam::Affine2, b: glam::Affine2) {
    assert!((a.translation - b.translation).length() < 1e-4, "translation: {a:?} != {b:?}");
    assert!((a.matrix2.x_axis - b.matrix2.x_axis).length() < 1e-4, "x axis: {a:?} != {b:?}");
    assert!((a.matrix2.y_axis - b.matrix2.y_axis).length() < 1e-4, "y axis: {a:?} != {b:?}");
}

#[test]
fn group_is_named_inherits_common_parent_keeps_subtrees_and_is_one_undo() {
    let mut doc = document();
    let parent = add_layer(&mut doc, 1, LayerSource::Group, 20, None);
    let first = add_layer(&mut doc, 2, LayerSource::Shape, 12, Some(parent));
    let nested = add_layer(&mut doc, 3, LayerSource::Shape, 11, Some(first));
    let second = add_layer(&mut doc, 4, LayerSource::Text, 8, Some(parent));
    let before = doc.edit_head();

    let group = doc
        .group_layers(&[first, nested, second, first])
        .unwrap()
        .expect("group");
    assert_eq!(doc.edit_head(), before + 1);
    let view = doc.view();
    assert_eq!(view.attrs(group).unwrap().unwrap().name, "Group");
    assert_eq!(view.attrs(group).unwrap().unwrap().parent, Some(parent));
    assert_eq!(view.meta(group).unwrap().unwrap().order, 12);
    assert_eq!(view.attrs(first).unwrap().unwrap().parent, Some(group));
    assert_eq!(view.attrs(second).unwrap().unwrap().parent, Some(group));
    assert_eq!(view.attrs(nested).unwrap().unwrap().parent, Some(first));
    drop(view);

    assert!(doc.undo());
    let view = doc.view();
    assert!(!view.has_layer(group));
    assert_eq!(view.attrs(first).unwrap().unwrap().parent, Some(parent));
    assert_eq!(view.attrs(second).unwrap().unwrap().parent, Some(parent));
}

#[test]
fn locked_or_frozen_children_reject_atomically() {
    let mut locked_doc = document();
    let locked = add_layer(&mut locked_doc, 1, LayerSource::Shape, 1, None);
    locked_doc
        .apply(Intent::SetAttrs {
            layer: locked,
            patch: LayerAttrsPatch { locked: Some(true), ..Default::default() },
        })
        .unwrap();
    let before = locked_doc.edit_head();
    let layers_before = locked_doc.view().layers();
    assert!(locked_doc.group_layers(&[locked]).is_err());
    assert_eq!(locked_doc.edit_head(), before);
    assert_eq!(locked_doc.view().layers(), layers_before);

    let mut frozen_doc = document();
    let frozen_parent = add_layer(&mut frozen_doc, 1, LayerSource::Group, 2, None);
    let child = add_layer(&mut frozen_doc, 2, LayerSource::Shape, 1, Some(frozen_parent));
    frozen_doc.apply(Intent::Freeze { group: frozen_parent }).unwrap();
    let before = frozen_doc.edit_head();
    let layers_before = frozen_doc.view().layers();
    assert!(frozen_doc.group_layers(&[child]).is_err());
    assert_eq!(frozen_doc.edit_head(), before);
    assert_eq!(frozen_doc.view().layers(), layers_before);
}

#[test]
fn layers_from_different_parents_reject_without_moving_them() {
    let mut doc = document();
    let parent = add_layer(&mut doc, 1, LayerSource::Group, 3, None);
    let nested = add_layer(&mut doc, 2, LayerSource::Shape, 2, Some(parent));
    let root = add_layer(&mut doc, 3, LayerSource::Shape, 1, None);
    let before = doc.edit_head();

    assert!(doc.group_layers(&[nested, root]).is_err());
    assert_eq!(doc.edit_head(), before);
    assert_eq!(doc.view().attrs(nested).unwrap().unwrap().parent, Some(parent));
    assert_eq!(doc.view().attrs(root).unwrap().unwrap().parent, None);
}

#[test]
fn static_ungroup_preserves_world_transform_and_is_one_undo() {
    let mut doc = document();
    let child = add_layer(&mut doc, 1, LayerSource::Shape, 1, None);
    set_position(&mut doc, child, [5.0, 3.0]);
    let group = doc.group_layers(&[child]).unwrap().unwrap();
    set_position(&mut doc, group, [10.0, 20.0]);
    let before_world = doc.view().resolve(child, RationalTime::ZERO).unwrap().unwrap().placement.transform;
    let before = doc.edit_head();

    let released = doc.ungroup_layers(&[group]).unwrap();
    assert_eq!(released, vec![child]);
    assert_eq!(doc.edit_head(), before + 1);
    let after_world = doc.view().resolve(child, RationalTime::ZERO).unwrap().unwrap().placement.transform;
    assert_affine_close(before_world, after_world);

    assert!(doc.undo());
    assert!(doc.view().has_layer(group));
    assert_eq!(doc.view().attrs(child).unwrap().unwrap().parent, Some(group));
}

#[test]
fn animated_group_or_animated_child_is_rejected_without_revision() {
    let mut group_doc = document();
    let child = add_layer(&mut group_doc, 1, LayerSource::Shape, 1, None);
    let group = group_doc.group_layers(&[child]).unwrap().unwrap();
    group_doc
        .apply(Intent::SetTrack {
            layer: group,
            property: PropertyId::new(property::POSITION).unwrap(),
            track: animated_position(),
        })
        .unwrap();
    let before = group_doc.edit_head();
    assert!(group_doc.ungroup_layers(&[group]).is_err());
    assert_eq!(group_doc.edit_head(), before);
    assert!(group_doc.view().has_layer(group));

    let mut child_doc = document();
    let child = add_layer(&mut child_doc, 1, LayerSource::Shape, 1, None);
    child_doc
        .apply(Intent::SetTrack {
            layer: child,
            property: PropertyId::new(property::POSITION).unwrap(),
            track: animated_position(),
        })
        .unwrap();
    let group = child_doc.group_layers(&[child]).unwrap().unwrap();
    set_position(&mut child_doc, group, [10.0, 0.0]);
    let before = child_doc.edit_head();
    assert!(child_doc.ungroup_layers(&[group]).is_err());
    assert_eq!(child_doc.edit_head(), before);
    assert!(child_doc.view().has_layer(group));
}

#[test]
fn nested_selected_groups_ungroup_one_level_only() {
    let mut doc = document();
    let leaf = add_layer(&mut doc, 1, LayerSource::Shape, 1, None);
    let inner = doc.group_layers(&[leaf]).unwrap().unwrap();
    let outer = doc.group_layers(&[inner]).unwrap().unwrap();

    let released = doc.ungroup_layers(&[outer, inner]).unwrap();
    assert_eq!(released, vec![inner]);
    assert!(!doc.view().has_layer(outer));
    assert!(doc.view().has_layer(inner));
    assert_eq!(doc.view().attrs(inner).unwrap().unwrap().parent, None);
    assert_eq!(doc.view().attrs(leaf).unwrap().unwrap().parent, Some(inner));
}
