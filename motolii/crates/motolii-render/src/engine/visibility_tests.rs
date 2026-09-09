use super::environment_tests::file_layer;
use super::reflection_tests::{effect, set};
use super::*;
use crate::doc::store::{
    BlendMode, Document, EffectId, EffectInstance, Intent, LayerAttrsPatch, LayerId, Value,
};

#[test]
#[ignore = "capture visibility diagnosis; requires saved user scene copy"]
fn capture_visibility_diagnosis() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source = std::env::var("MOTOLII_DIAGNOSTIC_SCENE").unwrap();
    let out = std::path::PathBuf::from(std::env::var("MOTOLII_DIAGNOSTIC_DIR").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let mut doc = Document::load(source).unwrap();
    let card = file_layer(
        &mut doc,
        6,
        6,
        &root.join("docs/reviews/assets/2026-09-09-glass-gallery/card.png"),
    );
    doc.apply(Intent::SetTiming {
        layer: card,
        timing: crate::doc::store::LayerTiming::place(0, None, 180),
    })
    .unwrap();
    for (p, v) in [
        ("position", Value::Vec2([1195.0, 810.0])),
        ("anchor", Value::Vec2([256.0, 86.0])),
        ("scale", Value::Vec2([0.78, 0.78])),
    ] {
        set(&mut doc, card, p, v);
    }
    doc.apply(Intent::SetEffects {
        layer: card,
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.glass".into(),
        }],
    })
    .unwrap();
    for (p, v) in [
        ("roughness", 0.12),
        ("transmission", 0.9),
        ("ior", 1.12),
        ("metallic", 0.0),
    ] {
        effect(&mut doc, card, 0, p, Value::F64(v));
    }
    doc.apply(Intent::SetAttrs {
        layer: LayerId(5),
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Normal),
            ..Default::default()
        },
    })
    .unwrap();
    doc.apply(Intent::SetAttrs {
        layer: card,
        patch: LayerAttrsPatch {
            blend_mode: Some(BlendMode::Normal),
            ..Default::default()
        },
    })
    .unwrap();
    let time =
        RationalTime::try_from_frame(17, doc.view().composition().unwrap().unwrap().fps).unwrap();
    let mut engine = Engine::new().unwrap();
    engine.set_reflection_cache_enabled(false);
    engine.set_gpu_instance_sharing_enabled(false);
    engine.compositor.reflection_diagnostic_enabled = true;
    engine.compositor.reflection_diagnostic_near = std::env::var("MOTOLII_DIAGNOSTIC_NEAR")
        .ok()
        .map(|v| v.parse().unwrap());
    let mut rows = Vec::new();
    let mut previous: Option<Vec<u8>> = None;
    let mut ball_input = None;
    for omit in [false, true] {
        engine.compositor.reflection_diagnostic_skip = if omit { ball_input } else { None };
        let positions: Vec<[f64; 2]> = std::env::var("MOTOLII_DIAGNOSTIC_POSITIONS")
            .ok()
            .map(|path| serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap())
            .unwrap_or_else(|| {
                (0..=32)
                    .map(|step| {
                        let u = f64::from(step) / 32.0;
                        [1198.0 + 33.0 * u, 514.7 + 24.0 * u]
                    })
                    .collect()
            });
        for (step, position) in positions.into_iter().enumerate() {
            set(&mut doc, LayerId(5), "position", Value::Vec2(position));
            let frame = engine.render_frame(&doc.view(), time).unwrap();
            if !omit && step == 0 {
                let mut observer_control = Engine::new().unwrap();
                observer_control.set_reflection_cache_enabled(false);
                observer_control.set_gpu_instance_sharing_enabled(false);
                observer_control.compositor.reflection_diagnostic_near =
                    engine.compositor.reflection_diagnostic_near;
                assert_eq!(
                    frame,
                    observer_control.render_frame(&doc.view(), time).unwrap(),
                    "diagnostic capture must not alter final pixels"
                );
            }
            assert!(
                engine.layer_failures().is_empty(),
                "{:?}",
                engine.layer_failures()
            );
            let name = format!("{}-{step:02}", if omit { "omit" } else { "full" });
            image::RgbaImage::from_raw(1600, 1000, frame.clone())
                .unwrap()
                .save(out.join(format!("{name}-frame.png")))
                .unwrap();
            let metadata = engine
                .compositor
                .reflection_diagnostic
                .take()
                .expect("capture metadata")
                .save(&out, &name);
            if ball_input.is_none() {
                ball_input = metadata["inputs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|i| {
                        i.get("model_bounds").is_some()
                            && i["world_from_object"][12]
                                .as_f64()
                                .is_some_and(|x| (x - position[0]).abs() < 2.0)
                    })
                    .and_then(|i| i["index"].as_u64())
                    .map(|x| x as usize);
                assert!(
                    ball_input.is_some(),
                    "identify sphere 2 from its world placement: {metadata}"
                );
            }
            let difference: u64 = previous.as_ref().map_or(0, |p| {
                p.iter()
                    .zip(&frame)
                    .map(|(a, b)| u64::from(a.abs_diff(*b)))
                    .sum()
            });
            rows.push(serde_json::json!({"name":name,"position":position,"omit_ball_from_captures":omit,"rgba_difference":difference}));
            previous = Some(frame);
        }
    }
    std::fs::write(
        out.join("sweep.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
    doc.save(out.join("reconstructed-endpoint.rrd")).unwrap();
}
