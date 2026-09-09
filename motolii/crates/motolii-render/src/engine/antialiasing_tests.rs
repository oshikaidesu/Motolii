use super::environment_tests::{SIZE, scene, sky_png};
use super::reflection_tests::set;
use super::*;
use crate::doc::store::{Document, Intent, LayerId, LayerSource, Value, property};

/// MSAA samples geometry coverage; it must not blur fully covered image interiors.
#[test]
fn upstream_msaa_smooths_mesh_and_rectangle_coverage_without_blurring_interiors() {
    let dir = tempfile::tempdir().unwrap();
    let sky = sky_png(dir.path(), "sky.png", 255, 255);
    let white = dir.path().join("white.png");
    image::RgbaImage::from_pixel(24, 24, image::Rgba([255, 255, 255, 255]))
        .save(&white)
        .unwrap();
    let mut aa = Engine::new().unwrap();
    let mut single = Engine::new().unwrap();
    single.compositor = crate::render::compositor::Compositor::with_device(
        single.gpu_device().clone(),
        single.gpu_queue().clone(),
        crate::render::compositor::PRESENTABLE_FORMAT,
        |_| re_renderer::RenderConfig {
            msaa_mode: re_renderer::MsaaMode::Off,
            ..Default::default()
        },
    )
    .unwrap();
    for rectangle in [false, true] {
        let mut doc = scene(dir.path(), &sky, false);
        doc.apply(Intent::SetAttrs {
            layer: LayerId(1),
            patch: crate::doc::store::LayerAttrsPatch {
                hidden: Some(true),
                ..Default::default()
            },
        })
        .unwrap();
        let layer = LayerId(2);
        if rectangle {
            doc.apply(Intent::SetSource {
                layer,
                source: LayerSource::File {
                    path: white.to_string_lossy().into_owned(),
                    fingerprint: None,
                },
            })
            .unwrap();
        }
        set(
            &mut doc,
            layer,
            "anchor",
            Value::Vec2(if rectangle { [12.0, 12.0] } else { [1.0, 1.0] }),
        );
        set(
            &mut doc,
            layer,
            property::SCALE,
            Value::Vec2(if rectangle { [1.0, 1.0] } else { [12.0, 12.0] }),
        );
        set(&mut doc, layer, "rotation", Value::F64(17.0));
        let before = single
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        let after = aa.render_frame(&doc.view(), RationalTime::ZERO).unwrap();
        assert!(aa.layer_failures().is_empty());
        let peak = before.chunks_exact(4).map(|p| p[0]).max().unwrap();
        assert!(peak > 64);
        let partial = |pixels: &[u8]| {
            pixels
                .chunks_exact(4)
                .filter(|p| p[0] > 4 && p[0] < peak - 4)
                .count()
        };
        assert!(
            partial(&after) > partial(&before) + 8,
            "rectangle={rectangle}: {} -> {} partial pixels",
            partial(&before),
            partial(&after)
        );
        for y in 28..36 {
            for x in 28..36 {
                let i = (y * SIZE as usize + x) * 4;
                assert_eq!(
                    &before[i..i + 4],
                    &after[i..i + 4],
                    "interior remains sharp"
                );
            }
        }
    }
}

