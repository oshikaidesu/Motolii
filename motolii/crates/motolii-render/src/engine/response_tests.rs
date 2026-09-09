use super::reflection_tests::set;
use super::*;
use crate::doc::store::{Document, LayerId, LayerSource, Value};

#[test]
#[ignore = "paired spatial reflection candidate experiment"]
fn gallery_probe_response_comparison() {
    let (mut doc, ball) = gallery_scene();
    let out = std::path::PathBuf::from(std::env::var("MOTOLII_RESPONSE_DIR").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let modes = ["baseline", "stable_receivers", "spatial_influence"];
    let mut engines: Vec<_> = (0..3)
        .map(|mode| {
            let mut e = Engine::new().unwrap();
            e.compositor.reflection_probe_experiment = [0, 1, 3][mode as usize];
            e
        })
        .collect();
    let xs: Vec<i32> = [870, 920, 970, 1020, 1060]
        .into_iter()
        .chain(1080..=1100)
        .chain([1120, 1170])
        .collect();
    let mut reference = std::collections::HashMap::new();
    let mut previous: Vec<Option<Vec<u8>>> = vec![None; 3];
    let mut rows = Vec::new();
    for warmup in 0..2 {
        set(
            &mut doc,
            ball,
            "position",
            Value::Vec2([869.0 + f64::from(warmup), 479.21]),
        );
        for engine in &mut engines {
            engine
                .render_frame(&doc.view(), RationalTime::ZERO)
                .unwrap();
        }
    }
    for (reverse, positions) in [
        (false, xs.clone()),
        (true, xs.iter().rev().copied().collect()),
    ] {
        for (step, x) in positions.into_iter().enumerate() {
            set(
                &mut doc,
                ball,
                "position",
                Value::Vec2([f64::from(x), 479.21]),
            );
            for j in 0..3 {
                let mode = (j + step) % 3;
                let engine = &mut engines[mode];
                let before = engine.surface_work();
                let pixels = engine
                    .render_frame(&doc.view(), RationalTime::ZERO)
                    .unwrap();
                let m = engine.frame_measurement();
                assert!(engine.layer_failures().is_empty());
                let delta: u64 = previous[mode].as_ref().map_or(0, |prev| {
                    prev.iter()
                        .zip(&pixels)
                        .map(|(a, b)| u64::from(a.abs_diff(*b)))
                        .sum()
                });
                rows.push(
                    serde_json::json!({"mode":modes[mode],"reverse":reverse,"x":x,
                    "total_us":m.total_us,"prepare_us":m.prepare_us,"wait_us":m.wait_us,
                    "captures":engine.surface_work().scene_captures-before.scene_captures,
                    "rgba_absolute_difference":delta}),
                );
                if reverse {
                    assert_eq!(
                        reference.remove(&(mode, x)).unwrap(),
                        pixels,
                        "direction-independent mode={} x={x}",
                        modes[mode]
                    );
                } else {
                    reference.insert((mode, x), pixels.clone());
                    if [870, 1088, 1089, 1090, 1091, 1170].contains(&x) {
                        image::RgbaImage::from_raw(1600, 1000, pixels.clone())
                            .unwrap()
                            .save(out.join(format!("{}-{x}.png", modes[mode])))
                            .unwrap();
                    }
                }
                previous[mode] = Some(pixels);
            }
        }
    }
    for (mode, engine) in engines.iter_mut().enumerate() {
        engine.compositor.reflection_entry = None;
        set(&mut doc, ball, "position", Value::Vec2([1090.0, 479.21]));
        let a = engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        let b = image::open(out.join(format!("{}-1090.png", modes[mode])))
            .unwrap()
            .to_rgba8()
            .into_raw();
        assert_eq!(a, b, "direct seek after cache eviction");
    }
    std::fs::write(
        out.join("measurements.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "adapter":engines[0].gpu_device().adapter_info().name,"resolution":[1600,1000],
            "reverse_pixel_equality":true,"direct_seek_pixel_equality":true,"records":rows
        }))
        .unwrap(),
    )
    .unwrap();
}

fn gallery_scene() -> (Document, LayerId) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let doc =
        Document::load(root.join("docs/reviews/assets/2026-09-09-glass-gallery/light-in-form.rrd"))
            .unwrap();
    let ball = doc
        .view()
        .resolved_layers(RationalTime::ZERO)
        .unwrap()
        .into_iter()
        .filter(
            |l| matches!(&l.source, LayerSource::File { path, .. } if path.ends_with("sphere.obj")),
        )
        .last()
        .unwrap()
        .id;
    (doc, ball)
}

#[test]
fn default_reflection_response_has_no_receiver_crossing_pop() {
    let (mut doc, ball) = gallery_scene();
    let mut engine = Engine::new().unwrap();
    let mut frames = Vec::new();
    for x in 1088..=1091 {
        set(
            &mut doc,
            ball,
            "position",
            Value::Vec2([f64::from(x), 479.21]),
        );
        frames.push(
            engine
                .render_frame(&doc.view(), RationalTime::ZERO)
                .unwrap(),
        );
        assert!(engine.layer_failures().is_empty());
    }
    let changes: Vec<u64> = frames
        .windows(2)
        .map(|p| {
            p[0].iter()
                .zip(&p[1])
                .map(|(a, b)| u64::from(a.abs_diff(*b)))
                .sum()
        })
        .collect();
    assert!(
        changes.iter().all(|&d| d > 0),
        "movement must affect the image"
    );
    assert!(
        changes[1] < 2 * changes[0].max(changes[2]),
        "isolated reflection pop: {changes:?}"
    );
    engine.compositor.reflection_entry = None;
    set(&mut doc, ball, "position", Value::Vec2([1089.0, 479.21]));
    assert_eq!(
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap(),
        frames[1]
    );
}

#[test]
fn coincident_receivers_keep_the_same_capture_budget() {
    use super::environment_tests::{SIZE, file_layer, scene, sky_png};
    use crate::doc::store::{EffectId, EffectInstance, Intent};
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 255, 64);
    let mut doc = scene(dir.path(), &sky, true);
    file_layer(&mut doc, 3, 2, &dir.path().join("quad.obj"));
    for layer in [LayerId(2), LayerId(3)] {
        set(&mut doc, layer, "scale", Value::Vec2([12.0, 12.0]));
        doc.apply(Intent::SetEffects {
            layer,
            effects: vec![EffectInstance {
                id: EffectId(0),
                plugin_id: "motolii.glass".into(),
            }],
        })
        .unwrap();
        super::reflection_tests::effect(&mut doc, layer, 0, "transmission", Value::F64(0.0));
    }
    let mut engine = Engine::new().unwrap();
    for dx in [-0.01, 0.0, 0.01] {
        set(
            &mut doc,
            LayerId(3),
            "position",
            Value::Vec2([f64::from(SIZE) / 2.0 + dx, f64::from(SIZE) / 2.0]),
        );
        let before = engine.surface_work();
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        assert!(engine.layer_failures().is_empty());
        assert_eq!(
            engine.surface_work().scene_captures - before.scene_captures,
            12
        );
    }
}
