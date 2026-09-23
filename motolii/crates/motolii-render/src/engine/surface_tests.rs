use motolii_edit::{Document, Intent};
use super::environment_tests::{file_layer, scene, sky_png, MESH_X, MESH_Y, SIZE};
use super::*;
use crate::doc::store::{
    property, EffectId, EffectInstance, LayerId, LayerSource,
    PropertyId, Value,
};
use crate::extensions::{placement};

pub(super) fn set(doc: &mut Document, layer: LayerId, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::new(name).unwrap(),
        value,
    })
    .unwrap();
}
pub(super) fn effect(doc: &mut Document, layer: LayerId, id: u32, name: &str, value: Value) {
    doc.apply(Intent::SetConstant {
        layer,
        property: PropertyId::effect_param(EffectId(id), name).unwrap(),
        value,
    })
    .unwrap();
}

/// The capture/draw budget belongs to the scene, not to each Repeater copy.
#[test]
fn repeated_mirrors_batch_and_do_not_copy_the_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut engine = Engine::new().unwrap();
    engine.set_gpu_instance_sharing_enabled(false);
    for count in [10, 100, 1000] {
        let mut doc = scene(dir.path(), &sky, true);
        let mesh = LayerId(2);
        doc.apply(Intent::SetEffects {
            layer: mesh,
            effects: vec![
                EffectInstance {
                    id: EffectId(0),
                    plugin_id: "motolii.glass".into(),
                },
                EffectInstance {
                    id: EffectId(1),
                    plugin_id: placement::REPEAT.into(),
                },
            ],
        })
        .unwrap();
        set(&mut doc, mesh, property::POSITION, Value::Vec2([0.0, 0.0]));
        set(&mut doc, mesh, property::SCALE, Value::Vec2([0.7, 0.7]));
        for (id, name, value) in [
            (0, "transmission", Value::F64(0.0)),
            (0, "metallic", Value::F64(1.0)),
            (1, "count", Value::F64(count as f64)),
            (1, "mode", Value::F64(2.0)),
            (1, "columns", Value::F64(32.0)),
            (1, "position_each", Value::Vec2([2.0, 2.0])),
        ] {
            effect(&mut doc, mesh, id, name, value);
        }
        // The world is made once per document frame: a redraw of the same frame draws only the view.
        let before = engine.surface_work();
        let started = std::time::Instant::now();
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        assert!(
            engine.layer_failures().is_empty(),
            "{:?}",
            engine.layer_failures()
        );
        assert_eq!(
            engine.drawn_layers(),
            count as usize + 1,
            "all requested mesh copies reach rendering"
        );
        let after = engine.surface_work();
        assert_eq!(
            after.main_runs - before.main_runs,
            1,
            "mirror copies need no backdrop barrier"
        );
        assert_eq!(after.backdrop_copies - before.backdrop_copies, 0);
        assert_eq!(
            after.mesh_batches - before.mesh_batches,
            1,
            "one main batch"
        );
        eprintln!("REPEATED_MIRRORS count={count} total_with_readback_us={} main_runs=1 backdrop_copies=0 mesh_batches=1",started.elapsed().as_micros());
        let before = engine.surface_work();
        engine.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = engine.surface_work();
        assert_eq!(after.main_runs - before.main_runs, 1, "the view is drawn again, count={count}");
    }
}

/// Standard glass refracts what is below it (2026-09-23): overlapping copies of one glass layer
/// are one group reading one backdrop; they do not refract each other.
#[test]
fn overlapping_glass_copies_read_one_shared_backdrop() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects {
        layer: mesh,
        effects: vec![
            EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.glass".into(),
            },
            EffectInstance {
                id: EffectId(1),
                plugin_id: placement::REPEAT.into(),
            },
        ],
    })
    .unwrap();
    effect(&mut doc, mesh, 0, "ior", Value::F64(1.0));
    effect(&mut doc, mesh, 0, "roughness", Value::F64(0.0));
    effect(&mut doc, mesh, 1, "count", Value::F64(3.0));
    effect(&mut doc, mesh, 1, "position_each", Value::Vec2([0.0, 0.0]));
    let mut engine = Engine::new().unwrap();
    let a = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert!(
        engine.layer_failures().is_empty(),
        "{:?}",
        engine.layer_failures()
    );
    let first = engine.surface_work();
    assert_eq!(first.backdrop_copies, 1, "three glass copies, one backdrop");
    let b = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    let second = engine.surface_work();
    // The backdrop is the view's picture: each draw of the view copies it again.
    assert_eq!(second.backdrop_copies - first.backdrop_copies, 1);
    assert_eq!(a, b);
    let i = ((MESH_Y * SIZE + MESH_X) * 4) as usize;
    assert!(
        a[i] > 240 && a[i + 1] > 240,
        "unit-index glass preserves the white background: {:?}",
        &a[i..i + 4]
    );
}