#[test]
#[ignore = "paired rendering benchmark; run explicitly without other GPU work"]
fn antialiasing_cost_comparison() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let source = root.join("docs/reviews/assets/2026-09-09-glass-gallery/light-in-form.rrd");
    let mut doc = Document::load(&source).unwrap();
    let ring = doc
        .view()
        .resolved_layers(RationalTime::ZERO)
        .unwrap()
        .into_iter()
        .find(
            |l| matches!(&l.source, LayerSource::File { path, .. } if path.ends_with("torus.obj")),
        )
        .unwrap()
        .id;
    let mut current = Engine::new().unwrap();
    let mut off = Engine::new().unwrap();
    off.compositor = crate::render::compositor::Compositor::with_device(
        off.gpu_device().clone(),
        off.gpu_queue().clone(),
        crate::render::compositor::PRESENTABLE_FORMAT,
        |_| re_renderer::RenderConfig {
            msaa_mode: re_renderer::MsaaMode::Off,
            ..Default::default()
        },
    )
    .unwrap();
    let mut rows = Vec::new();
    for scenario in ["static", "moving"] {
        for frame in 0..35 {
            if scenario == "moving" {
                set(
                    &mut doc,
                    ring,
                    "rotation.y",
                    Value::F64(-24.0 + frame as f64 * 0.3),
                );
            }
            for enabled in if frame % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            } {
                let engine = if enabled { &mut current } else { &mut off };
                let before = engine.surface_work();
                let pixels = engine
                    .render_frame(&doc.view(), RationalTime::ZERO)
                    .unwrap();
                assert_eq!(pixels.len(), 1600 * 1000 * 4);
                assert!(engine.layer_failures().is_empty());
                let m = engine.frame_measurement();
                if frame >= 5 {
                    rows.push(
                        serde_json::json!({"scenario":scenario,"frame":frame-5,"aa":enabled,
                        "total_us":m.total_us,"prepare_us":m.prepare_us,"submit_us":m.submit_us,
                        "wait_us":m.wait_us,"readback_us":m.readback_us,
                        "captures":engine.surface_work().scene_captures-before.scene_captures}),
                    );
                }
            }
        }
    }
    let output = std::env::var("MOTOLII_AA_COST_OUTPUT").expect("benchmark output path");
    std::fs::write(output, serde_json::to_vec_pretty(&serde_json::json!({
        "warmup":5,"samples":30,"resolution":[1600,1000],"comparison":"AA Off vs 4x MSAA with per-sample mesh shading",
        "gpu_timestamps":false,"records":rows
    })).unwrap()).unwrap();
}

