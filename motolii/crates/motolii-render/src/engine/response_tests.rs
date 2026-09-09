use super::reflection_tests::set;
use super::*;
use crate::doc::store::{Document, LayerSource, Value};

#[test]
#[ignore = "paired spatial reflection candidate experiment"]
fn gallery_probe_response_comparison() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut doc =
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
    let out = std::path::PathBuf::from(std::env::var("MOTOLII_RESPONSE_DIR").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let modes = ["baseline", "stable_receivers", "scene_anchors"];
    let mut engines: Vec<_> = (0..3)
        .map(|mode| {
            let mut e = Engine::new().unwrap();
            e.compositor.reflection_probe_experiment = mode;
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
