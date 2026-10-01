use motolii_edit::{Animate, Document, Intent};
use super::*;
use crate::doc::store::{
    property, Composition, Fps, LayerId, LayerMeta, LayerSource,
    LayerTiming, PropertyId, Value,
};

fn document(path: &std::path::Path) -> Document {
    let mut doc = Document::new().with_programs(crate::extensions::bundled());
    doc.apply(Intent::SetComposition(Composition {
        width: 64,
        height: 64,
        fps: Fps::try_new(30, 1).unwrap(),
        duration_frames: 1,
        background: [0.0, 0.0, 0.0, 1.0],
    }))
    .unwrap();
    let layer = LayerId(1);
    doc.apply_all([
        Intent::AddLayer(layer),
        Intent::SetMeta {
            layer,
            meta: LayerMeta {
                source: LayerSource::File {
                    path: path.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
                order: 0,
                timing: LayerTiming::place(0, None, 1),
            },
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::POSITION).unwrap(),
            value: Value::Vec2([16.0, 16.0]),
        },
        Intent::SetConstant {
            layer,
            property: PropertyId::new(property::SCALE).unwrap(),
            value: Value::Vec2([16.0, 16.0]),
        },
    ])
    .unwrap();
    doc
}

#[test]
fn repeated_spatial_frames_reuse_model_uploads_and_point_arrays() {
    let dir = tempfile::tempdir().unwrap();
    let obj = dir.path().join("triangle.obj");
    let ply = dir.path().join("points.ply");
    std::fs::write(&obj, "v -1 -1 0\nv 1 -1 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    std::fs::write(
        &ply,
        "ply\nformat ascii 1.0\nelement vertex 3\nproperty float x\nproperty float y\nproperty float z\nend_header\n-1 -1 0\n1 -1 0\n0 1 0\n",
    )
    .unwrap();

    let mut engine = Engine::new().unwrap();
    let obj_doc = document(&obj);
    engine
        .render_frame(&obj_doc.view(), RationalTime::ZERO)
        .unwrap();
    let model = engine.models.get(obj.to_str().unwrap()).unwrap().clone();
    engine
        .render_frame(&obj_doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(engine.models.len(), 1);
    assert!(std::sync::Arc::ptr_eq(
        &model,
        engine.models.get(obj.to_str().unwrap()).unwrap()
    ));

    let ply_doc = document(&ply);
    engine
        .render_frame(&ply_doc.view(), RationalTime::ZERO)
        .unwrap();
    let positions = engine
        .point_clouds
        .get(ply.to_str().unwrap())
        .unwrap()
        .positions
        .clone();
    engine
        .render_frame(&ply_doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(engine.point_clouds.len(), 1);
    assert!(std::sync::Arc::ptr_eq(
        &positions,
        &engine
            .point_clouds
            .get(ply.to_str().unwrap())
            .unwrap()
            .positions
    ));
}