#[test]
fn gpu_instance_sharing_matches_rebuilds_and_uploads_each_copy_once() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let mesh = LayerId(2);
    doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
        EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
        EffectInstance { id: EffectId(1), plugin_id: placement::REPEAT.into() },
    ] }).unwrap();
    set(&mut doc, mesh, property::POSITION, Value::Vec2([0.0, 0.0]));
    set(&mut doc, mesh, property::SCALE, Value::Vec2([0.7, 0.7]));
    effect(&mut doc, mesh, 0, "transmission", Value::F64(0.0));
    effect(&mut doc, mesh, 0, "metallic", Value::F64(1.0));
    effect(&mut doc, mesh, 1, "mode", Value::F64(2.0));
    effect(&mut doc, mesh, 1, "columns", Value::F64(32.0));
    effect(&mut doc, mesh, 1, "position_each", Value::Vec2([2.0, 2.0]));
    let mut oracle = Engine::new().unwrap();
    let mut shared = Engine::new().unwrap();
    oracle.set_gpu_instance_sharing_enabled(false);
    for count in [10, 100, 1000] {
        effect(&mut doc, mesh, 1, "count", Value::F64(count as f64));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = shared.surface_work();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(actual, expected, "shared GPU subset count={count}");
        assert_eq!(after.mesh_instances_uploaded - before.mesh_instances_uploaded, count);
        assert_eq!(after.mesh_batches - before.mesh_batches, 1);
        assert_eq!(shared.drawn_layers(), count as usize + 1);
    }
    for opacity in [0.5, 1.0] {
        set(&mut doc, mesh, property::OPACITY, Value::F64(opacity));
        effect(&mut doc, mesh, 1, "count", Value::F64(10.0));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert_eq!(actual, expected, "transparency fallback {opacity}");
    }
    for clipped in [false, true] {
        let mut effects = vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
            EffectInstance { id: EffectId(2), plugin_id: "motolii.turbulent_displace".into() },
        ];
        if clipped { effects.push(EffectInstance { id: EffectId(3), plugin_id: "motolii.clip".into() }); }
        effects.push(EffectInstance { id: EffectId(1), plugin_id: placement::REPEAT.into() });
        doc.apply(Intent::SetEffects { layer: mesh, effects }).unwrap();
        effect(&mut doc, mesh, 2, "amount", Value::F64(0.2));
        effect(&mut doc, mesh, 2, "evolution", Value::F64(0.5));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(expected, actual, "field with clip={clipped}");
        if clipped { assert!(shared.surface_work().mesh_batches - before.mesh_batches > 1); }
    }

}

#[test]
fn gpu_instance_subsets_preserve_rect_mesh_boundaries_and_surface_parameters() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "white.png", 255, 255);
    let mut doc = scene(dir.path(), &sky, true);
    let red = dir.path().join("red.png");
    image::RgbaImage::from_pixel(SIZE, SIZE, image::Rgba([255, 0, 0, 255])).save(&red).unwrap();
    let sender = file_layer(&mut doc, 3, 2, &red);
    set(&mut doc, sender, property::POSITION, Value::Vec2([-300.0, -300.0]));
    set(&mut doc, sender, property::SCALE, Value::Vec2([10.0, 10.0]));
    set(&mut doc, sender, "position.z", Value::F64(-200.0));
    let second = file_layer(&mut doc, 4, 3, &dir.path().join("quad.obj"));
    set(&mut doc, second, property::POSITION, Value::Vec2([35.0, 35.0]));
    set(&mut doc, second, property::SCALE, Value::Vec2([8.0, 8.0]));
    for mesh in [LayerId(2), second] {
        doc.apply(Intent::SetEffects { layer: mesh, effects: vec![
            EffectInstance { id: EffectId(0), plugin_id: "motolii.glass".into() },
        ] }).unwrap();
        effect(&mut doc, mesh, 0, "transmission", Value::F64(0.0));
        effect(&mut doc, mesh, 0, "metallic", Value::F64(1.0));
    }
    let mut oracle = Engine::new().unwrap();
    let mut shared = Engine::new().unwrap();
    oracle.set_gpu_instance_sharing_enabled(false);
    for roughness in [0.0, 0.4, 0.0] {
        effect(&mut doc, second, 0, "roughness", Value::F64(roughness));
        let expected = oracle.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let before = shared.surface_work();
        let actual = shared.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        let after = shared.surface_work();
        assert!(shared.layer_failures().is_empty());
        assert_eq!(expected, actual, "interleaved main runs roughness={roughness}");
        assert_eq!(after.mesh_instances_uploaded - before.mesh_instances_uploaded, 2);
        assert!(after.main_runs - before.main_runs >= 2);
    }
}