#[test]
#[ignore = "paired quality/performance comparison; run without concurrent GPU work"]
fn lightweight_antialiasing_comparison() {
    use re_renderer::{RenderConfig, SurfaceSampling};
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut doc =
        Document::load(root.join("docs/reviews/assets/2026-09-09-glass-gallery/light-in-form.rrd"))
            .unwrap();
    let out = std::path::PathBuf::from(std::env::var("MOTOLII_LIGHTWEIGHT_AA_DIR").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let ring = doc
        .view()
        .resolved_layers(RationalTime::ZERO)
        .unwrap()
        .into_iter()
        .find(
            |l| matches!(&l.source, LayerSource::File { path, .. } if path.ends_with("torus.obj")),
        )
        .unwrap()
        .id;
    let configs = [
        ("persistent_high", SurfaceSampling::Sample, false),
        ("transient_high", SurfaceSampling::Sample, true),
        ("pixel", SurfaceSampling::Pixel, true),
        ("filtered", SurfaceSampling::FilteredPixel, true),
    ]
    .map(|(name, surface_sampling, transient_attachments)| {
        (
            name,
            RenderConfig {
                surface_sampling,
                transient_attachments,
                ..Default::default()
            },
        )
    });
    let mut engines: Vec<_> = configs
        .iter()
        .map(|(_, config)| configured_engine(*config))
        .collect();
    let info = engines[0].gpu_device().adapter_info();
    let mut records = Vec::new();
    for scenario in ["static", "moving"] {
        for frame in 0..35 {
            if scenario == "moving" {
                set(
                    &mut doc,
                    ring,
                    "rotation.y",
                    Value::F64(-24.0 + frame as f64 * 0.3),
                );
            }
            let mut pixels = vec![Vec::new(); configs.len()];
            for j in 0..configs.len() {
                let i = (j + frame) % configs.len();
                let engine = &mut engines[i];
                let before = engine.surface_work();
                pixels[i] = engine
                    .render_frame(&doc.view(), RationalTime::ZERO)
                    .unwrap();
                assert!(engine.layer_failures().is_empty());
                let m = engine.frame_measurement();
                if frame >= 5 {
                    records.push(
                        serde_json::json!({"scenario":scenario,"frame":frame-5,"mode":configs[i].0,
                        "total_us":m.total_us,"prepare_us":m.prepare_us,"submit_us":m.submit_us,
                        "wait_us":m.wait_us,"readback_us":m.readback_us,
                        "captures":engine.surface_work().scene_captures-before.scene_captures}),
                    );
                }
            }
            assert_eq!(
                pixels[0], pixels[1],
                "transient must not change any pixel: {scenario}/{frame}"
            );
            if frame == 5 || frame == 34 {
                for (i, pixels) in pixels.into_iter().enumerate() {
                    image::RgbaImage::from_raw(1600, 1000, pixels)
                        .unwrap()
                        .save(out.join(format!("{}-{scenario}-{frame}.png", configs[i].0)))
                        .unwrap();
                }
            }
        }
    }
    std::fs::write(
        out.join("measurements.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "adapter":info.name,"transient_saves_memory":info.transient_saves_memory,
            "resolution":[1600,1000],"warmup":5,"samples":30,
            "transient_pixel_equality":true,"records":records,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn configured_engine(config: re_renderer::RenderConfig) -> Engine {
    let mut engine = Engine::new().unwrap();
    engine.compositor = crate::render::compositor::Compositor::with_device(
        engine.gpu_device().clone(),
        engine.gpu_queue().clone(),
        crate::render::compositor::PRESENTABLE_FORMAT,
        |_| config,
    )
    .unwrap();
    engine
}

fn reflection_fixture(dir: &std::path::Path) -> Document {
    use super::environment_tests::file_layer;
    use crate::doc::store::{EffectId, EffectInstance};
    let sky = sky_png(dir, "white-sky.png", 255, 255);
    let mut doc = scene(dir, &sky, true);
    let red = dir.join("reflection-source.png");
    image::RgbaImage::from_pixel(64, 64, image::Rgba([255, 0, 0, 255]))
        .save(&red)
        .unwrap();
    let sender = file_layer(&mut doc, 3, 0, &red);
    set(&mut doc, sender, "position", Value::Vec2([-300.0, -300.0]));
    set(&mut doc, sender, "scale", Value::Vec2([10.0, 10.0]));
    set(&mut doc, sender, "position.z", Value::F64(-200.0));
    doc.apply(Intent::SetEffects {
        layer: LayerId(2),
        effects: vec![EffectInstance {
            id: EffectId(0),
            plugin_id: "motolii.glass".into(),
        }],
    })
    .unwrap();
    super::reflection_tests::effect(&mut doc, LayerId(2), 0, "transmission", Value::F64(0.0));
    super::reflection_tests::effect(&mut doc, LayerId(2), 0, "metallic", Value::F64(1.0));
    doc
}

#[test]
fn transient_surface_targets_preserve_pixels_with_and_without_transmission() {
    let dir = tempfile::tempdir().unwrap();
    let mut doc = reflection_fixture(dir.path());
    super::reflection_tests::effect(&mut doc, LayerId(2), 0, "metallic", Value::F64(0.0));
    let mut persistent = configured_engine(re_renderer::RenderConfig {
        transient_attachments: false,
        ..Default::default()
    });
    let mut transient = configured_engine(re_renderer::RenderConfig {
        transient_attachments: true,
        ..Default::default()
    });
    let mut previous = None;
    for transmission in [0.0, 0.8, 0.0] {
        super::reflection_tests::effect(
            &mut doc,
            LayerId(2),
            0,
            "transmission",
            Value::F64(transmission),
        );
        let a = persistent
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        let b = transient
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap();
        assert_eq!(a, b, "transient changes no sample values");
        if let Some(previous) = previous.replace(a.clone()) {
            assert_ne!(previous, a, "transmission must affect the fixture");
        }
        assert!(persistent.layer_failures().is_empty() && transient.layer_failures().is_empty());
    }
}

#[test]
fn filtered_surfaces_refresh_on_edit_undo_camera_and_cache_eviction() {
    let dir = tempfile::tempdir().unwrap();
    let mut doc = reflection_fixture(dir.path());
    let mut engine = configured_engine(re_renderer::RenderConfig {
        surface_sampling: re_renderer::SurfaceSampling::FilteredPixel,
        ..Default::default()
    });
    let first = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert!(
        first
            .chunks_exact(4)
            .any(|p| p[0] as u16 > p[1] as u16 + 20 && p[0] as u16 > p[2] as u16 + 20)
    );
    let again = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_eq!(first, again);
    set(&mut doc, LayerId(3), "opacity", Value::F64(0.25));
    let changed = engine
        .render_frame(&doc.view(), RationalTime::ZERO)
        .unwrap();
    assert_ne!(first, changed);
    assert!(doc.undo());
    engine.clear_reflection_cache();
    assert_eq!(
        first,
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap()
    );
    let camera = crate::doc::core::ResolvedCamera {
        orbit_degrees: [15.0, 20.0],
        ..doc.view().resolve_camera(RationalTime::ZERO).unwrap()
    };
    engine
        .render_with_camera_override(&doc.view(), RationalTime::ZERO, true, Some(camera))
        .unwrap();
    assert_eq!(
        first,
        engine
            .render_frame(&doc.view(), RationalTime::ZERO)
            .unwrap()
    );
    assert!(engine.layer_failures().is_empty());
}
